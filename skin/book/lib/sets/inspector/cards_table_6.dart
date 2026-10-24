// Part of cards_table.dart (cut there by size, as it was).
part of 'cards_table.dart';

/// A card as it lies on the shelf (or flies): its colour and its name.
class _CardStub extends StatelessWidget {
  const _CardStub(this.c, {this.wide = false});
  final CardM c;
  final bool wide;
  @override
  Widget build(BuildContext context) => Container(
    width: wide ? 150 : null,
    height: 34,
    padding: const EdgeInsets.symmetric(horizontal: 8),
    decoration: BoxDecoration(
      color: Pop.card,
      borderRadius: BorderRadius.circular(6),
      border: Border.all(color: Grey.g26),
      boxShadow: const [BoxShadow(color: Color(0x66000000), offset: Offset(2, 2))], // (a card on a pile: the one under it shows)
    ),
    child: Row(mainAxisSize: MainAxisSize.min, children: [
      Container(width: 12, height: 12, decoration: BoxDecoration(color: c.tone, borderRadius: BorderRadius.circular(3))),
      const SizedBox(width: 7),
      Text(c.name, style: T.name(Grey.g95)),
    ]),
  );
}

/// The behaviour on a value: tapped, its knobs open; carried to another value, it moves there; dropped on the hand, it lets go.
class _DriveChip extends StatelessWidget {
  const _DriveChip(this.t, this.id, this.d);
  final CardTable t;
  final String id;
  final Drive d;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: () => t.toggleOpen(id),
    child: Draggable<String>(
      data: id,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: ValueListenableBuilder<bool>(
        valueListenable: t.refusing,
        builder: (context, no, _) => Transform.translate(offset: const Offset(-9, -9), child: _Lifted(no ? _Refused(_Chip(d.beh)) : _Chip(d.beh))),
      ),
      childWhenDragging: Opacity(opacity: .3, child: _Chip(d.beh)),
      onDragStarted: () => t.carry(id),
      onDragEnd: (_) => t.carry(null),
      child: Container(
        decoration: BoxDecoration(borderRadius: BorderRadius.circular(9), border: Border.all(color: t.open == id ? Grey.g95 : const Color(0x00000000))),
        child: _Chip(d.beh),
      ),
    ),
  );
}

class _Chip extends StatelessWidget {
  const _Chip(this.b, {this.small = false});
  final Beh b;
  final bool small;
  @override
  Widget build(BuildContext context) => Container(
    height: small ? 14 : 18,
    width: small ? 14 : 18,
    padding: EdgeInsets.all(small ? 3 : 4),
    decoration: BoxDecoration(color: b.c, borderRadius: BorderRadius.circular(9)),
    child: CustomPaint(painter: BehGlyph(b, N.g07)),
  );
}

/// A card in the air over a place that does not take it: greyed, crossed.
class _Refused extends StatelessWidget {
  const _Refused(this.child);
  final Widget child;
  @override
  Widget build(BuildContext context) => Stack(alignment: Alignment.center, children: [
    Opacity(opacity: .4, child: child),
    SizedBox.square(dimension: 14, child: CustomPaint(painter: _Cross(C.danger))),
  ]);
}

/// A card in the air: lifted, a little turned, with a shadow under it.
class _Lifted extends StatelessWidget {
  const _Lifted(this.child);
  final Widget child;
  @override
  Widget build(BuildContext context) => Transform.rotate(
    angle: -.06,
    child: DecoratedBox(
      decoration: const BoxDecoration(boxShadow: [BoxShadow(color: Color(0xAA000000), blurRadius: 16, offset: Offset(0, 10))]),
      child: child,
    ),
  );
}

// ---------------------------------------------------------------- the shelf and the hand

/// The shelf (cards the thing may wear) over the hand (behaviour cards). Cards and behaviour marks dropped here go back.
class _Hand extends StatelessWidget {
  const _Hand(this.t);
  final CardTable t;
  @override
  Widget build(BuildContext context) => DragTarget<Object>(
    onWillAcceptWithDetails: (e) => (e.data is CardM && t.cards.contains(e.data)) || e.data is String || e.data is Tie,
    onAcceptWithDetails: (e) => switch (e.data) {
      CardM c => t.shelve(c),
      String id => t.lift(id),
      Tie tie => t.untieOne(tie.v),
      _ => null,
    },
    builder: (context, cand, _) {
      final held = t.carrying, letGo = held is String || held is Tie, waiting = (held is CardM && t.cards.contains(held)) || letGo;
      return AnimatedContainer(
        duration: Mo.dur,
        padding: EdgeInsets.fromLTRB(Pop.inset, 8, Pop.inset, 10),
        decoration: BoxDecoration(
          color: cand.isNotEmpty ? Grey.g13 : Grey.g07,
          border: Border(top: BorderSide(color: cand.isNotEmpty ? Pop.accent : (waiting ? Pop.accent.withValues(alpha: .4) : Grey.g20), width: waiting ? 2 : 1)),
        ),
        child: Stack(children: [
          Column(mainAxisSize: MainAxisSize.min, children: [
          SizedBox(
            width: double.infinity,
            child: Wrap(spacing: 0, runSpacing: 6, children: [
              if (t.shelf.isEmpty && held is CardM) Container(width: 90, height: 34, decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: Grey.g26))), // (the shelf, empty: shown only while a card could come back to it)
              for (final c in t.shelf)
                Padding(
                  padding: const EdgeInsets.only(right: 8),
                  child: Draggable<CardM>(
                    data: c,
                    dragAnchorStrategy: pointerDragAnchorStrategy,
                    feedback: Transform.translate(offset: const Offset(-14, -17), child: _Lifted(_CardStub(c, wide: true))),
                    childWhenDragging: Opacity(opacity: .35, child: _CardStub(c)),
                    onDragStarted: () => t.carry(c),
                    onDragEnd: (_) => t.carry(null),
                    child: _CardStub(c),
                  ),
                ),
            ]),
          ),
          const SizedBox(height: 8),
          Row(children: [
            for (final b in t.hand)
              Expanded(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 3), child: _HandCard(t, b as Beh))),
          ]),
          ]),
          if (letGo)
            Positioned.fill(
              child: Container(
                decoration: BoxDecoration(color: Grey.g07.withValues(alpha: .88), borderRadius: BorderRadius.circular(8), border: Border.all(color: cand.isNotEmpty ? Pop.accent : Grey.g38)),
                child: Center(child: Row(mainAxisSize: MainAxisSize.min, children: [
                  SizedBox.square(dimension: 14, child: CustomPaint(painter: _Down(cand.isNotEmpty ? Pop.accent : Grey.g76))),
                  const SizedBox(width: 8),
                  Text('Remove', style: T.name(cand.isNotEmpty ? Pop.accent : Grey.g76)),
                ])),
              ),
            ),
        ]),
      );
    },
  );
}

class _HandCard extends StatelessWidget {
  const _HandCard(this.t, this.b);
  final CardTable t;
  final Beh b;
  Widget face({bool lifted = false}) => Container(
    height: 64,
    width: lifted ? 80 : null,
    padding: const EdgeInsets.all(8),
    decoration: BoxDecoration(
      color: Grey.g13,
      borderRadius: BorderRadius.circular(8),
      border: Border.all(color: lifted ? b.c : Grey.g26),
      boxShadow: const [BoxShadow(color: Color(0x66000000), offset: Offset(2, 2)), BoxShadow(color: Color(0x33000000), offset: Offset(4, 4))], // (a pile: the same card again under it)
    ),
    child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
      Container(
        width: 24,
        height: 24,
        decoration: BoxDecoration(color: b.c, borderRadius: BorderRadius.circular(6)),
        child: Padding(padding: const EdgeInsets.all(5), child: CustomPaint(painter: BehGlyph(b, N.g07))),
      ),
      FittedBox(fit: BoxFit.scaleDown, child: Text(b.word, maxLines: 1, style: T.name(Grey.g95))),
    ]),
  );
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: () => t.tapLay(b),
    child: TweenAnimationBuilder<double>(
      key: ValueKey(t.nope == b ? t.nopeN : -1),
      tween: Tween(begin: t.nope == b && t.nopeN > 0 ? 1 : 0, end: 0),
      duration: const Duration(milliseconds: 360),
      builder: (context, k, ch) => Transform.translate(offset: Offset(math.sin(k * math.pi * 6) * 5 * k, 0), child: ch), // (no value to go on: a shake, as a "no")
      child: Draggable<Beh>(
    data: b,
    dragAnchorStrategy: pointerDragAnchorStrategy,
    feedback: ListenableBuilder( // (its mark is the fingertip: what is under it takes it)
      listenable: Listenable.merge([t.ringOpen, t.refusing]),
      builder: (context, _) => t.ringOpen.value
          ? Transform.translate(offset: const Offset(-9, -9), child: _Chip(b))
          : Transform.translate(offset: const Offset(-20, -20), child: _Lifted(t.refusing.value ? _Refused(face(lifted: true)) : face(lifted: true))),
    ),
    childWhenDragging: face(),
    onDragStarted: () => t.carry(b),
    onDragEnd: (_) => t.carry(null),
    child: MouseRegion(cursor: SystemMouseCursors.grab, child: face()),
  ),
    ),
  );
}

/// Back: the arrow of taking the last change back.
class _Back extends CustomPainter {
  const _Back(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    cv.drawPath(Path()..moveTo(w * .1, h * .4)..lineTo(w * .62, h * .4)..arcToPoint(Offset(w * .62, h * .95), radius: Radius.circular(h * .28))..lineTo(w * .35, h * .95), st);
    cv.drawPath(Path()..moveTo(w * .35, h * .12)..lineTo(w * .08, h * .4)..lineTo(w * .35, h * .68), st);
  }

  @override
  bool shouldRepaint(_Back o) => o.c != c;
}

/// Down into the hand: what is carried back is let go.
class _Down extends CustomPainter {
  const _Down(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.6..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    cv.drawLine(Offset(w * .5, 0), Offset(w * .5, h * .62), st);
    cv.drawPath(Path()..moveTo(w * .22, h * .36)..lineTo(w * .5, h * .64)..lineTo(w * .78, h * .36), st);
    cv.drawPath(Path()..moveTo(0, h * .7)..lineTo(0, h)..lineTo(w, h)..lineTo(w, h * .7), st);
  }

  @override
  bool shouldRepaint(_Down o) => o.c != c;
}

class _Cross extends CustomPainter {
  const _Cross(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final st = Paint()..color = c..strokeWidth = 1.6..strokeCap = StrokeCap.round;
    cv.drawLine(Offset.zero, Offset(s.width, s.height), st);
    cv.drawLine(Offset(s.width, 0), Offset(0, s.height), st);
  }

  @override
  bool shouldRepaint(_Cross o) => o.c != c;
}

/// What a behaviour does, drawn: a wave, a staircase, a wobble, scattered dots.
class BehGlyph extends CustomPainter {
  const BehGlyph(this.b, this.c);
  final Beh b;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = w * .14..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final p = Path();
    switch (b) {
      case Beh.sway:
        for (var x = 0.0; x <= 1.0001; x += .05) {
          final y = .5 - .32 * math.sin(x * 2 * math.pi);
          x == 0 ? p.moveTo(0, y * w) : p.lineTo(x * w, y * w);
        }
      case Beh.stagger:
        p.moveTo(0, w * .9);
        for (var k = 0; k < 4; k++) {
          p
            ..lineTo(w * k / 4, w * (.9 - k * .25))
            ..lineTo(w * (k + 1) / 4, w * (.9 - k * .25));
        }
      case Beh.noise:
        for (var x = 0.0; x <= 1.0001; x += .1) {
          final y = .5 + .38 * (_hash(x * 9 + 2) - .5) * 2;
          x == 0 ? p.moveTo(0, y * w) : p.lineTo(x * w, y * w);
        }
      case Beh.random:
        for (final (x, y) in const [(.2, .3), (.7, .2), (.45, .55), (.15, .8), (.8, .75)]) {
          cv.drawCircle(Offset(x * w, y * w), w * .1, Paint()..color = c);
        }
        return;
    }
    cv.drawPath(p, st);
  }

  @override
  bool shouldRepaint(BehGlyph o) => o.b != b || o.c != c;
}
