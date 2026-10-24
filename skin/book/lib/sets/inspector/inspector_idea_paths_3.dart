// Part of inspector_idea_paths.dart (cut there by size, as it was).
part of 'inspector_idea_paths.dart';

class _PiThumbPaint extends CustomPainter {
  const _PiThumbPaint(this.cs, this.ink);
  final List<_PiC> cs;
  final Color ink;
  @override
  void paint(Canvas c, Size s) => c.drawPath(
    _piPath(cs, _PiView(s, _piBoundsB.inflate(6))),
    Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.1
      ..strokeJoin = StrokeJoin.round,
  );
  @override
  bool shouldRepaint(_PiThumbPaint o) => !identical(o.cs, cs) || o.ink != ink;
}

/// A handle on the joint before segment [i] that slides along it: the corner's radius.
_PiH? _piCornerH(String id, _PiC c, int i, double r, void Function(double) set, String Function() tip) {
  final n = c.segs.length;
  if (n <= i || (!c.closed && i == 0)) return null;
  final s = c.segs[i], lim = math.min(_piLen(s), _piLen(c.segs[(i - 1 + n) % n])) / 2;
  return _PiH(id, _piAt(s, r.clamp(0.0, lim)).$1, (q) => set((_piNear(s, q).clamp(0.0, lim) * 2).roundToDouble() / 2), from: s.first, tip: tip);
}

// ---- A: path stage --------------------------------------------------------------------------------------------------
const _piShapes = ['Star', 'Rect', 'Line'];
const _piBoundsA = Rect.fromLTRB(-125, -132, 125, 90);

/// One card, one stage: the result of the whole stack; the picked instance's handles sit on the path; the list below names, briefs,
/// switches and orders instances (any kind as often as wanted).
class _PiA extends StatelessWidget {
  const _PiA();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      {'pi_a_round1_radius': 6, 'pi_a_round3_radius': 5},
      strs: const {'pi_a_order': 'round1,pucker2,round3,wiggle4,trim5', 'pi_a_next': '6', 'pi_a_sel': 'trim5', 'pi_a_shape': 'Star'},
      flags: const {'pi_a_wiggle4.on': false},
    );
    final order = _piOrder(doc, 'a'), shape = doc.s2['pi_a_shape']!, open = doc.b['pi_a.open'] ?? true;
    final picked = doc.s2['pi_a_sel'], sel = order.contains(picked) ? picked : order.lastOrNull;
    bool on(String s) => doc.b['pi_a_$s.on'] ?? true;
    final base = [_piShapeA(shape)];
    final (ins, out) = _piRun(doc, 'a', base, order);
    final i = sel == null ? -1 : order.indexOf(sel), selIn = i < 0 ? null : ins[i], selOn = sel != null && on(sel);
    final start = sel != null && _piKind(sel) == 'trim' && selIn!.isNotEmpty && selIn.first.flat.isNotEmpty ? selIn.first.flat.first : null;
    return Sect(
      title: 'Path operations',
      tone: Pop.tonePath,
      mark: CardMark.path,
      open: open,
      onToggle: () => doc.flag('pi_a.open', !open),
      brief: [
        for (final s in order)
          if (on(s)) _piKinds[_piKind(s)]!.short,
      ],
      children: [
        _PiStage(
          key: const ValueKey('pi_a_stage'),
          height: 164,
          bounds: _piBoundsA,
          inks: [_PiInk(base, Grey.g26, 1), if (selOn && i > 0) _PiInk(selIn!, Grey.g38, 1.2), _PiInk(out, Pop.tonePath, 2.4)],
          marks: [if (start != null) (start, Grey.g63, 2.2)],
          handles: selOn ? _piHandlesA(doc, sel, selIn!) : const [],
          overlay: Dis(
            child: Segmented(items: _piShapes, index: _piShapes.indexOf(shape), onChanged: (i) => doc.str('pi_a_shape', _piShapes[i])),
          ),
          note: sel == null ? 'No path operations' : (selOn ? 'Drag on the path: ${_piTitle(order, sel)}' : '${_piTitle(order, sel)} is off'),
        ),
        const SizedBox(height: 4),
        if (order.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
            child: Text('No path operations. The path is drawn as built.', style: T.label(Grey.g56)),
          ),
        for (final s in order) ...[
          _PiARow(s, order, sel: s == sel, key: ValueKey('pi_a_row_$s')),
          if (s == sel) Padding(padding: const EdgeInsets.fromLTRB(0, 4, 0, 6), child: _PiFields('a', s)),
        ],
        const _PiAdd('a'),
      ],
    );
  }
}

List<_PiH> _piHandlesA(Doc doc, String inst, List<_PiC> inp) {
  if (inp.isEmpty || inp.first.segs.isEmpty) return const [];
  final c = inp.first, k = 'pi_a_$inst', g = _piG(doc, 'a', inst), op = _piKind(inst);
  void set(String p, double v) => doc.set('${k}_$p', v);
  switch (op) {
    case 'pucker':
      final ctr = _piCentre(c), s0 = c.segs.first, m = (s0.first + s0.last) / 2, r = (m - ctr).distance, u = _piUnit(m - ctr);
      if (r < 1) return const [];
      return [
        _PiH(
          '${k}_amount',
          m + u * (r * g('amount') / 200),
          (q) => set('amount', (_piDot(q - m, u) * 200 / r).clamp(-100.0, 100.0).roundToDouble()),
          from: m,
          tip: () => _piKinds[op]!.sum(g).first,
        ),
      ];
    case 'round':
      final h = _piCornerH('${k}_radius', c, c.closed ? 0 : 1, g('radius'), (v) => set('radius', v), () => 'Radius ${px(g('radius'))}');
      return [?h];
    case 'wiggle':
      final p = c.flat;
      final (q0, t) = _piAt(p, _piLen(p) * .3);
      final n = _piN(t) * _piSign(c);
      return [
        _PiH(
          '${k}_size',
          q0 + n * g('size'),
          (q) => set('size', (math.max(0.0, _piDot(q - q0, n)) * 2).roundToDouble() / 2),
          from: q0,
          tip: () => 'Size ${px(g('size'))}',
        ),
      ];
    case 'trim':
      final p = c.flat, l = _piLen(p), o = g('offset') / 360, s = g('start'), e = g('end');
      if (l < 1e-6) return const [];
      double w(double f) {
        final r = f - f.floorToDouble();
        return r == 0 && f > 0 ? 1 : r;
      }

      Offset at(double f) => _piAt(p, w(f) * l).$1;
      double frac(Offset q) => _piNear(p, q) / l;
      double pick(double f, double old) {
        var v = ((f - o) % 1) * 100;
        if ((v - old).abs() > 50) v = old > 50 ? 100 : 0;
        return v.roundToDouble();
      }

      final mid = (s + e) / 200 + o;
      return [
        _PiH('${k}_end', at(e / 100 + o), (q) => set('end', pick(frac(q), e)), tip: () => 'End ${brief(g('end'), 0)}%'),
        _PiH('${k}_start', at(s / 100 + o), (q) => set('start', pick(frac(q), s)), shape: 2, tip: () => 'Start ${brief(g('start'), 0)}%'),
        _PiH(
          '${k}_offset',
          at(mid),
          (q) {
            var d = frac(q) - w(mid) % 1;
            d -= d.roundToDouble();
            set('offset', g('offset') + d * 360);
          },
          shape: 1,
          color: Pop.tonePath,
          tip: () => 'Offset ${brief(g('offset'), 0)}°',
        ),
      ];
  }
  return const [];
}

class _PiARow extends StatelessWidget {
  const _PiARow(this.inst, this.order, {super.key, required this.sel});
  final String inst;
  final List<String> order;
  final bool sel;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, on = doc.b['pi_a_$inst.on'] ?? true, op = _piKind(inst), kd = _piKinds[op]!;
    return Hov(
      onTap: () => doc.str('pi_a_sel', inst),
      builder: (_, h) => Container(
        height: Pop.row,
        padding: const EdgeInsets.only(left: 6),
        decoration: BoxDecoration(color: sel ? Grey.g20 : (h ? Grey.g15 : null), borderRadius: BorderRadius.circular(6)),
        child: Row(
          children: [
            OpGlyph(op, hot: sel),
            const SizedBox(width: 8),
            SizedBox(
              width: 104,
              child: Text(
                _piTitle(order, inst),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.name(!on ? Grey.g56 : (sel || h ? Grey.g100 : Grey.g76)).copyWith(decoration: on ? null : TextDecoration.lineThrough),
              ),
            ),
            Expanded(child: _PiChips(kd.sum(_piG(doc, 'a', inst)), dim: !on)),
            OnOff(on: on, onChanged: (v) => doc.flag('pi_a_$inst.on', v)),
            _PiMore('a', inst),
          ],
        ),
      ),
    );
  }
}

// ---- B: before → after strip ----------------------------------------------------------------------------------------
const _piBoundsB = Rect.fromLTRB(-118, -78, 118, 78);

/// Up to this many instances the strip's frames share its width; past it they keep a fixed width and the strip scrolls sideways.
const _piStripFit = 5;

/// The stack as a film strip (one frame per instance: the shape after it) over cards whose heads carry an in → out pair;
/// open, a card is a wipe-compare stage.
class _PiB extends StatelessWidget {
  const _PiB();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed({}, strs: const {'pi_b_order': 'zigzag1,round2,offset3,twist4', 'pi_b_next': '5'}, flags: const {'pi_b_zigzag1.open': true});
    final order = _piOrder(doc, 'b'), base = [_piRect], fit = order.length <= _piStripFit;
    final (ins, out) = _piRun(doc, 'b', base, order);
    List<_PiC> after(int i) => i + 1 < ins.length ? ins[i + 1] : out;
    final frames = Row(
      mainAxisSize: fit ? MainAxisSize.max : MainAxisSize.min,
      children: [
        _PiFrame('Path', base, null, width: fit ? null : 48),
        for (final (i, s) in order.indexed) ...[
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Text(' › ', style: T.name(Grey.g44)),
          ),
          _PiFrame(_piTitle(order, s, short: true), after(i), s, width: fit ? null : 48),
        ],
      ],
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Container(
          color: Pop.card,
          margin: EdgeInsets.only(bottom: Pop.gap),
          padding: EdgeInsets.symmetric(horizontal: Pop.inset, vertical: Pop.insetY),
          child: fit ? frames : SingleChildScrollView(scrollDirection: Axis.horizontal, child: frames),
        ),
        for (final (i, s) in order.indexed) _PiBCard(s, _piTitle(order, s), ins[i], after(i), key: ValueKey('pi_b_$s')),
        const _PiAdd('b', card: true),
      ],
    );
  }
}

class _PiFrame extends StatelessWidget {
  const _PiFrame(this.label, this.cs, this.op, {this.width});
  final String label;
  final List<_PiC> cs;
  final String? op;
  final double? width;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, o = op, on = o == null || (doc.b['pi_b_$o.on'] ?? true), open = o != null && (doc.b['pi_b_$o.open'] ?? false);
    final w = width;
    Widget box(Widget child) => w == null ? Expanded(child: child) : SizedBox(width: w, child: child);
    return box(
      Hov(
        onTap: o == null ? null : () => doc.flag('pi_b_$o.open', !open),
        cursor: o == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
        builder: (_, h) => Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              height: 40,
              decoration: BoxDecoration(
                color: h ? Grey.g15 : Grey.g10,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: open ? Pop.tonePath.withValues(alpha: .6) : const Color(0x00000000)),
              ),
              child: _PiThumb(cs, o == null ? Grey.g63 : (on ? Pop.tonePath : Grey.g44)),
            ),
            const SizedBox(height: 3),
            Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: T.micro(!on ? Grey.g44 : (open || h ? Grey.g95 : Grey.g63)).copyWith(decoration: on ? null : TextDecoration.lineThrough),
            ),
          ],
        ),
      ),
    );
  }
}

class _PiBCard extends StatefulWidget {
  const _PiBCard(this.inst, this.title, this.inp, this.out, {super.key});
  final String inst, title;
  final List<_PiC> inp, out;
  @override
  State<_PiBCard> createState() => _PiBCardState();
}

class _PiBCardState extends State<_PiBCard> {
  double _wipe = -30;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, inst = widget.inst, op = _piKind(inst), k = 'pi_b_$inst', kd = _piKinds[op]!, g = _piG(doc, 'b', inst);
    final on = doc.b['$k.on'] ?? true, open = doc.b['$k.open'] ?? false, b = _piBoundsB;
    return Sect(
      title: widget.title,
      tone: Pop.tonePath,
      mark: CardMark.path,
      dim: !on,
      open: open,
      onToggle: () => doc.flag('$k.open', !open),
      brief: kd.sum(g),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _PiThumb(widget.inp, Grey.g56, w: 24, h: 16),
          Text(' › ', style: T.label(Grey.g44)),
          _PiThumb(widget.out, on ? Pop.tonePath : Grey.g44, w: 24, h: 16),
          const SizedBox(width: 10),
          OnOff(on: on, onChanged: (v) => doc.flag('$k.on', v)),
          _PiMore('b', inst),
        ],
      ),
      children: [
        _PiStage(
          key: ValueKey('${k}_stage'),
          height: 128,
          bounds: b,
          wipe: _wipe,
          rings: [if (op == 'twist' && on) (Offset(g('cx'), g('cy')), 46.0)],
          inks: [_PiInk(widget.inp, Grey.g20, 1), _PiInk(widget.inp, Grey.g76, 1.4, side: -1), _PiInk(widget.out, Pop.tonePath, 2.2, side: 1)],
          handles: [
            if (on) ..._piHandlesB(doc, inst, widget.inp),
            _PiH(
              '${k}_wipe',
              Offset(_wipe, b.top + 9),
              (q) => setState(() => _wipe = q.dx.clamp(b.left + 6, b.right - 6)),
              shape: 3,
              color: Grey.g63,
              tip: () => 'Drag to compare before and after',
            ),
          ],
          note: on ? null : 'Off: its input passes through',
        ),
        const SizedBox(height: 6),
        _PiFields('b', inst),
      ],
    );
  }
}

List<_PiH> _piHandlesB(Doc doc, String inst, List<_PiC> inp) {
  if (inp.isEmpty || inp.first.segs.isEmpty) return const [];
  final c = inp.first, k = 'pi_b_$inst', g = _piG(doc, 'b', inst), sg = _piSign(c), op = _piKind(inst);
  void set(String p, double v) => doc.set('${k}_$p', v);
  switch (op) {
    case 'round':
      var at = 0, best = double.infinity;
      for (final (i, s) in c.segs.indexed) {
        final d = (s.first - const Offset(80, -50)).distance;
        if (d < best) (at, best) = (i, d);
      }
      return [?_piCornerH('${k}_radius', c, at, g('radius'), (v) => set('radius', v), () => 'Radius ${px(g('radius'))}')];
    case 'zigzag':
      final s = c.segs.first, l = _piLen(s), n = g('ridges').round().clamp(1, 60);
      if (l < 4) return const [];
      final (b0, t) = _piAt(s, l / (n + 1));
      return [
        _PiH(
          '${k}_peak',
          b0 + _piN(t) * (sg * g('amp')),
          (q) {
            final d = _piNear(s, q).clamp(l / 61, l / 2);
            final (p, tt) = _piAt(s, d);
            doc.setMany({'${k}_ridges': (l / d - 1).roundToDouble().clamp(1, 60), '${k}_amp': (_piDot(q - p, _piN(tt) * sg) * 2).roundToDouble() / 2});
          },
          from: b0,
          tip: () => 'Size ${px(g('amp'))} · ${brief(g('ridges'), 0)} ridges (drag along to space them)',
        ),
      ];
    case 'offset':
      final p = c.flat, d = _piNear(p, const Offset(80, 0)), q0 = _piAt(p, d).$1;
      final n = _piN(_piUnit(_piAt(p, d + 14).$1 - _piAt(p, d - 14).$1)) * sg;
      return [
        _PiH(
          '${k}_amount',
          q0 + n * g('amount'),
          (q) => set('amount', (_piDot(q - q0, n) * 2).roundToDouble() / 2),
          from: q0,
          tip: () => 'Amount ${sgPaths(g('amount'), 1)} px',
        ),
      ];
    case 'twist':
      final ctr = Offset(g('cx'), g('cy')), ang = g('angle');
      return [
        _PiH(
          '${k}_angle',
          ctr + Offset.fromDirection((ang - 90) * math.pi / 180, 46),
          (q) {
            var d = math.atan2(q.dy - ctr.dy, q.dx - ctr.dx) * 180 / math.pi + 90 - ang;
            d -= (d / 360).roundToDouble() * 360;
            set('angle', (ang + d).roundToDouble());
          },
          from: ctr,
          tip: () => 'Angle ${sgPaths(g('angle'))}° (keep turning past a full turn)',
        ),
        _PiH(
          '${k}_centre',
          ctr,
          (q) => doc.setMany({'${k}_cx': q.dx.roundToDouble(), '${k}_cy': q.dy.roundToDouble()}),
          shape: 2,
          tip: () => 'Centre ${brief(g('cx'), 0)}, ${brief(g('cy'), 0)}',
        ),
      ];
  }
  return const [];
}

// ---- C: draw-on timeline --------------------------------------------------------------------------------------------
const _piCKeys = ['s0', 's1', 'e0', 'e1'], _piEases = ['Linear', 'Ease', 'Ease out'];
const _piBoundsC = Rect.fromLTRB(-128, -60, 128, 60);
const _piPresets = <String, Map<String, double>>{
  'Draw on': {'s0t': 0, 's0v': 0, 's1t': 2, 's1v': 0, 'e0t': 0, 'e0v': 0, 'e1t': 1.2, 'e1v': 100},
  'Draw off': {'s0t': .4, 's0v': 0, 's1t': 1.6, 's1v': 100, 'e0t': 0, 'e0v': 100, 'e1t': 2, 'e1v': 100},
  'Write & erase': {'s0t': .7, 's0v': 0, 's1t': 1.7, 's1v': 100, 'e0t': 0, 'e0v': 0, 'e1t': 1, 'e1v': 100},
  'From middle': {'s0t': 0, 's0v': 50, 's1t': 1.2, 's1v': 0, 'e0t': 0, 'e0v': 50, 'e1t': 1.2, 'e1v': 100},
};

double _piEase(int k, double u) => switch (k) {
  1 => u * u * (3 - 2 * u),
  2 => 1 - math.pow(1 - u, 3).toDouble(),
  _ => u,
};

/// The ease's curve, small: what the Ease link says is set.
class _PiEaseGlyph extends CustomPainter {
  const _PiEaseGlyph(this.ease, this.ink);
  final int ease;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final r = Rect.fromLTWH(1, 1, s.width - 2, s.height - 2), p = Path()..moveTo(r.left, r.bottom);
    for (var i = 1; i <= 16; i++) {
      p.lineTo(r.left + r.width * i / 16, r.bottom - r.height * _piEase(ease, i / 16));
    }
    c.drawPath(
      p,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.3
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(_PiEaseGlyph o) => o.ease != ease || o.ink != ink;
}

/// A Trim lane's value at [t] between its two keys.
double _piLane(Map<String, double> v, String lane, double t, int ease) {
  var (t0, v0, t1, v1) = (v['${lane}0t']!, v['${lane}0v']!, v['${lane}1t']!, v['${lane}1v']!);
  if (t1 < t0) (t0, v0, t1, v1) = (t1, v1, t0, v0);
  if (t <= t0) return v0;
  if (t >= t1) return v1;
  return v0 + (v1 - v0) * _piEase(ease, (t - t0) / (t1 - t0));
}

Map<String, double> _piCVals(Doc d) => {
  for (final k in _piCKeys)
    for (final p in const ['t', 'v']) '$k$p': d.get('pi_c_$k$p') ?? 0,
};

/// Trim Paths as an animation designed in place: the stage shows the stroke at the hovered (or current) time, with the heads' frame-by-frame spacing.
class _PiCTrim extends StatefulWidget {
  const _PiCTrim();
  @override
  State<_PiCTrim> createState() => _PiCTrimState();
}
