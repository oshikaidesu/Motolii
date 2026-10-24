// The editor as a collage of the sheets that were handed over.
// Left: the Canva mosaic. Stage: Posterlad 006's black form. Time: CHECKERS tiles butted together.
// Right: CONNECTIONS' black pocket and eye. The name sits on the cream, as on the Catholic Ventures sheet.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

const _cream = Color(0xFFF6F1E8);
const _ink = Color(0xFF161616);
const _navy = Color(0xFF2A2158);
const _yellow = Color(0xFFF0C431);
const _blue = Color(0xFF2F5BE0);
const _deep = Color(0xFF243E9E);
const _green = Color(0xFF3CB371);
const _orange = Color(0xFFF15A24);
const _pink = Color(0xFFF4A7C0);
const _red = Color(0xFFE23B2F);
const _violet = Color(0xFF5B3CC4);

const _names = ['sun', 'quarter', 'stripes', 'flower', 'arch', 'dots'];

class CollageCut extends StatefulWidget {
  const CollageCut({super.key});

  @override
  State<CollageCut> createState() => _CollageCutState();
}

class _CollageCutState extends State<CollageCut> {
  var pick = 2;

  static const _label = TextStyle(fontFamily: 'Futura', fontSize: 11, letterSpacing: 1.4, fontWeight: FontWeight.w500, color: _ink, decoration: TextDecoration.none);
  static const _quiet = TextStyle(fontFamily: 'Futura', fontSize: 11, letterSpacing: 1.2, color: Color(0xFF8A8175), decoration: TextDecoration.none);

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: _cream,
      child: Align(
        alignment: Alignment.topLeft,
        child: SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: SingleChildScrollView(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(28, 22, 28, 28),
              child: SizedBox(
                width: 640,
                height: 640,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Row(
                      children: [
                        const Text('COLLAGE', style: _label),
                        const SizedBox(width: 14),
                        const Text('CUBE', style: _quiet),
                        const Spacer(),
                        Text(_names[pick], style: _label.copyWith(color: _navy, fontSize: 18, letterSpacing: 0)),
                        const SizedBox(width: 28),
                        const Text('00:12', style: _label),
                      ],
                    ),
                    const SizedBox(height: 16),
                    Expanded(
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          SizedBox(
                            width: 112,
                            child: GridView.count(
                              crossAxisCount: 2,
                              mainAxisSpacing: 10,
                              crossAxisSpacing: 10,
                              childAspectRatio: 1,
                              physics: const NeverScrollableScrollPhysics(),
                              children: [
                                for (var i = 0; i < _names.length; i++)
                                  GestureDetector(
                                    onTap: () => setState(() => pick = i),
                                    child: CustomPaint(painter: _Motif(i, on: pick == i)),
                                  ),
                              ],
                            ),
                          ),
                          const SizedBox(width: 16),
                          Expanded(child: CustomPaint(painter: _Stage(pick), child: const SizedBox.expand())),
                          const SizedBox(width: 16),
                          SizedBox(
                            width: 132,
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(_names[pick], style: const TextStyle(fontFamily: 'Futura', fontSize: 22, height: 1.05, color: _navy, decoration: TextDecoration.none)),
                                const SizedBox(height: 14),
                                Expanded(child: CustomPaint(painter: _Pocket(pick), child: const SizedBox.expand())),
                              ],
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 14),
                    SizedBox(
                      height: 96,
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.end,
                        children: [
                          const SizedBox(width: 36, height: 36, child: CustomPaint(painter: _Motif(0))),
                          const SizedBox(width: 12),
                          for (var i = 0; i < _names.length; i++)
                            Expanded(
                              child: GestureDetector(
                                onTap: () => setState(() => pick = i),
                                child: Padding(
                                  padding: EdgeInsets.only(top: pick == i ? 0 : 10),
                                  child: Column(
                                    crossAxisAlignment: CrossAxisAlignment.stretch,
                                    children: [
                                      Text('0:0$i', style: _quiet.copyWith(fontSize: 9)),
                                      const SizedBox(height: 4),
                                      Expanded(child: CustomPaint(painter: _Motif(i, tile: true))),
                                    ],
                                  ),
                                ),
                              ),
                            ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _Stage extends CustomPainter {
  const _Stage(this.pick);
  final int pick;

  @override
  void paint(Canvas c, Size s) {
    final black = RRect.fromRectAndRadius(Rect.fromLTWH(36, 28, s.width * .36, s.height - 56), const Radius.circular(78));
    c.drawRRect(black, Paint()..color = _ink);
    c.drawArc(Rect.fromLTWH(s.width * .22, -10, 150, 150), 0, math.pi, true, Paint()..color = _orange);
    final pink = Paint()
      ..color = _pink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 16
      ..strokeCap = StrokeCap.round;
    c.drawArc(Rect.fromLTWH(8, s.height * .42, s.width * .42, 90), math.pi * .15, math.pi * .7, false, pink);
    c.drawCircle(Offset(s.width * .30, s.height * .30), 22, Paint()..color = _blue);
    c.drawCircle(Offset(s.width * .30, s.height * .30), 8, Paint()..color = _cream);
    _place(c, Rect.fromCenter(center: Offset(s.width * .72, s.height * .42), width: 96, height: 96), pick);
    c.drawCircle(Offset(s.width * .78, s.height * .72), 28, Paint()..color = _green);
    c.drawCircle(Offset(s.width * .84, s.height * .22), 16, Paint()..color = _yellow);
  }

  @override
  bool shouldRepaint(_Stage o) => o.pick != pick;
}

class _Pocket extends CustomPainter {
  const _Pocket(this.pick);
  final int pick;

  @override
  void paint(Canvas c, Size s) {
    final pocket = RRect.fromRectAndRadius(Rect.fromLTWH(0, 0, s.width, s.height - 36), const Radius.circular(48));
    c.drawRRect(pocket, Paint()..color = _ink);
    final eye = Offset(s.width * .48, s.height * .42);
    c.drawOval(Rect.fromCenter(center: eye, width: 78, height: 46), Paint()..color = _cream);
    c.drawCircle(eye, 14, Paint()..color = _pink);
    c.drawCircle(eye, 6, Paint()..color = _ink);
    _place(c, Rect.fromLTWH(18, 18, 64, 64), pick);
    var x = 16.0;
    for (final color in const [_orange, _red, _green, _blue, _pink]) {
      c.drawCircle(Offset(x, s.height - 16), 7, Paint()..color = color);
      x += 22;
    }
  }

  @override
  bool shouldRepaint(_Pocket o) => o.pick != pick;
}

class _Motif extends CustomPainter {
  const _Motif(this.kind, {this.on = false, this.tile = false});
  final int kind;
  final bool on, tile;

  @override
  void paint(Canvas c, Size s) {
    final r = Offset.zero & s;
    if (on) c.drawRect(r.inflate(0), Paint()..color = _ink);
    _place(c, on ? r.deflate(3) : r, kind);
  }

  @override
  bool shouldRepaint(_Motif o) => o.kind != kind || o.on != on || o.tile != tile;
}

void _place(Canvas c, Rect r, int kind) {
  c.save();
  c.clipRect(r);
  switch (kind) {
    case 0:
      _sun(c, r);
    case 1:
      c.drawRect(Rect.fromLTWH(r.left, r.top, r.width / 2, r.height), Paint()..color = _yellow);
      c.drawArc(Rect.fromLTWH(r.left + r.width / 2, r.top, r.width / 2, r.height), -math.pi / 2, math.pi, true, Paint()..color = _blue);
      c.drawCircle(r.center, r.shortestSide * .16, Paint()..color = _cream);
    case 2:
      c.drawRect(r, Paint()..color = _ink);
      final stripe = Paint()
        ..color = _cream
        ..strokeWidth = math.max(3, r.shortestSide * .08);
      for (var i = -2; i < 8; i++) {
        final x = r.left + i * r.width / 5;
        c.drawLine(Offset(x, r.bottom), Offset(x + r.width * .45, r.top), stripe);
      }
    case 3:
      c.drawRect(r, Paint()..color = _red);
      for (var i = 0; i < 4; i++) {
        c.save();
        c.translate(r.center.dx, r.center.dy);
        c.rotate(i * math.pi / 2);
        c.drawOval(Rect.fromCenter(center: Offset(0, -r.height * .22), width: r.width * .46, height: r.height * .46), Paint()..color = _cream);
        c.restore();
      }
    case 4:
      c.drawRect(r, Paint()..color = _green);
      c.drawArc(Rect.fromLTWH(r.left, r.top, r.width, r.height), math.pi, math.pi, true, Paint()..color = _blue);
      c.drawCircle(Offset(r.right - r.width * .28, r.top + r.height * .32), r.shortestSide * .16, Paint()..color = _yellow);
    case 5:
      c.drawRect(Rect.fromLTWH(r.left, r.top, r.width * .55, r.height), Paint()..color = _deep);
      c.drawCircle(Offset(r.left + r.width * .28, r.center.dy - 8), 7, Paint()..color = _cream);
      c.drawCircle(Offset(r.left + r.width * .28, r.center.dy + 10), 7, Paint()..color = _cream);
      c.drawCircle(Offset(r.right - r.width * .22, r.center.dy), r.shortestSide * .22, Paint()..color = _violet);
  }
  c.restore();
}

void _sun(Canvas c, Rect r) {
  c.drawRect(r, Paint()..color = _blue);
  final p = Paint()
    ..color = _cream
    ..strokeWidth = 2
    ..strokeCap = StrokeCap.round;
  for (var i = 0; i < 16; i++) {
    final a = i * math.pi / 8;
    c.drawLine(r.center + Offset(math.cos(a), math.sin(a)) * r.shortestSide * .18, r.center + Offset(math.cos(a), math.sin(a)) * r.shortestSide * .42, p);
  }
  c.drawCircle(r.center, r.shortestSide * .14, Paint()..color = _cream);
}
