import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../foundation/theme.dart';
import '../foundation/panel_catalog.dart';
import 'layout.dart';
import 'panel_ids.dart';
import '../foundation/metrics.dart';

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
  String? focusedPanel;
  @override
  Widget build(BuildContext context) => dockView(widget.layout);
  Widget dockView(DockNode node) {
    if (!node.leaf) {
      final firstEmpty = node.first!.leaves.every((n) => n.tabs.isEmpty);
      final secondEmpty = node.second!.leaves.every((n) => n.tabs.isEmpty);
      if (firstEmpty && !secondEmpty) return dockView(node.second!);
      if (secondEmpty && !firstEmpty) return dockView(node.first!);
    }
    if (!node.leaf)
      return LayoutBuilder(
        builder: (context, box) {
          final horizontal = node.axis == Axis.horizontal;
          final size = horizontal ? box.maxWidth : box.maxHeight;
          final first = node.firstExtent(size);
          final children = [
            SizedBox(
              width: horizontal ? first : null,
              height: horizontal ? null : first,
              child: dockView(node.first!),
            ),
            MouseRegion(
              cursor: horizontal
                  ? SystemMouseCursors.resizeColumn
                  : SystemMouseCursors.resizeRow,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onPanUpdate: (event) => setState(
                  () => node.drag(
                    horizontal ? event.delta.dx : event.delta.dy,
                    size,
                  ),
                ),
                onPanEnd: (_) => widget.onLayoutChanged(),
                child: Container(
                  width: horizontal ? EditorMetrics.s4 : null,
                  height: horizontal ? null : EditorMetrics.s4,
                  color: EditorTheme.line,
                ),
              ),
            ),
            Expanded(child: dockView(node.second!)),
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
            if (focused && focusedPanel != node.id)
              setState(() => focusedPanel = node.id);
          },
          child: Listener(
            onPointerDown: (_) {
              if (focusedPanel != node.id)
                setState(() => focusedPanel = node.id);
            },
            child: Container(
              decoration: BoxDecoration(
                color: EditorTheme.panel,
                border: candidates.isNotEmpty
                    ? Border.all(
                        color: EditorTheme.accent,
                        width: EditorMetrics.s2,
                      )
                    : Border.all(
                        color: focusedPanel == node.id
                            ? const Color(0xffacacac)
                            : Colors.transparent,
                        width: 1,
                      ),
                borderRadius: BorderRadius.circular(EditorMetrics.s3),
              ),
              child: Column(
                children: [
                  SizedBox(
                    height: EditorMetrics.row,
                    child: Row(
                      children: [
                        Expanded(
                          child: SingleChildScrollView(
                            scrollDirection: Axis.horizontal,
                            child: Row(
                              children: [
                                for (final name in node.tabs)
                                  Draggable<String>(
                                    data: name,
                                    feedback: Material(
                                      color: EditorTheme.raised,
                                      child: Padding(
                                        padding: const EdgeInsets.all(
                                          EditorMetrics.s8,
                                        ),
                                        child: Text(name),
                                      ),
                                    ),
                                    childWhenDragging: Opacity(
                                      opacity: .4,
                                      child: Text(name),
                                    ),
                                    child: GestureDetector(
                                      onSecondaryTapDown: (details) async {
                                        final action = await showMenu<String>(
                                          context: context,
                                          position: RelativeRect.fromLTRB(
                                            details.globalPosition.dx,
                                            details.globalPosition.dy,
                                            0,
                                            0,
                                          ),
                                          items: [
                                            const PopupMenuItem(
                                              value: 'detach',
                                              height: EditorMetrics.row,
                                              child: Text('Detach'),
                                            ),
                                            const PopupMenuItem(
                                              value: 'close',
                                              height: EditorMetrics.row,
                                              child: Text('Close'),
                                            ),
                                          ],
                                        );
                                        if (action == 'detach' ||
                                            action == 'window')
                                          widget.onDetach(name);
                                        if (action == 'close')
                                          widget.onClose(name);
                                      },
                                      child: InkWell(
                                        onTap: () =>
                                            setState(() => node.active = name),
                                        child: Container(
                                          height: EditorMetrics.row,
                                          alignment: Alignment.center,
                                          padding: const EdgeInsets.symmetric(
                                            horizontal: EditorMetrics.s5,
                                          ),
                                          decoration: BoxDecoration(
                                            color:
                                                node.tabs.length > 1 &&
                                                    node.active == name
                                                ? EditorTheme.tab
                                                : EditorTheme.app,
                                            border: const Border(
                                              right: BorderSide(
                                                color: EditorTheme.line,
                                                width: EditorMetrics.s2,
                                              ),
                                            ),
                                          ),
                                          child: Row(
                                            mainAxisSize: MainAxisSize.min,
                                            children: [
                                              Icon(
                                                panelSpec(name)?.icon ??
                                                    Icons.all_inbox_outlined,
                                                size: EditorMetrics.title,
                                              ),
                                              const SizedBox(
                                                width: EditorMetrics.s4,
                                              ),
                                              Text(
                                                name,
                                                style: TextStyle(
                                                  color:
                                                      node.tabs.length > 1 &&
                                                          node.active == name
                                                      ? EditorTheme.tabInk
                                                      : EditorTheme.ink,
                                                  fontWeight: FontWeight.w600,
                                                  fontSize: EditorMetrics.dense,
                                                ),
                                              ),
                                            ],
                                          ),
                                        ),
                                      ),
                                    ),
                                  ),
                              ],
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
                        ? const Center(
                            child: Text(
                              'Drop a panel here',
                              style: TextStyle(color: EditorTheme.muted),
                            ),
                          )
                        : IndexedStack(
                            index: math.max(0, node.tabs.indexOf(node.active)),
                            children: [
                              for (final name in node.tabs)
                                widget.panelBuilder(name),
                            ],
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
