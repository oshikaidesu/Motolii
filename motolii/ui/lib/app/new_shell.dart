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
import 'editor_window.dart' show freezeNotice;

/// The New projection of the same session: one face with the four regions
/// fixed in place (Browser, Stage, Inspector over Desk, Timeline), and the
/// panels Classic docks hosted as they are. Capabilities it cannot reach yet
/// are listed as NOT YET ROUTED in docs/stage5/ui-rebaseline/routing.md.
class NewShell extends StatefulWidget {
  const NewShell({super.key});
  @override
  State<NewShell> createState() => _NewShellState();
}

const _browserTabs = ['Create', 'Effects', 'Media', 'Colors', 'Fonts', 'Files'];
const _centerTabs = ['Stage', 'Camera', 'Notes', 'Web'];

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
  String browserTab = 'Create';
  String centerTab = 'Stage';

  /// Tabs opened once stay mounted, as Classic's dock keeps them, so a
  /// shelf's search or a Stage's view survives a trip to another tab.
  final opened = <String>{'Create', 'Stage'};
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
    } catch (_) {}
    setState(() => ready = true);
  }

  @override
  void dispose() {
    c
        .slice('animationAppearance', const ['animate'])
        .removeListener(_syncAnimation);
    c.browserTab.removeListener(_followBrowserTab);
    c.dispose();
    super.dispose();
  }

  void _syncAnimation() => EditorTheme.animating.value = c.animating;
  void _followBrowserTab() {
    if (_browserTabs.contains(c.browserTab.value))
      _place(c.browserTab.value, 'show');
  }

  /// A panel asked to be shown. Every region is always on the face, so
  /// showing one is choosing its tab; detaching into a window is not routed.
  Future<void> _place(String name, String placement) async {
    if (placement == 'hidden' || placement == 'window') return;
    setState(() {
      if (_browserTabs.contains(name)) browserTab = name;
      if (_centerTabs.contains(name)) centerTab = name;
      opened.add(name);
    });
    if (panelSpec(name)?.drawer == true) c.deskDrawer.value = name;
  }

  Future<void> menu(String action) async {
    setState(() => sheet = null);
    if (await documentAction(context, c, action)) return;
    await _place(action, 'show');
  }

  void toggleSheet(String name) =>
      setState(() => sheet = sheet == name ? null : name);

  Widget pane(String name) =>
      buildPanel(name, c, keys.putIfAbsent(name, () => GlobalKey()));

  /// The opened tabs of a region, the active one in front.
  Widget stack(List<String> tabs, String active) {
    final shown = tabs.where(opened.contains).toList();
    return IndexedStack(
      index: shown.indexOf(active),
      sizing: StackFit.expand,
      children: [for (final name in shown) pane(name)],
    );
  }

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

  Widget _face() => Row(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      SizedBox(
        width: ShellTokens.browserWidth,
        child: _Region(
          tabs: _browserTabs,
          labels: const {'Create': 'Objects'},
          active: browserTab,
          onPick: (name) => _place(name, 'show'),
          child: stack(_browserTabs, browserTab),
        ),
      ),
      const SizedBox(width: ShellTokens.ruleWidth),
      Expanded(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Expanded(
                    child: _Region(
                      tabs: _centerTabs,
                      active: centerTab,
                      onPick: (name) => _place(name, 'show'),
                      child: stack(_centerTabs, centerTab),
                    ),
                  ),
                  const SizedBox(width: ShellTokens.ruleWidth),
                  SizedBox(
                    width: ShellTokens.inspectorWidth,
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        Expanded(
                          child: _Region(
                            tabs: const ['Inspector'],
                            active: 'Inspector',
                            child: pane('Inspector'),
                          ),
                        ),
                        const SizedBox(height: ShellTokens.ruleWidth),
                        SizedBox(
                          height: deskHostSpec.height.px,
                          child: _Region(
                            tabs: const ['Desk'],
                            active: 'Desk',
                            child: pane('Desk'),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: ShellTokens.ruleWidth),
            SizedBox(
              height: ShellTokens.bottomHeight,
              child: _Region(
                tabs: const ['Timeline'],
                active: 'Timeline',
                child: pane('Timeline'),
              ),
            ),
          ],
        ),
      ),
    ],
  );

  List<Widget> _sheet(EditorTheme t) => [
    Positioned.fill(
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () => setState(() => sheet = null),
      ),
    ),
    Positioned(
      left: ShellTokens.browserWidth,
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

/// The strip across the top: the name, the document menus, the three sheets,
/// and the document's own name on the right.
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
            fontWeight: FontWeight.w600,
            letterSpacing: ShellTokens.wordmarkTracking,
            color: ShellTokens.ink,
          ),
        ),
        const SizedBox(width: ShellTokens.tabGap),
        _menu(context, 'File', fileActions),
        _menu(context, 'Edit', editActions.keys.toList()),
        _menu(context, 'View', [
          ..._browserTabs,
          ..._centerTabs,
          for (final spec in panelCatalog)
            if (spec.drawer) spec.name,
        ]),
        const SizedBox(width: ShellTokens.tabGap),
        for (final name in ['Composition', 'Export', 'Settings'])
          _Label(
            name,
            active: shell.sheet == name,
            onTap: () => shell.toggleSheet(name),
          ),
        const Spacer(),
        ValueListenableBuilder<Map<String, dynamic>>(
          valueListenable: c.slice('title', const ['dirty', 'path']),
          builder: (_, state, __) => Text(
            '${state['dirty'] == true ? '• ' : ''}'
            '${(state['path'] as String? ?? 'Untitled').split('/').last}',
            style: ShellTokens.kickerStyle(ShellTokens.inkMuted),
          ),
        ),
      ],
    ),
  );
}

/// An uppercase label that is pressed: a menu title or a tab.
class _Label extends StatelessWidget {
  const _Label(this.text, {this.onTap, this.active = false});
  final String text;
  final VoidCallback? onTap;
  final bool active;
  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    selected: active,
    label: text,
    child: MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
          alignment: Alignment.center,
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                color: active ? ShellTokens.sky : EditorTheme.clear,
                width: ShellTokens.activeRule,
              ),
            ),
          ),
          child: Text(
            text.toUpperCase(),
            style: ShellTokens.kickerStyle(
              active ? ShellTokens.ink : ShellTokens.inkMuted,
            ),
          ),
        ),
      ),
    ),
  );
}

/// One region of the face: a strip of tabs over the panel it shows.
class _Region extends StatelessWidget {
  const _Region({
    required this.tabs,
    required this.active,
    required this.child,
    this.labels = const {},
    this.onPick,
  });
  final List<String> tabs;
  final Map<String, String> labels;
  final String active;
  final ValueChanged<String>? onPick;
  final Widget child;
  @override
  Widget build(BuildContext context) => ColoredBox(
    color: ShellTokens.surface,
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          height: ShellTokens.header,
          color: ShellTokens.ground,
          child: SingleChildScrollView(
            primary: false,
            scrollDirection: Axis.horizontal,
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                for (final name in tabs)
                  _Label(
                    labels[name] ?? name,
                    active: name == active,
                    onTap: onPick == null ? null : () => onPick!(name),
                  ),
              ],
            ),
          ),
        ),
        const SizedBox(
          height: ShellTokens.ruleWidth,
          child: ColoredBox(color: ShellTokens.rule),
        ),
        Expanded(child: child),
      ],
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
