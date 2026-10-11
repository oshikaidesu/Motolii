// OP 03: three sheets of 20 OP-1 style miniatures (Noise, Warp / Distort, Shadow).
// Black screen, monoline drawings, encoder colours: blue = 1st parameter, green = 2nd, red = 3rd.
// One Ticker per sheet; every panel reads a tiny shared drag state.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

part 'op_03_kit.dart';
part 'op_03_noise.dart';
part 'op_03_warp.dart';
part 'op_03_shadow.dart';

WidgetbookComponent op03Set() => WidgetbookComponent(name: 'OP 03', useCases: [
      WidgetbookUseCase(name: 'Noise x20', builder: (_) => const _OpSheet(concept: 'Noise', panels: _noisePanels)),
      WidgetbookUseCase(name: 'Warp / Distort x20', builder: (_) => const _OpSheet(concept: 'Warp', panels: _warpPanels)),
      WidgetbookUseCase(name: 'Shadow x20', builder: (_) => const _OpSheet(concept: 'Shadow', panels: _shadowPanels)),
    ]);
