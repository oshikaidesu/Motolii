part of 'inspector_parts.dart';

// ---- ideas: placement and anchor (ids pli_) ----------------------------------------------------------------------------
/// Four ways to edit a Repeater and an Anchor where the panel does the work. A, B and C each own a Repeater (pli_a / pli_b / pli_c); D writes the layer's real anchor.
class PlacementIdeas extends StatelessWidget {
  const PlacementIdeas({super.key});
  @override
  Widget build(BuildContext context) => const Column(
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      IdeaLabel(
        'A',
        'Grab the array',
        'Shape the copies by grabbing them: copy 2 sets the step, its ring the turn, its corner the growth, the last copy\'s bar the fade. Pull + for more.',
      ),
      _PliA(),
      IdeaLabel(
        'B',
        'Handles of the shape',
        'Each shape brings its own handles: a spacing rule on a line, radius and arc ends on a circle, columns and gaps on a grid. Scatter shows its jitter as ghosts.',
      ),
      _PliB(),
      IdeaLabel(
        'C',
        'Progression',
        'Every property is a graph from copy 1 to N: drag the line\'s end for Each, the band\'s edge for Random. The whole array reads at a glance.',
      ),
      _PliC(),
      IdeaLabel(
        'D',
        'Free anchor',
        'No box: the layer itself, the pivot dragged anywhere, even outside. It snaps to corners, edges and centre; ghosts show the turn it makes.',
      ),
      _PliD(),
    ],
  );
}

const _pliL = 80.0;
const _pliRep = <String, double>{
  'count': 5, 'mode': 0, 'columns': 3, 'radius': 200, 'start': 0, 'sweep': 360, 'seed': 0, //
  'pos_each.x': 100, 'pos_each.y': 0, 'rot_each': 0, 'scale_each': 0, 'op_each': 0, 'delay_each': 0,
  'pos_rand.x': 0, 'pos_rand.y': 0, 'rot_rand': 0, 'scale_rand': 0, 'op_rand': 0, 'delay_rand': 0,
};
final _pliRotInk = TransformZone.rotate.color, _pliPivot = TransformZone.pivot.color;

void _pliSeed(Doc d, String p, [Map<String, double> over = const {}]) =>
    d.seed({for (final e in _pliRep.entries) '${p}_${e.key}': over[e.key] ?? e.value}, flags: {'$p.open': true});

Object _pliSig(Doc d, String p) => Object.hashAll([for (final k in _pliRep.keys) d.get('${p}_$k')]);

typedef _PliCopy = ({Offset at, Offset clean, double rot, double scale, double op, double delay});

/// The copies as placement.rs places them (2D part); [clean] is the place before Random.
List<_PliCopy> _pliCopies(Doc doc, String p, {int cap = 200}) {
  double g(String k) => doc.get('${p}_$k') ?? 0;
  final n = g('count').round().clamp(1, 1000), mode = g('mode').round(), cols = math.max(1, g('columns').round());
  final e = Offset(g('pos_each.x'), g('pos_each.y')), r = g('radius'), st = g('start'), sw = g('sweep'), seed = g('seed').round();
  return [
    for (var i = 0; i < math.min(n, cap); i++)
      () {
        double u(int ch) => _plNoise(seed, i, ch);
        final clean = switch (mode) {
          1 => () {
            final a = (st + sw * (sw >= 360 ? i / n : i / (math.max(n, 2) - 1))) * math.pi / 180;
            return Offset(r * math.cos(a), r * math.sin(a)) + e * i.toDouble();
          }(),
          2 => Offset(e.dx * (i % cols), e.dy * (i ~/ cols)),
          _ => e * i.toDouble(),
        };
        return (
          clean: clean,
          at: clean + Offset(g('pos_rand.x') * u(0), g('pos_rand.y') * u(1)),
          rot: g('rot_each') * i + g('rot_rand') * u(2),
          scale: math.pow(math.max(0.0, 1 + g('scale_each') / 100), i).toDouble() * math.max(0.0, 1 + g('scale_rand') / 100 * u(3)),
          op: (1 + g('op_each') * i + g('op_rand') * u(4)).clamp(0.0, 1.0).toDouble(),
          delay: g('delay_each') * i + g('delay_rand') * u(5),
        );
      }(),
  ];
}

/// Comp px to stage px.
class _PliView {
  const _PliView(this.o, this.k);
  final Offset o;
  final double k;
  Offset to(Offset p) => o + p * k;
  Offset from(Offset s) => (s - o) / k;
  static _PliView fit(Iterable<Offset> pts, Size s, {double pad = 22, double maxK = .4, double reach = _pliL}) {
    var r = Rect.fromPoints(pts.first, pts.first);
    for (final q in pts) {
      r = r.expandToInclude(Rect.fromPoints(q, q));
    }
    r = r.inflate(reach * .6);
    final k = math.min(maxK, math.min((s.width - 2 * pad) / r.width, (s.height - 2 * pad) / r.height));
    return _PliView(s.center(Offset.zero) - r.center * k, k);
  }
}

Paint _pliLine(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round;

void _pliText(Canvas c, String s, Offset at, {Color? ink, double ax = 0, double ay = 0}) {
  final tp = TextPainter(
    text: TextSpan(text: s, style: T.micro(ink)),
    textDirection: TextDirection.ltr,
  )..layout();
  tp.paint(c, at - Offset(tp.width * ax, tp.height * ay));
}

void _pliGround(Canvas c, Size s) => c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(6)), Paint()..color = Grey.g10);

void _pliDash(Canvas c, Offset a, Offset b, Paint p, [double on = 3, double off = 3]) {
  final d = b - a, n = d.distance;
  if (n < 1) return;
  final u = d / n;
  for (var t = 0.0; t < n; t += on + off) {
    c.drawLine(a + u * t, a + u * math.min(t + on, n), p);
  }
}

/// One copy: a rounded square turned by [rot]°, its tick showing where it faces.
void _pliBody(Canvas c, Offset at, double side, double rot, Paint p, {bool tick = true}) {
  c.save();
  c.translate(at.dx, at.dy);
  c.rotate(rot * math.pi / 180);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: side, height: side), Radius.circular(side * .2)), p);
  if (tick && side > 6) c.drawLine(Offset.zero, Offset(side / 2, 0), _pliLine(Grey.g10));
  c.restore();
}

void _pliKnob(Canvas c, Offset at, Color ink, {bool hot = false, double r = 3.5}) {
  c.drawCircle(at, r + 2, Paint()..color = Grey.g07);
  c.drawCircle(at, hot ? r + 1 : r, Paint()..color = ink);
}

Offset _pliRot(Offset v, double deg) {
  final a = deg * math.pi / 180, cs = math.cos(a), sn = math.sin(a);
  return Offset(v.dx * cs - v.dy * sn, v.dx * sn + v.dy * cs);
}

Offset _pliPolar(double deg) => Offset(math.cos(deg * math.pi / 180), math.sin(deg * math.pi / 180));
double _pliWrap(double a) => (a + math.pi) % (2 * math.pi) - math.pi;
double _pliDeg(double rad) => rad * 180 / math.pi;
String _pliSigned(double v, int dec) => '${v > 0 ? '+' : ''}${fmt(v, dec)}';

class _PliPaint extends CustomPainter {
  const _PliPaint(this.sig, this.draw);
  final Object sig;
  final void Function(Canvas, Size) draw;
  @override
  void paint(Canvas c, Size s) => draw(c, s);
  @override
  bool shouldRepaint(_PliPaint o) => o.sig != sig;
}

/// A small stage of handles. contract: it repaints only on a hover change, a drag or a Doc change; its view [V] is frozen while a drag runs
/// so the grabbed handle stays under the pointer; one drag is one undo; a press that does not move is a tap.
abstract class _PliStage<W extends StatefulWidget, H extends Object, V> extends State<W> {
  H? _hot, _grab;
  V? _held;
  Offset _down = Offset.zero;
  bool _moved = false;
  double get height;
  V fit(Doc d, Size s);
  H? hit(Doc d, V v, Size s, Offset p);
  void begin(Doc d, V v, Size s, H h, Offset p) {}
  void move(Doc d, V v, Size s, H h, Offset p, Offset dl);
  void tap(Doc d, V v, Size s, H h) {}
  Object sigOf(Doc d);
  void draw(Canvas c, Size s, Doc d, V v, H? hot);
  MouseCursor cursorFor(H h) => SystemMouseCursors.grab;

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked && !RowOff.offOf(context);
    return LayoutBuilder(
      builder: (context, box) {
        final s = Size(box.maxWidth, height), v = _held ?? fit(doc, s), h = _grab ?? _hot;
        void done(bool keep) {
          final g = _grab;
          if (g == null) return;
          if (keep && !_moved) tap(doc, v, s, g);
          keep ? doc.endGesture() : doc.cancelGesture();
          setState(() {
            _grab = null;
            _held = null;
          });
        }

        return MouseRegion(
          cursor: h == null ? MouseCursor.defer : (_grab != null ? SystemMouseCursors.grabbing : cursorFor(h)),
          onHover: (e) {
            final n = on ? hit(doc, v, s, e.localPosition) : null;
            if (n != _hot) setState(() => _hot = n);
          },
          onExit: (_) {
            if (_hot != null && _grab == null) setState(() => _hot = null);
          },
          child: Listener(
            onPointerDown: (e) {
              if (!on || e.buttons != kPrimaryButton) return;
              final n = hit(doc, v, s, e.localPosition);
              if (n == null) return;
              _down = e.localPosition;
              _moved = false;
              doc.beginGesture();
              setState(() {
                _grab = _hot = n;
                _held = v;
              });
              begin(doc, v, s, n, e.localPosition);
            },
            onPointerMove: (e) {
              final g = _grab;
              if (g == null || (!_moved && (e.localPosition - _down).distance < 2)) return;
              final dl = _moved ? e.localDelta : e.localPosition - _down;
              _moved = true;
              move(doc, v, s, g, e.localPosition, dl);
            },
            onPointerUp: (_) => done(true),
            onPointerCancel: (_) => done(false),
            child: CustomPaint(size: s, painter: _PliPaint(Object.hash(sigOf(doc), h, _grab, s, _held == null), (c, s) => draw(c, s, doc, v, h))),
          ),
        );
      },
    );
  }
}

/// The card around a concept; folds on its own flag so the three Repeaters never fold together.
class _PliCard extends StatelessWidget {
  const _PliCard({required this.p, required this.title, required this.tone, required this.mark, required this.brief, required this.children});
  final String p, title;
  final Color tone;
  final CardMark mark;
  final List<String> brief;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, open = doc.b['$p.open'] ?? true;
    return Sect(
      title: title,
      tone: tone,
      mark: mark,
      open: open,
      onToggle: () => doc.flag('$p.open', !open),
      brief: brief,
      children: [Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: children)],
    );
  }
}

/// A row of typed values under a stage, for the exact number.
class _PliCells extends StatelessWidget {
  const _PliCells(this.cells, {this.trailing});
  final List<Widget> cells;
  final Widget? trailing;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 4),
    child: Row(
      children: [
        for (final (i, c) in cells.indexed) ...[if (i > 0) const SizedBox(width: 4), Expanded(child: c)],
        if (trailing != null) ...[const SizedBox(width: 4), trailing!],
      ],
    ),
  );
}

Widget _pliSeedFold(Doc doc, String p) => AdvancedFold('$p.adv', summary: 'Seed ${brief(doc.get('${p}_seed') ?? 0, 0)}', [
  _pliNum(doc, '${p}_seed', 'Seed', min: 0, max: 9999, perPx: .2, whole: true),
]);

NumField _pliNum(Doc doc, String id, String label, {String unit = '', int dec = 0, double? min, double? max, double perPx = 1, bool whole = false}) => NumField(
  id: id,
  label: label,
  unit: unit,
  decimals: dec,
  min: min,
  max: max,
  perPx: perPx,
  step: dec > 0 ? math.pow(10, -dec).toDouble() : 1,
  onSet: whole ? (v) => doc.set(id, v.roundToDouble()) : null,
);

// ---- A: grab the array ------------------------------------------------------------------------------------------------
class _PliA extends StatelessWidget {
  const _PliA();
  static String id(String k) => 'pli_a_$k';
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    _pliSeed(doc, 'pli_a');
    double g(String k) => doc.get(id(k)) ?? 0;
    return _PliCard(
      p: 'pli_a',
      title: 'Repeater',
      tone: Pop.tonePlace,
      mark: CardMark.repeater,
      brief: ['${g('count').round()} × Line', 'Each ${brief(g('pos_each.x'), 0)}, ${brief(g('pos_each.y'), 0)} px'],
      children: [
        const _PliAStage(key: ValueKey('pli_a_stage')),
        _PliCells([
          _pliNum(doc, id('count'), 'Count', min: 1, max: 200, perPx: .1, whole: true),
          _pliNum(doc, id('pos_each.x'), 'X', unit: 'px'),
          _pliNum(doc, id('pos_each.y'), 'Y', unit: 'px'),
        ]),
        _PliCells([
          _pliNum(doc, id('rot_each'), 'Turn', unit: '°'),
          _pliNum(doc, id('scale_each'), 'Grow', unit: '%', min: -90, max: 100, perPx: .5),
          _pliNum(doc, id('op_each'), 'Fade', dec: 2, min: -1, max: 0, perPx: .005),
        ]),
      ],
    );
  }
}

enum _PliAH { move, ring, corner, fade, tail }

class _PliAStage extends StatefulWidget {
  const _PliAStage({super.key});
  @override
  State<_PliAStage> createState() => _PliAState();
}

/// Stage-space handles of copy 2, the last copy's fade bar and the + tail.
class _PliAG {
  _PliAG(this.v, this.cs, this.dir, Offset tailAt) : tail = v.to(tailAt) {
    if (cs.length < 2) return;
    final a = cs[1], b = v.to(a.at);
    c1 = b;
    half = _pliL * a.scale * v.k / 2;
    ring = half * math.sqrt2 + 9;
    corner = b + _pliRot(Offset(half, half), a.rot);
    knob = b + _pliRot(Offset(0, -ring), a.rot);
    final l = cs.last, lc = v.to(l.at), side = Offset(-dir.dy, dir.dx);
    final lh = cs.length == 2 ? ring : _pliL * l.scale * v.k / 2 * math.sqrt2;
    fade = Rect.fromCenter(center: lc + side * (lh + 10 + side.dx.abs() * 18), width: 36, height: 6);
    fadeX = fade!.left + fade!.width * l.op;
  }
  final _PliView v;
  final List<_PliCopy> cs;
  final Offset dir, tail;
  Offset? c1;
  double half = 0, ring = 0, fadeX = 0;
  Offset corner = Offset.zero, knob = Offset.zero;
  Rect? fade;
}

class _PliAState extends _PliStage<_PliAStage, _PliAH, _PliView> {
  static String id(String k) => _PliA.id(k);
  Offset _sum = Offset.zero, _e0 = Offset.zero;
  double _r0 = 0, _ang = 0;
  int _n0 = 1;
  @override
  double get height => 132;
  double g(Doc d, String k) => d.get(id(k)) ?? 0;
  Offset _each(Doc d) => Offset(g(d, 'pos_each.x'), g(d, 'pos_each.y'));

  ({List<_PliCopy> cs, Offset dir, Offset tail}) _geo(Doc d) {
    final cs = _pliCopies(d, 'pli_a'), e = _each(d), dir = e.distance < 1 ? const Offset(1, 0) : e / e.distance;
    return (cs: cs, dir: dir, tail: cs.last.at + dir * math.max(e.distance, _pliL * cs.last.scale / 2 + 40));
  }

  _PliAG _at(Doc d, _PliView v) {
    final q = _geo(d);
    return _PliAG(v, q.cs, q.dir, q.tail);
  }

  @override
  Object sigOf(Doc d) => _pliSig(d, 'pli_a');

  @override
  _PliView fit(Doc d, Size s) {
    final q = _geo(d);
    return _PliView.fit([for (final c in q.cs) c.at, q.tail], s, reach: _pliL * q.cs.fold(1.0, (a, c) => math.max(a, c.scale)));
  }

  @override
  _PliAH? hit(Doc d, _PliView v, Size s, Offset p) {
    final q = _at(d, v), c1 = q.c1;
    if ((p - q.tail).distance < 11) return _PliAH.tail;
    if (c1 == null) return null;
    if (q.fade!.inflate(6).contains(p)) return _PliAH.fade;
    if ((p - q.corner).distance < 7) return _PliAH.corner;
    if ((p - q.knob).distance < 7 || ((p - c1).distance - q.ring).abs() < 4) return _PliAH.ring;
    final l = _pliRot(p - c1, -q.cs[1].rot);
    return l.dx.abs() <= q.half + 3 && l.dy.abs() <= q.half + 3 ? _PliAH.move : null;
  }

  @override
  MouseCursor cursorFor(_PliAH h) => h == _PliAH.move ? SystemMouseCursors.move : SystemMouseCursors.grab;

  @override
  void begin(Doc d, _PliView v, Size s, _PliAH h, Offset p) {
    _sum = Offset.zero;
    _e0 = _each(d);
    _r0 = g(d, 'rot_each');
    _n0 = g(d, 'count').round();
    final c1 = _at(d, v).c1;
    if (c1 != null) _ang = (p - c1).direction;
  }

  @override
  void move(Doc d, _PliView v, Size s, _PliAH h, Offset p, Offset dl) {
    _sum += dl;
    final q = _at(d, v);
    switch (h) {
      case _PliAH.move:
        final e = _e0 + _sum / v.k;
        d.setMany({id('pos_each.x'): e.dx.roundToDouble(), id('pos_each.y'): e.dy.roundToDouble()});
      case _PliAH.ring:
        final a = (p - q.c1!).direction;
        _r0 += _pliDeg(_pliWrap(a - _ang));
        _ang = a;
        d.set(id('rot_each'), _r0.roundToDouble());
      case _PliAH.corner:
        final k = (p - q.c1!).distance / (_pliL / 2 * math.sqrt2 * v.k);
        d.set(id('scale_each'), ((k - 1) * 100).clamp(-90, 100).roundToDouble());
      case _PliAH.fade:
        final f = q.fade!, op = ((p.dx - f.left) / f.width).clamp(0.0, 1.0), n = g(d, 'count').round();
        d.set(id('op_each'), double.parse(((op - 1) / math.max(1, n - 1)).toStringAsFixed(3)));
      case _PliAH.tail:
        final step = math.max(_e0.distance * v.k, 14.0), along = _sum.dx * q.dir.dx + _sum.dy * q.dir.dy;
        d.set(id('count'), (_n0 + along / step).round().clamp(1, 200).toDouble());
    }
  }

  @override
  void tap(Doc d, _PliView v, Size s, _PliAH h) {
    if (h == _PliAH.tail) d.set(id('count'), math.min(200, g(d, 'count').round() + 1).toDouble());
  }

  @override
  void draw(Canvas c, Size s, Doc d, _PliView v, _PliAH? hot) {
    _pliGround(c, s);
    final q = _at(d, v), cs = q.cs, n = g(d, 'count').round();
    _pliDash(c, v.to(cs.first.at), q.tail, _pliLine(Grey.g26));
    for (var i = cs.length - 1; i >= 0; i--) {
      final e = cs[i], side = _pliL * e.scale * v.k, at = v.to(e.at);
      _pliBody(c, at, side, e.rot, Paint()..color = (i == 0 ? Pop.tonePlace : Grey.g91).withValues(alpha: math.max(.1, e.op)));
      if (e.op < .2) _pliBody(c, at, side, e.rot, _pliLine(Grey.g44), tick: false);
    }
    final c1 = q.c1;
    if (c1 != null) {
      final a = cs[1];
      _pliBody(c, c1, q.half * 2 + 4, a.rot, _pliLine(hot == _PliAH.move ? Pop.accentInk : Grey.g38, hot == _PliAH.move ? 1.4 : 1), tick: false);
      c.drawCircle(c1, q.ring, _pliLine(hot == _PliAH.ring ? _pliRotInk : Grey.g26));
      c.drawLine(c1, q.knob, _pliLine(_pliRotInk.withValues(alpha: .5)));
      _pliKnob(c, q.knob, _pliRotInk, hot: hot == _PliAH.ring);
      final cr = Rect.fromCenter(center: q.corner, width: hot == _PliAH.corner ? 9 : 7, height: hot == _PliAH.corner ? 9 : 7);
      c.drawRect(cr.inflate(1.5), Paint()..color = Grey.g07);
      c.drawRect(cr, Paint()..color = TransformZone.scale.color);
      final f = q.fade!, fr = RRect.fromRectAndRadius(f, const Radius.circular(3));
      c.drawRRect(fr, Paint()..color = Grey.g20);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(f.left, f.top, q.fadeX, f.bottom), const Radius.circular(3)), Paint()..color = Grey.g63);
      _pliKnob(c, Offset(q.fadeX, f.center.dy), Grey.g95, hot: hot == _PliAH.fade, r: 3);
    }
    final tr = RRect.fromRectAndRadius(Rect.fromCenter(center: q.tail, width: 20, height: 16), const Radius.circular(8));
    c.drawRRect(tr, Paint()..color = hot == _PliAH.tail ? Grey.g26 : Grey.g15);
    c.drawRRect(tr, _pliLine(hot == _PliAH.tail ? Grey.g76 : Grey.g44));
    c.drawLine(q.tail - const Offset(4, 0), q.tail + const Offset(4, 0), _pliLine(Grey.g91, 1.4));
    c.drawLine(q.tail - const Offset(0, 4), q.tail + const Offset(0, 4), _pliLine(Grey.g91, 1.4));
    final say = switch (hot) {
      _PliAH.move => 'Position each  ${fmt(g(d, 'pos_each.x'), 0)}, ${fmt(g(d, 'pos_each.y'), 0)} px',
      _PliAH.ring => 'Rotation each  ${_pliSigned(g(d, 'rot_each'), 0)}°',
      _PliAH.corner => 'Scale each  ${_pliSigned(g(d, 'scale_each'), 0)}%',
      _PliAH.fade => 'Last copy  ${(cs.last.op * 100).round()}% opacity',
      _PliAH.tail => '$n copies · pull for more',
      null => n > 1 ? '$n copies · grab copy 2, its ring, its corner' : 'Pull + for copies',
    };
    _pliText(c, say, const Offset(8, 7), ink: hot == null ? Grey.g56 : Grey.g91);
  }
}

// ---- B: handles of the shape ------------------------------------------------------------------------------------------
const _pliBShapes = ['Line', 'Circle', 'Grid'];

class _PliB extends StatelessWidget {
  const _PliB();
  static String id(String k) => 'pli_b_$k';

  /// placement.rs default_for: a Circle's Position Each is 0; an untouched Each follows the shape.
  static void pick(Doc d, int k) {
    final old = (d.get(id('mode')) ?? 0).round(), nx = k == 1 ? 0.0 : 100.0, ox = old == 1 ? 0.0 : 100.0;
    final untouched = d.get(id('pos_each.x')) == ox && d.get(id('pos_each.y')) == 0;
    d.d[id('pos_each.x')] = nx;
    d.setMany({id('mode'): k.toDouble(), if (untouched) id('pos_each.x'): nx});
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    _pliSeed(doc, 'pli_b', const {'mode': 1, 'count': 8, 'pos_each.x': 0, 'radius': 160, 'start': 150, 'sweep': 240, 'pos_rand.x': 24, 'pos_rand.y': 24});
    double g(String k) => doc.get(id(k)) ?? 0;
    final mode = g('mode').round().clamp(0, 2);
    return _PliCard(
      p: 'pli_b',
      title: 'Repeater',
      tone: Pop.tonePlace,
      mark: CardMark.repeater,
      brief: [
        '${g('count').round()} × ${_pliBShapes[mode]}',
        if (mode == 1) 'R ${brief(g('radius'), 0)} px',
        if (mode == 2) '${g('columns').round()} cols',
        if (g('pos_rand.x') > 0) 'Scatter ${brief(g('pos_rand.x'), 0)} px',
      ],
      children: [
        Dis(
          child: Segmented(items: _pliBShapes, index: mode, expand: true, onChanged: (k) => pick(doc, k)),
        ),
        const SizedBox(height: 4),
        const _PliBStage(key: ValueKey('pli_b_stage')),
        const SizedBox(height: 4),
        const _PliBScatter(key: ValueKey('pli_b_scatter')),
        _PliCells([
          _pliNum(doc, id('count'), 'Count', min: 1, max: 200, perPx: .1, whole: true),
          ...switch (mode) {
            1 => [_pliNum(doc, id('radius'), 'Radius', unit: 'px', min: 0), _pliNum(doc, id('sweep'), 'Sweep', unit: '°', min: 0, max: 360, perPx: 1.8)],
            2 => [_pliNum(doc, id('columns'), 'Cols', min: 1, max: 100, perPx: .1, whole: true), _pliNum(doc, id('pos_each.y'), 'Row', unit: 'px')],
            _ => [_pliNum(doc, id('pos_each.x'), 'X', unit: 'px'), _pliNum(doc, id('pos_each.y'), 'Y', unit: 'px')],
          },
        ]),
        _pliSeedFold(doc, 'pli_b'),
      ],
    );
  }
}

enum _PliBH { spacing, dir, start, sweep, radius, cells, gap }

class _PliBStage extends StatefulWidget {
  const _PliBStage({super.key});
  @override
  State<_PliBStage> createState() => _PliBState();
}

class _PliBState extends _PliStage<_PliBStage, _PliBH, _PliView> {
  static String id(String k) => _PliB.id(k);
  Offset _e0 = Offset.zero;
  double _a0 = 0, _ang = 0;
  @override
  double get height => 140;
  double g(Doc d, String k) => d.get(id(k)) ?? 0;
  int _mode(Doc d) => g(d, 'mode').round().clamp(0, 2);
  Offset _each(Doc d) => Offset(g(d, 'pos_each.x'), g(d, 'pos_each.y'));
  Offset _dir(Offset e) => e.distance < 1 ? const Offset(1, 0) : e / e.distance;
  int _cols(Doc d) => math.max(1, g(d, 'columns').round());
  int _rows(Doc d) => (g(d, 'count').round().clamp(1, 1000) / _cols(d)).ceil();

  @override
  Object sigOf(Doc d) => _pliSig(d, 'pli_b');

  @override
  _PliView fit(Doc d, Size s) {
    final cs = _pliCopies(d, 'pli_b'), e = _each(d), r = g(d, 'radius');
    return _PliView.fit(
      [
        for (final c in cs) c.clean,
        ...switch (_mode(d)) {
          1 => [Offset(-r, -r), Offset(r, r)],
          2 => [Offset(e.dx * (_cols(d) - 1), e.dy * (_rows(d) - 1)), e],
          _ => [cs.last.clean + _dir(e) * 60],
        },
      ],
      s,
      maxK: .35,
    );
  }

  /// Every handle of the current shape, in stage px.
  Map<_PliBH, Offset> _handles(Doc d, _PliView v) {
    final cs = _pliCopies(d, 'pli_b'), e = _each(d);
    switch (_mode(d)) {
      case 1:
        final o = v.to(Offset.zero), r = g(d, 'radius') * v.k, st = g(d, 'start'), sw = g(d, 'sweep');
        return {
          _PliBH.radius: o + _pliPolar(st + (sw >= 360 ? 180 : sw / 2)) * (r + 12),
          _PliBH.sweep: o + _pliPolar(st + sw) * math.max(r - 14, r * .55),
          _PliBH.start: v.to(cs.first.clean),
        };
      case 2:
        final half = _pliL * v.k / 2;
        return {_PliBH.cells: v.to(Offset(e.dx * (_cols(d) - 1), e.dy * (_rows(d) - 1))) + Offset(half + 4, half + 4), _PliBH.gap: v.to(e)};
      default:
        return {_PliBH.dir: v.to(cs.last.clean) + _dir(e) * (_pliL * v.k / 2 + 14), _PliBH.spacing: v.to(cs.length > 1 ? cs[1].clean : e)};
    }
  }

  @override
  _PliBH? hit(Doc d, _PliView v, Size s, Offset p) {
    for (final h in _handles(d, v).entries) {
      if ((p - h.value).distance < 9) return h.key;
    }
    return null;
  }

  @override
  void begin(Doc d, _PliView v, Size s, _PliBH h, Offset p) {
    _e0 = _each(d);
    _a0 = h == _PliBH.sweep ? g(d, 'sweep') : g(d, 'start');
    _ang = (p - v.to(Offset.zero)).direction;
  }

  @override
  void move(Doc d, _PliView v, Size s, _PliBH h, Offset p, Offset dl) {
    final lp = v.from(p);
    void each(Offset e) => d.setMany({id('pos_each.x'): e.dx.roundToDouble(), id('pos_each.y'): e.dy.roundToDouble()});
    double turn() {
      final a = (p - v.to(Offset.zero)).direction;
      _a0 += _pliDeg(_pliWrap(a - _ang));
      _ang = a;
      return _a0;
    }

    switch (h) {
      case _PliBH.spacing:
        final u = _dir(_e0);
        each(u * (lp.dx * u.dx + lp.dy * u.dy));
      case _PliBH.dir:
        var a = _pliDeg(lp.direction);
        final near = (a / 15).roundToDouble() * 15;
        if ((a - near).abs() < 4) a = near;
        each(_pliPolar(a) * (_e0.distance < 1 ? 100 : _e0.distance));
      case _PliBH.start:
        d.set(id('start'), turn().roundToDouble());
      case _PliBH.sweep:
        var w = turn().clamp(0.0, 360.0);
        for (final t in const [90.0, 180.0, 360.0]) {
          if ((w - t).abs() < 5) w = t;
        }
        d.set(id('sweep'), w.roundToDouble());
      case _PliBH.radius:
        d.set(id('radius'), lp.distance.roundToDouble());
      case _PliBH.gap:
        each(lp);
      case _PliBH.cells:
        int k(double at, double step, int now) => step.abs() < 1 ? now : (at / step).round().clamp(0, 11) + 1;
        final nc = k(lp.dx, _e0.dx, _cols(d)), nr = k(lp.dy, _e0.dy, _rows(d));
        d.setMany({id('columns'): nc.toDouble(), id('count'): (nc * nr).toDouble()});
    }
  }

  @override
  void draw(Canvas c, Size s, Doc d, _PliView v, _PliBH? hot) {
    _pliGround(c, s);
    final cs = _pliCopies(d, 'pli_b'), hs = _handles(d, v), mode = _mode(d), e = _each(d), side = _pliL * v.k;
    final o = v.to(Offset.zero), scatter = g(d, 'pos_rand.x') > 0 || g(d, 'pos_rand.y') > 0;
    var say = switch (mode) {
      1 => 'R ${fmt(g(d, 'radius'), 0)} px · from ${fmt(g(d, 'start'), 0)}° over ${fmt(g(d, 'sweep'), 0)}°',
      2 => '${_cols(d)} × ${_rows(d)} · gap ${fmt(e.dx, 0)}, ${fmt(e.dy, 0)} px',
      _ => '${fmt(e.distance, 0)} px apart at ${fmt(_pliDeg(e.direction), 0)}°',
    };
    if (mode == 1) {
      final r = g(d, 'radius') * v.k, st = g(d, 'start'), sw = g(d, 'sweep'), rect = Rect.fromCircle(center: o, radius: r);
      c.drawCircle(o, r, _pliLine(Grey.g20));
      c.drawArc(rect, st * math.pi / 180, sw * math.pi / 180, false, _pliLine(hot == _PliBH.start ? Pop.tonePlace : Grey.g44, 1.4));
      final ir = math.max(r - 14, r * .55);
      c.drawArc(Rect.fromCircle(center: o, radius: ir), st * math.pi / 180, sw * math.pi / 180, false, _pliLine(hot == _PliBH.sweep ? Grey.g91 : Grey.g26));
      c.drawLine(o - const Offset(4, 0), o + const Offset(4, 0), _pliLine(Grey.g38));
      c.drawLine(o - const Offset(0, 4), o + const Offset(0, 4), _pliLine(Grey.g38));
      final ra = hs[_PliBH.radius]!, u = (ra - o) / math.max(1, (ra - o).distance);
      c.drawLine(ra - u * 12, ra, _pliLine(hot == _PliBH.radius ? Grey.g91 : Grey.g44));
    } else if (mode == 2) {
      final far = v.to(Offset(e.dx * (_cols(d) - 1), e.dy * (_rows(d) - 1)));
      final box = Rect.fromPoints(o, far).inflate(side / 2 + 4);
      for (final (a, b) in [(box.topLeft, box.topRight), (box.topRight, box.bottomRight), (box.bottomRight, box.bottomLeft), (box.bottomLeft, box.topLeft)]) {
        _pliDash(c, a, b, _pliLine(hot == _PliBH.cells ? Grey.g63 : Grey.g26));
      }
      if (_cols(d) < 2 || _rows(d) < 2) _pliBody(c, hs[_PliBH.gap]!, side, 0, _pliLine(Grey.g38), tick: false);
    } else if (cs.length > 1) {
      final a = v.to(cs[0].clean), b = hs[_PliBH.spacing]!, u = _dir(e), nrm = Offset(-u.dy, u.dx) * (side / 2 + 8);
      c.drawLine(a + nrm, b + nrm, _pliLine(hot == _PliBH.spacing ? Grey.g91 : Grey.g44));
      for (final q in [a, b]) {
        c.drawLine(q + nrm * .7, q + nrm * 1.3, _pliLine(Grey.g44));
      }
      _pliDash(c, b, hs[_PliBH.dir]!, _pliLine(Grey.g26));
    }
    for (var i = cs.length - 1; i >= 0; i--) {
      final q = cs[i], at = v.to(q.at);
      if (scatter) {
        final cl = v.to(q.clean);
        _pliBody(c, cl, side, q.rot, _pliLine(Grey.g26), tick: false);
        c.drawLine(cl, at, _pliLine(Grey.g38));
      }
      _pliBody(c, at, side, q.rot, Paint()..color = (i == 0 ? Pop.tonePlace : Grey.g91).withValues(alpha: math.max(.1, q.op)));
    }
    for (final h in hs.entries) {
      final on = hot == h.key;
      if (h.key == _PliBH.start) {
        c.drawCircle(h.value, side / 2 + 4, _pliLine(on ? Pop.tonePlace : Grey.g44, on ? 1.6 : 1));
      } else if (h.key == _PliBH.cells) {
        final p = h.value, l = on ? 8.0 : 6.0;
        c.drawPath(
          Path()
            ..moveTo(p.dx - l, p.dy)
            ..lineTo(p.dx, p.dy)
            ..lineTo(p.dx, p.dy - l),
          _pliLine(on ? Grey.g95 : Grey.g76, 2),
        );
        _pliText(c, '${_cols(d)} × ${_rows(d)}', p + const Offset(4, 2), ink: on ? Grey.g95 : Grey.g63, ay: 0);
      } else {
        _pliKnob(c, h.value, h.key == _PliBH.radius || h.key == _PliBH.dir ? Grey.g76 : Grey.g95, hot: on);
      }
    }
    if (hot != null) {
      say = switch (hot) {
        _PliBH.spacing => 'Spacing  ${fmt(e.distance, 0)} px',
        _PliBH.dir => 'Direction  ${fmt(_pliDeg(e.direction), 0)}°',
        _PliBH.start => 'Start  ${fmt(g(d, 'start'), 0)}° · turn the first copy',
        _PliBH.sweep => 'Sweep  ${fmt(g(d, 'sweep'), 0)}°',
        _PliBH.radius => 'Radius  ${fmt(g(d, 'radius'), 0)} px',
        _PliBH.cells => '${_cols(d)} columns × ${_rows(d)} rows',
        _PliBH.gap => 'Gap  ${fmt(e.dx, 0)}, ${fmt(e.dy, 0)} px',
      };
    }
    _pliText(c, say, const Offset(8, 7), ink: hot == null ? Grey.g56 : Grey.g91);
  }
}

enum _PliBS { amount, seed }

/// Scatter as one amount: Position Random on both axes, its jitter drawn in the track; the die rolls the seed.
class _PliBScatter extends StatefulWidget {
  const _PliBScatter({super.key});
  @override
  State<_PliBScatter> createState() => _PliBScatterState();
}

class _PliBScatterState extends _PliStage<_PliBScatter, _PliBS, Size> {
  static const max = 200.0, x0 = 58.0;
  static String id(String k) => _PliB.id(k);
  @override
  double get height => Pop.cell;
  double _x1(Size s) => s.width - 64;
  @override
  Size fit(Doc d, Size s) => s;
  @override
  Object sigOf(Doc d) => Object.hash(d.get(id('pos_rand.x')), d.get(id('seed')));
  @override
  _PliBS? hit(Doc d, Size v, Size s, Offset p) => p.dx > s.width - 26 ? _PliBS.seed : _PliBS.amount;
  @override
  MouseCursor cursorFor(_PliBS h) => h == _PliBS.seed ? SystemMouseCursors.click : SystemMouseCursors.resizeLeftRight;
  void _set(Doc d, Size s, Offset p) {
    final a = (((p.dx - x0) / (_x1(s) - x0)).clamp(0.0, 1.0) * max).roundToDouble();
    d.setMany({id('pos_rand.x'): a, id('pos_rand.y'): a});
  }

  @override
  void begin(Doc d, Size v, Size s, _PliBS h, Offset p) {
    if (h == _PliBS.amount) _set(d, s, p);
  }

  @override
  void move(Doc d, Size v, Size s, _PliBS h, Offset p, Offset dl) {
    if (h == _PliBS.amount) _set(d, s, p);
  }

  @override
  void tap(Doc d, Size v, Size s, _PliBS h) {
    if (h == _PliBS.seed) d.set(id('seed'), ((d.get(id('seed')) ?? 0) + 1) % 10000);
  }

  @override
  void draw(Canvas c, Size s, Doc d, Size v, _PliBS? hot) {
    final a = (d.get(id('pos_rand.x')) ?? 0).clamp(0.0, max), seed = (d.get(id('seed')) ?? 0).round(), x1 = _x1(s), cy = s.height / 2;
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(6)), Paint()..color = hot == _PliBS.amount ? Grey.g20 : Pop.well);
    _pliText(c, 'Scatter', Offset(8, cy), ink: Grey.g63, ay: .5);
    final t = x0 + (x1 - x0) * a / max;
    c.drawLine(Offset(x0, cy), Offset(x1, cy), _pliLine(Grey.g26));
    for (var i = 0; i < 24; i++) {
      final x = x0 + (x1 - x0) * (i + .5) / 24;
      c.drawCircle(Offset(x, cy + _plNoise(seed, i, 1) * 8 * a / max), 1.4, Paint()..color = x <= t ? Grey.g91 : Grey.g38);
    }
    _pliKnob(c, Offset(t, cy), Grey.g95, hot: hot == _PliBS.amount, r: 3);
    _pliText(c, '${fmt(a.toDouble(), 0)} px', Offset(x1 + 8, cy), ink: Grey.g91, ay: .5);
    final die = Rect.fromCenter(center: Offset(s.width - 13, cy), width: 13, height: 13);
    c.drawRRect(RRect.fromRectAndRadius(die, const Radius.circular(3)), _pliLine(hot == _PliBS.seed ? Grey.g95 : Grey.g63, 1.2));
    for (final q in const [Offset(-3, -3), Offset(0, 0), Offset(3, 3)]) {
      c.drawCircle(die.center + q, 1, Paint()..color = hot == _PliBS.seed ? Grey.g95 : Grey.g63);
    }
  }
}

// ---- C: progression ---------------------------------------------------------------------------------------------------
/// A property across the copies. kind 0 adds Each per copy, 1 multiplies (Scale, shown as %), 2 adds and clamps to 0..1 (Opacity).
class _PliCRow {
  const _PliCRow(this.label, this.each, this.rand, this.ch, {this.unit = '', this.kind = 0, this.span = 100, this.dec = 0});
  final String label, each, rand, unit;
  final int ch, kind, dec;
  final double span;
  double base() => switch (kind) {
    1 => 100,
    2 => 1,
    _ => 0,
  };
  double at(double e, int i) => switch (kind) {
    1 => 100 * math.pow(math.max(0.0, 1 + e / 100), i).toDouble(),
    2 => (1 + e * i).clamp(0.0, 1.0).toDouble(),
    _ => e * i,
  };
  double band(double r, double v) => kind == 1 ? v * r / 100 : r;
  double real(double e, double r, int i, double u) => switch (kind) {
    1 => at(e, i) * math.max(0.0, 1 + r / 100 * u),
    2 => (1 + e * i + r * u).clamp(0.0, 1.0).toDouble(),
    _ => at(e, i) + r * u,
  };

  /// The Each that puts copy [n] at [v].
  double eachFor(double v, int n) => switch (kind) {
    1 => ((math.pow(math.max(v, 0.0) / 100, 1 / (n - 1)) - 1) * 100).clamp(-100.0, 100.0).toDouble(),
    2 => ((v.clamp(0.0, 1.0) - 1) / (n - 1)).clamp(-1.0, 1.0).toDouble(),
    _ => v / (n - 1),
  };
}

const _pliCRows = [
  _PliCRow('Position X', 'pos_each.x', 'pos_rand.x', 0, unit: 'px', span: 200),
  _PliCRow('Position Y', 'pos_each.y', 'pos_rand.y', 1, unit: 'px', span: 200),
  _PliCRow('Rotation', 'rot_each', 'rot_rand', 2, unit: '°', span: 90),
  _PliCRow('Scale', 'scale_each', 'scale_rand', 3, unit: '%', kind: 1, span: 60, dec: 1),
  _PliCRow('Opacity', 'op_each', 'op_rand', 4, kind: 2, dec: 3),
  _PliCRow('Delay', 'delay_each', 'delay_rand', 5, unit: 's', span: 1, dec: 2),
];

class _PliC extends StatelessWidget {
  const _PliC();
  static String id(String k) => 'pli_c_$k';
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    _pliSeed(doc, 'pli_c', const {'count': 8, 'pos_each.x': 60, 'rot_each': 15, 'scale_each': -6, 'rot_rand': 10});
    final n = (doc.get(id('count')) ?? 1).round().clamp(1, 1000), cs = _pliCopies(doc, 'pli_c');
    final moved = [
      for (final r in _pliCRows)
        if ((doc.get(id(r.each)) ?? 0) != 0 || (doc.get(id(r.rand)) ?? 0) != 0) r.label,
    ];
    return _PliCard(
      p: 'pli_c',
      title: 'Repeater',
      tone: Pop.tonePlace,
      mark: CardMark.repeater,
      brief: ['$n copies', if (moved.isNotEmpty) moved.join(' · ')],
      children: [
        _PlStrip(height: 48, painter: _RepPaint([for (final c in cs) (x: c.at.dx, y: c.at.dy, rot: c.rot, scale: c.scale, op: c.op)], n)),
        _PliCells([_pliNum(doc, id('count'), 'Count', min: 1, max: 200, perPx: .1, whole: true)]),
        Padding(
          padding: const EdgeInsets.only(top: 6, bottom: 2),
          child: Row(
            children: [
              const SizedBox(width: 82),
              Text('Copy 1', style: T.micro(Grey.g56)),
              const Spacer(),
              Text('Copy $n', style: T.micro(Grey.g56)),
            ],
          ),
        ),
        for (final r in _pliCRows) _PliCLine(r, key: ValueKey('pli_c_${r.each}')),
        _pliSeedFold(doc, 'pli_c'),
      ],
    );
  }
}

class _PliCLine extends StatelessWidget {
  const _PliCLine(this.r, {super.key});
  final _PliCRow r;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, e = doc.get(_PliC.id(r.each)) ?? 0, rr = doc.get(_PliC.id(r.rand)) ?? 0;
    final each = switch (r.kind) {
      1 => '×${fmt(1 + e / 100, 2)}',
      _ => '${_pliSigned(e, r.dec)}${r.unit}',
    };
    return Padding(
      padding: const EdgeInsets.only(top: 2),
      child: Row(
        children: [
          SizedBox(
            width: 82,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(r.label, maxLines: 1, style: T.micro(e != 0 || rr != 0 ? Grey.g91 : Grey.g63)),
                const SizedBox(height: 4),
                Text(rr == 0 ? each : '$each ±${fmt(rr, r.dec)}', maxLines: 1, overflow: TextOverflow.clip, softWrap: false, style: T.label(Grey.g56)),
              ],
            ),
          ),
          Expanded(child: _PliCGraph(r)),
        ],
      ),
    );
  }
}

enum _PliCH { end, band }

typedef _PliCRange = ({double lo, double hi});

class _PliCGraph extends StatefulWidget {
  const _PliCGraph(this.r);
  final _PliCRow r;
  @override
  State<_PliCGraph> createState() => _PliCGraphState();
}

class _PliCGraphState extends _PliStage<_PliCGraph, _PliCH, _PliCRange> {
  static const pad = 10.0;
  double _v0 = 0, _b0 = 0, _vm = 0, _acc = 0;
  _PliCRow get r => widget.r;
  @override
  double get height => 34;
  double _e(Doc d) => d.get(_PliC.id(r.each)) ?? 0;
  double _rand(Doc d) => d.get(_PliC.id(r.rand)) ?? 0;
  int _n(Doc d) => (d.get(_PliC.id('count')) ?? 1).round().clamp(1, 1000);
  double _x(Size s, int i, int n) => pad + (n > 1 ? i / (n - 1) : 0) * (s.width - 2 * pad);
  double _y(Size s, _PliCRange v, double val) => s.height - 4 - (val - v.lo) / (v.hi - v.lo) * (s.height - 8);
  double _mid(int n) => (n - 1) / 2;

  /// The band grip sits on the band's upper edge at the middle copy.
  Offset _grip(Doc d, Size s, _PliCRange v) {
    final n = _n(d), m = _mid(n), val = r.at(_e(d), m.round());
    return Offset(pad + .5 * (s.width - 2 * pad), _y(s, v, (val + r.band(_rand(d), val)).clamp(v.lo, v.hi)));
  }

  @override
  Object sigOf(Doc d) => Object.hash(_e(d), _rand(d), _n(d), d.get(_PliC.id('seed')));

  @override
  _PliCRange fit(Doc d, Size s) {
    if (r.kind == 2) return (lo: -.05, hi: 1.05);
    final n = _n(d), e = _e(d), rr = _rand(d);
    var lo = r.base(), hi = r.base();
    for (var i = 0; i < math.min(n, 200); i++) {
      final v = r.at(e, i), b = r.band(rr, v);
      lo = math.min(lo, v - b);
      hi = math.max(hi, v + b);
    }
    if (r.kind == 1) lo = math.max(0, lo);
    final grow = math.max(0.0, r.span - (hi - lo)) / 2, pad = (hi - lo + 2 * grow) * .12;
    return (lo: lo - grow - pad, hi: hi + grow + pad);
  }

  @override
  _PliCH? hit(Doc d, _PliCRange v, Size s, Offset p) {
    if ((p - _grip(d, s, v)).distance < 7) return _PliCH.band;
    return _n(d) > 1 ? _PliCH.end : null;
  }

  @override
  MouseCursor cursorFor(_PliCH h) => SystemMouseCursors.resizeUpDown;

  @override
  void begin(Doc d, _PliCRange v, Size s, _PliCH h, Offset p) {
    final n = _n(d);
    _v0 = r.at(_e(d), n - 1);
    _b0 = _rand(d);
    _vm = r.at(_e(d), _mid(n).round());
    _acc = 0;
  }

  @override
  void move(Doc d, _PliCRange v, Size s, _PliCH h, Offset p, Offset dl) {
    _acc += -dl.dy / (s.height - 8) * (v.hi - v.lo);
    double tidy(double x, int dec) => double.parse(x.toStringAsFixed(dec));
    if (h == _PliCH.end) {
      d.set(_PliC.id(r.each), tidy(r.eachFor(_v0 + _acc, _n(d)), r.dec));
    } else {
      final b = switch (r.kind) {
        1 => (_b0 + _acc / math.max(_vm, 1) * 100).clamp(0.0, 100.0),
        2 => (_b0 + _acc).clamp(0.0, 1.0),
        _ => math.max(0.0, _b0 + _acc),
      };
      d.set(_PliC.id(r.rand), tidy(b.toDouble(), r.dec));
    }
  }

  @override
  void draw(Canvas c, Size s, Doc d, _PliCRange v, _PliCH? hot) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(4)), Paint()..color = hot != null ? Grey.g13 : Grey.g10);
    final n = _n(d), e = _e(d), rr = _rand(d), seed = (d.get(_PliC.id('seed')) ?? 0).round(), m = math.min(n, 200);
    double y(double val) => _y(s, v, val.clamp(v.lo, v.hi));
    _pliDash(c, Offset(pad, y(r.base())), Offset(s.width - pad, y(r.base())), _pliLine(Grey.g26));
    final up = Path(), line = Path();
    final lo = <Offset>[];
    for (var i = 0; i < m; i++) {
      final x = _x(s, i, m), val = r.at(e, i), b = r.band(rr, val);
      final hi = r.kind == 2 ? (val + b).clamp(0.0, 1.0) : val + b, low = r.kind == 2 ? (val - b).clamp(0.0, 1.0) : val - b;
      i == 0 ? up.moveTo(x, y(hi.toDouble())) : up.lineTo(x, y(hi.toDouble()));
      i == 0 ? line.moveTo(x, y(val)) : line.lineTo(x, y(val));
      lo.add(Offset(x, y(low.toDouble())));
    }
    if (rr > 0 && m > 1) {
      for (final q in lo.reversed) {
        up.lineTo(q.dx, q.dy);
      }
      up.close();
      c.drawPath(up, Paint()..color = Pop.tonePlace.withValues(alpha: hot == _PliCH.band ? .3 : .18));
    }
    c.drawPath(line, _pliLine(hot == _PliCH.end ? Grey.g100 : Grey.g91, 1.4));
    final step = math.max(1, (m / 48).ceil());
    for (var i = 0; i < m; i += step) {
      c.drawCircle(Offset(_x(s, i, m), y(r.real(e, rr, i, _plNoise(seed, i, r.ch)))), 1.6, Paint()..color = rr > 0 ? Grey.g76 : Grey.g56);
    }
    if (n > 1) _pliKnob(c, Offset(_x(s, m - 1, m), y(r.at(e, m - 1))), Grey.g95, hot: hot == _PliCH.end, r: 3);
    final gp = _grip(d, s, v), gw = hot == _PliCH.band ? 14.0 : 10.0;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: gp, width: gw + 3, height: 6), const Radius.circular(3)), Paint()..color = Grey.g07);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: gp, width: gw, height: 3), const Radius.circular(1.5)), Paint()..color = Pop.tonePlace);
  }
}

// ---- D: free anchor ---------------------------------------------------------------------------------------------------
class _PliD extends StatelessWidget {
  const _PliD();
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    doc.seed(const {}, flags: {'pli_d.open': true});
    final ax = doc.get('anchor.x'), ay = doc.get('anchor.y'), az = doc.get('anchor.z');
    final centre = ax == 50 && ay == 50 && az == 0;
    return _PliCard(
      p: 'pli_d',
      title: 'Anchor',
      tone: _pliPivot,
      mark: CardMark.anchor,
      brief: [_PliDState.name(ax, ay, az)],
      children: [
        const _PliDStage(key: ValueKey('pli_d_stage')),
        _PliCells(
          [
            _pliNum(doc, 'anchor.x', 'X', unit: '%'),
            _pliNum(doc, 'anchor.y', 'Y', unit: '%'),
            _pliNum(doc, 'anchor.z', 'Z', unit: '%', min: -50, max: 50, perPx: .5),
          ],
          trailing: Hov(
            onTap: centre || x.cfg.locked
                ? null
                : () {
                    doc.beginGesture();
                    doc.setMany({'anchor.x': 50, 'anchor.y': 50, 'anchor.z': 0});
                    doc.endGesture();
                  },
            builder: (_, h) => Container(
              height: Pop.cell,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              alignment: Alignment.center,
              decoration: BoxDecoration(color: h && !centre ? Grey.g26 : Pop.well, borderRadius: BorderRadius.circular(6)),
              child: Text('Centre', style: T.label(centre ? Grey.g56 : (h ? Grey.g95 : Grey.g76))),
            ),
          ),
        ),
      ],
    );
  }
}

enum _PliDH { pivot, rail }

class _PliDStage extends StatefulWidget {
  const _PliDStage({super.key});
  @override
  State<_PliDStage> createState() => _PliDState();
}

/// The layer's own silhouette, no frame round it. contract: a press anywhere puts the pivot there (outside the layer too); it snaps to the
/// 0 / 50 / 100 lines within [snap] px unless Alt is held; Shift-drag or the rail moves it in depth; ghosts turn the layer about it.
class _PliDState extends _PliStage<_PliDStage, _PliDH, Rect> {
  static const snap = 7.0, swing = 14.0, rail = 34.0, top = 26.0, bottom = 114.0;
  static const _gem = [Offset(.2, 0), Offset(.8, 0), Offset(1, .34), Offset(.5, 1), Offset(0, .34)];

  static String name(double? x, double? y, double? z) {
    if (x == null || y == null || z == null) return 'Mixed';
    final inside = x >= 0 && x <= 100 && y >= 0 && y <= 100;
    final n = inside ? anchorName(x, y, z) : 'Custom';
    return n == 'Custom' ? '${fmt(x, 0)}%, ${fmt(y, 0)}%${z != 0 ? ' · Z ${fmt(z, 0)}' : ''}' : n;
  }

  @override
  double get height => 150;
  @override
  Rect fit(Doc d, Size s) => Rect.fromCenter(center: Offset((s.width - rail) / 2, 78), width: 132, height: 84);
  @override
  Object sigOf(Doc d) => Object.hash(d.get('anchor.x'), d.get('anchor.y'), d.get('anchor.z'));
  @override
  _PliDH? hit(Doc d, Rect b, Size s, Offset p) => p.dx > s.width - rail ? _PliDH.rail : _PliDH.pivot;
  @override
  MouseCursor cursorFor(_PliDH h) => h == _PliDH.rail ? SystemMouseCursors.resizeUpDown : SystemMouseCursors.precise;

  double _railY(double z) => top + (50 - z) / 100 * (bottom - top);

  void _place(Doc d, Rect b, Size s, Offset p) {
    final q = Offset(p.dx.clamp(6.0, s.width - rail - 6), p.dy.clamp(6.0, s.height - 6));
    final free = HardwareKeyboard.instance.isAltPressed;
    double one(double at, double lo, double len) {
      final pct = (at - lo) / len * 100;
      if (!free) {
        for (final t in const [0.0, 50.0, 100.0]) {
          if ((pct - t).abs() * len / 100 < snap) return t;
        }
      }
      return pct.roundToDouble();
    }

    d.setMany({'anchor.x': one(q.dx, b.left, b.width), 'anchor.y': one(q.dy, b.top, b.height)});
  }

  void _depth(Doc d, double z) {
    var v = z.clamp(-50.0, 50.0);
    for (final t in const [-50.0, 0.0, 50.0]) {
      if ((v - t).abs() < 5) v = t;
    }
    d.set('anchor.z', v.roundToDouble());
  }

  double _z = 0;

  @override
  void begin(Doc d, Rect b, Size s, _PliDH h, Offset p) {
    _z = d.get('anchor.z') ?? 0;
    if (h == _PliDH.rail) {
      _depth(d, 50 - (p.dy - top) / (bottom - top) * 100);
    } else if (!HardwareKeyboard.instance.isShiftPressed) {
      _place(d, b, s, p);
    }
  }

  @override
  void move(Doc d, Rect b, Size s, _PliDH h, Offset p, Offset dl) {
    if (h == _PliDH.rail) return _depth(d, 50 - (p.dy - top) / (bottom - top) * 100);
    if (HardwareKeyboard.instance.isShiftPressed) {
      _z -= dl.dy * .6;
      return _depth(d, _z);
    }
    _place(d, b, s, p);
  }

  @override
  void draw(Canvas c, Size s, Doc d, Rect b, _PliDH? hot) {
    final ax = d.get('anchor.x') ?? 50, ay = d.get('anchor.y') ?? 50, az = d.get('anchor.z') ?? 0;
    final plane = Offset(b.left + b.width * ax / 100, b.top + b.height * ay / 100);
    final gem = Path()..addPolygon([for (final q in _gem) Offset(b.left + q.dx * b.width, b.top + q.dy * b.height)], true);
    for (final (a, ink) in [(-swing, Grey.g26), (swing, Grey.g38)]) {
      c.save();
      c.translate(plane.dx, plane.dy);
      c.rotate(a * math.pi / 180);
      c.translate(-plane.dx, -plane.dy);
      c.drawPath(gem, _pliLine(ink));
      c.restore();
    }
    final far = [b.topLeft, b.topRight, b.bottomLeft, b.bottomRight].reduce((m, q) => (q - plane).distance > (m - plane).distance ? q : m);
    final rad = (far - plane).distance, a0 = (far - plane).direction;
    if (rad > 4) {
      final arc = Rect.fromCircle(center: plane, radius: rad), w = swing * math.pi / 180;
      c.drawArc(arc, a0 - w, 2 * w, false, _pliLine(_pliPivot.withValues(alpha: .45)));
      final tip = plane + Offset(math.cos(a0 + w), math.sin(a0 + w)) * rad, tan = Offset(-math.sin(a0 + w), math.cos(a0 + w));
      c.drawLine(tip, tip - tan * 5 + (tip - plane) / rad * 3, _pliLine(_pliPivot.withValues(alpha: .45)));
      c.drawLine(tip, tip - tan * 5 - (tip - plane) / rad * 3, _pliLine(_pliPivot.withValues(alpha: .45)));
    }
    c.drawPath(
      gem,
      Paint()
        ..shader = LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [const Color(0xFF3D6BFF).withValues(alpha: .34), const Color(0xFF8FB0FF).withValues(alpha: .16)],
        ).createShader(b),
    );
    c.drawPath(gem, _pliLine(const Color(0xFF8FB0FF).withValues(alpha: .7)));
    for (final (q, dx, dy) in [(b.topLeft, 1.0, 1.0), (b.topRight, -1.0, 1.0), (b.bottomLeft, 1.0, -1.0), (b.bottomRight, -1.0, -1.0)]) {
      c.drawLine(q, q + Offset(7 * dx, 0), _pliLine(Grey.g38));
      c.drawLine(q, q + Offset(0, 7 * dy), _pliLine(Grey.g38));
    }
    for (final y in const [0.0, 50.0, 100.0]) {
      for (final x in const [0.0, 50.0, 100.0]) {
        c.drawCircle(Offset(b.left + b.width * x / 100, b.top + b.height * y / 100), 1.6, Paint()..color = Grey.g56);
      }
    }
    final guide = _pliLine(_pliPivot.withValues(alpha: .55));
    if (const [0.0, 50.0, 100.0].contains(ax)) _pliDash(c, Offset(plane.dx, b.top - 12), Offset(plane.dx, b.bottom + 12), guide, 2, 3);
    if (const [0.0, 50.0, 100.0].contains(ay)) _pliDash(c, Offset(b.left - 12, plane.dy), Offset(b.right + 12, plane.dy), guide, 2, 3);
    final at = plane + const Offset(.7, -.7) * az * .3, ink = hot == _PliDH.pivot || _grab != null ? _pliPivot : Grey.g95;
    if (az != 0) {
      _pliDash(c, plane, at, _pliLine(_pliPivot.withValues(alpha: .7)), 2, 2);
      c.drawCircle(plane, 2.5, _pliLine(_pliPivot.withValues(alpha: .7)));
    }
    c.drawCircle(at, 7, Paint()..color = Grey.g07);
    c.drawCircle(at, 7, _pliLine(ink, 1.6));
    c.drawCircle(at, 2, Paint()..color = ink);
    final rx = s.width - rail / 2, railInk = hot == _PliDH.rail ? Grey.g76 : Grey.g38;
    c.drawLine(Offset(rx, top), Offset(rx, bottom), _pliLine(railInk));
    for (final z in const [-50.0, 0.0, 50.0]) {
      c.drawLine(Offset(rx - 3, _railY(z)), Offset(rx + 3, _railY(z)), _pliLine(railInk));
    }
    _pliText(c, 'Back', Offset(rx, top - 5), ink: Grey.g56, ax: .5, ay: 1);
    _pliText(c, 'Front', Offset(rx, bottom + 5), ink: Grey.g56, ax: .5);
    _pliKnob(c, Offset(rx, _railY(az)), hot == _PliDH.rail ? _pliPivot : Grey.g95, hot: hot == _PliDH.rail);
    _pliText(c, name(ax, ay, az), const Offset(8, 7), ink: hot != null ? Grey.g91 : Grey.g63);
    _pliText(c, 'Shift-drag for depth · Alt for free', Offset(8, s.height - 7), ink: Grey.g56, ay: 1);
  }
}
