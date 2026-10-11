part of 'pheno_e.dart';

// The shared body of the six miniatures: a value object (named parameters with range, default and unit), a spring, a local undo list,
// and one view that turns pointer events into model calls. A miniature never reads a widget; the model draws itself.

/// One real, named parameter: range, default, unit. [dec] is the decimals of the readout.
class PParam {
  const PParam(this.id, this.name, this.min, this.max, this.def, this.unit, {this.dec = 0});
  final String id, name, unit;
  final double min, max, def;
  final int dec;
  double clamp(double x) => x.clamp(min, max).toDouble();
  double norm(double x) => (x - min) / (max - min);
  String fmt(double x) => '${x.toStringAsFixed(dec)}$unit';
}

/// A quiet text placed against the frame's edge (the readout and the one-word caption).
class Edge {
  const Edge(this.text, {this.left, this.top, this.right, this.bottom, this.hot = false, this.align = TextAlign.left});
  final String text;
  final double? left, top, right, bottom;
  final bool hot; // true = a value that differs from its default
  final TextAlign align;
}

/// A damped spring on a 0..1 value: a small overshoot, settles by itself, immediate (no ease-in).
class Spr {
  Spr(double x)
      : x = x,
        t = x;
  double x, t, v = 0;
  bool get still => (x - t).abs() < .0004 && v.abs() < .004;
  void step(double dt, {double k = 1100, double z = .6}) {
    final c = 2 * z * math.sqrt(k);
    var left = dt;
    while (left > 0) {
      final h = math.min(left, 1 / 240);
      v += (-k * (x - t) - c * v) * h;
      x += v * h;
      left -= h;
    }
    if (still) {
      x = t;
      v = 0;
    }
  }
}

class Snap {
  Snap(this.v, this.x, this.sig);
  final Map<String, double> v;
  final Object? x;
  final String sig;
}

abstract class PhenoModel extends ChangeNotifier {
  PhenoModel(this.params, [Map<String, double> initial = const {}]) {
    for (final p in params) {
      byId[p.id] = p;
      v[p.id] = p.clamp(initial[p.id] ?? p.def);
      sp[p.id] = Spr(p.norm(v[p.id]!));
    }
  }

  final List<PParam> params;
  final Map<String, PParam> byId = {};
  final Map<String, double> v = {};
  final Map<String, Spr> sp = {};
  final List<Snap> _undo = [];
  int hoverCount = 0;
  int? active;
  Snap? _g;
  Duration? _last;
  double springK = 1100, springZ = .6;

  bool get hover => hoverCount > 0;
  int get undoDepth => _undo.length;
  double operator [](String id) => v[id]!;
  /// The value as the miniature shows it: the spring's position, so a change settles with a small overshoot.
  double d(String id) {
    final p = byId[id]!;
    return p.min + sp[id]!.x * (p.max - p.min);
  }

  void set(String id, double x) => v[id] = byId[id]!.clamp(x);
  void add(String id, double dx) => set(id, v[id]! + dx);
  bool changed(String id) => (v[id]! - byId[id]!.def).abs() > 1e-6;

  // ---- what a miniature says about itself ----
  String get caption;
  String get story => '';
  int? zoneAt(Offset p, Size s);
  void onDown(int zone, Offset p, Size s) {}
  void onMove(Offset p, Offset delta, Size s, bool fine) {}
  void onUp(Offset vel, bool moved) {}
  void onHover(Offset? p, Size s) {}
  void paintMini(Canvas c, Size s, int? hot);
  List<Edge> edges(Size s);
  /// Frames wanted for something the model itself runs (physics, a demo).
  bool get alive => false;
  void tick(double dt) {}
  Object? saveExtra() => null;
  void loadExtra(Object? x) {}
  String extraSig() => '';
  void resetExtra() {}

  bool get settling => sp.values.any((s) => !s.still);
  bool get wantsFrames => alive || settling;

  void advance(Duration now) {
    final last = _last;
    _last = now;
    if (last == null) return;
    var dt = (now - last).inMicroseconds / 1e6;
    if (dt <= 0) return;
    if (dt > .05) dt = .05;
    for (final p in params) {
      final s = sp[p.id]!;
      s.t = p.norm(v[p.id]!);
      s.step(dt, k: springK, z: springZ);
    }
    tick(dt);
    notifyListeners();
  }

  // ---- undo: one gesture = one step ----
  Snap snapshot() => Snap({...v}, saveExtra(), extraSig());
  void restore(Snap s) {
    v
      ..clear()
      ..addAll(s.v);
    loadExtra(s.x);
  }

  bool _same(Snap s) {
    if (s.sig != extraSig()) return false;
    for (final e in s.v.entries) {
      if ((v[e.key]! - e.value).abs() > 1e-9) return false;
    }
    return true;
  }

  void undoOne() {
    if (_undo.isEmpty) return;
    restore(_undo.removeLast());
    notifyListeners();
  }

  void resetAll() {
    final s = snapshot();
    for (final p in params) {
      v[p.id] = p.def;
    }
    resetExtra();
    if (!_same(s)) _undo.add(s);
    notifyListeners();
  }

  // ---- the gesture, called by every view of this model ----
  void down(int zone, Offset p, Size s) {
    _g = snapshot();
    active = zone;
    onDown(zone, p, s);
    notifyListeners();
  }

  void move(Offset p, Offset delta, Size s, bool fine) {
    if (active == null) return;
    onMove(p, delta, s, fine);
    notifyListeners();
  }

  void up(Offset vel, bool moved) {
    if (active == null) return;
    onUp(vel, moved);
    active = null;
    final g = _g;
    _g = null;
    if (g != null && !_same(g)) _undo.add(g);
    notifyListeners();
  }

  /// Esc during a drag: everything goes back and the gesture is over.
  void cancel() {
    final g = _g;
    if (active == null || g == null) return;
    restore(g);
    active = null;
    _g = null;
    notifyListeners();
  }
}

// ---- drawing helpers ----
Paint _ln(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;
Paint _fl(Color c) => Paint()..color = c;
Color get _hotInk => Role.of(N.g100, Role.selected);
Color get _chgInk => Role.of(N.g95, Role.changed);

void _dashed(Canvas c, Path p, Paint paint, {double on = 2, double off = 3}) {
  for (final m in p.computeMetrics()) {
    for (var d = 0.0; d < m.length; d += on + off) {
      c.drawPath(m.extractPath(d, math.min(d + on, m.length)), paint);
    }
  }
}

Offset _lerpO(Offset a, Offset b, double t) => Offset.lerp(a, b, t)!;
double _lerp(double a, double b, double t) => a + (b - a) * t;
double _cl(double x, [double lo = 0, double hi = 1]) => x.clamp(lo, hi).toDouble();
double _sgn(double x) => x < 0 ? -1 : 1;

/// A small marker of "you can grab this": a hairline ring, brighter when it is being dragged.
void _hotRing(Canvas c, Offset o, double r, {bool on = false}) => c.drawCircle(o, r, _ln(on ? _hotInk : N.g95.withValues(alpha: .45)));

// ---- the view ----
class PhenoView extends StatefulWidget {
  const PhenoView({super.key, required this.model, required this.size});
  final PhenoModel model;
  final Size size;
  @override
  State<PhenoView> createState() => _PhenoViewState();
}

class _PhenoViewState extends State<PhenoView> with SingleTickerProviderStateMixin {
  late final Ticker _tick = createTicker(_onTick);
  final FocusNode _focus = FocusNode();
  final VelocityTracker _vt = VelocityTracker.withKind(PointerDeviceKind.mouse);
  int? _zone;
  bool _in = false, _drag = false, _moved = false, _swallow = false;
  Offset _downP = Offset.zero, _lastP = Offset.zero, _clickP = Offset.zero;
  Duration _clickT = Duration.zero;

  PhenoModel get m => widget.model;

  @override
  void initState() {
    super.initState();
    m.addListener(_onModel);
    _onModel();
  }

  @override
  void dispose() {
    m.removeListener(_onModel);
    if (_in) m.hoverCount--;
    _tick.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _onModel() {
    if (m.wantsFrames && !_tick.isActive && mounted) _tick.start();
  }

  void _onTick(Duration _) {
    m.advance(SchedulerBinding.instance.currentFrameTimeStamp);
    if (!m.wantsFrames) _tick.stop();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    if (e.logicalKey == LogicalKeyboardKey.escape && _drag) {
      m.cancel();
      _drag = false;
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed) && !_drag) {
      m.undoOne();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void _down(PointerDownEvent e) {
    _focus.requestFocus();
    final p = e.localPosition;
    final dbl = e.timeStamp - _clickT < const Duration(milliseconds: 340) && (p - _clickP).distance < 8;
    _clickT = e.timeStamp;
    _clickP = p;
    if (dbl) {
      _swallow = true;
      m.resetAll();
      return;
    }
    final z = m.zoneAt(p, widget.size);
    if (z == null) return;
    _swallow = false;
    _drag = true;
    _moved = false;
    _downP = _lastP = p;
    _vt.addPosition(e.timeStamp, p);
    m.down(z, p, widget.size);
    _onModel();
  }

  void _move(PointerMoveEvent e) {
    if (!_drag) return;
    final p = e.localPosition;
    _vt.addPosition(e.timeStamp, p);
    if ((p - _downP).distance > 3) _moved = true;
    m.move(p, p - _lastP, widget.size, HardwareKeyboard.instance.isShiftPressed);
    _lastP = p;
    _onModel();
  }

  void _up(PointerUpEvent e) {
    if (_swallow) {
      _swallow = false;
      return;
    }
    if (!_drag) return;
    _drag = false;
    final vel = _vt.getVelocity().pixelsPerSecond;
    m.up(vel, _moved);
    _onModel();
  }

  void _hover(PointerHoverEvent e) {
    final z = m.zoneAt(e.localPosition, widget.size);
    if (z != _zone) setState(() => _zone = z);
    m.onHover(e.localPosition, widget.size);
  }

  @override
  Widget build(BuildContext context) {
    final sz = widget.size;
    return SizedBox.fromSize(
      size: sz,
      child: Focus(
        focusNode: _focus,
        onKeyEvent: _key,
        child: MouseRegion(
          cursor: m.active != null ? SystemMouseCursors.grabbing : (_zone != null ? SystemMouseCursors.grab : MouseCursor.defer),
          onEnter: (_) {
            _in = true;
            m.hoverCount++;
            _onModel();
          },
          onExit: (_) {
            if (_in) m.hoverCount--;
            _in = false;
            m.onHover(null, sz);
            if (_zone != null) setState(() => _zone = null);
            _onModel();
          },
          onHover: _hover,
          child: Listener(
            onPointerDown: _down,
            onPointerMove: _move,
            onPointerUp: _up,
            onPointerCancel: (_) {
              if (_drag) m.cancel();
              _drag = false;
            },
            child: DecoratedBox(
              decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(3),
                child: ListenableBuilder(
                  listenable: Listenable.merge([m, Role.changes]),
                  builder: (context, _) => Stack(children: [
                    Positioned.fill(child: CustomPaint(painter: _MiniPainter(m, m.active ?? _zone))),
                    for (final e in m.edges(sz))
                      Positioned(
                        left: e.left,
                        top: e.top,
                        right: e.right,
                        bottom: e.bottom,
                        child: IgnorePointer(
                          child: Text(e.text,
                              textAlign: e.align,
                              style: TextStyle(fontFamily: T.mono, fontSize: 10, height: 1.3, decoration: TextDecoration.none, color: T.ink(e.hot ? _chgInk : N.g63))),
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

class _MiniPainter extends CustomPainter {
  const _MiniPainter(this.m, this.hot);
  final PhenoModel m;
  final int? hot;
  @override
  void paint(Canvas c, Size s) => m.paintMini(c, s, hot);
  @override
  bool shouldRepaint(_MiniPainter o) => true;
}

/// The one-word caption of a miniature, drawn at its top-left corner.
Edge _cap(String w) => Edge(w, left: 7, top: 6);
