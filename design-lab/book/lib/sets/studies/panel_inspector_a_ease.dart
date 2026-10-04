part of 'panel_inspector_a.dart';

// I5: choosing the ease between two keys. Four panels share one curve model, one plot, one loop of motion.

class _E {
  const _E(this.name, this.jp, this.f, {this.bez});
  final String name, jp;
  final double Function(double) f;
  final List<double>? bez; // x1 y1 x2 y2 of a cubic bezier, when the ease is one
  double at(double t) => bez != null ? _bezAt(bez!, t) : f(t);
  String get nums => bez == null ? 'not a bezier' : bez!.map((x) => _fmtNum(x, 3).replaceFirst(RegExp(r'0+$'), '').replaceFirst(RegExp(r'\.$'), '')).join(', ');
}

double _lin(double t) => t;

double _bezAt(List<double> b, double x) {
  if (x <= 0) {
    return 0;
  }
  if (x >= 1) {
    return 1;
  }
  double cx(double t) => 3 * (1 - t) * (1 - t) * t * b[0] + 3 * (1 - t) * t * t * b[2] + t * t * t;
  double cy(double t) => 3 * (1 - t) * (1 - t) * t * b[1] + 3 * (1 - t) * t * t * b[3] + t * t * t;
  var lo = 0.0, hi = 1.0;
  for (var i = 0; i < 28; i++) {
    final mid = (lo + hi) / 2;
    if (cx(mid) < x) {
      lo = mid;
    } else {
      hi = mid;
    }
  }
  return cy((lo + hi) / 2);
}

_E _cb(String name, String jp, double a, double b, double c, double d) => _E(name, jp, _lin, bez: [a, b, c, d]);

final List<_E> _presets = [
  _cb('Linear', 'リニア', 0, 0, 1, 1),
  _cb('Ease', '標準', .25, .1, .25, 1),
  _cb('Ease In', 'ゆっくり始まる', .42, 0, 1, 1),
  _cb('Ease Out', 'ゆっくり終わる', 0, 0, .58, 1),
  _cb('Ease In Out', '両端がゆっくり', .42, 0, .58, 1),
  _cb('Easy Ease', 'イージーイーズ', .33, 0, .67, 1),
  _cb('Sine In', 'なめらか', .47, 0, .745, .715),
  _cb('Sine Out', 'なめらか', .39, .575, .565, 1),
  _cb('Sine In Out', 'なめらか', .445, .05, .55, .95),
  _cb('Quad In', 'ふつう', .55, .085, .68, .53),
  _cb('Quad Out', 'ふつう', .25, .46, .45, .94),
  _cb('Quad In Out', 'ふつう', .455, .03, .515, .955),
  _cb('Cubic In', 'しっかり', .55, .055, .675, .19),
  _cb('Cubic Out', 'しっかり', .215, .61, .355, 1),
  _cb('Cubic In Out', 'しっかり', .645, .045, .355, 1),
  _cb('Quart In', '強め', .895, .03, .685, .22),
  _cb('Quart Out', '強め', .165, .84, .44, 1),
  _cb('Quart In Out', '強め', .77, 0, .175, 1),
  _cb('Quint In', 'かなり強い', .755, .05, .855, .06),
  _cb('Quint Out', 'かなり強い', .23, 1, .32, 1),
  _cb('Quint In Out', 'かなり強い', .86, 0, .07, 1),
  _cb('Expo In', '急に', .95, .05, .795, .035),
  _cb('Expo Out', '急に止まる', .19, 1, .22, 1),
  _cb('Expo In Out', '急に', 1, 0, 0, 1),
  _cb('Circ In', '円弧', .6, .04, .98, .335),
  _cb('Circ Out', '円弧', .075, .82, .165, 1),
  _cb('Circ In Out', '円弧', .785, .135, .15, .86),
  _cb('Back In', '一度引く', .6, -.28, .735, .045),
  _cb('Back Out', '行き過ぎて戻る', .175, .885, .32, 1.275),
  _cb('Back In Out', '引いて行き過ぎる', .68, -.55, .265, 1.55),
];

double _bounce(double t) {
  const n = 7.5625, d = 2.75;
  if (t < 1 / d) {
    return n * t * t;
  }
  if (t < 2 / d) {
    final u = t - 1.5 / d;
    return n * u * u + .75;
  }
  if (t < 2.5 / d) {
    final u = t - 2.25 / d;
    return n * u * u + .9375;
  }
  final u = t - 2.625 / d;
  return n * u * u + .984375;
}

final List<_E> _words = [
  _E('ふわっ', 'Float · cubic out, long tail', (t) => _bezAt(const [.16, 1, .3, 1], t)),
  _E('すっと', 'Glide · in-out', (t) => _bezAt(const [.45, 0, .55, 1], t)),
  _E('ぴたっ', 'Snap · expo out', (t) => _bezAt(const [.19, 1, .22, 1], t)),
  _E('どん', 'Slam · quint in', (t) => _bezAt(const [.755, .05, .855, .06], t)),
  _E('びくっ', 'Anticipate · back in out', (t) => _bezAt(const [.68, -.55, .265, 1.55], t)),
  _E('ばね', 'Spring · damping 0.35', (t) => t >= 1 ? 1 : 1 - math.exp(-6.5 * t) * math.cos(9.5 * t)),
  _E('ぽよん', 'Boing · damping 0.2', (t) => t >= 1 ? 1 : 1 - math.exp(-4.2 * t) * math.cos(13 * t)),
  _E('バウンド', 'Bounce · 3 hits', _bounce),
  _E('カクカク', 'Steps · 4', (t) => (t * 4).floor() / 4),
  _E('ゆらっ', 'Wobble · out + sway', (t) => _bezAt(const [.2, .8, .3, 1], t) + math.sin(t * math.pi * 3) * (1 - t) * .09),
];

// ---- drawing ----

const _y0 = -.6, _y1 = 1.6; // the plot shows y from -0.6 to 1.6 so an overshoot is visible

class _PlotPaint extends CustomPainter {
  const _PlotPaint(this.applied, this.peek, this.handles, this.hot);
  final _E applied;
  final _E? peek;
  final bool handles;
  final int? hot;
  static const pad = 14.0;
  Rect rect(Size z) => Rect.fromLTRB(pad, pad, z.width - pad, z.height - pad);
  Offset pt(Rect r, double x, double y) => Offset(r.left + x * r.width, r.bottom - (y - _y0) / (_y1 - _y0) * r.height);

  @override
  void paint(Canvas c, Size z) {
    final r = rect(z);
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & z, const Radius.circular(4)), Paint()..color = N.g07);
    final hair = Paint()..color = const Color(0x14FFFFFF)..strokeWidth = 1;
    for (var i = 1; i < 4; i++) {
      c.drawLine(Offset(r.left + r.width * i / 4, r.top), Offset(r.left + r.width * i / 4, r.bottom), hair);
    }
    for (final y in [0.0, 1.0]) {
      c.drawLine(pt(r, 0, y), pt(r, 1, y), Paint()..color = const Color(0x40FFFFFF)..strokeWidth = 1);
    }
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & z, const Radius.circular(4)).deflate(.5), Paint()..color = N.g20..style = PaintingStyle.stroke);
    Path path(_E e) {
      final p = Path();
      for (var i = 0; i <= 80; i++) {
        final t = i / 80, o = pt(r, t, e.at(t));
        i == 0 ? p.moveTo(o.dx, o.dy) : p.lineTo(o.dx, o.dy);
      }
      return p;
    }

    if (peek != null) {
      c.drawPath(path(peek!), Paint()..color = N.g63..style = PaintingStyle.stroke..strokeWidth = 1.2);
    }
    c.drawPath(path(applied), Paint()..color = peek != null ? N.g44 : N.g95..style = PaintingStyle.stroke..strokeWidth = peek != null ? 1 : 1.6);
    if (handles && applied.bez != null && peek == null) {
      final b = applied.bez!;
      final h1 = pt(r, b[0], b[1]), h2 = pt(r, b[2], b[3]);
      final ln = Paint()..color = N.g56..strokeWidth = 1;
      c.drawLine(pt(r, 0, 0), h1, ln);
      c.drawLine(pt(r, 1, 1), h2, ln);
      for (final (i, h) in [h1, h2].indexed) {
        c.drawCircle(h, hot == i ? 6 : 4.5, Paint()..color = hot == i ? N.g100 : N.g95);
        c.drawCircle(h, hot == i ? 6 : 4.5, Paint()..color = N.g07..style = PaintingStyle.stroke);
      }
    }
    final tp = TextPainter(textDirection: TextDirection.ltr);
    for (final (s, y) in [('1', 1.0), ('0', 0.0)]) {
      tp.text = TextSpan(text: s, style: T.label(N.g76));
      tp.layout();
      tp.paint(c, Offset(2, pt(r, 0, y).dy - tp.height / 2));
    }
  }

  @override
  bool shouldRepaint(_PlotPaint o) => true;
}

/// The curve plot: draws the applied ease, optionally a peek of another one on top, and (when [onBez] is set) two handles you can drag.
class _Plot extends StatefulWidget {
  const _Plot({required this.applied, this.peek, this.onBez, this.onEnd, this.w = 258, this.h = 150});
  final _E applied;
  final _E? peek;
  final void Function(List<double>)? onBez;
  final VoidCallback? onEnd;
  final double w, h;
  @override
  State<_Plot> createState() => _PlotState();
}

class _PlotState extends State<_Plot> {
  int? _hot;
  bool _drag = false;
  Rect get _r => const _PlotPaint(_E('', '', _lin), null, false, null).rect(Size(widget.w, widget.h));
  Offset _pt(double x, double y) => const _PlotPaint(_E('', '', _lin), null, false, null).pt(_r, x, y);

  int? _near(Offset p) {
    final b = widget.applied.bez;
    if (b == null || widget.onBez == null) {
      return null;
    }
    final d1 = (p - _pt(b[0], b[1])).distance, d2 = (p - _pt(b[2], b[3])).distance;
    if (math.min(d1, d2) > 14) {
      return null;
    }
    return d1 <= d2 ? 0 : 1;
  }

  void _move(Offset p) {
    final b = [...widget.applied.bez!], r = _r, i = _hot!;
    var x = ((p.dx - r.left) / r.width).clamp(0.0, 1.0);
    var y = _y0 + (r.bottom - p.dy) / r.height * (_y1 - _y0);
    y = y.clamp(_y0, _y1).toDouble();
    if (HardwareKeyboard.instance.isShiftPressed) {
      // Shift = lock an axis: keep the nearer of the two
      final ox = b[i * 2], oy = b[i * 2 + 1];
      if ((x - ox).abs() > (y - oy).abs()) {
        y = oy;
      } else {
        x = ox;
      }
    }
    b[i * 2] = _round(x);
    b[i * 2 + 1] = _round(y);
    widget.onBez!(b);
  }

  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: _hot != null ? SystemMouseCursors.grab : SystemMouseCursors.basic,
        onHover: (e) {
          final h = _near(e.localPosition);
          if (h != _hot && !_drag) {
            setState(() => _hot = h);
          }
        },
        onExit: (_) => setState(() => _hot = null),
        child: GestureDetector(
          onPanStart: (d) {
            final h = _near(d.localPosition);
            if (h != null) {
              _drag = true;
              setState(() => _hot = h);
            }
          },
          onPanUpdate: (d) {
            if (_drag && _hot != null) {
              _move(d.localPosition);
            }
          },
          onPanEnd: (_) {
            if (_drag) {
              _drag = false;
              widget.onEnd?.call();
            }
          },
          child: CustomPaint(size: Size(widget.w, widget.h), painter: _PlotPaint(widget.applied, widget.peek, widget.onBez != null, _hot)),
        ),
      );
}

class _ThumbPaint extends CustomPainter {
  const _ThumbPaint(this.e, this.on, this.t);
  final _E e;
  final bool on;
  final double? t;
  @override
  void paint(Canvas c, Size z) {
    const y0 = -.3, y1 = 1.3;
    Offset pt(double x, double y) => Offset(2 + x * (z.width - 4), z.height - 2 - (y - y0) / (y1 - y0) * (z.height - 4));
    final p = Path();
    for (var i = 0; i <= 40; i++) {
      final x = i / 40, o = pt(x, e.at(x));
      i == 0 ? p.moveTo(o.dx, o.dy) : p.lineTo(o.dx, o.dy);
    }
    c.drawLine(pt(0, 0), pt(1, 0), Paint()..color = const Color(0x1AFFFFFF));
    c.drawLine(pt(0, 1), pt(1, 1), Paint()..color = const Color(0x1AFFFFFF));
    c.drawPath(p, Paint()..color = on ? N.g95 : N.g63..style = PaintingStyle.stroke..strokeWidth = on ? 1.5 : 1);
    if (t != null) {
      c.drawCircle(pt(t!, e.at(t!)), 2.5, Paint()..color = N.g95);
    }
  }

  @override
  bool shouldRepaint(_ThumbPaint o) => o.e != e || o.on != on || o.t != t;
}

Widget _thumb(_E e, {double w = 40, double h = 22, bool on = false, double? t}) => CustomPaint(size: Size(w, h), painter: _ThumbPaint(e, on, t));

/// One loop of motion for everything on a panel: 1.0 s of movement and 0.6 s of rest. Reduced motion shows the end pose.
class _Loop extends StatefulWidget {
  const _Loop({required this.builder});
  final Widget Function(BuildContext, double) builder;
  @override
  State<_Loop> createState() => _LoopState();
}

class _LoopState extends State<_Loop> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 1600))..repeat();
  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final still = MediaQuery.maybeOf(context)?.disableAnimations ?? false;
    return AnimatedBuilder(animation: _c, builder: (context, _) => widget.builder(context, still ? 1.0 : (_c.value * 1.6).clamp(0.0, 1.0)));
  }
}

/// A short track with the dot of the applied ease and (while previewing) a ring of the peeked one.
class _TrackPaint extends CustomPainter {
  const _TrackPaint(this.t, this.a, this.b);
  final double t;
  final _E a;
  final _E? b;
  @override
  void paint(Canvas c, Size z) {
    final y = z.height / 2, x0 = 8.0, x1 = z.width - 8;
    c.drawLine(Offset(x0, y), Offset(x1, y), Paint()..color = N.g26..strokeWidth = 1);
    c.drawLine(Offset(x0, y - 5), Offset(x0, y + 5), Paint()..color = N.g44);
    c.drawLine(Offset(x1, y - 5), Offset(x1, y + 5), Paint()..color = N.g44);
    if (b != null) {
      c.drawCircle(Offset(x0 + (x1 - x0) * b!.at(t), y), 6, Paint()..color = N.g63..style = PaintingStyle.stroke);
    }
    c.drawCircle(Offset(x0 + (x1 - x0) * a.at(t), y), 4.5, Paint()..color = N.g95);
  }

  @override
  bool shouldRepaint(_TrackPaint o) => true;
}

Widget _track(_E a, {_E? peek, double w = 258}) => _Loop(builder: (_, t) => CustomPaint(size: Size(w, 22), painter: _TrackPaint(t, a, peek)));

/// The interval the ease is for: two keys of one property.
Widget _interval(double w) => Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          Expanded(child: Text('00:00:12', style: T.value(N.g91))),
          Text('1.6 s', style: T.label(N.g76)),
          Expanded(child: Text('00:01:00', textAlign: TextAlign.right, style: T.value(N.g91))),
        ]),
        const SizedBox(height: 4),
        Row(children: [
          Expanded(child: Text('120 px', style: T.value(N.g76))),
          Expanded(child: Text('840 px', textAlign: TextAlign.right, style: T.value(N.g76))),
        ]),
      ]),
    );

/// The keys of the property the interval belongs to. Rows in a gutter-padded body: the two keys of the interval are lit full-bleed (g13).
Widget _keysList(double w, {int sel = 1}) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      for (final (i, k) in const [('00:00:00', '60', 'Linear'), ('00:00:12', '120', 'Bezier'), ('00:01:00', '840', 'Bezier'), ('00:02:15', '840', 'Hold'), ('00:03:24', '300', 'Bezier')].indexed)
        _Bleed(
          height: 24,
          fill: i == sel || i == sel + 1 ? N.g13 : null,
          child: Row(children: [
            SizedBox(width: 64, child: Text(k.$1, style: T.value(N.g76))),
            Expanded(child: Text(k.$2, style: T.value(N.g91))),
            Text(k.$3, style: T.label(N.g76)),
          ]),
        ),
    ]);

/// A one-line slider in the lab's grey: the value is the loudest thing. Edge ladder as a number cell: rest g20, hover g38, focus g76 (← → move it, Shift finer), pressed g63.
class _Slide extends StatefulWidget {
  const _Slide({required this.label, required this.value, required this.onChanged, this.hot = false, this.focus = false, this.press = false});
  final String label;
  final double value; // 0..1
  final bool hot, focus, press; // pinned states (Parts)
  final ValueChanged<double> onChanged;
  @override
  State<_Slide> createState() => _SlideState();
}

class _SlideState extends State<_Slide> {
  bool _f = false, _p = false, _h = false;
  final _node = FocusNode(debugLabel: 'slide');
  @override
  void dispose() {
    _node.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        void at(Offset p) => widget.onChanged((p.dx / box.maxWidth).clamp(0.0, 1.0));
        final v = widget.value, pose = widget.press || _p ? _Pose.press : (widget.focus || _f ? _Pose.focus : (widget.hot || _h ? _Pose.hover : _Pose.rest));
        return Focus(
          focusNode: _node,
          onFocusChange: (f) => setState(() => _f = f),
          onKeyEvent: (_, e) {
            if (e is KeyUpEvent) {
              return KeyEventResult.ignored;
            }
            final d = HardwareKeyboard.instance.isShiftPressed ? .01 : .05;
            if (e.logicalKey == LogicalKeyboardKey.arrowRight || e.logicalKey == LogicalKeyboardKey.arrowUp) {
              widget.onChanged((v + d).clamp(0.0, 1.0));
              return KeyEventResult.handled;
            }
            if (e.logicalKey == LogicalKeyboardKey.arrowLeft || e.logicalKey == LogicalKeyboardKey.arrowDown) {
              widget.onChanged((v - d).clamp(0.0, 1.0));
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: MouseRegion(
            cursor: SystemMouseCursors.resizeLeftRight,
            onEnter: (_) => setState(() => _h = true),
            onExit: (_) => setState(() => _h = false),
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanDown: (d) {
                setState(() => _p = true);
                _node.requestFocus();
                at(d.localPosition);
              },
              onPanUpdate: (d) => at(d.localPosition),
              onPanEnd: (_) => setState(() => _p = false),
              onPanCancel: () => setState(() => _p = false),
              child: SizedBox(
                height: _cellH,
                child: Stack(children: [
                  Positioned.fill(child: DecoratedBox(decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: _faceLine(pose))))),
                  Positioned(left: 0, top: 0, bottom: 0, width: box.maxWidth * v, child: DecoratedBox(decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(4)))),
                  Positioned(left: (box.maxWidth * v - 1).clamp(0.0, box.maxWidth - 2).toDouble(), top: 4, bottom: 4, width: 2, child: const DecoratedBox(decoration: BoxDecoration(color: N.g95))),
                  Positioned.fill(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: Row(children: [Text(widget.label, style: T.label(N.g76)), const Spacer(), Text('${(v * 100).round()} %', style: T.value(N.g95))]))),
                ]),
              ),
            ),
          ),
        );
      });
}

// ---- I5-a: named presets + handles ----

class _EaseA extends StatefulWidget {
  const _EaseA({this.w = _panelW});
  final double w;
  @override
  State<_EaseA> createState() => _EaseAState();
}

class _EaseAState extends State<_EaseA> {
  _E _applied = _presets[28];
  int? _peek;
  int _steps = 0;
  String _log = '';
  final _fn = FocusNode(debugLabel: 'ease-list');
  List<double>? _before;

  @override
  void dispose() {
    _fn.dispose();
    super.dispose();
  }

  void _apply(int i) => setState(() {
        _applied = _presets[i];
        _steps++;
        _log = 'wrote ${_presets[i].name} to the interval';
        _peek = null;
      });

  @override
  Widget build(BuildContext context) {
    final dock = widget.w < 250, w = widget.w - 2 * _gutter - 2;
    final peekE = _peek == null ? null : _presets[_peek!];
    final custom = _applied.name == 'custom';
    return Focus(
      focusNode: _fn,
      onKeyEvent: (_, e) {
        if (e is! KeyDownEvent) {
          return KeyEventResult.ignored;
        }
        if (e.logicalKey == LogicalKeyboardKey.arrowDown || e.logicalKey == LogicalKeyboardKey.arrowUp) {
          final d = e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1;
          setState(() => _peek = ((_peek ?? (d > 0 ? -1 : 0)) + d).clamp(0, _presets.length - 1));
          return KeyEventResult.handled;
        }
        if (e.logicalKey == LogicalKeyboardKey.enter && _peek != null) {
          _apply(_peek!);
          return KeyEventResult.handled;
        }
        if (e.logicalKey == LogicalKeyboardKey.escape) {
          setState(() {
            _peek = null;
            _log = 'Esc: peek removed, nothing written';
          });
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: _Frame(
        w: widget.w,
        head: const _Head(name: 'Position X', kind: 'Key interval'),
        foot: _FootBar(_log, steps: _steps),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 8),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _interval(w),
              const SizedBox(height: 8),
              Align(alignment: Alignment.centerLeft, child: _Plot(
                applied: _applied,
                peek: peekE,
                w: w,
                h: dock ? 120 : 148,
                onBez: (b) => setState(() {
                  _before ??= _applied.bez;
                  _applied = _E('custom', 'カスタム', _lin, bez: b);
                }),
                onEnd: () => setState(() {
                  _steps++;
                  _log = 'handle released: 1 undo step';
                  _before = null;
                }),
              )),
              const SizedBox(height: 8),
              Text(peekE != null ? peekE.name : (custom ? 'Custom' : _applied.name), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(peekE != null ? N.g76 : N.g95).copyWith(decoration: peekE != null ? TextDecoration.underline : TextDecoration.none, decorationStyle: TextDecorationStyle.dotted, decorationColor: N.g63)),
              const SizedBox(height: 4),
              Text(peekE != null ? peekE.nums : _applied.nums, maxLines: 1, style: T.value(N.g76)),
              const SizedBox(height: 8),
              _track(_applied, peek: peekE, w: w),
            ]),
          ),
          Container(height: 1, color: _rule),
          Padding(
            padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 4),
            child: Row(children: [
              Expanded(child: Text('PRESETS  プリセット', style: T.micro(N.g76).copyWith(letterSpacing: 1))),
              Text('${_presets.length}', style: T.label(N.g76)),
            ]),
          ),
          Expanded(
            child: MouseRegion(
              onExit: (_) => setState(() => _peek = null),
              child: ShaderMask(
                // the last visible row fades out instead of ending mid-height: the list goes on
                blendMode: BlendMode.dstIn,
                shaderCallback: (r) => LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: const [Color(0xFFFFFFFF), Color(0xFFFFFFFF), Color(0x00FFFFFF)], stops: [0, math.max(0, (r.height - 20) / r.height), 1]).createShader(r),
                child: ListView.builder(
                padding: const EdgeInsets.fromLTRB(_gutter, 0, _gutter, 16),
                itemCount: _presets.length,
                itemExtent: 28,
                itemBuilder: (_, i) {
                  final e = _presets[i], on = _applied.name == e.name, pk = _peek == i;
                  return MouseRegion(
                    onEnter: (_) => setState(() => _peek = i),
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: () {
                        _fn.requestFocus();
                        _apply(i);
                      },
                      child: _PresetRow(e, on: on, peek: pk, dock: dock, gloss: i == 0 || _presets[i - 1].jp != e.jp),
                    ),
                  );
                },
              ),
              ),
            ),
          ),
        ]),
      ),
    );
  }
}

// ---- I5-b: a direction and one strength ----

class _EaseB extends StatefulWidget {
  const _EaseB();
  @override
  State<_EaseB> createState() => _EaseBState();
}

class _EaseBState extends State<_EaseB> {
  int _dir = 1; // 0 in, 1 out, 2 both
  double _s = .55;
  int _steps = 0;
  @override
  Widget build(BuildContext context) {
    final infl = _s * .9;
    final bez = switch (_dir) { 0 => [infl, 0.0, 1.0, 1.0], 1 => [0.0, 0.0, 1 - infl, 1.0], _ => [infl, 0.0, 1 - infl, 1.0] };
    final e = _E('custom', '', _lin, bez: bez);
    final outInfl = _dir == 1 ? 0 : (infl * 100).round(), inInfl = _dir == 0 ? 0 : (infl * 100).round();
    const w = _panelW - 2 * _gutter - 2;
    return _Frame(
      w: _panelW,
      head: const _Head(name: 'Position X', kind: 'Key interval'),
      foot: _FootBar(_steps == 0 ? '' : const ['Ease in', 'Ease out', 'Both'][_dir], steps: _steps),
      child: _Body(children: [
        _interval(w),
        _gap(12),
        _Plot(applied: e, w: w, h: 148),
        _gap(12),
        Segmented(items: const ['Ease in', 'Ease out', 'Both'], index: _dir, expand: true, onChanged: (i) => setState(() {
              _dir = i;
              _steps++;
            })),
        _gap(8),
        _Slide(label: 'Strength', value: _s, onChanged: (v) => setState(() => _s = _round(v))),
        _gap(8),
        Row(children: [
          Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Text('Out', style: T.label(N.g76)), _gap(4), Text('$outInfl %', style: T.value(N.g95))])),
          Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Text('In', style: T.label(N.g76)), _gap(4), Text('$inInfl %', style: T.value(N.g95))])),
        ]),
        _gap(12),
        _track(e, w: w),
        _Sec('Keys  キー', [_keysList(w)]),
      ]),
    );
  }
}

// ---- I5-c: a dictionary of motions, each tile moves ----

class _EaseC extends StatefulWidget {
  const _EaseC({this.w = _panelW});
  final double w;
  @override
  State<_EaseC> createState() => _EaseCState();
}

class _EaseCState extends State<_EaseC> {
  int _sel = 0;
  int _steps = 0;
  @override
  Widget build(BuildContext context) {
    final dock = widget.w < 250, w = widget.w - 2 * _gutter - 2, cols = dock ? 1 : 2, tileW = (w - (cols - 1) * 8) / cols;
    return _Frame(
      w: widget.w,
      head: const _Head(name: 'Position X', kind: 'Key interval'),
      foot: _FootBar(_steps == 0 ? '' : _words[_sel].name, steps: _steps),
      child: _Body(children: [
        _interval(w),
        _Sec('Motion words  動きの言葉', [
          Wrap(spacing: 8, runSpacing: 8, children: [
            for (final (i, e) in _words.indexed)
              _Ctl(
                onTap: () => setState(() {
                  _sel = i;
                  _steps++;
                }),
                builder: (_, h, f, p) => _Tile(e, w: tileW, sel: _sel == i, hot: h, focus: f, press: p),
              ),
          ]),
        ]),
        _Sec('Keys  キー', [_keysList(w)]),
      ]),
    );
  }
}

// ---- I5-d: measure once, paste to many ----

class _Iv {
  const _Iv(this.prop, this.from, this.to, this.e);
  final String prop, from, to;
  final _E e;
}

class _EaseD extends StatefulWidget {
  const _EaseD({this.w = _panelW});
  final double w;
  @override
  State<_EaseD> createState() => _EaseDState();
}

class _EaseDState extends State<_EaseD> {
  late List<_Iv> _iv = _intervals();

  static List<_Iv> _intervals() {
    const props = ['Position X', 'Position Y', 'Scale', 'Rotation', 'Opacity', 'Glow Radius', 'Blurriness', 'Wave Height', 'Amount'];
    const times = ['00:00', '00:12', '01:00', '02:00'];
    const pick = [28, 1, 0, 0, 3, 0, 22, 4, 0, 13, 5, 0];
    return [
      for (final (i, p) in props.indexed)
        for (var k = 0; k < 3; k++)
          if ((i + k) % 5 != 4) _Iv(p, times[k], times[k + 1], _presets[pick[(i * 3 + k) % pick.length]]),
    ];
  }
  final Set<int> _sel = {0};
  _E? _clip;
  int _steps = 0;
  String _log = '';

  void _tap(int i) {
    setState(() {
      if (HardwareKeyboard.instance.isShiftPressed || HardwareKeyboard.instance.isMetaPressed) {
        _sel.contains(i) ? _sel.remove(i) : _sel.add(i);
      } else {
        _sel
          ..clear()
          ..add(i);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final dock = widget.w < 250;
    final canCopy = _sel.length == 1, canPaste = _clip != null && _sel.isNotEmpty;
    return _Frame(
      w: widget.w,
      head: const _Head(name: 'Jewel Field', kind: 'Shape group', sub: 'Key intervals'),
      foot: _FootBar(_log, steps: _steps),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _Clip(_clip),
        Padding(
          padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 8),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Row(children: [
              Expanded(child: _Act('Copy ease', on: canCopy, onTap: () => setState(() {
                    _clip = _iv[_sel.first].e;
                    _log = 'copied ${_clip!.name}';
                  }))),
              const SizedBox(width: 8),
              Expanded(child: _Act(dock ? 'Paste ${_sel.length}' : 'Paste to ${_sel.length}', on: canPaste, onTap: () => setState(() {
                    _iv = [for (final (i, v) in _iv.indexed) _sel.contains(i) ? _Iv(v.prop, v.from, v.to, _clip!) : v];
                    _steps++;
                    _log = 'pasted ${_clip!.name} to ${_sel.length} intervals: 1 undo step';
                  }))),
            ]),
          ]),
        ),
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.fromLTRB(_gutter, 0, _gutter, 16),
            itemCount: _iv.length,
            itemExtent: 32,
            itemBuilder: (_, i) {
              final v = _iv[i], on = _sel.contains(i);
              return _Ctl(onTap: () => _tap(i), builder: (_, h, f, p) => _IvRow(v, on: on, hot: h, focus: f, press: p, dock: dock));
            },
          ),
        ),
      ]),
    );
  }
}

/// The clipboard of I5-d, as a chip: a dashed empty slot, or the copied ease (thumb, name, numbers).
class _Clip extends StatelessWidget {
  const _Clip(this.e);
  final _E? e;
  @override
  Widget build(BuildContext context) => Container(
        height: 32,
        padding: const EdgeInsets.symmetric(horizontal: _gutter),
        decoration: BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: _rule))),
        child: Row(children: [
          SizedBox(width: 64, child: Text('CLIPBOARD', maxLines: 1, style: T.micro(N.g76).copyWith(letterSpacing: .3))),
          if (e == null)
            const CustomPaint(size: Size(36, 20), painter: _SlotPaint())
          else ...[
            _thumb(e!, w: 36, h: 20, on: true),
            const SizedBox(width: 8),
            Expanded(child: Text(e!.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g91))),
            Text(e!.nums, maxLines: 1, style: T.value(N.g76)),
          ],
        ]),
      );
}

/// An empty slot: a dashed 1 px outline.
class _SlotPaint extends CustomPainter {
  const _SlotPaint();
  @override
  void paint(Canvas c, Size z) => _dashed(c, Path()..addRRect(RRect.fromRectAndRadius((Offset.zero & z).deflate(.5), const Radius.circular(3))), Paint()..color = N.g44..style = PaintingStyle.stroke);

  @override
  bool shouldRepaint(_SlotPaint o) => false;
}

/// A word button (Copy ease, Paste to n, Show all rows): rest, hover, focus, pressed, off. See [_Btn].
class _Act extends StatelessWidget {
  const _Act(this.text, {required this.on, required this.onTap, this.hot = false, this.focus = false, this.pressed = false});
  final String text;
  final bool on, hot, focus, pressed; // hot / focus / pressed pin a state (Parts)
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Ctl(
        enabled: on,
        onTap: onTap,
        hover: hot ? true : null,
        focus: focus ? true : null,
        pressed: pressed ? true : null,
        builder: (_, h, f, p) => _Btn(text, on: on, hover: h, focus: f, pressed: p),
      );
}

/// One row of the preset list: a thumb, the name, the Japanese word. Selected = g20 + tick; peeked (hover or the arrow keys) = g15; pressed = g26. A list row has no focus of its own: the peek is the keyboard cursor.
class _PresetRow extends StatelessWidget {
  const _PresetRow(this.e, {this.on = false, this.peek = false, this.press = false, this.dock = false, this.gloss = true});
  final _E e;
  final bool on, peek, press, dock;

  /// The Japanese word shows once per run of rows that share it (the same word on every row is no information).
  final bool gloss;
  @override
  Widget build(BuildContext context) => _Bleed(
        height: 28,
        fill: press ? N.g26 : (on ? N.g20 : (peek ? N.g15 : null)),
        tick: on,
        child: Row(children: [
          _thumb(e, w: 36, h: 20, on: on || peek || press),
          const SizedBox(width: 8),
          Expanded(child: Text(e.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g91))),
          if (!dock && gloss) ...[const SizedBox(width: 8), ConstrainedBox(constraints: const BoxConstraints(maxWidth: 88), child: Text(e.jp, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)))],
        ]),
      );
}

/// One interval in the copy list (I5-d): property, times, thumb, ease name. Selected = g20 + tick; hover g15; focus 1 px g76 edge inside the row; pressed g26.
class _IvRow extends StatelessWidget {
  const _IvRow(this.v, {this.on = false, this.hot = false, this.focus = false, this.press = false, this.dock = false});
  final _Iv v;
  final bool on, hot, focus, press, dock;
  @override
  Widget build(BuildContext context) => _Bleed(
        height: 32,
        fill: press ? N.g26 : (on ? N.g20 : (hot ? N.g15 : null)),
        tick: on,
        child: Container(
          foregroundDecoration: focus ? BoxDecoration(border: Border.all(color: N.g76)) : null,
          child: Row(children: [
            SizedBox(width: dock ? 64 : 76, child: Text(v.prop, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g95))),
            if (!dock) SizedBox(width: 88, child: Text('${v.from} → ${v.to}', maxLines: 1, style: T.label(N.g76))),
            _thumb(v.e, w: 36, h: 20, on: on),
            const SizedBox(width: 8),
            Expanded(child: Text(v.e.name, maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.right, style: T.label(on ? N.g95 : N.g76))),
          ]),
        ),
      );
}

/// One motion word as a tile (I5-c). rest g13 + g20 edge; hover g44 edge; focus g95 edge; pressed g07 fill + g63 edge; selected g20 fill + accent edge. [t] is the moment of the loop.
class _Tile extends StatelessWidget {
  const _Tile(this.e, {required this.w, this.sel = false, this.hot = false, this.focus = false, this.press = false, this.t});
  final _E e;
  final double w;
  final bool sel, hot, focus, press;
  final double? t; // null = the running loop
  @override
  Widget build(BuildContext context) => Container(
        width: w,
        height: 80,
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: sel ? N.g20 : (press ? N.g07 : N.g13),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: sel ? Role.selected : (focus ? N.g95 : (press ? N.g63 : (hot ? N.g44 : N.g20)))),
        ),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text(e.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title(N.g95)),
          const SizedBox(height: 4),
          Text(e.jp, maxLines: 2, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(height: 1.2)),
          const Spacer(),
          t == null ? _Loop(builder: (_, t) => CustomPaint(size: Size(w - 18, 14), painter: _TrackPaint(t, e, null))) : CustomPaint(size: Size(w - 18, 14), painter: _TrackPaint(t!, e, null)),
        ]),
      );
}
