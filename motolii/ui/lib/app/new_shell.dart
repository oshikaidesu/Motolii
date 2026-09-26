import 'dart:async';
import 'dart:convert';

import 'package:docking/docking.dart';
import 'package:flutter/widgets.dart';

import '../foundation/leaves.dart';
import '../foundation/metrics.dart';
import '../foundation/panel_catalog.dart';
import '../foundation/panel_controls.dart' show EditorScale;
import '../foundation/shell_tokens.dart';
import '../foundation/theme.dart';
import '../input/editor_shortcuts.dart';
import '../panels/composition_controls.dart';
import '../panels/inspector.dart' show InspectorInstruments, InspectorPanel;
import '../panels/export_controls.dart';
import '../panels/registry.dart';
import 'new/desk/desk_faces.dart';
import '../session/editor_session.dart';
import 'editor_actions.dart';
import '../workspace/dock_workspace.dart';
import 'new/browser/shelf_panel.dart';
import 'new/console.dart';
import 'new/dock_theme.dart';
import 'new/inspector/new_effect.dart';
import 'new/inspector/new_layout.dart';
import 'new/inspector/new_transform.dart';
import 'new/shell_bar.dart';
import 'status_notice.dart' show freezeNotice;

/// The New projection of the same session: one face whose panels sit in an
/// off-the-shelf dock (Browser, Stage, Inspector, Timeline, Desk), and the
/// panels Classic docks hosted as they are. The default preset is the layout
/// the design started from; the user can resize, tab, split and move it. Capabilities it cannot reach yet
/// are listed as NOT YET ROUTED in docs/stage5/ui-rebaseline/routing.md.
class NewShell extends StatefulWidget {
  const NewShell({super.key});
  @override
  State<NewShell> createState() => _NewShellState();
}

const _browserTabs = ['Create', 'Effects', 'Media', 'Colors', 'Fonts', 'Files'];
const _centerTabs = ['Stage', 'Camera', 'Notes', 'Web'];
const _browserLabels = {'Create': 'Objects'};
const _resetLayout = 'Reset layout';

class _NewShellState extends State<NewShell> {
  final c = EditorSession();
  late final shortcuts = EditorShortcuts(
    c,
    onMenu: menu,
    hasSheet: () => sheet != null,
    closeSheet: () => setState(() => sheet = null),
    showComposition: () => setState(() => sheet = 'Composition'),
    showInspector: () {},
  );
  final keys = <String, GlobalKey>{};

  /// Rebuilt when the sentence changes, not when a frame plays.
  late final notice = c.slice(
    'notice',
    const [],
    derived: () => freezeNotice(c.state) ?? effectsNotice(c.state),
  );
  late final console = ConsoleLog(
    c,
    notice,
    (doc) => freezeNotice(doc) ?? effectsNotice(doc),
  );
  late final uiScale = EditorScale.of(context) ?? ValueNotifier(1.0);
  String? sheet;
  bool ready = false;

  @override
  void initState() {
    super.initState();
    c
        .slice('animationAppearance', const ['animate'])
        .addListener(_syncAnimation);
    console; // start listening now, so a message before the panel is opened is kept
    c.confirmClose = () => confirmReplacement(context, c);
    c.panelPlacementRequested = _place;
    c.windowClosed = _windowClosed;
    c.filesDropped = (paths) {
      if (paths.isNotEmpty) c.importPaths(paths);
    };
    c.browserTab.addListener(_followBrowserTab);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) EditorAppearance.of(context)?.value = ShellTokens.theme;
    });
    _initialize();
  }

  Future<void> _initialize() async {
    await c.initialize();
    if (!mounted) return;
    if (_panelWindow) {
      // A window that holds only the panels handed to it: no workspace of its own, nothing saved from here.
      try {
        final data = EditorSession.map(await c.native('readSettings'));
        c.restoreDeskWork(EditorSession.map(data['deskWork']));
        uiScale.value = (data['scale'] as num? ?? 1).toDouble().clamp(.5, 2.0);
      } catch (_) {}
      setState(() => ready = true);
      return;
    }
    try {
      final data = EditorSession.map(await c.native('readSettings'));
      c.restoreDeskWork(EditorSession.map(data['deskWork']));
      c.deskDefault.value = data['deskDefault'] as String? ?? 'Tools';
      uiScale.value = (data['scale'] as num? ?? 1).toDouble().clamp(.5, 2.0);
      dock.restore(data['newWorkspace']);
    } catch (_) {}
    _savedWorkspace = jsonEncode(dock.snapshot());
    dock.layout.addListener(_workspaceChanged);
    _listening = true;
    setState(() => ready = true);
  }

  @override
  void dispose() {
    c
        .slice('animationAppearance', const ['animate'])
        .removeListener(_syncAnimation);
    c.browserTab.removeListener(_followBrowserTab);
    console.dispose();
    if (_listening) dock.layout.removeListener(_workspaceChanged);
    final pending = _workspaceTimer != null;
    _workspaceTimer?.cancel();
    if (pending) _writeWorkspace();
    c.dispose();
    super.dispose();
  }

  void _syncAnimation() => EditorTheme.animating.value = c.animating;
  void _followBrowserTab() {
    if (_browserTabs.contains(c.browserTab.value))
      _place(c.browserTab.value, 'show');
  }

  /// A panel asked to be shown: choose its tab, or reopen it if it was closed.
  Future<void> _place(String name, String placement) async {
    if (placement == 'hidden') return;
    if (placement == 'window') {
      await _detach(name);
      return;
    }
    dock.activate(name);
    if (panelSpec(name)?.drawer == true) c.deskDrawer.value = name;
  }

  /// The workspace as last written (or as read at start): a layout change or a pointer release writes only
  /// when the saved form differs, after a short pause so a drag is one write.
  bool _listening = false;
  String _savedWorkspace = '';
  Timer? _workspaceTimer;

  void _workspaceChanged() {
    final now = dock.snapshot();
    if (jsonEncode(now) == _savedWorkspace) return;
    _workspaceTimer?.cancel();
    _workspaceTimer = Timer(const Duration(milliseconds: 400), _writeWorkspace);
  }

  void _writeWorkspace() {
    _workspaceTimer = null;
    final now = dock.snapshot();
    final text = jsonEncode(now);
    if (text == _savedWorkspace) return;
    _savedWorkspace = text;
    c.storeSetting('newWorkspace', now);
  }

  /// Panels that live in a window of their own, by the window's id. The window belongs to native, as in Classic.
  final detached = <String, List<String>>{};
  bool get _panelWindow => c.windowInfo['main'] == false;

  Future<void> _detach(String name) async {
    if (_panelWindow || !dock.defs.containsKey(name) || !dock.isOpen(name)) return;
    try {
      final info = EditorSession.map(await c.native('openPanelWindow', {'panels': [name]}));
      if (!mounted) return;
      detached['${info['id']}'] = [name];
      dock.close(name);
    } catch (e) {
      c.error.value = '$e';
    }
  }

  /// A panel window was closed: its panels come back to the workspace.
  void _windowClosed(Map<String, dynamic> info) {
    if (!mounted || _panelWindow) return;
    final names = detached.remove('${info['id']}') ?? (info['panels'] as List? ?? []).whereType<String>().toList();
    for (final name in names) {
      dock.activate(name);
    }
  }

  Future<void> menu(String action) async {
    setState(() => sheet = null);
    if (action == _resetLayout) {
      dock.reset();
      return;
    }
    if (await documentAction(context, c, action)) return;
    await _place(action, 'show');
  }

  void toggleSheet(String name) =>
      setState(() => sheet = sheet == name ? null : name);

  Widget pane(String name) =>
      buildPanel(name, c, keys.putIfAbsent(name, () => GlobalKey()), face: newDeskFace);

  /// Every panel the workspace can hold, by the id the routes use. The tab
  /// shows the panel's name; the panel keeps its own controls, not its name.
  late final dock = DockWorkspace(
    {
      for (final name in _browserTabs)
        name: PanelDef(
          name,
          (_browserLabels[name] ?? name).toUpperCase(),
          () => ShelfPanel(controller: c, name: name),
          minSize: 220,
        ),
      for (final name in _centerTabs)
        name: PanelDef(name, name.toUpperCase(), () => pane(name), minSize: 300),
      'Inspector': PanelDef(
        'Inspector',
        'INSPECTOR',
        () => InspectorPanel(
          key: keys.putIfAbsent('Inspector', () => GlobalKey()),
          controller: c,
          instruments: InspectorInstruments(
            transform: (context, controller) => NewTransform(controller: controller),
            layout: (context, controller, layer) => NewLayout(controller: controller, layer: layer),
            effectParams: (context, controller, layerId, effect) => NewEffectParams(
              key: ValueKey('new-effect:$layerId:${effect['id']}'),
              controller: controller,
              layerId: layerId,
              effectId: effect['id'] as Object,
            ),
          ),
        ),
        minSize: 240,
      ),
      'Timeline': PanelDef('Timeline', 'TIMELINE', () => pane('Timeline'), minSize: 120),
      'Desk': PanelDef('Desk', 'DESK', () => pane('Desk'), minSize: 200),
      'Console': PanelDef('Console', 'CONSOLE', () => NewConsole(log: console), minSize: 120),
    },
    (item) => DockingRow([
      DockingTabs([for (final name in _browserTabs) item(name)], weight: .19),
      DockingColumn([
        DockingRow([
          DockingTabs([for (final name in _centerTabs) item(name)], weight: .78),
          item('Inspector', weight: .22),
        ], weight: .7),
        DockingRow([
          item('Timeline', weight: .78),
          DockingTabs([item('Desk'), item('Console')], weight: .22),
        ], weight: .3),
      ], weight: .81),
    ]),
    onDetach: _detach,
  );

  @override
  Widget build(BuildContext context) {
    final t = EditorTheme.of(context);
    if (ready && _panelWindow) {
      final names = (c.windowInfo['panels'] as List? ?? const []).whereType<String>().toList();
      return ColoredBox(
        color: ShellTokens.surface,
        child: DefaultTextStyle(
          style: t.text,
          child: names.isEmpty ? const SizedBox.shrink() : (dock.defs[names.first]?.build() ?? pane(names.first)),
        ),
      );
    }
    return Focus(
      autofocus: true,
      onKeyEvent: shortcuts.handle,
      child: ColoredBox(
        color: ShellTokens.rule,
        child: DefaultTextStyle(
          style: t.text,
          child: Stack(
            children: [
              Column(
                children: [
                  NewTopBar(
                    c: c,
                    viewItems: [
                      ...dock.defs.keys,
                      for (final spec in panelCatalog)
                        if (spec.drawer) spec.name,
                      _resetLayout,
                    ],
                    sheet: sheet,
                    onMenu: menu,
                    onSheet: toggleSheet,
                  ),
                  const SizedBox(height: ShellTokens.ruleWidth),
                  Expanded(
                    child: ready
                        ? _face()
                        : const ColoredBox(
                            color: ShellTokens.ground,
                            child: Center(child: EditorSpinner()),
                          ),
                  ),
                  const SizedBox(height: ShellTokens.ruleWidth),
                  NewStatusBar(c: c, notice: notice),
                ],
              ),
              if (sheet != null) ..._sheet(t),
            ],
          ),
        ),
      ),
    );
  }

  Widget _face() => Listener(
    behavior: HitTestBehavior.translucent,
    onPointerUp: (_) => _workspaceChanged(),
    child: _dockView(),
  );

  Widget _dockView() => TabbedViewTheme(
    data: shellTabs(),
    child: MultiSplitViewTheme(
      data: shellSplit(),
      child: dock.view(),
    ),
  );

  List<Widget> _sheet(EditorTheme t) => [
    Positioned.fill(
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () => setState(() => sheet = null),
      ),
    ),
    Positioned(
      left: dock.rectOf('Create')?.right ?? ShellTokens.browserWidth,
      top: ShellTokens.topBar,
      child: Container(
        width: sheet == 'Settings'
            ? EditorMetrics.sheetWide
            : EditorMetrics.sheet,
        decoration: BoxDecoration(
          color: ShellTokens.surface,
          border: Border.all(color: ShellTokens.ruleStrong),
        ),
        child: switch (sheet) {
          'Composition' => CompositionControls(controller: c),
          'Export' => ExportControls(controller: c),
          _ => NewSettings(c: c, scale: uiScale),
        },
      ),
    ),
  ];
}

