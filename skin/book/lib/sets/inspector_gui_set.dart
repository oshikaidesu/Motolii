// set: inspector_gui. The OLD Motolii Inspector's abstraction (numbers turned into things you can grab), re-drawn in the lab's tone.
// Carried from the old code as CONTRACTS (not looks): one shared document so a drag and a number field are the same value; a gesture is one undo step;
// Esc during a drag puts everything back; Shift = fine (x0.1); a locked layer disables every hand; a turn is kept unwrapped (450 stays 450);
// 2D / 2.5D / 3D reveals Z and the other rotation axes without changing a value; the camera's point fields disable when a layer is the target;
// Layout: Grid gates the rest, only Fixed enables its size number, Hug / Fill / Fixed stay three different things.
// From Blender's N panel (number side): XYZ stacked fields, horizontal scrub, a vertical sweep edits several at once, a per-axis lock, Alt = all selected.
// Dropped: the four-mode strip (move / scale / rotate / anchor are four instruments side by side instead, always live), keys 1-4 and Tab,
// the slide-under dotted field on the pad (the frame outline says the same), justify / align dots, transition, ADVANCED fold, the child-in-layout page.
import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../kit.dart';
import '../parts/controls.dart' show PillSwitch, Segmented;
import '../tokens.dart';
import 'inspector_parts.dart' show Cap, Chip;
import 'foundations/text_hand.dart';
part 'inspector_gui_set_2.dart';
part 'inspector_gui_set_3.dart';
part 'inspector_gui_set_4.dart';
part 'inspector_gui_set_5.dart';
part 'inspector_gui_set_6.dart';

// ---- the document: one set of values that every hand and every field edits ---------------------------------------------------

enum AxisTone { fam, rgb, none }

/// How the same Transform / Camera / Layout is offered: numbers only, instruments with readouts, or both.
enum Mode { fields, gui, both }

const _defaults = <String, double>{
  'space': 1, 'pos.x': 180, 'pos.y': 60, 'pos.z': 0, 'rot.z': 30, 'rot.x': 0, 'rot.y': 0, 'rot.axis': 0,
  'scale.x': 1, 'scale.y': 1, 'link': 1, 'anchor.x': .5, 'anchor.y': .5,
  'cam.tx': 0, 'cam.ty': 0, 'cam.pitch': -12, 'cam.yaw': -28, 'cam.dist': 1, 'cam.roll': 0, 'cam.layer': 0,
  'lay.on': 1, 'lay.cols': 3, 'lay.gap': 24, 'lay.pad': 32, 'lay.hs': 0, 'lay.vs': 0, 'lay.w': 420, 'lay.h': 260,
};

class GDoc extends ChangeNotifier {
  final Map<String, double> v = {..._defaults};
  final Set<String> mixed = {}, locks = {};
  int weight = 1; // Gui.readoutInk step
  bool locked = false; // the layer is locked: every hand and field is disabled
  AxisTone tone = AxisTone.fam;
  String scope = 'this layer';
  int steps = 0;
  Map<String, double>? _snap;
  Set<String>? _mixedSnap;

  double operator [](String id) => v[id]!;
  bool get live => _snap != null;

  /// The colour of axis 0 / 1 / 2 under decision B; null = letters only.
  Color? ax(int i) => switch (tone) { AxisTone.fam => Gui.fam[i], AxisTone.rgb => Gui.rgb[i], AxisTone.none => null };

  void begin() {
    _snap ??= Map.of(v);
    _mixedSnap ??= Set.of(mixed);
  }

  /// A gesture is one undo step, and only if it changed something.
  void end() {
    final s = _snap;
    _snap = null;
    _mixedSnap = null;
    if (s != null && s.entries.any((e) => v[e.key] != e.value)) {
      steps++;
    }
    notifyListeners();
  }

  /// Esc during a drag: everything is where the gesture found it.
  void cancel() {
    final s = _snap;
    if (s == null) {
      return;
    }
    v
      ..clear()
      ..addAll(s);
    mixed
      ..clear()
      ..addAll(_mixedSnap!);
    _snap = null;
    _mixedSnap = null;
    notifyListeners();
  }

  void set(String id, double x) {
    v[id] = x;
    notifyListeners();
  }

  void add(String id, double dx) => set(id, v[id]! + dx);

  /// A typed number sets the absolute value and ends the mixed state.
  void type(String id, double x) {
    begin();
    mixed.remove(id);
    set(id, x);
    end();
  }

  void toggleLock(String id) {
    if (!locks.remove(id)) {
      locks.add(id);
    }
    notifyListeners();
  }

  void poke() => notifyListeners();
}

// ---- small drawing helpers -------------------------------------------------------------------------------------------------------

Paint _ln(Color c) => Paint()
  ..color = c
  ..style = PaintingStyle.stroke
  ..strokeWidth = 1;
Paint _fl(Color c) => Paint()..color = c;

Offset _rot(Offset v, double a) => Offset(v.dx * math.cos(a) - v.dy * math.sin(a), v.dx * math.sin(a) + v.dy * math.cos(a));

double _wrap(double da) {
  while (da > math.pi) {
    da -= 2 * math.pi;
  }
  while (da < -math.pi) {
    da += 2 * math.pi;
  }
  return da;
}

class _Fn extends CustomPainter {
  const _Fn(this.f);
  final void Function(Canvas, Size) f;
  @override
  void paint(Canvas canvas, Size size) => f(canvas, size);
  @override
  bool shouldRepaint(_Fn old) => true;
}

void _well(Canvas cv, Size sz, {Color fill = N.g07}) {
  final r = RRect.fromRectAndRadius(Offset.zero & sz, const Radius.circular(Gui.radius));
  cv.drawRRect(r, _fl(fill));
  cv.drawRRect(r.deflate(.5), _ln(N.g20));
}

String _fmt(double v, int dec) {
  final s = v.toStringAsFixed(dec);
  return double.parse(s) == 0 ? (0.0).toStringAsFixed(dec) : s;
}

// ---- the hand: one pointer contract for every instrument -----------------------------------------------------------------

/// Pick what is under the pointer (null = nothing), then drag it. Esc during a drag reverts, Shift is fine, a locked layer is inert.
class _Hand extends StatefulWidget {
  const _Hand({
    required this.d,
    required this.size,
    required this.paint,
    required this.pick,
    required this.drag,
    this.start,
    this.tap,
    this.overlay,
    this.onHot,
  });
  final GDoc d;
  final Size size;
  final void Function(Canvas, Size, Object? hot, bool live) paint;
  final Object? Function(Offset p) pick;
  final void Function(Object g, Offset p, Offset last, bool fine) drag;
  final void Function(Object g, Offset p, bool fine)? start;
  final void Function(Object g, Offset p)? tap; // a click that did not move
  final Widget Function(Object? hot)? overlay;
  final ValueChanged<Object?>? onHot; // what is under the pointer, for a readout outside the canvas
  @override
  State<_Hand> createState() => _HandState();
}

class _HandState extends State<_Hand> {
  final focus = FocusNode(debugLabel: 'hand');
  Object? grab;
  Offset down = Offset.zero, last = Offset.zero, hoverAt = Offset.zero;
  bool moved = false, inside = false;
  Object? lastHot;

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  bool get fine => HardwareKeyboard.instance.isShiftPressed;
  bool get enabled => !widget.d.locked;

  void _end() {
    if (grab != null) {
      widget.d.end();
    }
    setState(() => grab = null);
  }

  @override
  Widget build(BuildContext context) {
    final hot = grab ?? (inside && enabled ? widget.pick(hoverAt) : null);
    if (hot != lastHot) {
      lastHot = hot;
      WidgetsBinding.instance.addPostFrameCallback((_) => widget.onHot?.call(hot));
    }
    return Focus(
      focusNode: focus,
      onKeyEvent: (_, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && grab != null) {
          widget.d.cancel();
          setState(() => grab = null);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: MouseRegion(
        cursor: hot != null ? SystemMouseCursors.grab : SystemMouseCursors.basic,
        onEnter: (_) => inside = true,
        onExit: (_) => setState(() => inside = false),
        onHover: (e) => setState(() => hoverAt = e.localPosition),
        child: Listener(
          onPointerDown: (e) {
            if (!enabled) {
              return;
            }
            focus.requestFocus();
            down = last = e.localPosition;
            moved = false;
            final g = widget.pick(down);
            if (g != null) {
              widget.d.begin();
              widget.start?.call(g, down, fine);
              setState(() => grab = g);
            }
          },
          onPointerMove: (e) {
            final g = grab;
            if (g == null) {
              return;
            }
            if ((e.localPosition - down).distance > 2) {
              moved = true;
            }
            widget.drag(g, e.localPosition, last, fine);
            last = e.localPosition;
          },
          onPointerUp: (e) {
            final g = grab;
            if (g != null && !moved) {
              widget.tap?.call(g, e.localPosition);
            }
            _end();
          },
          onPointerCancel: (_) {
            widget.d.cancel(); // an interrupted drag is let go, whatever it was holding
            setState(() => grab = null);
          },
          child: SizedBox.fromSize(
            size: widget.size,
            child: Stack(children: [
              Positioned.fill(child: CustomPaint(painter: _Fn((c, s) => widget.paint(c, s, hot, grab != null)))),
              if (widget.overlay != null) Positioned.fill(child: IgnorePointer(child: widget.overlay!(hot))),
            ]),
          ),
        ),
      ),
    );
  }
}

Color _stroke(GDoc d, Object? hot, bool live) => d.locked ? Role.disabled : (live ? C.mode : (hot != null ? N.g100 : Gui.graphicInk[d.weight]));

// ---- number fields: Blender-style XYZ stack, also used one at a time as the readout beside an instrument -------------------

class Spec {
  const Spec(this.id, this.tag, {this.axis, this.color, this.scale = 1, this.unit = '', this.dec = 1, this.per = 1, this.on = true, this.word});
  final String id, tag, unit;
  final int? axis; // 0 X, 1 Y, 2 Z: coloured by decision B
  final Color? color; // a role colour that is not an axis (camera target / orbit / framing / roll)
  final double scale, per; // display = value x scale; per = display units per px of scrub
  final int dec;
  final bool on; // false: shown, not editable, says why through [word]
  final String? word;
}

/// One or several stacked fields. Drag horizontally = scrub; press on one and drag vertically across the others = edit them together;
/// a small tab at the right locks one axis; Alt = apply to all selected; Shift = fine; Esc = revert; a click types.
class XyzStack extends StatefulWidget {
  const XyzStack(this.d, this.specs, {super.key, this.gap = 1});
  final GDoc d;
  final double gap; // 1 = joined Blender-style stack; Gui.rowGap = separate fields
  final List<Spec> specs;
  @override
  State<XyzStack> createState() => _XyzStackState();
}
