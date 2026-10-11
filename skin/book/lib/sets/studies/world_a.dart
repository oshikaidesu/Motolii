// World A: six small WORLDS (OP-1 style), proposals 1-6 of research/op1-translation.md: Falloff, Direction / Attract, Scatter, Stagger, Along Path, Scale distribution.
// Each is one Widgetbook use case: the 320 x 200 world and the same world in a 282 px Inspector row between two plain number rows (they share one value object).
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import 'world_a_kit.dart';
import 'world_a_w1.dart';
import 'world_a_w2.dart';
import 'world_a_w3.dart';

WaDriven _driven(BuildContext c) => c.knobs.object.dropdown<WaDriven>(label: 'Driven', options: WaDriven.values, initialOption: WaDriven.off, labelBuilder: (d) => d.name);

String _p(List<WaSpec> sp, int i, String name, [String? shown]) {
  final s = sp[i];
  return '$name ${waRange(s)}${s.unit.isEmpty ? '' : s.unit} (${shown ?? s.fmt(s.def)})';
}

WidgetbookUseCase _uc(String name, WaMake make, List<WaSpec> specs, double rowH, String caption) =>
    WidgetbookUseCase(name: name, builder: (c) => onGround(WaPair(make: make, specs: specs, rowHeight: rowH, mode: _driven(c), caption: caption), padding: const EdgeInsets.all(32)));

WidgetbookComponent worldASet() => WidgetbookComponent(name: 'World A', useCases: [
      _uc(
          '1 Falloff',
          FalloffWorld.new,
          falloffSpecs,
          108,
          'Falloff: A ${_p(falloffSpecs, 0, 'radius')} / B ${_p(falloffSpecs, 1, 'curve')} / C ${_p(falloffSpecs, 2, 'strength')} / D centre x,y 0-100 (50,50)'
          ' / silhouette: ripples on a pond with one falling drop'),
      _uc(
          '2 Direction / Attract',
          DirectionWorld.new,
          directionSpecs,
          96,
          'Direction / Attract: A ${_p(directionSpecs, 0, 'angle')} / B ${_p(directionSpecs, 1, 'pull')} / C ${_p(directionSpecs, 2, 'strength')} / D target x,y 0-100 (64,36)'
          ' / silhouette: grass leaning in the wind'),
      _uc(
          '3 Scatter',
          ScatterWorld.new,
          scatterSpecs,
          96,
          'Scatter: A ${_p(scatterSpecs, 0, 'spread')}px / B ${_p(scatterSpecs, 1, 'seed')} (click = next throw) / C ${_p(scatterSpecs, 2, 'density')} / D ${_p(scatterSpecs, 3, 'falloff')}'
          ' / silhouette: seeds thrown on the ground'),
      _uc(
          '4 Stagger',
          StaggerWorld.new,
          staggerSpecs,
          108,
          'Stagger: A ${_p(staggerSpecs, 0, 'offset')} / B ${_p(staggerSpecs, 1, 'range')} / C ${_p(staggerSpecs, 2, 'direction')} / D ${_p(staggerSpecs, 3, 'ease')}'
          ' / silhouette: a cascade of cards with a walking line'),
      _uc(
          '5 Along Path',
          AlongWorld.new,
          alongSpecs,
          84,
          'Along Path: A ${_p(alongSpecs, 0, 'start')} / B ${_p(alongSpecs, 1, 'end')} / C ${_p(alongSpecs, 2, 'spacing')} / D ${_p(alongSpecs, 3, 'bias')}'
          ' / silhouette: a necklace between two pennants'),
      _uc(
          '6 Scale distribution',
          ScaleWorld.new,
          scaleSpecs,
          84,
          'Scale distribution: A ${_p(scaleSpecs, 0, 'min')} / B ${_p(scaleSpecs, 1, 'max')} / C ${_p(scaleSpecs, 2, 'curve')} / D ${_p(scaleSpecs, 3, 'jitter')}'
          ' / silhouette: a stream of soap bubbles'),
    ]);
