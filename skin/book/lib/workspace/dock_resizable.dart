// A part of a shelf whose height the hand sets: drag its grip, double-click puts it back.
import 'package:flutter/gestures.dart' show DragStartBehavior;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_parts.dart';
import 'ws.dart';

/// A band whose height the user sets: the grip on its lower edge drags it up or down (clamped to [min]..[max]);
/// a double-click on the grip returns to the default.
class DockResizable extends StatefulWidget {
  const DockResizable({
    super.key,
    required this.height,
    required this.max,
    required this.onHeight,
    required this.child,
    this.min = 72,
    this.reset = dockFilterHeight,
    this.gripKey = const ValueKey('browser:filters-grip'),
  });

  /// [reset] is where a double-click on the grip puts the height back.
  final double height, min, max, reset;
  final Key gripKey;
  final ValueChanged<double> onHeight;
  final Widget child;
  @override
  State<DockResizable> createState() => _DockResizableState();
}

class _DockResizableState extends State<DockResizable> {
  final _box = GlobalKey();
  bool _hover = false, _drag = false;
  double _h = 0, _from = 0;

  double _clamp(double v) => v.clamp(widget.min, widget.max < widget.min ? widget.min : widget.max);

  @override
  Widget build(BuildContext context) {
    final lit = _hover || _drag;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(key: _box, height: _clamp(widget.height), child: widget.child),
        MouseRegion(
          cursor: SystemMouseCursors.resizeRow,
          onEnter: (_) => setState(() => _hover = true),
          onExit: (_) => setState(() => _hover = false),
          child: GestureDetector(
            key: widget.gripKey,
            behavior: HitTestBehavior.opaque,
            dragStartBehavior: DragStartBehavior.down,
            onVerticalDragStart: (d) => setState(() {
              _drag = true;
              _from = d.globalPosition.dy;
              _h = _box.currentContext?.size?.height ?? widget.height;
            }),
            onVerticalDragUpdate: (d) => widget.onHeight(_clamp(_h + d.globalPosition.dy - _from)),
            onVerticalDragEnd: (_) => setState(() => _drag = false),
            onDoubleTap: () => widget.onHeight(widget.reset),
            child: Container(
              height: 7,
              color: lit ? WsT.raised : WsT.ground,
              alignment: Alignment.center,
              child: Container(
                width: 28,
                height: 3,
                decoration: BoxDecoration(color: lit ? WsT.accent : Grey.g38, borderRadius: BorderRadius.circular(1.5)),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
