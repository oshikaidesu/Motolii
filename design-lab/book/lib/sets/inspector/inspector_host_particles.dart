part of 'inspector_parts.dart';

// ---- host: particles ---------------------------------------------------------------------------------------------------
/// The particle layer's table (motolii-doc store/particles.rs ROWS): lengths in comp px, time in seconds, angles in degrees (0 = right, -90 = up).
/// Shares (randomness, opacity, bounce) are shown as percent; colours live in doc.s2 as #RRGGBB.
const _ptEmit = [
  Param(id: 'rate', label: 'Rate', look: ParamLook.scrub, type: WType.f32, min: 0, def: 40, unit: '/s'),
  Param(id: 'life', label: 'Life', look: ParamLook.scrub, type: WType.f32, min: .01, def: 2, unit: 's', dec: 2),
  Param(id: 'emitter', label: 'Emitter', look: ParamLook.size, type: WType.vec2, min: 0, unit: 'px'),
];
const _ptMotion = [
  Param(id: 'direction', label: 'Direction', look: ParamLook.angle, type: WType.f32, def: -90, unit: '°'),
  Param(id: 'spread', label: 'Spread', look: ParamLook.angle, type: WType.f32, min: 0, max: 360, def: 30, unit: '°'),
  _ptSpeed,
];
const _ptSpeed = Param(id: 'speed', label: 'Speed', look: ParamLook.scrub, type: WType.f32, def: 240, unit: 'px/s');
const _ptLife = [
  Param(id: 'size', label: 'Size', look: ParamLook.scrub, type: WType.f32, min: 0, def: 8, unit: 'px', dec: 1),
  Param(id: 'size_end', label: 'End Size', look: ParamLook.scrub, type: WType.f32, min: 0, def: 2, unit: 'px', dec: 1),
  Param(id: 'opacity_end', label: 'End Opacity', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%'),
];
const _ptGravity = Param(id: 'gravity', label: 'Gravity', look: ParamLook.bipolar, type: WType.f32, min: -2000, max: 2000, unit: 'px/s²');
const _ptWindX = Param(id: 'wind.x', label: 'Wind X', look: ParamLook.bipolar, type: WType.f32, min: -2000, max: 2000, unit: 'px/s²');
const _ptWindY = Param(id: 'wind.y', label: 'Wind Y', look: ParamLook.bipolar, type: WType.f32, min: -2000, max: 2000, unit: 'px/s²');
const _ptBounce = Param(id: 'bounce', label: 'Bounce', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%');
const _ptFloor = Param(id: 'floor', label: 'Floor', look: ParamLook.scrub, type: WType.f32, def: 300, unit: 'px');
const _ptTurb = Param(id: 'turbulence', label: 'Amount', look: ParamLook.scrub, type: WType.f32, min: 0, unit: 'px');
const _ptTurbSize = Param(id: 'turbulence_size', label: 'Scale', look: ParamLook.scrub, type: WType.f32, min: 1, def: 120, unit: 'px');
const _ptConnect = Param(id: 'connect', label: 'Distance', look: ParamLook.scrub, type: WType.f32, min: 0, unit: 'px');
const _ptLines = [
  Param(id: 'line_width', label: 'Line Width', look: ParamLook.scrub, type: WType.f32, min: 0, max: 100, def: 1, unit: 'px', dec: 1),
  Param(id: 'line_opacity', label: 'Line Opacity', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 60, unit: '%'),
];
const _ptRandom = [
  Param(id: 'life_random', label: 'Life Random', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 30, unit: '%'),
  Param(id: 'speed_random', label: 'Speed Random', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, def: 30, unit: '%'),
];
const _ptSeed = Param(id: 'seed', label: 'Seed', look: ParamLook.seed, type: WType.f32, min: 0, max: 9999);
const _ptAll = [
  ..._ptEmit,
  ..._ptMotion,
  ..._ptLife,
  _ptGravity,
  _ptWindX,
  _ptWindY,
  _ptBounce,
  _ptFloor,
  _ptTurb,
  _ptTurbSize,
  _ptConnect,
  ..._ptLines,
  ..._ptRandom,
  _ptSeed,
];

/// A particle layer (not an effect): one card, its thirty handles grouped by what they shape; the rarely used groups fold away.
/// contract: a group's dependent handles (Floor, Scale, the line pair) show only while the handle that turns them on is non-zero.
class ParticlesCard extends StatelessWidget {
  const ParticlesCard({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    doc.seed({for (final p in _ptAll) ...paramDefaults('pt', p)}, strs: const {'pt_color': '#FFFFFF', 'pt_color_end': '#FFFFFF'});
    // contract: re-provides Ctx on every doc change, so the card's const blocks and pictures follow even under a host that does not rebuild
    return ListenableBuilder(
      listenable: doc,
      builder: (context, _) => Ctx(cfg: x.cfg, doc: doc, child: _body(doc)),
    );
  }

  Widget _body(Doc doc) {
    double g(String id) => doc.get('pt_$id') ?? 0;
    final turb = doc.b['pt_fold.turb'] ?? false, links = doc.b['pt_fold.links'] ?? false;
    return Sect(
      title: 'Particles',
      tone: Pop.toneParticles,
      mark: CardMark.particles,
      keyIds: [for (final p in _ptAll) 'pt_${p.id}'],
      brief: ['${brief(g('rate'), 0)}/s', '${g('life').toStringAsFixed(1)} s', '${brief(g('direction'), 0)}°'],
      children: [
        const _PtPreview(),
        const _PtHead('Emit'),
        const ParamGrid('pt', _ptEmit),
        const _PtHead('Motion'),
        const Padding(
          padding: EdgeInsets.only(top: 4),
          child: Row(
            children: [
              Expanded(child: _PtAngle(_PtAngle.dir)),
              SizedBox(width: Pop.tileGap),
              Expanded(child: _PtAngle(_PtAngle.spread)),
              SizedBox(width: Pop.tileGap),
              Expanded(child: ParamTile('pt', _ptSpeed)),
            ],
          ),
        ),
        const _PtHead('Over life'),
        const _PtRamp(),
        const ParamGrid('pt', _ptLife),
        const SizedBox(height: Pop.tileGap),
        const Row(
          children: [
            Expanded(child: _PtColour('pt_color', 'Color')),
            SizedBox(width: Pop.tileGap),
            Expanded(child: _PtColour('pt_color_end', 'End Color')),
          ],
        ),
        const _PtHead('Forces'),
        ParamGrid('pt', [_ptGravity, _ptWindX, _ptWindY, _ptBounce, if (g('bounce') > 0) _ptFloor]),
        const _PtHead('Random'),
        const ParamGrid('pt', _ptRandom),
        const SizedBox(height: 4),
        _PtHead(
          'Turbulence',
          open: turb,
          summary: g('turbulence') > 0 ? '${brief(g('turbulence'), 0)} px' : 'Off',
          onTap: () => doc.flag('pt_fold.turb', !turb),
        ),
        if (turb) ParamGrid('pt', [_ptTurb, if (g('turbulence') > 0) _ptTurbSize]),
        _PtHead('Links', open: links, summary: g('connect') > 0 ? '${brief(g('connect'), 0)} px' : 'Off', onTap: () => doc.flag('pt_fold.links', !links)),
        if (links) ParamGrid('pt', [_ptConnect, if (g('connect') > 0) ..._ptLines]),
        AdvancedFold('pt.adv', summary: 'Seed ${brief(g('seed'), 0)}', const [
          ParamGrid('pt', [_ptSeed]),
        ]),
      ],
    );
  }
}

/// A group's name inside the card; given [onTap] it is a fold with its state summarised on the right, like Advanced.
class _PtHead extends StatelessWidget {
  const _PtHead(this.title, {this.open, this.summary, this.onTap});
  final String title;
  final bool? open;
  final String? summary;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) {
    final fold = onTap != null;
    return Hov(
      cursor: fold ? SystemMouseCursors.click : SystemMouseCursors.basic,
      onTap: onTap,
      builder: (_, h) => Container(
        height: fold ? 24 : 20,
        padding: EdgeInsets.only(top: fold ? 0 : 6),
        decoration: fold
            ? BoxDecoration(
                border: Border(top: BorderSide(color: Grey.g20)),
              )
            : null,
        child: Row(
          children: [
            if (fold) ...[FoldMark(open: open ?? false, hot: h), const SizedBox(width: 6)],
            Text(title, style: T.micro(fold && (h || (open ?? false)) ? Grey.g95 : Grey.g63).copyWith(fontWeight: FontWeight.w700)),
            const Spacer(),
            if (summary != null) Text(summary!, style: T.label(Grey.g56)),
          ],
        ),
      ),
    );
  }
}

/// Direction or Spread as a number block whose picture is the emission cone in the layer's own frame.
/// contract: 0° points right and -90° up (y down), unlike the generic dial; Direction's cone turns under the pointer, Spread's number scrubs.
class _PtAngle extends StatelessWidget {
  const _PtAngle(this.p);
  static const dir = 0, spread = 1;
  final int p;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, prm = _ptMotion[p], id = 'pt_${prm.id}';
    return _RowInfo(
      unit: null,
      keys: doc.keys[id] ?? KeyS.off,
      child: NumField(
        id: id,
        caption: prm.label,
        unit: '°',
        min: prm.min,
        max: prm.max,
        glyph: p == dir
            ? const _PtTurn()
            : CustomPaint(size: const Size(22, 22), painter: _ConePaint(doc.get('pt_direction') ?? 0, doc.get(id) ?? 0, false, true)),
        glyphOwnsPointer: p == dir,
      ),
    );
  }
}

class _PtTurn extends StatefulWidget {
  const _PtTurn();
  @override
  State<_PtTurn> createState() => _PtTurnState();
}

class _PtTurnState extends State<_PtTurn> {
  double? _last;
  bool _hot = false;
  double _at(Offset p) => math.atan2(p.dy - 11, p.dx - 11) * 180 / math.pi;
  void _end() {
    if (_last != null) Ctx.read(context).doc.endGesture();
    setState(() => _last = null);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked && !RowOff.offOf(context);
    return MouseRegion(
      cursor: on ? SystemMouseCursors.grab : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => _hot = true),
      onExit: (_) => setState(() => _hot = false),
      child: Listener(
        onPointerDown: on
            ? (e) {
                doc.beginGesture();
                setState(() => _last = _at(e.localPosition));
              }
            : null,
        onPointerMove: (e) {
          final last = _last;
          if (last == null) return;
          final a = _at(e.localPosition);
          final d = (a - last + 540) % 360 - 180;
          _last = a;
          doc.set('pt_direction', ((doc.get('pt_direction') ?? 0) + d).roundToDouble());
        },
        onPointerUp: (_) => _end(),
        onPointerCancel: (_) => _end(),
        child: CustomPaint(
          size: const Size(22, 22),
          painter: _ConePaint(doc.get('pt_direction') ?? 0, doc.get('pt_spread') ?? 0, _hot || _last != null, false),
        ),
      ),
    );
  }
}

class _ConePaint extends CustomPainter {
  const _ConePaint(this.dir, this.spread, this.hot, this.wide);
  final double dir, spread;
  final bool hot, wide;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), a = dir * math.pi / 180, sw = spread.clamp(0.0, 360.0) * math.pi / 180;
    c.drawCircle(o, 9, Paint()..color = hot ? Grey.g38 : Grey.g26);
    c.drawArc(Rect.fromCircle(center: o, radius: 8), a - sw / 2, math.max(sw, .03), true, Paint()..color = wide ? Grey.g76 : Grey.g44);
    c.drawLine(
      o,
      o + Offset(math.cos(a), math.sin(a)) * 8,
      Paint()
        ..color = wide ? Grey.g56 : Grey.g95
        ..strokeWidth = wide ? 1.2 : 2
        ..strokeCap = StrokeCap.round,
    );
    c.drawCircle(o, 1.8, Paint()..color = wide ? Grey.g56 : Grey.g95);
  }

  @override
  bool shouldRepaint(_ConePaint o) => o.dir != dir || o.spread != spread || o.hot != hot || o.wide != wide;
}

/// A colour handle as a block: name on top, the floating palette chooser under it, the swatch in the corner.
class _PtColour extends StatelessWidget {
  const _PtColour(this.id, this.label);
  final String id, label;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), on = !x.cfg.locked;
    return Container(
      height: Pop.tile,
      decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
      child: Stack(
        children: [
          Positioned(right: Pop.tilePad - 3, top: 4, width: Pop.glyph, height: Pop.glyph, child: _Swatch(x.doc.s2[id])),
          Padding(
            padding: const EdgeInsets.fromLTRB(Pop.tilePad, 6, Pop.glyph + Pop.tilePad, 3),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                LinkLine(to: LinkTo.colors, value: x.doc.s2[id] ?? '—', id: id),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// The handles the pictures read, packed in one list so a repaint is one compare.
List<double> _ptQ(Doc d) => [
  for (final id in const [
    'rate', 'life', 'life_random', 'emitter.x', 'emitter.y', 'direction', 'spread', 'speed', 'speed_random', 'size', 'size_end', //
    'opacity_end', 'gravity', 'wind.x', 'wind.y', 'bounce', 'floor', 'turbulence', 'turbulence_size', 'seed', 'connect', 'line_width', 'line_opacity',
  ])
    d.get('pt_$id') ?? 0,
];

bool _ptSame(List<double> a, List<double> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

/// A stand-in for the layer's noise(seed, index, channel): -1..1, the same for the same inputs.
double _ptU(int seed, int i, int ch) {
  var h = (seed * 73856093) ^ (i * 19349663) ^ (ch * 83492791) ^ 0x5bd1e995;
  h = ((h ^ (h >> 15)) * 0x2c1b3c6d) & 0xFFFFFFFF;
  h = ((h ^ (h >> 12)) * 0x297a2d39) & 0xFFFFFFFF;
  h ^= h >> 15;
  return (h & 0xFFFF) / 0xFFFF * 2 - 1;
}

/// The layer's ballistic(): constant acceleration, bouncing off y = floor (down is +) only while bounce > 0.
Offset _ptFly(Offset p0, Offset v0, Offset a, double age, double bounce, double floor) {
  Offset at(Offset p, Offset v, double t) => p + v * t + a * (.5 * t * t);
  if (bounce <= 0 || p0.dy > floor) return at(p0, v0, age);
  var p = p0, v = v0, left = age;
  for (var n = 0; n < 32; n++) {
    final qa = .5 * a.dy, qb = v.dy, qc = p.dy - floor;
    var hit = double.infinity;
    if (qa.abs() < 1e-9) {
      if (qb > 1e-9) hit = -qc / qb;
    } else {
      final disc = qb * qb - 4 * qa * qc;
      if (disc >= 0) {
        final r = math.sqrt(disc);
        for (final t in [(-qb - r) / (2 * qa), (-qb + r) / (2 * qa)]) {
          if (t > 1e-6 && t < hit) hit = t;
        }
      }
    }
    if (hit >= left) return at(p, v, left);
    p = Offset(at(p, v, hit).dx, floor);
    v = Offset(v.dx + a.dx * hit, -(v.dy + a.dy * hit) * bounce);
    left -= hit;
    if (v.dy.abs() < 1) return Offset(p.dx + v.dx * left + .5 * a.dx * left * left, floor);
  }
  return p;
}

/// A still of the stream as it would stand once full: dots at evenly spread ages, each with the faint path it came along.
/// contract: drawn from the handles only when they change; no ticker, no per-frame work.
class _PtPreview extends StatelessWidget {
  const _PtPreview();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, q = _ptQ(doc);
    final alive = math.min(q[0] * q[1], 100000);
    return Container(
      height: 76,
      margin: const EdgeInsets.only(bottom: 2),
      decoration: BoxDecoration(color: Grey.g10, borderRadius: BorderRadius.circular(6)),
      child: Stack(
        children: [
          Positioned.fill(
            child: ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: CustomPaint(painter: _SprayPaint(q, _hex(doc.s2['pt_color']) ?? Grey.g95, _hex(doc.s2['pt_color_end']) ?? Grey.g95)),
            ),
          ),
          Positioned(left: 7, top: 6, child: Text('${alive.round()} alive', style: T.label(Grey.g56))),
        ],
      ),
    );
  }
}

class _SprayPaint extends CustomPainter {
  const _SprayPaint(this.q, this.c0, this.c1);
  final List<double> q;
  final Color c0, c1;
  static const n = 36, steps = 8;

  @override
  void paint(Canvas c, Size s) {
    final [_, life0, lifeR, ex, ey, dir, spread, speed, speedR, size0, size1, op1, grav, wx, wy, bounce, floor, turb, turbS, seedV, connect, lw, lop] = q;
    final life = math.max(life0, .01), seed = seedV.round(), acc = Offset(wx, wy + grav);
    final trails = <List<Offset>>[], ages = <double>[];
    final f = 120 / math.max(turbS, 1);
    for (var i = 0; i < n; i++) {
      double u(int ch) => _ptU(seed, i, ch);
      final li = life * (1 - lifeR / 100 * (u(0) * .5 + .5)), age = life * (i + .5) / n;
      if (age >= li) continue;
      final a = (dir + spread * .5 * u(1)) * math.pi / 180, sp = speed * (1 + speedR / 100 * u(2));
      final p0 = Offset(ex * .5 * u(3), ey * .5 * u(4)), v0 = Offset(math.cos(a), math.sin(a)) * sp;
      Offset at(double t) =>
          _ptFly(p0, v0, acc, t, bounce / 100, floor) + Offset(math.sin(i * 1.7 + t * 3 * f), math.cos(i * 2.3 + t * 2.1 * f)) * (turb * t / life);
      trails.add([for (var k = 0; k <= steps; k++) at(age * k / steps)]);
      ages.add(age / li);
    }
    var r = Rect.fromCenter(center: Offset.zero, width: math.max(ex, 1), height: math.max(ey, 1));
    for (final t in trails) {
      for (final p in t) {
        r = r.expandToInclude(Rect.fromCenter(center: p, width: 0, height: 0));
      }
    }
    if (bounce > 0 && floor >= r.top) r = r.expandToInclude(Rect.fromLTWH(r.left, floor, 0, 0));
    r = Rect.fromCenter(center: r.center, width: math.max(r.width, 80), height: math.max(r.height, 40));
    final k = math.min((s.width - 20) / r.width, (s.height - 14) / r.height);
    final o = s.center(Offset.zero) - r.center * k;
    Offset m(Offset p) => o + p * k;
    if (bounce > 0) {
      final y = o.dy + floor * k;
      if (y > 0 && y < s.height) c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = Grey.g26);
    }
    final o0 = m(Offset.zero), d0 = dir * math.pi / 180, sw = spread.clamp(0.0, 360.0) * math.pi / 180;
    c.drawArc(Rect.fromCircle(center: o0, radius: 16), d0 - sw / 2, math.max(sw, .02), true, Paint()..color = Pop.toneParticles.withValues(alpha: .16));
    if (ex > 0 || ey > 0) {
      final er = Rect.fromCenter(center: o0, width: math.max(ex * k, 1), height: math.max(ey * k, 1));
      c.drawRect(
        er,
        Paint()
          ..color = Grey.g44
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
    final dots = <Offset>[];
    final mx = math.max(math.max(size0, size1), 1e-6);
    for (var i = 0; i < trails.length; i++) {
      final t = ages[i], pts = trails[i].map(m).toList(), col = Color.lerp(c0, c1, t)!, alpha = 1 + (op1 / 100 - 1) * t;
      c.drawPath(
        Path()..addPolygon(pts, false),
        Paint()
          ..color = col.withValues(alpha: .14 * alpha)
          ..style = PaintingStyle.stroke
          ..strokeWidth = .8,
      );
      final rad = .7 + 2.3 * (size0 + (size1 - size0) * t) / mx;
      dots.add(pts.last);
      c.drawCircle(pts.last, rad, Paint()..color = col.withValues(alpha: (.25 + .75 * alpha).clamp(0.0, 1.0)));
    }
    if (connect > 0) {
      final line = Paint()..strokeWidth = (lw * .5).clamp(.5, 2.0);
      final dk = connect * k;
      for (var i = 0; i < dots.length; i++) {
        for (var j = i + 1; j < dots.length; j++) {
          final d = (dots[i] - dots[j]).distance;
          if (d < dk) c.drawLine(dots[i], dots[j], line..color = c0.withValues(alpha: lop / 100 * (1 - d / dk) * .8));
        }
      }
    }
    c.drawCircle(o0, 2, Paint()..color = Pop.toneParticles);
  }

  @override
  bool shouldRepaint(_SprayPaint o) => !_ptSame(o.q, q) || o.c0 != c0 || o.c1 != c1;
}

/// Birth to death on one strip: each dot's size, opacity and colour at that share of its life.
class _PtRamp extends StatelessWidget {
  const _PtRamp();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return Padding(
      padding: const EdgeInsets.only(top: 2),
      child: SizedBox(
        height: 18,
        child: Row(
          children: [
            Text('Birth', style: T.label(Grey.g44)),
            const SizedBox(width: 6),
            Expanded(
              child: CustomPaint(
                painter: _RampPaint(
                  doc.get('pt_size') ?? 0,
                  doc.get('pt_size_end') ?? 0,
                  doc.get('pt_opacity_end') ?? 0,
                  _hex(doc.s2['pt_color']) ?? Grey.g95,
                  _hex(doc.s2['pt_color_end']) ?? Grey.g95,
                ),
              ),
            ),
            const SizedBox(width: 6),
            Text('Death', style: T.label(Grey.g44)),
          ],
        ),
      ),
    );
  }
}

Paint get _ptRing => Paint()
  ..color = Grey.g38
  ..style = PaintingStyle.stroke
  ..strokeWidth = .8;

class _RampPaint extends CustomPainter {
  const _RampPaint(this.s0, this.s1, this.op1, this.c0, this.c1);
  final double s0, s1, op1;
  final Color c0, c1;
  @override
  void paint(Canvas c, Size s) {
    const n = 11;
    final mx = math.max(math.max(s0, s1), 1e-6), y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = Grey.g20);
    for (var i = 0; i < n; i++) {
      final t = i / (n - 1), x = 6 + (s.width - 12) * t;
      final a = (1 + (op1 / 100 - 1) * t).clamp(0.0, 1.0);
      final col = Color.lerp(c0, c1, t)!;
      c.drawCircle(Offset(x, y), .8 + 5.2 * (s0 + (s1 - s0) * t) / mx, Paint()..color = col.withValues(alpha: a));
      if (a < .15) c.drawCircle(Offset(x, y), 2.5, _ptRing);
    }
  }

  @override
  bool shouldRepaint(_RampPaint o) => o.s0 != s0 || o.s1 != s1 || o.op1 != op1 || o.c0 != c0 || o.c1 != c1;
}
