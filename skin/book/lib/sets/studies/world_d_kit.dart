// World D, shared plumbing. A "world" is one small dedicated picture for one Effect / Relation / Property group (OP-1 style): up to four grab
// zones, each tied to a colour slot A..D (blue / green / white / orange = Fam.stagger / Fam.along / N.g95 / Fam.follow), an idle life that is
// subtle, a quiet 10 px mono readout at the edge, and a use-case knob "Driven" (Off | Wiggle | Pulse) that moves the parameters from outside so the
// picture itself is seen moving. The numbers live in a [WdDoc] (the small value object a host would read and write); a world holds no arithmetic
// of its own beyond drawing (one gesture = begin..end = one undo step; Esc restores; Shift = fine; double-click resets; hit areas >= 24 px).
import 'dart:math' as math;
import 'dart:ui' show PathMetric;

import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';

// ---- numbers ---------------------------------------------------------------------------------------------------------------------------

double wdClamp(double v, double a, double b) => v < a ? a : (v > b ? b : v);
double wdLerp(double a, double b, double t) => a + (b - a) * t;
double wdSmooth(double t) => t * t * (3 - 2 * t);

/// A smooth deterministic wobble in about -1..1; [u] radians, [seed] separates channels.
double wdNoise(double u, double seed) =>
    math.sin(u + seed * 7.1) * .5 + math.sin(u * 2.31 + seed * 3.7) * .3 + math.sin(u * 4.73 + seed * 1.3) * .2;

String wdNum(double v) => v == v.roundToDouble() ? v.toInt().toString() : v.toString();

// ---- colours ----------------------------------------------------------------------------------------------------------------------------

/// The four colour slots of the four grabs, identical in every world: A blue, B green, C white, D orange.
Color wdSlot(int z) => switch (z) { 0 => Fam.stagger.c, 1 => Fam.along.c, 2 => N.g95, _ => Fam.follow.c };

/// A world's own ink: a plain grey line.
Paint wdStroke(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;

Paint wdFill(Color c) => Paint()..color = c;

/// Grey ramp 0..1 between the ground and near-white (blend maths of the Opacity / Blend world draws in it).
Color wdGrey(double b) => Color.lerp(N.g10, N.g95, wdClamp(b, 0, 1))!;

Rect wdFat(Rect r, [double m = 24]) => Rect.fromCenter(center: r.center, width: math.max(r.width, m), height: math.max(r.height, m));

void wdDashPath(Canvas c, Path p, Paint paint, {double on = 2, double off = 3}) {
  for (final PathMetric m in p.computeMetrics()) {
    for (double s = 0; s < m.length; s += on + off) {
      c.drawPath(m.extractPath(s, math.min(s + on, m.length)), paint);
    }
  }
}

void wdDashLine(Canvas c, Offset a, Offset b, Paint p, {double on = 2, double off = 3}) {
  final d = b - a, len = d.distance;
  if (len < .5) return;
  final u = d / len;
  for (double s = 0; s < len; s += on + off) {
    c.drawLine(a + u * s, a + u * math.min(s + on, len), p);
  }
}

// ---- the value-object ---------------------------------------------------------------------------------------------------------------

enum WdDrive {
  off('Off'),
  wiggle('Wiggle'),
  pulse('Pulse');

  const WdDrive(this.label);
  final String label;
}

/// One real named parameter. [range] / [defText] override the printed range and default (enum and toggle parameters).
class WdSpec {
  const WdSpec(this.id, this.name, this.min, this.max, this.def,
      {this.unit = '', this.digits = 2, this.range, this.defText, this.driven = true, this.amp = .14});
  final String id, name, unit;
  final double min, max, def, amp;
  final int digits;
  final String? range, defText;
  final bool driven;

  String fmt(double v) {
    var s = v.toStringAsFixed(digits);
    if (v.abs() < 1 && digits > 0) s = s.replaceFirst('0.', '.');
    return unit.isEmpty ? s : '$s$unit';
  }

  String get cap => '$name ${range ?? '${wdNum(min)}–${wdNum(max)}${unit.isEmpty ? '' : ' $unit'}'} (${defText ?? '${wdNum(def)}${unit.isEmpty ? '' : ' $unit'}'})';
}

/// The values of one world: [base] is what the user set, [eff] is what is drawn (base + the outside drive of the Driven knob).
class WdDoc extends ChangeNotifier {
  WdDoc(this.specs) : _v = {for (final s in specs) s.id: s.def};
  final List<WdSpec> specs;
  final Map<String, double> _v;
  final List<Map<String, double>> _undo = [];
  Map<String, double>? _snap;
  WdDrive drive = WdDrive.off;
  double time = 0, dt = 0;
  final clock = ValueNotifier<int>(0);

  void tick(double d) {
    dt = d;
    time += d;
    clock.value++;
  }

  WdSpec spec(String id) => specs.firstWhere((s) => s.id == id);
  double base(String id) => _v[id]!;
  bool changed(String id) => (_v[id]! - spec(id).def).abs() > 1e-9;
  Map<String, double> get values => Map.unmodifiable(_v);

  double eff(String id) {
    final s = spec(id), b = _v[id]!;
    if (drive == WdDrive.off || !s.driven) return b;
    final k = specs.indexOf(s), span = s.max - s.min;
    double o;
    if (drive == WdDrive.wiggle) {
      o = wdNoise(time * 2.1, k + 1.0) * s.amp * span * 1.3;
    } else {
      final ph = (time / 1.7 + k * .19) % 1.0;
      final env = ph < .05 ? ph / .05 : math.exp(-(ph - .05) * 4.2);
      o = (k.isEven ? 1 : -1) * env * s.amp * 2.2 * span;
    }
    if (b + o > s.max || b + o < s.min) o = -o;
    return wdClamp(b + o, s.min, s.max);
  }

  String show(String id) => spec(id).fmt(eff(id));

  void set(String id, double x) {
    final s = spec(id), n = wdClamp(x, s.min, s.max);
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

// ---- the world --------------------------------------------------------------------------------------------------------------------

/// What the box knows about the pointer, and the eased emphasis of each of the four zones (0.72 idle, 1 hovered / grabbed, .28 while another is grabbed).
class WdView {
  Offset? ptr;
  int? hot, grab;
  final a = List<double>.filled(4, .72);
  bool get hover => ptr != null;
}

typedef WdR = (String text, bool changed);

abstract class WdMini {
  WdMini(this.doc);
  final WdDoc doc;
  final view = WdView();

  /// Parameter ids of each zone A..D (double-click on a zone resets these; elsewhere resets everything).
  List<List<String>> get zoneIds;

  /// The drawing area: the frame minus a thin margin and the bottom strip of the readout.
  Rect art(Size s) => Rect.fromLTRB(10, 10, s.width - 10, s.height - 22);

  List<WdR> readout();

  /// The slot colour (0..3) of each readout entry; the 3 px dot in front of a number is the key between number and picture.
  List<int> get readSlots => const [0, 1, 2, 3];
  int? zoneAt(Offset p, Size s);
  void down(int z, Offset p, Size s) {}
  void move(int z, Offset p, Offset d, bool fine, Size s) {}
  void up(int z) {}
  void cancelled(int z) {}
  void step(double dt, Size s) {}
  void paint(Canvas c, Size s);

  /// A zone's colour at its current emphasis.
  Color zc(int z, [double k = 1]) => wdSlot(z).withValues(alpha: wdClamp(view.a[z] * k, 0, 1));

  /// 1 px hairline of the grabbable shape while it is hovered or held (no text).
  void hint(Canvas c, Rect r, int z) {
    if (view.hot == z || view.grab == z) {
      c.drawRRect(RRect.fromRectAndRadius(r.deflate(.5), const Radius.circular(3)), wdStroke(wdSlot(z).withValues(alpha: .28)));
    }
  }

  void reset(int? z) => doc.resetIds(z == null ? doc.specs.map((s) => s.id) : zoneIds[z]);
}

class _WdPainter extends CustomPainter {
  _WdPainter(this.m, Listenable l) : super(repaint: l);
  final WdMini m;
  @override
  void paint(Canvas canvas, Size size) => m.paint(canvas, size);
  @override
  bool shouldRepaint(_WdPainter old) => true;
}

/// One world at a fixed size: frame, painter, pointer plumbing and the quiet readout.
class WdBox extends StatefulWidget {
  const WdBox({super.key, required this.doc, required this.make, required this.size});
  final WdDoc doc;
  final WdMini Function(WdDoc) make;
  final Size size;
  @override
  State<WdBox> createState() => _WdBoxState();
}

class _WdBoxState extends State<WdBox> {
  late final WdMini mini = widget.make(widget.doc);
  final _focus = FocusNode();
  bool _dead = false;
  Duration? _lastDown;
  Offset _lastDownP = Offset.zero;
  MouseCursor _cursor = SystemMouseCursors.basic;

  WdView get v => mini.view;

  @override
  void initState() {
    super.initState();
    widget.doc.clock.addListener(_onClock);
  }

  @override
  void dispose() {
    widget.doc.clock.removeListener(_onClock);
    _focus.dispose();
    super.dispose();
  }

  void _onClock() {
    final dt = widget.doc.dt;
    final k = 1 - math.exp(-dt * 16); // ~120 ms ease-out of the emphasis
    for (var z = 0; z < 4; z++) {
      final t = v.grab != null ? (v.grab == z ? 1.0 : .28) : (v.hot == z ? 1.0 : .72);
      v.a[z] += (t - v.a[z]) * k;
    }
    mini.step(dt, widget.size);
  }

  void _down(PointerDownEvent e) {
    if (e.buttons != kPrimaryButton) return;
    final p = e.localPosition, z = mini.zoneAt(p, widget.size);
    _focus.requestFocus();
    final dbl = _lastDown != null && e.timeStamp - _lastDown! < const Duration(milliseconds: 350) && (p - _lastDownP).distance < 8;
    _lastDown = dbl ? null : e.timeStamp;
    _lastDownP = p;
    if (dbl) {
      mini.reset(z);
      return;
    }
    if (z == null) return;
    _dead = false;
    widget.doc.begin();
    v.ptr = p;
    v.grab = z;
    mini.down(z, p, widget.size);
    setState(() => _cursor = SystemMouseCursors.grabbing);
  }

  void _move(PointerMoveEvent e) {
    final z = v.grab;
    if (z == null || _dead) return;
    v.ptr = e.localPosition;
    mini.move(z, e.localPosition, e.localDelta, HardwareKeyboard.instance.isShiftPressed, widget.size);
  }

  void _up(PointerEvent e) {
    final z = v.grab;
    if (z == null) return;
    if (!_dead) {
      mini.up(z);
      widget.doc.end();
    }
    v.grab = null;
    _dead = false;
    _hover(e.localPosition);
  }

  void _hover(Offset p) {
    v.ptr = p;
    v.hot = mini.zoneAt(p, widget.size);
    final want = v.hot == null ? SystemMouseCursors.basic : SystemMouseCursors.grab;
    if (want != _cursor) setState(() => _cursor = want);
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final z = v.grab;
    if (e.logicalKey == LogicalKeyboardKey.escape && z != null) {
      widget.doc.cancel();
      mini.cancelled(z);
      v.grab = null;
      _dead = true;
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      widget.doc.undo();
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
                    Positioned.fill(child: CustomPaint(painter: _WdPainter(mini, Listenable.merge([widget.doc.clock, widget.doc, Role.changes])))),
                    Positioned(
                      right: 8,
                      bottom: 5,
                      child: IgnorePointer(
                        child: ListenableBuilder(
                          listenable: Listenable.merge([widget.doc, widget.doc.clock]),
                          builder: (context, _) {
                            final r = mini.readout(), sl = mini.readSlots;
                            return Row(mainAxisSize: MainAxisSize.min, children: [
                              for (var i = 0; i < r.length; i++) ...[
                                Container(width: 3, height: 3, decoration: BoxDecoration(color: wdSlot(sl[i]), shape: BoxShape.circle)),
                                const SizedBox(width: 4),
                                Text('${r[i].$1}  ', style: T.value(r[i].$2 ? Role.of(N.g76, Role.changed) : N.g63).copyWith(fontSize: 10)),
                              ],
                            ]);
                          },
                        ),
                      ),
                    ),
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

// ---- the use case frame -----------------------------------------------------------------------------------------------------------

/// A plain neighbour row of the Inspector: a label and a number cell.
class WdNumRow extends StatelessWidget {
  const WdNumRow(this.label, this.value, {super.key});
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

/// One use case: the caption above the frame (outside it, one line), the large world (320x200), and the same world in a 282 px Inspector row
/// between two plain number rows. Both share one [WdDoc] (a drag in one is seen in the other) and one clock; [drive] is the Driven knob.
class WdStory extends StatefulWidget {
  const WdStory(
      {super.key, required this.name, required this.silhouette, required this.specs, required this.make, required this.rowH, required this.above, required this.below, required this.drive});
  final String name, silhouette;
  final List<WdSpec> specs;
  final WdMini Function(WdDoc) make;
  final double rowH;
  final (String, String) above, below;
  final WdDrive drive;
  @override
  State<WdStory> createState() => _WdStoryState();
}

class _WdStoryState extends State<WdStory> with SingleTickerProviderStateMixin {
  late final WdDoc doc = WdDoc(widget.specs)..drive = widget.drive;
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
    doc.tick(dt);
  }

  @override
  void didUpdateWidget(WdStory old) {
    super.didUpdateWidget(old);
    doc.drive = widget.drive;
  }

  @override
  void dispose() {
    _tk.dispose();
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g07,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text('${widget.name}  ·  ${[for (final s in widget.specs) s.cap].join('  ·  ')}  ·  silhouette: ${widget.silhouette}', style: T.label(N.g63)),
            const SizedBox(height: 14),
            Wrap(spacing: 40, runSpacing: 28, crossAxisAlignment: WrapCrossAlignment.start, children: [
              WdBox(doc: doc, make: widget.make, size: const Size(320, 200)),
              SizedBox(
                width: 282,
                child: Column(mainAxisSize: MainAxisSize.min, children: [
                  WdNumRow(widget.above.$1, widget.above.$2),
                  const SizedBox(height: 4),
                  WdBox(doc: doc, make: widget.make, size: Size(282, widget.rowH)),
                  const SizedBox(height: 4),
                  WdNumRow(widget.below.$1, widget.below.$2),
                ]),
              ),
            ]),
          ]),
        ),
      );
}
