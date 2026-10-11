// World E: the per-Effect / Relation worlds 25-30 of research/op1-translation.md (Easing, Spring, Wiggle, Time remap, Loop, Camera).
// Each is a small scene with up to four grab zones in the four colour slots (A blue, B green, C white, D orange) and a 'Driven' knob (Off | Wiggle | Pulse).
import 'package:widgetbook/widgetbook.dart';

import 'world_e_a.dart';
import 'world_e_b.dart';
import 'world_e_kit.dart';

WidgetbookComponent worldESet() => WidgetbookComponent(name: 'World E', useCases: [
      for (final d in [easingDef(), springDef(), wiggleDef(), remapDef(), loopDef(), cameraDef()]) weUseCase(d),
    ]);
