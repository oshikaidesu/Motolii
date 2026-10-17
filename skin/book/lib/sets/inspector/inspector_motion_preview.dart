// A card's motion preview, in the manner of Good Boy Ninja's Wiggle: a small square where plain squares lie and move as
// the thing and its copies will (sway, fall and bounce, drift, gather in a field), along the whole of their way drawn
// faint; played while a hand is anywhere on its card and a little while after a number of the card changes, at rest
// otherwise; pulled by hand, it sets them. How they move is the inside's (Motolii.path, asked again when the card's values
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
  static const samples = 60, most = 16; // (moments of one; copies shown at most)
  @override
  State<MotionPreview> createState() => _MotionPreviewState();
}

class _MotionPreviewState extends State<MotionPreview> with SingleTickerProviderStateMixin {
  late final _clock = AnimationController(vsync: this, duration: const Duration(seconds: 2));
  String _asked = '';
  List<List<double>> _paths = const []; // (how each of its copies moves, the first first: one for a thing on its own)
  bool _over = false, _moves = false;
  DateTime _changed = DateTime(0); // (a number of the card changed: it plays a while by itself)
  double _round = 2, _unit = .2;
  final _later = <Timer>[]; // (the asks and the stop after a change: let go with the card)

  void _after(int ms, void Function() does) => _later
    ..removeWhere((t) => !t.isActive)
    ..add(Timer(Duration(milliseconds: ms), () => mounted ? does() : null));

  void _play() { // (while a hand is on its card, or a little while after a number changed: still otherwise, no frames asked for)
    final touched = _over || DateTime.now().difference(_changed).inMilliseconds < (_round * 2000).clamp(1500, 4000);
    if (touched && _moves) {
      if (!_clock.isAnimating) _clock.repeat();
    } else if (_clock.isAnimating || _clock.value != 0) {
      _clock
        ..stop()
        ..value = 0;
    }
  }

  /// How each copy moves, as the inside has it now (fewer moments each when there are many).
  List<List<double>> _all(Doc doc) {
    final copies = (doc.get('v_copies') ?? 1).round().clamp(1, MotionPreview.most);
    return [
      for (var c = 0; c < copies; c++)
        if (doc.onPath?.call(_round, copies > 1 ? 24 : MotionPreview.samples, c) case final p? when p.length >= 8) p,
    ];
  }

  static bool _moving(List<List<double>> paths) =>
      paths.any((p) => [for (var i = 0; i + 3 < p.length; i += 4) (p[i] - p[0]).abs() + (p[i + 1] - p[1]).abs() + (p[i + 2] - p[2]).abs() + (p[i + 3] - p[3]).abs()].any((d) => d > 1e-6));

  void _ask(Doc doc) { // (how they move, as the inside has it now)
    if (!mounted) return;
    final all = _all(doc);
    setState(() {
      _paths = all;
      _moves = _moving(all);
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
    final asked = [for (final id in [...ids, 'v_copies']) doc.get(id)].join(' ');
    if (asked != _asked) { // (a number of the card changed: asked again once the inside has it, then played a while)
      final first = _asked.isEmpty;
      _asked = asked;
      _round = round;
      _clock.duration = Duration(milliseconds: (round * 1000).round());
      if (first) {
        _paths = _all(doc);
        _moves = _moving(_paths);
      } else {
        _changed = DateTime.now();
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
        final box = context.size;
        _unit = _paths.isNotEmpty && box != null ? _fit(_paths, box).k : .2; // (points of the box a point of the picture, as it lies when the hand takes it: kept while held)
        doc.beginGesture();
      },
      onPanUpdate: (d) {
        final pull = _pulls[widget.card];
        void by((String, bool)? to, double points) {
          if (to == null) return;
          final (id, length) = to;
          final now = doc.get('v_$id') ?? 0;
          doc.set('v_$id', now + (length ? points / _unit : points * .5));
        }

        by(pull?.across, d.delta.dx);
        by(pull?.up, -d.delta.dy);
      },
      onPanEnd: (_) => doc.endGesture(),
      child: CustomPaint(size: Size.infinite, painter: _MotionPaint(_paths, _clock)),
    );
  }
}

/// Where all of their ways lie in a box of size [s]: their middle (points of the picture) and how many points of the
/// box a point of the picture is. A small move moves little: one as wide as half a wide picture fills the box.
({double mx, double my, double k}) _fit(List<List<double>> paths, Size s) {
  const pad = 17.0;
  var x0 = paths.first[0], x1 = x0, y0 = paths.first[1], y1 = y0;
  for (final p in paths) {
    for (var k = 0; k + 3 < p.length; k += 4) {
      x0 = math.min(x0, p[k]);
      x1 = math.max(x1, p[k]);
      y0 = math.min(y0, p[k + 1]);
      y1 = math.max(y1, p[k + 1]);
    }
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
  'Field': (across: ('field_radius', true), up: ('field_size', false)), // (how far the field reaches; how much larger it makes them)
  'Connect': (across: ('connect_reach', true), up: null), // (how near copies must be to be joined)
};

class _MotionPaint extends CustomPainter {
  _MotionPaint(this.paths, this.clock) : super(repaint: clock);
  final List<List<double>> paths;
  final Animation<double> clock;

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(8)), Paint()..color = Grey.g07);
    if (paths.isEmpty) return;
    final f = _fit(paths, s); // (where they go, all of it: the box shows the whole of it; the stage's y goes up)
    Offset on(double x, double y) => s.center(Offset.zero) + Offset((x - f.mx) * f.k, -(y - f.my) * f.k);
    final one = paths.length == 1, half = one ? 11.0 : 7.0;
    final faint = Paint()
      ..color = Grey.g26
      ..strokeWidth = 1.2
      ..style = PaintingStyle.stroke;
    void square(Offset at, double turn, double size, Paint paint, {bool mark = false}) {
      c.save();
      c.translate(at.dx, at.dy);
      c.rotate(turn);
      c.scale(size);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCircle(center: Offset.zero, radius: half), Radius.circular(one ? 4 : 2)), paint);
      if (mark) {
        c.drawLine(Offset(-half + half * .36, -half + half * .27), Offset(half - half * .36, -half + half * .27), Paint() // (its top edge marked: how far it turns shows)
          ..color = Grey.g07
          ..strokeWidth = one ? 2 : 1.4
          ..strokeCap = StrokeCap.round);
      }
      c.restore();
    }

    for (final p in paths) {
      // (the whole of each way, drawn always: the line it goes along; for a thing on its own, a few of its moments too,
      //  each as turned and as large as it is then: what a number changes shows as a changed shape, still or playing)
      final n = p.length ~/ 4;
      final line = Path()..moveTo(on(p[0], p[1]).dx, on(p[0], p[1]).dy);
      for (var j = 1; j < n; j++) {
        line.lineTo(on(p[4 * j], p[4 * j + 1]).dx, on(p[4 * j], p[4 * j + 1]).dy);
      }
      c.drawPath(line, faint);
      if (one) {
        const ghosts = 8;
        final ghost = Paint()
          ..color = Pop.accent.withValues(alpha: .14)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2;
        for (var g = 0; g < ghosts; g++) {
          final j = (g * n / ghosts).floor();
          square(on(p[4 * j], p[4 * j + 1]), -(p[4 * j + 2] - p[2]) * math.pi / 180, p[3] == 0 ? 1 : (p[4 * j + 3] / p[3]).clamp(.2, 3.0), ghost);
        }
        final rest = on(p[0], p[1]), cross = Paint()
          ..color = Grey.g26
          ..strokeWidth = 1;
        c.drawLine(rest - const Offset(6, 0), rest + const Offset(6, 0), cross); // (where it rests)
        c.drawLine(rest - const Offset(0, 6), rest + const Offset(0, 6), cross);
      }
    }
    for (final (k, p) in paths.indexed) { // (each where it is now, turned and sized as it is now; the first brightest)
      final n = p.length ~/ 4, i = (clock.value * n).floor() % n, base = paths.first[3] == 0 ? 1.0 : paths.first[3];
      square(on(p[4 * i], p[4 * i + 1]), -(p[4 * i + 2] - (one ? p[2] : 0)) * math.pi / 180, (p[4 * i + 3] / base).clamp(.2, 3.0),
          Paint()..color = k == 0 ? Pop.accent : Pop.accent.withValues(alpha: .6), mark: true);
    }
  }

  @override
  bool shouldRepaint(_MotionPaint o) => o.paths != paths;
}
