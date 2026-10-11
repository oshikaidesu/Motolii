// set: OP 01. Three sheets (Glow, Scatter, Stagger), 20 panels each, in the OP-1 / Max for Live screen language:
// black ground, monoline drawings at 1-1.5 px, almost no fills, encoder colours (blue = parameter 1, green = 2, red = 3, white = the subject),
// large thin numerals as quiet readouts, tiny uppercase labels, whimsical line metaphors. The drawing is the control (no slider / knob / XY / bar).
// One Ticker per sheet. Every panel answers a drag; some read the drag as a flick, rub, spin, pinch or a drawn path. Double-tap = parameter 3 steps.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'op_01_kit.dart';
part 'op_01_glow.dart';
part 'op_01_scatter.dart';
part 'op_01_stagger.dart';

WidgetbookComponent op01Set() => WidgetbookComponent(name: 'OP 01', useCases: [
      WidgetbookUseCase(
          name: 'Glow x20',
          builder: (c) => const _Sheet(concept: 'GLOW', legend: [('RADIUS', _blue), ('INTENSITY', _green), ('THRESHOLD', _red)], specs: _glowSpecs)),
      WidgetbookUseCase(
          name: 'Scatter x20',
          builder: (c) => const _Sheet(concept: 'SCATTER', legend: [('AMOUNT', _blue), ('SPREAD', _green), ('SEED', _red)], specs: _scatterSpecs)),
      WidgetbookUseCase(
          name: 'Stagger x20',
          builder: (c) => const _Sheet(concept: 'STAGGER', legend: [('OFFSET', _blue), ('DIRECTION', _green), ('ORDER', _red)], specs: _staggerSpecs)),
    ]);
