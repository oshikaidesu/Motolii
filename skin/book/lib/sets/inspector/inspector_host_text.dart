part of 'inspector_parts.dart';

// ---- host: text ---------------------------------------------------------------------------------------------------------
/// contract: names follow motolii-doc store/text.rs (TextBasedOn, TextRangeUnits, TextShape, TextGrouping, TextJustify) and vector/text.rs (the CSS trio).
const _taBased = ['Chars', 'No spaces', 'Words', 'Lines'];
const _taBasedLong = ['Characters', 'Chars excl. spaces', 'Words', 'Lines'];
const _taShapes = ['Square', 'Ramp up', 'Ramp down', 'Triangle', 'Round', 'Smooth'];
const _taGroups = ['Chars', 'Word', 'Line', 'All'];
const _taSample = 'Motolii in motion';

const _taRange = [
  Param(id: 'start', label: 'Start', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%'),
  Param(id: 'end', label: 'End', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 40, unit: '%'),
  Param(id: 'offset', label: 'Offset', look: ParamLook.bipolar, type: WType.f32, min: -100, max: 100, unit: '%'),
];
const _taBasedP = Param(id: 'based', label: 'Based on', look: ParamLook.choice, type: WType.u32, min: 0, max: 3, options: _taBased);
const _taProps = [
  Param(id: 'pos', label: 'Position', look: ParamLook.point, type: WType.vec2, min: -200, max: 200, unit: 'px'),
  Param(id: 'opacity', label: 'Opacity', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 100, unit: '%'),
  Param(id: 'rot', label: 'Rotation', look: ParamLook.angle, type: WType.f32, min: -360, max: 360, unit: '°'),
  Param(id: 'scale', label: 'Scale', look: ParamLook.size, type: WType.vec2, min: 0, max: 400, def: 100, defY: 100, unit: '%'),
  Param(id: 'track', label: 'Tracking', look: ParamLook.bipolar, type: WType.f32, min: -200, max: 200),
  Param(id: 'fill', label: 'Fill', look: ParamLook.colour, type: WType.vec4),
  Param(id: 'wght', label: 'Weight (wght)', look: ParamLook.bipolar, type: WType.f32, min: -400, max: 400),
  Param(id: 'line', label: 'Line spacing', look: ParamLook.bipolar, type: WType.f32, min: -100, max: 100, unit: 'px'),
];
const _taAdv = [
  Param(id: 'units', label: 'Units', look: ParamLook.choice, type: WType.u32, min: 0, max: 1, options: ['Percent', 'Index']),
  Param(id: 'amount', label: 'Max amount', look: ParamLook.bipolar, type: WType.f32, min: -100, max: 100, def: 100, unit: '%'),
  Param(id: 'easeHi', label: 'Ease high', look: ParamLook.bipolar, type: WType.f32, min: -100, max: 100, unit: '%'),
  Param(id: 'easeLo', label: 'Ease low', look: ParamLook.bipolar, type: WType.f32, min: -100, max: 100, unit: '%'),
  Param(id: 'rand', label: 'Randomize', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1),
];
const _taSeedP = Param(id: 'seed', label: 'Seed', look: ParamLook.seed, type: WType.u32, min: 0, max: 9999);
const _paraGroupP = Param(id: 'grouping', label: 'Anchor grouping', look: ParamLook.choice, type: WType.u32, min: 0, max: 3, options: _taGroups);
const _taKeys = [
  'ta_start',
  'ta_end',
  'ta_offset',
  'ta_starti',
  'ta_endi',
  'ta_offseti',
  'ta_amount',
  'ta_pos',
  'ta_opacity',
  'ta_rot',
  'ta_scale',
  'ta_track',
  'ta_wght',
  'ta_line',
];

/// The selector counts these pieces of the sample, as [start, end) character spans in reading order.
List<(int, int)> _taUnits(String s, int based) => switch (based) {
  0 => [for (var i = 0; i < s.length; i++) (i, i + 1)],
  1 => [
    for (var i = 0; i < s.length; i++)
      if (s[i] != ' ') (i, i + 1),
  ],
  2 => [for (final m in RegExp(r'\S+').allMatches(s)) (m.start, m.end)],
  _ => [(0, s.length)],
};

/// How much one piece spanning [a, b] of the text is selected by [s, e] (0–1 of the whole), before Max amount.
/// contract: Square is the piece's coverage (AE: a half-covered letter is half selected); the other shapes sample the piece's centre.
double _taAmount(int shape, double s, double e, double a, double b, double hi, double lo) {
  if (e < s) (s, e) = (e, s);
  if (shape == 0) return b <= a ? (a >= s && a <= e ? 1 : 0) : ((math.min(b, e) - math.max(a, s)) / (b - a)).clamp(0.0, 1.0);
  final w = e - s, c = (a + b) / 2;
  final u = w <= 0 ? (c < s ? -1.0 : 2.0) : (c - s) / w;
  final inside = u >= 0 && u <= 1;
  final v = switch (shape) {
    1 => u.clamp(0.0, 1.0),
    2 => 1 - u.clamp(0.0, 1.0),
    3 => inside ? 1 - (2 * u - 1).abs() : 0.0,
    4 => inside ? math.sqrt(math.max(0, 1 - math.pow(2 * u - 1, 2))) : 0.0,
    _ => inside ? (1 - math.cos(2 * math.pi * u)) / 2 : 0.0,
  };
  double ease(double x, double by, bool top) {
    if (by == 0) return x;
    final p = 1 + by.abs() / 50, q = by > 0 ? p : 1 / p;
    return top ? 1 - math.pow(1 - x, q).toDouble() : math.pow(x, q).toDouble();
  }

  return ease(ease(v, lo, false), hi, true);
}

String _taWord(Doc d) {
  final t = (d.s2['text.content'] ?? '').split('\n').first.trim();
  return t.isEmpty ? _taSample : (t.length > 22 ? t.substring(0, 22) : t);
}

typedef _TaSel = ({List<double> amt, double s, double e, int n, bool index});

_TaSel _taSelect(Doc d, String word) {
  double g(String id, [double z = 0]) => d.get(id) ?? z;
  final index = g('ta_units') > .5, units = _taUnits(word, g('ta_based').round()), n = math.max(1, units.length);
  final span = index ? n.toDouble() : 100.0, sfx = index ? 'i' : '', off = g('ta_offset$sfx');
  final s = (g('ta_start$sfx') + off) / span, e = (g('ta_end$sfx', index ? 3 : 40) + off) / span;
  final shape = math.max(0, _taShapes.indexOf(d.s2['ta_shape'] ?? 'Square'));
  final order = List.generate(units.length, (i) => i);
  if (g('ta_rand') > .5) order.shuffle(math.Random(g('ta_seed').round()));
  final max = g('ta_amount', 100) / 100, amt = List.filled(word.length, 0.0);
  for (var k = 0; k < units.length; k++) {
    final p = order[k], a = _taAmount(shape, s, e, p / n, (p + 1) / n, g('ta_easeHi'), g('ta_easeLo')) * max;
    for (var i = units[k].$1; i < units[k].$2; i++) {
      amt[i] = a;
    }
  }
  return (amt: amt, s: s, e: e, n: n, index: index);
}

Map<String, double> _taDefaults(Iterable<Param> ps, String fx) => {for (final p in ps) ...paramDefaults(fx, p)};

/// Blocks laid like [ParamGrid] (2–5 across by width, a wide block takes two), but any widget may be a block.
class _TaGrid extends StatelessWidget {
  const _TaGrid(this.items, {this.on = true});
  final List<(int, Widget)> items;
  final bool on;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      const g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <List<(int, Widget)>>[[]];
      var used = 0;
      for (final (span, w) in items) {
        final s = math.min(span, cols);
        if (used + s > cols && rows.last.isNotEmpty) {
          rows.add([]);
          used = 0;
        }
        rows.last.add((s, w));
        used += s;
      }
      Widget row(List<(int, Widget)> r, bool last) {
        final filled = r.fold<int>(0, (a, it) => a + it.$1);
        return Row(
          children: [
            for (final (i, (s, w)) in r.indexed) ...[if (i > 0) const SizedBox(width: g), Expanded(flex: s, child: w)],
            if (last && filled < cols) ...[const SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: const EdgeInsets.only(top: 4),
        child: RowOff(
          off: !on,
          narrow: false,
          child: Column(
            children: [
              for (var i = 0; i < rows.length; i++) ...[if (i > 0) const SizedBox(height: g), row(rows[i], i == rows.length - 1)],
            ],
          ),
        ),
      );
    },
  );
}

(int, Widget) _taTile(String fx, Param p) => (ParamGrid.span(p), ParamTile(fx, p));

/// A block that is not a number: the name top-left, the control along the bottom, an optional picture top-right.
class _TaBlock extends StatelessWidget {
  const _TaBlock({required this.label, required this.child, this.glyph});
  final String label;
  final Widget child;
  final Widget? glyph;
  @override
  Widget build(BuildContext context) {
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return Container(
      height: Pop.tile,
      decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
      child: Stack(
        children: [
          // the picture stays in the name's line so the control below keeps the block's full width
          if (glyph != null)
            Positioned(
              right: Pop.tilePad - 3,
              top: 2,
              width: Pop.glyph,
              height: 16,
              child: Center(child: glyph),
            ),
          Padding(
            padding: const EdgeInsets.fromLTRB(Pop.tilePad, 6, Pop.tilePad, 4),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Padding(
                  padding: EdgeInsets.only(right: glyph != null ? Pop.glyph : 0),
                  child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                ),
                SizedBox(height: 22, child: child),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// The fold under a card's main blocks (as in an effect card): rarely touched values wait behind it.
class _TaFold extends StatelessWidget {
  const _TaFold(this.flag, this.word, this.count);
  final String flag, word;
  final int count;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed), open = x.doc.b[flag] ?? false;
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: Hov(
        onTap: () => x.doc.flag(flag, !open),
        builder: (_, h) => Container(
          height: 24,
          decoration: BoxDecoration(
            border: Border(top: BorderSide(color: ed.rule)),
          ),
          child: Row(
            children: [
              Text(word, style: ed.labStyle(h || open ? Grey.g95 : Grey.g63)),
              const Spacer(),
              Text(open ? 'Fold' : '$count more', style: T.label(Grey.g56)),
            ],
          ),
        ),
      ),
    );
  }
}

class _TaCap extends StatelessWidget {
  const _TaCap(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 8),
    child: Text(text.toUpperCase(), style: T.micro(Grey.g56).copyWith(letterSpacing: .4)),
  );
}

class TextAnimatorCard extends StatelessWidget {
  const TextAnimatorCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, word = _taWord(doc);
    final sel0 = _taUnits(word, (doc.get('ta_based') ?? 0).round()).length;
    doc.seed(
      {
        ..._taDefaults([..._taRange, _taBasedP, ..._taProps, ..._taAdv, _taSeedP], 'ta'),
        ...paramDefaults('para', _paraGroupP),
        'ta_starti': 0,
        'ta_endi': 3,
        'ta_offseti': 0,
      },
      strs: const {'ta_shape': 'Ramp up', 'ta_fill': '#E8C76A'},
    );
    final on = doc.b['ta.on'] ?? true, index = (doc.get('ta_units') ?? 0) > .5, rand = (doc.get('ta_rand') ?? 0) > .5;
    final n = math.max(1, sel0).toDouble();
    final range = index
        ? [
            Param(id: 'starti', label: 'Start', look: ParamLook.bar, type: WType.i32, min: 0, max: n),
            Param(id: 'endi', label: 'End', look: ParamLook.bar, type: WType.i32, min: 0, max: n, def: 3),
            Param(id: 'offseti', label: 'Offset', look: ParamLook.bipolar, type: WType.i32, min: -n, max: n),
          ]
        : _taRange;
    final shape = doc.s2['ta_shape'] ?? 'Square';
    String r(String id) => brief(doc.get(id), 0);
    final adv = [..._taAdv, _taSeedP];
    final open = doc.b['ta.adv'] ?? false;
    return Sect(
      title: 'Text Animator',
      tone: Pop.toneTextAnim,
      mark: CardMark.textAnim,
      dim: !on,
      keyIds: _taKeys,
      brief: [
        _taBasedLong[(doc.get('ta_based') ?? 0).round().clamp(0, 3)],
        index ? '#${r('ta_starti')}–${r('ta_endi')}' : '${r('ta_start')}–${r('ta_end')}%',
        shape,
        if ((doc.get(index ? 'ta_offseti' : 'ta_offset') ?? 0) != 0) 'Offset ${r(index ? 'ta_offseti' : 'ta_offset')}${index ? '' : '%'}',
        if (rand) 'Random',
      ],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('ta.on', v)),
      children: [
        _TaStrip(key: const ValueKey('ta_strip'), word: word, on: on),
        _TaGrid(on: on, [
          for (final p in range) _taTile('ta', p),
          _taTile('ta', _taBasedP),
          (
            1,
            _TaBlock(
              label: 'Shape',
              glyph: CustomPaint(size: const Size(22, 16), painter: _TaShapePaint(math.max(0, _taShapes.indexOf(shape)), on)),
              child: Chooser(id: 'ta_shape', options: _taShapes, word: 'Shapes', enabled: on),
            ),
          ),
        ]),
        const _TaCap('Properties'),
        _TaGrid(on: on, [
          for (final p in _taProps)
            if (p.look == ParamLook.colour)
              (
                1,
                _TaBlock(
                  label: p.label,
                  glyph: _Swatch(doc.s2['ta_${p.id}']),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Text(doc.s2['ta_${p.id}'] ?? '—', maxLines: 1, overflow: TextOverflow.ellipsis, style: _blockValue(on ? Grey.g95 : Grey.g56)),
                  ),
                ),
              )
            else
              _taTile('ta', p),
        ]),
        _TaFold('ta.adv', 'Advanced', adv.length + 1),
        if (open) _TaGrid(on: on, [for (final p in adv) _taTile('ta', p), _taTile('para', _paraGroupP)]),
      ],
    );
  }
}

/// The selector made visible: the sample drawn letter by letter, each lit, lifted and moved by how much the range picks it,
/// with the range itself as a bar underneath. Drag a bracket to move Start or End, drag between them to move Offset.
/// contract: the drag is sideways only (like every number), one drag is one undo, and the values it writes are the blocks' own ids.
class _TaStrip extends StatefulWidget {
  const _TaStrip({super.key, required this.word, required this.on});
  final String word;
  final bool on;
  @override
  State<_TaStrip> createState() => _TaStripState();
}

class _TaStripState extends State<_TaStrip> {
  static const _h = 48.0, _pad = 10.0;
  String? _id;
  double _acc = 0, _lo = 0, _hi = 0;
  int _hot = -1;

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, sel = _taSelect(doc, widget.word);
    final live = widget.on && !x.cfg.locked;
    double g(String id) => doc.get(id) ?? 0;
    final fx = (
      dx: g('ta_pos.x'),
      dy: g('ta_pos.y'),
      sx: doc.get('ta_scale.x') ?? 100,
      sy: doc.get('ta_scale.y') ?? 100,
      rot: g('ta_rot'),
      op: doc.get('ta_opacity') ?? 100,
      track: g('ta_track'),
    );
    int pick(double px, double w) {
      final tw = w - _pad * 2, sx = _pad + sel.s.clamp(0.0, 1.0) * tw, ex = _pad + sel.e.clamp(0.0, 1.0) * tw;
      final ds = (px - sx).abs(), de = (px - ex).abs();
      if (math.min(ds, de) <= 8) return ds <= de ? 0 : 1;
      if (px > math.min(sx, ex) && px < math.max(sx, ex)) return 2;
      return ds <= de ? 0 : 1;
    }

    return Padding(
      padding: const EdgeInsets.only(bottom: 2),
      child: LayoutBuilder(
        builder: (context, box) => MouseRegion(
          cursor: live ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.basic,
          onHover: live ? (e) => setState(() => _hot = pick(e.localPosition.dx, box.maxWidth)) : null,
          onExit: (_) => setState(() => _hot = -1),
          child: Listener(
            onPointerDown: live
                ? (e) {
                    final k = pick(e.localPosition.dx, box.maxWidth), sfx = sel.index ? 'i' : '';
                    final span = sel.index ? sel.n.toDouble() : 100.0;
                    _id = 'ta_${['start', 'end', 'offset'][k]}$sfx';
                    _lo = k == 2 ? -span : 0;
                    _hi = span;
                    _acc = doc.get(_id!) ?? 0;
                    doc.beginGesture();
                    setState(() => _hot = k);
                  }
                : null,
            onPointerMove: (e) {
              final id = _id;
              if (id == null) return;
              _acc = (_acc + e.delta.dx / (box.maxWidth - _pad * 2) * (sel.index ? sel.n : 100)).clamp(_lo, _hi);
              if (_acc.roundToDouble() != doc.get(id)) doc.set(id, _acc.roundToDouble());
            },
            onPointerUp: (_) => _end(doc),
            onPointerCancel: (_) => _end(doc),
            child: Container(
              height: _h,
              decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
              child: CustomPaint(
                size: Size(box.maxWidth, _h),
                painter: _TaStripPaint(widget.word, sel.amt, sel.s, sel.e, fx, widget.on ? Pop.toneTextAnim : Grey.g56, _id != null || _hot >= 0 ? _hot : -1),
              ),
            ),
          ),
        ),
      ),
    );
  }

  void _end(Doc doc) {
    if (_id != null) doc.endGesture();
    setState(() => _id = null);
  }
}

typedef _TaFx = ({double dx, double dy, double sx, double sy, double rot, double op, double track});

class _TaStripPaint extends CustomPainter {
  const _TaStripPaint(this.word, this.amt, this.s, this.e, this.fx, this.tone, this.hot);
  final String word;
  final List<double> amt;
  final double s, e;
  final _TaFx fx;
  final Color tone;
  final int hot;

  @override
  void paint(Canvas c, Size size) {
    const fs = 17.0, pad = _TaStripState._pad, bandY = 19.0;
    final base = T.title(Grey.g95).copyWith(fontSize: fs, fontWeight: FontWeight.w700, height: 1);
    TextPainter tp(String ch, Color col) => TextPainter(
      text: TextSpan(
        text: ch,
        style: base.copyWith(color: col),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    final chars = word.split(''), ghosts = [for (final ch in chars) tp(ch, Grey.g26)];
    final total = ghosts.fold(0.0, (a, t) => a + t.width), k = math.min(1.0, (size.width - pad * 2) / math.max(1, total));
    var x = (size.width - total * k) / 2, shift = 0.0;
    double lerp(double a, double b, double t) => a + (b - a) * t;
    for (var i = 0; i < chars.length; i++) {
      final a = amt[i], g = ghosts[i], w = g.width * k, cx = x + w / 2;
      c.save();
      c.translate(cx, bandY);
      c.scale(k);
      g.paint(c, Offset(-g.width / 2, -g.height / 2));
      c.restore();
      final lit = tp(chars[i], Color.lerp(Grey.g63, tone, a.abs().clamp(0.0, 1.0))!.withValues(alpha: lerp(1, fx.op / 100, a).clamp(0.0, 1.0)));
      c.save();
      c.translate(cx + shift + (fx.dx * .1).clamp(-10.0, 10.0) * a, bandY + (fx.dy * .1).clamp(-10.0, 10.0) * a - 4 * a.abs());
      c.rotate(fx.rot * a * math.pi / 180);
      c.scale(k * lerp(1, fx.sx / 100, a).clamp(0.05, 3.0), k * lerp(1, fx.sy / 100, a).clamp(0.05, 3.0));
      lit.paint(c, Offset(-lit.width / 2, -lit.height / 2));
      c.restore();
      x += w;
      shift += fx.track / 1000 * fs * k * a;
    }
    final tw = size.width - pad * 2, y = size.height - 8;
    final (lo, hi) = s <= e ? (s, e) : (e, s);
    final l = pad + lo.clamp(0.0, 1.0) * tw, r = pad + hi.clamp(0.0, 1.0) * tw;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(pad, y - 1.5, tw, 3), const Radius.circular(1.5)), Paint()..color = Grey.g20);
    if (r > l) c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(l, y - 1.5, r, y + 1.5), const Radius.circular(1.5)), Paint()..color = tone);
    void bracket(double bx, int dir, bool lit) {
      final p = Paint()
        ..color = lit ? Grey.g100 : Grey.g76
        ..strokeWidth = 1.5
        ..style = PaintingStyle.stroke;
      c.drawPath(
        Path()
          ..moveTo(bx + dir * 3, y - 5)
          ..lineTo(bx, y - 5)
          ..lineTo(bx, y + 5)
          ..lineTo(bx + dir * 3, y + 5),
        p,
      );
    }

    final sx = pad + s.clamp(0.0, 1.0) * tw, ex = pad + e.clamp(0.0, 1.0) * tw;
    bracket(sx, s <= e ? 1 : -1, hot == 0 || hot == 2);
    bracket(ex, s <= e ? -1 : 1, hot == 1 || hot == 2);
  }

  @override
  bool shouldRepaint(_TaStripPaint o) =>
      o.word != word ||
      o.amt.length != amt.length ||
      o.amt.indexed.any((e) => e.$2 != amt[e.$1]) ||
      o.s != s ||
      o.e != e ||
      o.fx != fx ||
      o.tone != tone ||
      o.hot != hot;
}

/// The selector's shape as a tiny curve (range drawn from 20 % to 80 %), so the chooser reads before its word.
class _TaShapePaint extends CustomPainter {
  const _TaShapePaint(this.shape, this.on);
  final int shape;
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    final r = Rect.fromCenter(center: s.center(Offset.zero), width: 18, height: 12);
    c.drawLine(r.bottomLeft, r.bottomRight, Paint()..color = Grey.g38);
    final path = Path();
    for (var i = 0; i <= 36; i++) {
      final u = i / 36, v = _taAmount(shape, .2, .8, u, u, 0, 0);
      final p = Offset(r.left + u * r.width, r.bottom - v * r.height);
      i == 0 ? path.moveTo(p.dx, p.dy) : path.lineTo(p.dx, p.dy);
    }
    c.drawPath(
      path,
      Paint()
        ..color = on ? Pop.toneTextAnim : Grey.g56
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.4
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(_TaShapePaint o) => o.shape != shape || o.on != on;
}

// ---- Text Morph (motolii-render extensions/text.rs: target = another text layer, amount 0–100 %) -----------------------------
const _tmTargets = ['None', 'Hero title', 'Caption', 'Lower third', 'Credits'];
const _tmAmount = Param(id: 'amount', label: 'Amount', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%');

class TextMorphCard extends StatelessWidget {
  const TextMorphCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(paramDefaults('tm', _tmAmount), strs: const {'tm_target': 'None'});
    final target = doc.s2['tm_target'] ?? 'None', has = target != 'None', t = ((doc.get('tm_amount') ?? 0) / 100).clamp(0.0, 1.0);
    return Sect(
      title: 'Text Morph',
      tone: Pop.toneTextAnim,
      mark: CardMark.textAnim,
      keyIds: const ['tm_amount'],
      brief: [has ? 'Into $target' : 'No target', '${brief(doc.get('tm_amount'), 0)}%'],
      children: [
        if (has)
          Container(
            height: 34,
            margin: const EdgeInsets.only(bottom: 2),
            decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
            child: CustomPaint(painter: _TmPaint(_taWord(doc), t)),
          ),
        _TaGrid([
          (
            2,
            const _TaBlock(
              label: 'Target',
              child: Chooser(id: 'tm_target', options: _tmTargets, word: 'Text layers'),
            ),
          ),
          (1, RowOff(off: !has, narrow: false, child: const ParamTile('tm', _tmAmount))),
        ]),
        if (!has) const Cap('Pick a text layer to morph into; Amount waits until then.'),
      ],
    );
  }
}

/// The outline blend, pictured: the sample's weight and spacing slide from this layer's toward the target's as Amount rises.
class _TmPaint extends CustomPainter {
  const _TmPaint(this.word, this.t);
  final String word;
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final style = T
        .title(Color.lerp(Grey.g76, Pop.toneTextAnim, t)!)
        .copyWith(
          fontSize: 16,
          height: 1,
          fontWeight: FontWeight.lerp(FontWeight.w300, FontWeight.w900, t),
          letterSpacing: 2.5 * t,
          fontStyle: t > .5 ? FontStyle.italic : FontStyle.normal,
        );
    final tp = TextPainter(
      text: TextSpan(text: word, style: style),
      textDirection: TextDirection.ltr,
      maxLines: 1,
    )..layout(maxWidth: s.width - 16);
    tp.paint(c, s.center(Offset.zero) - Offset(tp.width / 2, tp.height / 2));
  }

  @override
  bool shouldRepaint(_TmPaint o) => o.word != word || o.t != t;
}

// ---- Paragraph (TextDocument.justify + TextAlignmentOptions, the AE "More Options") --------------------------------------
const _paraJustify = ['Left', 'Center', 'Right'];
const _paraTrim = ['Normal', 'Space all', 'Space first', 'Trim start'];
const _paraHangEnd = ['None', 'Allow', 'Force'];
const _paraMain = [
  Param(id: 'indent', label: 'First indent', look: ParamLook.horizontal, type: WType.f32, min: -200, max: 200, unit: 'px'),
  Param(id: 'before', label: 'Space before', look: ParamLook.vertical, type: WType.f32, min: 0, max: 200, unit: 'px', upIncreases: false),
  Param(id: 'after', label: 'Space after', look: ParamLook.vertical, type: WType.f32, min: 0, max: 200, unit: 'px'),
];
const _paraMore = [
  _paraGroupP,
  Param(id: 'anchor', label: 'Anchor offset', look: ParamLook.point, type: WType.vec2, min: -500, max: 500, unit: 'px'),
  Param(id: 'autospace', label: 'Autospace', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1, def: 1),
  Param(id: 'hangFirst', label: 'Hang first', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1),
  Param(id: 'hangLast', label: 'Hang last', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1),
  Param(id: 'trim', label: 'Spacing trim', look: ParamLook.choice, type: WType.u32, min: 0, max: 3, options: _paraTrim),
  Param(id: 'hangEnd', label: 'Hang end', look: ParamLook.choice, type: WType.u32, min: 0, max: 2, options: _paraHangEnd),
];

class ParagraphCard extends StatelessWidget {
  const ParagraphCard({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    doc.seed({
      ..._taDefaults([..._paraMain, ..._paraMore], 'para'),
      'para_justify': 0,
      'para_leading': 0,
    });
    double g(String id) => doc.get(id) ?? 0;
    final j = g('para_justify').round().clamp(0, 2), lead = g('para_leading'), trim = g('para_trim').round().clamp(0, 3);
    final hang = g('para_hangFirst') > .5 || g('para_hangLast') > .5 || g('para_hangEnd') > .5;
    final open = doc.b['para.more'] ?? false;
    return Sect(
      title: 'Paragraph',
      tone: Pop.toneTextAnim,
      mark: CardMark.text,
      keyIds: const ['para_indent', 'para_before', 'para_after', 'para_leading'],
      brief: [
        _paraJustify[j],
        lead == 0 ? 'Leading auto' : 'Leading ${brief(lead, 0)} px',
        if (g('para_indent') != 0) 'Indent ${brief(g('para_indent'), 0)}',
        if (trim != 0) _paraTrim[trim],
        if (hang) 'Hanging',
      ],
      children: [
        _TaGrid([
          (2, _TaBlock(label: 'Justify', child: _ParaJustify(j))),
          _taTile('para', _paraMain[0]),
          _taTile('para', _paraMain[1]),
          _taTile('para', _paraMain[2]),
          (
            1,
            _RowInfo(
              unit: null,
              keys: doc.keys['para_leading'] ?? KeyS.off,
              child: NumField(
                id: 'para_leading',
                caption: 'Leading',
                unit: 'px',
                min: 0,
                max: 400,
                perPx: .5,
                zeroWord: 'Auto',
                enabled: !x.cfg.locked,
                glyph: CustomPaint(size: const Size(22, 22), painter: _ParaLeadPaint(lead)),
              ),
            ),
          ),
        ]),
        _TaFold('para.more', 'More options', _paraMore.length),
        if (open) _TaGrid([for (final p in _paraMore) _taTile('para', p)]),
      ],
    );
  }
}

/// Justify as three tiny paragraphs (ragged right, centred, ragged left) in the lab's segmented style.
class _ParaJustify extends StatelessWidget {
  const _ParaJustify(this.index);
  final int index;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return Dis(
      child: Container(
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: Grey.g07, borderRadius: BorderRadius.circular(7)),
        child: Row(
          children: [
            for (var i = 0; i < _paraJustify.length; i++)
              Expanded(
                child: Semantics(
                  button: true,
                  selected: i == index,
                  label: _paraJustify[i],
                  child: GestureDetector(
                    key: ValueKey('para_justify_$i'),
                    behavior: HitTestBehavior.opaque,
                    onTap: () => doc.set('para_justify', i.toDouble()),
                    child: AnimatedContainer(
                      duration: const Duration(milliseconds: 120),
                      alignment: Alignment.center,
                      decoration: BoxDecoration(
                        color: i == index ? Grey.g20 : null,
                        borderRadius: BorderRadius.circular(5),
                        border: Border(bottom: BorderSide(color: i == index ? Role.selected : const Color(0x00000000))),
                      ),
                      child: CustomPaint(size: const Size(14, 10), painter: _ParaJustPaint(i, i == index)),
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _ParaJustPaint extends CustomPainter {
  const _ParaJustPaint(this.align, this.on);
  final int align;
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()
      ..color = on ? Grey.g95 : Grey.g63
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round;
    for (final (i, w) in const [(0, 1.0), (1, .6), (2, .85), (3, .5)].map((e) => (e.$1, e.$2 * s.width))) {
      final y = 1 + i * (s.height - 2) / 3,
          x0 = switch (align) {
            0 => 0.0,
            1 => (s.width - w) / 2,
            _ => s.width - w,
          };
      c.drawLine(Offset(x0, y), Offset(x0 + w, y), p);
    }
  }

  @override
  bool shouldRepaint(_ParaJustPaint o) => o.align != align || o.on != on;
}

/// Leading as three lines whose gap follows the value (Auto draws the font's own gap).
class _ParaLeadPaint extends CustomPainter {
  const _ParaLeadPaint(this.lead);
  final double lead;
  @override
  void paint(Canvas c, Size s) {
    final gap = lead == 0 ? 5.0 : (lead / 6).clamp(2.0, 8.0), o = s.center(Offset.zero);
    final p = Paint()
      ..color = lead == 0 ? Grey.g56 : Grey.g91
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round;
    for (var i = -1; i <= 1; i++) {
      c.drawLine(Offset(o.dx - 8, o.dy + i * gap), Offset(o.dx + 8, o.dy + i * gap), p);
    }
  }

  @override
  bool shouldRepaint(_ParaLeadPaint o) => o.lead != lead;
}
