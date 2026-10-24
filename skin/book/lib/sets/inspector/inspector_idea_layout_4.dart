// Part of inspector_idea_layout.dart (cut there by size, as it was).
part of 'inspector_idea_layout.dart';

class _LyCPaint extends CustomPainter {
  const _LyCPaint(this.sig, this.outer, this.inner, this.kids, this.guts, this.sel, this.lit, this.over, this.w, this.hot);
  final String sig;
  final Rect outer, inner;
  final List<Rect> kids;
  final List<(Rect, bool)> guts;
  final int? sel;

  /// The gathered spot (x, y) in thirds; -1 on an axis = spread (Space *) or Stretch.
  final (int, int) lit;
  final bool over;
  final double w;
  final String? hot;
  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    c.drawRect(outer, Paint()..color = _lyPadC);
    c.drawRect(inner, Paint()..color = Grey.g10);
    double at(double lo, double hi, int k) => lo + (hi - lo) * (k * 2 + 1) / 6;
    for (var y = 0; y < 3; y++) {
      for (var x = 0; x < 3; x++) {
        final p = Offset(at(inner.left, inner.right, x), at(inner.top, inner.bottom, y));
        c.drawCircle(p, 1.6, Paint()..color = Grey.g38);
        if (hot == 'z$x$y') c.drawCircle(p, 6, _lyLine(Pop.accentInk, 1.4));
      }
    }
    final (lx, ly) = lit;
    final mark = Rect.fromCenter(
      center: Offset(lx < 0 ? inner.center.dx : at(inner.left, inner.right, lx), ly < 0 ? inner.center.dy : at(inner.top, inner.bottom, ly)),
      width: lx < 0 ? inner.width - 8 : 10,
      height: ly < 0 ? inner.height - 8 : 10,
    );
    c.drawRRect(RRect.fromRectAndRadius(mark, const Radius.circular(3)), _lyLine(Pop.toneLayout, 1.4));
    for (final (r, _) in guts) {
      c.drawRect(r, Paint()..color = hot == 'gap' ? _lyGapHot : _lyGapC);
    }
    for (final (i, k) in kids.indexed) {
      _lyKidBox(c, k, _lyCKids[i].name, sel: i == sel, hot: hot == 'k$i');
    }
    c.drawRect(outer, _lyLine(over ? Pop.tonePlace : Grey.g44));
    final on = hot == 'w';
    final grip = Rect.fromCenter(center: Offset(outer.right, outer.center.dy), width: on ? 5 : 4, height: 22);
    c.drawRRect(RRect.fromRectAndRadius(grip, const Radius.circular(2)), Paint()..color = on ? Pop.accent : Grey.g76);
    _lyText(
      c,
      '${w.round()} px${over ? ' · overflows' : ''}',
      Offset(math.max(outer.right - 40, 44), outer.bottom + 7),
      T.micro(over ? Pop.tonePlace : Grey.g56),
    );
  }

  @override
  bool shouldRepaint(_LyCPaint o) => o.sig != sig || o.hot != hot;
}
