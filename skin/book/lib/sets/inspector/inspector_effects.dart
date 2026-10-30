import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects_effect_card.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

import '../../tokens.dart';
import 'inspector_motion_preview.dart' show CardOver;
import 'inspector_parts.dart';
import '../panels/timeline_parts_parts.dart' show TpGlyph, TpGlyphView;
import '../foundations/text_hand.dart';

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
          TextHand(field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
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
          )),
        ],
      ),
    );
  }
}

/// A thing's own card's head: its values copied, given to another, as they come (After Effects' Reset).
class CardWords extends StatelessWidget {
  const CardWords(this.name, this.ids, {super.key});
  final String name;
  final List<String> ids;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return Row(mainAxisSize: MainAxisSize.min, children: [
      for (final (word, does) in [
        if (doc.onCardCopy case final copy?) ('Copy', () => copy(ids)),
        if (doc.onCardPaste case final paste?) ('Paste', paste),
        if (doc.onCardReset case final reset?) ('Reset', () => reset(ids)),
      ])
        Hov(
          key: ValueKey('card-${word.toLowerCase()}:$name'),
          onTap: does,
          builder: (_, h) => Padding(padding: EdgeInsets.only(right: Shape.s(6)), child: Text(word, style: T.label(h ? Grey.g95 : Grey.g56))),
        ),
    ]);
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
      trailing: (at < 0 || doc.onCardMove == null) && doc.onCardReset == null && doc.onCardCopy == null
          ? null
          : Row(mainAxisSize: MainAxisSize.min, children: [
              if (at < 0) CardWords(card.name, [for (final p in card.params) p.id]),
              if (at >= 0) ...[   // (an effect's card, as AbsentMotion's EffectHeader and Alight Motion's: its eye, ⋮ with the rest, the bin)
                if (doc.onCardEye != null)
                  Hov(
                    key: ValueKey('card-${doc.cardsOff.contains(card.name) ? 'show' : 'hide'}:${card.name}'),
                    onTap: () => doc.onCardEye!(card.name),
                    builder: (_, h) => Padding(padding: EdgeInsets.only(right: Shape.s(6)), child: TpGlyphView(doc.cardsOff.contains(card.name) ? TpGlyph.eyeOff : TpGlyph.eye, h ? Grey.g95 : Grey.g56)),
                  ),
                _CardMore(name: card.name, items: [   // (AbsentMotion's More: Effect Info, Reset to Defaults, Copy Effect, Paste Effect Values; Alight Motion's Duplicate; and a preset of it kept)
                  if (doc.onCardInfo case final info?) ('info', 'Effect Info', () => info(card.name)),
                  if (doc.onCardReset case final reset?) ('reset', 'Reset to Defaults', () => reset([for (final p in card.params) p.id])),
                  if (doc.onCardDuplicate case final duplicate?) ('duplicate', 'Duplicate', () => duplicate(card.name)),
                  if (doc.onCardCopy case final copy?) ('copy', 'Copy Effect', () => copy([for (final p in card.params) p.id])),
                  if (doc.onCardPaste case final paste?) ('paste', 'Paste Effect Values', paste),
                  if (doc.onCardSave case final save?) ('save', 'Save as Preset', () => save(card.name, [for (final p in card.params) p.id])),
                ]),
                if (doc.onCardRemove != null)
                  Hov(
                    key: ValueKey('card-remove:${card.name}'),
                    onTap: () => doc.onCardRemove!(card.name),
                    builder: (_, h) => Padding(padding: EdgeInsets.only(right: Shape.s(6)), child: CustomPaint(size: Size.square(Shape.z(12)), painter: _Bin(h ? Grey.g95 : Grey.g56))),
                  ),
              ],
              if (at >= 0 && doc.onCardMove != null && doc.movable.length > 1)   // (Alight Motion's ≡ on an effect's row, After Effects' dragging an effect in Effect Controls: held and dragged up or down its stack)
                Draggable<String>(
                  key: ValueKey('card-drag:${card.name}'),
                  data: card.name,
                  axis: Axis.vertical,
                  feedback: FxDragGhost(card),
                  child: MouseRegion(
                    cursor: SystemMouseCursors.grab,
                    child: SizedBox(width: Shape.z(16), height: Shape.z(16), child: CustomPaint(painter: _Grip(Grey.g56))),
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
    final shown = side == null ? sect : CardOver(child: sect); // (a hand anywhere on the card plays its preview)
    if (at < 0 || doc.onCardMove == null) return shown;
    return DragTarget<String>(   // (an effect let go on this one's card takes its place; the line says on which side it lands, as the stack's own EffectCard)
      onWillAcceptWithDetails: (d) => d.data != card.name && doc.movable.contains(d.data),
      onAcceptWithDetails: (d) => doc.onCardMove!(d.data, at - doc.movable.indexOf(d.data)),
      builder: (context, held, _) {
        final from = held.isEmpty ? -1 : doc.movable.indexOf(held.first!);
        final below = from >= 0 && from < at;
        return Stack(
          clipBehavior: Clip.none,
          children: [
            shown,
            if (from >= 0) Positioned(left: 0, right: 0, top: below ? null : 0, bottom: below ? 0 : null, height: Shape.z(2), child: ColoredBox(color: card.tone)),
          ],
        );
      },
    );
  }
}

/// Three short lines, ≡: an effect's card held there is dragged up or down its stack.
class _Grip extends CustomPainter {
  const _Grip(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final paint = Paint()
      ..color = ink
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round;
    for (final y in const [.32, .5, .68]) {
      c.drawLine(Offset(s.width * .25, s.height * y), Offset(s.width * .75, s.height * y), paint);
    }
  }

  @override
  bool shouldRepaint(_Grip o) => o.ink != ink;
}

/// An effect card's ⋮ (AbsentMotion's EffectHeader More): what else is done with it, in a short list over the panel,
/// as Chooser's lists look; one picked, it does that and closes.
class _CardMore extends StatefulWidget {
  const _CardMore({required this.name, required this.items});
  final String name;
  final List<(String key, String label, VoidCallback does)> items;
  @override
  State<_CardMore> createState() => _CardMoreState();
}

class _CardMoreState extends State<_CardMore> {
  final _link = LayerLink();
  final _portal = OverlayPortalController();
  @override
  Widget build(BuildContext context) => CompositedTransformTarget(
        link: _link,
        child: OverlayPortal(
          controller: _portal,
          overlayChildBuilder: (_) => Stack(children: [
            Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: _portal.hide)),
            CompositedTransformFollower(
              link: _link,
              targetAnchor: Alignment.bottomRight,
              followerAnchor: Alignment.topRight,
              child: TinyMenu(name: 'card', of: widget.name, items: widget.items, hide: _portal.hide),
            ),
          ]),
          child: Hov(
            key: ValueKey('card-more:${widget.name}'),
            onTap: widget.items.isEmpty ? null : _portal.show,
            builder: (_, h) => Padding(padding: EdgeInsets.only(right: Shape.s(6)), child: Text('⋮', style: T.label(h ? Grey.g95 : Grey.g56))),
          ),
        ),
      );
}

/// A bin: an effect taken off by it (AbsentMotion's Trash on its header).
class _Bin extends CustomPainter {
  _Bin(this.color);
  final Color color;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    final w = s.width;
    c.drawLine(Offset(w * .15, w * .25), Offset(w * .85, w * .25), p);
    c.drawLine(Offset(w * .4, w * .12), Offset(w * .6, w * .12), p);
    c.drawPath(Path()..moveTo(w * .25, w * .3)..lineTo(w * .3, w * .9)..lineTo(w * .7, w * .9)..lineTo(w * .75, w * .3), p);
  }

  @override
  bool shouldRepaint(_Bin o) => o.color != color;
}
