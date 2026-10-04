// World B, shared plumbing: a small WORLD (a box) per effect/relation. One box, four grab zones A-D (blue, green, white, orange = Fam.stagger,
// Fam.along, N.g95, Fam.follow), numbers only as a quiet 10 px mono readout at the edge. Contracts for every world: hover lights the grabbable
// zone (no text), Esc during a drag restores, Shift = fine (x0.1), double-click resets that zone, hit radius 14 px (>= 24 px target),
// 1 gesture = 1 undo step (local, never shown), the world decides nothing about arithmetic beyond its own geometry (the host would).
import 'dart:math' as math;

import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';

// ---- numbers -------------------------------------------------------------------------------------------------------------------------

double wbClamp(double v, double a, double b) => v < a ? a : (v > b ? b : v);
double wbLerp(double a, double b, double t) => a + (b - a) * t;
double wbRad(double deg) => deg * math.pi / 180;

/// Deterministic 0..1 from an index and a seed.
double wbHash(num i, num seed) {
  final x = math.sin(i * 12.9898 + seed * 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

/// Smooth deterministic wobble in -1..1; [seed] separates channels.
double wbNoise(double u, double seed) => math.sin(u + seed * 7.1) * .5 + math.sin(u * 2.31 + seed * 3.7) * .3 + math.sin(u * 4.73 + seed * 1.3) * .2;

double wbWrapDeg(double d) => ((d + 540) % 360) - 180;

/// Signed smallest difference a-b of two radian angles.
double wbAngDiff(double a, double b) {
  var d = a - b;
  while (d > math.pi) {
    d -= 2 * math.pi;
  }
  while (d < -math.pi) {
    d += 2 * math.pi;
  }
  return d;
}

/// Unit vector of a compass angle in degrees: 0 = up, clockwise.
Offset wbDir(double deg) => Offset(math.sin(wbRad(deg)), -math.cos(wbRad(deg)));

/// Compass angle in degrees of a vector.
double wbAngleOf(Offset d) => math.atan2(d.dx, -d.dy) * 180 / math.pi;

double wbEaseOut(double u) => 1 - (1 - u) * (1 - u) * (1 - u);

/// The four grab-zone colours: A Fam.stagger (blue), B Fam.along (green), C white, D Fam.follow (orange).
Color wbSlot(int z) => switch (z) { 0 => Fam.stagger.c, 1 => Fam.along.c, 2 => N.g95, _ => Fam.follow.c };
const wbSlotLetter = ['A', 'B', 'C', 'D'];

Paint wbStroke(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;

Paint wbFill(Color c) => Paint()..color = c;

/// A drawing area that keeps the bottom 20 px free for the readout.
Rect wbArea(Size s) => Rect.fromLTRB(10, 8, s.width - 10, s.height - 20);

/// Points along a straight line, at most [step] apart (hit samples for a line-shaped zone).
List<Offset> wbSamples(Offset a, Offset b, [double step = 8]) {
  final n = math.max(1, ((b - a).distance / step).ceil());
  return [for (var i = 0; i <= n; i++) Offset.lerp(a, b, i / n)!];
}

// ---- the value-object ------------------------------------------------------------------------------------------------------------------

/// One real, named parameter: id, name, range, default, unit. [driven] false = the Driven knob leaves it alone (integers, switches).
class WbSpec {
  const WbSpec(this.id, this.name, this.min, this.max, this.def, {this.unit = '', this.digits = 2, this.integer = false, this.driven = true, this.amp = .18});
  final String id, name, unit;
  final double min, max, def;
  final int digits;
  final bool integer, driven;
  final double amp;

  static String n(double v, int digits) {
    var s = v.toStringAsFixed(digits);
    if (v.abs() < 1 && digits > 0) s = s.replaceFirst('0.', '.');
    return s.replaceFirst('-', '−');
  }

  String fmt(double v) => n(v, digits) + unit;
  String get range => '${n(min, digits)}..${n(max, digits)}$unit';
}

enum WbDriven {
  off('Off'),
  wiggle('Wiggle'),
  pulse('Pulse');

  const WbDriven(this.label);
  final String label;
}

/// The values of one world: a plain map keyed by the spec ids. A gesture is begin..end and is one undo step.
class WbDoc extends ChangeNotifier {
  WbDoc(this.specs) : _v = {for (final s in specs) s.id: s.def};
  final List<WbSpec> specs;
  final Map<String, double> _v;
  final List<Map<String, double>> _undo = [];
  Map<String, double>? _snap;

  double operator [](String id) => _v[id]!;
  Map<String, double> get values => Map.unmodifiable(_v);
  WbSpec spec(String id) => specs.firstWhere((s) => s.id == id);
  bool changed(String id) => (_v[id]! - spec(id).def).abs() > 1e-9;
  bool get live => _snap != null;
  int get undoDepth => _undo.length;

  double _fit(WbSpec s, double x) {
    final c = wbClamp(x, s.min, s.max);
    return s.integer ? c.roundToDouble() : c;
  }

  void setAll(Map<String, double> m) {
    var any = false;
    for (final e in m.entries) {
      final s = spec(e.key), x = _fit(s, e.value);
      if ((_v[e.key]! - x).abs() > 1e-12) {
        _v[e.key] = x;
        any = true;
      }
    }
    if (any) notifyListeners();
  }

  void reset(Iterable<String> ids) {
    begin();
    setAll({for (final id in ids) id: spec(id).def});
    end();
  }

  void begin() => _snap ??= Map.of(_v);

  void end() {
    final s = _snap;
    _snap = null;
    if (s == null) return;
    if (s.entries.any((e) => (_v[e.key]! - e.value).abs() > 1e-12)) _undo.add(s);
  }

  void cancel() {
    final s = _snap;
    _snap = null;
    if (s == null) return;
    _v.addAll(s);
    notifyListeners();
  }

  void undo() {
    if (_undo.isEmpty) return;
    _v.addAll(_undo.removeLast());
    notifyListeners();
  }

  /// The values as seen right now when the parameters are driven from outside (the way OP-1 shows a modulated parameter): the base is untouched.
  Map<String, double> driven(WbDriven mode, double t) {
    if (mode == WbDriven.off) return Map.of(_v);
    final out = Map.of(_v);
    for (var i = 0; i < specs.length; i++) {
      final s = specs[i];
      if (!s.driven) continue;
      final range = s.max - s.min, base = _v[s.id]!, amp = s.amp * range;
      double off;
      if (mode == WbDriven.wiggle) {
        off = amp * wbNoise(t * (1.1 + .17 * i), i * 3.1);
      } else {
        const period = 1.9;
        final ph = (((t - i * .17) % period) + period) % period / period;
        final env = math.exp(-ph * 4.5); // starts the instant the beat lands, then eases out
        off = (base + amp > s.max ? -1 : 1) * amp * 1.6 * env;
      }
      out[s.id] = _fit(s, base + off);
    }
    return out;
  }
}

// ---- the world contract ----------------------------------------------------------------------------------------------------------------

class WbFrame {
  const WbFrame({required this.v, required this.t, required this.hot, required this.grab});
  final Map<String, double> v;
  final double t;
  final int? hot, grab;
  double operator [](String id) => v[id]!;
  bool lit(int z) => hot == z || grab == z;

  /// While a zone is held every other zone falls to g38.
  bool dim(int z) => grab != null && grab != z;
  Color mark(int z, [double a = .9]) => dim(z) ? N.g38 : wbSlot(z).withValues(alpha: lit(z) ? 1 : a);
  Color ink(int z, Color idle) => dim(z) ? N.g38 : idle;
  double wid(int z) => lit(z) ? 1.6 : 1.0;
}

abstract class WbWorld {
  String get name;
  String get silhouette;
  List<WbSpec> get specs;

  /// The spec ids each zone (A, B, C, D) owns: a double-click resets exactly these.
  List<List<String>> get zoneIds;

  /// Zones that a click (no drag) acts on, as a switch or a cycle.
  Set<int> get tapZones => const {};

  /// Called once per frame before painting; worlds keep their own small simulation here.
  void tick(double t, double dt, Map<String, double> v, Size s) {}

  /// Hit points per zone (sample a line into points with [wbSamples]). Empty = the zone is not grabbable now.
  List<List<Offset>> anchors(Size s, Map<String, double> v);

  /// New values for a drag of [z] from [p0] to [p] (already scaled by Shift). Must return [v0] unchanged when p == p0.
  Map<String, double> drag(int z, Offset p, Offset p0, Map<String, double> v0, Size s);
  Map<String, double>? tap(int z, Map<String, double> v) => null;
  void paint(Canvas c, Size s, WbFrame f);
  String readout(Map<String, double> v);

  String get caption {
    final parts = [for (var z = 0; z < specs.length; z++) '${wbSlotLetter[z]} ${specs[z].name} ${specs[z].range} (${specs[z].fmt(specs[z].def)})'];
    return '$name · ${parts.join(' · ')} · silhouette: $silhouette';
  }
}

// ---- the box ---------------------------------------------------------------------------------------------------------------------------

class WbBox extends StatefulWidget {
  const WbBox({super.key, required this.make, required this.doc, required this.mode, required this.width, required this.height});
  final WbWorld Function() make;
  final WbDoc doc;
  final WbDriven mode;
  final double width, height;
  @override
  State<WbBox> createState() => _WbBoxState();
}

class _WbBoxState extends State<WbBox> with SingleTickerProviderStateMixin {
  static const reach = 14.0; // hit radius: a 28 px target around every grab point
  late final WbWorld world = widget.make();
  late final Ticker _ticker;
  final ValueNotifier<int> _frame = ValueNotifier(0);
  double _t = 0;
  late Map<String, double> _v = widget.doc.values;
  int? _hot, _grab;
  Offset _ptr = Offset.zero, _p0 = Offset.zero;
  Map<String, double> _v0 = {};
  bool _moved = false;
  int _lastDownMs = -9999, _lastZone = -1;
  Offset _lastDownAt = Offset.zero;
  Size get _size => Size(widget.width, widget.height);

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick)..start();
    widget.doc.addListener(_docChanged);
    Role.changes.addListener(_docChanged);
  }

  @override
  void dispose() {
    _ticker.dispose();
    widget.doc.removeListener(_docChanged);
    Role.changes.removeListener(_docChanged);
    HardwareKeyboard.instance.removeHandler(_key);
    _frame.dispose();
    super.dispose();
  }

  void _docChanged() {
    _v = widget.doc.driven(widget.mode, _t); // a hit test right after an edit must see the edit
    _frame.value++;
  }

  void _onTick(Duration e) {
    final t = e.inMicroseconds / 1e6, dt = math.max(0.0, math.min(.05, t - _t));
    _t = t;
    _v = widget.doc.driven(widget.mode, t);
    world.tick(t, dt, _v, _size);
    _frame.value++;
  }

  int? _zoneAt(Offset p) {
    final a = world.anchors(_size, _v);
    int? best;
    var bd = reach;
    for (var z = 0; z < a.length; z++) {
      for (final q in a[z]) {
        final d = (q - p).distance;
        if (d <= bd) {
          bd = d;
          best = z;
        }
      }
    }
    return best;
  }

  Offset? _nearestAnchor(int z) {
    final a = world.anchors(_size, _v);
    if (z >= a.length || a[z].isEmpty) return null;
    return a[z].reduce((x, y) => (x - _ptr).distance <= (y - _ptr).distance ? x : y);
  }

  bool _key(KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _grab != null) {
      widget.doc.cancel();
      _grab = null;
      HardwareKeyboard.instance.removeHandler(_key);
      setState(() {});
      return true;
    }
    return false;
  }

  void _down(PointerDownEvent e) {
    _ptr = e.localPosition;
    final z = _zoneAt(e.localPosition);
    if (z == null) return;
    final now = (_t * 1000).round(); // the box clock (one frame resolution)
    if (z == _lastZone && now - _lastDownMs < 320 && (e.localPosition - _lastDownAt).distance < 10) {
      widget.doc.reset(world.zoneIds[z]); // double-click = default
      _lastDownMs = -9999;
      return;
    }
    _lastDownMs = now;
    _lastDownAt = e.localPosition;
    _lastZone = z;
    _grab = z;
    _p0 = e.localPosition;
    _v0 = widget.doc.values;
    _moved = false;
    widget.doc.begin();
    HardwareKeyboard.instance.addHandler(_key);
    setState(() {});
  }

  void _move(PointerMoveEvent e) {
    _ptr = e.localPosition;
    final z = _grab;
    if (z == null) return;
    if ((e.localPosition - _p0).distance > 4) _moved = true;
    final k = HardwareKeyboard.instance.isShiftPressed ? .1 : 1.0;
    final p = _p0 + (e.localPosition - _p0) * k;
    widget.doc.setAll(world.drag(z, p, _p0, _v0, _size));
  }

  void _up(PointerEvent e) {
    final z = _grab;
    if (z == null) return;
    if (!_moved && world.tapZones.contains(z)) {
      final m = world.tap(z, _v0);
      if (m != null) widget.doc.setAll(m);
    }
    widget.doc.end();
    _grab = null;
    HardwareKeyboard.instance.removeHandler(_key);
    setState(() => _hot = _zoneAt(e.localPosition));
  }

  @override
  Widget build(BuildContext context) {
    final s = _size;
    return SizedBox(
      width: s.width,
      height: s.height,
      child: MouseRegion(
        cursor: _grab != null ? SystemMouseCursors.grabbing : (_hot != null ? SystemMouseCursors.grab : MouseCursor.defer),
        onHover: (e) {
          _ptr = e.localPosition;
          final z = _zoneAt(e.localPosition);
          if (z != _hot) setState(() => _hot = z);
        },
        onExit: (_) {
          if (_grab == null && _hot != null) setState(() => _hot = null);
        },
        child: Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: _down,
          onPointerMove: _move,
          onPointerUp: _up,
          onPointerCancel: _up,
          child: ClipRRect(
            borderRadius: BorderRadius.circular(4),
            child: DecoratedBox(
              decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20), borderRadius: BorderRadius.circular(4)),
              child: Stack(children: [
                Positioned.fill(child: RepaintBoundary(child: CustomPaint(painter: _WbPainter(this)))),
                Positioned(
                  left: 8,
                  bottom: 5,
                  child: ValueListenableBuilder<int>(
                    valueListenable: _frame,
                    builder: (_, _, _) {
                      final changed = widget.doc.specs.any((s) => (_v[s.id]! - s.def).abs() > 1e-9);
                      return Text(world.readout(_v), maxLines: 1, softWrap: false, style: TextStyle(fontFamily: T.mono, fontSize: 10, height: 1, color: T.ink(changed ? N.g76 : N.g63), decoration: TextDecoration.none));
                    },
                  ),
                ),
              ]),
            ),
          ),
        ),
      ),
    );
  }
}

class _WbPainter extends CustomPainter {
  _WbPainter(this.s) : super(repaint: s._frame);
  final _WbBoxState s;
  @override
  void paint(Canvas canvas, Size size) {
    final f = WbFrame(v: s._v, t: s._t, hot: s._hot, grab: s._grab);
    s.world.paint(canvas, size, f);
    final z = s._grab ?? s._hot;
    if (z != null) {
      final q = s._nearestAnchor(z);
      if (q != null) canvas.drawCircle(q, 10, wbStroke(wbSlot(z).withValues(alpha: .38)));
    }
  }

  @override
  bool shouldRepaint(_WbPainter o) => true;
}
