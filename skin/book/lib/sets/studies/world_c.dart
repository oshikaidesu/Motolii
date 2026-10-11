// World C: per-Effect small WORLDS (proposals 13-18 of research/op1-translation.md). Each world is one box that is the whole effect, with up to four
// grab zones in the OP-1 colour slots (A blue, B green, C white, D orange). Use cases: the large world (320 x 200) and the same world in a 282 px
// Inspector row between two plain number rows. The 'Driven' knob (Off | Wiggle | Pulse) moves the parameters from outside so the picture is seen moving.
import 'package:widgetbook/widgetbook.dart';

import 'world_c_a.dart';
import 'world_c_b.dart';
import 'world_c_kit.dart';

WidgetbookComponent worldCSet() {
  final defs = [blurDef, glowDef, echoDef, waveDef, noiseDef, shadowDef];
  return WidgetbookComponent(name: 'World C', useCases: [
    for (final d in defs) worldLarge(d),
    for (final d in defs) worldRow(d),
  ]);
}
