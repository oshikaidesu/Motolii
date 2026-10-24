// Part of cards_table.dart (cut there by size, as it was).
part of 'cards_table.dart';

class _Ring extends CustomPainter {
  _Ring(this.t, this.time) : super(repaint: Listenable.merge([t, time]));
  final CardTable t;
  final ValueNotifier<double> time;
  @override
  void paint(Canvas c, Size size) {
    final k = math.min(size.width, size.height) / 720;
    for (final th in t.things) {
      th.ring ? _ring(c, size, k, th) : _square(c, size, k, th);
      if (t.things.length > 1) { // (each thing's name under it: what the names over the panel point at)
        final m = size.center((th.base + Offset(t.at('posX', 0, 1, of: th), t.at('posY', 0, 1, of: th))) * k);
        final r = (th.ring ? t.at('radius', 0, 1, of: th) + t.at('size', 0, 1, of: th) / 2 : t.at('size', 0, 1, of: th) * .72) * k;
        final worn = {for (final g in th.tags.values) g.tag}.toList()..sort((a, b) => a.n - b.n); // (the tags it wears, as their letters in their colours)
        final tp = TextPainter(
          text: TextSpan(style: T.micro(th == t.cur ? Pop.accentInk : Grey.g56).copyWith(fontSize: 11), children: [
            TextSpan(text: th.short),
            for (final g in worn) ...[
              TextSpan(text: ' ${g.letter}', style: TextStyle(color: g.c, fontWeight: FontWeight.w700)),
              // (Apparatus: the other end's name sits with the mark, the number stays on the tile)
              TextSpan(
                text: ' ${[
                  for (final e in th.tags.entries)
                    if (e.value.tag == g) t.valOr(e.key, of: th)?.label ?? e.key,
                ].join(' ')}',
                style: TextStyle(color: g.c),
              ),
            ],
          ]),
          textDirection: TextDirection.ltr,
          maxLines: 1,
          ellipsis: '…',
        )..layout(maxWidth: 150);
        tp.paint(c, m + Offset(-tp.width / 2, r + 8));
        if (t.litTag != null && t.clock < t.litUntil && worn.contains(t.litTag)) { // (the wearers of a tag just put on, or of a tapped mark)
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromCircle(center: m, radius: r + 4), const Radius.circular(10)), Paint()..color = t.litTag!.c..style = PaintingStyle.stroke..strokeWidth = 2);
        }
      }
    }
  }

  /// The picked thing, when there are several: a faint frame round it.
  void _picked(Canvas c, Rect r) => c.drawRRect(RRect.fromRectAndRadius(r.inflate(10), const Radius.circular(10)), Paint()..color = Pop.accent.withValues(alpha: .35)..style = PaintingStyle.stroke..strokeWidth = 1);

  Color _colour(ThingM th, double Function(String, double) v) => t.valOr('hue', of: th) == null || t.cardOf('hue', of: th)!.down
      ? Color.fromRGBO(226, 226, 220, v('opacity', 100) / 100) // (no colour card worn: plain)
      : HSLColor.fromAHSL(v('opacity', 100) / 100, v('hue', 72) % 360, .62, v('light', 62) / 100).toColor();

  void _square(Canvas c, Size size, double k, ThingM th) {
    double v(String id, double f) => t.at(id, 0, 1, fallback: f, of: th);
    final m = size.center((th.base + Offset(v('posX', 0), v('posY', 0))) * k), s = v('size', 120) * k;
    c.save();
    c.translate(m.dx, m.dy);
    c.rotate(v('turn', 0) * math.pi / 180);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: s, height: s), Radius.circular(s * .08)), Paint()..color = _colour(th, v));
    c.restore();
    if (t.things.length > 1 && th == t.cur) _picked(c, Rect.fromCenter(center: m, width: s * 1.42, height: s * 1.42));
  }

  void _ring(Canvas c, Size size, double k, ThingM th) {
    final n = t.at('count', 0, 1, fallback: 1, of: th).round();
    if (t.things.length > 1 && th == t.cur) {
      final r = (t.at('radius', 0, 1, of: th) + t.at('size', 0, 1, of: th)) * k;
      _picked(c, Rect.fromCircle(center: size.center((th.base + Offset(t.at('posX', 0, 1, of: th), t.at('posY', 0, 1, of: th))) * k), radius: r));
    }
    for (var i = 0; i < n; i++) {
      double v(String id, double f) => t.at(id, i, n, fallback: f, of: th);
      final m = size.center((th.base + Offset(v('posX', 0), v('posY', 0))) * k);
      final phase = (t.copiesFirstOf(th) ? i / n * 2 * math.pi : 0.0) + v('phase', 0) * 2 * math.pi; // (copies first: each copy waves on its own)
      final wt = (t.waveClock - v('delay', 0) * i / n) * 2.4 - phase, w = math.sin(wt);
      final a = 2 * math.pi * i / n + v('turn', 0) * math.pi / 180 - math.pi / 2;
      final h = _hash(i * 3.3 + .7) * 2 * math.pi;
      final off = Offset.fromDirection(a, v('radius', 0)) + Offset(v('waveX', 0) * math.cos(wt), -v('waveH', 0) * w) + Offset.fromDirection(h, v('scatter', 0) * _hash(i + .3));
      final grow = v('copySize', 100) / 100 * (1 + v('waveSize', 0) / 100 * w) * (1 + (_hash(i * 2.1) - .5) * v('scatterSize', 0) / 100);
      final turn = (v('turn', 0) + i * v('spin', 0) + (_hash(i * 1.9) * 2 - 1) * v('scatterTurn', 0) + v('waveRot', 0) * w) * math.pi / 180;
      final col = _colour(th, v);
      c.save();
      c.translate(m.dx + off.dx * k, m.dy + off.dy * k);
      c.scale(v('scaleX', 100) / 100 * math.cos(v('rotY', 0) * math.pi / 180), v('scaleY', 100) / 100 * math.cos(v('rotX', 0) * math.pi / 180));
      final star = _star(v('size', 44) * k / 2 * grow, turn);
      c.drawPath(star, Paint()..color = col);
      final sw = v('strokeW', 0);
      if (sw > 0) c.drawPath(star, Paint()..color = HSLColor.fromColor(col).withLightness(.9).toColor()..style = PaintingStyle.stroke..strokeWidth = sw * k);
      c.restore();
    }
  }

  Path _star(double r, double turn) {
    final p = Path();
    for (var j = 0; j < 10; j++) {
      final q = Offset.fromDirection(turn - math.pi / 2 + j * math.pi / 5, j.isEven ? r : r * .45);
      j == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    return p..close();
  }

  @override
  bool shouldRepaint(_Ring o) => true;
}

// ---------------------------------------------------------------- the inspector

class _Inspector extends StatelessWidget {
  const _Inspector(this.t);
  final CardTable t;
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
    Container(
      height: 30,
      padding: EdgeInsets.symmetric(horizontal: Pop.inset),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Grey.g20))),
      child: Row(children: [
        Expanded(child: SingleChildScrollView(scrollDirection: Axis.horizontal, child: Row(children: [
        for (final (i, th) in t.things.indexed)
          Padding(
            padding: const EdgeInsets.only(right: 6),
            child: GestureDetector(
              onTap: () => t.select(i),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(color: i == t.sel ? Grey.g26 : null, borderRadius: BorderRadius.circular(6)),
                child: Text(t.things.length > 2 ? th.short : th.name, style: T.title(i == t.sel ? Grey.g95 : Grey.g56)),
              ),
            ),
          ),
        ]))),
        GestureDetector(
          onTap: t.canUndo ? t.undo : null,
          child: Container(
            width: 26,
            height: 22,
            padding: const EdgeInsets.all(5),
            decoration: BoxDecoration(color: t.canUndo ? Pop.well : null, borderRadius: BorderRadius.circular(6)),
            child: CustomPaint(painter: _Back(t.canUndo ? Grey.g91 : Grey.g38)),
          ),
        ),
        const SizedBox(width: 2),
        GestureDetector(
          onTap: t.canRedo ? t.redo : null,
          child: Container(
            width: 26,
            height: 22,
            padding: const EdgeInsets.all(5),
            decoration: BoxDecoration(color: t.canRedo ? Pop.well : null, borderRadius: BorderRadius.circular(6)),
            child: Transform.flip(flipX: true, child: CustomPaint(painter: _Back(t.canRedo ? Grey.g91 : Grey.g38))),
          ),
        ),
      ]),
    ),
    for (final (i, b) in t.bundles.indexed) _BundleTile(t, i, b),
    Expanded(
      child: DragTarget<CardM>(
        onWillAcceptWithDetails: (e) => !t.cards.contains(e.data),
        onAcceptWithDetails: (e) => t.wear(e.data),
        builder: (context, cand, _) => _list(context),
      ),
    ),
  ]);

  Widget _list(BuildContext context) => ListView(
    padding: EdgeInsets.all(Pop.inset),
    children: [for (final (i, c) in t.cards.indexed) _CardSlot(t, c, i, key: ValueKey(c.name))],
  );
}

/// One card in the list, taking a card carried by its head: let go on its upper half the carried card goes above, on the lower under.
class _CardSlot extends StatelessWidget {
  const _CardSlot(this.t, this.c, this.i, {super.key});
  final CardTable t;
  final CardM c;
  final int i;
  @override
  Widget build(BuildContext context) => DragTarget<CardM>(
    onWillAcceptWithDetails: (e) => (t.cards.contains(e.data) || t.shelf.contains(e.data)) && e.data != c,
    onMove: (e) {
      final box = context.findRenderObject() as RenderBox?;
      if (box == null || e.data == c || !(t.cards.contains(e.data) || t.shelf.contains(e.data))) return;
      t.markInsert((c, box.globalToLocal(e.offset).dy > box.size.height / 2));
    },
    onLeave: (_) => t.markInsert(null),
    onAcceptWithDetails: (e) => t.moveCard(e.data, c, t.insertMark?.$2 ?? false),
    builder: (context, cand, _) {
      final m = t.insertMark, line = m != null && m.$1 == c ? m.$2 : null;
      return Padding(
        padding: EdgeInsets.only(bottom: Pop.gap * 3),
        child: Stack(clipBehavior: Clip.none, children: [
          _Card(t, c, i),
          if (line != null)
            Positioned(left: 0, right: 0, top: line ? null : -4, bottom: line ? -4 : null, height: 2, child: ColoredBox(color: Pop.accent)),
        ]),
      );
    },
  );
}

class _Card extends StatelessWidget {
  const _Card(this.t, this.c, this.i);
  final CardTable t;
  final CardM c;
  final int i;
  @override
  Widget build(BuildContext context) {
    final square = Container(width: 14, height: 14, decoration: BoxDecoration(color: c.down ? Grey.g26 : c.tone, borderRadius: BorderRadius.circular(3)));
    final row = Row(children: [
      Padding(padding: const EdgeInsets.fromLTRB(5, 8, 5, 8), child: SizedBox(width: 8, height: 14, child: CustomPaint(painter: _Grip(Grey.g44)))),
      square,
      const SizedBox(width: 8),
      GestureDetector(onTap: c.down ? null : () => t.flip(c), child: Text(c.name, style: T.title(c.down ? Grey.g44 : Grey.g95))),
      if (c.name == 'Wave' && !c.down) ...[const SizedBox(width: 8), GestureDetector(onTap: t.swapOrder, child: _Tag(t.copiesFirst ? 'Per copy' : 'Whole'))],
      const Spacer(),
      if (!c.down) _IconButton(_Ic.flip, on: c.flipped, onTap: () => t.flip(c)),
      const SizedBox(width: 2),
      _IconButton(c.down ? _Ic.eyeOff : _Ic.eye, on: c.down, onTap: () => t.turnDown(c)),
      const SizedBox(width: 4),
    ]);
    // (the whole head is where a card is held: up or down the list it moves there, onto the shelf it comes off)
    final head = Draggable<CardM>(
      data: c,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: Transform.translate(offset: const Offset(-14, -17), child: _Lifted(_CardStub(c, wide: true))),
      childWhenDragging: Opacity(opacity: .35, child: SizedBox(height: 30, child: row)),
      onDragStarted: () => t.carry(c),
      onDragEnd: (_) {
        t.markInsert(null);
        t.carry(null);
      },
      child: MouseRegion(cursor: SystemMouseCursors.grab, child: Container(height: 30, color: const Color(0x00000000), child: row)),
    );
    final opened = t.open != null && c.all.any((v) => v.id == t.open) ? t.drives[t.open] : null;
    final body = c.flipped
        ? Column(children: [for (final v in c.all) _BackRow(t, v)])
        : LayoutBuilder(
            builder: (context, box) {
              final w = (box.maxWidth - Pop.tileGap * 2) / 3;
              return Wrap(spacing: Pop.tileGap, runSpacing: Pop.tileGap, children: [for (final v in c.front) SizedBox(width: w, child: _Tile(t, v))]);
            },
          );
    return AnimatedOpacity(
      duration: Mo.dur,
      opacity: c.down ? .45 : 1,
      child: TweenAnimationBuilder<double>(
        key: ValueKey(t.orderN),
        tween: Tween(begin: t.orderN == 0 ? 1 : 0, end: 1),
        duration: Duration(milliseconds: 260 + 140 * i),
        builder: (context, a, ch) {
          final lit = t.orderN == 0 || (c.name != 'Copies' && c.name != 'Wave') ? 0.0 : math.max(0.0, 1 - ((a * (260 + 140 * i) - 140 * i) / 260 - .5).abs() * 2); // (each card lights after the one above)
          return Container(foregroundDecoration: BoxDecoration(border: Border.all(color: Pop.accent.withValues(alpha: lit), width: 1.5)), child: ch);
        },
        child: Container(
        color: Pop.card,
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          head,
          Padding(
            padding: EdgeInsets.fromLTRB(Pop.inset, 0, Pop.inset, Pop.insetY + 2),
            child: c.down
                ? _MarkRow(t, c.all)
                : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                    AnimatedSwitcher(
                      duration: Mo.dur * 2,
                      transitionBuilder: _flipIn,
                      layoutBuilder: (cur, prev) => Stack(alignment: Alignment.topLeft, children: [...prev, ?cur]),
                      child: KeyedSubtree(key: ValueKey(c.flipped), child: body),
                    ),
                    if (!c.flipped) _MarkRow(t, c.back, prefix: 'Back'),
                    if (opened != null) _DriveKnobs(t, t.valOr(t.open!)!, opened),
                    if (t.openTag != null && c.all.any((v) => v.id == t.openTag) && t.tags[t.openTag] != null) _Wearers(t, t.tags[t.openTag]!.tag),
                  ]),
          ),
        ]),
      ),
      ),
    );
  }

  static Widget _flipIn(Widget child, Animation<double> a) => AnimatedBuilder(
    animation: a,
    builder: (context, ch) => Transform(alignment: Alignment.center, transform: Matrix4.identity()..setEntry(3, 2, .002)..rotateX((1 - a.value) * math.pi / 2), child: ch),
    child: child,
  );
}

/// What lies on values not in sight (a face-down card's, a face-up card's back), small: so nothing moves unseen.
class _MarkRow extends StatelessWidget {
  const _MarkRow(this.t, this.vals, {this.prefix});
  final CardTable t;
  final List<Val> vals;
  final String? prefix;
  @override
  Widget build(BuildContext context) {
    final on = [for (final v in vals) if (t.drives[v.id] != null || t.bundleOf(v) != null) v];
    if (on.isEmpty) return SizedBox(height: prefix == null ? 4 : 0);
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: SizedBox(
        height: 18,
        child: Row(children: [
          if (prefix != null) ...[Text(prefix!, style: T.micro(Grey.g44)), const SizedBox(width: 6)],
          for (final v in on) ...[
            if (t.drives[v.id] != null) ...[_Chip(t.drives[v.id]!.beh, small: true), const SizedBox(width: 3)],
            if (t.bundleOf(v) != null) ...[_TieMark(t.bundleOf(v)!), const SizedBox(width: 3)],
            Text(v.label, style: T.label(Grey.g63)),
            const SizedBox(width: 10),
          ],
        ]),
      ),
    );
  }
}

class _Tag extends StatelessWidget {
  const _Tag(this.word);
  final String word;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
    decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(8)),
    child: Text(word, style: T.micro(Grey.g76)),
  );
}

enum _Ic { flip, eye, eyeOff }

class _IconButton extends StatelessWidget {
  const _IconButton(this.ic, {this.on = false, this.onTap});
  final _Ic ic;
  final bool on;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: onTap,
    behavior: HitTestBehavior.opaque,
    child: Container(
      width: 24,
      height: 22,
      decoration: BoxDecoration(color: on ? Grey.g26 : null, borderRadius: BorderRadius.circular(6)),
      padding: const EdgeInsets.all(5),
      child: CustomPaint(painter: _IcPaint(ic, on ? Grey.g95 : Grey.g56)),
    ),
  );
}

class _IcPaint extends CustomPainter {
  const _IcPaint(this.ic, this.c);
  final _Ic ic;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    switch (ic) {
      case _Ic.flip: // a card half turned over: its near edge tall, its far edge short
        cv.drawPath(Path()..moveTo(w * .15, 0)..lineTo(w * .85, h * .2)..lineTo(w * .85, h * .8)..lineTo(w * .15, h)..close(), st);
        cv.drawLine(Offset(w * .5, h * .1), Offset(w * .5, h * .9), st..color = c.withValues(alpha: .5));
      case _Ic.eye || _Ic.eyeOff:
        final p = Path()
          ..moveTo(0, h * .5)
          ..quadraticBezierTo(w * .5, -h * .15, w, h * .5)
          ..quadraticBezierTo(w * .5, h * 1.15, 0, h * .5);
        cv.drawPath(p, st);
        cv.drawCircle(Offset(w * .5, h * .5), w * .14, Paint()..color = c);
        if (ic == _Ic.eyeOff) cv.drawLine(Offset(w * .1, h * .95), Offset(w * .9, h * .05), st);
    }
  }

  @override
  bool shouldRepaint(_IcPaint o) => o.ic != ic || o.c != c;
}

/// Six dots: where a card is taken to move it up or down.
class _Grip extends CustomPainter {
  const _Grip(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    for (var x = 0; x < 2; x++) {
      for (var y = 0; y < 3; y++) {
        cv.drawCircle(Offset(1.5 + x * (s.width - 3), 1.5 + y * (s.height - 3) / 2), 1.3, Paint()..color = c);
      }
    }
  }

  @override
  bool shouldRepaint(_Grip o) => o.c != c;
}

/// What a value tile or a back row does with what is dropped on it: a behaviour drives it, a behaviour's mark moves to it,
/// another value ties to it. [may]: what the hand holds now could go here (outlined before it arrives).
class _ValDrop extends StatelessWidget {
  const _ValDrop(this.t, this.v, this.builder);
  final CardTable t;
  final Val v;
  final Widget Function(BuildContext, Beh? over, bool tying, bool may) builder;
  bool _ok(Object? o) => switch (o) {
    Beh() => CardTable.canDrive(v),
    String() => CardTable.canDrive(v),
    Tie tie => tie.v != v && CardTable.canDrive(v),
    Val() => o != v && CardTable.canDrive(o) && CardTable.canDrive(v), // (a count is neither moved nor tied)
    _ => false,
  };
  @override
  Widget build(BuildContext context) => DragTarget<Object>(
    onWillAcceptWithDetails: (e) {
      final ok = _ok(e.data);
      if (ok && e.data is Beh) t.hover((v.id, e.data as Beh));
      t.refusing.value = !ok;
      return ok;
    },
    onLeave: (_) {
      t.hover(null);
      t.refusing.value = false;
    },
    onAcceptWithDetails: (e) => switch (e.data) {
      Beh b => t.lay(v.id, b),
      String from => t.moveDrive(from, v.id),
      Tie tie => t.tie(tie.v, v),
      Val a => t.tie(a, v),
      _ => null,
    },
    builder: (context, cands, _) {
      final mark = cands.whereType<String>().firstOrNull;
      final over = cands.whereType<Beh>().firstOrNull ?? (mark != null && mark != v.id ? t.drives[mark]?.beh : null);
      return builder(context, over, cands.whereType<Val>().isNotEmpty, _ok(t.carrying));
    },
  );
}
