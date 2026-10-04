// Paper: a parameter is a thing to hold, not a number in a box. One small object per meaning (a vial for Opacity, a dial for
// Rotation, a pad for Position, dice for a Seed, a lever for a Boolean), each with its own reach: drag, Shift for fine, double
// click for the default. The look stays the product's flat dark housing; the toy is in the object and how it answers the hand.
// Motion is event-driven only (an implicit animation after a touch, a spring that stops at rest): nothing ticks while idle.
// Not wired to anything; every value is local to its object.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/physics.dart' show SpringDescription, SpringSimulation;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:motolii_ui/desks/parts.dart' show kBlue, kMint, kPink, kViolet, kYellow;
import 'package:motolii_ui/theme/neutral.dart';

TextStyle _sans(double s, Color c, [FontWeight w = FontWeight.w500]) => TextStyle(fontFamily: 'Inter', fontSize: s, color: c, fontWeight: w, height: 1, decoration: TextDecoration.none);
TextStyle _mono(double s, Color c) => TextStyle(fontFamily: 'Menlo', fontSize: s, color: c, height: 1, decoration: TextDecoration.none);
Paint _fill(Color c) => Paint()..color = c;
Paint _line(Color c, double w) => Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = w..strokeCap = StrokeCap.round;

class _P extends CustomPainter {
  _P(this.f);
  final void Function(Canvas, Size) f;
  @override
  void paint(Canvas c, Size s) => f(c, s);
  @override
  bool shouldRepaint(_P o) => true;
}

Widget _paint(void Function(Canvas, Size) f) => CustomPaint(painter: _P(f));

void _well(Canvas c, Size s, [double r = 6]) => c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, Radius.circular(r)), _fill(N.g07));

void _checker(Canvas c, Size s, [double cell = 6]) {
  final a = _fill(N.g26), b = _fill(N.g33);
  for (var y = 0; y * cell < s.height; y++) {
    for (var x = 0; x * cell < s.width; x++) {
      c.drawRect(Rect.fromLTWH(x * cell, y * cell, cell, cell), (x + y).isEven ? a : b);
    }
  }
}

// ---- how a toy sits: a cell (object, then name and readout under it) or a row of the Inspector (name and readout left, object right)
enum Seat { cell, row }

Widget _seat(Seat seat, String name, String read, Size size, Widget object) => seat == Seat.cell
    ? Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        object,
        const SizedBox(height: 6),
        SizedBox(width: size.width, child: Text(name, maxLines: 1, overflow: TextOverflow.clip, style: _sans(10, N.g76))),
        const SizedBox(height: 3),
        SizedBox(width: size.width, child: Text(read, maxLines: 1, overflow: TextOverflow.clip, style: _mono(9.5, N.g51))),
      ])
    : Padding(
        padding: const EdgeInsets.symmetric(vertical: 5),
        child: Row(children: [
          Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(name, maxLines: 1, style: _sans(10.5, N.g82)),
            const SizedBox(height: 4),
            Text(read, maxLines: 1, style: _mono(9.5, N.g51)),
          ])),
          object,
        ]),
      );

/// A press you can feel: the object sinks a little under the pointer and comes back.
class _Press extends StatefulWidget {
  const _Press({required this.onTap, required this.child});
  final VoidCallback onTap;
  final Widget child;
  @override
  State<_Press> createState() => _PressState();
}

class _PressState extends State<_Press> {
  bool down = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTapDown: (_) => setState(() => down = true),
          onTapUp: (_) => setState(() => down = false),
          onTapCancel: () => setState(() => down = false),
          onTap: widget.onTap,
          child: AnimatedScale(scale: down ? .93 : 1, duration: const Duration(milliseconds: 80), curve: Curves.easeOut, child: widget.child),
        ),
      );
}

/// A value that is a choice or a switch: the toy gets it and a way to set it.
class Hold<T> extends StatefulWidget {
  const Hold(this.initial, this.builder, {super.key});
  final T initial;
  final Widget Function(T value, ValueChanged<T> set) builder;
  @override
  State<Hold<T>> createState() => _HoldState<T>();
}

class _HoldState<T> extends State<Hold<T>> {
  late T value = widget.initial;
  @override
  Widget build(BuildContext context) => widget.builder(value, (v) => setState(() => value = v));
}

// ---- the grab: a 0..1 value held by dragging the object
enum Reach { x, y, round, pad }

class Grab extends StatefulWidget {
  const Grab(this.name, {super.key, required this.read, required this.builder, this.reach = Reach.x, this.v = .5, this.w = .5, this.size = const Size(96, 56), this.seat = Seat.cell, this.tone = kBlue, this.pixels = 140});
  final String name;
  final String Function(double v, double w) read;
  final Widget Function(double v, double w, bool held) builder;
  final Reach reach;
  final double v, w, pixels;
  final Size size;
  final Seat seat;
  final Color tone;
  @override
  State<Grab> createState() => _GrabState();
}

class _GrabState extends State<Grab> {
  late double v = widget.v, w = widget.w;
  bool held = false, hover = false;

  double get _fine => HardwareKeyboard.instance.isShiftPressed ? .1 : 1;

  void _pad(Offset p) => setState(() {
        v = (p.dx / widget.size.width).clamp(0.0, 1.0);
        w = (1 - p.dy / widget.size.height).clamp(0.0, 1.0);
      });

  void _update(DragUpdateDetails d) {
    switch (widget.reach) {
      case Reach.x:
        setState(() => v = (v + d.delta.dx / widget.pixels * _fine).clamp(0.0, 1.0));
      case Reach.y:
        setState(() => v = (v - d.delta.dy / widget.pixels * _fine).clamp(0.0, 1.0));
      case Reach.round:
        final c = Offset(widget.size.width / 2, widget.size.height / 2);
        final p = d.localPosition - c, q = p - d.delta;
        var da = math.atan2(p.dy, p.dx) - math.atan2(q.dy, q.dx);
        if (da > math.pi) da -= 2 * math.pi;
        if (da < -math.pi) da += 2 * math.pi;
        setState(() => v = ((v + da / (2 * math.pi) * _fine) % 1 + 1) % 1);
      case Reach.pad:
        _pad(d.localPosition);
    }
  }

  @override
  Widget build(BuildContext context) {
    final lit = held || hover;
    final object = MouseRegion(
      cursor: held ? SystemMouseCursors.grabbing : SystemMouseCursors.grab,
      onEnter: (_) => setState(() => hover = true),
      onExit: (_) => setState(() => hover = false),
      child: GestureDetector(
        onPanStart: (d) { setState(() => held = true); if (widget.reach == Reach.pad) _pad(d.localPosition); },
        onPanUpdate: _update,
        onPanEnd: (_) => setState(() => held = false),
        onPanCancel: () => setState(() => held = false),
        onDoubleTap: () => setState(() { v = widget.v; w = widget.w; }),
        child: AnimatedScale(
          scale: held ? .97 : 1,
          duration: const Duration(milliseconds: 90),
          curve: Curves.easeOut,
          child: Container(
            width: widget.size.width,
            height: widget.size.height,
            foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: lit ? widget.tone.withValues(alpha: held ? .9 : .5) : N.clear)),
            child: widget.builder(v, w, held),
          ),
        ),
      ),
    );
    return _seat(widget.seat, widget.name, widget.read(v, w), widget.size, object);
  }
}

// =========================================================================================================== LOOK

Widget opacityWash(Seat seat) => Grab('Opacity', seat: seat, v: 1, tone: kBlue, read: (v, _) => '${(v * 100).round()} %', builder: (v, w, h) => ClipRRect(borderRadius: BorderRadius.circular(6), child: _paint((c, s) { _checker(c, s); c.drawRect(Offset.zero & s, _fill(kBlue.withValues(alpha: v))); })));

Widget opacityVial(Seat seat) => Grab('Opacity', seat: seat, reach: Reach.y, v: .72, size: const Size(40, 64), tone: kBlue, read: (v, _) => '${(v * 100).round()} %', builder: (v, w, h) => _paint((c, s) {
      _well(c, s, 8);
      final inner = Rect.fromLTWH(4, 4, s.width - 8, s.height - 8);
      final level = inner.bottom - inner.height * v;
      c.save();
      c.clipRRect(RRect.fromRectAndRadius(inner, const Radius.circular(5)));
      c.drawRect(Rect.fromLTRB(inner.left, level, inner.right, inner.bottom), _fill(kBlue.withValues(alpha: .85)));
      c.drawRect(Rect.fromLTWH(inner.left, level, inner.width, 2), _fill(N.g100.withValues(alpha: .7)));
      c.restore();
      for (var i = 1; i < 4; i++) { final y = inner.top + inner.height * i / 4; c.drawLine(Offset(s.width - 12, y), Offset(s.width - 5, y), _line(N.g100.withValues(alpha: .35), 1)); }
    }));

Widget huePuck(Seat seat) => Grab('Hue', seat: seat, v: .58, size: const Size(112, 28), pixels: 160, tone: kViolet, read: (v, _) => '#${(HSVColor.fromAHSV(1, v * 360, .75, .95).toColor().toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0')}', builder: (v, w, h) => _paint((c, s) {
      final strip = RRect.fromRectAndRadius(Rect.fromLTWH(0, 8, s.width, s.height - 16), const Radius.circular(6));
      c.drawRRect(strip, Paint()..shader = ui.Gradient.linear(Offset.zero, Offset(s.width, 0), [for (var i = 0; i <= 6; i++) HSVColor.fromAHSV(1, i * 60.0, .75, .95).toColor()], [for (var i = 0; i <= 6; i++) i / 6]));
      final p = Offset(6 + (s.width - 12) * v, s.height / 2);
      c.drawCircle(p, h ? 12 : 10, _fill(N.g100));
      c.drawCircle(p, h ? 9 : 7.5, _fill(HSVColor.fromAHSV(1, v * 360, .75, .95).toColor()));
    }));

Widget blurFrost(Seat seat) => Grab('Blur', seat: seat, v: .35, tone: kMint, read: (v, _) => '${(v * 24).toStringAsFixed(1)} px', builder: (v, w, h) {
      final scene = _paint((c, s) {
        c.drawRect(Offset.zero & s, _fill(N.g13));
        for (var i = 0; i < 9; i++) { c.drawRect(Rect.fromLTWH(6.0 + i * 10, 0, 4, s.height), _fill(N.g44)); }
        c.drawCircle(s.center(Offset.zero), 15, _fill(kMint));
      });
      return ClipRRect(borderRadius: BorderRadius.circular(6), child: v < .01 ? scene : ImageFiltered(imageFilter: ui.ImageFilter.blur(sigmaX: v * 7, sigmaY: v * 7, tileMode: TileMode.clamp), child: scene));
    });

Widget blurBand(Seat seat) => Grab('Feather', seat: seat, v: .4, size: const Size(112, 28), tone: kMint, read: (v, _) => '${(v * 60).round()} px', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      final r = RRect.fromRectAndRadius(Rect.fromLTWH(3, 3, s.width - 6, s.height - 6), const Radius.circular(4));
      final a = (.5 - v / 2).clamp(0.0, 1.0), b = (.5 + v / 2).clamp(0.0, 1.0);
      c.drawRRect(r, Paint()..shader = ui.Gradient.linear(r.outerRect.centerLeft, r.outerRect.centerRight, [kMint, kMint, N.g20, N.g20], [0, a, b, 1]));
    }));

Widget blendOverlap(Seat seat) => Grab('Blend  Screen', seat: seat, v: .45, tone: kPink, read: (v, _) => 'overlap ${(v * 100).round()} %', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      final a = Offset(s.width / 2 - 22 * (1 - v) - 4, s.height / 2), b = Offset(s.width / 2 + 22 * (1 - v) + 4, s.height / 2);
      c.saveLayer(Offset.zero & s, Paint());
      c.drawCircle(a, 19, _fill(kPink));
      c.drawCircle(b, 19, Paint()..color = kYellow..blendMode = BlendMode.screen);
      c.restore();
    }));

// =========================================================================================================== SPACE

Widget positionPad(Seat seat) => Grab('Position', seat: seat, reach: Reach.pad, size: const Size(80, 80), tone: kYellow, read: (v, w) => '${(v * 1920).round()}, ${((1 - w) * 1080).round()}', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      for (var i = 1; i < 4; i++) { c.drawLine(Offset(s.width * i / 4, 0), Offset(s.width * i / 4, s.height), _line(N.g20, 1)); c.drawLine(Offset(0, s.height * i / 4), Offset(s.width, s.height * i / 4), _line(N.g20, 1)); }
      final p = Offset(s.width * v, s.height * (1 - w));
      c.drawLine(Offset(0, p.dy), Offset(s.width, p.dy), _line(kYellow.withValues(alpha: .35), 1));
      c.drawLine(Offset(p.dx, 0), Offset(p.dx, s.height), _line(kYellow.withValues(alpha: .35), 1));
      c.drawCircle(p, h ? 9 : 7, _fill(kYellow));
      c.drawCircle(p, 2.5, _fill(N.g10));
    }));

Widget rotationDial(Seat seat) => Grab('Rotation', seat: seat, reach: Reach.round, v: .125, size: const Size(64, 64), tone: kPink, read: (v, _) => '${(v * 360).round()}°', builder: (v, w, h) => _paint((c, s) {
      final o = s.center(Offset.zero), r = s.width / 2 - 2;
      c.drawCircle(o, r, _fill(N.g07));
      for (var i = 0; i < 12; i++) { final a = i * math.pi / 6; c.drawLine(o + Offset(math.cos(a), math.sin(a)) * (r - 1), o + Offset(math.cos(a), math.sin(a)) * (r - 4), _line(N.g38, 1)); }
      c.drawArc(Rect.fromCircle(center: o, radius: r - 2), -math.pi / 2, v * 2 * math.pi, false, _line(kPink, 3));
      c.drawCircle(o, r - 9, _fill(h ? N.g38 : N.g33));
      final a = -math.pi / 2 + v * 2 * math.pi, d = Offset(math.cos(a), math.sin(a));
      c.drawLine(o + d * (r - 24), o + d * (r - 12), _line(N.g100, 3));
    }));

Widget rotationCard(Seat seat) => Grab('Rotation', seat: seat, reach: Reach.round, v: .08, size: const Size(96, 64), tone: kPink, read: (v, _) => '${(v * 360).round()}°', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      final o = s.center(Offset.zero);
      final card = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 44, height: 28), const Radius.circular(4));
      c.save();
      c.translate(o.dx, o.dy);
      c.drawRRect(card, _line(N.g33, 1));
      c.rotate(v * 2 * math.pi);
      c.drawRRect(card, _fill(kPink));
      c.drawRect(Rect.fromLTWH(14, -3, 8, 6), _fill(N.g10));
      c.restore();
    }));

Widget scaleGhost(Seat seat) => Grab('Scale', seat: seat, v: .5, size: const Size(96, 64), tone: kBlue, read: (v, _) => '${(v * 200).round()} %', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      final o = s.center(Offset.zero);
      c.drawRect(Rect.fromCenter(center: o, width: 40, height: 26), _line(N.g38, 1));
      final k = math.max(v * 2, .04);
      c.drawRect(Rect.fromCenter(center: o, width: 40 * k, height: 26 * k), _fill(kBlue.withValues(alpha: .9)));
    }));

Widget cornerRadius(Seat seat) => Grab('Roundness', seat: seat, v: .3, size: const Size(64, 64), pixels: 100, tone: kYellow, read: (v, _) => '${(v * 24).round()} px', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: s.center(Offset.zero), width: 44, height: 44), Radius.circular(v * 22)), _fill(kYellow));
    }));

Widget strokeWidth(Seat seat) => Grab('Stroke', seat: seat, v: .3, size: const Size(96, 40), tone: kMint, read: (v, _) => '${(1 + v * 14).toStringAsFixed(1)} px', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      c.drawLine(Offset(18, s.height - 12), Offset(s.width - 18, 12), _line(kMint, 1 + v * 14));
    }));

// =========================================================================================================== TIME AND CHANCE

Widget timeTape(Seat seat) => Grab('Delay', seat: seat, v: .3, size: const Size(112, 32), pixels: 220, tone: kViolet, read: (v, _) => '${(v * 10).toStringAsFixed(2)} s', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      c.save();
      c.clipRect(Offset.zero & s);
      final t = v * 10, mid = s.width / 2;
      for (var i = 0; i <= 44; i++) {
        final sec = i * .25, x = mid + (sec - t) * 40, big = i % 4 == 0;
        c.drawLine(Offset(x, s.height - 4), Offset(x, s.height - (big ? 14 : 8)), _line(big ? N.g56 : N.g38, 1));
      }
      c.restore();
      c.drawLine(Offset(mid, 3), Offset(mid, s.height - 3), _line(kViolet, 2));
    }));

Widget countDots(Seat seat) => Grab('Count', seat: seat, v: .25, size: const Size(112, 36), tone: kViolet, read: (v, _) => '${(v * 24).round()}', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      final n = (v * 24).round();
      for (var i = 0; i < 24; i++) { c.drawCircle(Offset(10 + (i % 12) * 8.6, 11 + (i ~/ 12) * 14), 2.6, _fill(i < n ? kViolet : N.g26)); }
    }));

Widget easePad(Seat seat) => Grab('Ease', seat: seat, reach: Reach.pad, v: .42, w: .1, size: const Size(80, 80), tone: kBlue, read: (v, w) => 'cubic ${v.toStringAsFixed(2)}, ${w.toStringAsFixed(2)}', builder: (v, w, h) => _paint((c, s) {
      _well(c, s);
      c.drawLine(Offset(0, s.height), Offset(s.width, 0), _line(N.g20, 1));
      final c1 = Offset(s.width * v, s.height * (1 - w)), c2 = Offset(s.width * .75, 0);
      final p = Path()..moveTo(0, s.height)..cubicTo(c1.dx, c1.dy, c2.dx, c2.dy, s.width, 0);
      c.drawLine(Offset(0, s.height), c1, _line(N.g38, 1));
      c.drawPath(p, _line(kBlue, 2.5));
      c.drawCircle(c1, h ? 6 : 5, _fill(N.g100));
    }));

class _Spring extends StatefulWidget {
  const _Spring(this.seat);
  final Seat seat;
  @override
  State<_Spring> createState() => _SpringState();
}

class _SpringState extends State<_Spring> with SingleTickerProviderStateMixin {
  late final ctl = AnimationController.unbounded(vsync: this);
  static const _size = Size(112, 40);

  @override
  void dispose() { ctl.dispose(); super.dispose(); }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
        animation: ctl,
        builder: (_, _) => _seat(widget.seat, 'Spring', 'k 220 · damping 7', _size, MouseRegion(
          cursor: SystemMouseCursors.grab,
          child: GestureDetector(
            onHorizontalDragStart: (_) => ctl.stop(),
            onHorizontalDragUpdate: (d) => ctl.value = (ctl.value + d.delta.dx).clamp(-38.0, 38.0),
            onHorizontalDragEnd: (d) => ctl.animateWith(SpringSimulation(const SpringDescription(mass: 1, stiffness: 220, damping: 7), ctl.value, 0, d.velocity.pixelsPerSecond.dx)),
            child: SizedBox(width: _size.width, height: _size.height, child: _paint((c, s) {
              _well(c, s);
              final y = s.height / 2, x = 76 + ctl.value;
              c.drawRect(Rect.fromLTWH(0, 4, 5, s.height - 8), _fill(N.g44));
              final zig = Path()..moveTo(5, y);
              const turns = 9;
              final span = x - 12 - 5;
              for (var i = 0; i < turns; i++) { zig.lineTo(5 + span * (i + .5) / turns, y + (i.isEven ? -6 : 6)); }
              zig.lineTo(x - 12, y);
              c.drawPath(zig, _line(N.g56, 1.5));
              c.drawCircle(Offset(x, y), 11, _fill(kViolet));
            })),
          ),
        )),
      );
}

Widget springBall(Seat seat) => _Spring(seat);

const _pips = [[4], [0, 8], [0, 4, 8], [0, 2, 6, 8], [0, 2, 4, 6, 8], [0, 2, 3, 5, 6, 8]];

class _Dice extends StatefulWidget {
  const _Dice(this.seat);
  final Seat seat;
  @override
  State<_Dice> createState() => _DiceState();
}

class _DiceState extends State<_Dice> with SingleTickerProviderStateMixin {
  late final ctl = AnimationController(vsync: this, duration: const Duration(milliseconds: 460));
  final rng = math.Random(7);
  int seed = 1337;
  static const _size = Size(64, 64);

  @override
  void dispose() { ctl.dispose(); super.dispose(); }

  void _roll() => setState(() { seed = rng.nextInt(100000); ctl.forward(from: 0); });

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
        animation: ctl,
        builder: (_, _) => _seat(widget.seat, 'Seed', '$seed', _size, _Press(
          onTap: _roll,
          child: SizedBox(width: _size.width, height: _size.height, child: _paint((c, s) {
            _well(c, s, 8);
            final t = ctl.value, rolling = ctl.isAnimating;
            final face = rolling ? (seed + (t * 9).floor() * 5) % 6 : seed % 6;
            c.save();
            c.translate(s.width / 2, s.height / 2);
            c.rotate(Curves.easeOut.transform(t) * 2 * math.pi);
            c.scale(1 + .14 * math.sin(t * math.pi));
            c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 44, height: 44), const Radius.circular(9)), _fill(N.g91));
            for (final i in _pips[face]) { c.drawCircle(Offset((i % 3 - 1) * 12.0, (i ~/ 3 - 1) * 12.0), 3.6, _fill(N.g10)); }
            c.restore();
          })),
        )),
      );
}

Widget diceSeed(Seat seat) => _Dice(seat);

// =========================================================================================================== SWITCHES AND CHOICES

Widget slideSwitch(Seat seat) => Hold<bool>(true, (on, set) => _seat(seat, 'Visible', on ? 'On' : 'Off', const Size(56, 32), _Press(
      onTap: () => set(!on),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOutBack,
        width: 56,
        height: 32,
        padding: const EdgeInsets.all(3),
        alignment: on ? Alignment.centerRight : Alignment.centerLeft,
        decoration: BoxDecoration(color: on ? kMint : N.g20, borderRadius: BorderRadius.circular(16)),
        child: Container(width: 26, height: 26, decoration: BoxDecoration(color: N.g95, shape: BoxShape.circle, boxShadow: const [BoxShadow(color: N.shade40, blurRadius: 3, offset: Offset(0, 1))])),
      ),
    )));

Widget leverSwitch(Seat seat) => Hold<bool>(true, (on, set) => _seat(seat, 'Visible', on ? 'On' : 'Off', const Size(44, 64), _Press(
      onTap: () => set(!on),
      child: SizedBox(width: 44, height: 64, child: TweenAnimationBuilder<double>(
        tween: Tween<double>(end: on ? 1 : 0),
        duration: const Duration(milliseconds: 220),
        curve: Curves.easeOutBack,
        builder: (_, t, _) => _paint((c, s) {
          _well(c, s, 8);
          final cx = s.width / 2, cy = s.height / 2;
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, cy), width: 8, height: 40), const Radius.circular(4)), _fill(N.g13));
          final knob = Offset(cx, cy + (1 - 2 * t) * 17);
          c.drawLine(Offset(cx, cy), knob, _line(N.g63, 5));
          c.drawCircle(knob, 8, _fill(N.g91));
          c.drawCircle(Offset(cx, 8), 2.5, _fill(Color.lerp(N.g26, kMint, t.clamp(0.0, 1.0))!));
        }),
      )),
    )));

Widget litPad(Seat seat) => Hold<bool>(false, (on, set) => _seat(seat, 'Mute', on ? 'On' : 'Off', const Size(56, 56), _Press(
      onTap: () => set(!on),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 140),
        width: 56,
        height: 56,
        decoration: BoxDecoration(
          color: on ? kPink : N.g15,
          borderRadius: BorderRadius.circular(8),
          boxShadow: [if (on) BoxShadow(color: kPink.withValues(alpha: .55), blurRadius: 14)],
        ),
        alignment: Alignment.bottomCenter,
        padding: const EdgeInsets.only(bottom: 6),
        child: Container(width: 14, height: 3, decoration: BoxDecoration(color: on ? N.g10.withValues(alpha: .6) : N.g33, borderRadius: BorderRadius.circular(2))),
      ),
    )));

Widget blendSegments(Seat seat) => Hold<int>(0, (i, set) {
      const names = ['Add', 'Mult', 'Screen'];
      const w = 138.0, k = (w - 4) / 3;
      return _seat(seat, 'Blend', names[i], const Size(w, 28), SizedBox(width: w, height: 28, child: DecoratedBox(
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6)),
        child: Stack(children: [
          AnimatedPositioned(duration: const Duration(milliseconds: 200), curve: Curves.easeOutBack, left: 2 + i * k, top: 2, width: k, height: 24, child: DecoratedBox(decoration: BoxDecoration(color: kPink, borderRadius: BorderRadius.circular(4)))),
          Row(children: [
            for (var n = 0; n < 3; n++)
              Expanded(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => set(n), child: Center(child: Text(names[n], style: _sans(10, i == n ? N.g10 : N.g69, i == n ? FontWeight.w700 : FontWeight.w500))))),
          ]),
        ]),
      )));
    });

Widget easeDetent(Seat seat) => Hold<int>(1, (i, set) {
      const names = ['Linear', 'Ease in', 'Ease out', 'Ease both'];
      const size = Size(112, 44);
      return _seat(seat, 'Easing', names[i], size, SizedBox(width: size.width, height: size.height, child: DecoratedBox(
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6)),
        child: Stack(children: [
          Center(child: AnimatedSwitcher(
            duration: const Duration(milliseconds: 170),
            switchInCurve: Curves.easeOutCubic,
            transitionBuilder: (child, a) => FadeTransition(opacity: a, child: SlideTransition(position: Tween(begin: const Offset(0, .6), end: Offset.zero).animate(a), child: child)),
            child: Text(names[i], key: ValueKey(i), style: _sans(11, N.g91, FontWeight.w600)),
          )),
          Positioned(right: 7, top: 0, bottom: 0, child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
            SizedBox(width: 10, height: 8, child: _paint((c, s) => c.drawPath(Path()..moveTo(0, s.height)..lineTo(s.width / 2, 0)..lineTo(s.width, s.height)..close(), _fill(N.g44)))),
            const SizedBox(height: 4),
            SizedBox(width: 10, height: 8, child: _paint((c, s) => c.drawPath(Path()..moveTo(0, 0)..lineTo(s.width / 2, s.height)..lineTo(s.width, 0)..close(), _fill(N.g44)))),
          ])),
          Column(children: [
            Expanded(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => set((i + names.length - 1) % names.length))),
            Expanded(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => set((i + 1) % names.length))),
          ]),
        ]),
      )));
    });

Widget anchorGrid(Seat seat) => Hold<int>(4, (i, set) {
      const names = ['Top left', 'Top', 'Top right', 'Left', 'Center', 'Right', 'Bottom left', 'Bottom', 'Bottom right'];
      return _seat(seat, 'Anchor', names[i], const Size(64, 64), SizedBox(width: 64, height: 64, child: DecoratedBox(
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(8)),
        child: Padding(padding: const EdgeInsets.all(4), child: Stack(children: [
          AnimatedAlign(duration: const Duration(milliseconds: 180), curve: Curves.easeOutBack, alignment: Alignment((i % 3 - 1).toDouble(), (i ~/ 3 - 1).toDouble()), child: Container(width: 18, height: 18, decoration: BoxDecoration(color: kYellow, borderRadius: BorderRadius.circular(5)))),
          for (var n = 0; n < 9; n++)
            Align(alignment: Alignment((n % 3 - 1).toDouble(), (n ~/ 3 - 1).toDouble()), child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => set(n), child: SizedBox(width: 18, height: 18, child: Center(child: Container(width: 3, height: 3, decoration: BoxDecoration(color: n == i ? N.g10 : N.g38, shape: BoxShape.circle)))))),
        ])),
      )));
    });

// =========================================================================================================== the sheets

/// Every draft by the meaning of the value it edits. A name is a draft to compare, not a decision.
final Map<String, Map<String, Widget Function(Seat)>> toyGroups = {
  'Look': {'Opacity wash': opacityWash, 'Opacity vial': opacityVial, 'Hue puck': huePuck, 'Blur frost': blurFrost, 'Feather band': blurBand, 'Blend overlap': blendOverlap},
  'Space': {'Position pad': positionPad, 'Rotation dial': rotationDial, 'Rotation card': rotationCard, 'Scale ghost': scaleGhost, 'Roundness': cornerRadius, 'Stroke': strokeWidth},
  'Time and chance': {'Time tape': timeTape, 'Count dots': countDots, 'Ease pad': easePad, 'Spring ball': springBall, 'Dice seed': diceSeed},
  'Switches and choices': {'Slide switch': slideSwitch, 'Lever': leverSwitch, 'Lit pad': litPad, 'Segments': blendSegments, 'Detent': easeDetent, 'Anchor grid': anchorGrid},
};

/// The shelf: the drafts as cells, grouped, to see which object suits which meaning.
class ToyShelf extends StatelessWidget {
  const ToyShelf({super.key, this.only});
  final String? only;
  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g10,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            for (final g in toyGroups.entries)
              if (only == null || only == g.key) ...[
                Text(g.key.toUpperCase(), style: _sans(9.5, N.g51, FontWeight.w600).copyWith(letterSpacing: 1.3)),
                const SizedBox(height: 10),
                Wrap(spacing: 22, runSpacing: 20, children: [for (final t in g.value.values) t(Seat.cell)]),
                const SizedBox(height: 26),
              ],
          ]),
        ),
      );
}

/// The same drafts as rows of the Inspector at its real width, to judge them in context and at density.
class ToyRows extends StatelessWidget {
  const ToyRows(this.names, {super.key});
  final List<String> names;
  @override
  Widget build(BuildContext context) {
    final all = {for (final g in toyGroups.values) ...g};
    return ColoredBox(
      color: N.g10,
      child: SingleChildScrollView(
        padding: const EdgeInsets.fromLTRB(12, 10, 12, 16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          for (final n in names) ...[all[n]!(Seat.row), Container(height: .5, color: N.g20)],
        ]),
      ),
    );
  }
}

/// Sheets of rows: a Transform card, an effect card, a Time card.
const toyRowSets = <String, List<String>>{
  'Transform': ['Position pad', 'Rotation dial', 'Scale ghost', 'Anchor grid', 'Opacity vial'],
  'Look': ['Opacity wash', 'Hue puck', 'Blur frost', 'Feather band', 'Blend overlap', 'Segments'],
  'Time and chance': ['Time tape', 'Ease pad', 'Spring ball', 'Count dots', 'Dice seed', 'Lever'],
  'Switches': ['Slide switch', 'Lever', 'Lit pad', 'Segments', 'Detent'],
};
