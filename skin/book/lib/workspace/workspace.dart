// The whole window: top bar, then the panes (panes.dart) — browser shelves | Stage over Timeline, Inspector over Desk, rearrangeable.
// Each seat is its own file and reads the shared Ws.
import 'dart:math' as math;

import 'package:docking/docking.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart';
import 'desk.dart';
import 'dock.dart';
import 'dock_glyphs.dart';
import 'panes.dart';
import 'stage.dart';
import 'stage_art.dart';
import 'timeline.dart';
import 'top_bar.dart';
import 'ws.dart';

class Workspace extends StatefulWidget {
  const Workspace({super.key});
  @override
  State<Workspace> createState() => _WorkspaceState();
}

class _WorkspaceState extends State<Workspace> {
  final _ws = Ws();

  @override
  void dispose() {
    _ws.dispose();
    super.dispose();
  }

  /// Enter keeps a try-on and Esc drops it, wherever focus is in the window.
  KeyEventResult _key(FocusNode _, KeyEvent e) {
    if (e is! KeyDownEvent || _ws.trial == null) return KeyEventResult.ignored;
    switch (e.logicalKey) {
      case LogicalKeyboardKey.escape:
        _ws.dropFx();
      case LogicalKeyboardKey.enter || LogicalKeyboardKey.numpadEnter:
        _ws.keepFx();
      default:
        return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  /// The panes drag their tabs on an overlay; the window brings its own so it works under any host.
  late final _entry = OverlayEntry(
    maintainState: true,
    builder: (context) => LayoutBuilder(
      builder: (context, box) {
        const window = _Window();
        if (box.maxWidth >= WsT.minWindow.width && box.maxHeight >= WsT.minWindow.height) return window;
        return FittedBox(
          child: SizedBox.fromSize(size: WsT.refWindow, child: window),
        );
      },
    ),
  );

  @override
  Widget build(BuildContext context) => WsScope(
    ws: _ws,
    child: Focus(
      autofocus: true,
      onKeyEvent: _key,
      child: ColoredBox(
        color: WsT.ground,
        child: Overlay(initialEntries: [_entry]),
      ),
    ),
  );
}

class _Window extends StatelessWidget {
  const _Window();
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const SizedBox(height: WsT.topBar, child: WsTopBar()),
      const SizedBox(height: WsT.gutter),
      Expanded(
        child: WsPanes(panes: _panes, preset: _preset, shelves: {for (final s in dockShelves) s.$1}),
      ),
    ],
  );
}

final _panes = [
  for (final (name, g, _) in dockShelves) WsPane(name, g, () => WsShelf(name), minSize: 240, iconOnly: true),
  WsPane('Stage', DockG.stage, () => const WsStage(), minSize: 320),
  WsPane('Inspector', DockG.inspector, () => const _InspectorSeat(), minSize: 200),
  WsPane('Timeline', DockG.timeline, () => const WsTimeline(), minSize: 160),
  WsPane('Desk', DockG.desk, () => const WsDesk(), minSize: WsT.lower),
];

/// A pane's tab strip, added on top of the reference heights so the seats keep theirs.
const _strip = 22.0;

/// The reference arrangement: the browser shelves (one strip of tabs) | Stage over Timeline on the left, Inspector over Desk on the right.
/// Side and lower panes start at their reference sizes in px and the panes without one share what is left.
/// contract: give the filling panes no weight; `multi_split_view` turns sizes into weights and would scale them against any weight given.
DockingArea _preset(DockingItem Function(String name, {double? size, double? weight}) item) => DockingRow([
  DockingColumn([
    DockingRow([
      DockingTabs([for (final (name, _, _) in dockShelves) item(name)], size: WsT.dock),
      item('Stage'),
    ]),
    item('Timeline', size: WsT.lower + _strip),
  ]),
  DockingColumn([item('Inspector'), item('Desk', size: WsT.lower + _strip)], size: WsT.inspector, minimalSize: 340),
]);

Kind _kindOf(WsLayer? l) => switch (l?.kind) {
  null || WsKind.audio => Kind.none,
  WsKind.text => Kind.text,
  WsKind.camera => Kind.camera,
  WsKind.group => Kind.group,
  _ => Kind.shape,
};

String _hex(Color c) => '#${(c.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}';

/// The paint each layer wears on the stage, so the Inspector's Fill names the colour you see.
const _fills = {'blob': ArtInk.orange, 'ring': ArtInk.lime, 'title': ArtInk.cream, 'chips': ArtInk.pink, 'confetti': ArtInk.pink, 'bg': ArtInk.blue};

/// The selected layer's transform as the stage draws it at this frame, in the Inspector's ids.
Map<String, double> _transform(Ws ws, String? id) {
  final b = WsArt.boxOf(ArtScene.of(ws, camera: true), id);
  if (b == null) return const {};
  final s = (b.scale * 1000).roundToDouble() / 10, r = (b.rot * 180 / math.pi * 10).roundToDouble() / 10;
  return {'pos.x': b.anchor.dx.roundToDouble(), 'pos.y': b.anchor.dy.roundToDouble(), 'scale.x': s, 'scale.y': s, 'rot.z': r};
}

/// The Inspector of the lab, fed by the workspace's selection; its panel links open the dock's shelves or the desk.
class _InspectorSeat extends StatelessWidget {
  const _InspectorSeat();
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), l = ws.layer, k = _kindOf(l), fill = _fills[l?.id];
    return LayoutBuilder(
      builder: (context, box) => InspHost(
        key: ValueKey(k),
        cfg: const Cfg(),
        kind: k,
        title: l?.name,
        onRoute: ws.route,
        values: _transform(ws, l?.id),
        strs: {if (fill != null) 'fill': _hex(fill)},
        child: InspectorPanel(directTransform: true, width: box.maxWidth, height: box.maxHeight),
      ),
    );
  }
}
