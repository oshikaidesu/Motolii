// Leading as three lines whose gap follows the value (Auto draws the font's own gap).
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

class ParagraphCard extends StatelessWidget {
  const ParagraphCard({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    doc.seed({
      ...taDefaults([..._paraMain, ..._paraMore], 'para'),
      'para_justify': 0,
      'para_leading': 0,
    });
    double g(String id) => doc.get(id) ?? 0;
    final j = g('para_justify').round().clamp(0, 2), lead = g('para_leading'), trim = g('para_trim').round().clamp(0, 3);
    final hang = g('para_hangFirst') > .5 || g('para_hangLast') > .5 || g('para_hangEnd') > .5;
    final open = doc.b['para.more'] ?? false;
    return Sect(
      title: 'Paragraph',
      tone: Pop.toneTextAnim,
      mark: CardMark.text,
      keyIds: const ['para_indent', 'para_before', 'para_after', 'para_leading'],
      brief: [
        _paraJustify[j],
        lead == 0 ? 'Leading auto' : 'Leading ${brief(lead, 0)} px',
        if (g('para_indent') != 0) 'Indent ${brief(g('para_indent'), 0)}',
        if (trim != 0) _paraTrim[trim],
        if (hang) 'Hanging',
      ],
      children: [
        TaGrid([
          (2, TaBlock(label: 'Justify', child: _ParaJustify(j))),
          taTile('para', _paraMain[0]),
          taTile('para', _paraMain[1]),
          taTile('para', _paraMain[2]),
          (
            1,
            RowInfo(
              unit: null,
              keys: doc.keys['para_leading'] ?? KeyS.off,
              child: NumField(
                id: 'para_leading',
                caption: 'Leading',
                unit: 'px',
                min: 0,
                max: 400,
                perPx: .5,
                zeroWord: 'Auto',
                enabled: !x.cfg.locked,
                glyph: CustomPaint(size: const Size(22, 22), painter: _ParaLeadPaint(lead)),
              ),
            ),
          ),
        ]),
        TaFold('para.more', 'More options', _paraMore.length),
        if (open) TaGrid([for (final p in _paraMore) taTile('para', p)]),
      ],
    );
  }
}

const _paraMore = [
  paraGroupP,
  Param(id: 'anchor', label: 'Anchor offset', look: ParamLook.point, type: WType.vec2, min: -500, max: 500, unit: 'px'),
  Param(id: 'autospace', label: 'Autospace', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1, def: 1),
  Param(id: 'hangFirst', label: 'Hang first', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1),
  Param(id: 'hangLast', label: 'Hang last', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1),
  Param(id: 'trim', label: 'Spacing trim', look: ParamLook.choice, type: WType.u32, min: 0, max: 3, options: _paraTrim),
  Param(id: 'hangEnd', label: 'Hang end', look: ParamLook.choice, type: WType.u32, min: 0, max: 2, options: _paraHangEnd),
];

const _paraHangEnd = ['None', 'Allow', 'Force'];
const _paraTrim = ['Normal', 'Space all', 'Space first', 'Trim start'];

const _paraMain = [
  Param(id: 'indent', label: 'First indent', look: ParamLook.horizontal, type: WType.f32, min: -200, max: 200, unit: 'px'),
  Param(id: 'before', label: 'Space before', look: ParamLook.vertical, type: WType.f32, min: 0, max: 200, unit: 'px', upIncreases: false),
  Param(id: 'after', label: 'Space after', look: ParamLook.vertical, type: WType.f32, min: 0, max: 200, unit: 'px'),
];

/// Leading as three lines whose gap follows the value (Auto draws the font's own gap).
class _ParaLeadPaint extends CustomPainter {
  const _ParaLeadPaint(this.lead);
  final double lead;
  @override
  void paint(Canvas c, Size s) {
    final gap = lead == 0 ? 5.0 : (lead / 6).clamp(2.0, 8.0), o = s.center(Offset.zero);
    final p = Paint()
      ..color = lead == 0 ? Grey.g56 : Grey.g91
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round;
    for (var i = -1; i <= 1; i++) {
      c.drawLine(Offset(o.dx - 8, o.dy + i * gap), Offset(o.dx + 8, o.dy + i * gap), p);
    }
  }

  @override
  bool shouldRepaint(_ParaLeadPaint o) => o.lead != lead;
}

/// Justify as three tiny paragraphs (ragged right, centred, ragged left) in the lab's segmented style.
class _ParaJustify extends StatelessWidget {
  const _ParaJustify(this.index);
  final int index;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return Dis(
      child: Container(
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: Grey.g07, borderRadius: BorderRadius.circular(7)),
        child: Row(
          children: [
            for (var i = 0; i < _paraJustify.length; i++)
              Expanded(
                child: Semantics(
                  button: true,
                  selected: i == index,
                  label: _paraJustify[i],
                  child: GestureDetector(
                    key: ValueKey('para_justify_$i'),
                    behavior: HitTestBehavior.opaque,
                    onTap: () => doc.set('para_justify', i.toDouble()),
                    child: AnimatedContainer(
                      duration: const Duration(milliseconds: 120),
                      alignment: Alignment.center,
                      decoration: BoxDecoration(
                        color: i == index ? Grey.g20 : null,
                        borderRadius: BorderRadius.circular(5),
                        border: Border(bottom: BorderSide(color: i == index ? Role.selected : const Color(0x00000000))),
                      ),
                      child: CustomPaint(size: const Size(14, 10), painter: ParaJustPaint(i, i == index)),
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

const _paraJustify = ['Left', 'Center', 'Right'];
