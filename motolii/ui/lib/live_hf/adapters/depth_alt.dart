// TEMPORARY — the Skin Swap Proof for the desks. A deliberately different skin over the same DepthHost: no floor
// plan, a table of rows; dragging a row moves that thing on the plan (right = +x, down = +z), dragging the camera row
// moves the eye. Its own pixel-to-world scale is this skin's business. If it works with no change to DepthHost /
// DepthController, the desk's meaning lives there. Delete after the proof.
import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show mono;
import '../../hf/desk/depth.dart';
import '../../hf/neutral.dart';

final altDepthSkin = ValueNotifier(false);

class AltDepth extends StatefulWidget {
  const AltDepth({super.key, required this.host});
  final DepthHost host;
  @override
  State<AltDepth> createState() => _AltDepthState();
}

class _AltDepthState extends State<AltDepth> {
  /// World units per pixel of drag in this table (the plan skin has its own).
  static const _unit = 4.0;
  Offset _moved = Offset.zero;

  Widget _row(int hit, String label, String value, {bool on = false}) => GestureDetector(
        key: ValueKey('alt-depth-$hit'),
        behavior: HitTestBehavior.opaque,
        onPanStart: (_) {
          _moved = Offset.zero;
          widget.host.press(hit);
        },
        onPanUpdate: (d) {
          _moved += d.delta * _unit;
          if (hit == -1) {
            final eye = widget.host.camera!;
            widget.host.drag(-1, eye.x + _moved.dx, eye.z + _moved.dy);
          } else {
            widget.host.drag(hit, _moved.dx, _moved.dy);
          }
        },
        onPanEnd: (_) => widget.host.release(),
        onPanCancel: () => widget.host.release(cancel: true),
        child: Container(
          height: 24,
          color: on ? N.g20 : null,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          child: Row(children: [
            Expanded(child: Text(label, style: mono(11, c: N.g95))),
            Text(value, style: mono(11, c: N.g63)),
          ]),
        ),
      );

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g00,
        child: ListenableBuilder(
          listenable: widget.host,
          builder: (context, _) {
            final h = widget.host;
            return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              if (h.camera case final eye? when h.cameraSelectable) _row(-1, '[camera]', '${eye.x.round()}, ${eye.z.round()}'),
              for (final (i, item) in h.items.indexed) _row(i, item.locked ? '${item.name} (locked)' : item.name, '${item.x.round()}, ${item.z.round()}', on: item.selected),
            ]);
          },
        ),
      );
}
