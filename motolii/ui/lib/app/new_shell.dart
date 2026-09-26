import 'dart:async';
import 'dart:convert';

import 'package:docking/docking.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../foundation/glyphs.dart';
import '../foundation/leaves.dart';
import '../foundation/metrics.dart';
import '../foundation/panel_catalog.dart';
import '../foundation/panel_controls.dart';
import '../foundation/shell_tokens.dart';
import '../foundation/theme.dart';
import '../input/editor_shortcuts.dart';
import '../panels/browser.dart';
import '../panels/composition_controls.dart';
import '../panels/export_controls.dart';
import '../panels/registry.dart';
import '../session/editor_session.dart';
import 'editor_actions.dart';
import '../workspace/dock_workspace.dart';
import 'new/browser.dart';
import 'new/dock_theme.dart';
import 'editor_window.dart' show freezeNotice;

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
  String? sheet;
  bool ready = false;

  @override
  void initState() {
    super.initState();
    c
        .slice('animationAppearance', const ['animate'])
        .addListener(_syncAnimation);
    c.confirmClose = () => confirmReplacement(context, c);
    c.panelPlacementRequested = _place;
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
    try {
      final data = EditorSession.map(await c.native('readSettings'));
      c.restoreDeskWork(EditorSession.map(data['deskWork']));
      c.deskDefault.value = data['deskDefault'] as String? ?? 'Tools';
      dock.restore(data['newWorkspace']);
    } catch (_) {}
    _savedWorkspace = jsonEncode(dock.snapshot());
    dock.layout.addListener(_workspaceChanged);
    setState(() => ready = true);
  }

  @override
  void dispose() {
    c
        .slice('animationAppearance', const ['animate'])
        .removeListener(_syncAnimation);
    c.browserTab.removeListener(_followBrowserTab);
    if (ready) dock.layout.removeListener(_workspaceChanged);
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
  /// Detaching into a window is not routed.
  Future<void> _place(String name, String placement) async {
    if (placement == 'hidden' || placement == 'window') return;
    dock.activate(name);
    if (panelSpec(name)?.drawer == true) c.deskDrawer.value = name;
  }

  /// The workspace as last written (or as read at start): a layout change or a pointer release writes only
  /// when the saved form differs, after a short pause so a drag is one write.
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
      buildPanel(name, c, keys.putIfAbsent(name, () => GlobalKey()));

  /// Every panel the workspace can hold, by the id the routes use. The tab
  /// shows the panel's name; the panel keeps its own controls, not its name.
  late final dock = DockWorkspace(
    {
      for (final name in _browserTabs)
        name: PanelDef(
          name,
          (_browserLabels[name] ?? name).toUpperCase(),
          () => NewBrowser(controller: c, tab: name),
          minSize: 220,
        ),
      for (final name in _centerTabs)
        name: PanelDef(name, name.toUpperCase(), () => pane(name), minSize: 300),
      'Inspector': PanelDef('Inspector', 'INSPECTOR', () => pane('Inspector'), minSize: 240),
      'Timeline': PanelDef('Timeline', 'TIMELINE', () => pane('Timeline'), minSize: 120),
      'Desk': PanelDef('Desk', 'DESK', () => pane('Desk'), minSize: 200),
    },
    (item) => DockingRow([
      DockingTabs([for (final name in _browserTabs) item(name)], weight: .19),
      DockingColumn([
        DockingRow([
          DockingTabs([for (final name in _centerTabs) item(name)], weight: .78),
          item('Inspector', weight: .22),
        ], weight: .7),
        DockingRow([item('Timeline', weight: .78), item('Desk', weight: .22)], weight: .3),
      ], weight: .81),
    ]),
  );

  @override
  Widget build(BuildContext context) {
    final t = EditorTheme.of(context);
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
                  _TopBar(shell: this),
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
                  _StatusBar(c: c, notice: notice),
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
          _ => _Settings(c: c),
        },
      ),
    ),
  ];
}

/// The strip across the top, read left to right as an instrument: the name,
/// the document menus, the transport and its readouts, the three sheets,
/// and the document's own name.
class _TopBar extends StatelessWidget {
  const _TopBar({required this.shell});
  final _NewShellState shell;
  EditorSession get c => shell.c;

  Widget _menu(BuildContext context, String label, List<String> items) =>
      Builder(
        builder: (context) => _Label(
          label,
          onTap: () async {
            final box = context.findRenderObject() as RenderBox;
            final chosen = await showEditorMenu<String>(
              context,
              box.localToGlobal(box.size.bottomLeft(Offset.zero)),
              [
                for (final item in items)
                  EditorMenuItem(value: item, child: Text(item)),
              ],
            );
            if (chosen != null) shell.menu(chosen);
          },
        ),
      );

  @override
  Widget build(BuildContext context) => Container(
    height: ShellTokens.topBar,
    color: ShellTokens.ground,
    padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
    child: Row(
      children: [
        const Text(
          'Motolii',
          style: TextStyle(
            fontSize: ShellTokens.wordmark,
            fontWeight: FontWeight.w700,
            letterSpacing: ShellTokens.wordmarkTracking,
            color: ShellTokens.ink,
          ),
        ),
        const SizedBox(width: ShellTokens.tabGap),
        _menu(context, 'File', fileActions),
        _menu(context, 'Edit', editActions.keys.toList()),
        _menu(context, 'View', [
          ...shell.dock.defs.keys,
          for (final spec in panelCatalog)
            if (spec.drawer) spec.name,
          _resetLayout,
        ]),
        const SizedBox(width: ShellTokens.tabGap),
        _Transport(c: c),
        const Spacer(),
        _Segment(
          names: const ['Composition', 'Export', 'Settings'],
          active: shell.sheet,
          onPick: shell.toggleSheet,
        ),
        const SizedBox(width: ShellTokens.tabGap),
        ValueListenableBuilder<Map<String, dynamic>>(
          valueListenable: c.slice('title', const ['dirty', 'path']),
          builder: (_, state, __) => Text(
            '${state['dirty'] == true ? '● ' : ''}'
            '${(state['path'] as String? ?? 'Untitled').split('/').last}',
            style: ShellTokens.kickerStyle(ShellTokens.inkMuted),
          ),
        ),
      ],
    ),
  );
}

/// Play, back to the start, and Animate, then where the playhead is. Only
/// the frame readout follows playback, and it is one line of text.
class _Transport extends StatelessWidget {
  const _Transport({required this.c});
  final EditorSession c;

  static String _clock(int frame, num fps) {
    final rate = fps <= 0 ? 1 : fps;
    final seconds = frame / rate;
    final m = seconds ~/ 60;
    final sec = (seconds % 60).floor();
    final ff = (frame % rate.round().clamp(1, 1000)).toString().padLeft(2, '0');
    return '${m.toString().padLeft(2, '0')}:${sec.toString().padLeft(2, '0')}:$ff';
  }

  @override
  Widget build(BuildContext context) {
    final timing = c.slice('transport', const ['fps', 'durationFrames']);
    final animate = c.slice('transportAnimate', const ['animate']);
    return Row(
      children: [
        ValueListenableBuilder<bool>(
          valueListenable: c.playing,
          builder: (_, playing, __) => _Key(
            tip: playing ? 'Pause (Space)' : 'Play (Space)',
            lit: playing ? ShellTokens.mint : null,
            onTap: c.togglePlayback,
            child: playing
                ? const _Bars()
                : const Icon(
                    Glyph.play_arrow_outlined,
                    size: ShellTokens.readout + EditorMetrics.s4,
                    color: ShellTokens.ink,
                  ),
          ),
        ),
        _Key(
          tip: 'Stop and go to the start (Home)',
          onTap: () {
            c.stopPlayback();
            c.seek(0);
          },
          child: Container(
            width: ShellTokens.keyGlyph - EditorMetrics.s2,
            height: ShellTokens.keyGlyph - EditorMetrics.s2,
            color: ShellTokens.ink,
          ),
        ),
        ValueListenableBuilder(
          valueListenable: animate,
          builder: (_, __, ___) => _Key(
            tip: 'Animate (A): values you touch become keys',
            lit: c.animating ? ShellTokens.pink : null,
            onTap: c.supports('animate')
                ? () => c.setAnimate(!c.animating)
                : null,
            child: Container(
              width: ShellTokens.keyGlyph,
              height: ShellTokens.keyGlyph,
              decoration: BoxDecoration(
                color: c.animating ? ShellTokens.inkOnAccent : ShellTokens.pink,
                shape: BoxShape.circle,
              ),
            ),
          ),
        ),
        const SizedBox(width: ShellTokens.readoutGap),
        ValueListenableBuilder(
          valueListenable: timing,
          builder: (_, __, ___) {
            final fps = (c.state['fps'] as num?) ?? 30;
            final duration = (c.state['durationFrames'] as num?)?.toInt() ?? 0;
            return Row(
              children: [
                Text(
                  fps.toStringAsFixed(2),
                  style: ShellTokens.readoutStyle(ShellTokens.ink),
                ),
                Text(
                  ' FPS',
                  style: ShellTokens.kickerStyle(ShellTokens.inkFaint),
                ),
                const _Tick(),
                ValueListenableBuilder<int>(
                  valueListenable: c.frame,
                  builder: (_, frame, __) => Text(
                    '${_clock(frame, fps)}  '
                    '${frame.toString().padLeft(4, '0')}'
                    '/${duration.toString().padLeft(4, '0')}',
                    style: ShellTokens.readoutStyle(ShellTokens.ink),
                  ),
                ),
              ],
            );
          },
        ),
      ],
    );
  }
}

class _Tick extends StatelessWidget {
  const _Tick();
  @override
  Widget build(BuildContext context) => Container(
    width: ShellTokens.ruleWidth,
    height: ShellTokens.readout,
    margin: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
    color: ShellTokens.ruleStrong,
  );
}

class _Bars extends StatelessWidget {
  const _Bars();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      for (var i = 0; i < 2; i++) ...[
        if (i == 1) const SizedBox(width: EditorMetrics.s3),
        Container(
          width: EditorMetrics.s3,
          height: ShellTokens.keyGlyph,
          color: ShellTokens.inkOnAccent,
        ),
      ],
    ],
  );
}

/// A square key of the transport: flat, ruled, lit in its colour when on.
class _Key extends StatelessWidget {
  const _Key({required this.tip, required this.child, this.onTap, this.lit});
  final String tip;
  final Widget child;
  final VoidCallback? onTap;
  final Color? lit;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: tip,
    child: Semantics(
      button: true,
      label: tip,
      child: MouseRegion(
        cursor: onTap == null
            ? SystemMouseCursors.basic
            : SystemMouseCursors.click,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: onTap,
          child: Container(
            width: ShellTokens.key,
            height: ShellTokens.key,
            margin: const EdgeInsets.only(right: EditorMetrics.s2),
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: lit ?? ShellTokens.raised,
              border: Border.all(color: lit ?? ShellTokens.ruleStrong),
            ),
            child: child,
          ),
        ),
      ),
    ),
  );
}

/// Neighbouring choices in one ruled box, the chosen one lit.
class _Segment extends StatelessWidget {
  const _Segment({required this.names, required this.onPick, this.active});
  final List<String> names;
  final String? active;
  final ValueChanged<String> onPick;
  @override
  Widget build(BuildContext context) => Container(
    height: ShellTokens.key,
    decoration: BoxDecoration(
      border: Border.all(color: ShellTokens.ruleStrong),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        for (final name in names)
          Semantics(
            button: true,
            selected: name == active,
            label: name,
            child: MouseRegion(
              cursor: SystemMouseCursors.click,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => onPick(name),
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: ShellTokens.gutter + EditorMetrics.s4,
                  ),
                  alignment: Alignment.center,
                  color: name == active ? ShellTokens.sky : EditorTheme.clear,
                  child: Text(
                    name.toUpperCase(),
                    style: ShellTokens.kickerStyle(
                      name == active
                          ? ShellTokens.inkOnAccent
                          : ShellTokens.ink,
                    ),
                  ),
                ),
              ),
            ),
          ),
      ],
    ),
  );
}

/// An uppercase menu title that is pressed.
class _Label extends StatelessWidget {
  const _Label(this.text, {this.onTap});
  final String text;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    label: text,
    child: MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
          alignment: Alignment.center,
          child: Text(
            text.toUpperCase(),
            style: ShellTokens.kickerStyle(ShellTokens.ink),
          ),
        ),
      ),
    ),
  );
}

class _StatusBar extends StatelessWidget {
  const _StatusBar({required this.c, required this.notice});
  final EditorSession c;
  final ValueListenable<Map<String, dynamic>> notice;
  @override
  Widget build(BuildContext context) => Container(
    height: ShellTokens.statusBar,
    color: ShellTokens.ground,
    padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
    alignment: Alignment.centerLeft,
    child: ValueListenableBuilder<String?>(
      valueListenable: c.error,
      builder: (_, message, __) => ValueListenableBuilder(
        valueListenable: notice,
        builder: (_, doc, __) => Text(
          message ?? freezeNotice(doc) ?? effectsNotice(doc),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: ShellTokens.kickerStyle(ShellTokens.inkMuted),
        ),
      ),
    ),
  );
}

/// The preferences the New face can already reach; the panel placement table
/// and the Classic theme file belong to Classic's dock and are not routed.
class _Settings extends StatelessWidget {
  const _Settings({required this.c});
  final EditorSession c;
  Widget _row(String label, Widget control) => SizedBox(
    height: EditorMetrics.control,
    child: Row(
      children: [
        Expanded(child: Text(label)),
        control,
      ],
    ),
  );
  @override
  Widget build(BuildContext context) => ValueListenableBuilder(
    valueListenable: c.deskWork,
    builder: (context, _, __) => Padding(
      padding: const EdgeInsets.all(ShellTokens.gutter),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _row(
            'Browser tile size',
            SizedBox(
              width: EditorMetrics.s200,
              child: EditorZoomBar(
                keyPrefix: 'settings:browserTile',
                base: BrowserSize.base,
                min: BrowserSize.min,
                max: BrowserSize.max,
                value: BrowserSize.tile(c),
                onChanged: (v) => c.storeDesk('browserTile', v),
              ),
            ),
          ),
          _row(
            'Animate: key the start too',
            EditorSwitch(
              key: const ValueKey('settings:animateFrom'),
              on: c.animateFrom,
              glyph: Glyph.diamond_outlined,
              label:
                  'When Animate is turned on, remember the frame; the first '
                  'touch at another frame keys both that frame and this one',
              onChanged: (on) => c.storeDesk('animateFrom', on),
            ),
          ),
          _row(
            'New layers',
            SizedBox(
              width: EditorMetrics.s76,
              child: EditorChoice<String>(
                key: const ValueKey('settings:flatProjection'),
                value: c.flatProjection,
                choices: const [MapEntry('2.5D', '2.5D'), MapEntry('3D', '3D')],
                onChanged: (v) => c.storeDesk('flatProjection', v),
              ),
            ),
          ),
        ],
      ),
    ),
  );
}
