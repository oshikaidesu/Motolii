// Part of inspector_idea_layer.dart (cut there by size, as it was).
part of 'inspector_idea_layer.dart';

class _LiMaskStageState extends State<_LiMaskStage> {
  (_LiGrab, int, int)? _hot, _grab;
  Offset _last = Offset.zero;
  Size _size = Size.zero;

  (_LiGrab, int, int)? _hit(Doc doc, Offset q) {
    final ms = _liMasks(doc, _liLayerRect(_size)), sel = _liSel(doc), s = ms[sel];
    for (final (i, p) in s.pts.indexed) {
      if ((p - q).distance <= 6) return (_LiGrab.vertex, sel, i);
    }
    final d = _liSd(s.pts, q), de = (d - s.ex).abs(), dr = (d - s.ex - _liRingGap - s.fe).abs();
    if (math.min(de, dr) <= 4) return (dr < de ? _LiGrab.ring : _LiGrab.edge, sel, 0);
    if (_liSd(s.grown(), q) < 0) return (_LiGrab.body, sel, 0);
    for (var m = 0; m < ms.length; m++) {
      if (m != sel && _liSd(ms[m].grown(), q) < 0) return (_LiGrab.body, m, 0);
    }
    return null;
  }

  void _move(Doc doc, Offset q) {
    final g = _grab;
    if (g == null) return;
    final r = _liLayerRect(_size), k = r.width / _liLayerW, m = g.$2, mk = _liMasks(doc, r)[m];
    switch (g.$1) {
      case _LiGrab.vertex:
        doc.setMany({
          'li_m$m.p${g.$3}.x': _liRound(((q.dx - r.left) / r.width).clamp(-.1, 1.1), 4),
          'li_m$m.p${g.$3}.y': _liRound(((q.dy - r.top) / r.height).clamp(-.1, 1.1), 4),
        });
      case _LiGrab.edge:
        doc.set('li_m$m.ex', (_liSd(mk.pts, q) / k).clamp(-100.0, 100.0).roundToDouble());
      case _LiGrab.ring:
        doc.set('li_m$m.fe', ((_liSd(mk.pts, q) - mk.ex - _liRingGap) / k).clamp(0.0, 200.0).roundToDouble());
      case _LiGrab.body:
        final d = q - _last;
        doc.setMany({
          for (var i = 0; i < mk.pts.length; i++) ...{
            'li_m$m.p$i.x': _liRound((doc.get('li_m$m.p$i.x') ?? 0) + d.dx / r.width, 4),
            'li_m$m.p$i.y': _liRound((doc.get('li_m$m.p$i.y') ?? 0) + d.dy / r.height, 4),
          },
        });
    }
    _last = q;
  }

  void _end(Doc doc) {
    if (_grab == null) return;
    doc.endGesture();
    setState(() => _grab = null);
  }

  String _note(Doc doc) {
    final g = _grab, h = _hot, sel = _liSel(doc);
    if (g != null) {
      return switch (g.$1) {
        _LiGrab.edge => 'Expansion ${_liSigned(doc.get('li_m${g.$2}.ex') ?? 0, 0)} px',
        _LiGrab.ring => 'Feather ${brief(doc.get('li_m${g.$2}.fe'), 0)} px',
        _LiGrab.vertex => 'Point ${g.$3 + 1}',
        _LiGrab.body => 'Move ${_liMaskNames[g.$2]}',
      };
    }
    return switch (h?.$1) {
      _LiGrab.vertex => 'Drag the point',
      _LiGrab.edge => 'Edge: drag in or out to expand',
      _LiGrab.ring => 'Ring: drag out to feather',
      _LiGrab.body => h!.$2 == sel ? 'Drag to move' : 'Click to edit ${_liMaskNames[h.$2]}',
      null => '${_liMaskNames[sel]} · points, edge, ring',
    };
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return LayoutBuilder(
      builder: (context, box) {
        _size = Size(box.maxWidth, _liStageH);
        final r = _liLayerRect(_size), g = _grab ?? _hot;
        return MouseRegion(
          cursor: !on || g == null
              ? SystemMouseCursors.basic
              : switch (g.$1) {
                  _LiGrab.vertex => SystemMouseCursors.precise,
                  _LiGrab.body => g.$2 == _liSel(doc) ? SystemMouseCursors.move : SystemMouseCursors.click,
                  _ => SystemMouseCursors.grab,
                },
          onHover: (e) {
            final h = on ? _hit(doc, e.localPosition) : null;
            if (h != _hot) setState(() => _hot = h);
          },
          onExit: (_) {
            if (_hot != null) setState(() => _hot = null);
          },
          child: Listener(
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final h = _hit(doc, e.localPosition);
              if (h == null) return;
              _last = e.localPosition;
              doc.beginGesture();
              setState(() => _grab = h);
              if ('${h.$2}' != doc.s2['li_m.sel']) doc.str('li_m.sel', '${h.$2}');
            },
            onPointerMove: (e) => _move(doc, e.localPosition),
            onPointerUp: (_) => _end(doc),
            onPointerCancel: (_) {
              if (_grab == null) return;
              doc.cancelGesture();
              setState(() => _grab = null);
            },
            child: CustomPaint(size: _size, painter: _LiMaskPaint(_liMasks(doc, r), _liSel(doc), r, g, _note(doc))),
          ),
        );
      },
    );
  }
}

class _LiMaskPaint extends CustomPainter {
  const _LiMaskPaint(this.ms, this.sel, this.r, this.hot, this.note);
  final List<_LiMask> ms;
  final int sel;
  final Rect r;
  final (_LiGrab, int, int)? hot;
  final String note;

  static const _blend = [BlendMode.srcOver, BlendMode.dstOut, BlendMode.dstIn, BlendMode.xor];
  static const _ops = [PathOperation.union, PathOperation.difference, PathOperation.intersect, PathOperation.xor];

  @override
  void paint(Canvas c, Size s) {
    final box = Offset.zero & s, full = Path()..addRect(box), startFull = ms.first.mode == 1 || ms.first.mode == 2;
    c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(6)), Paint()..color = Pop.well);
    _liChecker(c, r);
    c.saveLayer(r, Paint()..color = const Color(0x2E000000));
    _liArt(c, r);
    c.restore();
    Path shape(_LiMask m) => m.inv ? Path.combine(PathOperation.difference, full, m.path()) : m.path();
    // the feathered picture: each mask's alpha laid down by its mode, then the art kept where the alpha is
    c.saveLayer(box, Paint());
    if (startFull) c.drawRect(box, Paint()..color = Grey.g100);
    for (final m in ms) {
      final p = Paint()
        ..color = Grey.g100.withValues(alpha: m.op)
        ..blendMode = _blend[m.mode];
      if (m.fe > .3) p.maskFilter = MaskFilter.blur(BlurStyle.normal, m.fe / 2);
      c.drawPath(shape(m), p);
    }
    c.saveLayer(box, Paint()..blendMode = BlendMode.srcIn);
    _liArt(c, r);
    c.restore();
    c.restore();
    // the hard cut, by path operations, as a thin line
    var cut = startFull ? (Path()..addRect(r)) : Path();
    for (final m in ms) {
      cut = Path.combine(_ops[m.mode], cut, shape(m));
    }
    c.drawPath(Path.combine(PathOperation.intersect, cut, Path()..addRect(r)), _liLine(Grey.g100.withValues(alpha: .5)));
    for (var m = 0; m < ms.length; m++) {
      final k = ms[m];
      if (m != sel) {
        _liDash(c, k.path(), _liLine(k.tone.withValues(alpha: .8)));
        continue;
      }
      final mine = hot != null && hot!.$2 == m;
      if (k.ex.abs() > .5) c.drawPath(_liPath(k.pts, k.smooth), _liLine(Grey.g56));
      c.drawPath(k.path(), _liLine(k.tone, mine && hot!.$1 == _LiGrab.edge ? 3 : 1.5));
      final ring = mine && hot!.$1 == _LiGrab.ring;
      _liDash(c, k.path(_liRingGap + k.fe), _liLine(k.tone.withValues(alpha: ring ? 1 : .55), ring ? 1.8 : 1));
      for (final (i, p) in k.pts.indexed) {
        final h = mine && hot!.$1 == _LiGrab.vertex && hot!.$3 == i, q = Rect.fromCenter(center: p, width: h ? 8 : 6, height: h ? 8 : 6);
        c.drawRect(q, Paint()..color = h ? k.tone : Grey.g07);
        c.drawRect(q, _liLine(k.tone));
      }
    }
    _liText(c, note, Offset(12, s.height - 15), T.label(Grey.g76));
  }

  @override
  bool shouldRepaint(_LiMaskPaint o) => true;
}

// ---- B. Time strip --------------------------------------------------------------------------------------------------
const _liSrcLen = 2.0, _liWin = 6.0, _liFps = 30.0, _liLoopModes = ['Off', 'Loop', 'Ping-pong'];
const _liBrTop = 14.0, _liBrBot = 28.0, _liBarTop = 31.0, _liBarBot = 51.0, _liGhTop = 55.0, _liGhBot = 67.0, _liStripH = 90.0;

/// contract: comp time t reads source time (t - delay) × speed; Loop folds it into [0, loop), Ping-pong runs every other fold backwards;
/// without a loop the layer ends with its source ([_liSrcLen]). Ghost is the same clock read [ghost] seconds later.
class _LiClock {
  _LiClock(Doc doc)
    : sp = math.max(.05, (doc.get('li_t.speed') ?? 100) / 100),
      loop = doc.get('li_t.loop') ?? 1,
      delay = doc.get('li_t.delay') ?? 0,
      mode = math.max(0, _liLoopModes.indexOf(doc.s2['li_t.mode'] ?? 'Off')),
      ghost = (doc.b['li_t.ghostOn'] ?? false) ? (doc.get('li_t.ghost') ?? 0) / _liFps : null;
  final double sp, loop, delay;
  final int mode;
  final double? ghost;
  double get cycle => mode == 0 || loop <= 1e-6 ? _liSrcLen : loop;
  double get cycleEnd => delay + cycle / sp;
  double? src(double t) {
    final s = (t - delay) * sp;
    if (s < 0) return null;
    if (mode == 0 || loop <= 1e-6) return s <= _liSrcLen ? s : null;
    final r = (s / loop).floor(), a = s - r * loop;
    return mode == 2 && r.isOdd ? loop - a : a;
  }
}

double _liX(Size s, double t) => 8 + (s.width - 16) * t / _liWin;
double _liT(Size s, double x) => (x - 8) / (s.width - 16) * _liWin;

class _LiTimeCard extends StatelessWidget {
  const _LiTimeCard();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      const {'li_t.speed': 100, 'li_t.loop': 1.2, 'li_t.delay': .3, 'li_t.ghost': 6},
      strs: const {'li_t.mode': 'Ping-pong'},
      flags: const {'li_t.ghostOn': true},
    );
    final k = _LiClock(doc);
    return Sect(
      title: 'Time strip',
      tone: Pop.toneTime,
      mark: CardMark.clock,
      keyIds: const ['li_t.speed', 'li_t.loop', 'li_t.delay', 'li_t.ghost'],
      brief: [
        'Speed ${brief(k.sp * 100, 0)}%',
        if (k.mode > 0) '${_liLoopModes[k.mode]} ${brief(k.loop, 2)} s',
        if (k.delay != 0) '${_liSigned(k.delay, 2)} s',
        if (k.ghost != null) 'Ghost ${brief(doc.get('li_t.ghost'), 0)} f',
      ],
      children: [
        const _LiTimeStrip(),
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: SizedBox(
            height: Pop.cell,
            child: Dis(
              child: Segmented(items: _liLoopModes, index: k.mode, expand: true, onChanged: (i) => doc.str('li_t.mode', _liLoopModes[i])),
            ),
          ),
        ),
        _liRow([
          ldNum(doc, 'li_t.speed', 'Speed', unit: '%', min: 5, max: 800),
          ldNum(doc, 'li_t.loop', 'Loop', unit: 's', dec: 2, min: .1, max: _liSrcLen, perPx: .01, step: .1, enabled: k.mode > 0),
          ldNum(doc, 'li_t.delay', 'Delay', unit: 's', dec: 2, min: -2, max: 4, perPx: .01, step: .1, bipolar: true),
          ldNum(doc, 'li_t.ghost', 'Ghost', unit: 'f', min: -60, max: 60, perPx: .25, bipolar: true, enabled: k.ghost != null),
        ]),
      ],
    );
  }
}

enum _LiTGrab { speed, loop, bar, ghost }

/// contract: the bracket row sets the loop length (a click flips Loop / Ping-pong), the bar's end grip sets speed, the bar body the delay,
/// the faint row the ghost (a click turns it on or off). Hover only reads; one undo per drag.
class _LiTimeStrip extends StatefulWidget {
  const _LiTimeStrip();
  @override
  State<_LiTimeStrip> createState() => _LiTimeStripState();
}

class _LiTimeStripState extends State<_LiTimeStrip> {
  double? _hx;
  _LiTGrab? _grab, _hot;
  Offset _down = Offset.zero;
  double _start = 0, _acc = 0;
  bool _moved = false;
  Size _size = Size.zero;

  _LiTGrab? _hit(_LiClock k, Offset q) {
    final y = q.dy;
    if (y >= _liBrTop - 3 && y < _liBarTop - 1) return _LiTGrab.loop;
    if (y >= _liBarTop - 1 && y <= _liBarBot + 1) {
      if ((q.dx - _liX(_size, k.cycleEnd)).abs() <= 6) return _LiTGrab.speed;
      if (q.dx >= _liX(_size, k.delay) - 2 && (k.mode > 0 || q.dx <= _liX(_size, k.cycleEnd))) return _LiTGrab.bar;
      return null;
    }
    if (y > _liBarBot + 1 && y <= _liGhBot + 3) return _LiTGrab.ghost;
    return null;
  }

  void _move(Doc doc, Offset q, double dx) {
    final g = _grab;
    if (g == null) return;
    if (!_moved && (q - _down).distance < 3) return;
    _moved = true;
    _acc += dx;
    final k = _LiClock(doc), t = _liT(_size, q.dx), perPx = _liWin / (_size.width - 16);
    switch (g) {
      case _LiTGrab.loop:
        if (k.mode == 0) doc.s2['li_t.mode'] = 'Loop';
        doc.set('li_t.loop', _liRound(((t - k.delay) * k.sp).clamp(.1, _liSrcLen), 2));
      case _LiTGrab.speed:
        doc.set('li_t.speed', (k.cycle / math.max(.05, t - k.delay) * 100).clamp(5.0, 800.0).roundToDouble());
      case _LiTGrab.bar:
        doc.set('li_t.delay', _liRound((_start + _acc * perPx).clamp(-2.0, 4.0), 2));
      case _LiTGrab.ghost:
        doc.b['li_t.ghostOn'] = true;
        doc.set('li_t.ghost', (_start + _acc * perPx * _liFps).clamp(-60.0, 60.0).roundToDouble());
    }
  }

  void _up(Doc doc) {
    final g = _grab;
    if (g == null) return;
    if (!_moved) {
      final k = _LiClock(doc);
      if (g == _LiTGrab.loop) doc.str('li_t.mode', k.mode == 2 ? 'Loop' : 'Ping-pong');
      if (g == _LiTGrab.ghost) doc.flag('li_t.ghostOn', k.ghost == null);
    }
    doc.endGesture();
    setState(() => _grab = null);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked, k = _LiClock(doc);
    return LayoutBuilder(
      builder: (context, box) {
        _size = Size(box.maxWidth, _liStripH);
        final g = _grab ?? _hot;
        return MouseRegion(
          cursor: !on || g == null
              ? SystemMouseCursors.basic
              : g == _LiTGrab.bar
              ? SystemMouseCursors.grab
              : SystemMouseCursors.resizeLeftRight,
          onHover: (e) => setState(() {
            _hx = e.localPosition.dx;
            _hot = on ? _hit(k, e.localPosition) : null;
          }),
          onExit: (_) => setState(() {
            _hx = null;
            _hot = null;
          }),
          child: Listener(
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final h = _hit(k, e.localPosition);
              if (h == null) return;
              _down = e.localPosition;
              _moved = false;
              _acc = 0;
              _start = h == _LiTGrab.ghost ? (doc.get('li_t.ghost') ?? 0) : k.delay;
              doc.beginGesture();
              setState(() => _grab = h);
            },
            onPointerMove: (e) {
              _hx = e.localPosition.dx;
              _move(doc, e.localPosition, e.delta.dx);
            },
            onPointerUp: (_) => _up(doc),
            onPointerCancel: (_) {
              if (_grab == null) return;
              doc.cancelGesture();
              setState(() => _grab = null);
            },
            child: CustomPaint(size: _size, painter: _LiTimePaint(k, _hx, g)),
          ),
        );
      },
    );
  }
}
