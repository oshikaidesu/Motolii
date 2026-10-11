// set: Phenomenon E. A parameter becomes a miniature of what it means: many numbers -> one meaning (a face, a bank of moods, a wire, a comet, a top, dough).
// Each miniature IS the phenomenon: a drag acts on it the way the thing would behave; the numbers are small readouts at its edge.
// Contracts kept: Esc during a drag restores, Shift = fine, double-click resets, one gesture = one undo step (Cmd/Ctrl+Z, local list), hit areas >= 24 px.
// Every miniature is a PhenoModel (pheno_e_core.dart): named parameters with range / default / unit in `params`, read and written through `v` / `set`.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../tokens.dart';

part 'pheno_e_core.dart';
part 'pheno_e_face.dart';
part 'pheno_e_wire.dart';
part 'pheno_e_motion.dart';

WidgetbookComponent phenoESet() => WidgetbookComponent(name: 'Phenomenon E', useCases: [
      _phenoCase('1 Macro: mood', 'Macro: Blur + Glow + Noise + Sharpen + Saturation + Contrast + Chroma shift + Vignette | silhouette: a small round face',
          MacroModel.new, const Size(258, 120), ('Opacity', '100 %'), ('Rotation', '0.0°')),
      _phenoCase('2 Variations: moods', 'Variations: the same eight numbers, kept as a bank of eight faces (from, to, mix) | silhouette: one big face beside a 4 x 2 shelf of tiny faces',
          VariationsModel.new, const Size(258, 120), ('Opacity', '100 %'), ('Rotation', '0.0°')),
      _phenoCase('3 React-to: wire', 'React-to: input low/high (velocity) + output low/high (glow %) + curve | silhouette: two small orbs joined by a sagging wire',
          ReactModel.new, const Size(258, 96), ('Position X', '960'), ('Opacity', '100 %')),
      _phenoCase('4 Echo: comet', 'Echo: feedback (echoes) + spacing (frames) + decay (%) | silhouette: a dot on a figure-eight path dragging a trail of fading beads',
          EchoModel.new, const Size(258, 96), ('Position X', '960'), ('Opacity', '100 %')),
      _phenoCase('5 Spin: top', 'Spin: speed (deg/s, sign = direction) + friction (%) | silhouette: a striped spinning top standing on a grainy floor',
          SpinModel.new, const Size(258, 120), ('Position Y', '540'), ('Opacity', '100 %')),
      _phenoCase('6 Scale: dough', 'Scale: x + y + link | silhouette: a speckled lump of dough with corner brackets',
          DoughModel.new, const Size(258, 108), ('Rotation', '0.0°'), ('Opacity', '100 %')),
    ]);

WidgetbookUseCase _phenoCase(String name, String caption, PhenoModel Function() make, Size row, (String, String) above, (String, String) below) =>
    WidgetbookUseCase(name: name, builder: (c) => onGround(_PhenoCase(caption: caption, make: make, row: row, above: above, below: below), padding: const EdgeInsets.all(24)));

class _PhenoCase extends StatefulWidget {
  const _PhenoCase({required this.caption, required this.make, required this.row, required this.above, required this.below});
  final String caption;
  final PhenoModel Function() make;
  final Size row;
  final (String, String) above, below;
  @override
  State<_PhenoCase> createState() => _PhenoCaseState();
}

class _PhenoCaseState extends State<_PhenoCase> {
  late final PhenoModel m = widget.make();

  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  Widget _label(String t) => Padding(padding: const EdgeInsets.only(bottom: 6), child: Text(t, style: T.label(N.g63)));

  Widget _numRow((String, String) r) => SizedBox(
        height: 28,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: Row(children: [
            SizedBox(width: 96, child: Text(r.$1, style: T.label(N.g76))),
            Expanded(
              child: Container(
                height: 22,
                alignment: Alignment.centerRight,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g20)),
                child: Text(r.$2, style: T.value(N.g91)),
              ),
            ),
          ]),
        ),
      );

  @override
  Widget build(BuildContext context) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        // the caption lives outside the frames: name, merged parameters, silhouette
        SizedBox(width: 640, child: Text(widget.caption, style: T.label(N.g76), maxLines: 3)),
        const SizedBox(height: 14),
        Wrap(spacing: 28, runSpacing: 20, crossAxisAlignment: WrapCrossAlignment.start, children: [
          Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
            _label('large 320 x 200'),
            PhenoView(model: m, size: const Size(320, 200)),
          ]),
          Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
            _label('in a 282 px Inspector row'),
            Container(
              width: 282,
              padding: const EdgeInsets.symmetric(vertical: 6),
              decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
              child: Column(mainAxisSize: MainAxisSize.min, children: [
                _numRow(widget.above),
                Padding(padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4), child: PhenoView(model: m, size: widget.row)),
                _numRow(widget.below),
              ]),
            ),
          ]),
        ]),
      ]);
}
