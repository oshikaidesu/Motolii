import 'dart:math' as math;

import '../foundation/glyphs.dart';

import 'package:flutter/widgets.dart';

import '../foundation/theme.dart';
import '../foundation/panel_catalog.dart';
import 'layout.dart';
import 'panel_ids.dart';
import '../foundation/metrics.dart';
import '../foundation/leaves.dart';

class WorkspaceView extends StatefulWidget {
  const WorkspaceView({
    super.key,
    required this.layout,
    required this.panelBuilder,
    required this.onMove,
    required this.onClose,
    required this.onDetach,
    required this.onLayoutChanged,
  });
  final DockNode layout;
  final Widget Function(String) panelBuilder;
  final void Function(String, DockNode, String) onMove;
  final void Function(String) onClose, onDetach;
  final VoidCallback onLayoutChanged;
  @override
  State<WorkspaceView> createState() => _WorkspaceViewState();
}

class _WorkspaceViewState extends State<WorkspaceView> {
  /// Which pane wears the ring. Held as a listenable, not as state: the ring
  /// is one border on two panes, and a click that moves it must not rebuild
  /// every panel in the window.
  final focusedPanel = ValueNotifier<String?>(null);
  @override
  void dispose() {
    focusedPanel.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => dockView(widget.layout);

  /// A divider drag and a tab click are each one node's own state: the split
  /// and the leaf hold it, so the panels they hand back are the same objects
  /// and do not rebuild.
  Widget dockView(DockNode node) {
    if (!node.leaf) {
      final firstEmpty = node.first!.leaves.every((n) => n.tabs.isEmpty);
      final secondEmpty = node.second!.leaves.every((n) => n.tabs.isEmpty);
      if (firstEmpty && !secondEmpty) return dockView(node.second!);
      if (secondEmpty && !firstEmpty) return dockView(node.first!);
      return _Split(
        node: node,
        first: dockView(node.first!),
        second: dockView(node.second!),
        onLayoutChanged: widget.onLayoutChanged,
      );
    }
    return _Leaf(
      node: node,
      focusedPanel: focusedPanel,
      panelBuilder: widget.panelBuilder,
      onMove: widget.onMove,
      onClose: widget.onClose,
      onDetach: widget.onDetach,
    );
  }
}

class _Split extends StatefulWidget {
  const _Split({
    required this.node,
    required this.first,
    required this.second,
    required this.onLayoutChanged,
  });
  final DockNode node;
  final Widget first, second;
  final VoidCallback onLayoutChanged;
  @override
  State<_Split> createState() => _SplitState();
}

class _SplitState extends State<_Split> {
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final node = widget.node;
      final horizontal = node.axis == Axis.horizontal;
      final size = horizontal ? box.maxWidth : box.maxHeight;
      final first = node.firstExtent(size);
      final children = [
        SizedBox(
          width: horizontal ? first : null,
          height: horizontal ? null : first,
          child: widget.first,
        ),
        MouseRegion(
          cursor: horizontal
              ? SystemMouseCursors.resizeColumn
              : SystemMouseCursors.resizeRow,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanUpdate: (event) => setState(
              () =>
                  node.drag(horizontal ? event.delta.dx : event.delta.dy, size),
            ),
            onPanEnd: (_) => widget.onLayoutChanged(),
            child: Container(
              width: horizontal ? EditorMetrics.s4 : null,
              height: horizontal ? null : EditorMetrics.s4,
              color: EditorTheme.of(context).line,
            ),
          ),
        ),
        Expanded(child: widget.second),
      ];
      return horizontal
          ? Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: children,
            )
          : Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: children,
            );
    },
  );
}

class _Leaf extends StatefulWidget {
  const _Leaf({
    required this.node,
    required this.focusedPanel,
    required this.panelBuilder,
    required this.onMove,
    required this.onClose,
    required this.onDetach,
  });
  final DockNode node;
  final ValueNotifier<String?> focusedPanel;
  final Widget Function(String) panelBuilder;
  final void Function(String, DockNode, String) onMove;
  final void Function(String) onClose, onDetach;
  @override
  State<_Leaf> createState() => _LeafState();
}

class _LeafState extends State<_Leaf> {
  /// The tab in front, as a signal: the strip and the stack's index follow it,
  /// the panels behind it are built once and stay.
  late final active = ValueNotifier<String>(widget.node.active);

  @override
  void didUpdateWidget(_Leaf old) {
    super.didUpdateWidget(old);
    if (active.value != widget.node.active) active.value = widget.node.active;
  }

  @override
  void dispose() {
    active.dispose();
    super.dispose();
  }

  void _show(String name) {
    widget.node.active = name;
    active.value = name;
  }

  /// What a tab takes with its word beside the icon: icon, gap, padding both
  /// sides and the rule. Names repeat, so each is measured once.
  static final _labelled = <String, double>{};
  static double _labelledWidth(String name) => _labelled[name] ??= () {
    final painter = TextPainter(
      text: TextSpan(
        text: name,
        style: const TextStyle(
          fontFamily: EditorTheme.fontFamily,
          fontWeight: FontWeight.w600,
          fontSize: EditorMetrics.dense,
        ),
      ),
      maxLines: 1,
      textDirection: TextDirection.ltr,
    )..layout();
    final width = painter.width;
    painter.dispose();
    return width +
        EditorMetrics.title +
        EditorMetrics.s4 +
        EditorMetrics.s5 * 2 +
        EditorMetrics.s2;
  }();

  /// Tabs that do not fit with their words keep the word only for the one in
  /// front; the rest are their icon, named on hover. A strip that scrolled its
  /// last tab out of sight cut it mid-word with nothing to say so.
  bool _fits(List<String> tabs, double room) =>
      tabs.length < 2 ||
      tabs.fold<double>(0, (sum, name) => sum + _labelledWidth(name)) <= room;

  Widget _tab(String name, String shown, {bool compact = false}) =>
      Draggable<String>(
        data: name,
        feedback: DefaultTextStyle(
          style: DefaultTextStyle.of(context).style,
          child: ColoredBox(
            color: EditorTheme.of(context).raised,
            child: Padding(
              padding: const EdgeInsets.all(EditorMetrics.s8),
              child: Text(name),
            ),
          ),
        ),
        childWhenDragging: Opacity(opacity: .4, child: Text(name)),
        child: GestureDetector(
          onSecondaryTapDown: (details) async {
            final action = await showEditorMenu<String>(
              context,
              details.globalPosition,
              [
                const EditorMenuItem(value: 'detach', child: Text('Detach')),
                const EditorMenuItem(value: 'close', child: Text('Close')),
              ],
            );
            if (action == 'detach' || action == 'window') widget.onDetach(name);
            if (action == 'close') widget.onClose(name);
          },
          child: EditorPress(
            onTap: () => _show(name),
            child: Container(
              height: EditorMetrics.row,
              alignment: Alignment.center,
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
              decoration: BoxDecoration(
                color: widget.node.tabs.length > 1 && shown == name
                    ? EditorTheme.of(context).tab
                    : EditorTheme.of(context).app,
                border: Border(
                  right: BorderSide(
                    color: EditorTheme.of(context).line,
                    width: EditorMetrics.s2,
                  ),
                ),
              ),
              child: EditorTooltip(
                message: name,
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      panelSpec(name)?.icon ?? Glyph.all_inbox_outlined,
                      size: EditorMetrics.title,
                    ),
                    if (!compact) ...[
                      const SizedBox(width: EditorMetrics.s4),
                      Text(
                        name,
                        style: TextStyle(
                          color: widget.node.tabs.length > 1 && shown == name
                              ? EditorTheme.of(context).tabInk
                              : EditorTheme.of(context).ink,
                          fontWeight: FontWeight.w600,
                          fontSize: EditorMetrics.dense,
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final node = widget.node;
    final panes = [for (final name in node.tabs) widget.panelBuilder(name)];
    return LayoutBuilder(
      builder: (context, box) => DragTarget<String>(
        onWillAcceptWithDetails: (d) => paneNames.contains(d.data),
        onAcceptWithDetails: (d) {
          final render = context.findRenderObject() as RenderBox;
          final p = render.globalToLocal(d.offset);
          final edge = p.dy < 20
              ? 'center'
              : p.dx < box.maxWidth * .18
              ? 'left'
              : p.dx > box.maxWidth * .82
              ? 'right'
              : p.dy < box.maxHeight * .22
              ? 'top'
              : p.dy > box.maxHeight * .78
              ? 'bottom'
              : 'center';
          widget.onMove(d.data, node, edge);
        },
        builder: (context, candidates, rejected) => Focus(
          onFocusChange: (focused) {
            if (focused) widget.focusedPanel.value = node.id;
          },
          child: Listener(
            onPointerDown: (_) => widget.focusedPanel.value = node.id,
            child: ValueListenableBuilder<String?>(
              valueListenable: widget.focusedPanel,
              builder: (context, focused, child) => Container(
                decoration: BoxDecoration(
                  color: EditorTheme.of(context).panel,
                  border: candidates.isNotEmpty
                      ? Border.all(
                          color: EditorTheme.of(context).accent,
                          width: EditorMetrics.s2,
                        )
                      : Border.all(
                          color: focused == node.id
                              ? EditorInk.of(context).focusRing
                              : EditorTheme.clear,
                          width: 1,
                        ),
                  borderRadius: BorderRadius.circular(EditorMetrics.s3),
                ),
                child: child,
              ),
              child: Column(
                children: [
                  SizedBox(
                    height: EditorMetrics.row,
                    child: Row(
                      children: [
                        Expanded(
                          child: LayoutBuilder(
                            builder: (context, room) => SingleChildScrollView(
                              scrollDirection: Axis.horizontal,
                              child: ValueListenableBuilder<String>(
                                valueListenable: active,
                                builder: (context, shown, _) {
                                  final fits = _fits(node.tabs, room.maxWidth);
                                  return Row(
                                    children: [
                                      for (final name in node.tabs)
                                        _tab(
                                          name,
                                          shown,
                                          compact: !fits && name != shown,
                                        ),
                                    ],
                                  );
                                },
                              ),
                            ),
                          ),
                        ),
                        if (node.tabs.isNotEmpty)
                          EditorButton(
                            '×',
                            () => widget.onClose(node.active),
                            tooltip: 'Close panel',
                          ),
                      ],
                    ),
                  ),
                  Expanded(
                    child: node.tabs.isEmpty
                        ? Center(
                            child: Text(
                              'Drop a panel here',
                              style: TextStyle(
                                color: EditorTheme.of(context).muted,
                              ),
                            ),
                          )
                        : ValueListenableBuilder<String>(
                            valueListenable: active,
                            builder: (context, shown, _) => IndexedStack(
                              index: math.max(0, node.tabs.indexOf(shown)),
                              children: panes,
                            ),
                          ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
