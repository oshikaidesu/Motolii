// World A, shared plumbing: a small WORLD per Effect / Relation (OP-1 style). Four colour-matched grab zones A-D, one value object, one
// 'Driven' knob (Off | Wiggle | Pulse) that moves the parameters from outside so the graphic is seen moving. Numbers are a quiet 10 px readout.
// Contracts: drag is zone-specific, hover = the grabbable mark brightens (no text, no geometry change), Esc restores, Shift = fine (x0.1),
// double-click resets the zone, hit areas are >= 24 px (distance thresholds >= 12 px), 1 gesture = 1 undo step (local list, never shown).
import 'dart:math' as math;

import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';

// ---- numbers ---------------------------------------------------------------------------------------------------------------------

double waClamp(double v, double a, double b) => v < a ? a : (v > b ? b : v);
double waLerp(double a, double b, double t) => a + (b - a) * t;
double waSmooth(double t) {
  final x = waClamp(t, 0, 1);
  return x * x * (3 - 2 * x);
}

/// A smooth deterministic wobble in -1..1; [u] radians, [seed] separates channels.
double waNoise(double u, double seed) => math.sin(u + seed * 7.1) * .5 + math.sin(u * 2.31 + seed * 3.7) * .3 + math.sin(u * 4.73 + seed * 1.3) * .2;

/// A deterministic pseudo-random number in 0..1 from an integer.
double waRnd(int n) {
  final x = math.sin(n * 12.9898 + 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

Offset waPolar(Offset c, double r, double a) => Offset(c.dx + r * math.cos(a), c.dy + r * math.sin(a));
Offset waUnit(Offset v) {
  final d = v.distance;
  return d < 1e-9 ? const Offset(1, 0) : v / d;
}

Paint waStroke(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint waFill(Color c) => Paint()..color = c;

/// An arc of a circle as a stroked path (angles in radians).
void waArc(Canvas cv, Offset c, double r, double a0, double a1, Paint p) =>
    cv.drawArc(Rect.fromCircle(center: c, radius: r), a0, a1 - a0, false, p);

// ---- colours: the four fixed slots (R2 / R3) ----------------------------------------------------------------------------------------

/// A = Fam.stagger (blue), B = Fam.along (green), C = N.g95 (white), D = Fam.follow (orange). Same slot, same position, in every world.
Color waSlot(int z) => switch (z) { 0 => Role.linkedFor(Fam.stagger), 1 => Role.linkedFor(Fam.along), 2 => N.g95, _ => Role.linkedFor(Fam.follow) };

/// Hover / grab accent: quiet white in Grey, the selection hue in a palette.
Color get waHot => Role.of(N.g95, Role.selected);

// ---- the value-object ---------------------------------------------------------------------------------------------------------------

/// One real, named parameter: id, range, default, unit. [step] = an integer that is not wiggled (it hops on a Pulse instead).
class WaSpec {
  const WaSpec(this.id, this.min, this.max, this.def, {this.unit = '', this.digits = 2, this.step = false});
  final String id, unit;
  final double min, max, def;
  final int digits;
  final bool step;

  String fmt(double v) {
    var s = v.toStringAsFixed(digits);
    if (v.abs() < 1 && digits > 0) s = s.replaceFirst('0.', '.');
    return '$s$unit';
  }
}

/// How the parameters are driven from outside, the way OP-1 shows modulation.
enum WaDriven { off, wiggle, pulse }

/// The values of one world. A world never does arithmetic on values it did not draw: it sends 'what was grabbed' to here.
/// [eff] = the value as the picture shows it (base + the current drive); [] = the base the user set.
class WaDoc extends ChangeNotifier {
  WaDoc(this.specs) : _v = {for (final s in specs) s.id: s.def};
  final List<WaSpec> specs;
  final Map<String, double> _v;
  final List<Map<String, double>> _undo = [];
  Map<String, double>? _snap;
  final Set<String> suspended = {}; // ids under the finger: the drive is off for them so the picture does not fight the hand

  double operator [](String id) => _v[id]!;
  WaSpec spec(String id) => specs.firstWhere((s) => s.id == id);
  Map<String, double> get values => Map.unmodifiable(_v);
  bool changed(String id) => (_v[id]! - spec(id).def).abs() > 1e-9;
  bool changedAny(Iterable<String> ids) => ids.any(changed);

  static int _h(String id) => id.codeUnits.fold(7, (a, b) => (a * 31 + b) & 0xFFFFFF);

  /// The drive: Wiggle = a smooth drift around the base; Pulse = a shared beat (every 1.7 s) that pushes each value toward its roomier side.
  double eff(String id, double t, WaDriven mode) {
    final s = spec(id), b = _v[id]!;
    if (mode == WaDriven.off || suspended.contains(id)) return b;
    final span = s.max - s.min, h = _h(id);
    if (s.step) {
      if (mode != WaDriven.pulse) return b;
      final k = (t / 1.7).floor();
      final n = (s.max - s.min + 1).round();
      return s.min + ((b - s.min).round() + k) % n;
    }
    if (mode == WaDriven.wiggle) return waClamp(b + waNoise(t * 1.25 + (h % 97) * .31, (h % 13).toDouble()) * span * .17, s.min, s.max);
    final f = (t / 1.7) % 1.0;
    final env = f < .07 ? f / .07 : math.exp(-(f - .07) * 3.6);
    final dir = (b - s.min) > (s.max - b) ? -1.0 : 1.0;
    return waClamp(b + dir * env * span * .30, s.min, s.max);
  }

  void set(String id, double x) {
    final s = spec(id);
    final n = waClamp(s.step ? x.roundToDouble() : x, s.min, s.max);
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

// ---- what a painter and a hit test see ------------------------------------------------------------------------------------------------

class WaCtx {
  const WaCtx(this.doc, this.t, this.mode, this.hot, this.grab);
  final WaDoc doc;
  final double t;
  final WaDriven mode;
  final int? hot, grab;

  /// The value as shown right now.
  double v(String id) => doc.eff(id, t, mode);

  /// Mark colour of zone [z]: its slot colour; while another zone is held it drops to g38 (R2).
  Color col(int z) => grab != null && grab != z ? N.g38 : waSlot(z);

  /// The mark of [z] is hovered or held.
  bool hl(int z) => hot == z || grab == z;

  /// A zone's mark: stroke colour with the quiet alpha, brighter when hot.
  Paint mark(int z, [double w = 1]) => waStroke(col(z).withValues(alpha: hl(z) ? 1 : .82), hl(z) ? w + .5 : w);
}

/// Pick the nearest candidate (zone, distance, threshold) that is inside its threshold (distance / threshold, so a big zone does not beat a close small one).
int? waPick(List<(int, double, double)> c) {
  int? best;
  var bd = 2.0;
  for (final (z, d, th) in c) {
    if (d > th) continue;
    final r = d / th;
    if (r < bd) {
      bd = r;
      best = z;
    }
  }
  return best;
}

// ---- a world ------------------------------------------------------------------------------------------------------------------------

abstract class WaWorld {
  WaWorld(this.doc);
  final WaDoc doc;
  int? hot, grab;

  /// The ids each zone writes (A..D); used for the drive suspension and the double-click reset.
  List<List<String>> get zoneIds;

  /// The drawing area: the frame minus the strip of the readout (bottom).
  Rect art(Size s) => Rect.fromLTRB(8, 8, s.width - 8, s.height - 22);

  /// Zone under [p], or null. Every zone is >= 24 px wide (threshold >= 12 px).
  int? zoneAt(Offset p, Size s, WaCtx x);

  /// Where the held mark sits now (so the grab does not jump); null = the pointer itself.
  Offset? anchor(int z, Offset ptr, Size s, WaCtx x) => null;

  void begin(int z, Offset vp, Size s, WaCtx x) {}
  void drag(int z, Offset vp, Size s, WaCtx x);
  void tap(int z, Size s, WaCtx x) {}
  void paint(Canvas c, Size s, WaCtx x);

  /// The four quiet readout strings, A..D, from the shown values.
  List<String> readout(WaCtx x);

  void set(String id, double v) => doc.set(id, v);
}

// ---- the box ------------------------------------------------------------------------------------------------------------------------

typedef WaMake = WaWorld Function(WaDoc doc);

class WaBox extends StatefulWidget {
  const WaBox({super.key, required this.doc, required this.make, required this.size, required this.mode});
  final WaDoc doc;
  final WaMake make;
  final Size size;
  final WaDriven mode;

  @override
  State<WaBox> createState() => _WaBoxState();
}

class _WaBoxState extends State<WaBox> with SingleTickerProviderStateMixin {
  late final WaWorld world = widget.make(widget.doc);
  late final Ticker _tk;
  final _clock = ValueNotifier<double>(0);
  final _focus = FocusNode();
  Offset _vp = Offset.zero;
  bool _cancelled = false;
  double _moved = 0;
  int _lastZone = -1;
  Duration _lastAt = Duration.zero;
  MouseCursor _cursor = MouseCursor.defer;

  @override
  void initState() {
    super.initState();
    _tk = createTicker((d) => _clock.value = d.inMicroseconds / 1e6)..start();
  }

  @override
  void dispose() {
    _tk.dispose();
    _clock.dispose();
    _focus.dispose();
    super.dispose();
  }

  Size get _s => widget.size;
  WaCtx _ctx() => WaCtx(widget.doc, _clock.value, widget.mode, world.hot, world.grab);

  void _setHot(int? z) {
    world.hot = z;
    final c = z == null ? MouseCursor.defer : (world.grab != null ? SystemMouseCursors.grabbing : SystemMouseCursors.grab);
    if (c != _cursor) setState(() => _cursor = c);
  }

  void _down0(PointerDownEvent e) {
    if (e.buttons != kPrimaryButton) return;
    _focus.requestFocus();
    final z = world.zoneAt(e.localPosition, _s, _ctx());
    if (z != null && z == _lastZone && e.timeStamp - _lastAt < const Duration(milliseconds: 350)) {
      widget.doc.resetIds(world.zoneIds[z]);
      _lastZone = -1;
      _cancelled = true;
      return;
    }
    _lastZone = z ?? -1;
    _lastAt = e.timeStamp;
    _cancelled = false;
    if (z == null) return;
    final c = _ctx();
    widget.doc.suspended..clear()..addAll(world.zoneIds[z]);
    widget.doc.begin();
    world.grab = z;
    _vp = world.anchor(z, e.localPosition, _s, c) ?? e.localPosition;
    _moved = 0;
    world.begin(z, _vp, _s, c);
    _setHot(z);
  }

  void _move(PointerMoveEvent e) {
    final z = world.grab;
    if (z == null || _cancelled) return;
    _moved += e.delta.distance;
    _vp += e.delta * (HardwareKeyboard.instance.isShiftPressed ? .1 : 1.0);
    world.drag(z, _vp, _s, _ctx());
  }

  void _up(PointerEvent e) {
    final z = world.grab;
    if (z != null && !_cancelled && _moved < 3) world.tap(z, _s, _ctx());
    widget.doc.end();
    widget.doc.suspended.clear();
    world.grab = null;
    _setHot(world.zoneAt(e.localPosition, _s, _ctx()));
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && world.grab != null) {
      widget.doc.cancel();
      widget.doc.suspended.clear();
      world.grab = null;
      _cancelled = true;
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final w = world;
    return Focus(
      focusNode: _focus,
      onKeyEvent: _key,
      child: SizedBox.fromSize(
        size: _s,
        child: ClipRRect(
          borderRadius: BorderRadius.circular(3),
          child: DecoratedBox(
            decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g26), borderRadius: BorderRadius.circular(3)),
            child: MouseRegion(
              cursor: _cursor,
              onHover: (e) => _setHot(w.grab ?? w.zoneAt(e.localPosition, _s, _ctx())),
              onExit: (_) {
                if (w.grab == null) _setHot(null);
              },
              child: Listener(
                behavior: HitTestBehavior.opaque,
                onPointerDown: _down0,
                onPointerMove: _move,
                onPointerUp: _up,
                onPointerCancel: _up,
                child: Stack(children: [
                  Positioned.fill(
                    child: CustomPaint(painter: _WaPainter(w, widget.doc, () => widget.mode, _clock)),
                  ),
                  Positioned(left: 9, right: 9, bottom: 5, child: _Readout(w, widget.doc, () => widget.mode, _clock)),
                ]),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _WaPainter extends CustomPainter {
  _WaPainter(this.w, this.doc, this.mode, this.clock) : super(repaint: Listenable.merge([clock, doc]));
  final WaWorld w;
  final WaDoc doc;
  final WaDriven Function() mode;
  final ValueNotifier<double> clock;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    w.paint(canvas, size, WaCtx(doc, clock.value, mode(), w.hot, w.grab));
    canvas.restore();
  }

  @override
  bool shouldRepaint(_WaPainter old) => true;
}

/// The quiet numbers: 10 px mono, g63, a 3 px dot in the zone's slot colour in front of each (the dot is the key between number and picture).
class _Readout extends StatelessWidget {
  const _Readout(this.w, this.doc, this.mode, this.clock);
  final WaWorld w;
  final WaDoc doc;
  final WaDriven Function() mode;
  final ValueNotifier<double> clock;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([clock, doc, Role.changes, T.hidden]),
        builder: (_, _) {
          final x = WaCtx(doc, clock.value, mode(), w.hot, w.grab);
          final r = w.readout(x);
          return Row(mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
            for (var z = 0; z < 4; z++)
              Row(mainAxisSize: MainAxisSize.min, children: [
                Container(width: 3, height: 3, decoration: BoxDecoration(color: x.col(z), shape: BoxShape.circle)),
                const SizedBox(width: 4),
                Text(r[z],
                    maxLines: 1,
                    style: TextStyle(
                        fontFamily: T.mono,
                        fontSize: 10,
                        height: 1,
                        decoration: TextDecoration.none,
                        color: T.ink(doc.changedAny(w.zoneIds[z]) ? Role.changed : (x.grab != null && x.grab != z ? N.g56 : N.g63)))),
              ]),
          ]);
        },
      );
}

// ---- the two frames of one world: large, and an Inspector row between two plain number rows ------------------------------------------------

class WaPair extends StatefulWidget {
  const WaPair({super.key, required this.make, required this.specs, required this.rowHeight, required this.mode, required this.caption, this.large = const Size(320, 200)});
  final WaMake make;
  final List<WaSpec> specs;
  final double rowHeight;
  final WaDriven mode;
  final String caption;
  final Size large;

  @override
  State<WaPair> createState() => _WaPairState();
}

class _WaPairState extends State<WaPair> {
  late final WaDoc doc = WaDoc(widget.specs);

  @override
  Widget build(BuildContext context) {
    Widget numRow(String l, String v) => Container(
          height: 28,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g20))),
          child: Row(children: [
            Text(l, style: T.label()),
            const Spacer(),
            Text(v, style: T.value(N.g91)),
          ]),
        );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Wrap(spacing: 40, runSpacing: 24, crossAxisAlignment: WrapCrossAlignment.center, children: [
        WaBox(doc: doc, make: widget.make, size: widget.large, mode: widget.mode),
        // the Inspector row: 282 px wide, the world between two plain number rows
        Container(
          width: 282,
          color: N.g13,
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            numRow('Opacity', '100%'),
            Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: WaBox(doc: doc, make: widget.make, size: Size(282, widget.rowHeight), mode: widget.mode)),
            numRow('Blend', 'Normal'),
          ]),
        ),
      ]),
      const SizedBox(height: 14),
      Text(widget.caption, maxLines: 1, softWrap: false, style: T.label(N.g63)),
    ]);
  }
}

String waRange(WaSpec s) {
  String f(double v) => s.fmt(v).replaceFirst('.00', '');
  return '${f(s.min)}-${f(s.max)}';
}
