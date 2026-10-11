part of 'panel_inspector_a.dart';

// I3: instruments that edit the same _M as the number cells (one value, many hands). A mini Stage stands next to the panel in I3-a and I3-d only to show the contract; it is local to the use case.

const _gSpace = _Grp('3D  空間', [
  _R('Z Position', [_P('pos.z', '', 0, unit: 'px', dec: 1)]),
  _R('X Rotation', [_P('rot.x', '', 0, unit: '°', dec: 1)]),
  _R('Y Rotation', [_P('rot.y', '', 0, unit: '°', dec: 1)]),
]);

_M _spaceModel() => _M(groups: [..._allGroups, _gSpace], keys: const {});

const _comp = Size(1920, 1080);
const _bodyHalf = Size(180, 110); // the jewel at 100 %, half extents in comp px

/// The edge of an instrument: rest g20, hover g38, held g63 (the same ladder as a number field).
Color _hotLine(int hot) => const [N.g20, N.g38, N.g63][hot];

double _fine() => HardwareKeyboard.instance.isShiftPressed ? .1 : 1; // I3-b: Shift = finer on an instrument

// ---- the pad ----

class _Pad extends StatefulWidget {
  const _Pad(this.m, {this.w = 258, this.h = 112});
  final _M m;
  final double w, h;
  @override
  State<_Pad> createState() => _PadState();
}

class _PadState extends State<_Pad> {
  bool _live = false, _hover = false;
  double get _s => math.min((widget.w - 24) / _comp.width, (widget.h - 16) / _comp.height);
  @override
  Widget build(BuildContext context) {
    final m = widget.m;
    return MouseRegion(
      cursor: SystemMouseCursors.precise,
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      child: GestureDetector(
        onPanStart: (_) {
          setState(() => _live = true);
          m.begin();
        },
        onPanUpdate: (d) {
          if (_live) {
            final k = _fine() / _s;
            m.add('pos.x', d.delta.dx * k);
            m.add('pos.y', d.delta.dy * k);
          }
        },
        onPanEnd: (_) {
          setState(() => _live = false);
          m.end('pad: Position');
        },
        onPanCancel: () {
          setState(() => _live = false);
          m.end('pad: Position');
        },
        child: CustomPaint(size: Size(widget.w, widget.h), painter: _PadPaint(m.at('pos.x'), m.at('pos.y'), _s, m.changed('pos.x') || m.changed('pos.y'), _live ? 2 : (_hover ? 1 : 0))),
      ),
    );
  }
}

class _PadPaint extends CustomPainter {
  const _PadPaint(this.x, this.y, this.s, this.changed, [this.hot = 0]);
  final double x, y, s;
  final bool changed;
  final int hot;
  @override
  void paint(Canvas c, Size z) {
    final box = Rect.fromLTWH(0, 0, z.width, z.height);
    c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(4)), Paint()..color = N.g07);
    final comp = Rect.fromCenter(center: box.center, width: _comp.width * s, height: _comp.height * s);
    c.drawRect(comp, Paint()..color = N.g13);
    final hair = Paint()..color = const Color(0x1AFFFFFF)..strokeWidth = 1;
    c.drawLine(Offset(comp.left + comp.width / 3, comp.top), Offset(comp.left + comp.width / 3, comp.bottom), hair);
    c.drawLine(Offset(comp.left + comp.width * 2 / 3, comp.top), Offset(comp.left + comp.width * 2 / 3, comp.bottom), hair);
    c.drawLine(Offset(comp.left, comp.top + comp.height / 2), Offset(comp.right, comp.top + comp.height / 2), hair);
    c.drawRect(comp, Paint()..color = N.g26..style = PaintingStyle.stroke);
    c.drawRRect(RRect.fromRectAndRadius(box.deflate(.5), const Radius.circular(4)), Paint()..color = _hotLine(hot)..style = PaintingStyle.stroke);
    final p = Offset(comp.left + x * s, comp.top + y * s).dx.isFinite ? Offset((comp.left + x * s).clamp(4.0, z.width - 4), (comp.top + y * s).clamp(4.0, z.height - 4)) : box.center;
    final cross = Paint()..color = const Color(0x4DFFFFFF)..strokeWidth = 1;
    c.drawLine(Offset(p.dx, 0), Offset(p.dx, z.height), cross);
    c.drawLine(Offset(0, p.dy), Offset(z.width, p.dy), cross);
    final r = hot == 2 ? 6.0 : 5.0;
    if (changed || hot > 0) {
      c.drawCircle(p, r, Paint()..color = N.g95); // touched: filled, with an outer ring
      c.drawCircle(p, r + 3, Paint()..color = N.g38..style = PaintingStyle.stroke);
    } else {
      c.drawCircle(p, r, Paint()..color = N.g07); // untouched: hollow
      c.drawCircle(p, r, Paint()..color = N.g76..style = PaintingStyle.stroke..strokeWidth = 1.2);
    }
  }

  @override
  bool shouldRepaint(_PadPaint o) => o.x != x || o.y != y || o.s != s || o.changed != changed || o.hot != hot;
}

// ---- the dial ----

class _Dial extends StatefulWidget {
  const _Dial(this.m, {this.size = 96});
  final _M m;
  final double size;
  @override
  State<_Dial> createState() => _DialState();
}

class _DialState extends State<_Dial> {
  double _prev = 0;
  bool _live = false, _hover = false;
  double _ang(Offset p) => math.atan2(p.dy - widget.size / 2, p.dx - widget.size / 2);
  @override
  Widget build(BuildContext context) {
    final m = widget.m;
    return MouseRegion(
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      child: GestureDetector(
      onPanStart: (d) {
        setState(() => _live = true);
        _prev = _ang(d.localPosition);
        m.begin();
      },
      onPanUpdate: (d) {
        if (!_live) {
          return;
        }
        var da = _ang(d.localPosition) - _prev;
        while (da > math.pi) {
          da -= 2 * math.pi;
        }
        while (da < -math.pi) {
          da += 2 * math.pi;
        }
        _prev = _ang(d.localPosition);
        m.add('rot', da * 180 / math.pi * _fine());
      },
      onPanEnd: (_) {
        setState(() => _live = false);
        m.end('dial: Rotation');
      },
      onPanCancel: () {
        setState(() => _live = false);
        m.end('dial: Rotation');
      },
      child: CustomPaint(size: Size.square(widget.size), painter: _DialPaint(m.at('rot'), m.changed('rot'), _live ? 2 : (_hover ? 1 : 0))),
      ),
    );
  }
}

class _DialPaint extends CustomPainter {
  const _DialPaint(this.deg, this.changed, [this.hot = 0]);
  final double deg;
  final bool changed;
  final int hot;
  @override
  void paint(Canvas c, Size z) {
    final o = z.center(Offset.zero), r = z.width / 2 - 3;
    c.drawCircle(o, r, Paint()..color = N.g07);
    c.drawCircle(o, r, Paint()..color = _hotLine(hot)..style = PaintingStyle.stroke);
    for (var i = 0; i < 24; i++) {
      final a = i * math.pi / 12, major = i % 6 == 0;
      c.drawLine(o + Offset(math.cos(a), math.sin(a)) * (r - (major ? 7 : 3)), o + Offset(math.cos(a), math.sin(a)) * r, Paint()..color = major ? N.g44 : N.g26..strokeWidth = 1);
    }
    final a = (deg - 90) * math.pi / 180;
    final hand = changed || hot > 0 ? N.g95 : N.g76;
    c.drawLine(o, o + Offset(math.cos(a), math.sin(a)) * (r - 4), Paint()..color = hand..strokeWidth = 1.5);
    final knob = o + Offset(math.cos(a), math.sin(a)) * (r - 4), kr = hot == 2 ? 4.5 : 3.5;
    if (changed || hot > 0) {
      c.drawCircle(knob, kr, Paint()..color = hand);
    } else {
      c.drawCircle(knob, kr, Paint()..color = N.g07);
      c.drawCircle(knob, kr, Paint()..color = hand..style = PaintingStyle.stroke..strokeWidth = 1.2);
    }
    c.drawCircle(o, 2, Paint()..color = N.g63);
  }

  @override
  bool shouldRepaint(_DialPaint o) => o.deg != deg || o.changed != changed || o.hot != hot;
}

// ---- the scale box ----

class _ScaleBox extends StatefulWidget {
  const _ScaleBox(this.m, {this.size = 96});
  final _M m;
  final double size;
  @override
  State<_ScaleBox> createState() => _ScaleBoxState();
}

class _ScaleBoxState extends State<_ScaleBox> {
  bool _live = false, _hover = false;
  @override
  Widget build(BuildContext context) {
    final m = widget.m;
    return MouseRegion(
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      child: GestureDetector(
      onPanStart: (_) {
        setState(() => _live = true);
        m.begin();
      },
      onPanUpdate: (d) {
        if (_live) {
          final k = (d.delta.dx - d.delta.dy) * .5 * _fine();
          m.add('scale.x', k);
          m.add('scale.y', k);
        }
      },
      onPanEnd: (_) {
        setState(() => _live = false);
        m.end('box: Scale (linked)');
      },
      onPanCancel: () {
        setState(() => _live = false);
        m.end('box: Scale (linked)');
      },
      child: CustomPaint(size: Size.square(widget.size), painter: _ScalePaint(m.at('scale.x') / 100, m.changed('scale.x'), _live ? 2 : (_hover ? 1 : 0))),
      ),
    );
  }
}

class _ScalePaint extends CustomPainter {
  const _ScalePaint(this.k, this.changed, [this.hot = 0]);
  final double k;
  final bool changed;
  final int hot;
  @override
  void paint(Canvas c, Size z) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & z, const Radius.circular(4)), Paint()..color = N.g07);
    c.drawRRect(RRect.fromRectAndRadius((Offset.zero & z).deflate(.5), const Radius.circular(4)), Paint()..color = _hotLine(hot)..style = PaintingStyle.stroke);
    final o = z.center(Offset.zero), base = z.width * .26;
    c.drawRect(Rect.fromCenter(center: o, width: base * 2, height: base * 2), Paint()..color = N.g26..style = PaintingStyle.stroke);
    final h = (base * k.clamp(.1, 1.6)).toDouble();
    final r = Rect.fromCenter(center: o, width: h * 2, height: h * 2);
    c.drawRect(r, Paint()..color = N.g20);
    final ink = changed || hot > 0 ? N.g95 : N.g76;
    c.drawRect(r, Paint()..color = ink..style = PaintingStyle.stroke);
    for (final q in [r.topLeft, r.topRight, r.bottomLeft, r.bottomRight]) {
      final h = Rect.fromCenter(center: q, width: hot == 2 ? 8 : 6, height: hot == 2 ? 8 : 6);
      if (changed || hot > 0) {
        c.drawRect(h, Paint()..color = ink);
      } else {
        c.drawRect(h, Paint()..color = N.g07);
        c.drawRect(h, Paint()..color = ink..style = PaintingStyle.stroke);
      }
    }
  }

  @override
  bool shouldRepaint(_ScalePaint o) => o.k != k || o.changed != changed || o.hot != hot;
}

// ---- the nine points of the anchor ----

class _Anchor9 extends StatelessWidget {
  const _Anchor9(this.m, {this.hotPoint});
  final _M m;
  final int? hotPoint; // a specimen's hover
  @override
  Widget build(BuildContext context) {
    final ax = m.at('anchor.x'), ay = m.at('anchor.y');
    int? hit;
    for (var i = 0; i < 9; i++) {
      if ((ax - (i % 3) * 960).abs() < .5 && (ay - (i ~/ 3) * 540).abs() < .5) {
        hit = i;
      }
    }
    return Container(
      width: 72,
      height: 52,
      decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
      child: Stack(children: [
        for (var i = 0; i < 9; i++)
          Positioned(
            left: 8 + (i % 3) * 25.0 - 12,
            top: 6 + (i ~/ 3) * 20.0 - 10,
            width: 24,
            height: 20,
            child: Hov(
              onTap: () {
                m.begin();
                m.set('anchor.x', (i % 3) * 960.0, notify: false);
                m.set('anchor.y', (i ~/ 3) * 540.0, notify: false);
                m.end('anchor: ${const ['Top left', 'Top', 'Top right', 'Left', 'Centre', 'Right', 'Bottom left', 'Bottom', 'Bottom right'][i]}');
              },
              builder: (_, h0) {
                final h = h0 || hotPoint == i;
                final d = hit == i ? 6.0 : (h ? 7.0 : 5.0);
                return Center(child: Container(width: hit == i ? 12 : d, height: hit == i ? 12 : d, alignment: Alignment.center, decoration: hit == i ? BoxDecoration(border: Border.all(color: N.g63), shape: BoxShape.circle) : null, child: Container(width: d, height: d, decoration: BoxDecoration(color: hit == i ? N.g95 : (h ? N.g76 : N.g44), shape: BoxShape.circle))));
              },
            ),
          ),
      ]),
    );
  }
}

// ---- the mock Stage (I3-a, I3-d): a layer you can grab, drawn from the same _M ----

enum _Hand { none, pos, rot, scale }

class _Stage extends StatefulWidget {
  const _Stage(this.m, {this.hand = _Hand.none, this.interactive = true});
  final _M m;
  static const w = 336.0;
  final _Hand hand;
  final bool interactive;
  @override
  State<_Stage> createState() => _StageState();
}

class _StageState extends State<_Stage> {
  _Hand _mode = _Hand.none;
  double _prevA = 0;
  double get _s => _Stage.w / _comp.width;
  Offset get _pos => Offset(widget.m.at('pos.x') * _s, widget.m.at('pos.y') * _s);
  List<Offset> _corners() {
    final m = widget.m, k = m.at('scale.x') / 100, a = m.at('rot') * math.pi / 180;
    return [
      for (final q in const [Offset(-1, -1), Offset(1, -1), Offset(1, 1), Offset(-1, 1)])
        _pos + _rotate(Offset(q.dx * _bodyHalf.width * k, q.dy * _bodyHalf.height * k) * _s, a),
    ];
  }

  Offset _rotate(Offset v, double a) => Offset(v.dx * math.cos(a) - v.dy * math.sin(a), v.dx * math.sin(a) + v.dy * math.cos(a));

  @override
  Widget build(BuildContext context) {
    final m = widget.m, h = _Stage.w * 9 / 16;
    final shown = widget.hand != _Hand.none ? widget.hand : _mode;
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Container(
        width: _Stage.w,
        height: h,
        clipBehavior: Clip.hardEdge,
        decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
        child: GestureDetector(
          onPanStart: !widget.interactive
              ? null
              : (d) {
                  final p = d.localPosition, cs = _corners();
                  if (cs.any((q) => (q - p).distance < 10)) {
                    _mode = _Hand.scale;
                  } else if (cs.any((q) => (q - p).distance < 24)) {
                    _mode = _Hand.rot;
                    _prevA = math.atan2(p.dy - _pos.dy, p.dx - _pos.dx);
                  } else {
                    _mode = _Hand.pos;
                  }
                  m.begin();
                  setState(() {});
                },
          onPanUpdate: !widget.interactive
              ? null
              : (d) {
                  if (_mode == _Hand.pos) {
                    m.add('pos.x', d.delta.dx / _s);
                    m.add('pos.y', d.delta.dy / _s);
                  } else if (_mode == _Hand.scale) {
                    final dir = (_corners().first - _pos);
                    final u = dir / (dir.distance == 0 ? 1 : dir.distance);
                    final k = (d.delta.dx * u.dx + d.delta.dy * u.dy) / (dir.distance == 0 ? 1 : dir.distance) * m.at('scale.x');
                    m.add('scale.x', k);
                    m.add('scale.y', k);
                  } else if (_mode == _Hand.rot) {
                    final a = math.atan2(d.localPosition.dy - _pos.dy, d.localPosition.dx - _pos.dx);
                    var da = a - _prevA;
                    while (da > math.pi) {
                      da -= 2 * math.pi;
                    }
                    while (da < -math.pi) {
                      da += 2 * math.pi;
                    }
                    _prevA = a;
                    m.add('rot', da * 180 / math.pi);
                  }
                },
          onPanEnd: !widget.interactive
              ? null
              : (_) {
                  m.end('Stage: ${_mode == _Hand.pos ? 'move' : _mode == _Hand.scale ? 'scale' : 'rotate'}');
                  setState(() => _mode = _Hand.none);
                },
          onPanCancel: !widget.interactive ? null : () => setState(() => _mode = _Hand.none),
          child: CustomPaint(painter: _StagePaint(_pos, _corners(), m.at('rot'), m.at('scale.x'), shown, m.at('pos.x'), m.at('pos.y'))),
        ),
      ),
      const SizedBox(height: 8),
      SizedBox(width: _Stage.w, child: Text('Stage (mock) · 1920 x 1080 · layer Jewel Field', style: T.label(N.g76))),
    ]);
  }
}

class _StagePaint extends CustomPainter {
  const _StagePaint(this.pos, this.corners, this.rot, this.scale, this.hand, this.px, this.py);
  final Offset pos;
  final List<Offset> corners;
  final double rot, scale, px, py;
  final _Hand hand;
  @override
  void paint(Canvas c, Size z) {
    final hair = Paint()..color = const Color(0x14FFFFFF)..strokeWidth = 1;
    for (var i = 1; i < 3; i++) {
      c.drawLine(Offset(z.width * i / 3, 0), Offset(z.width * i / 3, z.height), hair);
      c.drawLine(Offset(0, z.height * i / 3), Offset(z.width, z.height * i / 3), hair);
    }
    // the jewel: a hexagon in the rotated box
    final body = Path()..moveTo(corners[0].dx, corners[0].dy);
    final mid = (corners[0] + corners[1]) / 2, midB = (corners[3] + corners[2]) / 2;
    body
      ..lineTo(mid.dx, mid.dy)
      ..lineTo(corners[1].dx, corners[1].dy)
      ..lineTo(corners[2].dx, corners[2].dy)
      ..lineTo(midB.dx, midB.dy)
      ..lineTo(corners[3].dx, corners[3].dy)
      ..close();
    c.drawPath(body, Paint()..color = N.g26);
    c.drawPath(body, Paint()..color = N.g63..style = PaintingStyle.stroke);
    final frame = Path()..addPolygon(corners, true);
    c.drawPath(frame, Paint()..color = N.g95..style = PaintingStyle.stroke);
    final big = hand == _Hand.scale ? 7.0 : 5.0;
    for (final q in corners) {
      c.drawRect(Rect.fromCenter(center: q, width: big, height: big), Paint()..color = N.g95);
    }
    c.drawCircle(pos, 2.5, Paint()..color = N.g95);
    final tp = TextPainter(textDirection: TextDirection.ltr, maxLines: 1);
    void tag(String s, Offset at) {
      tp.text = TextSpan(text: s, style: T.value(N.g95));
      tp.layout();
      final r = Rect.fromLTWH((at.dx + 8).clamp(2.0, z.width - tp.width - 10), (at.dy + 8).clamp(2.0, z.height - tp.height - 6), tp.width + 8, tp.height + 4);
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(3)), Paint()..color = N.g13);
      tp.paint(c, r.topLeft + const Offset(4, 2));
    }

    switch (hand) {
      case _Hand.pos:
        final cross = Paint()..color = const Color(0x66FFFFFF)..strokeWidth = 1;
        c.drawLine(Offset(pos.dx, 0), Offset(pos.dx, z.height), cross);
        c.drawLine(Offset(0, pos.dy), Offset(z.width, pos.dy), cross);
        tag('${px.toStringAsFixed(1)}, ${py.toStringAsFixed(1)}', pos);
      case _Hand.rot:
        c.drawCircle(pos, (corners[0] - pos).distance + 14, Paint()..color = const Color(0x66FFFFFF)..style = PaintingStyle.stroke);
        tag('${rot.toStringAsFixed(1)}°', pos);
      case _Hand.scale:
        tag('${scale.toStringAsFixed(1)} %', pos);
      case _Hand.none:
        break;
    }
  }

  @override
  bool shouldRepaint(_StagePaint o) => true;
}

// ---- I3-c: a row that unfolds its instrument ----

class _UnfoldRow extends StatelessWidget {
  const _UnfoldRow({required this.title, required this.row, required this.m, required this.open, required this.onToggle, required this.instrument, this.dock = false, this.hotArrow, this.focusArrow, this.pressArrow});
  final String title;
  final _R row;
  final _M m;
  final bool open, dock;
  final bool? hotArrow, focusArrow, pressArrow; // pinned states of the fold arrow (Parts)
  final VoidCallback onToggle;
  final Widget instrument;

  /// The fold arrow sits in the right column (the left edge stays one line): g63 at rest, g95 on hover, hover plate g15 / focus 1 px g76 edge / pressed g26 plate around it. Open turns it down; the open row is selected (g20 + tick).
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
        _prow(
          row, m,
          dock: dock,
          selected: open,
          trail: _Ctl(
            onTap: onToggle,
            hover: hotArrow,
            focus: focusArrow,
            pressed: pressArrow,
            builder: (_, h, f, p) => Container(
              width: _keyW,
              height: _cellH,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: p ? N.g26 : (h ? N.g15 : null), borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g76 : const Color(0x00000000))),
              child: Transform.rotate(angle: open ? math.pi / 2 : 0, child: Text('›', style: T.name(h || f || p || open ? N.g95 : N.g63))),
            ),
          ),
        ),
        AnimatedSize(
          duration: Mo.dur,
          curve: Mo.ease,
          alignment: Alignment.topCenter,
          child: open ? Padding(padding: const EdgeInsets.only(top: 4, bottom: 8), child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [instrument])) : const SizedBox(width: double.infinity),
        ),
      ]);
}
