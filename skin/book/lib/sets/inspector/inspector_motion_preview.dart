// A card's motion preview, in the manner of Good Boy Ninja's Wiggle: a small box where a plain square moves as the
// thing will (sways, falls and bounces, drifts) along the whole of its way drawn faint; played while a hand is anywhere on
// its card and a little while after a number of the card changes, at rest otherwise; pulled by hand, it sets them. How it moves is the inside's (Motolii.path, asked once when the card's values
// change); the window only plays it back.
import 'dart:async' show Timer;
import 'dart:math' as math;

import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// A hand anywhere on a card (its name, its preview, its tiles): the card's preview plays while it is there.
class CardOver extends StatefulWidget {
  const CardOver({super.key, required this.child});
  final Widget child;
  static ValueListenable<bool>? of(BuildContext context) => context.dependOnInheritedWidgetOfExactType<_Over>()?.notifier;
  @override
  State<CardOver> createState() => _CardOverState();
}

class _CardOverState extends State<CardOver> {
  final _on = ValueNotifier(false);
  @override
  void dispose() {
    _on.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) => _on.value = true,
    onExit: (_) => _on.value = false,
    child: _Over(notifier: _on, child: widget.child),
  );
}

class _Over extends InheritedNotifier<ValueNotifier<bool>> {
  const _Over({required super.notifier, required super.child});
}

class MotionPreview extends StatefulWidget {
  const MotionPreview(this.card, {super.key});

  /// The card whose values it shows (its values, when they change, are asked again).
  final String card;
  static const samples = 60;
  @override
  State<MotionPreview> createState() => _MotionPreviewState();
}

class _MotionPreviewState extends State<MotionPreview> with SingleTickerProviderStateMixin {
  late final _clock = AnimationController(vsync: this, duration: const Duration(seconds: 2));
  String _asked = '';
  List<double>? _path;
  bool _over = false, _moves = false;
  DateTime _changed = DateTime(0); // (a number of the card changed: it plays a while by itself)
  double _round = 2, _unit = .2;
  final _later = <Timer>[]; // (the asks and the stop after a change: let go with the card)

  void _after(int ms, void Function() does) => _later
    ..removeWhere((t) => !t.isActive)
    ..add(Timer(Duration(milliseconds: ms), () => mounted ? does() : null));

  void _play() { // (while a hand is on it, or a little while after a number changed: still otherwise, no frames asked for)
    final touched = _over || DateTime.now().difference(_changed).inMilliseconds < (_round * 2000).clamp(1500, 4000);
    if (touched && _moves) {
      if (!_clock.isAnimating) _clock.repeat();
    } else if (_clock.isAnimating || _clock.value != 0) {
      _clock
        ..stop()
        ..value = 0;
    }
  }

  void _ask(Doc doc) { // (how it moves, as the inside has it now)
    if (!mounted) return;
    final p = doc.onPath?.call(_round, MotionPreview.samples);
    setState(() {
      _path = p;
      _moves = p != null && [for (var i = 0; i + 3 < p.length; i += 4) (p[i] - p[0]).abs() + (p[i + 1] - p[1]).abs() + (p[i + 2] - p[2]).abs() + (p[i + 3] - p[3]).abs()].any((d) => d > 1e-6);
    });
    _play();
  }

  @override
  void dispose() {
    for (final t in _later) {
      t.cancel();
    }
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
    if (asked != _asked) { // (a number of the card changed: asked again once the inside has it, then played a while)
      final first = _asked.isEmpty;
      _asked = asked;
      _round = round;
      _clock.duration = Duration(milliseconds: (round * 1000).round());
      if (!first) _changed = DateTime.now();
      if (first) {
        _path ??= doc.onPath?.call(round, MotionPreview.samples);
        final p = _path;
        _moves = p != null && [for (var i = 0; i + 3 < p.length; i += 4) (p[i] - p[0]).abs() + (p[i + 1] - p[1]).abs() + (p[i + 2] - p[2]).abs() + (p[i + 3] - p[3]).abs()].any((d) => d > 1e-6);
      } else {
        _after(60, () => _ask(doc)); // (what the hand set reaches the inside in its queue)
        _after(250, () => _ask(doc));
        _after((round * 2000).clamp(1500, 4000).round() + 50, _play); // (still again after a while)
      }
    }
    _over = CardOver.of(context)?.value ?? false; // (a hand on the card, anywhere: it plays)
    _play();
    return GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanStart: (d) {
            final p = _path, box = context.size;
            _unit = p != null && p.length >= 8 && box != null ? _fit(p, box).k : .2; // (points of the box a point of the picture, as it lies when the hand takes it: kept while held)
            doc.beginGesture();
          },
          onPanUpdate: (d) {
            final pull = _pulls[widget.card];
            void by((String, bool)? to, double points) {
              if (to == null) return;
              final (id, units) = to;
              final now = doc.get('v_$id') ?? 0;
              doc.set('v_$id', now + (units ? points / _unit : points * .5));
            }

            by(pull?.across, d.delta.dx);
            by(pull?.up, -d.delta.dy);
          },
          onPanEnd: (_) => doc.endGesture(),
          child: CustomPaint(size: Size.infinite, painter: _MotionPaint(_path, _clock)),
        );
  }
}

/// Where the whole of its way lies in a box of size [s]: its middle (points of the picture) and how many points of the
/// box a point of the picture is. A small move moves little: one as wide as half a wide picture fills the box.
({double mx, double my, double k}) _fit(List<double> p, Size s) {
  const pad = 17.0;
  final n = p.length ~/ 4;
  var x0 = p[0], x1 = p[0], y0 = p[1], y1 = p[1];
  for (var k = 0; k < n; k++) {
    x0 = math.min(x0, p[4 * k]);
    x1 = math.max(x1, p[4 * k]);
    y0 = math.min(y0, p[4 * k + 1]);
    y1 = math.max(y1, p[4 * k + 1]);
  }
  final k = math.min((s.width / 2 - pad) / math.max((x1 - x0) / 2, 480.0), (s.height / 2 - pad) / math.max((y1 - y0) / 2, 480.0 * (s.height / s.width)));
  return (mx: (x0 + x1) / 2, my: (y0 + y1) / 2, k: k);
}

/// What a hand pulling the preview changes, by card: across and up and down, each a value and how it reads the pull
/// (a length: as far as the picture goes under the hand; degrees or a share: half a point a point).
const _pulls = <String, ({(String, bool)? across, (String, bool)? up})>{
  'Wave': (across: ('wave_x', true), up: ('wave_rotate', false)), // (how wide it sways; how far it tips)
  'Physics': (across: ('bounce', false), up: ('y', true)), // (how much it bounces; how high it falls from)
  'Scatter': (across: ('drift_x', true), up: ('drift_y', true)), // (how fast it drifts, and which way)
};

class _MotionPaint extends CustomPainter {
  _MotionPaint(this.path, this.clock) : super(repaint: clock);
  final List<double>? path;
  final Animation<double> clock;

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(8)), Paint()..color = Grey.g07);
    const half = 11.0;
    var at = s.center(Offset.zero), rest = at, turn = 0.0, size = 1.0;
    final p = path;
    if (p != null && p.length >= 8) {
      final n = p.length ~/ 4, i = (clock.value * n).floor() % n;
      final f = _fit(p, s); // (where it goes, all of it: the box shows the whole of it; the stage's y goes up)
      Offset on(double x, double y) => s.center(Offset.zero) + Offset((x - f.mx) * f.k, -(y - f.my) * f.k);
      rest = on(p[0], p[1]);
      at = on(p[4 * i], p[4 * i + 1]);
      turn = -(p[4 * i + 2] - p[2]) * math.pi / 180;
      size = p[3] == 0 ? 1 : (p[4 * i + 3] / p[3]).clamp(.2, 3.0);
      // (the whole of its way, drawn always: the line it goes along, and a few of its moments, faint, each as turned and
      //  as large as it is then: what a number changes shows as a changed shape, still or playing)
      final line = Path()..moveTo(on(p[0], p[1]).dx, on(p[0], p[1]).dy);
      for (var j = 1; j < n; j++) {
        line.lineTo(on(p[4 * j], p[4 * j + 1]).dx, on(p[4 * j], p[4 * j + 1]).dy);
      }
      c.drawPath(line, Paint()
        ..color = Grey.g26
        ..strokeWidth = 1.2
        ..style = PaintingStyle.stroke);
      const ghosts = 8;
      for (var g = 0; g < ghosts; g++) {
        final j = (g * n / ghosts).floor(), o = on(p[4 * j], p[4 * j + 1]);
        c.save();
        c.translate(o.dx, o.dy);
        c.rotate(-(p[4 * j + 2] - p[2]) * math.pi / 180);
        c.scale(p[3] == 0 ? 1 : (p[4 * j + 3] / p[3]).clamp(.2, 3.0));
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCircle(center: Offset.zero, radius: half), const Radius.circular(4)), Paint()
          ..color = Pop.accent.withValues(alpha: .14)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2);
        c.restore();
      }
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
