// A trackpad's two fingers, as Flutter hands them over: since Flutter 3.3 a trackpad's two-finger move and pinch come as
// pan/zoom events, not as a wheel's scroll (docs.flutter.dev/release/breaking-changes/trackpad-gestures). What they
// mean is Apple's: two fingers move what is shown, a pinch zooms it about the fingers (developer.apple.com/design/
// human-interface-guidelines/pointing-devices).
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

/// The hands that press and drag: a mouse (a trackpad's click and drag is one too), a finger, a pen. Not a trackpad's
/// two fingers, which a drag would otherwise take for one (GestureDetector.supportedDevices).
const wsPressing = {PointerDeviceKind.mouse, PointerDeviceKind.touch, PointerDeviceKind.stylus, PointerDeviceKind.invertedStylus, PointerDeviceKind.unknown};

/// [child] told of a trackpad's two fingers on it: [onPan] each move, by the way the fingers went (local points);
/// [onPinch] each change of their spread, by how many times, about where they are; [onPinchStep] +1 or -1 each time
/// the spread has opened or closed by a quarter (for a zoom of a few steps).
class WsTwoFingers extends StatefulWidget {
  const WsTwoFingers({super.key, this.onPan, this.onPinch, this.onPinchStep, required this.child});
  final void Function(Offset by, Offset at)? onPan;
  final void Function(double by, Offset at)? onPinch;
  final void Function(int step)? onPinchStep;
  final Widget child;

  @override
  State<WsTwoFingers> createState() => _WsTwoFingersState();
}

class _WsTwoFingersState extends State<WsTwoFingers> {
  double _scale = 1, _spread = 1; // (Flutter's scale is from where the fingers came down: each change is told as a ratio)

  @override
  Widget build(BuildContext context) => Listener(
    onPointerPanZoomStart: (_) => _scale = _spread = 1,
    onPointerPanZoomUpdate: (e) {
      if (e.scale != _scale) {
        widget.onPinch?.call(e.scale / _scale, e.localPosition);
        _spread *= e.scale / _scale;
        _scale = e.scale;
        if (widget.onPinchStep != null && (_spread > 1.25 || _spread < .8)) {
          widget.onPinchStep!(_spread > 1 ? 1 : -1);
          _spread = 1;
        }
      }
      if (e.localPanDelta != Offset.zero) widget.onPan?.call(e.localPanDelta, e.localPosition);
    },
    child: widget.child,
  );
}
