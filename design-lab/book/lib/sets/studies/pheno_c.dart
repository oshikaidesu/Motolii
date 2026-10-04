// Phenomenon C: material and surface. Six parameters turned into miniatures of what they mean (roughness, noise, warp, extrude, stroke, fog).
// Each miniature is a toy over a PcDoc (named, ranged values: the later integration reads and writes those); the number is a quiet readout, never the control.
// Parts: _core (value object, toy base, host, painter, row frame), then one file per miniature.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart' show mapEquals;
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../tokens.dart';

part 'pheno_c_core.dart';
part 'pheno_c_rough.dart';
part 'pheno_c_noise.dart';
part 'pheno_c_warp.dart';
part 'pheno_c_extrude.dart';
part 'pheno_c_stroke.dart';
part 'pheno_c_fog.dart';

WidgetbookComponent phenoCSet() => WidgetbookComponent(name: 'Phenomenon C', useCases: [
      _case(
        'Roughness: a bead you rub',
        'Roughness | roughness 0-1 | silhouette: a glossy bead that frosts over, light fanning off it',
        _RoughToy.new,
        label: 'Roughness',
        rowH: 96,
        above: ('Metallic', '0.00'),
        below: ('Specular', '0.50'),
      ),
      _case(
        'Noise: a ridge you pinch, pull, slide',
        'Noise | frequency + amplitude + evolution | silhouette: a ridge line over layered contours, gravel below',
        _NoiseToy.new,
        label: 'Noise',
        rowH: 84,
        above: ('Seed', '1204'),
        below: ('Scale', '100.0'),
      ),
      _case(
        'Warp: a grid you grab',
        'Warp | strength + radius | silhouette: a rubber grid with a bulge in it',
        _WarpToy.new,
        label: 'Warp',
        rowH: 112,
        above: ('Center X', '960'),
        below: ('Anchor', '0.50'),
      ),
      _case(
        'Extrude: a block you pull up',
        'Extrude | depth + bevel | silhouette: a hex block rising out of its flat footprint',
        _ExtrudeToy.new,
        label: 'Extrude',
        rowH: 116,
        above: ('Position Z', '0.0'),
        below: ('Light', '45°'),
      ),
      _case(
        'Stroke: a pen stroke you press and tap',
        'Stroke | width + dash + cap | silhouette: one fat brush stroke, cut into dashes',
        _StrokeToy.new,
        label: 'Stroke',
        rowH: 96,
        above: ('Color', '#E7E7E7'),
        below: ('Miter', '4.0'),
      ),
      _case(
        'Opacity: a pane you fog and wipe',
        'Opacity | opacity 0-100% | silhouette: a small cat behind a misted pane',
        _FogToy.new,
        label: 'Opacity',
        rowH: 96,
        above: ('Blend', 'Normal'),
        below: ('Mask', 'None'),
      ),
    ]);

WidgetbookUseCase _case(String name, String caption, _Toy Function() make, {required String label, required double rowH, required (String, String) above, required (String, String) below}) =>
    uc(name, (c) => _Story(make: make, caption: caption, label: label, rowH: rowH, above: above, below: below), width: 640);
