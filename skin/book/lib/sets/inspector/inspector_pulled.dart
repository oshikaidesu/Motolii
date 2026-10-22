// A card's drawing a hand pulls (the shadow, the trim, the light, the copies): a press on one of its knobs is the
// drawing's at once, before the inspector's scroll can take it (the number field's own claim, ScrubClaim); then each
// move and the letting go are told as they come.
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_number_field.dart' show ScrubClaim;

class Pulled extends StatefulWidget {
  const Pulled({super.key, required this.takes, required this.onDown, required this.onMove, required this.onUp, required this.child});

  /// Whether a press at this point is on something to pull.
  final bool Function(Offset at) takes;
  final void Function(Offset at) onDown, onMove;
  final VoidCallback onUp;
  final Widget child;

  @override
  State<Pulled> createState() => _PulledState();
}

class _PulledState extends State<Pulled> {
  bool _held = false;

  @override
  Widget build(BuildContext context) => RawGestureDetector(
    gestures: {ScrubClaim: GestureRecognizerFactoryWithHandlers<ScrubClaim>(ScrubClaim.new, (r) => r.takes = (e) => widget.takes(e.localPosition))},
    child: Listener(
      onPointerDown: (e) {
        if (e.buttons != kPrimaryButton || !widget.takes(e.localPosition)) return;
        _held = true;
        widget.onDown(e.localPosition);
      },
      onPointerMove: (e) => _held ? widget.onMove(e.localPosition) : null,
      onPointerUp: (_) => _up(),
      onPointerCancel: (_) => _up(),
      child: widget.child,
    ),
  );

  void _up() {
    if (!_held) return;
    _held = false;
    widget.onUp();
  }
}
