// The dose study, part 2: widget 6 (spatial pad), widget 7 (plain controls: takes NO dose), widget 8 (too-far meter).
import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'dose_parts.dart';

TextPainter _tp(String s, Color c) => TextPainter(text: TextSpan(text: s, style: T.label(c)), textDirection: TextDirection.ltr)..layout();

// ---------------------------------------------------------------- 6 spatial pad

class _XYPainter extends CustomPainter {
  const _XYPainter(this.d, this.x, this.y);
  final Dose d;
  final double x, y; // -1..1, y up
  @override
  void paint(Canvas c, Size s) {
    final r = (Offset.zero & s).deflate(Dose.hair / 2);
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    c.drawPath(cutPath(r, d.padCut), Paint()..color = N.g07);
    for (final f in [.25, .5, .75]) {
      line.color = N.g100.withValues(alpha: f == .5 ? Dose.crossAlpha : Dose.hairIdle);
      c.drawLine(Offset(r.left + f * r.width, r.top), Offset(r.left + f * r.width, r.bottom), line);
      c.drawLine(Offset(r.left, r.top + f * r.height), Offset(r.right, r.top + f * r.height), line);
    }
    c.drawPath(cutPath(r, d.padCut), line..color = N.g100.withValues(alpha: Dose.frameAlpha));
    final p = Offset(r.left + (x + 1) / 2 * r.width, r.bottom - (y + 1) / 2 * r.height);
    line.color = N.g100.withValues(alpha: Dose.crossAlpha);
    c.drawLine(Offset(p.dx, r.top), Offset(p.dx, r.bottom), line);
    c.drawLine(Offset(r.left, p.dy), Offset(r.right, p.dy), line);
    if (d.padCornerTicks) {
      line.color = N.g56;
      const t = Dose.cornerTick;
      for (final k in [(r.topLeft, 1.0, 1.0), (r.topRight, -1.0, 1.0), (r.bottomLeft, 1.0, -1.0), (r.bottomRight, -1.0, -1.0)]) {
        if (d.padCut > 0 && k.$1 != r.topLeft && k.$1 != r.bottomRight) continue;
        c.drawLine(k.$1, k.$1 + Offset(k.$2 * t, 0), line);
        c.drawLine(k.$1, k.$1 + Offset(0, k.$3 * t), line);
      }
    }
    // axis ticks on the frame at the cursor: grey at dose <= 1, a 1px hue at 2+
    c.drawLine(Offset(p.dx, r.top), Offset(p.dx, r.top + Dose.tickMid), line..color = d.padHueTicks ? Dose.hueX : N.g56);
    c.drawLine(Offset(r.left, p.dy), Offset(r.left + Dose.tickMid, p.dy), line..color = d.padHueTicks ? Dose.hueY : N.g56);
    c.drawCircle(p, Dose.cursorR, line..color = d.padHueCursor ? Dose.hueR : N.g95);
    final lx = _tp('X', N.g56), ly = _tp('Y', N.g56);
    lx.paint(c, Offset(r.right - lx.width - Dose.gap, r.bottom - lx.height - Dose.gap));
    ly.paint(c, Offset(r.left + Dose.gap, r.top + Dose.gap + Dose.tickMid));
  }

  @override
  bool shouldRepaint(_XYPainter o) => o.d != d || o.x != x || o.y != y;
}

class _RailPainter extends CustomPainter {
  const _RailPainter(this.d, this.z);
  final Dose d;
  final double z; // 0..1
  @override
  void paint(Canvas c, Size s) {
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair..color = N.g100.withValues(alpha: Dose.frameAlpha);
    final cx = s.width / 2, top = Dose.diamond, bot = s.height - Dose.diamond - Dose.kickerPx;
    c.drawLine(Offset(cx, top), Offset(cx, bot), line);
    final tick = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair..color = d.padHueTicks ? Dose.hueZ : N.g56;
    for (var i = 0; i < 5; i++) {
      final y = bot - (bot - top) * i / 4;
      c.drawLine(Offset(cx - Dose.tickMinor, y), Offset(cx + Dose.tickMinor, y), tick);
    }
    final y = bot - (bot - top) * z, h = Dose.diamond;
    c.drawPath(Path()..moveTo(cx, y - h)..lineTo(cx + h, y)..lineTo(cx, y + h)..lineTo(cx - h, y)..close(), Paint()..color = N.g95);
    final lz = _tp('Z', N.g56);
    lz.paint(c, Offset(cx - lz.width / 2, s.height - lz.height));
  }

  @override
  bool shouldRepaint(_RailPainter o) => o.d != d || o.z != z;
}

class _DialPainter extends CustomPainter {
  const _DialPainter(this.d, this.deg, this.flash);
  final Dose d;
  final double deg;
  final bool flash;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), r = Dose.dialR - Dose.diamond;
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    c.drawCircle(o, r, line..color = N.g100.withValues(alpha: Dose.frameAlpha));
    for (var a = 0; a < 360; a += 5) {
      final len = a % 90 == 0 ? Dose.tickMajor : (a % 15 == 0 ? Dose.tickMid : Dose.tickMinor);
      final v = Offset(math.sin(a * math.pi / 180), -math.cos(a * math.pi / 180));
      c.drawLine(o + v * r, o + v * (r - len), line..color = a % 90 == 0 && d.padHueTicks ? Dose.hueR : (a % 15 == 0 ? N.g56 : N.g100.withValues(alpha: Dose.frameAlpha)));
    }
    for (final a in [0, 90, 180, 270]) {
      final t = _tp('$a', N.g56), v = Offset(math.sin(a * math.pi / 180), -math.cos(a * math.pi / 180)), at = o + v * (r - Dose.tickMajor - Dose.gapL - 2);
      t.paint(c, at - Offset(t.width / 2, t.height / 2));
    }
    final h = o + Offset(math.sin(deg * math.pi / 180), -math.cos(deg * math.pi / 180)) * r;
    c.drawLine(o, h, line..color = N.g100.withValues(alpha: Dose.crossAlpha));
    c.drawCircle(h, Dose.cursorR, Paint()..color = N.g95);
    if (flash) c.drawCircle(h, Dose.cursorR + Dose.gap, line..color = Dose.hueR);
  }

  @override
  bool shouldRepaint(_DialPainter o) => o.d != d || o.deg != deg || o.flash != flash;
}

class _ScalePainter extends CustomPainter {
  const _ScalePainter(this.d, this.sx, this.sy);
  final Dose d;
  final double sx, sy;
  static List<Offset> handles(Size s, double sx, double sy) {
    final w = Dose.scaleBase * sx, h = Dose.scaleBase * sy, o = s.center(Offset.zero);
    return [for (final hy in [-1, 0, 1]) for (final hx in [-1, 0, 1]) if (hx != 0 || hy != 0) o + Offset(hx * w / 2, hy * h / 2)];
  }

  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), w = Dose.scaleBase * sx, h = Dose.scaleBase * sy, box = Rect.fromCenter(center: o, width: w, height: h);
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    c.drawRect(Rect.fromCenter(center: o, width: Dose.scaleBase, height: Dose.scaleBase), line..color = N.g100.withValues(alpha: Dose.hairIdle));
    c.drawRect(box, line..color = N.g100.withValues(alpha: Dose.frameAlpha * 2));
    if ((sx - sy).abs() < .001) c.drawLine(box.topLeft, box.bottomRight, line..color = N.g100.withValues(alpha: Dose.crossAlpha));
    for (final p in handles(s, sx, sy)) {
      final sq = Rect.fromCenter(center: p, width: Dose.handle, height: Dose.handle);
      c.drawRect(sq, Paint()..color = N.g10);
      c.drawRect(sq, line..color = N.g95);
    }
  }

  @override
  bool shouldRepaint(_ScalePainter o) => o.d != d || o.sx != sx || o.sy != sy;
}

/// X/Y pad + Z rail + rotation dial + scale box, all draggable; the numbers are a secondary readout. Grey ticks at dose <= 1, 1px hue ticks at 2+, hue cursor and cut corners at 3.
class SpatialPad extends StatefulWidget {
  const SpatialPad({super.key, required this.d});
  final Dose d;
  @override
  State<SpatialPad> createState() => _SpatialPadState();
}

class _SpatialPadState extends State<SpatialPad> {
  double x = .3, y = -.2, z = .4, deg = 45, sx = 1, sy = 1;
  bool flash = false;
  Timer? _flashTimer;
  int _handle = -1;

  @override
  void dispose() {
    _flashTimer?.cancel();
    super.dispose();
  }

  void _xy(Offset p) => setState(() {
        x = (p.dx / Dose.padSize * 2 - 1).clamp(-1.0, 1.0);
        y = (1 - p.dy / Dose.padSize * 2).clamp(-1.0, 1.0);
      });

  void _rail(Offset p) {
    const h = Dose.padSize, top = Dose.diamond, bot = h - Dose.diamond - Dose.kickerPx;
    setState(() => z = ((bot - p.dy) / (bot - top)).clamp(0.0, 1.0));
  }

  void _dial(Offset p) {
    final v = p - const Offset(Dose.dialR, Dose.dialR);
    final a = (math.atan2(v.dx, -v.dy) * 180 / math.pi + 360) % 360;
    if (widget.d.detentFlashMs > 0 && (a / 15).floor() != (deg / 15).floor()) {
      flash = true;
      _flashTimer?.cancel();
      _flashTimer = Timer(Duration(milliseconds: widget.d.detentFlashMs), () => mounted ? setState(() => flash = false) : null);
    }
    setState(() => deg = a);
  }

  void _scaleDown(Offset p) {
    const s = Size.square(Dose.scaleBox);
    final hs = _ScalePainter.handles(s, sx, sy);
    var best = -1, bd = Dose.hit / 2;
    for (var i = 0; i < hs.length; i++) {
      final dd = (hs[i] - p).distance;
      if (dd < bd) {
        bd = dd;
        best = i;
      }
    }
    _handle = best;
  }

  void _scaleMove(Offset p) {
    if (_handle < 0) return;
    const hxy = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];
    final (hx, hy) = hxy[_handle];
    final o = const Size.square(Dose.scaleBox).center(Offset.zero);
    double v(double dist) => (dist * 2 / Dose.scaleBase).clamp(.25, 2.0);
    setState(() {
      if (hx != 0 && hy != 0) {
        final k = (v((p.dx - o.dx).abs()) + v((p.dy - o.dy).abs())) / 2;
        sx = sy = k;
      } else if (hx != 0) {
        sx = v((p.dx - o.dx).abs());
      } else {
        sy = v((p.dy - o.dy).abs());
      }
    });
  }

  Widget _drag(double w, double h, CustomPainter p, void Function(Offset) down, void Function(Offset) move) => GestureDetector(
        onPanDown: (e) => down(e.localPosition),
        onPanUpdate: (e) => move(e.localPosition),
        child: MouseRegion(cursor: SystemMouseCursors.precise, child: SizedBox(width: w, height: h, child: CustomPaint(painter: p))),
      );

  @override
  Widget build(BuildContext context) {
    final d = widget.d;
    String f(double v, String u) => '${v < 0 ? '-' : '+'}${v.abs().round()}$u';
    return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        _drag(Dose.padSize, Dose.padSize, _XYPainter(d, x, y), _xy, _xy),
        _drag(Dose.railW, Dose.padSize, _RailPainter(d, z), _rail, _rail),
        const SizedBox(width: Dose.gapL),
        _drag(Dose.dialR * 2, Dose.dialR * 2, _DialPainter(d, deg, flash), _dial, _dial),
        const SizedBox(width: Dose.gapL),
        _drag(Dose.scaleBox, Dose.scaleBox, _ScalePainter(d, sx, sy), _scaleDown, _scaleMove),
      ]),
      const SizedBox(height: Dose.gapL),
      Text('X ${f(x * Dose.padRange, ' px')}  Y ${f(y * Dose.padRange, ' px')}  Z ${f(z * Dose.zRange, ' px')}  R ${f(deg, '°')}  S ${(sx * 100).round()}% × ${(sy * 100).round()}%', style: T.value(N.g76)),
    ]);
  }
}

// ---------------------------------------------------------------- 7 plain control group: no Dose in, so it cannot differ

/// Numeric field + 30-row list + blend list. It has no dose parameter and reads nothing from `Dose`: identical at every dose by construction.
class PlainControls extends StatelessWidget {
  const PlainControls({super.key});
  static const _w = 200.0, _listH = 138.0, _fieldH = 22.0;
  static const _blend = [
    'Normal', 'Dissolve', '-', 'Darken', 'Multiply', 'Color Burn', 'Linear Burn', 'Darker Color', '-', 'Add', 'Lighten', 'Screen', 'Color Dodge', 'Linear Dodge', 'Lighter Color', '-',
    'Overlay', 'Soft Light', 'Hard Light', 'Vivid Light', 'Linear Light', 'Pin Light', 'Hard Mix', '-', 'Difference', 'Exclusion', 'Subtract', 'Divide', '-', 'Hue', 'Saturation', 'Color', 'Luminosity',
  ];
  @override
  Widget build(BuildContext context) => Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        SizedBox(width: _w, child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
          const _NumField(),
          const SizedBox(height: 8),
          SizedBox(height: _listH, child: _PlainList(items: [for (var i = 1; i <= 30; i++) 'Layer ${i.toString().padLeft(2, '0')}'])),
        ])),
        const SizedBox(width: 12),
        const SizedBox(width: _w, height: _fieldH + 8 + _listH, child: _PlainList(items: _blend, filter: true)),
      ]);
}

class _NumField extends StatefulWidget {
  const _NumField();
  @override
  State<_NumField> createState() => _NumFieldState();
}

class _NumFieldState extends State<_NumField> {
  double v = 100;
  @override
  Widget build(BuildContext context) => Focus(
        onKeyEvent: (n, e) {
          if (e is KeyUpEvent) return KeyEventResult.ignored;
          if (e.logicalKey == LogicalKeyboardKey.arrowUp) {
            setState(() => v += 1);
          } else if (e.logicalKey == LogicalKeyboardKey.arrowDown) {
            setState(() => v -= 1);
          } else {
            return KeyEventResult.ignored;
          }
          return KeyEventResult.handled;
        },
        child: Builder(builder: (context) => GestureDetector(
              onTap: () => Focus.of(context).requestFocus(),
              onHorizontalDragUpdate: (e) => setState(() => v += e.delta.dx),
              child: Container(
                height: PlainControls._fieldH,
                padding: const EdgeInsets.symmetric(horizontal: 9),
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: Focus.of(context).hasFocus ? N.g44 : N.g20)),
                child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                  Text('Position X', style: T.label(N.g76)),
                  const Spacer(),
                  Text(v.toStringAsFixed(1), style: T.value(N.g95)),
                ]),
              ),
            )),
      );
}

class _PlainList extends StatefulWidget {
  const _PlainList({required this.items, this.filter = false});
  final List<String> items;
  final bool filter;
  @override
  State<_PlainList> createState() => _PlainListState();
}

class _PlainListState extends State<_PlainList> {
  int sel = 0;
  int? hover;
  String q = '';

  List<String> get _shown => [for (final s in widget.items) if (q.isEmpty ? true : s != '-' && s.toLowerCase().contains(q.toLowerCase())) s];

  @override
  Widget build(BuildContext context) {
    final rows = _shown, real = [for (final s in rows) if (s != '-') s];
    return Focus(
      onKeyEvent: (n, e) {
        if (e is KeyUpEvent) return KeyEventResult.ignored;
        final k = e.logicalKey;
        if (k == LogicalKeyboardKey.arrowDown) {
          setState(() => sel = math.min(sel + 1, real.length - 1));
        } else if (k == LogicalKeyboardKey.arrowUp) {
          setState(() => sel = math.max(sel - 1, 0));
        } else if (k == LogicalKeyboardKey.escape) {
          setState(() => q = '');
        } else if (widget.filter && k == LogicalKeyboardKey.backspace && q.isNotEmpty) {
          setState(() => q = q.substring(0, q.length - 1));
        } else if (widget.filter && e.character != null && e.character!.length == 1 && e.character!.codeUnitAt(0) >= 32) {
          setState(() {
            q += e.character!;
            sel = 0;
          });
        } else {
          return KeyEventResult.ignored;
        }
        return KeyEventResult.handled;
      },
      child: Builder(builder: (context) {
        var idx = -1;
        return GestureDetector(
          onTap: () => Focus.of(context).requestFocus(),
          child: DecoratedBox(
            decoration: BoxDecoration(color: N.g10, border: Border.all(color: Focus.of(context).hasFocus ? N.g44 : N.g20)),
            child: Flex(direction: Axis.vertical, children: [
              if (widget.filter) SizedBox(height: TL.height, child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: Align(alignment: Alignment.centerLeft, child: Text(q.isEmpty ? 'type to filter' : q, style: q.isEmpty ? T.label(N.g56) : T.value(N.g95))))),
              Expanded(
                child: ListView(padding: EdgeInsets.zero, children: [
                  for (var i = 0; i < rows.length; i++)
                    if (rows[i] == '-')
                      Container(height: TL.height, color: i.isEven ? N.bandA : N.bandB, alignment: Alignment.topCenter, child: Container(height: 1, color: N.g26))
                    else
                      Builder(builder: (context) {
                        final my = ++idx, on = my == sel;
                        return MouseRegion(
                          onEnter: (_) => setState(() => hover = my),
                          onExit: (_) => setState(() => hover = null),
                          child: GestureDetector(
                            onTap: () => setState(() => sel = my),
                            child: Container(
                              height: TL.height,
                              padding: const EdgeInsets.only(left: 10),
                              alignment: Alignment.centerLeft,
                              decoration: BoxDecoration(
                                color: on ? N.g20 : (hover == my ? N.g15 : (i.isEven ? N.bandA : N.bandB)),
                                border: Border(left: BorderSide(color: on ? N.g95 : const Color(0x00000000), width: 2), bottom: const BorderSide(color: N.rowLine)),
                              ),
                              child: Text(rows[i], style: T.name(on ? N.g95 : N.g76)),
                            ),
                          ),
                        );
                      }),
                ]),
              ),
            ]),
          ),
        );
      }),
    );
  }
}

// ---------------------------------------------------------------- 8 too-far meter

/// WCAG relative-luminance contrast of an opaque foreground on an opaque background.
double contrast(Color fg, Color bg) {
  final a = fg.computeLuminance(), b = bg.computeLuminance();
  return (math.max(a, b) + .05) / (math.min(a, b) + .05);
}

class Ink {
  const Ink(this.what, this.fg, this.bg);
  final String what;
  final Color fg, bg;
}

/// The parts of a screen a dose can season, with what each one costs in section-9 terms. Counting rules (the lab's reading of section 9):
/// one loud element = one framed selection, one kicker, one badge, one halftone strip, one spatial pad at a loud dose.
class Piece {
  const Piece(this.name, this.classes, this.loud, this.inks);
  final String name;
  final List<String> classes;
  final int Function(Dose) loud;
  final List<Ink> Function(Dose) inks;
}

final pieces = [
  Piece('Header', const ['header', 'kicker'], (d) => (d.kickerHeader ? 1 : 0) + (d.halftone ? 1 : 0), (d) => d.kickerHeader ? const [Ink('header kicker', N.g76, N.g13), Ink('header title', N.g95, N.g13)] : const [Ink('header title', N.g95, N.g13)]),
  Piece('Chips', const ['selection', 'kicker'], (d) => d.kickerHeader ? 2 : 0, (d) => d.kickerHeader ? [for (final f in Fam.all) Ink('chip kicker ${f.name}', f.c, N.g10)] : const []),
  Piece('Plates', const ['selection'], (d) => d.selHue ? 1 : 0, (d) => const []),
  Piece('Easing', const ['selection'], (d) => d.selHue ? 1 : 0, (d) => const []),
  Piece('Tile', const ['selection', 'kicker', 'badge'], (d) => (d.selHue ? 1 : 0) + (d.kickerItems ? 1 : 0) + (d.badgeLoud ? d.badgesMax : 0), (d) => [const Ink('tile name', N.g95, N.g13), const Ink('badge', N.g91, N.g13), if (d.kickerItems) const Ink('tile kicker', N.g63, N.g13)]),
  Piece('Pad', const ['pad'], (d) => d.level >= 2 ? 1 : 0, (d) => const [Ink('pad letters', N.g56, N.g07)]),
];

class TooFarMeter extends StatefulWidget {
  const TooFarMeter({super.key, required this.d});
  final Dose d;
  @override
  State<TooFarMeter> createState() => _TooFarMeterState();
}

class _TooFarMeterState extends State<TooFarMeter> {
  // default screen: an Inspector-like panel (header, chips, pad)
  final on = {'Header': true, 'Chips': true, 'Pad': true};

  static List<Ink> get _own => [Ink('meter label', N.g76, N.g13), Ink('meter note', N.g56, N.g13), Ink('meter value', N.g95, N.g13), Ink('meter over', Role.error, N.g13)];

  @override
  Widget build(BuildContext context) {
    final d = widget.d;
    final used = [for (final p in pieces) if (on[p.name] ?? false) p];
    final loud = used.fold<int>(0, (a, p) => a + p.loud(d));
    final inks = [for (final p in used) ...p.inks(d), ..._own];
    final worst = inks.reduce((a, b) => contrast(a.fg, a.bg) <= contrast(b.fg, b.bg) ? a : b);
    final ratio = contrast(worst.fg, worst.bg);
    final classes = {for (final p in used) ...p.classes};
    final early = [for (final k in classes) if (Dose.classMin[k]! > d.level) k];
    Widget row(String k, String v, bool bad, {String? note}) => Padding(
          padding: const EdgeInsets.only(bottom: Dose.gap),
          child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.start, children: [
            SizedBox(width: 128, child: Text(k, style: T.label(N.g76))),
            Expanded(child: bad ? Row(children: [const ErrMark(size: 10, gap: 5), Flexible(child: Text(v, style: T.value(Role.error)))]) : Text(v, style: T.value(N.g95))),
          ]),
        );
    return Container(
      width: 300,
      padding: const EdgeInsets.all(Dose.gapL + Dose.gap),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(Dose.radius), border: Border.all(color: N.g20)),
      child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text('TOO-FAR METER · DOSE ${d.level}', style: T.micro(N.g76)),
        const SizedBox(height: Dose.gapL),
        Wrap(spacing: Dose.gap, runSpacing: Dose.gap, children: [
          for (final p in pieces)
            GestureDetector(
              onTap: () => setState(() => on[p.name] = !(on[p.name] ?? false)),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: Dose.gapL, vertical: Dose.gap),
                decoration: BoxDecoration(color: (on[p.name] ?? false) ? N.g20 : null, borderRadius: BorderRadius.circular(Dose.radius), border: Border.all(color: (on[p.name] ?? false) ? N.g44 : N.g20)),
                child: Text(p.name, style: T.label((on[p.name] ?? false) ? N.g95 : N.g63)),
              ),
            ),
        ]),
        const SizedBox(height: Dose.gapL),
        row('loud elements', '$loud / ${d.loudMax}${loud > d.loudMax ? '  OVER' : ''}', loud > d.loudMax),
        row('worst text contrast', '${ratio.toStringAsFixed(2)} : 1 ${ratio < Dose.minContrast ? 'LOW' : 'ok'}\n${worst.what}', ratio < Dose.minContrast),
        ValueListenableBuilder<int>(valueListenable: DoseMotion.running, builder: (context, n, _) => row('motion running', '$n${n > 1 ? '  (limit 1)' : ''}', n > 1)),
        row('classes too early', early.isEmpty ? 'none' : early.join(', '), early.isNotEmpty),
        row('saturated pixels', 'limit ${d.satPctMax}% — measure on a screenshot', false),
        Text('Counts the toggled pieces at this dose. Motion is the number of selection / hover animations running in this window.', style: T.label(N.g56).copyWith(height: 1.3)),
      ]),
    );
  }
}
