// World E, shared plumbing: a small WORLD (one Effect / Relation / Property concept drawn as a little scene) with up to four grab zones.
// Zone i is colour slot i: A = Fam.stagger (blue), B = Fam.along (green), C = N.g95 (white), D = Fam.follow (orange); same order in every world.
// Contracts: no words inside the frame; the numbers are a quiet 10 px mono readout at the bottom edge (always there); the world breathes when idle;
// 'Driven' (Off | Wiggle | Pulse) moves the parameters from outside so the graphic is seen moving; one gesture = one undo step (local list);
// Esc during a drag restores; Shift = fine (x0.1); double-click resets the zone's parameter (none under the pointer = all); hit areas >= 24 px.
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../tokens.dart';

// ---- numbers and colours ----------------------------------------------------------------------------------------------------------

double weClamp(double v, double a, double b) => v < a ? a : (v > b ? b : v);
double weLerp(double a, double b, double t) => a + (b - a) * t;
double weWrap(double a) {
  while (a > math.pi) {
    a -= 2 * math.pi;
  }
  while (a < -math.pi) {
    a += 2 * math.pi;
  }
  return a;
}

/// A smooth deterministic wobble in -1..1; [u] radians, [seed] separates channels.
double weNoise(double u, double seed) => math.sin(u + seed * 7.1) * .5 + math.sin(u * 2.31 + seed * 3.7) * .3 + math.sin(u * 4.73 + seed * 1.3) * .2;

/// A stable pseudo-random 0..1 from two ints (for dust, patterns).
double weHash(int a, int b) {
  var h = (a * 374761393 + b * 668265263) & 0x7fffffff;
  h = ((h ^ (h >> 13)) * 1274126177) & 0x7fffffff;
  return (h & 0xffff) / 65535.0;
}

/// The four colour slots, in the order of the encoders of the OP-1: A blue, B green, C white, D orange (existing tokens, no new hue).
Color weSlot(int i) => const [Fam.stagger, Fam.along, null, Fam.follow][i]?.c ?? N.g95;
Color weHi(Color c) => Color.lerp(c, N.g100, .45)!;

Paint weLine(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint weFill(Color c) => Paint()..color = c;

void weDash(Canvas c, Offset a, Offset b, Paint p, {double on = 2, double off = 3}) {
  final d = b - a, len = d.distance;
  if (len < .5) return;
  final u = d / len;
  for (double s = 0; s < len; s += on + off) {
    c.drawLine(a + u * s, a + u * math.min(s + on, len), p);
  }
}

double weDistSeg(Offset p, Offset a, Offset b) {
  final ab = b - a, l2 = ab.dx * ab.dx + ab.dy * ab.dy;
  if (l2 < 1e-9) return (p - a).distance;
  final t = weClamp(((p - a).dx * ab.dx + (p - a).dy * ab.dy) / l2, 0, 1);
  return (p - (a + ab * t)).distance;
}

// ---- the value-object -------------------------------------------------------------------------------------------------------------

/// One real, named parameter: name, range, default, unit (as in research/op1-translation.md; ranges are [inferred] drafts).
class WSpec {
  const WSpec(this.id, this.name, this.min, this.max, this.def, {this.unit = '', this.digits = 2, this.integer = false, this.drive = true, this.show, this.range});
  final String id, name, unit;
  final double min, max, def;
  final int digits;
  final bool integer, drive;

  /// Custom readout text (e.g. the infinity of a loop count) and the range text of the caption.
  final String Function(double v)? show;
  final String? range;

  String fmt(double v) {
    if (show != null) return show!(v);
    var s = v.toStringAsFixed(digits);
    if (v.abs() < 1 && digits > 0) s = s.replaceFirst('0.', '.');
    return unit.isEmpty ? s : '$s$unit';
  }

  String get caption {
    String n(double x) => x.toStringAsFixed(digits);
    final u = unit.isEmpty ? '' : (unit == '%' || unit == '°' ? unit : ' $unit');
    return '$name ${range ?? '${n(min)}-${n(max)}$u'} (${show != null ? show!(def) : '${n(def)}$u'})';
  }
}

/// How the parameters are moved from outside, so the graphic is seen moving (the way the OP-1 shows modulation).
enum Driven { off, wiggle, pulse }

/// The values of one world: a plain map keyed by spec id. A gesture is begin..end and is one undo step. [driven] and the clock live here so two
/// views of one world (large and in the Inspector row) show the same thing.
class WDoc extends ChangeNotifier {
  WDoc(this.specs) : _v = {for (final s in specs) s.id: s.def};
  final List<WSpec> specs;
  final Map<String, double> _v;
  final List<Map<String, double>> _undo = [];
  Map<String, double>? _snap;
  Driven driven = Driven.off;
  double t = 0, dt = 0;
  final frame = ValueNotifier<int>(0);

  double operator [](String id) => _v[id]!;
  WSpec spec(String id) => specs.firstWhere((s) => s.id == id);
  int index(String id) => specs.indexWhere((s) => s.id == id);
  bool changed(String id) => (_v[id]! - spec(id).def).abs() > 1e-9;
  Map<String, double> get values => Map.unmodifiable(_v);
  bool get live => _snap != null;

  /// The value the world is drawn with: the base value plus the outside drive.
  double eff(String id) {
    final s = spec(id), b = _v[id]!;
    if (driven == Driven.off || !s.drive) return b;
    final i = index(id);
    final span = s.max - s.min;
    double m;
    if (driven == Driven.wiggle) {
      m = weNoise(t * 1.9 + i * 2.3, i.toDouble()) * .26;
    } else {
      final ph = ((t / 1.5) + i * .21) % 1.0;
      final env = ph < .1 ? ph / .1 : math.exp(-(ph - .1) * 4.2);
      m = env * .3 * ((b - s.min) < (s.max - b) ? 1 : -1);
    }
    return weClamp(b + m * span, s.min, s.max);
  }

  String show(String id) => spec(id).fmt(eff(id));

  void set(String id, double x) {
    final s = spec(id);
    var n = weClamp(x, s.min, s.max);
    if (s.integer) n = n.roundToDouble();
    if ((n - _v[id]!).abs() < 1e-12) return;
    _v[id] = n;
    notifyListeners();
  }

  void advance(double d) {
    dt = d;
    t += d;
    frame.value++;
  }

  void begin() => _snap ??= Map.of(_v);

  void end() {
    final s = _snap;
    _snap = null;
    if (s == null) return;
    for (final k in s.keys) {
      if ((s[k]! - _v[k]!).abs() > 1e-12) {
        _undo.add(s);
        return;
      }
    }
  }

  void cancel() {
    final s = _snap;
    _snap = null;
    if (s == null) return;
    _v..clear()..addAll(s);
    notifyListeners();
  }

  void resetIds(Iterable<String> ids) {
    begin();
    for (final id in ids) {
      _v[id] = spec(id).def;
    }
    notifyListeners();
    end();
  }

  void undo() {
    if (_undo.isEmpty) return;
    _v..clear()..addAll(_undo.removeLast());
    notifyListeners();
  }

  @override
  void dispose() {
    frame.dispose();
    super.dispose();
  }
}

// ---- the world --------------------------------------------------------------------------------------------------------------------

class WView {
  Offset? ptr;
  int? hot, grab;
}

abstract class WeWorld {
  WeWorld(this.doc);
  final WDoc doc;
  final view = WView();

  /// Effective value (base + outside drive) of spec [i] in the order A..D.
  double v(int i) => doc.eff(doc.specs[i].id);
  double base(int i) => doc[doc.specs[i].id];
  void put(int i, double x) => doc.set(doc.specs[i].id, x);

  /// The drawing area: the frame minus a margin and the strip of the readout at the bottom.
  Rect art(Size s) => Rect.fromLTRB(10, 8, s.width - 10, s.height - 19);

  bool on(int z) => view.hot == z || view.grab == z;

  /// The colour of zone [z]'s marks now: its slot colour (quiet), brighter under the pointer, and everything else drops to g38 while one zone is held.
  Color slot(int z, [double a = .85]) {
    if (view.grab != null && view.grab != z) return N.g38;
    final c = weSlot(z);
    return on(z) ? weHi(c) : c.withValues(alpha: a);
  }

  /// Zones are >= 24 px: each world pads its hit tests itself. The nearest zone wins.
  int? zoneAt(Offset p, Size s);
  void down(int z, Offset p, Size s) {}
  void move(int z, Offset p, Offset d, bool fine, Size s);
  void up(int z) {}
  void cancelled(int z) => up(z);
  List<String> zoneIds(int z) => [doc.specs[z].id];
  void synced() {}
  void step(double dt, Size s) {}
  void paint(Canvas c, Size s);

  /// A 1 px ring that tells "this is grabbable" under the pointer (no text).
  void ring(Canvas c, Offset o, double r, int z) {
    if (on(z)) c.drawCircle(o, r, weLine(weHi(weSlot(z)).withValues(alpha: .8)));
  }
}

class _WPainter extends CustomPainter {
  _WPainter(this.w, Listenable l) : super(repaint: l);
  final WeWorld w;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    w.paint(canvas, size);
    canvas.restore();
  }

  @override
  bool shouldRepaint(_WPainter old) => true;
}

/// One world at a fixed size: frame, painter, pointer plumbing, readout.
class WBox extends StatefulWidget {
  const WBox({super.key, required this.doc, required this.make, required this.size});
  final WDoc doc;
  final WeWorld Function(WDoc) make;
  final Size size;
  @override
  State<WBox> createState() => _WBoxState();
}

class _WBoxState extends State<WBox> {
  late final WeWorld world = widget.make(widget.doc);
  final _rev = ValueNotifier<int>(0);
  final _focus = FocusNode();
  bool _dead = false;
  Duration? _lastDown;
  Offset _lastDownP = Offset.zero;
  MouseCursor _cursor = SystemMouseCursors.basic;

  WView get v => world.view;

  @override
  void initState() {
    super.initState();
    widget.doc.frame.addListener(_frame);
  }

  @override
  void dispose() {
    widget.doc.frame.removeListener(_frame);
    _rev.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _frame() {
    world.step(widget.doc.dt, widget.size);
    _rev.value++;
  }

  void _down(PointerDownEvent e) {
    if (e.buttons != kPrimaryButton) return;
    final p = e.localPosition, z = world.zoneAt(p, widget.size);
    _focus.requestFocus();
    final dbl = _lastDown != null && e.timeStamp - _lastDown! < const Duration(milliseconds: 350) && (p - _lastDownP).distance < 8;
    _lastDown = dbl ? null : e.timeStamp;
    _lastDownP = p;
    if (dbl) {
      widget.doc.resetIds(z == null ? widget.doc.specs.map((s) => s.id) : world.zoneIds(z));
      world.synced();
      _rev.value++;
      return;
    }
    if (z == null) return;
    _dead = false;
    widget.doc.begin();
    v.ptr = p;
    v.grab = z;
    world.down(z, p, widget.size);
    setState(() => _cursor = SystemMouseCursors.grabbing);
  }

  void _move(PointerMoveEvent e) {
    final z = v.grab;
    if (z == null || _dead) return;
    v.ptr = e.localPosition;
    world.move(z, e.localPosition, e.localDelta, HardwareKeyboard.instance.isShiftPressed, widget.size);
    _rev.value++;
  }

  void _up(PointerEvent e) {
    final z = v.grab;
    if (z == null) return;
    if (!_dead) {
      world.up(z);
      widget.doc.end();
    }
    v.grab = null;
    _dead = false;
    _hover(e.localPosition);
  }

  void _hover(Offset p) {
    v.ptr = p;
    v.hot = world.zoneAt(p, widget.size);
    final want = v.hot == null ? SystemMouseCursors.basic : SystemMouseCursors.grab;
    if (want != _cursor) setState(() => _cursor = want);
    _rev.value++;
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final z = v.grab;
    if (e.logicalKey == LogicalKeyboardKey.escape && z != null) {
      widget.doc.cancel();
      world.cancelled(z);
      world.synced();
      v.grab = null;
      _dead = true;
      _rev.value++;
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      widget.doc.undo();
      world.synced();
      _rev.value++;
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final sz = widget.size;
    return ListenableBuilder(
      listenable: Role.changes,
      builder: (context, _) => SizedBox(
        width: sz.width,
        height: sz.height,
        child: Focus(
          focusNode: _focus,
          onKeyEvent: _key,
          child: MouseRegion(
            cursor: _cursor,
            onHover: (e) => _hover(e.localPosition),
            onExit: (_) {
              v.ptr = null;
              v.hot = null;
              if (v.grab == null && _cursor != SystemMouseCursors.basic) setState(() => _cursor = SystemMouseCursors.basic);
              _rev.value++;
            },
            child: Listener(
              onPointerDown: _down,
              onPointerMove: _move,
              onPointerUp: _up,
              onPointerCancel: _up,
              child: DecoratedBox(
                decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20), borderRadius: BorderRadius.circular(Gui.radius)),
                child: ClipRRect(
                  borderRadius: BorderRadius.circular(Gui.radius - 1),
                  child: Stack(children: [
                    Positioned.fill(child: CustomPaint(painter: _WPainter(world, Listenable.merge([_rev, widget.doc, Role.changes])))),
                    Positioned(right: 8, bottom: 4, child: IgnorePointer(child: _Readout(widget.doc))),
                  ]),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// The numbers, underneath: 10 px mono, g63, a 3 px dot of the zone's slot colour before each; a value that differs from its default turns Role.changed.
class _Readout extends StatelessWidget {
  const _Readout(this.doc);
  final WDoc doc;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([doc, doc.frame]),
        builder: (context, _) => Row(mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < doc.specs.length; i++) ...[
            if (i > 0) const SizedBox(width: 7),
            Container(width: 3, height: 3, decoration: BoxDecoration(color: weSlot(i), shape: BoxShape.circle)),
            const SizedBox(width: 3),
            Text(doc.show(doc.specs[i].id), style: T.value(doc.changed(doc.specs[i].id) ? Role.of(N.g76, Role.changed) : N.g63).copyWith(fontSize: 10)),
          ],
        ]),
      );
}

// ---- the use case frame -----------------------------------------------------------------------------------------------------------

/// A plain neighbour row of the Inspector: a label and a number cell.
class WNumRow extends StatelessWidget {
  const WNumRow(this.label, this.value, {super.key});
  final String label, value;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: 282,
        height: 24,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 10),
          child: Row(children: [
            Expanded(child: Text(label, style: T.label(N.g63))),
            Container(
              width: 84,
              height: 20,
              alignment: Alignment.centerRight,
              padding: const EdgeInsets.symmetric(horizontal: 6),
              decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(3)),
              child: Text(value, style: T.value(N.g91)),
            ),
          ]),
        ),
      );
}

/// What one world is: its name, its four parameters (A..D), the maker, and the two plain neighbour rows of its Inspector context.
class WorldDef {
  const WorldDef({required this.name, required this.specs, required this.make, required this.silhouette, required this.rowH, required this.above, required this.below});
  final String name, silhouette;
  final List<WSpec> specs;
  final WeWorld Function(WDoc) make;
  final double rowH;
  final (String, String) above, below;

  String get caption => '$name  ·  ${specs.map((s) => s.caption).join('  ·  ')}  ·  silhouette: $silhouette';
}

/// One use case: the caption (outside the frame, one line), the large world (320x200) and the same world in a 282 px Inspector row between two plain
/// number rows. Both views share one [WDoc]; one ticker (this one) moves the clock of both.
class WStory extends StatefulWidget {
  const WStory({super.key, required this.def, required this.driven});
  final WorldDef def;
  final Driven driven;
  @override
  State<WStory> createState() => _WStoryState();
}

class _WStoryState extends State<WStory> with SingleTickerProviderStateMixin {
  late final WDoc doc = WDoc(widget.def.specs)..driven = widget.driven;
  late final Ticker _tk = createTicker(_tick);
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk.start();
  }

  void _tick(Duration e) {
    final dt = math.min((e - _last).inMicroseconds / 1e6, .05);
    _last = e;
    doc.advance(dt);
  }

  @override
  void didUpdateWidget(WStory old) {
    super.didUpdateWidget(old);
    doc.driven = widget.driven;
  }

  @override
  void dispose() {
    _tk.dispose();
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final d = widget.def;
    return ColoredBox(
      color: N.g07,
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          SingleChildScrollView(scrollDirection: Axis.horizontal, child: Text(d.caption, maxLines: 1, softWrap: false, style: T.label(N.g63))),
          const SizedBox(height: 14),
          Wrap(spacing: 40, runSpacing: 28, crossAxisAlignment: WrapCrossAlignment.start, children: [
            WBox(doc: doc, make: d.make, size: const Size(320, 200)),
            SizedBox(
              width: 282,
              child: Column(mainAxisSize: MainAxisSize.min, children: [
                WNumRow(d.above.$1, d.above.$2),
                const SizedBox(height: 4),
                WBox(doc: doc, make: d.make, size: Size(282, d.rowH)),
                const SizedBox(height: 4),
                WNumRow(d.below.$1, d.below.$2),
              ]),
            ),
          ]),
        ]),
      ),
    );
  }
}

/// The use case of a world: one knob, 'Driven'.
WidgetbookUseCase weUseCase(WorldDef d) => WidgetbookUseCase(
      name: d.name,
      builder: (c) => WStory(def: d, driven: c.knobs.object.dropdown<Driven>(label: 'Driven', options: Driven.values, initialOption: Driven.off, labelBuilder: (m) => switch (m) { Driven.off => 'Off', Driven.wiggle => 'Wiggle', Driven.pulse => 'Pulse' })),
    );
