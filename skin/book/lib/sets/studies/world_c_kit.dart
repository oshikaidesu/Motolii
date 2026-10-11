// World C, shared plumbing: one Effect turned into a small WORLD (a box) whose picture is the whole effect, with up to four grab zones.
// The four zones keep the OP-1 colour slots: A = Fam.stagger (blue), B = Fam.along (green), C = N.g95 (white), D = Fam.follow (orange); the same
// slot always sits in the same order (research/op1-translation.md R2, R3). A world never does arithmetic on meaning: it only reports what was
// grabbed and the new numbers (host decides the rest, R10). Contracts: one gesture = one undo step (local list), Esc during a drag restores,
// Shift = fine (x0.1), double-click on a zone resets its parameters, hit areas >= 24 px, hover = a 1 px ring on the grabbable mark (no text),
// idle life is slow and quiet and slows further while the pointer is on the box, the number is a quiet 10 px mono readout at the edge.
// Test knob 'Driven' (Off | Wiggle | Pulse) moves the parameters from outside so the picture is seen moving.
import 'dart:math' as math;

import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../tokens.dart';

// ---- numbers ---------------------------------------------------------------------------------------------------------------------------

double wcClamp(double v, double a, double b) => v < a ? a : (v > b ? b : v);
double wcLerp(double a, double b, double t) => a + (b - a) * t;
double wcSmooth(double a, double b, double x) {
  final t = wcClamp((x - a) / (b - a), 0, 1);
  return t * t * (3 - 2 * t);
}

/// Smooth deterministic wobble in -1..1; [u] radians, [seed] separates channels.
double wcNoise(double u, double seed) =>
    math.sin(u + seed * 7.1) * .5 + math.sin(u * 2.31 + seed * 3.7) * .3 + math.sin(u * 4.73 + seed * 1.3) * .2;

/// Signed smallest difference of two angles in radians (-pi..pi).
double wcAngDiff(double a, double b) {
  var d = (a - b) % (2 * math.pi);
  if (d > math.pi) d -= 2 * math.pi;
  if (d < -math.pi) d += 2 * math.pi;
  return d;
}

Offset wcDir(double rad) => Offset(math.cos(rad), math.sin(rad));

Paint wcStroke(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;

Paint wcFill(Color c) => Paint()..color = c;

/// The four OP-1 colour slots, as existing tokens: A blue, B green, C white, D orange.
Color wcSlot(int i) => switch (i) { 0 => Fam.stagger.c, 1 => Fam.along.c, 2 => N.g95, _ => Fam.follow.c };
const wcSlotName = ['A', 'B', 'C', 'D'];

// ---- the value-object ------------------------------------------------------------------------------------------------------------------

/// One real parameter: name, range, default, unit. [scale] only changes how it is shown (seconds shown as ms).
class WSpec {
  const WSpec(this.id, this.name, this.min, this.max, this.def, {this.unit = '', this.digits = 0, this.integer = false, this.wrap = false, this.scale = 1});
  final String id, name, unit;
  final double min, max, def, scale;
  final int digits;
  final bool integer, wrap;

  String _n(double v) => (v * scale).toStringAsFixed(digits);
  String fmt(double v) => '${_n(v)}$unit';
  String get range => '${_n(min)}-${_n(max)}$unit';
  String get defText => '${_n(def)}$unit';
}

/// The numbers of one world. A plain map keyed by the spec ids; a gesture is begin..end and is one undo step.
class WDoc extends ChangeNotifier {
  WDoc(this.specs) : _v = {for (final s in specs) s.id: s.def};
  final List<WSpec> specs;
  final Map<String, double> _v;
  final List<Map<String, double>> _undo = [];
  Map<String, double>? _snap;

  double operator [](String id) => _v[id]!;
  WSpec spec(String id) => specs.firstWhere((s) => s.id == id);
  Map<String, double> get values => Map.unmodifiable(_v);
  bool changed(String id) => (_v[id]! - spec(id).def).abs() > 1e-9;
  bool get live => _snap != null;
  int get undoDepth => _undo.length;

  double norm(WSpec s, double x) {
    if (s.integer) x = x.roundToDouble();
    if (s.wrap) {
      final r = s.max - s.min;
      return ((x - s.min) % r + r) % r + s.min;
    }
    return wcClamp(x, s.min, s.max);
  }

  void set(String id, double x) {
    final n = norm(spec(id), x);
    if ((n - _v[id]!).abs() < 1e-12) return;
    _v[id] = n;
    notifyListeners();
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

  /// Esc during a drag: everything goes back, no undo step.
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
}

// ---- driving from outside (the test knob) ----------------------------------------------------------------------------------------------

enum Driven { off, wiggle, pulse }

/// How far parameter [i] is pushed at real time [t], as a fraction of its range.
double wcDriveOff(Driven m, int i, double t, bool wrap) {
  switch (m) {
    case Driven.off:
      return 0;
    case Driven.wiggle:
      return wcNoise(t * 1.5 + i * 2.3, i + 1.0) * (wrap ? .12 : .26);
    case Driven.pulse:
      final x = (t * .7 + i * .11) % 1.0;
      return (math.exp(-x * 5.5) * 1.3 - .25) * (wrap ? .1 : .34);
  }
}

// ---- a world ---------------------------------------------------------------------------------------------------------------------------

/// A world: state (displayed numbers, idle clock, pointer) + zones + painter. Subclasses draw with `v(id)` and the zone colour helpers.
abstract class WWorld {
  WWorld(this.doc) {
    for (final s in doc.specs) {
      _disp[s.id] = doc[s.id];
      _cur[s.id] = doc[s.id];
    }
  }
  final WDoc doc;
  Driven driven = Driven.off;
  final Map<String, double> _disp = {}, _cur = {};
  double t = 0; // idle clock: runs slower while the pointer is on the box
  double clock = 0; // real clock: drives the Driven knob
  double _rate = 1;
  Offset? ptr;
  int? hot, grab;
  Offset drag0 = Offset.zero; // pointer position at grab

  double v(String id) => _cur[id]!;

  /// Per-frame step. Displayed numbers glide to the stored ones (45 ms), then the Driven offset is added.
  void tick(double dt) {
    final target = (ptr != null || grab != null) ? .08 : 1.0;
    _rate += (target - _rate) * (1 - math.exp(-dt * 7));
    t += dt * _rate;
    clock += dt;
    final k = 1 - math.exp(-dt / .045);
    var i = 0;
    for (final s in doc.specs) {
      var d = doc[s.id] - _disp[s.id]!;
      final r = s.max - s.min;
      if (s.wrap) d = ((d + r / 2) % r + r) % r - r / 2;
      var x = _disp[s.id]! + d * k;
      if (s.wrap) x = ((x - s.min) % r + r) % r + s.min;
      _disp[s.id] = x;
      var c = x + wcDriveOff(driven, i, clock, s.wrap) * r;
      c = s.wrap ? ((c - s.min) % r + r) % r + s.min : wcClamp(c, s.min, s.max);
      _cur[s.id] = c;
      i++;
    }
    step(dt);
  }

  /// Extra per-frame integration a world may need (a phase that accumulates).
  void step(double dt) {}

  // colour helpers: a zone mark carries its slot colour; while another zone is grabbed it falls back to g38 (R2).
  Color zc(int i) => grab != null && grab != i ? N.g38 : wcSlot(i);
  bool lit(int i) => hot == i || grab == i;
  double zw(int i) => lit(i) ? 1.7 : 1.0;

  /// The 1 px hover / grab ring on a point mark.
  void ring(Canvas c, Offset p, int i, [double r = 9]) {
    if (!lit(i)) return;
    c.drawCircle(p, r, wcStroke(wcSlot(i).withValues(alpha: grab == i ? .9 : .6)));
  }

  String get title;
  String get silhouette;
  List<String> zoneIds(int z) => [doc.specs[z].id];

  /// Which grabbable zone is under [p] (null = none). Each zone's reach is >= 12 px around its mark (24 px hit area).
  int? zoneAt(Offset p, Size s);

  /// Called once at pointer-down on zone [z] so the world can remember geometry (the mark's position then).
  void onDown(int z, Offset p, Size s) {}

  /// [p] is the virtual pointer (start + accumulated, Shift makes it x0.1), [acc] the accumulated move, [start] the numbers at grab.
  void onDrag(int z, Offset p, Offset acc, Size s, Map<String, double> start);

  void paint(Canvas c, Size s);

  /// Angle change (radians) of the virtual pointer around [c] since the grab; unwrapped by [wcAngDiff].
  double angDelta(Offset c, Offset p) => wcAngDiff((p - c).direction, (drag0 - c).direction);

  /// Quiet readout: every parameter, changed ones in Role.changed.
  List<(String, bool)> readout() => [for (final s in doc.specs) (s.fmt(v(s.id)), doc.changed(s.id))];
}

// ---- the box ---------------------------------------------------------------------------------------------------------------------------

class _WPainter extends CustomPainter {
  _WPainter(this.w, Listenable l) : super(repaint: l);
  final WWorld w;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.clipRect(Offset.zero & size);
    w.paint(canvas, size);
  }

  @override
  bool shouldRepaint(_WPainter old) => true;
}

/// One world at a fixed size: frame, painter, pointer plumbing and the quiet readout.
class WorldBox extends StatefulWidget {
  const WorldBox({super.key, required this.def, required this.size, this.driven = Driven.off, this.onDoc});
  final WorldDef def;
  final Size size;
  final Driven driven;
  /// Lets a test reach the value-object.
  final void Function(WDoc doc)? onDoc;
  @override
  State<WorldBox> createState() => _WorldBoxState();
}

class _WorldBoxState extends State<WorldBox> with SingleTickerProviderStateMixin {
  late final WWorld world;
  late final Ticker _tk = createTicker(_tick);
  final _rev = ValueNotifier<int>(0);
  final _focus = FocusNode();
  Duration _last = Duration.zero;
  Duration? _lastDown;
  int? _lastDownZone;
  Offset _acc = Offset.zero;
  Map<String, double> _start = const {};
  bool _dead = false;
  MouseCursor _cursor = SystemMouseCursors.basic;

  Size get _s => widget.size;

  @override
  void initState() {
    super.initState();
    final doc = WDoc(widget.def.specs);
    world = widget.def.make(doc);
    world.driven = widget.driven;
    widget.onDoc?.call(doc);
    doc.addListener(_bump);
    _tk.start();
  }

  @override
  void didUpdateWidget(WorldBox old) {
    super.didUpdateWidget(old);
    world.driven = widget.driven;
  }

  @override
  void dispose() {
    _tk.dispose();
    _focus.dispose();
    world.doc.removeListener(_bump);
    super.dispose();
  }

  void _bump() => _rev.value++;

  void _tick(Duration d) {
    final dt = _last == Duration.zero ? 1 / 60 : math.min(.05, (d - _last).inMicroseconds / 1e6);
    _last = d;
    world.tick(dt);
    _rev.value++;
  }

  void _setCursor(MouseCursor c) {
    if (c != _cursor) setState(() => _cursor = c);
  }

  void _hover(Offset p) {
    world.ptr = p;
    world.hot = world.zoneAt(p, _s);
    _setCursor(world.grab != null ? SystemMouseCursors.grabbing : (world.hot != null ? SystemMouseCursors.grab : SystemMouseCursors.basic));
  }

  void _down(PointerDownEvent e) {
    final p = e.localPosition;
    world.ptr = p;
    final z = world.zoneAt(p, _s);
    world.hot = z;
    _focus.requestFocus();
    if (z == null) return;
    final now = e.timeStamp;
    if (_lastDown != null && _lastDownZone == z && (now - _lastDown!).inMilliseconds < 380) {
      _lastDown = null;
      world.doc.resetIds(world.zoneIds(z));
      _dead = true;
      return;
    }
    _lastDown = now;
    _lastDownZone = z;
    _dead = false;
    world.grab = z;
    world.drag0 = p;
    world.onDown(z, p, _s);
    world.doc.begin();
    _acc = Offset.zero;
    _start = world.doc.values;
    _setCursor(SystemMouseCursors.grabbing);
  }

  void _move(PointerMoveEvent e) {
    final z = world.grab;
    if (z == null || _dead) {
      _hover(e.localPosition);
      return;
    }
    final fine = HardwareKeyboard.instance.isShiftPressed;
    _acc += e.delta * (fine ? .1 : 1);
    world.ptr = e.localPosition;
    world.onDrag(z, world.drag0 + _acc, _acc, _s, _start);
  }

  void _up(PointerEvent e) {
    if (world.grab != null) world.doc.end();
    world.grab = null;
    _dead = false;
    _hover(e.localPosition);
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && world.grab != null) {
      world.doc.cancel();
      world.grab = null;
      _dead = true;
      _setCursor(SystemMouseCursors.basic);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: _s.width,
      height: _s.height,
      child: Focus(
        focusNode: _focus,
        onKeyEvent: _key,
        child: MouseRegion(
          cursor: _cursor,
          onHover: (e) => _hover(e.localPosition),
          onExit: (_) {
            if (world.grab == null) {
              world.ptr = null;
              world.hot = null;
              _setCursor(SystemMouseCursors.basic);
            }
          },
          child: Listener(
            onPointerDown: _down,
            onPointerMove: _move,
            onPointerUp: _up,
            onPointerCancel: _up,
            child: DecoratedBox(
              decoration: BoxDecoration(color: N.g07, border: Border.all(color: N.g26, width: 1)),
              child: Stack(children: [
                Positioned.fill(child: CustomPaint(painter: _WPainter(world, _rev))),
                Positioned(right: 5, bottom: 3, child: IgnorePointer(child: ListenableBuilder(listenable: _rev, builder: (_, _) => _Readout(world.readout())))),
              ]),
            ),
          ),
        ),
      ),
    );
  }
}

class _Readout extends StatelessWidget {
  const _Readout(this.parts);
  final List<(String, bool)> parts;
  @override
  Widget build(BuildContext context) {
    final base = TextStyle(fontFamily: T.mono, fontSize: 10, height: 1, decoration: TextDecoration.none);
    return ListenableBuilder(
      listenable: Role.changes,
      builder: (_, _) => DecoratedBox(
        decoration: BoxDecoration(color: N.g07.withValues(alpha: .8)),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 2),
          child: Row(mainAxisSize: MainAxisSize.min, children: [
            for (var i = 0; i < parts.length; i++) ...[
              Container(width: 4, height: 4, decoration: BoxDecoration(color: wcSlot(i), shape: BoxShape.circle)),
              const SizedBox(width: 3),
              Text(parts[i].$1, maxLines: 1, softWrap: false, style: base.copyWith(color: T.ink(parts[i].$2 ? Role.changed : N.g63))),
              if (i < parts.length - 1) const SizedBox(width: 8),
            ],
          ]),
        ),
      ),
    );
  }
}

// ---- use cases -------------------------------------------------------------------------------------------------------------------------

/// A world's identity for the book: number, name, silhouette phrase, specs and the constructor.
class WorldDef {
  const WorldDef(this.no, this.name, this.silhouette, this.specs, this.make);
  final int no;
  final String name, silhouette;
  final List<WSpec> specs;
  final WWorld Function(WDoc) make;

  /// One line: name, merged parameters (name range (default)), silhouette phrase.
  String get caption =>
      '$no $name  ·  ${[for (var i = 0; i < specs.length; i++) '${wcSlotName[i]} ${specs[i].name} ${specs[i].range} (${specs[i].defText})'].join('  ')}  ·  silhouette: $silhouette';
}

Driven wcDrivenKnob(BuildContext c) =>
    c.knobs.object.dropdown<Driven>(label: 'Driven', options: Driven.values, initialOption: Driven.off, labelBuilder: (d) => d.name);

Widget _caption(String text) => Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Text(text, maxLines: 1, softWrap: false, overflow: TextOverflow.visible, style: T.label(N.g63)),
    );

class _NumRow extends StatelessWidget {
  const _NumRow(this.label, this.value);
  final String label, value;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g20, width: 1))),
        child: Row(children: [
          Expanded(child: Text(label, style: T.label(N.g63))),
          Text(value, style: T.value(N.g91)),
        ]),
      );
}

/// The large version (320 x 200) with its caption outside the frame.
WidgetbookUseCase worldLarge(WorldDef d) => WidgetbookUseCase(
      name: '${d.no} ${d.name} · large',
      builder: (c) {
        final dr = wcDrivenKnob(c);
        return ColoredBox(
          color: N.g07,
          child: Center(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              WorldBox(key: ValueKey('L${d.no}'), def: d, size: const Size(320, 200), driven: dr),
              _caption(d.caption),
            ]),
          ),
        );
      },
    );

/// The same world in a 282 px Inspector row between two plain number rows.
WidgetbookUseCase worldRow(WorldDef d, {double height = 112}) => WidgetbookUseCase(
      name: '${d.no} ${d.name} · inspector row',
      builder: (c) {
        final dr = wcDrivenKnob(c);
        final h = c.knobs.double.slider(label: 'Row height', initialValue: height, min: 72, max: 132, divisions: 60);
        return ColoredBox(
          color: N.g07,
          child: Center(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Container(
                width: 282,
                color: N.g10,
                child: Column(mainAxisSize: MainAxisSize.min, children: [
                  const _NumRow('Opacity', '100 %'),
                  WorldBox(key: ValueKey('R${d.no}'), def: d, size: Size(282, h), driven: dr),
                  const _NumRow('Blend', 'Normal'),
                ]),
              ),
              _caption(d.caption),
            ]),
          ),
        );
      },
    );
