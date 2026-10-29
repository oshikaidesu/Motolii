// Part of inspector_gui_set.dart (cut there by size, as it was).
part of 'inspector_gui_set.dart';

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
      final field = TextHand(field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
        controller: ctrl,
        focusNode: editFocus,
        style: T.value(N.g95),
        cursorColor: Role.selected,
        backgroundCursorColor: N.g20,
        textAlign: TextAlign.right,
        onSubmitted: (_) => _commitEdit(),
        onTapOutside: (_) => _commitEdit(),
      ));
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
