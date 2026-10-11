part of 'inspector_parts.dart';

// ---- host features: placement (motolii-render extensions placement.rs / blob.rs / motion.rs) --------------------------
const _plShapes = ['Line', 'Circle', 'Grid'], _plPicks = ['Random', 'Iterate'], _plTransforms = ['Each', 'Whole'];
const _plAxes = ['Horizontal', 'Vertical', 'Both', 'Radial'];
const _plSources = ['Brightness', 'Motion', 'Color'], _plFits = ['Position', 'Box'];
const _plLayers = ['None', 'Footage', 'Camera feed', 'Hero title', 'Jewel Field', 'Background'];
const _plUnit = Param(id: 'u', label: '', look: ParamLook.horizontal, type: WType.f32, min: 0, max: 1);

/// Blocks flowed like [ParamGrid]: every run starts on a new line; a block takes `span` columns of a grid that widens with the panel.
class _PlGrid extends StatelessWidget {
  const _PlGrid(this.runs);
  final List<List<(int, Widget)>> runs;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      const g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <(List<(int, Widget)>, bool)>[];
      for (final run in runs.where((r) => r.isNotEmpty)) {
        var cur = <(int, Widget)>[], used = 0;
        for (final (s0, w) in run) {
          final s = math.min(s0, cols);
          if (used + s > cols) {
            rows.add((cur, false));
            cur = [];
            used = 0;
          }
          cur.add((s, w));
          used += s;
        }
        rows.add((cur, true));
      }
      Widget line(List<(int, Widget)> items, bool last) {
        final filled = items.fold<int>(0, (a, e) => a + e.$1);
        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final (i, (s, w)) in items.indexed) ...[if (i > 0) const SizedBox(width: g), Expanded(flex: s, child: w)],
            if (last && filled < cols) ...[const SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: const EdgeInsets.only(top: 4),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (final (i, (r, last)) in rows.indexed) ...[if (i > 0) const SizedBox(height: g), line(r, last)],
          ],
        ),
      );
    },
  );
}

/// A number block. contract: [whole] values land on integers; [tick] false drops the range bar for ranges too wide to read as a fill.
Widget _plNum(
  BuildContext context,
  String id,
  String caption, {
  String unit = '',
  int dec = 0,
  double? min,
  double? max,
  double perPx = 1,
  bool bipolar = false,
  bool whole = false,
  bool tick = true,
  Widget? glyph,
  bool owns = false,
  bool tall = false,
}) {
  final doc = Ctx.of(context).doc;
  return _RowInfo(
    unit: null,
    keys: doc.keys[id] ?? KeyS.off,
    child: NumField(
      id: id,
      caption: caption,
      unit: unit,
      decimals: dec,
      min: min,
      max: tick ? max : null,
      perPx: perPx,
      step: dec > 0 ? math.pow(10, 1 - dec).toDouble() : 1,
      bipolar: bipolar,
      glyph: glyph,
      glyphOwnsPointer: owns,
      glyphTall: tall,
      enabled: !RowOff.offOf(context),
      onSet: whole || !tick ? (v) => doc.set(id, (whole ? v.roundToDouble() : v).clamp(min ?? -1e18, max ?? 1e18)) : null,
    ),
  );
}

Widget _plGauge(double? v, [Param p = _plUnit, _GaugeKind k = _GaugeKind.across]) => SizedBox.square(
  dimension: Pop.glyph,
  child: FittedBox(child: _Gauge(k, v, p)),
);

Param _plCount(double lo, double hi) => Param(id: 'n', label: '', look: ParamLook.stepper, type: WType.u32, min: lo, max: hi, def: lo);

/// The block skeleton for what is not a number: name top-left, the control along the bottom.
class _PlShell extends StatelessWidget {
  const _PlShell({required this.label, required this.child});
  final String label;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return Container(
      height: Pop.tile,
      decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
      child: Stack(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(Pop.tilePad, 6, Pop.tilePad, 4),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                child,
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// A short choice (<= 4) as a block; the index lives in doc.v like an effect's enum.
class _PlChoice extends StatelessWidget {
  const _PlChoice(this.id, this.label, this.options, {this.onPick});
  final String id, label;
  final List<String> options;
  final ValueChanged<int>? onPick;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return _PlShell(
      label: label,
      child: SizedBox(
        height: 22,
        child: Dis(
          child: Segmented(
            items: options,
            index: (doc.get(id) ?? 0).round().clamp(0, options.length - 1),
            expand: true,
            onChanged: (k) => onPick != null ? onPick!(k) : doc.set(id, k.toDouble()),
          ),
        ),
      ),
    );
  }
}

Widget _plToggle(String fx, String id, String label) => ParamTile(fx, Param(id: id, label: label, look: ParamLook.toggle, type: WType.u32, min: 0, max: 1));

class _PlSub extends StatelessWidget {
  const _PlSub(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 8, bottom: 0),
    child: Text(text.toUpperCase(), style: T.micro(Grey.g56).copyWith(fontWeight: FontWeight.w700, letterSpacing: .4)),
  );
}

/// A live picture as wide as the card. contract: dragging it moves both axes (it is a glyph, not a number); one drag is one undo.
class _PlStrip extends StatelessWidget {
  const _PlStrip({required this.painter, this.height = 56, this.onDrag});
  final CustomPainter painter;
  final double height;
  final ValueChanged<Offset>? onDrag;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), on = onDrag != null && !x.cfg.locked && !RowOff.offOf(context);
    final pic = Container(
      height: height,
      decoration: BoxDecoration(color: Grey.g10, borderRadius: BorderRadius.circular(6)),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(6),
        child: CustomPaint(painter: painter, size: Size.infinite),
      ),
    );
    if (!on) return pic;
    return MouseRegion(
      cursor: SystemMouseCursors.move,
      child: Listener(
        onPointerDown: (_) => x.doc.beginGesture(),
        onPointerMove: (e) => onDrag!(e.delta),
        onPointerUp: (_) => x.doc.endGesture(),
        onPointerCancel: (_) => x.doc.endGesture(),
        child: pic,
      ),
    );
  }
}

/// Same (seed, index, channel) gives the same value in [-1, 1].
double _plNoise(int seed, int i, int ch) {
  var h = (seed * 73856093) ^ (i * 19349663) ^ (ch * 83492791);
  h = ((h ^ (h >> 13)) * 1274126177) & 0x7FFFFFFF;
  h ^= h >> 16;
  return (h & 0xFFFF) / 0xFFFF * 2 - 1;
}

// ---- Repeater ---------------------------------------------------------------------------------------------------------
enum _PlMk { x, y, z, rot, scale, op, delay }

class _PlRow {
  const _PlRow(this.label, this.mk, this.each, this.rand, {this.unit = '', this.dec = 0, this.lo, this.hi, this.rhi, this.perPx = 1});
  final String label, unit, rand;
  final String? each;
  final _PlMk? mk;
  final int dec;
  final double? lo, hi, rhi;
  final double perPx;
}

const _repRows = [
  _PlRow('Position X', _PlMk.x, 'rep_pos_each.x', 'rep_pos_rand.x', unit: 'px'),
  _PlRow('Position Y', _PlMk.y, 'rep_pos_each.y', 'rep_pos_rand.y', unit: 'px'),
  _PlRow('Position Z', _PlMk.z, 'rep_posz_each', 'rep_posz_rand', unit: 'px'),
  _PlRow('Rotation', _PlMk.rot, 'rep_rot_each', 'rep_rot_rand', unit: '°', rhi: 360),
  _PlRow('Scale', _PlMk.scale, 'rep_scale_each', 'rep_scale_rand', unit: '%', lo: -100, hi: 100, rhi: 100, perPx: .5),
  _PlRow('Opacity', _PlMk.op, 'rep_op_each', 'rep_op_rand', dec: 2, lo: -1, hi: 1, rhi: 1, perPx: .01),
];
const _repAdvanced = [
  _PlRow('Delay', _PlMk.delay, 'rep_delay_each', 'rep_delay_rand', unit: 's', dec: 2, perPx: .01),
  _PlRow('Seed', null, null, 'rep_seed', rhi: 9999, perPx: .2),
];

const _repDefaults = <String, double>{
  'rep_count': 3, 'rep_mode': 0, 'rep_pick': 0, 'rep_transform': 0, 'rep_columns': 3, 'rep_radius': 200, 'rep_start': 0, 'rep_sweep': 360, //
  'rep_pos_each.x': 100, 'rep_pos_each.y': 0, 'rep_posz_each': 0, 'rep_rot_each': 0, 'rep_scale_each': 0, 'rep_op_each': 0, 'rep_delay_each': 0,
  'rep_pos_rand.x': 0, 'rep_pos_rand.y': 0, 'rep_posz_rand': 0, 'rep_rot_rand': 0, 'rep_scale_rand': 0, 'rep_op_rand': 0, 'rep_delay_rand': 0,
  'rep_seed': 0,
};

typedef _PlCopy = ({double x, double y, double rot, double scale, double op});

/// The copies as placement.rs places them (2D part), capped for the picture.
List<_PlCopy> _repCopies(Doc doc) {
  double g(String id) => doc.get(id) ?? 0;
  final n = g('rep_count').round().clamp(1, 1000), mode = g('rep_mode').round(), cols = math.max(1, g('rep_columns').round());
  final ex = g('rep_pos_each.x'), ey = g('rep_pos_each.y'), rx = g('rep_pos_rand.x'), ry = g('rep_pos_rand.y'), seed = g('rep_seed').round();
  final r = g('rep_radius'), start = g('rep_start'), sweep = g('rep_sweep');
  return [
    for (var i = 0; i < math.min(n, 400); i++)
      () {
        double u(int ch) => _plNoise(seed, i, ch);
        var (x, y) = switch (mode) {
          1 => () {
            final a = (start + sweep * (sweep >= 360 ? i / n : i / (math.max(n, 2) - 1))) * math.pi / 180;
            return (r * math.cos(a) + ex * i, r * math.sin(a) + ey * i);
          }(),
          2 => (ex * (i % cols), ey * (i ~/ cols)),
          _ => (ex * i, ey * i),
        };
        x += rx * u(0);
        y += ry * u(1);
        return (
          x: x,
          y: y,
          rot: g('rep_rot_each') * i + g('rep_rot_rand') * u(2),
          scale: math.pow(math.max(0.0, 1 + g('rep_scale_each') / 100), i).toDouble() * math.max(0.0, 1 + g('rep_scale_rand') / 100 * u(3)),
          op: (1 + g('rep_op_each') * i + g('rep_op_rand') * u(4)).clamp(0.0, 1.0),
        );
      }(),
  ];
}

class _RepPaint extends CustomPainter {
  const _RepPaint(this.copies, this.total);
  final List<_PlCopy> copies;
  final int total;
  @override
  void paint(Canvas c, Size s) {
    if (copies.isEmpty) return;
    var x0 = double.infinity, x1 = -double.infinity, y0 = double.infinity, y1 = -double.infinity;
    for (final p in copies) {
      x0 = math.min(x0, p.x);
      x1 = math.max(x1, p.x);
      y0 = math.min(y0, p.y);
      y1 = math.max(y1, p.y);
    }
    const pad = 12.0;
    final kx = x1 - x0 < 1e-3 ? double.infinity : (s.width - 2 * pad) / (x1 - x0);
    final ky = y1 - y0 < 1e-3 ? double.infinity : (s.height - 2 * pad) / (y1 - y0);
    var k = math.min(math.min(kx, ky), 2.0);
    if (!k.isFinite) k = 1;
    final o = s.center(Offset.zero) - Offset((x0 + x1) / 2, (y0 + y1) / 2) * k;
    final step = copies.length > 1 ? (Offset(copies[1].x, copies[1].y) - Offset(copies[0].x, copies[0].y)).distance * k : 0.0;
    final base = step > 0 ? (step * .6).clamp(2.5, 8.0) : 8.0;
    for (var i = copies.length - 1; i >= 0; i--) {
      final p = copies[i], side = (base * p.scale).clamp(.8, 28.0);
      c.save();
      c.translate(o.dx + p.x * k, o.dy + p.y * k);
      c.rotate(p.rot * math.pi / 180);
      final r = RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: side, height: side), Radius.circular(side * .2));
      c.drawRRect(r, Paint()..color = (i == 0 ? Pop.tonePlace : Grey.g91).withValues(alpha: math.max(.12, p.op)));
      if (p.rot != 0 && side > 4) {
        c.drawLine(
          Offset.zero,
          Offset(side / 2, 0),
          Paint()
            ..color = Grey.g10
            ..strokeWidth = 1,
        );
      }
      c.restore();
    }
    if (total > copies.length) {
      final tp = TextPainter(
        text: TextSpan(text: '+${total - copies.length}', style: T.micro(Grey.g56)),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(c, Offset(s.width - tp.width - 6, s.height - tp.height - 4));
    }
  }

  @override
  bool shouldRepaint(_RepPaint o) => o.copies != copies || o.total != total;
}

/// The start and sweep of a circle as an arc.
class _PlArc extends StatelessWidget {
  const _PlArc(this.start, this.sweep);
  final double start, sweep;
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(22, 22), painter: _PlArcPaint(start, sweep));
}

class _PlArcPaint extends CustomPainter {
  const _PlArcPaint(this.start, this.sweep);
  final double start, sweep;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), r = Rect.fromCircle(center: o, radius: 8);
    c.drawCircle(o, 8, Paint()..color = Grey.g26);
    c.drawArc(r, start * math.pi / 180, sweep.clamp(0, 360) * math.pi / 180, true, Paint()..color = Grey.g91);
  }

  @override
  bool shouldRepaint(_PlArcPaint o) => o.start != start || o.sweep != sweep;
}

/// A row's small picture of what it moves.
class _PlMark extends StatelessWidget {
  const _PlMark(this.mk);
  final _PlMk mk;
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(22, 18), painter: _PlMarkPaint(mk));
}

class _PlMarkPaint extends CustomPainter {
  const _PlMarkPaint(this.mk);
  final _PlMk mk;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero);
    final ln = Paint()
      ..color = Grey.g63
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.3
      ..strokeCap = StrokeCap.round;
    void arrow(Offset a, Offset b) {
      c.drawLine(a, b, ln);
      final d = (b - a) / (b - a).distance * 3;
      c.drawLine(b, b - d + Offset(-d.dy, d.dx), ln);
      c.drawLine(b, b - d - Offset(-d.dy, d.dx), ln);
    }

    switch (mk) {
      case _PlMk.x:
        arrow(o - const Offset(6, 0), o + const Offset(6, 0));
      case _PlMk.y:
        arrow(o - const Offset(0, 6), o + const Offset(0, 6));
      case _PlMk.z:
        arrow(o + const Offset(-5, 5), o + const Offset(5, -5));
      case _PlMk.rot:
        c.drawArc(Rect.fromCircle(center: o, radius: 5.5), -math.pi * .9, math.pi * 1.5, false, ln);
        final tip = o + Offset(math.cos(math.pi * .6), math.sin(math.pi * .6)) * 5.5;
        c.drawLine(tip, tip + const Offset(3, -.5), ln);
        c.drawLine(tip, tip + const Offset(.5, -3), ln);
      case _PlMk.scale:
        c.drawRect(Rect.fromCenter(center: o, width: 12, height: 12), ln);
        c.drawRect(Rect.fromLTWH(o.dx - 6, o.dy, 6, 6), Paint()..color = Grey.g63);
      case _PlMk.op:
        c.drawCircle(o, 6, ln);
        c.drawArc(Rect.fromCircle(center: o, radius: 6), -math.pi / 2, math.pi, true, Paint()..color = Grey.g63);
      case _PlMk.delay:
        c.drawCircle(o, 6, ln);
        c.drawLine(o, o + const Offset(0, -4), ln);
        c.drawLine(o, o + const Offset(3, 0), ln);
    }
  }

  @override
  bool shouldRepaint(_PlMarkPaint o) => o.mk != mk;
}

/// The copies' grid: one row per property, Each (by copy number) beside Random (by seed).
class _RepMatrix extends StatelessWidget {
  const _RepMatrix(this.rows, {this.header = true, required this.enabled});
  final List<_PlRow> rows;
  final bool header, enabled;
  @override
  Widget build(BuildContext context) {
    final cfg = Ctx.of(context).cfg;
    Widget head(String s) => Expanded(
      child: Text(
        s,
        textAlign: TextAlign.center,
        style: T.micro(Grey.g63).copyWith(fontWeight: FontWeight.w700),
      ),
    );
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (header)
          Padding(
            padding: const EdgeInsets.only(top: 6, bottom: 2),
            child: SizedBox(
              height: 14,
              child: Row(
                children: [
                  const SizedBox(width: _labelWPop),
                  head('Each'),
                  const SizedBox(width: 4),
                  head('Random'),
                  if (cfg.resetArrow) const SizedBox(width: _resetW),
                ],
              ),
            ),
          ),
        for (final r in rows)
          PropRow(
            id: 'rep_row.${r.label}',
            ids: [?r.each, r.rand],
            label: r.label,
            enabled: enabled,
            lead: r.mk != null ? _PlMark(r.mk!) : _Dice(id: r.rand, p: _plCount(0, r.rhi ?? 9999)),
            cells: [
              if (r.each case final e?)
                NumField(id: e, unit: r.unit, decimals: r.dec, min: r.lo, max: r.hi, perPx: r.perPx, step: r.dec > 0 ? .1 : 1, bipolar: r.lo != null)
              else
                const SizedBox(height: Pop.cell),
              NumField(id: r.rand, unit: r.unit, decimals: r.dec, min: 0, max: r.rhi, perPx: r.perPx, step: r.dec > 0 ? .1 : 1),
            ],
          ),
      ],
    );
  }
}

class RepeaterCard extends StatelessWidget {
  const RepeaterCard({super.key});

  /// placement.rs default_for: a Circle's Position Each defaults to 0 so the ring closes round the layer; an untouched Each follows the shape.
  static void _pickMode(Doc doc, int k) {
    final old = (doc.get('rep_mode') ?? 0).round();
    final nx = k == 1 ? 0.0 : 100.0, ox = old == 1 ? 0.0 : 100.0;
    final untouched = doc.get('rep_pos_each.x') == ox && doc.get('rep_pos_each.y') == 0;
    doc.d['rep_pos_each.x'] = nx;
    doc.setMany({'rep_mode': k.toDouble(), if (untouched) 'rep_pos_each.x': nx});
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc..seed(_repDefaults, flags: {'rep.on': true});
    final on = doc.b['rep.on'] ?? true, mode = (doc.get('rep_mode') ?? 0).round().clamp(0, 2);
    double g(String id) => doc.get(id) ?? 0;
    final copies = _repCopies(doc), total = g('rep_count').round().clamp(1, 1000);
    final random = _repRows.any((r) => g(r.rand) != 0) || g('rep_delay_rand') != 0;
    return Sect(
      title: 'Repeater',
      tone: Pop.tonePlace,
      mark: CardMark.repeater,
      dim: !on,
      keyIds: _repDefaults.keys.toList(),
      brief: [
        '$total × ${_plShapes[mode]}',
        if (mode == 1) 'R ${brief(g('rep_radius'), 0)} px',
        if (mode == 2) '${brief(g('rep_columns'), 0)} cols',
        'Each ${brief(g('rep_pos_each.x'), 0)}, ${brief(g('rep_pos_each.y'), 0)} px',
        if (random) 'Random',
      ],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('rep.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _PlStrip(painter: _RepPaint(copies, total)),
              _PlGrid([
                [
                  (
                    1,
                    _plNum(
                      context,
                      'rep_count',
                      'Count',
                      min: 1,
                      max: 1000,
                      perPx: .2,
                      whole: true,
                      tick: false,
                      glyph: _Steps(id: 'rep_count', p: _plCount(1, 1000)),
                      owns: true,
                      tall: true,
                    ),
                  ),
                  (2, _PlChoice('rep_mode', 'Along', _plShapes, onPick: (k) => _pickMode(doc, k))),
                ],
                [
                  if (mode == 2)
                    (
                      1,
                      _plNum(
                        context,
                        'rep_columns',
                        'Columns',
                        min: 1,
                        max: 1000,
                        perPx: .1,
                        whole: true,
                        tick: false,
                        glyph: _Steps(id: 'rep_columns', p: _plCount(1, 1000)),
                        owns: true,
                        tall: true,
                      ),
                    ),
                  if (mode == 1) ...[
                    (1, _plNum(context, 'rep_radius', 'Radius', unit: 'px', min: 0, tick: false)),
                    (
                      1,
                      _plNum(
                        context,
                        'rep_start',
                        'Start',
                        unit: '°',
                        glyph: _AngleDial(
                          id: 'rep_start',
                          p: const Param(id: 'start', label: '', look: ParamLook.angle, type: WType.f32),
                        ),
                        owns: true,
                      ),
                    ),
                    (1, _plNum(context, 'rep_sweep', 'Sweep', unit: '°', min: 0, max: 360, perPx: 1.8, glyph: _PlArc(g('rep_start'), g('rep_sweep')))),
                  ],
                  (1, _PlChoice('rep_pick', 'Pick', _plPicks)),
                  (1, _PlChoice('rep_transform', 'Transform', _plTransforms)),
                ],
              ]),
              _RepMatrix(_repRows, enabled: on),
              AdvancedFold('rep.adv', summary: '${_repAdvanced.length} more', [_RepMatrix(_repAdvanced, header: false, enabled: on)]),
            ],
          ),
        ),
      ],
    );
  }
}

// ---- Mirror -----------------------------------------------------------------------------------------------------------
/// A small flag and its reflections about the centre, as placement.rs mirrors() places them: x ↦ R(θ) S (x − c) + c.
class _MirPaint extends CustomPainter {
  const _MirPaint(this.axis, this.segments, this.cx, this.cy);
  final int axis, segments;
  final double cx, cy;
  static const k = .25;
  static const flag = [Offset(20, -60), Offset(90, -40), Offset(30, -12)];
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), cc = Offset(cx, cy);
    final copies = switch (axis) {
      0 => [(0.0, 1.0, 1.0), (0.0, -1.0, 1.0)],
      1 => [(0.0, 1.0, 1.0), (0.0, 1.0, -1.0)],
      2 => [(0.0, 1.0, 1.0), (0.0, -1.0, 1.0), (0.0, 1.0, -1.0), (0.0, -1.0, -1.0)],
      _ => [for (var i = 0; i < segments; i++) (2 * math.pi * i / segments, i.isOdd ? -1.0 : 1.0, 1.0)],
    };
    final guide = Paint()
      ..color = Grey.g38
      ..strokeWidth = 1;
    final at = o + cc * k;
    if (axis == 0 || axis == 2) c.drawLine(Offset(at.dx, 0), Offset(at.dx, s.height), guide);
    if (axis == 1 || axis == 2) c.drawLine(Offset(0, at.dy), Offset(s.width, at.dy), guide);
    if (axis == 3) {
      for (var i = 0; i < segments; i++) {
        final a = 2 * math.pi * (i + .5) / segments;
        c.drawLine(at, at + Offset(math.cos(a), math.sin(a)) * s.width, guide);
      }
    }
    for (final (i, (th, sx, sy)) in copies.indexed) {
      Offset tf(Offset p) {
        final d = Offset((p.dx - cc.dx) * sx, (p.dy - cc.dy) * sy);
        return o + (Offset(d.dx * math.cos(th) - d.dy * math.sin(th), d.dx * math.sin(th) + d.dy * math.cos(th)) + cc) * k;
      }

      final path = Path()..addPolygon([for (final p in flag) tf(p)], true);
      c.drawPath(path, Paint()..color = i == 0 ? Pop.tonePlace : Grey.g76.withValues(alpha: .55));
    }
    c.drawCircle(at, 2.5, Paint()..color = Grey.g95);
  }

  @override
  bool shouldRepaint(_MirPaint o) => o.axis != axis || o.segments != segments || o.cx != cx || o.cy != cy;
}

class MirrorCard extends StatelessWidget {
  const MirrorCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc..seed({'mir_mode': 0, 'mir_segments': 6, 'mir_centre.x': 0, 'mir_centre.y': 0}, flags: {'mir.on': true});
    final on = doc.b['mir.on'] ?? true, mode = (doc.get('mir_mode') ?? 0).round().clamp(0, 3);
    final seg = (doc.get('mir_segments') ?? 6).round().clamp(2, 64), cx = doc.get('mir_centre.x') ?? 0, cy = doc.get('mir_centre.y') ?? 0;
    return Sect(
      title: 'Mirror',
      tone: Pop.tonePlace,
      mark: CardMark.mirror,
      dim: !on,
      keyIds: const ['mir_segments', 'mir_centre.x', 'mir_centre.y'],
      brief: [mode == 3 ? 'Radial × $seg' : _plAxes[mode], 'Centre ${brief(cx, 0)}, ${brief(cy, 0)} px'],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('mir.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _PlStrip(
                height: 64,
                painter: _MirPaint(mode, seg, cx, cy),
                onDrag: (d) =>
                    doc.setMany({'mir_centre.x': (cx + d.dx / _MirPaint.k).roundToDouble(), 'mir_centre.y': (cy + d.dy / _MirPaint.k).roundToDouble()}),
              ),
              _PlGrid([
                [(3, const _PlChoice('mir_mode', 'Axis', _plAxes))],
                [
                  (1, _plNum(context, 'mir_centre.x', 'Centre X', unit: 'px')),
                  (1, _plNum(context, 'mir_centre.y', 'Y', unit: 'px')),
                  if (mode == 3)
                    (
                      1,
                      _plNum(
                        context,
                        'mir_segments',
                        'Segments',
                        min: 2,
                        max: 64,
                        perPx: .1,
                        whole: true,
                        glyph: _Steps(id: 'mir_segments', p: _plCount(2, 64)),
                        owns: true,
                        tall: true,
                      ),
                    ),
                ],
              ]),
            ],
          ),
        ),
      ],
    );
  }
}

// ---- Blob Track -------------------------------------------------------------------------------------------------------
class BlobTrackCard extends StatelessWidget {
  const BlobTrackCard({super.key});
  static const _defaults = <String, double>{
    'blob_mode': 0, 'blob_threshold': .5, 'blob_invert': 0, 'blob_red': 1, 'blob_green': 0, 'blob_blue': 0, 'blob_tolerance': .25, //
    'blob_detail': 960, 'blob_min_area': 200, 'blob_max_area': 1e9, 'blob_separation': 2, 'blob_max_blobs': 100,
    'blob_persist': 1, 'blob_max_move': 40, 'blob_revive': 5, 'blob_fit': 1, 'blob_material.x': 100, 'blob_material.y': 100,
  };
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc..seed(_defaults, strs: {'blob_source': _plLayers.first}, flags: {'blob.on': true});
    double g(String id) => doc.get(id) ?? 0;
    final on = doc.b['blob.on'] ?? true, mode = g('blob_mode').round().clamp(0, 2), src = doc.s2['blob_source'] ?? _plLayers.first;
    String hex =
        '#${[g('blob_red'), g('blob_green'), g('blob_blue')].map((c) => (c.clamp(0, 1) * 255).round().toRadixString(16).padLeft(2, '0')).join().toUpperCase()}';
    Widget unit(String id, String label) => _plNum(context, id, label, dec: 2, min: 0, max: 1, perPx: .005, glyph: _plGauge(doc.get(id)));
    return Sect(
      title: 'Blob Track',
      tone: Pop.tonePlace,
      mark: CardMark.track,
      dim: !on,
      keyIds: _defaults.keys.toList(),
      brief: [src == _plLayers.first ? 'No layer' : src, '${_plSources[mode]} ${brief(g('blob_threshold'), 2)}', _plFits[g('blob_fit').round().clamp(0, 1)]],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('blob.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const _PlSub('Find'),
              _PlGrid([
                [
                  (
                    3,
                    const _PlShell(
                      label: 'Track Layer',
                      child: SizedBox(
                        height: 22,
                        child: Chooser(id: 'blob_source', options: _plLayers, word: 'Layers', searchable: true),
                      ),
                    ),
                  ),
                ],
                [
                  (2, const _PlChoice('blob_mode', 'Find By', _plSources)),
                  (1, unit('blob_threshold', 'Threshold')),
                  (1, _plToggle('blob', 'invert', 'Invert')),
                ],
              ]),
              if (mode == 2) ...[
                const _PlSub('Colour'),
                _PlGrid([
                  [
                    (1, unit('blob_red', 'Red')),
                    (1, unit('blob_green', 'Green')),
                    (1, unit('blob_blue', 'Blue')),
                    (1, _plNum(context, 'blob_tolerance', 'Tolerance', dec: 2, min: 0, max: 2, perPx: .01, glyph: _Swatch(hex))),
                  ],
                ]),
              ],
              const _PlSub('Place'),
              _PlGrid([
                [
                  (1, const _PlChoice('blob_fit', 'Fit', _plFits)),
                  (1, _plNum(context, 'blob_material.x', 'Material W', unit: 'px', min: .001, tick: false)),
                  (1, _plNum(context, 'blob_material.y', 'H', unit: 'px', min: .001, tick: false)),
                ],
              ]),
              AdvancedFold('blob.adv', summary: '${g('blob_persist') > 0 ? 8 : 6} more', [
                const _PlSub('Filter'),
                _PlGrid([
                  [
                    (1, _plNum(context, 'blob_detail', 'Detail', unit: 'px', min: 120, max: 3840, perPx: 10, whole: true)),
                    (1, _plNum(context, 'blob_min_area', 'Min Size', unit: 'px²', min: 0, max: 1e9, perPx: 10, whole: true, tick: false)),
                    (1, _plNum(context, 'blob_max_area', 'Max Size', unit: 'px²', min: 0, max: 1e9, perPx: 10, whole: true, tick: false)),
                    (1, _plNum(context, 'blob_separation', 'Separation', unit: 'px', min: 0, max: 200, perPx: .5, dec: 1)),
                    (1, _plNum(context, 'blob_max_blobs', 'Max Blobs', min: 1, max: 1000, perPx: .5, whole: true, tick: false)),
                  ],
                ]),
                const _PlSub('Track'),
                _PlGrid([
                  [
                    (1, _plToggle('blob', 'persist', 'Keep IDs')),
                    if (g('blob_persist') > 0) ...[
                      (1, _plNum(context, 'blob_max_move', 'Max Move', unit: 'px', min: 0, max: 10000, whole: true, tick: false)),
                      (1, _plNum(context, 'blob_revive', 'Revive Frames', min: 0, max: 1000, perPx: .2, whole: true, tick: false)),
                    ],
                  ],
                ]),
              ]),
            ],
          ),
        ),
      ],
    );
  }
}

// ---- Motion Blur ------------------------------------------------------------------------------------------------------
/// A dot and its trail, the trail as long as Tune frames of travel.
class _MbPaint extends CustomPainter {
  const _MbPaint(this.tune);
  final double tune;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), w = 4.5 * tune.clamp(0, 4);
    for (var i = 5; i >= 1; i--) {
      c.drawCircle(o + Offset(-w * i / 5 + w / 2, 0), 3.2, Paint()..color = Grey.g91.withValues(alpha: .12 + .1 * (5 - i)));
    }
    c.drawCircle(o + Offset(w / 2, 0), 3.2, Paint()..color = Grey.g95);
  }

  @override
  bool shouldRepaint(_MbPaint o) => o.tune != tune;
}

class MotionBlurCard extends StatelessWidget {
  const MotionBlurCard({super.key});
  static const _parts = [('mb_position', 'Position'), ('mb_scale', 'Scale'), ('mb_angle', 'Angle')];
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc..seed({'mb_tune': 1, 'mb_position': 1, 'mb_scale': 1, 'mb_angle': 1}, flags: {'mb.on': true});
    final on = doc.b['mb.on'] ?? true, tune = doc.get('mb_tune') ?? 1;
    final picked = [
      for (final (id, name) in _parts)
        if ((doc.get(id) ?? 1) >= .5) name,
    ];
    final none = tune <= 0 || picked.isEmpty;
    final live = on && !x.cfg.locked;
    return Sect(
      title: 'Motion Blur',
      tone: Pop.tonePlace,
      mark: CardMark.motionBlur,
      dim: !on,
      keyIds: const ['mb_tune'],
      brief: none ? ['No blur'] : ['Tune ${brief(tune, 2)}', picked.join(' · ')],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('mb.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _PlGrid([
                [
                  (
                    1,
                    _plNum(
                      context,
                      'mb_tune',
                      'Tune',
                      dec: 2,
                      min: 0,
                      max: 4,
                      perPx: .02,
                      glyph: CustomPaint(size: const Size(22, 22), painter: _MbPaint(tune)),
                    ),
                  ),
                  (
                    2,
                    _PlShell(
                      label: 'Blur',
                      child: Row(
                        children: [
                          for (final (i, (id, name)) in _parts.indexed) ...[
                            if (i > 0) const SizedBox(width: 4),
                            Flexible(
                              child: FittedBox(
                                fit: BoxFit.scaleDown,
                                child: Chip(name, on: (doc.get(id) ?? 1) >= .5, enabled: live, onTap: () => doc.set(id, (doc.get(id) ?? 1) >= .5 ? 0 : 1)),
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                  ),
                ],
              ]),
              if (none) Cap(tune <= 0 ? 'No blur while Tune is 0' : 'No blur: pick Position, Scale or Angle'),
            ],
          ),
        ),
      ],
    );
  }
}
