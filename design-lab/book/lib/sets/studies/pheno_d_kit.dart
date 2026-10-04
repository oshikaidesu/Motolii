// Phenomenon D, shared plumbing: a parameter turned into a miniature of the phenomenon it means.
// A miniature is a [PhMini] (state + gestures + painter) driven by one [PhBox]; its true numbers live in a [PhDoc] (the value-object a later
// integration reads and writes). Contracts kept here for all six: one gesture = one undo step (local list, never shown), Esc during a drag
// restores, Shift = fine (x0.1), double-click resets, hover = a small highlight (no text), hit zones are >= 24 px, the number is a quiet
// 10 px mono readout at the edge, the one-word caption is optional, and nothing animates unless the pointer is on it (or it is still settling).
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/scheduler.dart' show Ticker;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../tokens.dart';

// ---- numbers and colours ------------------------------------------------------------------------------------------------------------

double phClamp(double v, double a, double b) => v < a ? a : (v > b ? b : v);
double phLerp(double a, double b, double t) => a + (b - a) * t;

/// A smooth, deterministic wobble in -1..1 (three incommensurate sines); [u] is in radians, [seed] separates channels.
double phNoise(double u, double seed) =>
    math.sin(u + seed * 7.1) * .5 + math.sin(u * 2.31 + seed * 3.7) * .3 + math.sin(u * 4.73 + seed * 1.3) * .2;

/// Hover / grab highlight: Grey keeps it quiet white, a palette uses the selection hue (1 px edges and tiny marks only).
Color get phHot => Role.of(N.g95, Role.selected);

/// The one small accent of a miniature (its runner / ball / bead): a relation family hue in a palette, near-white in Grey.
Color phAcc(Fam f) => Role.of(N.g95, f.c);

Paint phStroke(Color c, [double w = 1]) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..strokeCap = StrokeCap.round
  ..strokeJoin = StrokeJoin.round;

Paint phFill(Color c) => Paint()..color = c;

/// A dashed straight hairline (rest lines, guides).
void phDash(Canvas c, Offset a, Offset b, Paint p, {double on = 2, double off = 3}) {
  final d = b - a, len = d.distance;
  if (len < .5) return;
  final u = d / len;
  for (double s = 0; s < len; s += on + off) {
    c.drawLine(a + u * s, a + u * math.min(s + on, len), p);
  }
}

// ---- the value-object -----------------------------------------------------------------------------------------------------------------

/// One real, named parameter: id, range, default, unit.
class PhSpec {
  const PhSpec(this.id, this.min, this.max, this.def, {this.unit = '', this.digits = 2});
  final String id, unit;
  final double min, max, def;
  final int digits;

  String fmt(double v) {
    var s = v.toStringAsFixed(digits);
    if (v.abs() < 1 && digits > 0) s = s.replaceFirst('0.', '.');
    return unit.isEmpty ? s : '$s$unit';
  }
}

/// The values of one miniature. Plain `Map<String,double>` underneath, keyed by the spec ids; a gesture is begin..end and is one undo step.
class PhDoc extends ChangeNotifier {
  PhDoc(this.specs) : _v = {for (final s in specs) s.id: s.def};
  final List<PhSpec> specs;
  final Map<String, double> _v;
  final List<Map<String, double>> _undo = []; // local, never shown
  Map<String, double>? _snap;

  double operator [](String id) => _v[id]!;
  PhSpec spec(String id) => specs.firstWhere((s) => s.id == id);
  Map<String, double> get values => Map.unmodifiable(_v);
  bool changed(String id) => (_v[id]! - spec(id).def).abs() > 1e-9;
  bool get live => _snap != null;
  String show(String id) => spec(id).fmt(_v[id]!);

  void set(String id, double x) {
    final s = spec(id);
    final n = phClamp(x, s.min, s.max);
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

// ---- the miniature ----------------------------------------------------------------------------------------------------------------------

/// What the box knows about the pointer; the painter reads it for the hover / grab highlight.
class PhView {
  Offset? ptr;
  int? hot, grab;
  bool get hover => ptr != null;
}

typedef PhR = (String text, bool changed);

abstract class PhMini {
  PhMini(this.doc);
  final PhDoc doc;
  final view = PhView();

  /// The inner drawing area: the frame minus room for the caption (top) and the readout (bottom).
  Rect art(Size s) => Rect.fromLTRB(12, 20, s.width - 12, s.height - 19);

  /// At most one word (10 px); empty = none. A miniature must be understandable without it.
  String get cap => '';
  List<PhR> readout();

  /// Which grabbable zone is under [p] (null = none). Zones are >= 24 px wide / tall.
  int? zoneAt(Offset p, Size s);
  void down(int z, Offset p, Size s) {}
  void move(int z, Offset p, Offset d, bool fine, Size s) {}
  void up(int z) {}

  /// Esc: the doc is already restored; the miniature drops its own grab state.
  void cancelled(int z) => up(z);

  /// Double-click on zone [z] (null = nothing grabbed): one undo step. Default resets everything.
  void reset(int? z) => doc.resetIds(doc.specs.map((s) => s.id));

  /// An undo / reset moved the doc under the miniature.
  void synced() {}

  /// Advance by [dt] seconds. Returns true while the miniature is still settling by itself (the box also ticks while the pointer is on it).
  bool step(double dt, Size s) => false;

  void paint(Canvas c, Size s);
}

class _PhPainter extends CustomPainter {
  _PhPainter(this.m, Listenable l) : super(repaint: l);
  final PhMini m;
  @override
  void paint(Canvas canvas, Size size) => m.paint(canvas, size);
  @override
  bool shouldRepaint(_PhPainter old) => true;
}

/// One miniature at a fixed size: frame, painter, pointer plumbing, caption, readout.
class PhBox extends StatefulWidget {
  const PhBox({super.key, required this.doc, required this.make, required this.size});
  final PhDoc doc;
  final PhMini Function(PhDoc) make;
  final Size size;
  @override
  State<PhBox> createState() => _PhBoxState();
}

class _PhBoxState extends State<PhBox> with SingleTickerProviderStateMixin {
  late final PhMini mini = widget.make(widget.doc);
  late final Ticker _tk = createTicker(_tick);
  final _rev = ValueNotifier<int>(0);
  final _focus = FocusNode();
  Duration _last = Duration.zero;
  bool _dead = false; // an Esc cancelled this drag; ignore the rest of it
  Duration? _lastDown;
  Offset _lastDownP = Offset.zero;
  MouseCursor _cursor = SystemMouseCursors.basic;

  PhView get v => mini.view;

  @override
  void initState() {
    super.initState();
    widget.doc.addListener(_docMoved);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _wake(); // idle life: a miniature that returns true from step() keeps a little motion going
    });
  }

  @override
  void dispose() {
    widget.doc.removeListener(_docMoved);
    _tk.dispose();
    _rev.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _docMoved() => _rev.value++;

  void _wake() {
    if (!_tk.isActive) {
      _last = Duration.zero;
      _tk.start();
    }
  }

  void _tick(Duration e) {
    final dt = math.min((e - _last).inMicroseconds / 1e6, .05);
    _last = e;
    final alive = mini.step(dt, widget.size);
    _rev.value++;
    if (!alive && !v.hover && v.grab == null) _tk.stop();
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
      mini.synced();
      _rev.value++;
      _wake();
      return;
    }
    if (z == null) return;
    _dead = false;
    widget.doc.begin();
    v.ptr = p;
    v.grab = z;
    mini.down(z, p, widget.size);
    setState(() => _cursor = SystemMouseCursors.grabbing);
    _wake();
  }

  void _move(PointerMoveEvent e) {
    final z = v.grab;
    if (z == null || _dead) return;
    v.ptr = e.localPosition;
    mini.move(z, e.localPosition, e.localDelta, HardwareKeyboard.instance.isShiftPressed, widget.size);
    _rev.value++;
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
    _wake();
  }

  void _hover(Offset p) {
    v.ptr = p;
    v.hot = mini.zoneAt(p, widget.size);
    final want = v.hot == null ? SystemMouseCursors.basic : SystemMouseCursors.grab;
    if (want != _cursor) setState(() => _cursor = want);
    _wake();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final z = v.grab;
    if (e.logicalKey == LogicalKeyboardKey.escape && z != null) {
      widget.doc.cancel();
      mini.cancelled(z);
      mini.synced();
      v.grab = null;
      _dead = true;
      _rev.value++;
      _wake();
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      widget.doc.undo();
      mini.synced();
      _rev.value++;
      _wake();
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
                    Positioned.fill(child: CustomPaint(painter: _PhPainter(mini, Listenable.merge([_rev, widget.doc, Role.changes])))),
                    if (mini.cap.isNotEmpty) Positioned(left: 8, top: 5, child: IgnorePointer(child: Text(mini.cap, style: T.label(N.g63)))),
                    Positioned(
                      right: 8,
                      bottom: 5,
                      child: IgnorePointer(
                        child: ListenableBuilder(
                          listenable: widget.doc,
                          builder: (context, _) => Text.rich(TextSpan(children: [
                            for (final r in mini.readout())
                              TextSpan(text: '${r.$1}  ', style: T.value(r.$2 ? Role.of(N.g76, Role.changed) : N.g63).copyWith(fontSize: 10)),
                          ])),
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

// ---- the use case frame -----------------------------------------------------------------------------------------------------------------

/// A plain neighbour row of the Inspector: a label and a number cell.
class PhNumRow extends StatelessWidget {
  const PhNumRow(this.label, this.value, {super.key});
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

/// One use case: the story caption above the frame, the large miniature (320x200), and the same miniature in a 282 px Inspector row
/// between two plain number rows. Both share one [PhDoc], so a drag in one is seen in the other.
class PhStory extends StatefulWidget {
  const PhStory({super.key, required this.caption, required this.specs, required this.make, required this.rowH, required this.above, required this.below});
  final String caption;
  final List<PhSpec> specs;
  final PhMini Function(PhDoc) make;
  final double rowH;
  final (String, String) above, below;
  @override
  State<PhStory> createState() => _PhStoryState();
}

class _PhStoryState extends State<PhStory> {
  late final PhDoc doc = PhDoc(widget.specs);

  @override
  void dispose() {
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g07,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(widget.caption, style: T.label(N.g63)),
            const SizedBox(height: 14),
            Wrap(spacing: 40, runSpacing: 28, crossAxisAlignment: WrapCrossAlignment.start, children: [
              PhBox(doc: doc, make: widget.make, size: const Size(320, 200)),
              SizedBox(
                width: 282,
                child: Column(mainAxisSize: MainAxisSize.min, children: [
                  PhNumRow(widget.above.$1, widget.above.$2),
                  const SizedBox(height: 4),
                  PhBox(doc: doc, make: widget.make, size: Size(282, widget.rowH)),
                  const SizedBox(height: 4),
                  PhNumRow(widget.below.$1, widget.below.$2),
                ]),
              ),
            ]),
          ]),
        ),
      );
}
