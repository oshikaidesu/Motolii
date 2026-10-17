// The Wave card's preview, Good Boy Ninja's Wiggle manner: a small square where a face sways as the thing will, played
// over and over while a hand is on it or pulls the card's numbers below it, and at rest otherwise. How it moves is the inside's (Motolii.path,
// asked once when the values change); the window only plays it back.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

class WavePreview extends StatefulWidget {
  const WavePreview({super.key});
  static const height = 96.0, samples = 60;
  @override
  State<WavePreview> createState() => _WavePreviewState();
}

class _WavePreviewState extends State<WavePreview> with SingleTickerProviderStateMixin {
  late final _clock = AnimationController(vsync: this, duration: const Duration(seconds: 1));
  String _asked = '';
  List<double>? _path;
  bool _over = false, _moves = false;

  void _play(bool touched) { // (only while touched: still otherwise, no frames asked for)
    if (touched && _moves) {
      if (!_clock.isAnimating) _clock.repeat();
    } else if (_clock.isAnimating || _clock.value != 0) {
      _clock
        ..stop()
        ..value = 0;
    }
  }

  @override
  void dispose() {
    _clock.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    final wave = {for (final e in doc.v.entries) if (e.key.startsWith('v_wave') || e.key == 'v_noise_scale') e.key: e.value};
    final speed = (doc.get('v_wave_speed') ?? 1).abs(), round = speed > 0 ? (1 / speed).clamp(.25, 4.0) : 2.0; // (one round of the wave, or two seconds)
    final asked = '$wave';
    if (asked != _asked) { // (asked again only when a number of the wave changed: the preview then plays by itself)
      _asked = asked;
      final p = _path = doc.onPath?.call(round, WavePreview.samples);
      _moves = p != null && [for (var i = 0; i + 3 < p.length; i += 4) (p[i] - p[0]).abs() + (p[i + 1] - p[1]).abs() + (p[i + 2] - p[2]).abs() + (p[i + 3] - p[3]).abs()].any((d) => d > 1e-6);
      _clock.duration = Duration(milliseconds: (round * 1000).round());
    }
    _play(_over || doc.inGesture);
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: MouseRegion(
        onEnter: (_) => _play(_over = true),
        onExit: (_) => _play(_over = false),
        child: SizedBox(height: WavePreview.height, child: CustomPaint(size: Size.infinite, painter: _WavePaint(_path, _clock))),
      ),
    );
  }
}

class _WavePaint extends CustomPainter {
  _WavePaint(this.path, this.clock) : super(repaint: clock);
  final List<double>? path;
  final Animation<double> clock;

  @override
  void paint(Canvas c, Size s) {
    final box = RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(8));
    c.drawRRect(box, Paint()..color = Grey.g07);
    const face = 14.0;
    final o = s.center(Offset.zero);
    var at = Offset.zero, turn = 0.0, size = 1.0;
    final p = path;
    if (p != null && p.length >= 8) {
      final n = p.length ~/ 4, i = (clock.value * n).floor() % n;
      var reach = 0.0; // (the farthest it goes from its first place, in units of the stage)
      for (var k = 0; k < n; k++) {
        reach = math.max(reach, math.max((p[4 * k] - p[0]).abs(), (p[4 * k + 1] - p[1]).abs()));
      }
      final k = (s.width / 2 - face - 6) / math.max(reach, 3.0); // (a small wave moves little; one as wide as half the stage fills the box's width)
      final room = s.height / 2 - face - 4; // (up and down as far as the box is high)
      at = Offset((p[4 * i] - p[0]) * k, (-(p[4 * i + 1] - p[1]) * k).clamp(-room, room)); // (the stage's y goes up)
      turn = -(p[4 * i + 2] - p[2]) * math.pi / 180;
      size = p[3] == 0 ? 1 : (p[4 * i + 3] / p[3]).clamp(.2, 3.0);
    }
    // (where it rests: a faint cross)
    final faint = Paint()
      ..color = Grey.g26
      ..strokeWidth = 1;
    c.drawLine(o - const Offset(6, 0), o + const Offset(6, 0), faint);
    c.drawLine(o - const Offset(0, 6), o + const Offset(0, 6), faint);
    c.save();
    c.translate(o.dx + at.dx, o.dy + at.dy);
    c.rotate(turn);
    c.scale(size);
    c.drawCircle(Offset.zero, face, Paint()..color = Pop.accent);
    final ink = Paint()
      ..color = Grey.g07
      ..strokeWidth = 1.8
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke;
    c.drawCircle(const Offset(-4.5, -3.5), 1.6, Paint()..color = Grey.g07);
    c.drawCircle(const Offset(4.5, -3.5), 1.6, Paint()..color = Grey.g07);
    c.drawArc(Rect.fromCircle(center: const Offset(0, 1), radius: 6), .35, math.pi - .7, false, ink);
    c.restore();
  }

  @override
  bool shouldRepaint(_WavePaint o) => o.path != path;
}
