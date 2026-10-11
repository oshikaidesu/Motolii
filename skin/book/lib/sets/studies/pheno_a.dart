// set: Phenomenon A (scatter family). A PARAMETER becomes a MINIATURE OF THE PHENOMENON IT MEANS: the control IS the picture (a flock, a fan of cards,
// a chain of copies, a lattice, a field with a region, a well with dust). Not a slider drawn in another shape.
// Frame of this file: the value object (PhenoParam / PhenoValues / PhenoController), the interaction shell (PhenoView), and the use-case frames.
// Each miniature lives in its own part file (pheno_a_*.dart): a PhenoSpec (what the numbers are, where you can grab, what a drag does) and a PhenoSim (the little physics + the drawing).
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../tokens.dart';

part 'pheno_a_scatter.dart';
part 'pheno_a_stagger.dart';
part 'pheno_a_repeat.dart';
part 'pheno_a_grid.dart';
part 'pheno_a_falloff.dart';
part 'pheno_a_gravity.dart';

// ---------------------------------------------------------------- value object

/// One real, named parameter: its range, default and unit. A later integration reads / writes exactly these.
class PhenoParam {
  const PhenoParam(this.id, this.name, this.min, this.max, this.def, {this.unit = '', this.prefix = '', this.digits = 0});
  final String id, name, unit, prefix;
  final double min, max, def;
  final int digits;

  double clamp(double v) {
    final c = v.clamp(min, max).toDouble();
    if (digits == 0) return c.roundToDouble();
    return double.parse(c.toStringAsFixed(digits));
  }

  String fmt(double v) => '$prefix${v.toStringAsFixed(digits)}$unit';
}

/// The current values of one miniature, immutable. [] reads by id; [with_] returns a clamped copy.
class PhenoValues {
  const PhenoValues(this.params, this._v);
  factory PhenoValues.defaults(List<PhenoParam> ps) => PhenoValues(ps, {for (final p in ps) p.id: p.def});
  final List<PhenoParam> params;
  final Map<String, double> _v;

  double operator [](String id) => _v[id]!;
  Map<String, double> toMap() => Map.of(_v);
  PhenoValues with_(Map<String, double> m) => PhenoValues(params, {
        for (final p in params) p.id: m.containsKey(p.id) ? p.clamp(m[p.id]!) : _v[p.id]!,
      });
  bool changed(String id) => params.firstWhere((p) => p.id == id).def != _v[id];

  @override
  bool operator ==(Object other) => other is PhenoValues && params.every((p) => _v[p.id] == other._v[p.id]);
  @override
  int get hashCode => Object.hashAll(params.map((p) => _v[p.id]));
  @override
  String toString() => params.map((p) => '${p.id}=${p.fmt(_v[p.id]!)}').join(' ');
}

/// Holds the values and a local undo list. One gesture = begin / update* / end = one undo step. Both views of a use case share one controller.
class PhenoController extends ChangeNotifier {
  PhenoController(this.params) : _v = PhenoValues.defaults(params);
  final List<PhenoParam> params;
  PhenoValues _v;
  PhenoValues? _start;
  final _undo = <PhenoValues>[];

  PhenoValues get value => _v;
  bool get dragging => _start != null;

  void begin() => _start = _v;
  void update(Map<String, double> m) {
    _v = _v.with_(m);
    notifyListeners();
  }

  void end() {
    final s = _start;
    _start = null;
    if (s != null && s != _v) _undo.add(s);
    notifyListeners();
  }

  /// Esc during a drag: back to where the gesture began, no undo step.
  void cancel() {
    final s = _start;
    if (s == null) return;
    _v = s;
    _start = null;
    notifyListeners();
  }

  /// A whole gesture in one call (tap, double-click reset).
  void commit(Map<String, double> m) {
    begin();
    update(m);
    end();
  }

  void undo() {
    if (_undo.isEmpty) return;
    _v = _undo.removeLast();
    notifyListeners();
  }
}

// ---------------------------------------------------------------- the two halves of a miniature

/// What the pointer sees: the frame state handed to the sim every tick.
class PhenoCtx {
  const PhenoCtx(this.v, this.hover, this.active, this.ptr, this.t, this.live);
  final PhenoValues v;
  final String? hover, active;
  final Offset? ptr;
  final double t;
  /// The pointer is over the miniature or dragging it: the only time a little life is allowed.
  final bool live;
}

abstract class PhenoSim {
  /// Advance by [dt] seconds toward the values; true while anything is still moving.
  bool step(double dt, Size s, PhenoCtx c);
  void paint(Canvas cv, Size s, PhenoCtx c);
}

abstract class PhenoSpec {
  const PhenoSpec();
  String get name;
  /// One word, 10 px, inside the miniature (hidden by the Words addon).
  String get word;
  /// The story line outside the frame: name, merged parameters, silhouette.
  String get caption;
  List<PhenoParam> get params;
  double get rowHeight;
  PhenoSim newSim();

  /// The grabbable thing under [p], or null. Every zone is at least 24 px wide (an invisible padding around a drawn mark).
  String? zoneAt(Size s, Offset p, PhenoValues v);

  /// New values for a drag in [zone]. [p0] where it started, [p] where the pointer is now (Shift already scaled the movement), [v0] the values at the start.
  Map<String, double> drag(String zone, Size s, Offset p0, Offset p, PhenoValues v0);

  /// A press-and-release without movement.
  Map<String, double>? tap(String zone, PhenoValues v) => null;

  /// Which parameters a double-click in [zone] resets.
  List<String> zoneParams(String zone) => [for (final p in params) p.id];

  String fmt(PhenoParam p, double v) => p.fmt(v);
}

// ---------------------------------------------------------------- small shared helpers

/// The drawing area: inside the well, clear of the caption (top-left) and the readout (bottom-right).
Rect _cr(Size s) => Rect.fromLTRB(10, 20, s.width - 10, s.height - 22);

double _hash(int a, int b) {
  final x = math.sin(a * 12.9898 + b * 78.233) * 43758.5453;
  return x - x.floorToDouble();
}

Paint _line(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w;
Paint _fill(Color c) => Paint()..color = c;

Color get _hot => Role.of(N.g95, Role.selected);

/// The mark colour of a grabbable zone: the active one is hot, a hovered one a quiet white; null when neither (nothing is drawn).
Color? _mark(PhenoCtx c, String zone) => c.active == zone ? _hot : (c.hover == zone && c.active == null ? N.g95.withValues(alpha: .6) : null);

/// A critically-to-slightly-under damped spring (a small settle is allowed, nothing rings).
class _Sp {
  _Sp();
  double x = 0, v = 0;
  bool step(double to, double dt, {double k = 220, double z = .75}) {
    final d = 2 * z * math.sqrt(k);
    final n = (dt / .008).ceil().clamp(1, 8);
    final h = dt / n;
    for (var i = 0; i < n; i++) {
      v += ((to - x) * k - v * d) * h;
      x += v * h;
    }
    return (to - x).abs() > .002 || v.abs() > .02;
  }
}

// ---------------------------------------------------------------- the interaction shell

class _Poke extends ChangeNotifier {
  void poke() => notifyListeners();
}

/// A miniature on a quiet well: a one-word caption, the picture, a small mono readout of the true values at its edge.
/// Gestures: drag a grabbable part, hover shows what is grabbable (a small highlight), Shift = fine, double-click resets that part,
/// Esc during a drag restores, Cmd/Ctrl+Z undoes one gesture.
class PhenoView extends StatefulWidget {
  const PhenoView({super.key, required this.spec, required this.controller, required this.size});
  final PhenoSpec spec;
  final PhenoController controller;
  final Size size;
  @override
  State<PhenoView> createState() => _PhenoViewState();
}

class _PhenoViewState extends State<PhenoView> with SingleTickerProviderStateMixin {
  late final PhenoSim _sim = widget.spec.newSim();
  late final Ticker _tick = createTicker(_onTick);
  final _repaint = _Poke();
  final _focus = FocusNode();
  Duration _last = Duration.zero;
  double _t = 0, _calm = 0;
  String? _hover, _active, _lastZone;
  Offset? _ptr, _p0, _eff, _prev;
  PhenoValues? _v0;
  bool _moved = false;
  int _lastDownMs = -9999, _downMs = 0;

  PhenoSpec get spec => widget.spec;
  PhenoController get ctl => widget.controller;
  bool get _live => _ptr != null || _active != null;

  @override
  void initState() {
    super.initState();
    ctl.addListener(_wake);
    _wake();
  }

  @override
  void dispose() {
    ctl.removeListener(_wake);
    _tick.dispose();
    _focus.dispose();
    _repaint.dispose();
    super.dispose();
  }

  void _wake() {
    _calm = 0;
    if (!_tick.isActive) {
      _last = Duration.zero;
      _tick.start();
    }
    if (mounted) setState(() {});
  }

  PhenoCtx _ctx() => PhenoCtx(ctl.value, _hover, _active, _ptr, _t, _live);

  void _onTick(Duration el) {
    final dt = _last == Duration.zero ? .016 : ((el - _last).inMicroseconds / 1e6).clamp(0.0, .033);
    _last = el;
    _t += dt;
    final moving = _sim.step(dt, widget.size, _ctx());
    _repaint.poke();
    if (moving || _live) {
      _calm = 0;
    } else {
      _calm += dt;
      if (_calm > .3) _tick.stop();
    }
  }

  void _setHover(String? z) {
    if (z != _hover) setState(() => _hover = z);
  }

  void _onHover(PointerHoverEvent e) {
    final wasNull = _ptr == null;
    _ptr = e.localPosition;
    _setHover(spec.zoneAt(widget.size, e.localPosition, ctl.value));
    if (wasNull) _wake();
  }

  void _onExit(PointerExitEvent e) {
    if (_active != null) return;
    _ptr = null;
    _setHover(null);
    _wake();
  }

  void _down(PointerDownEvent e) {
    _focus.requestFocus();
    final z = spec.zoneAt(widget.size, e.localPosition, ctl.value);
    if (z == null) return;
    final ms = e.timeStamp.inMilliseconds;
    if (z == _lastZone && ms - _lastDownMs < 320) {
      _lastDownMs = -9999;
      ctl.commit({for (final id in spec.zoneParams(z)) id: ctl.params.firstWhere((p) => p.id == id).def});
      return;
    }
    _downMs = ms;
    _active = z;
    _p0 = _eff = _prev = e.localPosition;
    _v0 = ctl.value;
    _moved = false;
    ctl.begin();
    setState(() {});
  }

  void _move(PointerMoveEvent e) {
    final z = _active;
    if (z == null) return;
    _ptr = e.localPosition;
    // Shift = fine: the movement is scaled, accumulated, so pressing Shift mid-drag never jumps.
    final k = HardwareKeyboard.instance.isShiftPressed ? .2 : 1.0;
    _eff = _eff! + (e.localPosition - _prev!) * k;
    _prev = e.localPosition;
    if ((e.localPosition - _p0!).distance > 3) _moved = true;
    ctl.update(spec.drag(z, widget.size, _p0!, _eff!, _v0!));
  }

  void _up(PointerEvent e) {
    final z = _active;
    if (z == null) return;
    // Only a click (no movement) can be the first half of a double-click.
    _lastDownMs = _moved ? -9999 : _downMs;
    _lastZone = z;
    if (!_moved) {
      final t = spec.tap(z, _v0!);
      if (t != null) ctl.update(t);
    }
    ctl.end();
    _active = null;
    final inside = Offset.zero & widget.size;
    if (!inside.contains(e.localPosition)) _ptr = null;
    _setHover(_ptr == null ? null : spec.zoneAt(widget.size, _ptr!, ctl.value));
    _wake();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    if (e.logicalKey == LogicalKeyboardKey.escape && _active != null) {
      ctl.cancel();
      _active = null;
      _lastDownMs = -9999;
      _wake();
      return KeyEventResult.handled;
    }
    final kb = HardwareKeyboard.instance;
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (kb.isMetaPressed || kb.isControlPressed) && _active == null) {
      ctl.undo();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  Widget _readout() {
    final v = ctl.value;
    final spans = <InlineSpan>[];
    for (final p in spec.params) {
      if (spans.isNotEmpty) spans.add(const TextSpan(text: '  '));
      spans.add(TextSpan(text: spec.fmt(p, v[p.id]), style: TextStyle(color: T.ink(v.changed(p.id) ? Role.changed : N.g63))));
    }
    return Text.rich(TextSpan(children: spans),
        maxLines: 1,
        overflow: TextOverflow.clip,
        style: TextStyle(fontFamily: T.mono, fontSize: 10, height: 1, color: T.ink(N.g63), decoration: TextDecoration.none));
  }

  @override
  Widget build(BuildContext context) {
    final s = widget.size;
    return Focus(
      focusNode: _focus,
      onKeyEvent: _key,
      child: MouseRegion(
        cursor: _active != null ? SystemMouseCursors.grabbing : (_hover != null ? SystemMouseCursors.grab : MouseCursor.defer),
        onHover: _onHover,
        onExit: _onExit,
        child: Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: _down,
          onPointerMove: _move,
          onPointerUp: _up,
          onPointerCancel: _up,
          child: SizedBox(
            width: s.width,
            height: s.height,
            child: ClipRRect(
              borderRadius: BorderRadius.circular(Dose.radius),
              child: Stack(children: [
                Positioned.fill(child: CustomPaint(painter: _WellPainter(this))),
                Positioned(left: 8, top: 6, child: Text(spec.word, style: T.label(N.g63))),
                Positioned(right: 8, bottom: 5, child: ListenableBuilder(listenable: ctl, builder: (_, _) => _readout())),
              ]),
            ),
          ),
        ),
      ),
    );
  }
}

class _WellPainter extends CustomPainter {
  _WellPainter(this.st) : super(repaint: Listenable.merge([st._repaint, Role.changes]));
  final _PhenoViewState st;
  @override
  void paint(Canvas cv, Size s) {
    final r = RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(Dose.radius));
    cv.drawRRect(r, _fill(N.g07));
    cv.save();
    cv.clipRRect(r);
    st._sim.paint(cv, s, st._ctx());
    cv.restore();
    cv.drawRRect(r.deflate(.5), _line(N.g20));
  }

  @override
  bool shouldRepaint(_WellPainter o) => true;
}

// ---------------------------------------------------------------- the use cases

/// One neighbour row of the Inspector: a quiet label and a plain number cell (what the miniature stands among).
Widget _numRow(String label, String value) => SizedBox(
      height: 28,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: Row(children: [
          SizedBox(width: 96, child: Text(label, style: T.label(N.g76))),
          Container(
            width: 96,
            height: 20,
            alignment: Alignment.centerRight,
            padding: const EdgeInsets.symmetric(horizontal: 6),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g20)),
            child: Text(value, style: T.value(N.g91)),
          ),
        ]),
      ),
    );

class _PhenoCase extends StatefulWidget {
  const _PhenoCase(this.spec, {super.key});
  final PhenoSpec spec;
  @override
  State<_PhenoCase> createState() => _PhenoCaseState();
}

class _PhenoCaseState extends State<_PhenoCase> {
  late final PhenoController _c = PhenoController(widget.spec.params);
  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final sp = widget.spec;
    return ListenableBuilder(
      listenable: Role.changes,
      builder: (_, _) => ColoredBox(
        color: N.g07,
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
            // The story caption: outside the frames, one line.
            Text(sp.caption, style: T.label(N.g76), maxLines: 1, softWrap: false),
            const SizedBox(height: 16),
            Wrap(spacing: 24, runSpacing: 24, children: [
              PhenoView(spec: sp, controller: _c, size: const Size(320, 200)),
              // The same miniature in a 282 px Inspector column, between two plain number rows.
              Container(
                width: 282,
                decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(Dose.radius), border: Border.all(color: N.g20)),
                child: Column(mainAxisSize: MainAxisSize.min, children: [
                  _numRow('Opacity', '100.0 %'),
                  Padding(padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4), child: PhenoView(spec: sp, controller: _c, size: Size(258, sp.rowHeight))),
                  _numRow('Rotation', '0.0 °'),
                ]),
              ),
            ]),
          ]),
        ),
      ),
    );
  }
}

WidgetbookComponent phenoASet() => WidgetbookComponent(name: 'Phenomenon A', useCases: [
      for (final sp in const <PhenoSpec>[_ScatterSpec(), _StaggerSpec(), _RepeatSpec(), _GridSpec(), _FalloffSpec(), _GravitySpec()])
        WidgetbookUseCase(name: sp.name, builder: (c) => _PhenoCase(sp, key: ValueKey(sp.name))),
    ]);
