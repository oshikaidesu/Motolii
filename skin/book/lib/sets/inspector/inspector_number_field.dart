import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_number_field_num_field_state.dart';

import '../../tokens.dart';

TextStyle blockValue(Color c) => T.value(c).copyWith(fontSize: T.px(14), fontWeight: FontWeight.w600, height: 1.1);

class NumField extends StatefulWidget {
  const NumField({
    super.key,
    required this.id,
    this.label,
    this.unit = '',
    this.decimals = 0,
    this.min,
    this.max,
    this.step = 1,
    this.perPx = 1,
    this.zeroWord,
    this.enabled = true,
    this.axis,
    this.turns = false,
    this.onSet,
    this.onReset,
    this.bipolar = false,
    this.caption,
    this.glyph,
    this.glyphOwnsPointer = false,
    this.glyphTall = false,
  });
  final String id;
  final String? label, unit, zeroWord;
  final int decimals;
  final double? min, max;
  final double step, perPx;
  final bool enabled, turns;
  final Color? axis;
  final ValueChanged<double>? onSet;
  final VoidCallback? onReset;

  /// A range around zero fills out from the middle.
  final bool bipolar;

  /// Block form: [caption] on top, the value large under it, [glyph] (a picture of the value) in the top-right corner.
  /// With [glyphOwnsPointer] a press on the glyph is the glyph's (a dial, a die), not a scrub.
  /// contract: every number in the Inspector scrubs sideways, whatever it means (a Y, a height); only a glyph may move otherwise.
  final String? caption;
  final Widget? glyph;
  final bool glyphOwnsPointer;

  /// The glyph runs the block's full height on the right (stacked buttons) instead of sitting in the corner.
  final bool glyphTall;
  @override
  State<NumField> createState() => NumFieldState();
}


/// The field's claim on a press that scrubs it: taken at once, before a scroll round it can (Flutter's EagerGestureRecognizer, asked first).
class ScrubClaim extends OneSequenceGestureRecognizer {
  bool Function(PointerDownEvent) takes = (_) => false;
  @override
  void addAllowedPointer(PointerDownEvent event) {
    if (!takes(event)) return;
    super.addAllowedPointer(event);
    resolve(GestureDisposition.accepted);
    stopTrackingPointer(event.pointer);
  }

  @override
  String get debugDescription => 'scrub';
  @override
  void didStopTrackingLastPointer(int pointer) {}
  @override
  void handleEvent(PointerEvent event) {}
}
