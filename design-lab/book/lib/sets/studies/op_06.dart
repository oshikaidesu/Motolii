// OP 06: two sheets of 20 panels in the OP-1 / Max for Live language — black ground, monoline drawings, one colour per parameter.
// Mask: softness = blue, expand = green, invert = red-magenta, the masked subject = white.
// Stroke: width = blue, start = green, end = white, dash = red-magenta.
// Every panel answers a drag; some read it as a flick, rub, pinch, spin or a drawn line. Tap flips the toggle (invert / dash on).
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'op_06_kit.dart';
part 'op_06_mask.dart';
part 'op_06_stroke.dart';

WidgetbookComponent op06Set() => WidgetbookComponent(name: 'OP 06', useCases: [
      WidgetbookUseCase(name: 'Mask feather / reveal x20', builder: (c) => const Op06Sheet(concept: 'mask', panels: op06MaskPanels)),
      WidgetbookUseCase(name: 'Stroke / Trim paths x20', builder: (c) => const Op06Sheet(concept: 'stroke', panels: op06StrokePanels)),
    ]);
