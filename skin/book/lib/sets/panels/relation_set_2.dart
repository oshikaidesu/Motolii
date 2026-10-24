// Part of relation_set.dart (cut there by size, as it was).
part of 'relation_set.dart';

class _LinkPainter extends CustomPainter {
  _LinkPainter(this.link, this.k, this.dashed, this.flow, this.t) : super(repaint: null);
  final _Link link;
  final Skin k;
  final bool dashed, flow;
  final double t;
  @override
  void paint(Canvas cv, Size s) {
    for (var x = 12.0; x < s.width; x += 24) {
      for (var y = 12.0; y < s.height; y += 24) {
        cv.drawCircle(Offset(x, y), .8, Paint()..color = N.g20);
      }
    }
    if (dashed) {
      if (k.glow) {
        for (final m in link.path.computeMetrics()) {
          for (var d = 0.0; d < m.length; d += 9) {
            cv.drawPath(m.extractPath(d, math.min(d + 5, m.length)), k.stroke(k.c.withValues(alpha: .3), 5)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5));
          }
        }
      }
      _dash(cv, link.path, k.stroke(k.c, 1.6), on: 5, off: 4);
    } else {
      k.line(cv, link.path, w: 1.6);
    }
    final m = link.path.computeMetrics().first, end = m.getTangentForOffset(m.length)!, u = Offset.fromDirection(-end.angle);
    k.dot(cv, link.a, 4);
    cv.drawCircle(link.a, 4, k.stroke(N.g100, 1.2));
    final head = Path()
      ..moveTo(link.b.dx, link.b.dy)
      ..lineTo(link.b.dx - u.dx * 9 + u.dy * 4.5, link.b.dy - u.dy * 9 - u.dx * 4.5)
      ..lineTo(link.b.dx - u.dx * 9 - u.dy * 4.5, link.b.dy - u.dy * 9 + u.dx * 4.5)
      ..close();
    cv.drawPath(head, k.fill(k.c));
    if (flow) {
      final f = (t % 1.0), pos = m.getTangentForOffset(m.length * f)!.position;
      k.dot(cv, pos, 2.6, col: N.g95);
    }
  }

  @override
  bool shouldRepaint(_LinkPainter o) => true;
}

class _ConnectionDemo extends StatefulWidget {
  const _ConnectionDemo({required this.look, required this.fam, required this.style, required this.dashed, required this.label, required this.flow});
  final Look look;
  final Fam fam;
  final int style;
  final bool dashed, label, flow;
  @override
  State<_ConnectionDemo> createState() => _ConnectionDemoState();
}

class _ConnectionDemoState extends State<_ConnectionDemo> with SingleTickerProviderStateMixin {
  Offset a = const Offset(40, 50), b = const Offset(330, 190);
  late final AnimationController clock = AnimationController(vsync: this, duration: const Duration(seconds: 2))..repeat();
  static const w = 520.0, h = 300.0;
  @override
  void dispose() {
    clock.dispose();
    super.dispose();
  }

  Widget layer(String name, Offset p, ValueChanged<Offset> set, bool badge) => Positioned(
        left: p.dx,
        top: p.dy,
        child: GestureDetector(
          onPanUpdate: (d) => set(Offset((p.dx + d.delta.dx).clamp(0.0, w - _layerW), (p.dy + d.delta.dy).clamp(0.0, h - _layerH))),
          child: MouseRegion(
            cursor: SystemMouseCursors.grab,
            child: Container(
              width: _layerW,
              height: _layerH,
              padding: const EdgeInsets.symmetric(horizontal: 10),
              decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26)),
              child: Row(children: [
                Expanded(child: Text(name, style: T.name())),
                if (badge) FamIcon(widget.fam, size: 13),
              ]),
            ),
          ),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final link = _link(a, b, widget.style), m = link.path.computeMetrics().first, mid = m.getTangentForOffset(m.length / 2)!.position;
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: Container(
        width: w,
        height: h,
        color: N.g07,
        child: Stack(children: [
          Positioned.fill(
            child: AnimatedBuilder(
              animation: clock,
              builder: (_, _) => CustomPaint(painter: _LinkPainter(link, Skin(widget.look, widget.fam.c), widget.dashed, widget.flow, clock.value)),
            ),
          ),
          layer('Leader', a, (p) => setState(() => a = p), false),
          layer('Gem 04', b, (p) => setState(() => b = p), true),
          if (widget.label)
            Positioned(left: mid.dx, top: mid.dy, child: FractionalTranslation(translation: const Offset(-.5, -.5), child: RelationChip(widget.fam, look: widget.look, compact: true, selected: true))),
        ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- empty state
class _EmptyDemo extends StatefulWidget {
  const _EmptyDemo(this.look);
  final Look look;
  @override
  State<_EmptyDemo> createState() => _EmptyDemoState();
}

class _EmptyDemoState extends State<_EmptyDemo> {
  final picked = <Fam>[];
  @override
  Widget build(BuildContext context) {
    final look = widget.look;
    return SizedBox(
      width: 320,
      child: Container(
          padding: const EdgeInsets.fromLTRB(20, 22, 20, 20),
          decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            if (picked.isEmpty) ...[
              SizedBox(
                height: 34,
                child: Row(mainAxisAlignment: MainAxisAlignment.center, children: [
                  for (var i = 0; i < 3; i++)
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 6),
                      child: FamIcon(Fam.all[i * 2], size: 24, color: N.g56),
                    ),
                ]),
              ),
              const SizedBox(height: 14),
              Text('No relations yet', style: T.title(N.g76)),
              const SizedBox(height: 6),
              Text('まだ関係がありません', style: T.label(N.g44)),
              const SizedBox(height: 4),
              Text('Connect this layer to another to start.', style: T.label(N.g56)),
              const SizedBox(height: 16),
              _Hover(
                onTap: () => setState(() => picked.add(Fam.scatter)),
                builder: (h) => AnimatedContainer(
                  duration: const Duration(milliseconds: 120),
                  curve: Curves.easeOut,
                  height: 28,
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  decoration: BoxDecoration(color: h ? N.g20 : N.g13, borderRadius: BorderRadius.circular(7), border: Border.all(color: h ? N.g44 : N.g20)),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    const _Glyph(_G.plus, color: N.g91),
                    const SizedBox(width: 7),
                    Text('Add Relation', style: T.name()),
                  ]),
                ),
              ),
              const SizedBox(height: 14),
              Text('START WITH', style: T.micro(N.g44).copyWith(letterSpacing: .8)),
              const SizedBox(height: 8),
              Wrap(spacing: 6, runSpacing: 6, alignment: WrapAlignment.center, children: [
                for (final f in Fam.all) RelationChip(f, look: look, compact: true, onTap: () => setState(() => picked.add(f))),
              ]),
            ] else ...[
              Wrap(spacing: 6, runSpacing: 6, children: [
                for (var i = 0; i < picked.length; i++) RelationChip(picked[i], look: look, onRemove: () => setState(() => picked.removeAt(i))),
              ]),
              const SizedBox(height: 10),
              Text('Remove them all to see the empty state again.', style: T.label(N.g56)),
            ],
          ]),
      ),
    );
  }
}

// ---------------------------------------------------------------- usage flow (add -> tune -> see)
class _ResultPainter extends CustomPainter {
  _ResultPainter(this.k, this.centre, this.radius, this.curve) : super(repaint: centre);
  final Skin k;
  final ValueNotifier<(Offset, double)> centre;
  final double radius, curve;
  @override
  void paint(Canvas cv, Size s) {
    final (c, r) = centre.value;
    final rnd = math.Random(11);
    for (var i = 0; i < 110; i++) {
      final p = Offset(rnd.nextDouble(), rnd.nextDouble()), d = (p - c).distance / math.max(.05, r), f = math.pow(1 - d.clamp(0.0, 1.0), curve).toDouble();
      k.dot(cv, p * s.width, 1.2 + 3.6 * f, a: .25 + .75 * f);
    }
  }

  @override
  bool shouldRepaint(_ResultPainter o) => o.k.look != k.look;
}

class _Flow extends StatefulWidget {
  const _Flow(this.look, this.curve);
  final Look look;
  final double curve;
  @override
  State<_Flow> createState() => _FlowState();
}

class _FlowState extends State<_Flow> {
  final centre = ValueNotifier<(Offset, double)>((const Offset(.5, .5), .3));
  @override
  void dispose() {
    centre.dispose();
    super.dispose();
  }

  Widget step(String no, String en, String jp, Widget child) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Text('$no  $en', style: T.name(N.g76)),
        const SizedBox(height: 4),
        Text(jp, style: T.label(N.g56)),
        const SizedBox(height: 10),
        child,
      ]);

  Widget arrow() => const Padding(padding: EdgeInsets.fromLTRB(14, 120, 14, 0), child: RotatedBox(quarterTurns: 3, child: _Glyph(_G.chevron, color: N.g44, size: 22)));

  @override
  Widget build(BuildContext context) => Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        step('1', 'Add a relation', 'Relationを追加', RelationMenuList(look: widget.look)),
        arrow(),
        step('2', 'Tune with the gadget', 'ガジェットで調整', FalloffGadget(look: widget.look, curve: widget.curve, onChanged: (c, r) => centre.value = (c, r))),
        arrow(),
        step('3', 'See it at once', 'すぐに反映', Container(
          width: 200,
          height: 200,
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(8), border: Border.all(color: N.g20)),
          child: ClipRRect(borderRadius: BorderRadius.circular(8), child: CustomPaint(painter: _ResultPainter(Skin(widget.look, Fam.scatter.c), centre, 0, widget.curve))),
        )),
      ]);
}

// ---------------------------------------------------------------- use cases
Widget _scroll(Widget child) => Stack(children: [
      SingleChildScrollView(padding: const EdgeInsets.only(bottom: 24), child: child),
      Positioned(left: 0, right: 0, bottom: 0, height: 24, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [N.g07.withValues(alpha: 0), N.g07]))))),
    ]);

Widget _page(List<Widget> kids) => _scroll(Wrap(spacing: 14, runSpacing: 14, children: kids));

WidgetbookComponent relationSetSet() => WidgetbookComponent(name: 'relation_set', useCases: [
      uc('Gadgets 1-4 · Space (Falloff, Direction, Scatter, Region)', (c) {
        final look = lookKnob(c);
        return _page([
          FalloffGadget(look: look, size: 150, curve: c.knobs.double.slider(label: 'Falloff curve', initialValue: 1.6, min: .4, max: 4), rings: c.knobs.int.slider(label: 'Rings', initialValue: 3, min: 1, max: 6)),
          DirectionGadget(look: look, size: 150, pull: c.knobs.double.slider(label: 'Attract (0 = direction only)', initialValue: .55, min: 0, max: 1), cells: c.knobs.int.slider(label: 'Arrow grid', initialValue: 7, min: 4, max: 10)),
          ScatterGadget(look: look, size: 150, density: c.knobs.int.slider(label: 'Density', initialValue: 60, min: 8, max: 160), seed: c.knobs.int.slider(label: 'Seed', initialValue: 3, min: 1, max: 30)),
          RegionGadget(look: look, size: 150, feather: c.knobs.double.slider(label: 'Feather', initialValue: .4, min: 0, max: 1), invert: c.knobs.boolean(label: 'Invert', initialValue: false)),
        ]);
      }, width: 792, height: 552),
      uc('Gadgets 5 · Path and time (Along Path, Stagger, Follow, Curve)', (c) {
        final look = lookKnob(c);
        return _page([
          AlongPathGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Path items', initialValue: 7, min: 2, max: 16), bias: c.knobs.double.slider(label: 'Path bias', initialValue: 1, min: .4, max: 2.5)),
          StaggerGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Stagger items', initialValue: 8, min: 3, max: 14), ease: c.knobs.double.slider(label: 'Stagger ease', initialValue: 1, min: .5, max: 2.5), preview: c.knobs.boolean(label: 'Stagger preview', initialValue: true)),
          FollowGadget(look: look, size: 150, lag: c.knobs.double.slider(label: 'Follow lag (s)', initialValue: .35, min: .05, max: 1.2), trail: c.knobs.int.slider(label: 'Follow trail', initialValue: 36, min: 8, max: 80)),
          CurveGadget(look: look, size: 150, preset: c.knobs.int.slider(label: 'Curve preset (0 linear .. 4 back)', initialValue: 1, min: 0, max: 4), preview: c.knobs.boolean(label: 'Curve preview', initialValue: true)),
        ]);
      }, width: 792, height: 552),
      uc('Gadgets 6 · Distribution (Scale, Rotation, Color, Repeat)', (c) {
        final look = lookKnob(c);
        return _page([
          ScaleGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Scale items', initialValue: 7, min: 3, max: 10)),
          RotationGadget(look: look, size: 150, count: c.knobs.int.slider(label: 'Rotation items', initialValue: 9, min: 3, max: 12), jitter: c.knobs.double.slider(label: 'Rotation random', initialValue: .15, min: 0, max: 1)),
          ColorGadget(look: look, size: 150, ease: c.knobs.double.slider(label: 'Gradient ease', initialValue: 1, min: .4, max: 2.5)),
          RepeatGadget(look: look, size: 150, cols: c.knobs.int.slider(label: 'Columns', initialValue: 5, min: 2, max: 8), rows: c.knobs.int.slider(label: 'Rows', initialValue: 3, min: 2, max: 6), jitter: c.knobs.double.slider(label: 'Grid jitter', initialValue: .25, min: 0, max: 1)),
        ]);
      }, width: 792, height: 552),
      uc('Gadgets 7 · Signal (Noise, Graph / Link, Audio)', (c) {
        final look = lookKnob(c);
        return _page([
          NoiseGadget(look: look, size: 150, roughness: c.knobs.double.slider(label: 'Roughness', initialValue: .45, min: 0, max: 1), drift: c.knobs.double.slider(label: 'Drift (0 = still)', initialValue: .5, min: 0, max: 2)),
          GraphGadget(look: look, size: 150, strength: c.knobs.double.slider(label: 'Link strength', initialValue: .7, min: .1, max: 1), type: c.knobs.int.slider(label: 'Link type (0 line, 1 curve, 2 step)', initialValue: 1, min: 0, max: 2), pulse: c.knobs.boolean(label: 'Pulse', initialValue: true)),
          AudioGadget(look: look, size: 150, smooth: c.knobs.double.slider(label: 'Audio smooth', initialValue: .5, min: 0, max: 1)),
        ]);
      }, width: 792, height: 340),
      uc('Gadget · one, large', (c) {
        final look = lookKnob(c);
        final no = c.knobs.int.slider(label: 'Gadget (1-15)', initialValue: 1, min: 1, max: 15);
        return _scroll( Column(mainAxisSize: MainAxisSize.min, children: [
          Text(gadgetNames[no - 1], style: T.label(N.g56)),
          const SizedBox(height: 10),
          gadgetByNo(no, look: look, size: c.knobs.double.slider(label: 'Size', initialValue: 340, min: 200, max: 440), framed: c.knobs.boolean(label: 'Framed', initialValue: true)),
        ]));
      }, width: 520, height: 552),
      uc('Gadget wall · all 15', (c) {
        final look = lookKnob(c);
        final size = c.knobs.double.slider(label: 'Size', initialValue: 130, min: 110, max: 170);
        return _page([for (var i = 1; i <= 15; i++) gadgetByNo(i, look: look, size: size)]);
      }, width: 792, height: 552),
      uc('Chip · compact, for lists', (c) => _ChipDemo(lookKnob(c), c.knobs.boolean(label: 'Compact', initialValue: false), c.knobs.boolean(label: 'Show value', initialValue: true)), width: 480, height: 160),
      uc('Badge · on a layer row', (c) => Align(child: _LayerRows(lookKnob(c), c.knobs.int.slider(label: 'Badge style (0 dot, 1 mark, 2 bar)', initialValue: 1, min: 0, max: 2), c.knobs.double.slider(label: 'Row scale', initialValue: 1.4, min: 1, max: 2))), width: 340, height: 200),
      uc('Add relation menu', (c) => _AddMenuDemo(lookKnob(c), c.knobs.boolean(label: 'Start open', initialValue: true)), width: 260, height: 420),
      uc('Card · collapsed / expanded', (c) {
        final look = lookKnob(c), f = Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 0, min: 0, max: 5)];
        return _scroll( Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          RelCard(f, look: look, mode: CardMode.expanded),
          const SizedBox(height: 10),
          RelCard(Fam.all[(Fam.all.indexOf(f) + 1) % 6], look: look, mode: CardMode.collapsed),
        ]));
      }, width: 400, height: 520),
      uc('Card · disabled / in error', (c) {
        final look = lookKnob(c), f = Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 5, min: 0, max: 5)];
        final msg = c.knobs.string(label: 'Error message', initialValue: 'Target layer "Gem 04" was deleted');
        return _scroll( Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          RelCard(f, look: look, mode: CardMode.disabled),
          const SizedBox(height: 10),
          RelCard(f, look: look, mode: CardMode.error, message: msg),
        ]));
      }, width: 400, height: 552),
      uc('Connection line · drag the layers', (c) => _ConnectionDemo(
            look: lookKnob(c),
            fam: Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 5, min: 0, max: 5)],
            style: c.knobs.int.slider(label: 'Line (0 straight, 1 curve, 2 elbow)', initialValue: 1, min: 0, max: 2),
            dashed: c.knobs.boolean(label: 'Dashed', initialValue: false),
            label: c.knobs.boolean(label: 'Label', initialValue: true),
            flow: c.knobs.boolean(label: 'Flow', initialValue: true),
          ), width: 520, height: 300),
      uc('Empty state · no relations yet', (c) => Align(child: _EmptyDemo(lookKnob(c))), width: 340, height: 330),
      uc('Flow · add, tune, see', (c) => _Flow(lookKnob(c), c.knobs.double.slider(label: 'Result falloff', initialValue: 1.6, min: .4, max: 4)), width: 760, height: 400),
      uc('Card list · inspector column', (c) {
        final look = lookKnob(c);
        return _scroll( Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          RelCard(Fam.scatter, look: look, mode: CardMode.expanded),
          const SizedBox(height: 6),
          RelCard(Fam.stagger, look: look, mode: CardMode.collapsed),
          const SizedBox(height: 6),
          RelCard(Fam.along, look: look, mode: CardMode.collapsed),
          const SizedBox(height: 6),
          RelCard(Fam.face, look: look, mode: CardMode.disabled),
        ]));
      }, width: 372, height: 560),
    ]);
