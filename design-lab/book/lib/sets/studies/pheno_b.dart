// set: Phenomenon B (light and optics). A PARAMETER turned into a MINIATURE OF THE PHENOMENON IT MEANS (owner, 2026-10-02): the earlier drafts were
// "a slider drawn in another shape"; here the number becomes a place / a shape / a bit of physics, and the Inspector row becomes a small toy.
// Six miniatures: Glow star, Refraction ray, Shadow with a carried sun, Depth of field cards, Blur bokeh, Mask feather rub.
// Contracts shared by all six (the owner's hard rules): the control IS the phenomenon (no slider / knob / XY / bar / stepper / dial as the main control);
// the number is secondary (a quiet 10 px mono readout at the edge, g63); no words inside the miniature (the Words addon set to Hidden changes nothing that matters);
// drag acts on the miniature the way the phenomenon behaves; hover lights what can be grabbed (a 1 px mark, no text); Esc during a drag restores;
// Shift = fine (x0.1); double-click resets (on a handle: its values, elsewhere: all); one gesture = one undo step (Cmd/Ctrl+Z, local list);
// hit radius 12 = a 24 px target; motion = springs that settle with a little overshoot, tiny idle life only while the pointer is over the miniature.
// Each miniature reads and writes a small value object (GlowValues, RefractionValues, ShadowValues, DofValues, BlurValues, FeatherValues) with real ranges, defaults and units,
// so a later integration reads / writes the same names. Colour only through Role / N tokens in small marks and 1 px edges.
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/foundation.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../tokens.dart';

part 'pheno_b_glow.dart';
part 'pheno_b_refract.dart';
part 'pheno_b_shadow.dart';
part 'pheno_b_dof.dart';
part 'pheno_b_blur.dart';
part 'pheno_b_feather.dart';

WidgetbookComponent phenoBSet() => WidgetbookComponent(name: 'Phenomenon B', useCases: [
      WidgetbookUseCase(
        name: 'B1 Glow star',
        builder: (c) => onGround(const _PhenoCase(
          caption: 'Glow star: merges radius + intensity + threshold; silhouette: a four-spike sparkle star in a soft halo',
          label: 'Glow',
          rowH: 112,
          above: ('Opacity', '100 %'),
          below: ('Blend', 'Add'),
          make: _makeGlow,
        )),
      ),
      WidgetbookUseCase(
        name: 'B2 Refraction ray',
        builder: (c) => onGround(const _PhenoCase(
          caption: 'Refraction ray: merges index of refraction (one number, shown as the bend); silhouette: a slanted glass slab with a kinked beam through it',
          label: 'Refraction',
          rowH: 112,
          above: ('Roughness', '0.06'),
          below: ('Thickness', '12.0 px'),
          make: _makeRefract,
        )),
      ),
      WidgetbookUseCase(
        name: 'B3 Shadow and sun',
        builder: (c) => onGround(const _PhenoCase(
          caption: 'Shadow and sun: merges direction + distance + softness; silhouette: a small block on a dotted floor with its shadow and a sun on a short tether',
          label: 'Shadow',
          rowH: 120,
          above: ('Opacity', '75 %'),
          below: ('Colour', '#000000'),
          make: _makeShadow,
        )),
      ),
      WidgetbookUseCase(
        name: 'B4 Depth of field',
        builder: (c) => onGround(const _PhenoCase(
          caption: 'Depth of field: merges focus distance + aperture; silhouette: a diagonal file of little photo cards with viewfinder frames, near ones smeared',
          label: 'Focus',
          rowH: 120,
          above: ('Samples', '16'),
          below: ('Bokeh', 'Disc'),
          make: _makeDof,
        )),
      ),
      WidgetbookUseCase(
        name: 'B5 Blur bokeh',
        builder: (c) => onGround(const _PhenoCase(
          caption: 'Blur bokeh: merges radius + blades + direction + streak; silhouette: one bright dot in a translucent polygon that stretches into a streak',
          label: 'Blur',
          rowH: 112,
          above: ('Quality', 'Draft'),
          below: ('Edge', 'Clamp'),
          make: _makeBlur,
        )),
      ),
      WidgetbookUseCase(
        name: 'B6 Mask feather',
        builder: (c) => onGround(const _PhenoCase(
          caption: 'Mask feather: merges softness + expansion; silhouette: a pale pebble-shaped matte on a dotted ground whose rim you rub soft or push out',
          label: 'Mask',
          rowH: 120,
          above: ('Opacity', '100 %'),
          below: ('Invert', 'Off'),
          make: _makeFeather,
        )),
      ),
    ]);

// ---- value objects -------------------------------------------------------------------------------------------------------------

/// One real parameter: id, range, default, unit. [prefix] and [unit] only dress the readout.
class PhSpec {
  const PhSpec(this.id, this.min, this.max, this.def, {this.unit = '', this.prefix = '', this.decimals = 0});
  final String id, unit, prefix;
  final double min, max, def;
  final int decimals;
  double clamp(double x) => x.clamp(min, max).toDouble();
  String fmt(double x) => '$prefix${x.toStringAsFixed(decimals)}$unit';
}

/// The shared base of every miniature's value object: named values with ranges, a gesture = one undo step, Esc restores. A later integration
/// reads [] and calls [set]; nothing here knows about the drawing.
class PhenoValues extends ChangeNotifier {
  PhenoValues(this.specs) : _v = {for (final s in specs) s.id: s.def};
  final List<PhSpec> specs;
  final Map<String, double> _v;
  final List<Map<String, double>> _undo = [];
  Map<String, double>? _snap;

  Iterable<String> get ids => specs.map((s) => s.id);
  PhSpec spec(String id) => specs.firstWhere((s) => s.id == id);
  double operator [](String id) => _v[id]!;
  bool changed(String id) => (_v[id]! - spec(id).def).abs() > 1e-9;
  String fmt(String id) => spec(id).fmt(_v[id]!);

  void set(String id, double x) {
    final nx = spec(id).clamp(x);
    if ((nx - _v[id]!).abs() < 1e-9) return;
    _v[id] = nx;
    notifyListeners();
  }

  void begin() => _snap ??= Map.of(_v);

  /// A gesture is one undo step, and only if it changed something.
  void end() {
    final s = _snap;
    _snap = null;
    if (s != null && !mapEquals(s, _v)) _undo.add(s);
  }

  /// Esc during a drag: everything goes back.
  void cancel() {
    final s = _snap;
    _snap = null;
    if (s == null) return;
    _v
      ..clear()
      ..addAll(s);
    notifyListeners();
  }

  void undo() {
    if (_undo.isEmpty) return;
    _v
      ..clear()
      ..addAll(_undo.removeLast());
    notifyListeners();
  }

  void reset(Iterable<String> which) {
    begin();
    for (final id in which) {
      set(id, spec(id).def);
    }
    end();
  }
}

// ---- the miniature's behaviour -------------------------------------------------------------------------------------------------

class PhRead {
  const PhRead(this.text, this.zones, this.changed);
  final String text;
  final Set<String> zones;
  final bool changed;
}

/// What the canvas tells a miniature each frame.
class PhCtx {
  String? hot;
  bool dragging = false, hover = false;
  double time = 0;
  Offset? ptr;
}

abstract class PhenoSim {
  PhenoSim(this.v);
  final PhenoValues v;

  /// The grabbable zone under [p] (hit radii are 12 = a 24 px target), or null.
  String? zoneAt(Offset p, Size s);
  void down(String zone, Offset p, Size s) {}
  void drag(String zone, Offset p, Offset delta, Size s, {required bool fine, required bool alt});
  void cancelled() {}

  /// What a double-click on [zone] resets (null = empty ground = everything).
  List<String> resetIds(String? zone) => v.ids.toList();

  /// Advance the springs; true while something still moves (or the pointer is over and the idle life runs).
  bool tick(double dt, PhCtx x, Size s);
  void paint(Canvas c, Size s, PhCtx x);
  List<PhRead> readout();
}

/// A spring that settles with a little overshoot (k 600, c 30: a ~120 ms ease with one soft rebound).
class Spr {
  Spr(this.x) : t = x;
  double x, v = 0, t;
  void step(double dt, {double k = 600, double c = 30}) {
    final n = (dt / .004).ceil().clamp(1, 12);
    final h = dt / n;
    for (var i = 0; i < n; i++) {
      v += (k * (t - x) - c * v) * h;
      x += v * h;
    }
  }

  bool moving([double eps = .004]) => (t - x).abs() > eps || v.abs() > eps * 8;
}

/// Fine mode: relative to where the grab began, x0.1.
double _fine(double a, double a0, double v0, bool fine) => fine ? v0 + (a - a0) * .1 : a;

double _wrapPi(double a) {
  var r = a;
  while (r > math.pi) {
    r -= 2 * math.pi;
  }
  while (r < -math.pi) {
    r += 2 * math.pi;
  }
  return r;
}

Color _al(Color c, double a) => c.withValues(alpha: a.clamp(0.0, 1.0).toDouble());
Paint _hair(Color c, [double w = 1]) => Paint()
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..color = c;
Paint _fill(Color c) => Paint()..color = c;

/// The fine ground every miniature sits on: a lattice of tiny dots (like a sensor or a dotted page), so the black is never empty.
void _lattice(Canvas c, Size s, {double pitch = 12, double a = .5}) {
  final p = _fill(_al(N.g26, a));
  final nx = (s.width / pitch).floor(), ny = (s.height / pitch).floor();
  final ox = (s.width - nx * pitch) / 2, oy = (s.height - ny * pitch) / 2;
  for (var i = 0; i <= nx; i++) {
    for (var j = 0; j <= ny; j++) {
      c.drawCircle(Offset(ox + i * pitch, oy + j * pitch), .6, p);
    }
  }
}

/// The warm tone of light: grey in Grey, a touch of Role.key in a palette.
Color get _light => Color.lerp(N.g95, Role.key, .5)!;

// ---- the canvas: pointer, keys, ticker, readout (shared by all six) ------------------------------------------------------------

class PhenoCanvas extends StatefulWidget {
  const PhenoCanvas({super.key, required this.values, required this.make});
  final PhenoValues values;
  final PhenoSim Function(PhenoValues) make;
  @override
  State<PhenoCanvas> createState() => _PhenoCanvasState();
}

class _PhenoCanvasState extends State<PhenoCanvas> with SingleTickerProviderStateMixin {
  late final PhenoSim sim = widget.make(widget.values);
  late final Ticker _tick = createTicker(_onTick);
  final _frame = ValueNotifier<int>(0);
  final _hot = ValueNotifier<String?>(null);
  final _ctx = PhCtx();
  final _focus = FocusNode();
  Duration _last = Duration.zero;
  Size _size = Size.zero;
  String? _down;
  bool _ignore = false;
  Duration? _lastTap;
  Offset? _lastTapAt;

  @override
  void initState() {
    super.initState();
    widget.values.addListener(_wake);
    _wake();
  }

  @override
  void dispose() {
    widget.values.removeListener(_wake);
    _tick.dispose();
    _focus.dispose();
    _frame.dispose();
    _hot.dispose();
    super.dispose();
  }

  void _wake() {
    if (!mounted || _tick.isActive) return;
    _last = Duration.zero;
    _tick.start();
  }

  void _onTick(Duration e) {
    final dt = ((e - _last).inMicroseconds / 1e6).clamp(0.0, .05).toDouble();
    _last = e;
    _ctx.time += dt;
    final more = _size.isEmpty ? false : sim.tick(dt, _ctx, _size);
    _frame.value++;
    if (!more) _tick.stop();
  }

  void _setHot(String? z) {
    _ctx.hot = z;
    if (_hot.value != z) _hot.value = z;
  }

  void _hover(Offset p, bool inside) {
    _ctx.hover = inside;
    _ctx.ptr = inside ? p : null;
    if (!_ctx.dragging) _setHot(inside ? sim.zoneAt(p, _size) : null);
    _wake();
  }

  void _downAt(PointerDownEvent e) {
    if (e.buttons != kPrimaryButton) return;
    final p = e.localPosition, now = e.timeStamp;
    _focus.requestFocus();
    final zone = sim.zoneAt(p, _size);
    if (_lastTap != null && (now - _lastTap!).inMilliseconds < 380 && (p - _lastTapAt!).distance < 8) {
      _lastTap = null;
      widget.values.reset(sim.resetIds(zone));
      return;
    }
    _lastTap = now;
    _lastTapAt = p;
    if (zone == null) return;
    _down = zone;
    _ignore = false;
    _ctx.dragging = true;
    _ctx.ptr = p;
    widget.values.begin();
    sim.down(zone, p, _size);
    _setHot(zone);
    _wake();
  }

  void _move(PointerMoveEvent e) {
    _ctx.ptr = e.localPosition;
    final z = _down;
    if (!_ctx.dragging || _ignore || z == null) return;
    final kb = HardwareKeyboard.instance;
    sim.drag(z, e.localPosition, e.delta, _size, fine: kb.isShiftPressed, alt: kb.isAltPressed);
    _wake();
  }

  void _up(PointerEvent e) {
    if (_ctx.dragging) {
      if (e is PointerCancelEvent && !_ignore) {
        widget.values.cancel();
        sim.cancelled();
      } else {
        widget.values.end();
      }
    }
    _ctx.dragging = false;
    _down = null;
    _ignore = false;
    _setHot(_ctx.hover ? sim.zoneAt(e.localPosition, _size) : null);
    _wake();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    if (e.logicalKey == LogicalKeyboardKey.escape && _ctx.dragging) {
      widget.values.cancel();
      sim.cancelled();
      _ignore = true;
      _wake();
      return KeyEventResult.handled;
    }
    final kb = HardwareKeyboard.instance;
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (kb.isMetaPressed || kb.isControlPressed) && !_ctx.dragging) {
      widget.values.undo();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        _size = box.biggest;
        return Focus(
          focusNode: _focus,
          onKeyEvent: _key,
          child: ValueListenableBuilder<String?>(
            valueListenable: _hot,
            builder: (context, hot, _) => MouseRegion(
              cursor: hot == null ? SystemMouseCursors.basic : (_ctx.dragging ? SystemMouseCursors.grabbing : SystemMouseCursors.grab),
              onEnter: (e) => _hover(e.localPosition, true),
              onHover: (e) => _hover(e.localPosition, true),
              onExit: (e) => _hover(e.localPosition, false),
              child: Listener(
                behavior: HitTestBehavior.opaque,
                onPointerDown: _downAt,
                onPointerMove: _move,
                onPointerUp: _up,
                onPointerCancel: _up,
                child: Stack(fit: StackFit.expand, children: [
                  CustomPaint(painter: _PhPainter(sim, _ctx, Listenable.merge([_frame, Role.changes]))),
                  Positioned(
                    left: 7,
                    bottom: 5,
                    right: 4,
                    child: IgnorePointer(
                      child: ListenableBuilder(
                        listenable: Listenable.merge([widget.values, _hot]),
                        builder: (context, _) => Wrap(spacing: 8, children: [
                          for (final r in sim.readout())
                            Text(r.text,
                                maxLines: 1,
                                style: T.value(hot != null && r.zones.contains(hot) ? N.g95 : (r.changed ? Role.of(N.g76, Role.changed) : N.g63)).copyWith(fontSize: 10)),
                        ]),
                      ),
                    ),
                  ),
                ]),
              ),
            ),
          ),
        );
      });
}

class _PhPainter extends CustomPainter {
  _PhPainter(this.sim, this.ctx, Listenable repaint) : super(repaint: repaint);
  final PhenoSim sim;
  final PhCtx ctx;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    sim.paint(canvas, size, ctx);
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _PhPainter old) => true;
}

// ---- the use case: caption outside, the large miniature (320x200) and the same one inside a 282 px Inspector row --------------

class _PhenoCase extends StatefulWidget {
  const _PhenoCase({required this.caption, required this.label, required this.rowH, required this.above, required this.below, required this.make});
  final String caption, label;
  final double rowH;
  final (String, String) above, below;
  final (PhenoValues, PhenoSim Function(PhenoValues)) Function() make;
  @override
  State<_PhenoCase> createState() => _PhenoCaseState();
}

class _PhenoCaseState extends State<_PhenoCase> {
  late final (PhenoValues, PhenoSim Function(PhenoValues)) _m = widget.make();

  Widget _well(double? w, double h) => Container(
        width: w,
        height: h,
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: ClipRRect(borderRadius: BorderRadius.circular(3), child: PhenoCanvas(values: _m.$1, make: _m.$2)),
      );

  Widget _num((String, String) r) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Row(children: [
          SizedBox(width: 62, child: Text(r.$1, maxLines: 1, style: T.label(N.g63))),
          const SizedBox(width: 6),
          Expanded(
            child: Container(
              height: 22,
              alignment: Alignment.centerRight,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
              child: Text(r.$2, style: T.value(N.g91)),
            ),
          ),
        ]),
      );

  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Text(widget.caption, style: T.label(N.g63)),
        const SizedBox(height: 12),
        Wrap(spacing: 24, runSpacing: 16, crossAxisAlignment: WrapCrossAlignment.start, children: [
          _well(320, 200),
          Container(
            width: 282,
            padding: const EdgeInsets.symmetric(vertical: 4),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
            child: Column(mainAxisSize: MainAxisSize.min, children: [
              _num(widget.above),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  SizedBox(width: 62, child: Padding(padding: const EdgeInsets.only(top: 5), child: Text(widget.label, maxLines: 1, style: T.label(N.g63)))),
                  const SizedBox(width: 6),
                  Expanded(child: _well(null, widget.rowH)),
                ]),
              ),
              _num(widget.below),
            ]),
          ),
        ]),
      ]);
}
