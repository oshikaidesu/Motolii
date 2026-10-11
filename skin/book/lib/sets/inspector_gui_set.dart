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

class _XyzStackState extends State<XyzStack> {
  final focus = FocusNode(debugLabel: 'xyz'), editFocus = FocusNode(debugLabel: 'edit');
  final ctrl = TextEditingController();
  Set<int> active = {};
  int? downRow, edit;
  Offset start = Offset.zero;
  double lastX = 0;
  bool moved = false, hover = false, dragging = false;
  static const rowH = Gui.field;
  double get gap => widget.gap;

  GDoc get d => widget.d;
  List<Spec> get specs => widget.specs;

  @override
  void dispose() {
    focus.dispose();
    editFocus.dispose();
    ctrl.dispose();
    super.dispose();
  }

  bool _can(int i) => !d.locked && specs[i].on && !d.locks.contains(specs[i].id);
  int _rowAt(double y) => (y / (rowH + gap)).floor().clamp(0, specs.length - 1);

  void _commitEdit() {
    final i = edit;
    if (i == null) {
      return;
    }
    final x = double.tryParse(ctrl.text.trim());
    setState(() => edit = null);
    if (x != null) {
      d.type(specs[i].id, x / specs[i].scale);
    }
  }

  void _escape() {
    if (dragging) {
      d.cancel();
      setState(() {
        dragging = false;
        active = {};
      });
    } else if (edit != null) {
      setState(() => edit = null);
    }
  }

  @override
  Widget build(BuildContext context) {
    final width = specs.length;
    return Focus(
      focusNode: focus,
      onKeyEvent: (_, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && (dragging || edit != null)) {
          _escape();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: MouseRegion(
        cursor: d.locked ? SystemMouseCursors.basic : SystemMouseCursors.resizeLeftRight,
        onEnter: (_) => setState(() => hover = true),
        onExit: (_) => setState(() => hover = false),
        child: LayoutBuilder(builder: (context, box) {
          return Listener(
            onPointerDown: (e) {
              if (d.locked || edit != null) {
                return;
              }
              final i = _rowAt(e.localPosition.dy);
              if (!specs[i].on) {
                return;
              }
              if (e.localPosition.dx > box.maxWidth - 16 && !d.locked) {
                d.toggleLock(specs[i].id); // the lock tab
                return;
              }
              focus.requestFocus();
              downRow = i;
              start = e.localPosition;
              lastX = e.localPosition.dx;
              moved = false;
              if (_can(i)) {
                d.begin();
                setState(() {
                  dragging = true;
                  active = {i};
                });
              }
            },
            onPointerMove: (e) {
              if (!dragging || downRow == null) {
                return;
              }
              if ((e.localPosition - start).distance > 3) {
                moved = true;
              }
              final r = _rowAt(e.localPosition.dy), a = math.min(downRow!, r), b = math.max(downRow!, r);
              final next = {for (var i = a; i <= b; i++) if (_can(i)) i};
              final dx = e.localPosition.dx - lastX;
              lastX = e.localPosition.dx;
              final hk = HardwareKeyboard.instance;
              // one rule for the whole panel: lab default Shift = fine; the R1 run sets Shift x10, Alt x0.1 (options-inspector.md D-I3)
              final mult = GuiRule.shiftBig ? (hk.isShiftPressed ? 10.0 : (hk.isAltPressed ? .1 : 1.0)) : (hk.isShiftPressed ? Gui.fine : 1.0);
              d.scope = HardwareKeyboard.instance.isAltPressed ? 'all 3 selected' : 'this layer';
              if (moved && dx != 0) {
                for (final i in next) {
                  d.add(specs[i].id, dx * specs[i].per * mult / specs[i].scale);
                }
              }
              setState(() => active = next);
            },
            onPointerUp: (e) {
              if (!dragging) {
                return;
              }
              final i = downRow;
              d.end();
              setState(() {
                dragging = false;
                active = {};
              });
              if (!moved && i != null) {
                final s = specs[i];
                ctrl.text = d.mixed.contains(s.id) ? '' : _fmt(d[s.id] * s.scale, s.dec); // a mixed value opens empty and keeps saying — until a number is typed
                setState(() => edit = i);
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  editFocus.requestFocus();
                  ctrl.selection = TextSelection(baseOffset: 0, extentOffset: ctrl.text.length);
                });
              }
            },
            onPointerCancel: (_) {
              d.cancel();
              setState(() {
                dragging = false;
                active = {};
              });
            },
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              for (var i = 0; i < width; i++) ...[
                if (i > 0) SizedBox(height: gap),
                _row(i, box.maxWidth < 100),
              ],
            ]),
          );
        }),
      ),
    );
  }

  Widget _row(int i, bool narrow) {
    final s = specs[i], first = i == 0 || gap > 1, last = i == specs.length - 1 || gap > 1;
    final mixed = d.mixed.contains(s.id), locked = d.locks.contains(s.id), on = s.on && !d.locked, act = active.contains(i);
    final col = s.color ?? (s.axis == null ? null : d.ax(s.axis!));
    final r = Radius.circular(Gui.radius);
    final ink = !on ? N.g56 : (locked ? N.g56 : N.g95);
    Widget value;
    if (edit == i) {
      final field = EditableText(
        controller: ctrl,
        focusNode: editFocus,
        style: T.value(N.g95),
        cursorColor: Role.selected,
        backgroundCursorColor: N.g20,
        textAlign: TextAlign.right,
        onSubmitted: (_) => _commitEdit(),
        onTapOutside: (_) => _commitEdit(),
      );
      value = mixed
          ? Stack(alignment: Alignment.centerRight, children: [
              ValueListenableBuilder<TextEditingValue>(valueListenable: ctrl, builder: (_, v, _) => v.text.isEmpty ? Text('—', style: T.value(N.g63)) : const SizedBox.shrink()),
              field,
            ])
          : field;
    } else {
      final plain = s.word ?? (mixed ? '—' : _fmt(d[s.id] * s.scale, s.dec));
      // narrow cell (< 100 px): the unit joins the value so nothing is cut; the value is scaled down before it is ever clipped
      value = FittedBox(
        fit: BoxFit.scaleDown,
        alignment: Alignment.centerRight,
        child: Text.rich(
          TextSpan(text: plain, children: [if (narrow && s.unit.isNotEmpty && s.word == null && !mixed) TextSpan(text: ' ${s.unit}', style: T.label(N.g56))]),
          maxLines: 1,
          textAlign: TextAlign.right,
          style: T.value(s.word != null || mixed ? N.g63 : ink),
        ),
      );
    }
    return Container(
      height: rowH,
      padding: const EdgeInsets.only(left: 6, right: 5),
      decoration: BoxDecoration(
        color: act ? N.g20 : N.g07,
        borderRadius: BorderRadius.vertical(top: first ? r : Radius.zero, bottom: last ? r : Radius.zero),
        border: Border.all(color: act ? N.g44 : N.g20),
      ),
      child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
        // the axis mark: a 1 px bar in the axis colour; mixed = a neutral ring; none = letters only
        SizedBox(
          width: 7,
          child: col != null && on ? Align(alignment: Alignment.centerLeft, child: Container(width: 1, height: 10, color: mixed ? N.g56 : col)) : null, // mixed: the axis bar goes neutral; the ring sits at the right end, away from the letter
        ),
        SizedBox(width: 14, child: Text(s.tag, maxLines: 1, overflow: TextOverflow.clip, style: T.label(on ? N.g63 : N.g56))),
        Expanded(child: value),
        if (!narrow && s.unit.isNotEmpty && s.word == null && !mixed) Padding(padding: const EdgeInsets.only(left: 3), child: Text(s.unit, style: T.label(N.g56))),
        SizedBox(
          width: narrow && !locked && !mixed ? 0 : 11,
          child: (hover || locked) && s.on && !d.locked
              ? Align(alignment: Alignment.centerRight, child: Container(width: 6, height: 6, decoration: BoxDecoration(color: locked ? N.g76 : null, border: Border.all(color: locked ? N.g76 : N.g44))))
              : (mixed ? Align(alignment: Alignment.centerRight, child: Container(width: 6, height: 6, decoration: BoxDecoration(shape: BoxShape.circle, border: Border.all(color: N.g56)))) : null),
        ),
      ]),
    );
  }
}

// ---- 1. the Transform instruments --------------------------------------------------------------------------------------------

/// Position pad: the body sits where Position says; grab it or press anywhere (Shift = no jump, fine) and drag.
class _PadGeo {
  _PadGeo(this.size, this.d, {this.boost = 1});
  final double boost; // a bigger body, for the unified gizmo
  final Size size;
  final GDoc d;
  double get k => Gui.span / size.width; // composition px per pad px
  Offset get c => size.center(Offset.zero);
  Offset at(double x, double y) => c + Offset(x, y) / k;
  Offset get puck {
    final p = at(d['pos.x'], d['pos.y']);
    return Offset(p.dx.clamp(4.0, size.width - 4), p.dy.clamp(4.0, size.height - 4));
  }

  double get depth => d['space'] == 2 ? (1 + d['pos.z'] / (Gui.zSpan * 2)).clamp(.6, 1.4) : 1.0;
  Offset get half {
    var hx = boost * Gui.bodyW * d['scale.x'].abs().clamp(.3, 1.6) * depth * (size.height < 100 ? .75 : 1);
    var hy = boost * Gui.bodyH * d['scale.y'].abs().clamp(.3, 1.6) * depth * (size.height < 100 ? .75 : 1);
    if (d['space'] > 0) {
      hx *= math.cos(d['rot.y'] * math.pi / 180).abs().clamp(.25, 1.0);
      hy *= math.cos(d['rot.x'] * math.pi / 180).abs().clamp(.25, 1.0);
    }
    return Offset(hx, hy);
  }

  double get rad => d['rot.z'] * math.pi / 180;
  List<Offset> corners(Offset c0, Offset h) => [for (final s in const [Offset(-1, -1), Offset(1, -1), Offset(1, 1), Offset(-1, 1)]) c0 + _rot(Offset(s.dx * h.dx, s.dy * h.dy), rad)];
}

Widget _padHand(GDoc d, Size size) {
  final g = _PadGeo(size, d);
  return _Hand(
    d: d,
    size: size,
    pick: (p) => (p - g.puck).distance < 90 ? ((_rot(p - g.puck, -g.rad).dx.abs() <= g.half.dx + 6 && _rot(p - g.puck, -g.rad).dy.abs() <= g.half.dy + 6) ? 'body' : 'free') : 'free',
    start: (grab, p, fine) {
      if (grab == 'free' && !fine) {
        d.set('pos.x', (p.dx - g.c.dx) * g.k);
        d.set('pos.y', (p.dy - g.c.dy) * g.k);
      }
    },
    drag: (grab, p, last, fine) {
      final dd = (p - last) * g.k * (fine ? Gui.fine : 1);
      d.add('pos.x', dd.dx);
      d.add('pos.y', dd.dy);
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final c = g.c;
      for (final f in const [1 / 3, 2 / 3]) {
        cv.drawLine(Offset(sz.width * f, 0), Offset(sz.width * f, sz.height), _ln(N.g15));
        cv.drawLine(Offset(0, sz.height * f), Offset(sz.width, sz.height * f), _ln(N.g15));
      }
      cv.drawLine(Offset(c.dx, 0), Offset(c.dx, sz.height), _ln(N.g20));
      cv.drawLine(Offset(0, c.dy), Offset(sz.width, c.dy), _ln(N.g20));
      cv.drawRect(Rect.fromCenter(center: c, width: Gui.comp.width / g.k, height: Gui.comp.height / g.k), _ln(N.g26)); // the composition frame
      final h = g.half;
      final pk = g.puck, cs = g.corners(pk, h), body = Path()..addPolygon(cs, true);
      cv.drawPath(body, _fl(d.locked ? N.g13 : (live ? N.g20 : N.g15)));
      cv.drawPath(body, _ln(_stroke(d, hot, live)));
      final top = (cs[0] + cs[1]) / 2, up = _rot(const Offset(0, -1), g.rad);
      cv.drawLine(top, top + up * 5, _ln(_stroke(d, hot, live))); // a notch so a turn can be read
      final pv = pk + _rot(Offset((d['anchor.x'] * 2 - 1) * h.dx, (d['anchor.y'] * 2 - 1) * h.dy), g.rad); // the pivot lives on the body
      cv.drawCircle(pv, 2.5, _fl(N.g95));
      // where X and Y are: a tick on the top edge and on the left edge, in the axis colour of their number fields
      cv.drawLine(Offset(pk.dx, 0), Offset(pk.dx, 6), _ln(d.ax(0) ?? N.g63));
      cv.drawLine(Offset(0, pk.dy), Offset(6, pk.dy), _ln(d.ax(1) ?? N.g63));
    },
  );
}

/// The Z rail: depth, only outside 2D.
Widget _railHand(GDoc d, Size size) {
  final k = Gui.zSpan / size.height;
  Offset puck() => Offset(size.width / 2, (size.height / 2 - d['pos.z'] / k).clamp(5.0, size.height - 5));
  return _Hand(
    d: d,
    size: size,
    pick: (p) => 'z',
    start: (g, p, fine) {
      if (!fine) {
        d.set('pos.z', (size.height / 2 - p.dy) * k);
      }
    },
    drag: (g, p, last, fine) => d.add('pos.z', -(p.dy - last.dy) * k * (fine ? Gui.fine : 1)),
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final cx = sz.width / 2;
      cv.drawLine(Offset(cx, 4), Offset(cx, sz.height - 4), _ln(N.g26));
      for (var y = 10.0; y < sz.height - 4; y += 12) {
        cv.drawLine(Offset(cx - 3, y), Offset(cx + 3, y), _ln(N.g20));
      }
      cv.drawLine(Offset(cx - 6, sz.height / 2), Offset(cx + 6, sz.height / 2), _ln(N.g44));
      final r = Rect.fromCenter(center: puck(), width: 14, height: 7);
      cv.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(2)), _fl(d.locked ? N.g13 : N.g20));
      cv.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(2)), _ln(_stroke(d, hot, live)));
      cv.drawLine(Offset(0, puck().dy), Offset(4, puck().dy), _ln(d.ax(2) ?? N.g63));
    },
  );
}

/// Rotation dial: grab anywhere on or inside the ring and turn. Turns are kept: 450 stays 450.
Widget _dialHand(GDoc d, Size size) {
  final id = const ['rot.z', 'rot.x', 'rot.y'][d['rot.axis'].round().clamp(0, 2)];
  final axis = const [2, 0, 1][d['rot.axis'].round().clamp(0, 2)];
  final c = size.center(Offset.zero), r = math.min(size.width, size.height) / 2 - 10;
  final turns = (d[id] / 360).truncate();
  return _Hand(
    d: d,
    size: size,
    pick: (p) => (p - c).distance < r + 14 ? 'ring' : null,
    drag: (g, p, last, fine) => d.add(id, _wrap((p - c).direction - (last - c).direction) * 180 / math.pi * (fine ? Gui.fine : 1)),
    overlay: turns == 0 ? null : (_) => Center(child: Text('$turns×', style: T.micro(N.g63))),
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      cv.drawCircle(c, r, _ln(N.g26));
      for (var i = 0; i < 12; i++) {
        final a = i * math.pi / 6, major = i % 3 == 0, u = Offset(math.cos(a), math.sin(a));
        cv.drawLine(c + u * (r - (major ? 5 : 3)), c + u * r, _ln(major ? N.g63 : N.g38));
      }
      final col = d.ax(axis) ?? N.g95, a0 = -math.pi / 2, sweep = (d[id] % 360) * math.pi / 180;
      cv.drawArc(Rect.fromCircle(center: c, radius: r - 8), a0, d[id] >= 0 ? sweep : sweep - 2 * math.pi, false, _ln(d.locked ? Role.disabled : col.withValues(alpha: .8)));
      final a = a0 + d[id] * math.pi / 180, tip = c + Offset(math.cos(a), math.sin(a)) * r;
      cv.drawLine(c, tip, _ln(N.g44));
      cv.drawCircle(tip, 4, _fl(d.locked ? N.g38 : N.g95));
      cv.drawCircle(tip, 4, _ln(_stroke(d, hot, live)));
    },
  );
}

/// Scale handles: corners scale both (one factor while Link is on), edges scale one axis. A factor is applied to the value, never an absolute jump.
Widget _scaleHand(GDoc d, Size size) {
  final c = size.center(Offset.zero), u = math.min(size.width, size.height) * .27;
  Offset half() => Offset((u * d['scale.x'].abs()).clamp(8.0, size.width / 2 - 6), (u * d['scale.y'].abs()).clamp(8.0, size.height / 2 - 6));
  List<Offset> pts() {
    final h = half();
    return [
      c + Offset(-h.dx, -h.dy), c + Offset(h.dx, -h.dy), c + Offset(h.dx, h.dy), c + Offset(-h.dx, h.dy), // 0-3 corners
      c + Offset(-h.dx, 0), c + Offset(h.dx, 0), c + Offset(0, -h.dy), c + Offset(0, h.dy), // 4-7 edges
    ];
  }

  return _Hand(
    d: d,
    size: size,
    pick: (p) {
      final ps = pts();
      for (var i = 0; i < 8; i++) {
        if ((p - ps[i]).distance < 10) {
          return i;
        }
      }
      return null;
    },
    drag: (g, p, last, fine) {
      final i = g as int;
      var fx = 1.0, fy = 1.0;
      double ratio(double a, double b) => b.abs() < 2 ? 1.0 : (a / b).abs();
      if (i < 4 || i == 4 || i == 5) {
        fx = ratio(p.dx - c.dx, last.dx - c.dx);
      }
      if (i < 4 || i == 6 || i == 7) {
        fy = ratio(p.dy - c.dy, last.dy - c.dy);
      }
      if (fine) {
        fx = math.pow(fx, Gui.fine).toDouble();
        fy = math.pow(fy, Gui.fine).toDouble();
      }
      if (d['link'] > .5) {
        final f = i < 4 ? ((p - c).distance / math.max(2, (last - c).distance)) : (i < 6 ? fx : fy);
        fx = fy = fine ? math.pow(f, Gui.fine).toDouble() : f;
      }
      d.set('scale.x', d['scale.x'] * fx);
      d.set('scale.y', d['scale.y'] * fy);
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      cv.drawRect(Rect.fromCenter(center: c, width: u * 2, height: u * 2), _ln(N.g26)); // 100 %
      final ps = pts(), h = half(), col = _stroke(d, hot, live);
      final r = Rect.fromCenter(center: c, width: h.dx * 2, height: h.dy * 2);
      cv.drawRect(r, _fl(d.locked ? N.g13 : N.g15));
      cv.drawRect(r, _ln(col));
      for (var i = 0; i < 8; i++) {
        final big = i < 4, on = hot == i;
        final sq = Rect.fromCenter(center: ps[i], width: big ? 7 : 5, height: big ? 7 : 5);
        cv.drawRect(sq, _fl(on ? N.g95 : N.g07));
        cv.drawRect(sq, _ln(d.locked ? Role.disabled : (on ? C.mode : N.g76)));
      }
    },
  );
}

const _anchorNames = ['Top left', 'Top', 'Top right', 'Left', 'Centre', 'Right', 'Bottom left', 'Bottom', 'Bottom right'];
int? _anchorIndex(GDoc d) {
  for (var i = 0; i < 9; i++) {
    if ((d['anchor.x'] - (i % 3) / 2).abs() < .001 && (d['anchor.y'] - (i ~/ 3) / 2).abs() < .001) {
      return i;
    }
  }
  return null;
}

/// Nine-point anchor: one click. A typed value off the nine reads Custom (a hollow mark where it really is).
Widget _anchorHand(GDoc d, Size size) {
  final c = size.center(Offset.zero), box = Rect.fromCenter(center: c, width: size.width - 22, height: (size.width - 22) * .66);
  Offset pt(int i) => Offset(box.left + box.width * (i % 3) / 2, box.top + box.height * (i ~/ 3) / 2);
  return _Hand(
    d: d,
    size: size,
    pick: (p) {
      for (var i = 0; i < 9; i++) {
        if ((p - pt(i)).distance < 10) {
          return i;
        }
      }
      return null;
    },
    drag: (g, p, last, fine) {},
    tap: (g, p) {
      final i = g as int;
      d.set('anchor.x', (i % 3) / 2);
      d.set('anchor.y', (i ~/ 3) / 2);
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      cv.drawRect(box, _ln(N.g38));
      final sel = _anchorIndex(d);
      for (var i = 0; i < 9; i++) {
        final on = sel == i, hv = hot == i;
        cv.drawCircle(pt(i), on ? 4 : (hv ? 4 : Gui.dot), on ? _fl(d.locked ? Role.disabled : N.g95) : _ln(hv ? N.g95 : N.g56));
        if (on && !d.locked) {
          cv.drawCircle(pt(i), 6, _ln(C.mode));
        }
      }
      if (sel == null) {
        final o = Offset(box.left + box.width * d['anchor.x'], box.top + box.height * d['anchor.y']);
        cv.drawCircle(o, 4, _ln(N.g95));
      }
    },
  );
}

/// 2D / 2.5D / 3D as three small drawings: choosing is graphic. Changes no value, only what is revealed.
class _SpaceChips extends StatelessWidget {
  const _SpaceChips(this.d);
  final GDoc d;
  static const names = ['2D', '2.5D', '3D'];
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(7)),
        child: Row(children: [
          for (var i = 0; i < 3; i++)
            Expanded(
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: d.locked
                    ? null
                    : () {
                        d.begin();
                        d.set('space', i.toDouble());
                        d.end();
                      },
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 120),
                  curve: Curves.easeOut,
                  height: Gui.tile,
                  decoration: BoxDecoration(
                    color: d['space'] == i ? N.g20 : null,
                    borderRadius: BorderRadius.circular(5),
                    border: Border(bottom: BorderSide(color: d['space'] == i ? Role.selected : const Color(0x00000000))),
                  ),
                  child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
                    SizedBox(width: 26, height: 14, child: CustomPaint(painter: _Fn((cv, s) => _spaceIcon(cv, s, i, d.locked ? Role.disabled : (d['space'] == i ? N.g95 : N.g63))))),
                    const SizedBox(height: 4),
                    Text(names[i], style: T.micro(d['space'] == i ? N.g95 : N.g63)),
                  ]),
                ),
              ),
            ),
        ]),
      );
}

void _spaceIcon(Canvas cv, Size s, int i, Color col) {
  final r = Rect.fromCenter(center: s.center(Offset.zero), width: 16, height: 10);
  switch (i) {
    case 0:
      cv.drawRect(r, _ln(col));
    case 1:
      cv.drawRect(r.shift(const Offset(4, -2)), _ln(col.withValues(alpha: .5)));
      cv.drawRect(r.shift(const Offset(-2, 1)), _ln(col));
    default:
      final f = r.shift(const Offset(-3, 2)), b = r.shift(const Offset(3, -2));
      for (final (a, c) in [(f.topLeft, b.topLeft), (f.topRight, b.topRight), (f.bottomRight, b.bottomRight), (f.bottomLeft, b.bottomLeft)]) {
        cv.drawLine(a, c, _ln(col.withValues(alpha: .5)));
      }
      cv.drawRect(b, _ln(col.withValues(alpha: .5)));
      cv.drawRect(f, _ln(col));
  }
}

// ---- the specs shared by the number side and the readouts --------------------------------------------------------------------

List<Spec> _posSpecs(GDoc d) => [
      const Spec('pos.x', 'X', axis: 0, unit: 'px'),
      const Spec('pos.y', 'Y', axis: 1, unit: 'px'),
      if (d['space'] > 0) const Spec('pos.z', 'Z', axis: 2, unit: 'px'),
    ];
List<Spec> _rotSpecs(GDoc d) => d['space'] > 0
    ? const [Spec('rot.x', 'X', axis: 0, unit: '°', per: .5), Spec('rot.y', 'Y', axis: 1, unit: '°', per: .5), Spec('rot.z', 'Z', axis: 2, unit: '°', per: .5)]
    : const [Spec('rot.z', 'Z', axis: 2, unit: '°', per: .5)];
const _scaleSpecs = [Spec('scale.x', 'X', axis: 0, scale: 100, unit: '%', per: .5), Spec('scale.y', 'Y', axis: 1, scale: 100, unit: '%', per: .5)];
const _anchorSpecs = [Spec('anchor.x', 'X', axis: 0, scale: 100, unit: '%', per: .5), Spec('anchor.y', 'Y', axis: 1, scale: 100, unit: '%', per: .5)];

Widget _title(String t, {Widget? trailing}) => SizedBox(
      height: 16,
      child: Row(children: [
        Text(t.toUpperCase(), style: T.micro(N.g56).copyWith(letterSpacing: .4)),
        const SizedBox(width: 6),
        Expanded(child: Container(height: 1, color: N.g15)),
        if (trailing != null) ...[const SizedBox(width: 6), trailing],
      ]),
    );

Widget _lockedWord(GDoc d) => d.locked ? Text('Locked', style: T.label(N.g76)) : const SizedBox.shrink();
Widget _gap(double h) => SizedBox(height: h);

/// (b) and the base of (c): the instruments, each with its numbers beside it as a quiet readout.
class TransformGui extends StatelessWidget {
  const TransformGui(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    final three = d['space'] > 0;
    const w = Gui.content, colW = (w - 16) / 3;
    final axisIdx = d['rot.axis'].round().clamp(0, 2);
    final anchorName = _anchorIndex(d) == null ? 'Custom' : _anchorNames[_anchorIndex(d)!];
    Widget col(String t, Widget hand, Widget below, {Widget? trailing}) => SizedBox(
          width: colW,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            SizedBox(height: 20, child: Row(children: [Text(t, style: T.label(N.g63)), const Spacer(), ?trailing])),
            hand,
            _gap(4),
            below,
          ]),
        );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Space', trailing: _lockedWord(d)),
      _gap(6),
      _SpaceChips(d),
      _gap(12),
      _title('Position'),
      _gap(6),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        _padHand(d, Size(three ? w - Gui.rail - 6 : w, Gui.padH)),
        if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.padH))],
      ]),
      _gap(4),
      Row(children: [for (final (i, s) in _posSpecs(d).indexed) ...[if (i > 0) const SizedBox(width: 4), Expanded(child: XyzStack(d, [s]))]]),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        col(
          'Rotation',
          _dialHand(d, const Size(colW, colW)),
          Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            XyzStack(d, [
              Spec(const ['rot.z', 'rot.x', 'rot.y'][axisIdx], const ['Z', 'X', 'Y'][axisIdx], axis: const [2, 0, 1][axisIdx], unit: '°', per: .5),
            ]),
            if (three) ...[
              _gap(4),
              Row(children: [
                for (var i = 0; i < 3; i++)
                  Chip(const ['Z', 'X', 'Y'][i], on: axisIdx == i, enabled: !d.locked, onTap: () {
                    d.set('rot.axis', i.toDouble());
                  }),
              ]),
            ],
          ]),
        ),
        const SizedBox(width: 8),
        col(
          'Scale',
          _scaleHand(d, Size(colW, colW)),
          XyzStack(d, _scaleSpecs, gap: Gui.rowGap),
          trailing: Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () {
            d.set('link', d['link'] > .5 ? 0 : 1); // changes no artwork, only how the next drag behaves
          }),
        ),
        const SizedBox(width: 8),
        col(
          'Anchor',
          _anchorHand(d, Size(colW, colW)),
          Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            XyzStack(d, _anchorSpecs, gap: Gui.rowGap),
            _gap(4),
            Text(anchorName, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56)),
          ]),
        ),
      ]),
    ]);
  }
}

/// (a) Blender-style: three XYZ stacks, nothing to grab.
class TransformFields extends StatelessWidget {
  const TransformFields(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    Widget grp(String t, List<Spec> s, {Widget? trailing}) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_title(t, trailing: trailing), _gap(6), XyzStack(d, s), _gap(12)]);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Space', trailing: _lockedWord(d)),
      _gap(6),
      _SpaceChips(d),
      _gap(12),
      grp('Position', _posSpecs(d)),
      grp('Rotation', _rotSpecs(d)),
      grp('Scale', _scaleSpecs, trailing: Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () => d.set('link', d['link'] > .5 ? 0 : 1))),
      grp('Anchor', _anchorSpecs),
    ]);
  }
}

/// (c) Both: the stacks are the primary, the instruments sit beside them as helpers and edit the same numbers.
class TransformBoth extends StatelessWidget {
  const TransformBoth(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) => _build(math.min(box.maxWidth, Gui.content)));

  // helpers take whatever width the column really has, minus the stacks: they shrink, they are never cut
  Widget _build(double w) {
    const stackW = 112.0, h = 74.0;
    final helperW = w - stackW - 8;
    Widget row(String t, List<Spec> s, Widget helper, {Widget? trailing}) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _title(t, trailing: trailing),
          _gap(6),
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
            SizedBox(width: stackW, child: XyzStack(d, s)),
            const SizedBox(width: 8),
            helper,
          ]),
          _gap(12),
        ]);
    final axisIdx = d['rot.axis'].round().clamp(0, 2);
    // with the stacks primary, the dial follows the field you last touched; here it follows the axis chip
    final rotHelper = Column(children: [
      _dialHand(d, Size(helperW, h - 22)),
      if (d['space'] > 0)
        Row(children: [
          for (var i = 0; i < 3; i++)
            Chip(const ['Z', 'X', 'Y'][i], on: axisIdx == i, enabled: !d.locked, onTap: () => d.set('rot.axis', i.toDouble())),
        ]),
    ]);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Space', trailing: _lockedWord(d)),
      _gap(6),
      _SpaceChips(d),
      _gap(12),
      row('Position', _posSpecs(d), _padHand(d, Size(helperW, h))),
      row('Rotation', _rotSpecs(d), rotHelper),
      row('Scale', _scaleSpecs, _scaleHand(d, Size(helperW, h)), trailing: Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () => d.set('link', d['link'] > .5 ? 0 : 1))),
      row('Anchor', _anchorSpecs, _anchorHand(d, Size(helperW, h))),
    ]);
  }
}

// ---- 2. the Camera face ---------------------------------------------------------------------------------------------------------

List<double> _eye(GDoc d) {
  final p = d['cam.pitch'] * math.pi / 180, y = d['cam.yaw'] * math.pi / 180;
  return [-math.sin(y) * math.cos(p), math.sin(p), -math.cos(y) * math.cos(p)];
}

class _CamGeo {
  _CamGeo(this.size, this.d);
  final Size size;
  final GDoc d;
  Offset get c => size.center(Offset.zero);
  // an oblique view of the unit sphere, so a camera straight in front of the target still stands apart from it
  Offset get eye {
    final e = _eye(d);
    var v = Offset(e[0] - .62 * e[2], e[1] + .46 * e[2]);
    if (v.distance < .6) {
      v = v.distance < 1e-3 ? const Offset(.6, 0) : v / v.distance * .6;
    }
    return c + v * (Gui.orbitR * .95);
  }

  bool get behind => _eye(d)[2] > 0;
  Offset get ray {
    final v = eye - c;
    return v.distance < 1e-3 ? const Offset(-.8, .6) : v / v.distance;
  }

  Offset get framing {
    final t = (.5 + .28 * (math.log(d['cam.dist']) / math.ln10)).clamp(.22, .9);
    return c + (eye - c) * t;
  }

  Offset get roll {
    final a = (-90 + d['cam.roll']) * math.pi / 180;
    return c + Offset(math.cos(a), math.sin(a)) * Gui.ringR;
  }
}

Widget _faceHand(GDoc d, Size size) {
  final g = _CamGeo(size, d);
  final tone = [Fam.face.c, Fam.attach.c, Fam.stagger.c, Fam.scatter.c];
  final byLayer = d['cam.layer'] > .5;
  return _Hand(
    d: d,
    size: size,
    pick: (p) {
      if ((p - g.c).distance < 9 && !byLayer) {
        return 'target';
      }
      if ((p - g.framing).distance < 10) {
        return 'framing';
      }
      if ((p - g.eye).distance < 12) {
        return 'eye';
      }
      if (((p - g.c).distance - Gui.ringR).abs() < 9) {
        return 'roll';
      }
      return null;
    },
    drag: (grab, p, last, fine) {
      final k = fine ? Gui.fine : 1.0, dd = (p - last) * k;
      switch (grab) {
        case 'target':
          d.add('cam.tx', dd.dx * 4);
          d.add('cam.ty', dd.dy * 4);
        case 'eye': // the camera follows the pointer around the target
          d.set('cam.pitch', (d['cam.pitch'] + dd.dy * .8).clamp(-89.0, 89.0));
          d.add('cam.yaw', -dd.dx * .8);
        case 'framing': // outward along the ray is farther
          d.set('cam.dist', (d['cam.dist'] * math.exp((dd.dx * g.ray.dx + dd.dy * g.ray.dy) * .02)).clamp(.1, 50.0));
        case 'roll':
          d.add('cam.roll', _wrap((p - g.c).direction - (last - g.c).direction) * 180 / math.pi * k);
      }
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final a = d.locked ? .4 : 1.0, c = g.c, eye = g.eye;
      final dots = _fl(N.g15); // a quiet field that slides under the target as it moves: the world moves, the face stays
      for (var x = (-d['cam.tx'] / 4) % 16; x < sz.width; x += 16) {
        for (var y = (-d['cam.ty'] / 4) % 16; y < sz.height; y += 16) {
          cv.drawCircle(Offset(x, y), .7, dots);
        }
      }
      cv.drawCircle(c, Gui.ringR, _ln(tone[3].withValues(alpha: (hot == 'roll' || live && hot == 'roll' ? .9 : .35) * a)));
      for (var i = 0; i < 12; i++) {
        final t = i * math.pi / 6, u = Offset(math.cos(t), math.sin(t));
        cv.drawLine(c + u * (Gui.ringR - 3), c + u * (Gui.ringR + 3), _ln(tone[3].withValues(alpha: .3 * a)));
      }
      cv.drawCircle(c, Gui.orbitR, _ln(tone[1].withValues(alpha: .5 * a)));
      cv.drawOval(Rect.fromCenter(center: c, width: Gui.orbitR * 2, height: Gui.orbitR * .7), _ln(tone[1].withValues(alpha: .25 * a)));
      cv.drawLine(eye, c, _ln(tone[2].withValues(alpha: .9 * a)));
      final fh = g.framing;
      cv.save();
      cv.translate(fh.dx, fh.dy);
      cv.rotate(g.ray.direction + math.pi / 2);
      final bar = Rect.fromCenter(center: Offset.zero, width: 12, height: 4);
      cv.drawRect(bar, _fl(N.g07));
      cv.drawRect(bar, _ln(tone[2].withValues(alpha: a)));
      cv.restore();
      cv.save();
      cv.translate(eye.dx, eye.dy);
      cv.rotate((c - eye).direction);
      final body = const Rect.fromLTRB(-9, -6, 5, 6), lens = Path()..moveTo(5, -3.5)..lineTo(11, -6.5)..lineTo(11, 6.5)..lineTo(5, 3.5)..close();
      cv.drawRect(body, _fl(g.behind ? N.g07 : tone[1].withValues(alpha: .25 * a))); // hollow when the eye is on the far side
      cv.drawRect(body, _ln(tone[1].withValues(alpha: a)));
      cv.drawPath(lens, _ln(tone[1].withValues(alpha: a)));
      cv.restore();
      cv.drawCircle(c, 5, _fl(byLayer ? N.g07 : tone[0].withValues(alpha: .25 * a)));
      cv.drawCircle(c, 5, _ln(tone[0].withValues(alpha: a)));
      final cross = _ln(tone[0].withValues(alpha: .8 * a));
      for (final o in const [Offset(-1, 0), Offset(1, 0), Offset(0, -1), Offset(0, 1)]) {
        cv.drawLine(c + o * 8, c + o * 11, cross);
      }
      final rh = g.roll;
      cv.drawCircle(rh, 4.5, _fl(N.g07));
      cv.drawCircle(rh, 4.5, _ln(tone[3].withValues(alpha: a)));
    },
  );
}

List<Spec> _camSpecs(GDoc d, String part) {
  final byLayer = d['cam.layer'] > .5;
  final role = [Fam.face.c, Fam.attach.c, Fam.stagger.c, Fam.scatter.c];
  switch (part) {
    case 'target':
      return [
        Spec('cam.tx', 'X', color: role[0], unit: 'px', per: 4, on: !byLayer, word: byLayer ? 'Jewel 02' : null),
        Spec('cam.ty', 'Y', color: role[0], unit: 'px', per: 4, on: !byLayer, word: byLayer ? 'its layer' : null),
      ];
    case 'orbit':
      return [Spec('cam.pitch', 'P', color: role[1], unit: '°', per: .5), Spec('cam.yaw', 'Y', color: role[1], unit: '°', per: .5)];
    case 'framing':
      return [Spec('cam.dist', 'F', color: role[2], unit: '×', dec: 2, per: .01)];
    default:
      return [Spec('cam.roll', 'R', color: role[3], unit: '°', per: .5)];
  }
}

Widget _camNumbers(GDoc d) {
  Widget part(String t, String key) => Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [Text(t, style: T.label(N.g63)), _gap(4), XyzStack(d, _camSpecs(d, key))]));
  return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
    Row(crossAxisAlignment: CrossAxisAlignment.start, children: [part('Target', 'target'), const SizedBox(width: 8), part('Orbit', 'orbit')]),
    _gap(8),
    Row(crossAxisAlignment: CrossAxisAlignment.start, children: [part('Framing', 'framing'), const SizedBox(width: 8), part('Roll', 'roll')]),
  ]);
}

class CameraGui extends StatelessWidget {
  const CameraGui(this.d, {super.key, this.mode = Mode.gui});
  final GDoc d;
  final Mode mode;
  @override
  Widget build(BuildContext context) {
    final face = _faceHand(d, const Size(Gui.content, Gui.faceH));
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Camera', trailing: _lockedWord(d)),
      _gap(6),
      if (mode == Mode.fields) _camNumbers(d) else if (mode == Mode.gui) ...[face, _gap(6), _camNumbers(d)] else ...[_camNumbers(d), _gap(8), face],
      if (mode != Mode.fields && d['cam.layer'] > .5) Cap('Target follows Jewel 02: the point fields disable and say why.'),
    ]);
  }
}

// ---- 3. the Layout mini-diagram ------------------------------------------------------------------------------------------------

class _LayGeo {
  _LayGeo(this.size, this.d);
  final Size size;
  final GDoc d;
  bool get on => d['lay.on'] > .5;
  int get cols => d['lay.cols'].round().clamp(1, Gui.layN);
  int get rows => (Gui.layN / cols).ceil();
  double get gap => on ? d['lay.gap'] * Gui.layK : 3;
  double get pad => on ? d['lay.pad'] * Gui.layK : 0;
  int get hs => on ? d['lay.hs'].round() : 0;
  int get vs => on ? d['lay.vs'].round() : 0;
  Rect get avail => Rect.fromLTRB(8, 8, size.width - 8, size.height - 8);
  Size get content => Size(cols * Gui.layCw + (cols - 1) * gap, rows * Gui.layCh + (rows - 1) * gap);
  Rect get parent {
    final w = hs == 0 ? content.width + 2 * pad : (hs == 1 ? avail.width : (d['lay.w'] * Gui.layK).clamp(40.0, avail.width));
    final h = vs == 0 ? content.height + 2 * pad : (vs == 1 ? avail.height : (d['lay.h'] * Gui.layK).clamp(30.0, avail.height));
    return Rect.fromLTWH(avail.left, avail.top, math.min(w, avail.width), math.min(h, avail.height));
  }

  Rect get inner => Rect.fromLTRB(parent.left + pad, parent.top + pad, math.max(parent.left + pad, parent.right - pad), math.max(parent.top + pad, parent.bottom - pad));
  List<Rect> get boxes => [for (var i = 0; i < Gui.layN; i++) Rect.fromLTWH(inner.left + (i % cols) * (Gui.layCw + gap), inner.top + (i ~/ cols) * (Gui.layCh + gap), Gui.layCw, Gui.layCh)];
  Offset get gapAt {
    final b = boxes;
    return cols > 1 ? Offset((b[0].right + b[1].left) / 2, b[0].center.dy) : Offset(b[0].center.dx, (b[0].bottom + b[1].top) / 2);
  }

  Offset get colAt => Offset(boxes[cols - 1].right + 7, boxes[0].center.dy);
  Offset get padAt => Offset(parent.left + pad, parent.top + pad);
  Offset get rightAt => Offset(parent.right, parent.center.dy);
  Offset get bottomAt => Offset(parent.center.dx, parent.bottom);
}

Widget _layoutHand(GDoc d, Size size) {
  final g = _LayGeo(size, d);
  return _Hand(
    d: d,
    size: size,
    pick: (p) {
      if ((p - g.colAt).distance < 9) {
        return 'cols';
      }
      if (!g.on) {
        return null; // Grid gates the rest
      }
      if ((p - g.gapAt).distance < 9) {
        return 'gap';
      }
      if ((p - g.padAt).distance < 9 && g.pad > 0) {
        return 'pad';
      }
      if ((p - g.rightAt).distance < 9 && g.hs != 0 || (p - g.rightAt).distance < 9) {
        return 'right';
      }
      if ((p - g.bottomAt).distance < 9) {
        return 'bottom';
      }
      return null;
    },
    start: (grab, p, fine) {
      // pulling an edge turns Hug / Fill into Fixed at the size it already has (nothing jumps)
      if (grab == 'right' && d['lay.hs'] != 2) {
        d.set('lay.w', g.parent.width / Gui.layK);
        d.set('lay.hs', 2);
      }
      if (grab == 'bottom' && d['lay.vs'] != 2) {
        d.set('lay.h', g.parent.height / Gui.layK);
        d.set('lay.vs', 2);
      }
    },
    drag: (grab, p, last, fine) {
      final k = fine ? Gui.fine : 1.0, dd = (p - last) * k;
      switch (grab) {
        case 'cols':
          d.set('lay.cols', (d['lay.cols'] + dd.dx / (Gui.layCw + g.gap)).clamp(1.0, Gui.layN + .49));
        case 'gap':
          d.set('lay.gap', (d['lay.gap'] + (g.cols > 1 ? dd.dx : dd.dy) / Gui.layK).clamp(0.0, 80.0));
        case 'pad':
          d.set('lay.pad', (d['lay.pad'] + (dd.dx + dd.dy) / 2 / Gui.layK).clamp(0.0, 80.0));
        case 'right':
          d.set('lay.w', (d['lay.w'] + dd.dx / Gui.layK).clamp(60.0, 800.0));
        case 'bottom':
          d.set('lay.h', (d['lay.h'] + dd.dy / Gui.layK).clamp(40.0, 600.0));
      }
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final role = [Fam.along.c, Fam.stagger.c, Fam.scatter.c], a = g.on ? 1.0 : .5;
      cv.drawRect(g.avail, _ln(N.g15)); // the space on offer
      final pr = g.parent;
      cv.drawRect(pr, _ln((d.locked ? Role.disabled : N.g76).withValues(alpha: a)));
      if (g.pad > 0) {
        cv.drawRect(g.inner, _ln(role[1].withValues(alpha: .35)));
      }
      for (final b in g.boxes) {
        cv.drawRect(b, _fl(N.g20.withValues(alpha: a)));
        cv.drawRect(b, _ln(N.g44.withValues(alpha: a)));
      }
      void dot(Offset o, Color c, String key, {bool square = false}) {
        final on = hot == key, col = d.locked ? Role.disabled : c;
        if (square) {
          final r = Rect.fromCenter(center: o, width: Gui.handle + 2, height: Gui.handle + 2);
          cv.drawRect(r, _fl(N.g07));
          cv.drawRect(r, _ln(on ? N.g95 : col));
        } else {
          cv.drawCircle(o, on ? 4 : Gui.dot, _fl(on ? N.g95 : col));
        }
      }

      cv.drawLine(g.colAt + const Offset(0, -7), g.colAt + const Offset(0, 7), _ln(d.locked ? Role.disabled : role[0]));
      cv.drawLine(g.colAt + const Offset(-2, 0), g.colAt + const Offset(2, 0), _ln(d.locked ? Role.disabled : role[0]));
      if (g.on) {
        dot(g.gapAt, role[1], 'gap');
        if (g.pad > 0) {
          dot(g.padAt, role[1], 'pad', square: true);
        }
        cv.drawLine(g.rightAt + const Offset(0, -7), g.rightAt + const Offset(0, 7), _ln(hot == 'right' ? N.g95 : (d.locked ? Role.disabled : role[2])));
        cv.drawLine(g.bottomAt + const Offset(-7, 0), g.bottomAt + const Offset(7, 0), _ln(hot == 'bottom' ? N.g95 : (d.locked ? Role.disabled : role[2])));
      }
    },
  );
}

class _SizingGlyph extends StatelessWidget {
  const _SizingGlyph(this.i, this.col);
  final int i;
  final Color col;
  @override
  Widget build(BuildContext context) => CustomPaint(
        size: const Size(16, 10),
        painter: _Fn((c, s) {
          final w = s.width, cy = s.height / 2, p = _ln(col), f = _fl(col);
          switch (i) {
            case 0: // Hug: the walls close in
              c.drawLine(const Offset(.5, 0), Offset(.5, s.height), p);
              c.drawLine(Offset(w - .5, 0), Offset(w - .5, s.height), p);
              c.drawPath(Path()..moveTo(2.5, cy - 2.4)..lineTo(5.5, cy)..lineTo(2.5, cy + 2.4)..close(), f);
              c.drawPath(Path()..moveTo(w - 2.5, cy - 2.4)..lineTo(w - 5.5, cy)..lineTo(w - 2.5, cy + 2.4)..close(), f);
            case 1: // Fill: the walls reach out
              c.drawLine(Offset(w * .3, cy), Offset(w * .7, cy), p);
              c.drawPath(Path()..moveTo(w * .3, cy - 2.6)..lineTo(0, cy)..lineTo(w * .3, cy + 2.6)..close(), f);
              c.drawPath(Path()..moveTo(w * .7, cy - 2.6)..lineTo(w, cy)..lineTo(w * .7, cy + 2.6)..close(), f);
            default: // Fixed: a set size
              c.drawRect(Rect.fromLTWH(w * .2, 1.5, w * .6, s.height - 3), p);
          }
        }),
      );
}

const _sizingNames = ['Hug', 'Fill', 'Fixed'];

Widget _sizeLine(GDoc d, String axis) {
  final id = axis == 'W' ? 'lay.hs' : 'lay.vs', on = d['lay.on'] > .5 && !d.locked, cur = d[id].round(), fixed = cur == 2;
  return Row(children: [
    SizedBox(width: 14, child: Text(axis, style: T.label(N.g63))),
    for (var i = 0; i < 3; i++)
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: on
            ? () {
                d.begin();
                d.set(id, i.toDouble());
                d.end();
              }
            : null,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          width: 28,
          height: Gui.field,
          margin: const EdgeInsets.only(right: 2),
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: cur == i ? N.g20 : N.g07,
            borderRadius: BorderRadius.circular(5),
            border: Border(bottom: BorderSide(color: cur == i && on ? Role.selected : const Color(0x00000000))),
          ),
          child: _SizingGlyph(i, !on ? N.g44 : (cur == i ? N.g95 : N.g63)),
        ),
      ),
    const SizedBox(width: 4),
    Expanded(child: XyzStack(d, [Spec(axis == 'W' ? 'lay.w' : 'lay.h', '', unit: 'px', dec: 0, per: 2, on: on && fixed, word: fixed || !d.locked && !on ? null : _sizingNames[cur])])),
  ]);
}

Widget _layNumbers(GDoc d) {
  final on = d['lay.on'] > .5;
  Widget f(String t, Spec s) => Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [Text(t, style: T.label(N.g63)), _gap(4), XyzStack(d, [s])]));
  return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
    Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      f('Columns', Spec('lay.cols', 'C', color: Fam.along.c, dec: 0, per: .05)),
      const SizedBox(width: 6),
      f('Gap', Spec('lay.gap', 'G', color: Fam.stagger.c, unit: 'px', dec: 0, on: on)),
      const SizedBox(width: 6),
      f('Padding', Spec('lay.pad', 'P', color: Fam.stagger.c, unit: 'px', dec: 0, on: on)),
    ]),
    _gap(8),
    _sizeLine(d, 'W'),
    _gap(2),
    _sizeLine(d, 'H'),
  ]);
}

class LayoutGui extends StatelessWidget {
  const LayoutGui(this.d, {super.key, this.mode = Mode.gui});
  final GDoc d;
  final Mode mode;
  @override
  Widget build(BuildContext context) {
    final diagram = _layoutHand(d, const Size(Gui.content, Gui.layoutH));
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _title('Layout', trailing: Row(mainAxisSize: MainAxisSize.min, children: [
        Text('Grid', style: T.label(d['lay.on'] > .5 ? N.g95 : N.g56)),
        const SizedBox(width: 5),
        PillSwitch(on: d['lay.on'] > .5, onChanged: d.locked ? null : (v) {
          d.begin();
          d.set('lay.on', v ? 1 : 0); // Grid gates the rest; nothing it gates is lost
          d.end();
        }),
      ])),
      _gap(6),
      if (mode == Mode.fields) _layNumbers(d) else if (mode == Mode.gui) ...[diagram, _gap(6), _layNumbers(d)] else ...[_layNumbers(d), _gap(8), diagram],
    ]);
  }
}

// ---- hierarchy options: the SAME Transform block in three arrangements (research/precedents-gui.md) ---------------------------------

Widget _axisChips(GDoc d) => Row(children: [
      for (var i = 0; i < 3; i++) Chip(const ['Z', 'X', 'Y'][i], on: d['rot.axis'].round() == i, enabled: !d.locked, onTap: () => d.set('rot.axis', i.toDouble())),
    ]);

Spec _rotSpec(GDoc d) {
  final i = d['rot.axis'].round().clamp(0, 2);
  return Spec(const ['rot.z', 'rot.x', 'rot.y'][i], const ['Z', 'X', 'Y'][i], axis: const [2, 0, 1][i], unit: '°', per: .5);
}

/// A graphic with its numbers as a quiet readout (one line per axis). Hover, focus or a drag on the cell turns the readout into the real fields.
class _Cell extends StatefulWidget {
  const _Cell(this.d, this.hand, this.specs, {this.even = false, this.horizontal = false});
  final GDoc d;
  final Widget hand;
  final List<Spec> specs;
  final bool even; // the readout takes the balanced step whatever the knob says (option 3: equal weight)
  final bool horizontal; // the readouts sit side by side (the wide pad)
  @override
  State<_Cell> createState() => _CellState();
}

class _CellState extends State<_Cell> {
  bool hover = false;
  @override
  Widget build(BuildContext context) {
    final d = widget.d, w = widget.even ? 1 : d.weight, shown = (hover || d.live) && !d.locked;
    Widget line(Spec s) => SizedBox(
          height: Gui.field,
          child: Row(children: [
            SizedBox(width: 14, child: Text(s.tag, maxLines: 1, style: T.label(N.g56))),
            Expanded(
              child: FittedBox(
                fit: BoxFit.scaleDown,
                alignment: Alignment.centerLeft,
                child: Text(d.mixed.contains(s.id) ? '—' : '${_fmt(d[s.id] * s.scale, s.dec)}${s.unit.isEmpty ? '' : ' ${s.unit}'}', maxLines: 1, style: T.value(Gui.readoutInk[w]).copyWith(fontSize: Gui.readoutSize[w])),
              ),
            ),
          ]),
        );
    final fields = widget.horizontal
        ? Row(children: [for (final (i, s) in widget.specs.indexed) ...[if (i > 0) const SizedBox(width: Gui.rowGap), Expanded(child: shown ? XyzStack(d, [s], gap: Gui.rowGap) : line(s))]])
        : (shown
            ? XyzStack(d, widget.specs, gap: Gui.rowGap)
            : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [for (final (i, s) in widget.specs.indexed) ...[if (i > 0) const SizedBox(height: Gui.rowGap), line(s)]]));
    return MouseRegion(
      onEnter: (_) => setState(() => hover = true),
      onExit: (_) => setState(() => hover = false),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [widget.hand, const SizedBox(height: Gui.rowGap), fields]),
    );
  }
}

Widget _lab(String t, {Widget? trailing}) => SizedBox(height: 20, child: Row(children: [Text(t, style: T.label(N.g63)), const Spacer(), ?trailing]));
Widget _linkChip(GDoc d) => Chip('Link', on: d['link'] > .5, enabled: !d.locked, onTap: () => d.set('link', d['link'] > .5 ? 0 : 1));

/// Option 1. Instruments primary: the pad is the largest thing, dial / scale / anchor share one cell size, numbers are a readout.
/// From precedents-gui.md: Pattern 1C lines 26-30 (Ableton: the value appears with the knob), candidates P1-c line 75 and P2-c line 81 (one larger pad over a bank row of equal small instruments).
class TransformInstrumentsFirst extends StatelessWidget {
  const TransformInstrumentsFirst(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    final three = d['space'] > 0;
    const w = Gui.content, colW = (w - 16) / 3;
    Widget cell(String t, Widget hand, List<Spec> specs, {Widget? trailing, Widget? under}) => SizedBox(
          width: colW,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_lab(t, trailing: trailing), _Cell(d, hand, specs), if (under != null) ...[const SizedBox(height: Gui.rowGap), under]]),
        );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      _lab('Position'),
      _Cell(
        d,
        Row(children: [_padHand(d, Size(three ? w - Gui.rail - 6 : w, Gui.padH + 16)), if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.padH + 16))]]),
        _posSpecs(d),
        horizontal: true,
      ),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        cell('Rotation', _dialHand(d, Size(colW, colW)), [_rotSpec(d)], under: three ? _axisChips(d) : null),
        const SizedBox(width: 8),
        cell('Scale', _scaleHand(d, Size(colW, colW)), _scaleSpecs, trailing: _linkChip(d)),
        const SizedBox(width: 8),
        cell('Anchor', _anchorHand(d, Size(colW, colW)), _anchorSpecs),
      ]),
    ]);
  }
}

/// Option 2. Figma-like: the fields are the primary at full weight, one small helper of one size sits beside each.
/// From precedents-gui.md: Pattern 1B lines 20-24 (Figma: numbers always visible, the graphic is a smaller key beside them), candidate P2-d line 82 (instruments as keys of identical size in the row grid).
class TransformFieldsFirst extends StatelessWidget {
  const TransformFieldsFirst(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    const key = 72.0;
    Widget row(String t, List<Spec> specs, Widget helper, {Widget? trailing, Widget? under}) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _title(t, trailing: trailing),
          _gap(6),
          Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [XyzStack(d, specs, gap: Gui.rowGap), if (under != null) ...[const SizedBox(height: Gui.rowGap), under]])),
            const SizedBox(width: 8),
            SizedBox(width: key, height: key, child: helper),
          ]),
          _gap(12),
        ]);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      row('Position', _posSpecs(d), _padHand(d, const Size(key, key))),
      row('Rotation', [_rotSpec(d)], _dialHand(d, const Size(key, key)), under: d['space'] > 0 ? _axisChips(d) : null),
      row('Scale', _scaleSpecs, _scaleHand(d, const Size(key, key)), trailing: _linkChip(d)),
      row('Anchor', _anchorSpecs, _anchorHand(d, const Size(key, key))),
    ]);
  }
}

/// Option 3. One shared cell grid: four instruments of the same size in 2 x 2, an equal-weight readout under each.
/// From precedents-gui.md: Pattern 2C lines 47-48 (Ableton: a uniform bank, one diameter, one label slot), P2 finding line 52, candidate P2-a line 79 (every instrument a square cell of one side, a shared top edge).
class TransformGrid extends StatelessWidget {
  const TransformGrid(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    const side = (Gui.content - 8) / 2;
    Widget cell(String t, Widget hand, List<Spec> specs, {Widget? trailing, Widget? under}) => SizedBox(
          width: side,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [_lab(t, trailing: trailing), _Cell(d, hand, specs, even: true), if (under != null) ...[const SizedBox(height: Gui.rowGap), under]]),
        );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        cell('Position', _padHand(d, const Size(side, side)), _posSpecs(d)),
        const SizedBox(width: 8),
        cell('Rotation', _dialHand(d, const Size(side, side)), [_rotSpec(d)], under: d['space'] > 0 ? _axisChips(d) : null),
      ]),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        cell('Scale', _scaleHand(d, const Size(side, side)), _scaleSpecs, trailing: _linkChip(d)),
        const SizedBox(width: 8),
        cell('Anchor', _anchorHand(d, const Size(side, side)), _anchorSpecs),
      ]),
    ]);
  }
}

// ---- UX options: four interaction models for the SAME Transform block --------------------------------------------------------------

enum GMode { all, move, scale, rotate, anchor }

String _zoneName(Object? hot) {
  if (hot == null) {
    return '';
  }
  if (hot == 'move') {
    return 'Move: drag the body';
  }
  if (hot == 'rot') {
    return 'Rotate: drag in the ring';
  }
  if (hot == 'pivot' || (hot is String && hot.startsWith('a'))) {
    return 'Anchor: the pivot';
  }
  if (hot is int) {
    return hot < 4 ? 'Scale: corner' : 'Scale: edge';
  }
  return '';
}

/// One canvas that shows the layer's box. [GMode.all] = U1 (five zones at once); the other modes = U2 (only that mode's handles are live and drawn).
Widget _gizmoHand(GDoc d, Size size, GMode mode, {ValueChanged<Object?>? onHot}) {
  final g = _PadGeo(size, d, boost: Gui.boost);
  final all = mode == GMode.all;
  Offset local(Offset p) => _rot(p - g.puck, -g.rad);
  bool inBody(Offset p) {
    final l = local(p);
    return l.dx.abs() <= g.half.dx && l.dy.abs() <= g.half.dy;
  }

  List<Offset> edges() {
    final h = g.half;
    return [for (final o in [Offset(-h.dx, 0), Offset(h.dx, 0), Offset(0, -h.dy), Offset(0, h.dy)]) g.puck + _rot(o, g.rad)];
  }

  Offset nine(int i) => g.puck + _rot(Offset(((i % 3) - 1) * g.half.dx, ((i ~/ 3) - 1) * g.half.dy), g.rad);
  Offset pivot() => g.puck + _rot(Offset((d['anchor.x'] * 2 - 1) * g.half.dx, (d['anchor.y'] * 2 - 1) * g.half.dy), g.rad);
  final ringR = g.half.distance + 20;
  final rotId = _rotSpec(d).id;
  return _Hand(
    d: d,
    size: size,
    onHot: onHot,
    pick: (p) {
      if (mode == GMode.anchor) {
        for (var i = 0; i < 9; i++) {
          if ((p - nine(i)).distance < 11) {
            return 'a$i';
          }
        }
        return null;
      }
      if (all && (p - pivot()).distance < 8) {
        return 'pivot';
      }
      if (all || mode == GMode.scale) {
        final cs = g.corners(g.puck, g.half), es = edges();
        for (var i = 0; i < 4; i++) {
          if ((p - cs[i]).distance < 10) {
            return i;
          }
        }
        for (var i = 0; i < 4; i++) {
          if ((p - es[i]).distance < 9) {
            return 4 + i;
          }
        }
      }
      if (all && !inBody(p)) {
        for (final c in g.corners(g.puck, g.half)) {
          final dist = (p - c).distance;
          if (dist >= 10 && dist <= Gui.ringBand) {
            return 'rot'; // the ring just outside a corner
          }
        }
      }
      if (mode == GMode.rotate && ((p - g.puck).distance - ringR).abs() < 12) {
        return 'rot';
      }
      if ((all && inBody(p)) || mode == GMode.move) {
        return 'move';
      }
      return null;
    },
    start: (grab, p, fine) {
      if (mode == GMode.move && grab == 'move' && !fine && !inBody(p)) {
        d.set('pos.x', (p.dx - g.c.dx) * g.k);
        d.set('pos.y', (p.dy - g.c.dy) * g.k);
      }
    },
    tap: (grab, p) {
      if (grab is String && grab.startsWith('a')) {
        final i = int.parse(grab.substring(1));
        d.set('anchor.x', (i % 3) / 2);
        d.set('anchor.y', (i ~/ 3) / 2);
      }
    },
    drag: (grab, p, last, fine) {
      final k = fine ? Gui.fine : 1.0;
      if (grab == 'move') {
        final dd = (p - last) * g.k * k;
        d.add('pos.x', dd.dx);
        d.add('pos.y', dd.dy);
      } else if (grab == 'pivot') {
        final dl = local(p) - local(last);
        d.set('anchor.x', (d['anchor.x'] + dl.dx / (2 * g.half.dx) * k).clamp(0.0, 1.0));
        d.set('anchor.y', (d['anchor.y'] + dl.dy / (2 * g.half.dy) * k).clamp(0.0, 1.0));
      } else if (grab == 'rot') {
        d.add(rotId, _wrap((p - g.puck).direction - (last - g.puck).direction) * 180 / math.pi * k);
      } else if (grab is int) {
        final lp = local(p), ll = local(last);
        double ratio(double a, double b) => b.abs() < 2 ? 1.0 : (a / b).abs();
        var fx = grab < 4 || grab < 6 ? ratio(lp.dx, ll.dx) : 1.0, fy = grab < 4 || grab >= 6 ? ratio(lp.dy, ll.dy) : 1.0;
        if (d['link'] > .5) {
          fx = fy = grab < 4 ? lp.distance / math.max(2, ll.distance) : (grab < 6 ? fx : fy);
        }
        d.set('scale.x', d['scale.x'] * math.pow(fx, k));
        d.set('scale.y', d['scale.y'] * math.pow(fy, k));
      }
    },
    paint: (cv, sz, hot, live) {
      _well(cv, sz);
      final c = g.c;
      for (final f in const [1 / 3, 2 / 3]) {
        cv.drawLine(Offset(sz.width * f, 0), Offset(sz.width * f, sz.height), _ln(N.g15));
        cv.drawLine(Offset(0, sz.height * f), Offset(sz.width, sz.height * f), _ln(N.g15));
      }
      cv.drawLine(Offset(c.dx, 0), Offset(c.dx, sz.height), _ln(N.g20));
      cv.drawLine(Offset(0, c.dy), Offset(sz.width, c.dy), _ln(N.g20));
      cv.drawRect(Rect.fromCenter(center: c, width: Gui.comp.width / g.k, height: Gui.comp.height / g.k), _ln(N.g26));
      final pk = g.puck, cs = g.corners(pk, g.half), col = _stroke(d, hot, live), body = Path()..addPolygon(cs, true);
      cv.drawPath(body, _fl(d.locked ? N.g13 : (hot == 'move' || live && hot == 'move' ? N.g20 : N.g15)));
      cv.drawPath(body, _ln(col));
      final top = (cs[0] + cs[1]) / 2;
      cv.drawLine(top, top + _rot(const Offset(0, -1), g.rad) * 5, _ln(col));
      void sq(Offset o, double r, Object key) {
        final on = hot == key, rr = Rect.fromCenter(center: o, width: r, height: r);
        cv.drawRect(rr, _fl(on ? N.g95 : N.g07));
        cv.drawRect(rr, _ln(d.locked ? Role.disabled : (on ? C.mode : N.g76)));
      }

      if (all || mode == GMode.scale) {
        final es = edges();
        for (var i = 0; i < 4; i++) {
          sq(cs[i], 7, i);
          sq(es[i], 5, 4 + i);
        }
      }
      if (all && hot == 'rot') {
        for (final cn in cs) {
          cv.drawCircle(cn, 18, _ln(Role.selected.withValues(alpha: .5))); // the ring zone shows on hover only
        }
      }
      if (mode == GMode.rotate) {
        cv.drawCircle(pk, ringR, _ln(d.locked ? N.g38 : N.g38));
        final a = -math.pi / 2 + d[rotId] * math.pi / 180, tip = pk + Offset(math.cos(a), math.sin(a)) * ringR;
        cv.drawLine(pk, tip, _ln(N.g44));
        cv.drawCircle(tip, 4, _fl(N.g95));
        cv.drawCircle(tip, 4, _ln(col));
      }
      if (mode == GMode.anchor) {
        final sel = _anchorIndex(d);
        for (var i = 0; i < 9; i++) {
          final on = sel == i, hv = hot == 'a$i';
          cv.drawCircle(nine(i), on || hv ? 4 : Gui.dot, on ? _fl(d.locked ? Role.disabled : N.g95) : _ln(hv ? N.g95 : N.g56));
          if (on && !d.locked) {
            cv.drawCircle(nine(i), 6, _ln(C.mode));
          }
        }
        if (sel == null) {
          cv.drawCircle(pivot(), 4, _ln(N.g95));
        }
      }
      if (all) {
        final pv = pivot();
        cv.drawCircle(pv, 3, _fl(d.locked ? Role.disabled : N.g95));
        cv.drawCircle(pv, 5, _ln(hot == 'pivot' ? C.mode : N.g44));
      }
    },
  );
}

String _vals(GDoc d, List<Spec> specs) => [
      for (final s in specs) '${s.tag} ${d.mixed.contains(s.id) ? '—' : _fmt(d[s.id] * s.scale, s.dec)}${s.unit.isEmpty ? '' : ' ${s.unit}'}',
    ].join('   ');

/// U1. Unified object gizmo: move, scale, rotate and anchor on one canvas; the readout names the zone under the pointer.
class TransformUnified extends StatefulWidget {
  const TransformUnified(this.d, {super.key});
  final GDoc d;
  @override
  State<TransformUnified> createState() => _TransformUnifiedState();
}

class _TransformUnifiedState extends State<TransformUnified> {
  Object? hot;
  @override
  Widget build(BuildContext context) {
    final d = widget.d, three = d['space'] > 0;
    final w = Gui.content - (three ? Gui.rail + 6 : 0);
    final zone = _zoneName(hot);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        _gizmoHand(d, Size(w, Gui.gizmoH), GMode.all, onHot: (h) {
          if (mounted && h != hot) {
            setState(() => hot = h);
          }
        }),
        if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.gizmoH))],
      ]),
      _gap(6),
      SizedBox(
        height: Gui.field,
        child: Row(children: [
          Expanded(
            child: FittedBox(
              fit: BoxFit.scaleDown,
              alignment: Alignment.centerLeft,
              child: Text(zone.isNotEmpty ? zone : _vals(d, [..._posSpecs(d).take(2), _rotSpec(d), ..._scaleSpecs]), maxLines: 1, style: T.value(N.g76)),
            ),
          ),
          _linkChip(d),
        ]),
      ),
    ]);
  }
}

/// U2. Mode-switch single canvas: Move / Scale / Rotate / Anchor (keys 1-4); the canvas and the numbers belong to the active mode only.
class TransformModes extends StatefulWidget {
  const TransformModes(this.d, {super.key});
  final GDoc d;
  @override
  State<TransformModes> createState() => _TransformModesState();
}

class _TransformModesState extends State<TransformModes> {
  GMode mode = GMode.move;
  final focus = FocusNode(debugLabel: 'modes');
  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  void _set(int i) => setState(() => mode = const [GMode.move, GMode.scale, GMode.rotate, GMode.anchor][i]);

  @override
  Widget build(BuildContext context) {
    final d = widget.d, i = mode.index - 1;
    final specs = switch (mode) {
      GMode.move => _posSpecs(d),
      GMode.scale => _scaleSpecs,
      GMode.rotate => [_rotSpec(d)],
      _ => _anchorSpecs,
    };
    return Focus(
      focusNode: focus,
      onKeyEvent: (_, e) {
        final k = {LogicalKeyboardKey.digit1: 0, LogicalKeyboardKey.digit2: 1, LogicalKeyboardKey.digit3: 2, LogicalKeyboardKey.digit4: 3}[e.logicalKey];
        if (e is KeyDownEvent && k != null) {
          _set(k);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _SpaceChips(d),
        _gap(12),
        Segmented(items: const ['Move', 'Scale', 'Rotate', 'Anchor'], index: i, expand: true, onChanged: (n) {
          focus.requestFocus();
          _set(n);
        }),
        _gap(8),
        _gizmoHand(d, const Size(Gui.content, Gui.gizmoH), mode),
        _gap(8),
        XyzStack(d, specs, gap: Gui.rowGap),
        if (mode == GMode.scale) ...[_gap(4), Align(alignment: Alignment.centerLeft, child: _linkChip(d))],
        if (mode == GMode.rotate && d['space'] > 0) ...[_gap(4), _axisChips(d)],
      ]),
    );
  }
}

/// U3. Scrub-first: the Blender-style stacks are the only controls; a small preview of the resulting box has no handles.
class TransformScrub extends StatelessWidget {
  const TransformScrub(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _title('Preview'),
        _gap(6),
        IgnorePointer(
          child: SizedBox(
            height: Gui.previewH,
            child: CustomPaint(painter: _Fn((cv, sz) {
              final g = _PadGeo(sz, d);
              _well(cv, sz);
              cv.drawRect(Rect.fromCenter(center: g.c, width: Gui.comp.width / g.k, height: Gui.comp.height / g.k), _ln(N.g26));
              final pk = g.puck, cs = g.corners(pk, g.half);
              cv.drawPath(Path()..addPolygon(cs, true), _fl(N.g15));
              cv.drawPath(Path()..addPolygon(cs, true), _ln(N.g63));
              cv.drawCircle(pk + _rot(Offset((d['anchor.x'] * 2 - 1) * g.half.dx, (d['anchor.y'] * 2 - 1) * g.half.dy), g.rad), 2.5, _fl(N.g95));
            })),
          ),
        ),
        _gap(12),
        TransformFields(d),
      ]);
}

/// U4. Summon on focus: numbers only at rest; hovering or focusing a row opens its instrument inline; Esc or leaving closes it.
class _SummonRow extends StatefulWidget {
  const _SummonRow(this.d, this.title, this.specs, this.instrument);
  final GDoc d;
  final String title;
  final List<Spec> specs;
  final Widget instrument;
  @override
  State<_SummonRow> createState() => _SummonRowState();
}

class _SummonRowState extends State<_SummonRow> {
  bool hover = false, focused = false, dismissed = false;
  @override
  Widget build(BuildContext context) {
    final d = widget.d, open = (hover || focused || d.live) && !dismissed && !d.locked;
    return Focus(
      onFocusChange: (f) => setState(() {
        focused = f;
        if (!f) {
          dismissed = false;
        }
      }),
      onKeyEvent: (_, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && open) {
          setState(() => dismissed = true); // Esc closes the summoned instrument
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: MouseRegion(
        onEnter: (_) => setState(() => hover = true),
        onExit: (_) => setState(() {
          hover = false;
          dismissed = false;
        }),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(
            height: Gui.field,
            child: Row(children: [
              SizedBox(width: 54, child: Text(widget.title, style: T.label(open ? N.g95 : N.g63))),
              for (final (i, s) in widget.specs.indexed) ...[if (i > 0) const SizedBox(width: Gui.rowGap), Expanded(child: XyzStack(d, [s]))],
            ]),
          ),
          AnimatedSize(
            duration: const Duration(milliseconds: 120),
            curve: Curves.easeOut,
            alignment: Alignment.topCenter,
            child: open ? Padding(padding: const EdgeInsets.only(top: 6, bottom: 2), child: widget.instrument) : const SizedBox(width: double.infinity),
          ),
        ]),
      ),
    );
  }
}

class TransformSummon extends StatelessWidget {
  const TransformSummon(this.d, {super.key});
  final GDoc d;
  @override
  Widget build(BuildContext context) {
    final three = d['space'] > 0;
    final padW = Gui.content - (three ? Gui.rail + 6 : 0);
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _SpaceChips(d),
      _gap(12),
      _SummonRow(d, 'Position', _posSpecs(d), Row(children: [_padHand(d, Size(padW, Gui.padH)), if (three) ...[const SizedBox(width: 6), _railHand(d, const Size(Gui.rail, Gui.padH))]])),
      _gap(4),
      _SummonRow(d, 'Rotation', [_rotSpec(d)], Row(children: [_dialHand(d, const Size(Gui.cell, Gui.cell)), const SizedBox(width: 8), if (three) _axisChips(d)])),
      _gap(4),
      _SummonRow(d, 'Scale', _scaleSpecs, Row(children: [_scaleHand(d, const Size(Gui.cell, Gui.cell)), const SizedBox(width: 8), _linkChip(d)])),
      _gap(4),
      _SummonRow(d, 'Anchor', _anchorSpecs, Align(alignment: Alignment.centerLeft, child: _anchorHand(d, Gui.anchorBox))),
    ]);
  }
}

// ---- hosting: the document, the knobs, the inspector column ---------------------------------------------------------------------

class _Host extends StatefulWidget {
  const _Host({required this.space, required this.locked, required this.mixed, required this.tone, required this.build, this.layer = false, this.weight = 1});
  final int space;
  final bool locked, mixed, layer;
  final int weight;
  final AxisTone tone;
  final Widget Function(BuildContext, GDoc) build;
  @override
  State<_Host> createState() => _HostState();
}

class _HostState extends State<_Host> {
  final doc = GDoc();

  void _apply({bool spaceToo = true}) {
    if (spaceToo) {
      doc.v['space'] = widget.space.toDouble();
    }
    doc
      ..locked = widget.locked
      ..tone = widget.tone
      ..weight = widget.weight
      ..v['cam.layer'] = widget.layer ? 1 : 0;
    doc.mixed
      ..clear()
      ..addAll(widget.mixed ? const ['pos.x', 'scale.x', 'rot.z'] : const []);
  }

  @override
  void initState() {
    super.initState();
    _apply();
  }

  @override
  void didUpdateWidget(_Host old) {
    super.didUpdateWidget(old);
    _apply(spaceToo: old.space != widget.space);
  }

  @override
  void dispose() {
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(listenable: doc, builder: (context, _) => widget.build(context, doc));
}

Widget _hosted(BuildContext c, Widget Function(BuildContext, GDoc) build, {bool layerKnob = false, bool weightKnob = false}) => _Host(
      weight: weightKnob ? c.knobs.object.dropdown<int>(label: 'Number weight (graphic leads / balanced / numbers lead)', options: const [0, 1, 2], initialOption: 0, labelBuilder: (i) => const ['0  graphic leads', '1  balanced', '2  numbers lead'][i]) : 1,
      space: c.knobs.object.dropdown<int>(label: 'Space', options: const [0, 1, 2], initialOption: 1, labelBuilder: (i) => const ['2D', '2.5D', '3D'][i]),
      locked: c.knobs.boolean(label: 'Locked layer'),
      mixed: c.knobs.boolean(label: 'Mixed (3 layers selected)'),
      tone: c.knobs.object.dropdown<AxisTone>(label: 'Axis colour (decision B)', options: AxisTone.values, initialOption: AxisTone.fam, labelBuilder: (t) => switch (t) {
            AxisTone.fam => '1  Fam hues (now)',
            AxisTone.rgb => '2  X red / Y green / Z blue',
            AxisTone.none => '3  none, letters only',
          }),
      layer: layerKnob && c.knobs.boolean(label: 'Target is a layer'),
      build: (ctx, d) => SingleChildScrollView(child: build(ctx, d)), // the whole story scrolls vertically: nothing can overflow the viewport
    );

Widget _col(Widget child, {String? head}) => Container(
      width: Gui.content + 26, // 12 + 256 + 12 gutters, plus the 1 px hairline each side
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        if (head != null) ...[Text(head, style: T.title()), _gap(12)],
        child,
      ]),
    );

/// The long explanation lives in the Widgetbook story chrome, beside the 282 px frame, never inside it.
Widget _help(GDoc d, String text) => SizedBox(
      width: 320,
      child: Text('Undo steps: ${d.steps} (one per gesture; Esc leaves none) · last scrub applied to: ${d.scope}\n$_rules\n\n$text', style: T.label(N.g63).copyWith(height: 1.4)),
    );

Widget _story(Widget frame, GDoc d, String text) => Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [frame, const SizedBox(width: 20), _help(d, text)]);

const _rules = 'Shift = fine · Esc during a drag = revert · locked layer = inert';

WidgetbookComponent inspectorGuiSet() => WidgetbookComponent(name: 'inspector-gui', useCases: [
      uc('Transform gui', (c) => _hosted(c, (ctx, d) => _story(_col(TransformGui(d)), d,
          'Pad: press anywhere and the body jumps there (Shift: no jump). Dial: turn from the ring or inside it; 450 stays 450 and shows 1×. Scale: corners scale both (one factor while Link is on), edges one axis. Anchor: one click; a typed value off the nine reads Custom. 2D / 2.5D / 3D reveals Z, the other rotation axes and the Z rail without changing a value. Mixed: X, scale X and rotation read — and the pad moves every ghost by the same amount.')), width: 640),
      uc('Camera face', (c) => _hosted(c, layerKnob: true, (ctx, d) => _story(_col(CameraGui(d)), d,
          'Target (dot) · Orbit (camera, pitch and yaw around the target; hollow = on the far side) · Framing (bar on the ray, outward = farther, exponential) · Roll (ring, turns kept). Turn on "Target is a layer": the point fields disable and say which layer.')), width: 640),
      uc('Layout mini-diagram', (c) => _hosted(c, (ctx, d) => _story(_col(LayoutGui(d)), d,
          'Columns bar (right of row one) · Gap dot · Padding corner · pull the right or bottom edge: Hug / Fill becomes Fixed at the size it already has. Hug · Fill · Fixed are three drawings, not words. Grid off dims the diagram and disables everything but Columns; only Fixed enables its size number.')), width: 640),
      uc('Assembled: (a) fields, (b) instruments, (c) both', (c) => _hosted(c, layerKnob: true, (ctx, d) {
            Widget scroll(String head, Widget Function() build) => SizedBox(height: 700, child: SingleChildScrollView(child: _col(build(), head: head)));
            Widget stack(List<Widget> kids) => Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [for (final (i, k) in kids.indexed) ...[if (i > 0) _gap(16), k]]);
            return SingleChildScrollView(scrollDirection: Axis.horizontal, child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                scroll('(a) Fields only', () => stack([TransformFields(d), CameraGui(d, mode: Mode.fields), LayoutGui(d, mode: Mode.fields)])),
                const SizedBox(width: 16),
                scroll('(b) Pad, dial, handles', () => stack([TransformGui(d), CameraGui(d), LayoutGui(d)])),
                const SizedBox(width: 16),
                scroll('(c) Both', () => stack([TransformBoth(d), CameraGui(d, mode: Mode.both), LayoutGui(d, mode: Mode.both)])),
              ]),
              const SizedBox(height: 12),
              _help(d, 'Fields: drag a field sideways to scrub, press and drag across the stack to edit several at once, the small tab at the right locks an axis, Alt = apply to all selected, a click types. All three columns edit ONE document: change a value in any of them and the other two follow.'),
            ]));
          }), width: 920),
      uc('Hierarchy options', (c) => _hosted(c, weightKnob: true, (ctx, d) {
            Widget option(String head, String note, Widget body) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                  SizedBox(width: Gui.content + 26, child: Text(head, style: T.title())),
                  const SizedBox(height: 6),
                  SizedBox(width: Gui.content + 26, child: Text(note, style: T.label(N.g63).copyWith(height: 1.4))),
                  const SizedBox(height: 10),
                  SizedBox(height: 640, child: SingleChildScrollView(child: _col(body))),
                ]);
            return SingleChildScrollView(scrollDirection: Axis.horizontal, child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                option('Option 1  Instruments first', 'precedents-gui.md: Pattern 1C (lines 26-30), P1-c (75), P2-c (81). The pad leads; dial, scale, anchor share one cell size. Numbers are a readout and become fields on hover or drag.', TransformInstrumentsFirst(d)),
                const SizedBox(width: 16),
                option('Option 2  Fields first', 'precedents-gui.md: Pattern 1B (lines 20-24), P2-d (82). Full-weight fields, one 72 px helper of one size beside each.', TransformFieldsFirst(d)),
                const SizedBox(width: 16),
                option('Option 3  One cell grid', 'precedents-gui.md: Pattern 2C (47-48), finding (52), P2-a (79). Four equal square cells, an equal-weight readout under each (hover = fields).', TransformGrid(d)),
              ]),
              const SizedBox(height: 12),
              _help(d, 'The same values in all three. Knob "Number weight" moves Gui.readoutInk / readoutSize / graphicInk: 0 graphic leads, 1 balanced, 2 numbers lead. The instrument strokes follow it in all three; the readouts follow it in option 1 only (option 3 keeps its readout at the balanced step on purpose, option 2 fields are always full weight).'),
            ]));
          }), width: 920),
      uc('UX options', (c) => _hosted(c, (ctx, d) {
            Widget option(String head, String contract, String facts, Widget body) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                  SizedBox(width: Gui.content + 26, child: Text(head, style: T.title())),
                  const SizedBox(height: 6),
                  SizedBox(width: Gui.content + 26, child: Text('Contract: $contract\n\n$facts', style: T.label(N.g63).copyWith(height: 1.4))),
                  const SizedBox(height: 10),
                  SizedBox(height: 560, child: SingleChildScrollView(child: _col(body))),
                ]);
            return SingleChildScrollView(scrollDirection: Axis.horizontal, child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                option(
                  'U1  Unified object gizmo',
                  'one canvas, five zones: body moves, corner or edge scales (Link keeps the ratio), the ring just outside a corner rotates, the centre dot moves the anchor; hovering names the zone.',
                  'In 1 gesture: move, scale, rotate or anchor, with no mode change.\nClicks before the first drag: X 0, rotation 0, scale 0, anchor 0. Typing a number: not possible here (readout only).\nHidden at rest: every number except one readout line; the rotate zone is drawn only while hovered.',
                  TransformUnified(d),
                ),
                const SizedBox(width: 16),
                option(
                  'U2  Mode switch, one canvas',
                  'a tool row Move / Scale / Rotate / Anchor (keys 1-4); the canvas draws and accepts only the active mode\'s handles, and only that mode\'s numbers are shown.',
                  'In 1 gesture: the active mode\'s change only.\nClicks (or one key) to change: X 1 (Move, if not active) + 1 drag; rotation 1 + drag; scale 1 + drag; anchor 1 + click on a point. Typing: 1 more click on a field.\nHidden at rest: the other three modes\' handles and numbers.',
                  TransformModes(d),
                ),
                const SizedBox(width: 16),
                option(
                  'U3  Scrub-first fields',
                  'Blender-style stacks are the only controls (scrub sideways, sweep vertically to edit several, lock per axis); a preview of the resulting box has no handles.',
                  'In 1 gesture: scrub one field, or sweep several and scrub them together.\nClicks to change: X 0 + 1 scrub; rotation 0 + 1 scrub; scale 0 + 1 scrub; anchor 0 + 1 scrub (or 1 click to type).\nHidden at rest: nothing but the lock tabs (shown on hover); there is nothing to grab on the preview.',
                  TransformScrub(d),
                ),
                const SizedBox(width: 16),
                option(
                  'U4  Summon on focus',
                  'numbers only at rest; hovering or focusing a row opens its instrument inline (Position pad, Rotation dial, Scale box, Anchor nine points); Esc or leaving the row closes it.',
                  'In 1 gesture: scrub a field, or hover a row and drag its instrument.\nClicks to change: X 0 (hover opens the pad) + 1 drag; rotation 0 + 1 drag; scale 0 + 1 drag; anchor 0 + 1 click on a point. Typing: 1 click on a field.\nHidden at rest: all four instruments.',
                  TransformSummon(d),
                ),
              ]),
              const SizedBox(height: 12),
              _help(d, 'All four edit the one document, so a change in any frame shows in the other three. Common to all: Esc during a drag reverts, Shift is fine, a locked layer is inert, a mixed value reads — with a neutral ring, one gesture is one undo step.'),
            ]));
          }), width: 920),
    ]);

// ---- public hooks for the workflow use case (sets/workflow_set.dart): the unified gizmo and its box, nothing else is exposed ----------

/// U1 on a given canvas size: the one canvas that moves, scales, rotates and re-anchors the layer in [d].
Widget unifiedGizmo(GDoc d, Size size, {ValueChanged<Object?>? onHot}) => _gizmoHand(d, size, GMode.all, onHot: onHot);

/// The four corners (clockwise from top left) of [d]'s box on a gizmo canvas of [size], so other panels can draw and hit-test the same box.
List<Offset> gizmoCorners(GDoc d, Size size) {
  final g = _PadGeo(size, d, boost: Gui.boost);
  return g.corners(g.puck, g.half);
}

/// The zone name the gizmo shows while hovered (move, scale, rotate, anchor).
String gizmoZoneName(Object? hot) => _zoneName(hot);

/// One scrub rule for the number fields. false = the lab's first rule (Shift = fine). true = options-inspector.md D-I3: Shift x10, Alt x0.1 (the R1 run).
abstract final class GuiRule {
  static bool shiftBig = false;
}
