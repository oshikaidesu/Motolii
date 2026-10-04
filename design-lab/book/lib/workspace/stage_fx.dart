// Stand-ins for effects on the lab's stage. In Motolii the renderer (Rust, wgpu) draws the real shader; here each family gets one
// recognisable look so a try-on reads at a glance. contract: pure function of its inputs, like the rest of the artwork.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';

import 'fx_catalog.dart';
import 'stage_art.dart';

/// Draws a layer through its effects. [draw] paints the layer at a frame offset (Echo reads earlier frames).
/// The last family is outermost, so a try-on (always last) wraps whatever is kept.
void paintFx(Canvas c, List<FxFamily> fams, Rect bounds, void Function(Canvas c, int dt) draw) {
  if (fams.isEmpty) return draw(c, 0);
  final rest = fams.sublist(0, fams.length - 1);
  void inner(Canvas c, int dt) => paintFx(c, rest, bounds, (c, d) => draw(c, d + dt));
  final room = bounds.inflate(160);
  switch (fams.last) {
    case FxFamily.blur:
      c.saveLayer(room, Paint()..imageFilter = ui.ImageFilter.blur(sigmaX: 14, sigmaY: 14));
      inner(c, 0);
      c.restore();
    case FxFamily.light:
      c.saveLayer(
        room,
        Paint()
          ..imageFilter = ui.ImageFilter.blur(sigmaX: 34, sigmaY: 34)
          ..blendMode = BlendMode.plus,
      );
      inner(c, 0);
      c.restore();
      inner(c, 0);
    case FxFamily.color:
      c.saveLayer(room, Paint()..colorFilter = ColorFilter.matrix(_hue(2.2)));
      inner(c, 0);
      c.restore();
    case FxFamily.stylize:
      c.saveLayer(room, Paint()..colorFilter = const ColorFilter.mode(ArtInk.ink, BlendMode.srcIn));
      c.translate(24, 24);
      inner(c, 0);
      c.restore();
      inner(c, 0);
    case FxFamily.distort:
      final o = bounds.center;
      c.save();
      c.translate(o.dx, o.dy);
      c.skew(.24, 0);
      c.translate(-o.dx, -o.dy);
      inner(c, 0);
      c.restore();
    case FxFamily.space:
      for (var i = 8; i > 0; i--) {
        c.saveLayer(room, Paint()..colorFilter = const ColorFilter.mode(ArtInk.navy, BlendMode.srcIn));
        c.translate(i * 4.0, i * 4.0);
        inner(c, 0);
        c.restore();
      }
      inner(c, 0);
    case FxFamily.path:
      c.save();
      c.clipRect(Rect.fromLTRB(room.left, room.top, bounds.left + bounds.width * .62, room.bottom));
      inner(c, 0);
      c.restore();
    case FxFamily.place:
      for (final (i, a) in const [(2, .3), (1, .55)]) {
        c.saveLayer(room.translate(-bounds.width * .42 * i, 0), Paint()..color = Color.fromRGBO(0, 0, 0, a));
        c.translate(-bounds.width * .42 * i, 0);
        inner(c, 0);
        c.restore();
      }
      inner(c, 0);
    case FxFamily.time:
      for (final (dt, a) in const [(-12, .22), (-6, .45)]) {
        c.saveLayer(room, Paint()..color = Color.fromRGBO(0, 0, 0, a));
        inner(c, dt);
        c.restore();
      }
      inner(c, 0);
  }
}

/// A hue rotation by [a] radians, luminance kept.
List<double> _hue(double a) {
  final cs = math.cos(a), sn = math.sin(a);
  const r = .213, g = .715, b = .072;
  return [
    r + cs * (1 - r) + sn * -r,
    g + cs * -g + sn * -g,
    b + cs * -b + sn * (1 - b),
    0,
    0,
    r + cs * -r + sn * .143,
    g + cs * (1 - g) + sn * .140,
    b + cs * -b + sn * -.283,
    0,
    0,
    r + cs * -r + sn * -(1 - r),
    g + cs * -g + sn * g,
    b + cs * (1 - b) + sn * b,
    0,
    0,
    0,
    0,
    0,
    1,
    0,
  ];
}
