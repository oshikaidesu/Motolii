// A card's motion preview, in the manner of Good Boy Ninja's Wiggle: a small box where a plain square moves as the
// thing will (sways, falls and bounces, drifts), played over and over while a hand is on it or pulls the card's numbers
// below it, and at rest otherwise. How it moves is the inside's (Motolii.path, asked once when the card's values
// change); the window only plays it back.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

class MotionPreview extends StatefulWidget {
  const MotionPreview(this.card, {super.key});

  /// The card whose values it shows (its values, when they change, are asked again).
  final String card;
  static const height = 96.0, samples = 60;
  @override
  State<MotionPreview> createState() => _MotionPreviewState();
}

class _MotionPreviewState extends State<MotionPreview> with SingleTickerProviderStateMixin {
  late final _clock = AnimationController(vsync: this, duration: const Duration(seconds: 2));
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
    final ids = doc.cards.where((c) => c.name == widget.card).firstOrNull?.params.map((p) => 'v_${p.id}') ?? const <String>[];
    final speed = widget.card == 'Wave' ? (doc.get('v_wave_speed') ?? 1).abs() : 0.0;
    final round = speed > 0 ? (1 / speed).clamp(.25, 4.0) : 2.0; // (one round of a wave; two seconds of anything else)
    final asked = [for (final id in ids) doc.get(id)].join(' ');
    if (asked != _asked) { // (asked again only when a number of the card changed: the preview then plays by itself)
      _asked = asked;
      final p = _path = doc.onPath?.call(round, MotionPreview.samples);
      _moves = p != null && [for (var i = 0; i + 3 < p.length; i += 4) (p[i] - p[0]).abs() + (p[i + 1] - p[1]).abs() + (p[i + 2] - p[2]).abs() + (p[i + 3] - p[3]).abs()].any((d) => d > 1e-6);
      _clock.duration = Duration(milliseconds: (round * 1000).round());
    }
    _play(_over || doc.inGesture);
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: MouseRegion(
        onEnter: (_) => _play(_over = true),
        onExit: (_) => _play(_over = false),
        child: SizedBox(height: MotionPreview.height, child: CustomPaint(size: Size.infinite, painter: _MotionPaint(_path, _clock))),
      ),
    );
  }
}

class _MotionPaint extends CustomPainter {
  _MotionPaint(this.path, this.clock) : super(repaint: clock);
  final List<double>? path;
  final Animation<double> clock;

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(8)), Paint()..color = Grey.g07);
    const half = 11.0, pad = half + 6;
    var at = s.center(Offset.zero), rest = at, turn = 0.0, size = 1.0;
    final p = path;
    if (p != null && p.length >= 8) {
      final n = p.length ~/ 4, i = (clock.value * n).floor() % n;
      var x0 = p[0], x1 = p[0], y0 = p[1], y1 = p[1]; // (where it goes, all of it: the box shows the whole of it)
      for (var k = 0; k < n; k++) {
        x0 = math.min(x0, p[4 * k]);
        x1 = math.max(x1, p[4 * k]);
        y0 = math.min(y0, p[4 * k + 1]);
        y1 = math.max(y1, p[4 * k + 1]);
      }
      // (a small move moves little: as wide as half the stage fills the box; the stage's y goes up)
      final k = math.min((s.width / 2 - pad) / math.max((x1 - x0) / 2, 3.0), (s.height / 2 - pad) / math.max((y1 - y0) / 2, 3.0 * (s.height / s.width)));
      Offset on(double x, double y) => s.center(Offset.zero) + Offset((x - (x0 + x1) / 2) * k, -(y - (y0 + y1) / 2) * k);
      rest = on(p[0], p[1]);
      at = on(p[4 * i], p[4 * i + 1]);
      turn = -(p[4 * i + 2] - p[2]) * math.pi / 180;
      size = p[3] == 0 ? 1 : (p[4 * i + 3] / p[3]).clamp(.2, 3.0);
    }
    final faint = Paint()
      ..color = Grey.g26
      ..strokeWidth = 1;
    c.drawLine(rest - const Offset(6, 0), rest + const Offset(6, 0), faint); // (where it rests)
    c.drawLine(rest - const Offset(0, 6), rest + const Offset(0, 6), faint);
    c.save();
    c.translate(at.dx, at.dy);
    c.rotate(turn);
    c.scale(size);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCircle(center: Offset.zero, radius: half), const Radius.circular(4)), Paint()..color = Pop.accent);
    c.drawLine(const Offset(-half + 4, -half + 3), const Offset(half - 4, -half + 3), Paint() // (its top edge marked: how far it turns shows)
      ..color = Grey.g07
      ..strokeWidth = 2
      ..strokeCap = StrokeCap.round);
    c.restore();
  }

  @override
  bool shouldRepaint(_MotionPaint o) => o.path != path;
}
