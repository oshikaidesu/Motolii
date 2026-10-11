// Pop 02: three sheets (Falloff, Echo/Trail, Blur), each 20 genuinely different pop GUI ideas for one concept, 5 x 4 panels so one
// screenshot of the canvas shows the whole sheet. Pop palette (not the calm Role tokens), flat vivid colour on dark panels, every panel drags.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'pop_02_kit.dart';
part 'pop_02_falloff.dart';
part 'pop_02_echo.dart';
part 'pop_02_blur.dart';

WidgetbookComponent pop02Set() => WidgetbookComponent(name: 'Pop 02', useCases: [
      WidgetbookUseCase(name: 'Falloff x20', builder: (c) => _Sheet(concept: 'Falloff', panels: _falloff)),
      WidgetbookUseCase(name: 'Echo/Trail x20', builder: (c) => _Sheet(concept: 'Echo', panels: _echo)),
      WidgetbookUseCase(name: 'Blur x20', builder: (c) => _Sheet(concept: 'Blur', panels: _blur)),
    ]);

/// For a render check only.
List<Widget> pop02Sheets() => [
      _Sheet(concept: 'Falloff', panels: _falloff),
      _Sheet(concept: 'Echo', panels: _echo),
      _Sheet(concept: 'Blur', panels: _blur),
    ];
