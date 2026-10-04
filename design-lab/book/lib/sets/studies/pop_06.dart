// Pop 06: two sheets of 20 pop panels each — Mask feather / reveal, Stroke / Trim paths.
// Each panel: drag right = first value, drag up = second value, tap = flip (invert / dash). One Ticker per sheet.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_06_kit.dart';
part 'pop_06_mask.dart';
part 'pop_06_stroke.dart';

WidgetbookComponent pop06Set() => WidgetbookComponent(name: 'Pop 06', useCases: [
      WidgetbookUseCase(name: 'Mask feather / reveal x20', builder: (c) => const Pop06Sheet(concept: 'Mask', panels: pop06MaskPanels)),
      WidgetbookUseCase(name: 'Stroke / Trim paths x20', builder: (c) => const Pop06Sheet(concept: 'Stroke', panels: pop06StrokePanels)),
    ]);
