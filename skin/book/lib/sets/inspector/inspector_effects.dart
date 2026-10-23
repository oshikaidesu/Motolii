import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects_effect_card.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

import '../../tokens.dart';
import 'inspector_motion_preview.dart' show CardOver;
import 'inspector_parts.dart';

// ---- 10. effects ------------------------------------------------------------------------------------------------------
/// An effect as a plugin ships it: a name and a WGSL uniform struct. Its rows are whatever [readParams] makes of the struct.
class Fx {
  Fx(this.id, this.name, this.wgsl, {Color? tone, Color Function()? toneOf, required this.mark, this.vendor}) : _tone = tone, _toneOf = toneOf;
  final String id, name, wgsl;
  final String? vendor;
  final Color? _tone;
  final Color Function()? _toneOf; // (a skin's colour: read when drawn, so a card made before the skin changed wears the new one)
  Color get tone => _toneOf?.call() ?? _tone!;
  final CardMark mark;
  late final List<Decl> decls = parseWgsl(wgsl);
  late final List<Param> params = [for (final d in decls) infer(d)];
}


class EffectsStack extends StatelessWidget {
  const EffectsStack({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final cards = [
      for (var i = 0; i < doc.fxOrder.length; i++) EffectCard(doc.fxDefs.firstWhere((f) => f.id == doc.fxOrder[i]), key: ValueKey(doc.fxOrder[i]), first: i == 0),
    ];
    final none =
        doc.query.isNotEmpty &&
        !doc.fxOrder.any((id) => doc.fxDefs.firstWhere((f) => f.id == id).params.any((p) => p.label.toLowerCase().contains(doc.query.toLowerCase())));
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (x.cfg.maybe) const ParamSearch(),
        if (none)
          Padding(
            padding: EdgeInsets.symmetric(vertical: Shape.s(8)),
            child: Text('No parameter matches', style: T.label(Grey.g56)),
          )
        else
          ...cards,
      ],
    );
  }
}

/// H3 (maybe): search flattens the groups, unfolds Advanced, and says so when nothing matches.
class ParamSearch extends StatefulWidget {
  const ParamSearch({super.key});
  @override
  State<ParamSearch> createState() => _ParamSearchState();
}

class _ParamSearchState extends State<ParamSearch> {
  final _c = TextEditingController();
  final _f = FocusNode(debugLabel: 'param-search');
  @override
  void dispose() {
    _c.dispose();
    _f.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed);
    return Container(
      height: cellH,
      margin: EdgeInsets.only(bottom: Shape.s(4)),
      padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
      alignment: Alignment.centerLeft,
      decoration: BoxDecoration(
        color: Grey.g13,
        borderRadius: BorderRadius.all(Shape.r(4)),
        border: Border.all(color: ed.rule),
      ),
      child: Stack(
        alignment: Alignment.centerLeft,
        children: [
          if (_c.text.isEmpty) Text('Search parameters', style: T.label(Grey.g56)),
          EditableText(
            controller: _c,
            focusNode: _f,
            style: T.name(Grey.g95),
            cursorColor: Grey.g95,
            backgroundCursorColor: Grey.g20,
            maxLines: 1,
            onChanged: (s) {
              x.doc.query = s;
              x.doc.poke();
              setState(() {});
            },
          ),
        ],
      ),
    );
  }
}

/// A card of the host's own values, in the effects' manner: its Params, each field a value (keyed `v_` and its name).
class ValuesCard extends StatelessWidget {
  ValuesCard(this.card, {super.key, this.top, this.side});
  final Fx card;
  final Widget? top; // (a drawing of what the values make, above them: the window's own for a card it knows)
  final Widget? side; // (or a square one beside the first of them, Good Boy Ninja's Wiggle's way: its tiles to its right, the rest under both)
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), k = 'v.${card.name}', adv = doc.b['$k.adv'] ?? false;
    final heroes = [for (final p in card.params) if (!p.advanced) p], rest = [for (final p in card.params) if (p.advanced) p];
    // (the first row, then the rest folded under the name of what they are; the fold says nothing more than its glyph)
    final at = doc.movable.indexOf(card.name);   // (one of its effects: up or down its stack, After Effects' dragging a card)
    final sect = Sect(
      title: card.name,
      tone: card.tone,
      mark: card.mark,
      trailing: at < 0 || doc.onCardMove == null
          ? null
          : Row(mainAxisSize: MainAxisSize.min, children: [
              for (final (by, up) in const [(-1, true), (1, false)])
                Hov(
                  key: ValueKey('card-move:${card.name}:$by'),
                  onTap: (up ? at > 0 : at < doc.movable.length - 1) ? () => doc.onCardMove!(card.name, by) : null,
                  builder: (_, h) => SizedBox(
                    width: Shape.z(16),
                    height: Shape.z(16),
                    child: CustomPaint(painter: _Chevron(up, (up ? at > 0 : at < doc.movable.length - 1) ? (h ? Grey.g95 : Grey.g56) : Grey.g26)),
                  ),
                ),
            ]),
      children: [
        ?top,
        if (side case final square?)
          LayoutBuilder(
            builder: (context, box) {
              final g = Pop.tileGap, edge = Pop.tile * 3 + g * 2; // (the square as high as three rows of tiles)
              final right = box.maxWidth - edge - g, cols = ((right + g) / (Pop.tileMin + g)).floor();
              if (cols < 1) return Column(children: [SizedBox(height: edge, child: square), SizedBox(height: g), ParamGrid('v', heroes)]); // (too narrow: above them)
              var n = 0, room = cols * 3; // (as many tiles as three rows beside it hold)
              for (final p in heroes) {
                room -= math.min(ParamGrid.span(p), cols);
                if (room < 0) break;
                n++;
              }
              return Column(
                children: [
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      SizedBox.square(dimension: edge, child: square),
                      SizedBox(width: g),
                      Expanded(child: ParamGrid('v', heroes.take(n).toList())),
                    ],
                  ),
                  if (n < heroes.length) ...[SizedBox(height: g), ParamGrid('v', heroes.skip(n).toList())],
                ],
              );
            },
          )
        else
          ParamGrid('v', heroes),
        if (rest.isNotEmpty) ...[
          Padding(
            padding: EdgeInsets.only(top: Shape.s(4)),
            child: Hov(
              onTap: () => doc.flag('$k.adv', !adv),
              builder: (_, h) => Container(
                height: Shape.z(24),
                decoration: BoxDecoration(border: Border(top: BorderSide(color: ed.rule))),
                child: Row(
                  children: [
                    Text(rest.first.fold ?? 'Advanced', style: ed.labStyle(h || adv ? Grey.g95 : Grey.g63)),
                    const Spacer(),
                    FoldMark(open: adv, hot: h || adv),
                  ],
                ),
              ),
            ),
          ),
          if (adv) ParamGrid('v', rest),
        ],
      ],
    );
    return side == null ? sect : CardOver(child: sect); // (a hand anywhere on the card plays its preview)
  }
}

/// A small chevron, up or down: a card moved up or down its stack.
class _Chevron extends CustomPainter {
  const _Chevron(this.up, this.ink);
  final bool up;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final y0 = s.height * (up ? .62 : .38), y1 = s.height * (up ? .38 : .62);
    c.drawPath(
      Path()
        ..moveTo(s.width * .3, y0)
        ..lineTo(s.width * .5, y1)
        ..lineTo(s.width * .7, y0),
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.4
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(_Chevron o) => o.up != up || o.ink != ink;
}
