// The marks of values the window knows, each a small drawing of what its value does (the Transform card's manner, found
// with the words hidden): letters spread as far apart as the spacing, a stem leaning as far as the slant, a corner as
// round as the roundness, a wave as tall as the sway, a ball as shiny as the metal, a bounce as high as the bounce...
// A value with no mark here shows its gauge. Keyed by the inside's own value names ('v_' as a card names them).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';

typedef ValueMark = void Function(Canvas c, Size s, double? v, Color ink);

/// The drawn marks, by value id. A third party's card may add its own here.
final valueMarks = <String, ValueMark>{
  'v_letter_spacing': (c, s, v, ink) { // (three letters, as far apart as the spacing)
    final gap = 1.5 + 4 * ((v ?? 1) / 3).clamp(0.0, 1.0), w = 3.0, start = s.width / 2 - (w * 3 + gap * 2) / 2;
    for (var i = 0; i < 3; i++) {
      c.drawRect(Rect.fromLTWH(start + i * (w + gap), 5, w, s.height - 10), Paint()..color = ink);
    }
  },
  'v_line_spacing': (c, s, v, ink) { // (three lines, as far apart as the spacing)
    final gap = 2 + 4 * ((v ?? 1) / 3).clamp(0.0, 1.0);
    for (var i = 0; i < 3; i++) {
      c.drawLine(Offset(4, s.height / 2 + (i - 1) * gap * 1.4), Offset(s.width - 4, s.height / 2 + (i - 1) * gap * 1.4), _line(ink));
    }
  },
  'v_italic': (c, s, v, ink) { // (a letter's stem, leaning as far as the slant)
    final lean = 10 * ((v ?? 0) / .5).clamp(-1.0, 1.0);
    c.drawPath(Path()
      ..moveTo(s.width / 2 - 3 + lean / 2, 4)
      ..lineTo(s.width / 2 + 3 + lean / 2, 4)
      ..lineTo(s.width / 2 + 3 - lean / 2, s.height - 4)
      ..lineTo(s.width / 2 - 3 - lean / 2, s.height - 4)
      ..close(), Paint()..color = ink);
  },
  'v_round': (c, s, v, ink) { // (a square whose corners are as round as the roundness)
    final r = 7 * ((v ?? 0) / 50).clamp(0.0, 1.0); // (shown in hundredths, up to half)
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: s.center(Offset.zero), width: 14, height: 14), Radius.circular(r)), _line(ink)..strokeWidth = 1.8);
  },
  for (final id in const ['v_wave_x', 'v_wave_y', 'v_wave_z', 'v_wave_rotate', 'v_wave_size', 'v_warp'])
    id: (c, s, v, ink) { // (a wave, as tall as the sway)
      final a = 2 + 5 * ((v ?? 0).abs() / 1).clamp(0.0, 1.0), p = Path()..moveTo(2, s.height / 2);
      for (var x = 2.0; x <= s.width - 2; x += 1) {
        p.lineTo(x, s.height / 2 - math.sin((x - 2) / (s.width - 4) * math.pi * 2) * a);
      }
      c.drawPath(p, _line(ink)..style = PaintingStyle.stroke);
    },
  'v_metallic': (c, s, v, ink) { // (a ball, its highlight as sharp and bright as the metal)
    final m = ((v ?? 0) / 100).clamp(0.0, 1.0), o = s.center(Offset.zero);
    c.drawCircle(o, 7.5, Paint()..color = Grey.g44);
    c.drawCircle(o + const Offset(-2.5, -2.5), 1.5 + 2 * m, Paint()
      ..color = Color.lerp(Grey.g56, const Color(0xFFFFFFFF), m)!
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, 2.5 - 2 * m));
  },
  'v_roughness': (c, s, v, ink) { // (a ball, its highlight as spread as the roughness)
    final r = ((v ?? 0) / 100).clamp(0.0, 1.0), o = s.center(Offset.zero);
    c.drawCircle(o, 7.5, Paint()..color = Grey.g44);
    c.drawCircle(o + const Offset(-2.5, -2.5), 1.5 + 3 * r, Paint()
      ..color = Grey.g91.withValues(alpha: 1 - .6 * r)
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, .5 + 3 * r));
  },
  'v_bounce': (c, s, v, ink) { // (a ball's path off the ground, as high as the bounce)
    final h = 3 + 10 * ((v ?? 0) / 100).clamp(0.0, 1.0), ground = s.height - 3;
    c.drawLine(Offset(2, ground), Offset(s.width - 2, ground), _line(Grey.g56));
    c.drawPath(Path()
      ..moveTo(3, ground - 9)
      ..quadraticBezierTo(7, ground + 2, 9, ground)
      ..quadraticBezierTo(13, ground - h * 2, s.width - 4, ground), _line(ink)..style = PaintingStyle.stroke);
    c.drawCircle(Offset(s.width - 4, ground - 2.5), 2.5, Paint()..color = ink);
  },
  'v_shadow_softness': (c, s, v, ink) { // (a block's shadow, as soft as the softness)
    final soft = ((v ?? 0) / 100).clamp(0.0, 1.0);
    c.drawOval(Rect.fromCenter(center: Offset(s.width / 2 + 2, s.height - 5), width: 14 + 4 * soft, height: 4), Paint()
      ..color = const Color(0xFF000000)
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, .4 + 3 * soft));
    c.drawRect(Rect.fromCenter(center: Offset(s.width / 2, s.height / 2 - 2), width: 9, height: 9), Paint()..color = ink);
  },
  'v_copy_columns': (c, s, v, ink) { // (copies in rows of so many)
    final n = (v ?? 0).round().clamp(0, 4), cols = n == 0 ? 4 : n;
    for (var i = 0; i < 8; i++) {
      c.drawRect(Rect.fromLTWH(3 + (i % cols) * 4.5, 4 + (i ~/ cols) * 4.5, 3, 3), Paint()..color = ink.withValues(alpha: i ~/ cols < 3 ? 1 : 0));
    }
  },
  'v_rotate_x': (c, s, v, ink) => _tilt(c, s, v, ink), // (a camera's pitch, a thing's turn about x: a plane tilted so far)
};

void _tilt(Canvas c, Size s, double? v, Color ink) {
  final a = (v ?? 0) * math.pi / 180, o = s.center(Offset.zero), d = Offset(math.cos(a), math.sin(a)) * 8;
  c.drawLine(Offset(2, o.dy), Offset(s.width - 2, o.dy), _line(Grey.g44));
  c.drawLine(o - d, o + d, _line(ink)..strokeWidth = 2);
}

Paint _line(Color ink) => Paint()
  ..color = ink
  ..strokeWidth = 1.4
  ..strokeCap = StrokeCap.round
  ..style = PaintingStyle.stroke;

/// A value's drawn mark, as a widget of the gauge's size.
class ValueMarkGlyph extends StatelessWidget {
  const ValueMarkGlyph(this.mark, this.value, {super.key, this.ink = const Color(0xFFC2C2C2)});
  final ValueMark mark;
  final double? value;
  final Color ink;
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(22, 22), painter: _MarkPaint(mark, value, ink));
}

class _MarkPaint extends CustomPainter {
  const _MarkPaint(this.mark, this.v, this.ink);
  final ValueMark mark;
  final double? v;
  final Color ink;
  @override
  void paint(Canvas c, Size s) => mark(c, s, v, ink);
  @override
  bool shouldRepaint(_MarkPaint o) => o.v != v || o.mark != mark || o.ink != ink;
}
