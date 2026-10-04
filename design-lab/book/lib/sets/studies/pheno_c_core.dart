part of 'pheno_c.dart';

const _readH = 14.0;

Color _a(Color c, double x) => c.withValues(alpha: x.clamp(0.0, 1.0).toDouble());
Paint _ln(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w;
Paint _fl(Color c) => Paint()..color = c;
Color get _hot => N.g95;
Color get _act => Role.selected;

double _hash(int n) {
  var x = (n * 374761393 + 668265263) & 0x7fffffff;
  x = ((x ^ (x >> 13)) * 1274126177) & 0x7fffffff;
  x ^= x >> 16;
  return (x & 0xffff) / 65535.0;
}

double _noise1(double u) {
  final i = u.floor();
  final f = u - i;
  final s = f * f * (3 - 2 * f);
  final a = _hash(i) * 2 - 1, b = _hash(i + 1) * 2 - 1;
  return a + (b - a) * s;
}

/// One named, ranged value a miniature edits. [names] turns an enum-like value into its word.
class PcParam {
  const PcParam(this.id, this.label, this.min, this.max, this.def, {this.unit = '', this.dec = 0, this.names});
  final String id, label, unit;
  final double min, max, def;
  final int dec;
  final List<String>? names;
  String fmt(double v) => names != null ? names![v.round().clamp(0, names!.length - 1)] : '${v.toStringAsFixed(dec)}$unit';
}

/// The value object of a miniature: read and write by id. A gesture is begin..end (one undo step); cancel restores.
class PcDoc extends ChangeNotifier {
  PcDoc(this.params) : v = {for (final p in params) p.id: p.def};
  final List<PcParam> params;
  final Map<String, double> v;
  final List<Map<String, double>> undo = [];
  Map<String, double>? _snap;

  PcParam param(String id) => params.firstWhere((p) => p.id == id);
  double operator [](String id) => v[id]!;
  String fmt(String id) => param(id).fmt(v[id]!);
  bool get live => _snap != null;
  bool get isChanged => params.any((p) => v[p.id] != p.def);

  void set(String id, double x) {
    final p = param(id);
    final n = x.clamp(p.min, p.max).toDouble();
    if (v[id] == n) return;
    v[id] = n;
    notifyListeners();
  }

  void begin() => _snap ??= Map.of(v);

  void end() {
    final s = _snap;
    _snap = null;
    if (s != null && !mapEquals(s, v)) undo.add(s);
  }

  void cancel() {
    final s = _snap;
    _snap = null;
    if (s == null) return;
    v
      ..clear()
      ..addAll(s);
    notifyListeners();
  }

  void reset() {
    begin();
    for (final p in params) {
      v[p.id] = p.def;
    }
    notifyListeners();
    end();
  }

  bool undoOne() {
    if (undo.isEmpty) return false;
    final s = undo.removeLast();
    v
      ..clear()
      ..addAll(s);
    notifyListeners();
    return true;
  }
}

/// A damped spring: a little overshoot, then still.
class _Sp {
  _Sp(this.x);
  double x, v = 0;
  void to(double t, double dt, {double k = 260, double d = 17}) {
    final n = (dt / .004).ceil().clamp(1, 24);
    final h = dt / n;
    for (var i = 0; i < n; i++) {
      v += (k * (t - x) - d * v) * h;
      x += v * h;
    }
  }

  bool at(double t, [double e = .002]) => (x - t).abs() < e && v.abs() < e * 6;
}

/// A miniature: owns the doc, the grab zones, the physics and the drawing. Hover only highlights; a drag acts on the thing itself.
abstract class _Toy extends ChangeNotifier {
  _Toy(this.doc) {
    doc.addListener(notifyListeners);
  }
  final PcDoc doc;
  String? hover;
  bool hovering = false, pressed = false, dragging = false;
  Object? _extra;
  final _clock = Stopwatch()..start();
  double get now => _clock.elapsedMicroseconds / 1e6;

  String? zoneAt(Offset p, Size s);
  List<String> readout();
  void paint(Canvas c, Size s);
  bool get busy => pressed || dragging;
  void step(double dt) {}
  void grab(Offset p, Size s, String zone) {}
  void drag(Offset p, Offset d, Size s, bool fine) {}
  void release(Offset p, Size s) => finish();
  Object? saveExtra() => null;
  void restoreExtra(Object? o) {}
  void stopMotion() {}
  void docReset() {}

  void poke() => notifyListeners();

  bool pointerDown(Offset p, Size s) {
    final z = zoneAt(p, s);
    if (z == null) return false;
    doc.begin();
    _extra = saveExtra();
    pressed = true;
    dragging = true;
    hover = z;
    grab(p, s, z);
    notifyListeners();
    return true;
  }

  void pointerMove(Offset p, Offset d, Size s, bool fine) {
    if (!dragging || !pressed) return;
    drag(p, d, s, fine);
    notifyListeners();
  }

  void pointerUp(Offset p, Size s) {
    if (!dragging) return;
    pressed = false;
    release(p, s);
    notifyListeners();
  }

  void finish() {
    dragging = false;
    doc.end();
  }

  void cancelGesture() {
    if (!dragging) return;
    stopMotion();
    restoreExtra(_extra);
    doc.cancel();
    dragging = false;
    pressed = false;
    notifyListeners();
  }

  void resetAll() {
    stopMotion();
    doc.reset();
    docReset();
    notifyListeners();
  }

  void hoverAt(Offset? p, Size s) {
    if (pressed) return;
    final z = p == null ? null : zoneAt(p, s);
    if (z != hover || hovering != (z != null)) {
      hover = z;
      hovering = z != null;
      notifyListeners();
    }
  }

  @override
  void dispose() {
    doc.removeListener(notifyListeners);
    super.dispose();
  }
}

class _ToyPainter extends CustomPainter {
  _ToyPainter(this.toy) : super(repaint: Listenable.merge([toy, Role.changes]));
  final _Toy toy;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.clipRect(Offset.zero & size);
    toy.paint(canvas, size);
    canvas.restore();
    if (toy.doc.isChanged) {
      canvas.drawPath(Path()..moveTo(size.width, 0)..lineTo(size.width - 5, 0)..lineTo(size.width, 5)..close(), _fl(Role.of(N.g76, Role.changed)));
    }
  }

  @override
  bool shouldRepaint(_ToyPainter old) => false;
}

/// One view of a toy at a size. Two views of the same toy edit the same doc.
class _PcView extends StatefulWidget {
  const _PcView({required this.toy, required this.w, required this.h});
  final _Toy toy;
  final double w, h;

  @override
  State<_PcView> createState() => _PcViewState();
}

class _PcViewState extends State<_PcView> {
  final _focus = FocusNode();
  late final _ToyPainter _painter = _ToyPainter(widget.toy);
  Offset? _last, _lastDown;
  int _lastDownMs = -1000;
  bool _down = false;

  Size get _art => Size(widget.w, widget.h - _readH);

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  void _onDown(PointerDownEvent e) {
    _focus.requestFocus();
    final p = e.localPosition;
    final ms = DateTime.now().millisecondsSinceEpoch;
    if (_lastDown != null && ms - _lastDownMs < 340 && (p - _lastDown!).distance < 10) {
      _lastDown = null;
      _down = false;
      widget.toy.resetAll();
      return;
    }
    _lastDown = p;
    _lastDownMs = ms;
    _last = p;
    _down = widget.toy.pointerDown(p, _art);
  }

  void _onMove(PointerMoveEvent e) {
    if (!_down) return;
    final d = e.localPosition - _last!;
    _last = e.localPosition;
    widget.toy.pointerMove(e.localPosition, d, _art, HardwareKeyboard.instance.isShiftPressed);
  }

  void _onUp(PointerEvent e) {
    if (!_down) return;
    _down = false;
    widget.toy.pointerUp(e.localPosition, _art);
  }

  KeyEventResult _onKey(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    if (e.logicalKey == LogicalKeyboardKey.escape && widget.toy.dragging) {
      widget.toy.cancelGesture();
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      widget.toy.doc.undoOne();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final toy = widget.toy;
    final art = _art;
    final surface = Listener(
      behavior: HitTestBehavior.opaque,
      onPointerDown: _onDown,
      onPointerMove: _onMove,
      onPointerUp: _onUp,
      onPointerCancel: _onUp,
      child: Focus(focusNode: _focus, onKeyEvent: _onKey, child: CustomPaint(size: art, painter: _painter)),
    );
    return SizedBox(
      width: widget.w,
      height: widget.h,
      child: Stack(children: [
        Positioned(
          left: 0,
          top: 0,
          width: art.width,
          height: art.height,
          child: ListenableBuilder(
            listenable: toy,
            child: surface,
            builder: (c, child) => MouseRegion(
              cursor: toy.hover == null ? MouseCursor.defer : (toy.pressed ? SystemMouseCursors.grabbing : SystemMouseCursors.grab),
              onHover: (e) => toy.hoverAt(e.localPosition, art),
              onExit: (e) => toy.hoverAt(null, art),
              child: child,
            ),
          ),
        ),
        Positioned(
          right: 6,
          bottom: 2,
          child: IgnorePointer(
            child: ListenableBuilder(
              listenable: toy,
              builder: (c, _) => Text(toy.readout().join('  '), maxLines: 1, style: TextStyle(fontFamily: T.mono, fontSize: 10, color: T.ink(N.g63), height: 1, decoration: TextDecoration.none)),
            ),
          ),
        ),
      ]),
    );
  }
}

/// Hosts one toy: the ticker (runs only while the toy is busy), the large 320x200 view and the 282 px Inspector row beside it.
class _Story extends StatefulWidget {
  const _Story({required this.make, required this.caption, required this.label, required this.rowH, required this.above, required this.below});
  final _Toy Function() make;
  final String caption, label;
  final double rowH;
  final (String, String) above, below;

  @override
  State<_Story> createState() => _StoryState();
}

class _StoryState extends State<_Story> with SingleTickerProviderStateMixin {
  late final _Toy toy = widget.make();
  late final Ticker _tk = createTicker(_tick);
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    toy.addListener(_wake);
    _wake();
  }

  @override
  void dispose() {
    toy.removeListener(_wake);
    _tk.dispose();
    toy.dispose();
    super.dispose();
  }

  void _wake() {
    if (toy.busy && !_tk.isActive) {
      _last = Duration.zero;
      _tk.start();
    }
  }

  void _tick(Duration e) {
    final dt = ((e - _last).inMicroseconds / 1e6).clamp(0.0, .05).toDouble();
    _last = e;
    toy.step(dt);
    toy.poke();
    if (!toy.busy) _tk.stop();
  }

  Widget _numRow(String label, String value) => SizedBox(
        height: 28,
        child: Padding(
          padding: const EdgeInsets.only(left: 12, right: 8),
          child: Row(children: [
            SizedBox(width: 72, child: Text(label, maxLines: 1, style: T.label(N.g76))),
            const Spacer(),
            Container(
              width: 64,
              height: 20,
              alignment: Alignment.centerRight,
              padding: const EdgeInsets.symmetric(horizontal: 6),
              decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(2), border: Border.all(color: N.g20)),
              child: Text(value, maxLines: 1, style: T.value(N.g91)),
            ),
          ]),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final hair = Container(height: 1, color: N.g15);
    return Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
      Text(
        widget.caption,
        maxLines: 1,
        softWrap: false,
        overflow: TextOverflow.ellipsis,
        style: T.label(N.g63),
      ),
      const SizedBox(height: 10),
      Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        SizedBox(
          width: 320,
          height: 200,
          child: Stack(children: [
            const Positioned.fill(child: ColoredBox(color: N.g10)),
            _PcView(toy: toy, w: 320, h: 200),
            Positioned.fill(child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: N.g20))))),
          ]),
        ),
        const SizedBox(width: 24),
        Container(
          width: 282,
          color: N.g10,
          foregroundDecoration: BoxDecoration(border: Border.all(color: N.g20)),
          child: Column(mainAxisSize: MainAxisSize.min, children: [
            _numRow(widget.above.$1, widget.above.$2),
            hair,
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 4, 8, 4),
              child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                SizedBox(width: 72, child: Text(widget.label, maxLines: 1, style: T.label(N.g76))),
                const SizedBox(width: 4),
                _PcView(toy: toy, w: 186, h: widget.rowH),
              ]),
            ),
            hair,
            _numRow(widget.below.$1, widget.below.$2),
          ]),
        ),
      ]),
    ]);
  }
}
