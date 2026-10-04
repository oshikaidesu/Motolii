// OP 02: three sheets (Falloff, Echo/Trail, Blur), each 20 different ideas for one concept in the OP-1 / Max for Live screen
// language: black ground, monoline drawings, encoder colours, large thin numerals. The drawing is the control; every panel drags.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'op_02_kit.dart';
part 'op_02_falloff.dart';
part 'op_02_echo.dart';
part 'op_02_blur.dart';

WidgetbookComponent op02Set() => WidgetbookComponent(name: 'OP 02', useCases: [
      WidgetbookUseCase(name: 'Falloff x20', builder: (c) => _Sheet(concept: 'Falloff', panels: _falloff)),
      WidgetbookUseCase(name: 'Echo/Trail x20', builder: (c) => _Sheet(concept: 'Echo', panels: _echo)),
      WidgetbookUseCase(name: 'Blur x20', builder: (c) => _Sheet(concept: 'Blur', panels: _blur)),
    ]);

/// For a render check only.
List<Widget> op02Sheets() => [
      _Sheet(concept: 'Falloff', panels: _falloff),
      _Sheet(concept: 'Echo', panels: _echo),
      _Sheet(concept: 'Blur', panels: _blur),
    ];
