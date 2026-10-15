// The outline blend, pictured: the sample's weight and spacing slide from this layer's toward the target's as Amount rises.
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_tile.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import 'package:lab_book/tokens.dart';

class TextMorphCard extends StatelessWidget {
  const TextMorphCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(paramDefaults('tm', _tmAmount), strs: const {'tm_target': 'None'});
    final target = doc.s2['tm_target'] ?? 'None', has = target != 'None', t = ((doc.get('tm_amount') ?? 0) / 100).clamp(0.0, 1.0);
    return Sect(
      title: 'Text Morph',
      tone: Pop.toneTextAnim,
      mark: CardMark.textAnim,
      keyIds: const ['tm_amount'],
      brief: [has ? 'Into $target' : 'No target', '${brief(doc.get('tm_amount'), 0)}%'],
      children: [
        if (has)
          Container(
            height: 34,
            margin: const EdgeInsets.only(bottom: 2),
            decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
            child: CustomPaint(painter: _TmPaint(taWord(doc), t)),
          ),
        TaGrid([
          (
            2,
            const TaBlock(
              label: 'Target',
              child: Chooser(id: 'tm_target', options: _tmTargets),
            ),
          ),
          (1, RowOff(off: !has, narrow: false, child: const ParamTile('tm', _tmAmount))),
        ]),
        if (!has) const Cap('Pick a text layer to morph into; Amount waits until then.'),
      ],
    );
  }
}

const _tmTargets = ['None', 'Hero title', 'Caption', 'Lower third', 'Credits'];
const _tmAmount = Param(id: 'amount', label: 'Amount', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%');

/// The outline blend, pictured: the sample's weight and spacing slide from this layer's toward the target's as Amount rises.
class _TmPaint extends CustomPainter {
  const _TmPaint(this.word, this.t);
  final String word;
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final style = T
        .title(Color.lerp(Grey.g76, Pop.toneTextAnim, t)!)
        .copyWith(
          fontSize: 16,
          height: 1,
          fontWeight: FontWeight.lerp(FontWeight.w300, FontWeight.w900, t),
          letterSpacing: 2.5 * t,
          fontStyle: t > .5 ? FontStyle.italic : FontStyle.normal,
        );
    final tp = TextPainter(
      text: TextSpan(text: word, style: style),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout(maxWidth: s.width - 16);
    tp.paint(c, s.center(Offset.zero) - Offset(tp.width / 2, tp.height / 2));
  }

  @override
  bool shouldRepaint(_TmPaint o) => o.word != word || o.t != t;
}
