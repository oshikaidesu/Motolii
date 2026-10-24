// Part of inspector_host_paths.dart (cut there by size, as it was).
part of 'inspector_host_paths.dart';

/// A card's More: order, duplicate, reset, remove.
class _OpMore extends StatelessWidget {
  const _OpMore(this.inst, this.op);
  final String inst;
  final _PathOp op;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, o = _popOrder(doc), i = o.indexOf(inst), k = 'pop_$inst';
    void move(int by) => _popSet(
      doc,
      o
        ..removeAt(i)
        ..insert(i + by, inst),
    );
    final items = <(String, bool, VoidCallback, bool)>[
      ('Move earlier', i > 0, () => move(-1), false),
      ('Move later', i < o.length - 1, () => move(1), false),
      ('Duplicate', true, () => _popAdd(doc, op, at: i + 1, copy: {for (final id in _popDefaults(inst, op).keys) id: doc.get(id) ?? 0}), false),
      if (op.params.isNotEmpty) ('Reset to defaults', true, () => doc.resetIds(_popDefaults(inst, op).keys), false),
      ('Remove', true, () => _popSet(doc, o..remove(inst)), true),
    ];
    return Float(
      right: true,
      width: 160,
      need: items.length * 22 + 8,
      anchor: (open, toggle) => Hov(
        onTap: Ctx.of(context).cfg.locked ? null : toggle,
        builder: (_, h) => Padding(
          padding: const EdgeInsets.only(right: 12),
          child: Text('More', key: ValueKey('$k.more'), style: T.label(open || h ? Grey.g95 : Grey.g56)),
        ),
      ),
      menu: (close) => Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final (s, ok, go, danger) in items)
            Hov(
              cursor: ok ? SystemMouseCursors.click : SystemMouseCursors.basic,
              onTap: ok
                  ? () {
                      close();
                      go();
                    }
                  : null,
              builder: (_, h) => Container(
                height: 22,
                padding: const EdgeInsets.symmetric(horizontal: 8),
                decoration: BoxDecoration(color: h && ok ? Grey.g26 : null, borderRadius: BorderRadius.circular(5)),
                alignment: Alignment.centerLeft,
                child: Row(
                  children: [
                    if (danger) const ErrMark(size: 10, gap: 5),
                    Text(s, style: T.name(!ok ? Grey.g56 : (danger ? Role.error : Grey.g91))),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// The last row of the stack: opens every kind as a floating two-column list; a pick goes to the bottom of the stack, open.
class _AddOp extends StatelessWidget {
  const _AddOp();
  static const _itemH = 22.0;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, rows = (_kPathOps.length / 2).ceil();
    return Float(
      need: rows * _itemH + 8,
      anchor: (open, toggle) => Hov(
        onTap: x.cfg.locked ? null : toggle,
        builder: (_, h) => Container(
          height: Pop.row,
          color: h || open ? Grey.g15 : Pop.card,
          padding: EdgeInsets.symmetric(horizontal: Pop.inset),
          child: Row(
            children: [
              SizedBox(
                width: 8 + Pop.inset,
                child: Text('+', style: T.name(h || open ? Grey.g95 : Grey.g63).copyWith(fontWeight: FontWeight.w700)),
              ),
              Text('Add path operation', style: T.name(x.cfg.locked ? Grey.g44 : (h || open ? Grey.g95 : Grey.g76))),
              const Spacer(),
              if (open) Text('Close', style: T.label(Grey.g56)),
            ],
          ),
        ),
      ),
      menu: (close) => Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var r = 0; r < rows; r++)
            Row(
              children: [
                for (final op in _kPathOps.skip(r * 2).take(2))
                  Expanded(
                    child: Hov(
                      onTap: () {
                        close();
                        _popAdd(doc, op);
                      },
                      builder: (_, h) => Container(
                        height: _itemH,
                        padding: const EdgeInsets.symmetric(horizontal: 6),
                        decoration: BoxDecoration(color: h ? Grey.g26 : null, borderRadius: BorderRadius.circular(5)),
                        child: Row(
                          children: [
                            OpGlyph(op.id, hot: h),
                            const SizedBox(width: 6),
                            Expanded(
                              child: Text(op.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(h ? Grey.g95 : Grey.g76)),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                if (r * 2 + 1 >= _kPathOps.length) const Spacer(),
              ],
            ),
        ],
      ),
    );
  }
}

/// What each kind does to a line, drawn on one: a trimmed arc, a rounded corner, a zigzag.
class OpGlyph extends StatelessWidget {
  const OpGlyph(this.type, {super.key, this.hot = false});
  final String type;
  final bool hot;
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(14, 14), painter: _OpGlyphPaint(type, hot ? Grey.g95 : Pop.tonePath));
}

class _OpGlyphPaint extends CustomPainter {
  const _OpGlyphPaint(this.type, this.ink);
  final String type;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.3
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    final dot = Paint()..color = ink;
    Offset q(double x, double y) => Offset(x * s.width, y * s.height);
    void line(List<(double, double)> pts) => c.drawPath(Path()..addPolygon([for (final (x, y) in pts) q(x, y)], false), p);
    switch (type) {
      case 'trim':
        c.drawArc(Rect.fromLTRB(q(.12, 0).dx, q(0, .12).dy, q(.88, 0).dx, q(0, .88).dy), -math.pi / 2, math.pi * 1.3, false, p);
        c.drawCircle(q(.5, .12), 1.6, dot);
      case 'round':
        c.drawPath(
          Path()
            ..moveTo(q(.15, .9).dx, q(.15, .9).dy)
            ..lineTo(q(.15, .5).dx, q(.15, .5).dy)
            ..quadraticBezierTo(q(.15, .15).dx, q(.15, .15).dy, q(.5, .15).dx, q(.5, .15).dy)
            ..lineTo(q(.9, .15).dx, q(.9, .15).dy),
          p,
        );
      case 'pucker':
        final o = q(.5, .5), path = Path();
        for (var i = 0; i < 4; i++) {
          final a = i * math.pi / 2 - math.pi / 2, b = a + math.pi / 2;
          final t0 = o + Offset(math.cos(a), math.sin(a)) * s.width * .45, t1 = o + Offset(math.cos(b), math.sin(b)) * s.width * .45;
          if (i == 0) path.moveTo(t0.dx, t0.dy);
          path.quadraticBezierTo(o.dx, o.dy, t1.dx, t1.dy);
        }
        c.drawPath(path, p);
      case 'zigzag':
        line([(.05, .5), (.22, .2), (.39, .8), (.56, .2), (.73, .8), (.95, .4)]);
      case 'offset':
        c.drawRect(Rect.fromPoints(q(.35, .35), q(.65, .65)), p);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromPoints(q(.08, .08), q(.92, .92)), const Radius.circular(3)), p);
      case 'twist':
        c.drawPath(
          Path()
            ..moveTo(q(.1, .85).dx, q(.1, .85).dy)
            ..cubicTo(q(.9, .85).dx, q(.9, .85).dy, q(.1, .15).dx, q(.1, .15).dy, q(.9, .15).dx, q(.9, .15).dy),
          p,
        );
      case 'wiggle':
        line([(.05, .55), (.18, .3), (.3, .7), (.45, .4), (.58, .75), (.72, .25), (.85, .6), (.95, .45)]);
      case 'smooth':
        c.drawPath(
          Path()
            ..moveTo(q(.08, .8).dx, q(.08, .8).dy)
            ..cubicTo(q(.5, .95).dx, q(.5, .95).dy, q(.5, .05).dx, q(.5, .05).dy, q(.92, .2).dx, q(.92, .2).dy),
          p,
        );
      case 'subdivide':
        line([(.1, .75), (.9, .25)]);
        for (var i = 0; i < 4; i++) {
          c.drawCircle(q(.1 + i * .8 / 3, .75 - i * .5 / 3), 1.5, dot);
        }
      case 'reverse':
        line([(.1, .32), (.9, .32), (.72, .14)]);
        line([(.9, .68), (.1, .68), (.28, .86)]);
      case 'extend':
        line([(.3, .5), (.7, .5)]);
        line([(.2, .32), (.04, .5), (.2, .68)]);
        line([(.8, .32), (.96, .5), (.8, .68)]);
      case 'chop':
        for (var i = 0; i < 3; i++) {
          line([(.05 + i * .34, .5), (.23 + i * .34, .5)]);
        }
      case 'resample':
        for (var i = 0; i < 5; i++) {
          final a = math.pi * (1 + i / 4);
          c.drawCircle(q(.5 + math.cos(a) * .4, .75 + math.sin(a) * .55), 1.4, dot);
        }
      case 'bend':
        c.drawPath(
          Path()
            ..moveTo(q(.08, .8).dx, q(.08, .8).dy)
            ..quadraticBezierTo(q(.5, -.1).dx, q(.5, -.1).dy, q(.92, .8).dx, q(.92, .8).dy),
          p,
        );
      case 'osc':
        c.drawPath(Path()..addPolygon([for (var i = 0; i <= 24; i++) q(.05 + i * .9 / 24, .5 - math.sin(i / 24 * math.pi * 4) * .32)], false), p);
    }
  }

  @override
  bool shouldRepaint(_OpGlyphPaint o) => o.type != type || o.ink != ink;
}
