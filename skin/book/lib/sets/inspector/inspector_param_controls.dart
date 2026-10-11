part of 'inspector_parts.dart';

/// An effect's parameters as blocks that fill the panel's width: three across in a normal panel, more when it is widened,
/// so thirty parameters stay a short list. Order is the shader's; a block that needs room (a point, a size, a choice) takes two.
class ParamGrid extends StatelessWidget {
  const ParamGrid(this.fx, this.params, {super.key, this.enabled = true});
  final String fx;
  final List<Param> params;
  final bool enabled;

  static int span(Param p) => switch (p.look) {
    ParamLook.point || ParamLook.size || ParamLook.choice => 2,
    _ => 1,
  };

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      const g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <List<Param>>[[]];
      var used = 0;
      for (final p in params) {
        final s = math.min(span(p), cols);
        if (used + s > cols) {
          rows.add([]);
          used = 0;
        }
        rows.last.add(p);
        used += s;
      }
      // contract: a row cut short by a wide block stretches to the edge, so the grid never shows holes; only the last row may end early
      Widget row(List<Param> ps, bool last) {
        final filled = ps.fold<int>(0, (a, p) => a + math.min(span(p), cols));
        return Row(
          children: [
            for (var i = 0; i < ps.length; i++) ...[
              if (i > 0) const SizedBox(width: g),
              Expanded(
                flex: math.min(span(ps[i]), cols),
                child: RowOff(off: !enabled, narrow: false, child: ParamTile(fx, ps[i])),
              ),
            ],
            if (last && filled < cols) ...[const SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: const EdgeInsets.only(top: 4),
        child: Column(
          children: [
            for (var i = 0; i < rows.length; i++) ...[if (i > 0) const SizedBox(height: g), row(rows[i], i == rows.length - 1)],
          ],
        ),
      );
    },
  );
}

/// One parameter as a block: the name on top, the number large under it, a picture of the value in the top-right corner.
/// contract: every block still shows the number and takes typing, scrubbing and the sideways slide; the picture is added, never a replacement.
/// contract: numbers scrub sideways, all of them; a picture may be handled its own way (a dial turns, a pad moves both ways).
class ParamTile extends StatelessWidget {
  const ParamTile(this.fx, this.p, {super.key});
  final String fx;
  final Param p;

  String get id => '${fx}_${p.id}';

  Widget _num(BuildContext context, {String? sub, String? caption, Widget? glyph, bool owns = false, bool tall = false}) {
    final fid = sub == null ? id : '$id.$sub';
    return _RowInfo(
      unit: null,
      keys: Ctx.of(context).doc.keys[id] ?? KeyS.off,
      child: NumField(
        id: fid,
        caption: caption ?? p.label,
        glyph: glyph,
        glyphOwnsPointer: owns,
        glyphTall: tall,
        unit: p.unit,
        decimals: p.dec,
        min: p.min,
        max: p.max,
        perPx: p.perPx,
        step: p.dec > 0 ? p.step * 10 : 1,
        bipolar: p.look == ParamLook.bipolar,
        enabled: !RowOff.offOf(context),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, v = doc.get(id);
    final on = !x.cfg.locked && !RowOff.offOf(context);
    Widget gauge(_GaugeKind k, {double? vx, double? vy}) => SizedBox.square(
      dimension: Pop.glyph,
      child: FittedBox(
        child: _Gauge(k, v, p, x: vx, y: vy),
      ),
    );
    // the same skeleton as a number block: name at the top-left, the control along the bottom, the picture top-right
    Widget shell(Widget bottom, {VoidCallback? onTap, Widget? glyph}) => Hov(
      onTap: on ? onTap : null,
      builder: (_, h) => Container(
        height: Pop.tile,
        decoration: BoxDecoration(color: h && onTap != null && on ? Grey.g20 : Pop.well, borderRadius: BorderRadius.circular(6)),
        child: Stack(
          children: [
            if (glyph != null)
              Positioned(
                right: Pop.tilePad - 3,
                top: 4,
                width: Pop.glyph,
                height: Pop.glyph,
                child: Center(child: glyph),
              ),
            Padding(
              padding: EdgeInsets.fromLTRB(Pop.tilePad, 6, glyph != null ? Pop.glyph + Pop.tilePad : Pop.tilePad, 4),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(p.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                  bottom,
                ],
              ),
            ),
          ],
        ),
      ),
    );
    switch (p.look) {
      case ParamLook.toggle:
        final b = (v ?? 0) > 0;
        return shell(
          onTap: () => doc.set(id, b ? 0 : 1),
          Padding(
            padding: const EdgeInsets.only(bottom: 3),
            child: Row(
              children: [
                IgnorePointer(
                  child: Opacity(
                    opacity: on ? 1 : .4,
                    child: _PopSwitch(on: b, onChanged: (_) {}),
                  ),
                ),
                const SizedBox(width: 6),
                Text(b ? 'On' : 'Off', style: T.label(on ? Grey.g76 : Grey.g56)),
              ],
            ),
          ),
        );
      case ParamLook.choice:
        return shell(
          SizedBox(
            height: 22,
            child: p.options.length <= 4
                ? Dis(
                    child: Segmented(
                      items: p.options,
                      index: (v ?? 0).round().clamp(0, p.options.length - 1),
                      expand: true,
                      onChanged: (i) => doc.set(id, i.toDouble()),
                    ),
                  )
                : _ChoiceField(id: id, options: p.options),
          ),
        );
      case ParamLook.colour:
        return shell(
          glyph: _Swatch(doc.s2[id]),
          Padding(
            padding: const EdgeInsets.only(bottom: 3),
            child: LinkLine(to: LinkTo.colors, value: doc.s2[id] ?? '—', id: id),
          ),
        );
      case ParamLook.angle:
        return _num(
          context,
          glyph: _AngleDial(id: id, p: p),
          owns: true,
        );
      case ParamLook.percent:
        return _num(context, glyph: gauge(_GaugeKind.pie));
      case ParamLook.vertical:
        return _num(context, glyph: gauge(p.upIncreases ? _GaugeKind.up : _GaugeKind.down));
      case ParamLook.horizontal:
        return _num(context, glyph: gauge(_GaugeKind.across));
      case ParamLook.bipolar:
        return _num(context, glyph: gauge(_GaugeKind.centre));
      case ParamLook.bar:
        return _num(context, glyph: gauge(_GaugeKind.ramp));
      case ParamLook.scrub:
        return _num(context, glyph: gauge(_GaugeKind.free));
      case ParamLook.seed:
        return _num(
          context,
          glyph: SizedBox.square(
            dimension: Pop.glyph,
            child: FittedBox(
              child: _Dice(id: id, p: p),
            ),
          ),
          owns: true,
        );
      case ParamLook.stepper:
        return _num(
          context,
          glyph: _Steps(id: id, p: p),
          owns: true,
          tall: true,
        );
      case ParamLook.point || ParamLook.size:
        final point = p.look == ParamLook.point;
        return Row(
          children: [
            _PadGlyph(id: id, p: p, size: !point),
            const SizedBox(width: Pop.tileGap),
            Expanded(
              child: _num(context, sub: 'x', caption: '${p.label} ${point ? 'X' : 'W'}'),
            ),
            const SizedBox(width: Pop.tileGap),
            Expanded(
              child: _num(context, sub: 'y', caption: point ? 'Y' : 'H'),
            ),
          ],
        );
    }
  }
}

Color? _hex(String? s) => s != null && RegExp(r'^#[0-9A-Fa-f]{6}$').hasMatch(s) ? Color(int.parse(s.substring(1), radix: 16) | 0xFF000000) : null;

/// The default value of every id a parameter owns, for a fresh layer and for Reset.
Map<String, double> paramDefaults(String fx, Param p) => switch (p.look) {
  ParamLook.point || ParamLook.size => {'${fx}_${p.id}.x': p.def, '${fx}_${p.id}.y': p.defY},
  ParamLook.colour => const {},
  _ => {'${fx}_${p.id}': p.def},
};

/// A colour parameter's default as #RRGGBB, from its 0–1 components.
String paramColour(Decl d) {
  final c = [for (final s in d.attrs['default'] ?? const <String>[]) double.tryParse(s) ?? 1];
  int ch(int i) => ((c.elementAtOrNull(i) ?? 1) * 255).round().clamp(0, 255);
  return '#${[ch(0), ch(1), ch(2)].map((e) => e.toRadixString(16).padLeft(2, '0')).join().toUpperCase()}';
}

class _Swatch extends StatelessWidget {
  const _Swatch(this.hex);
  final String? hex;
  @override
  Widget build(BuildContext context) => Center(
    child: Container(
      width: 16,
      height: 16,
      decoration: BoxDecoration(
        color: _hex(hex) ?? Grey.g26,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: Grey.g38),
      ),
    ),
  );
}

enum _GaugeKind { pie, up, down, across, centre, ramp, free, count, point, box }

/// The value as a small live picture beside its label: a pie for a share, a level for a height, a dot for a point.
class _Gauge extends StatelessWidget {
  const _Gauge(this.kind, this.v, this.p, {this.x, this.y});
  final _GaugeKind kind;
  final double? v, x, y;
  final Param? p;
  @override
  Widget build(BuildContext context) => CustomPaint(size: const Size(22, 22), painter: _GaugePaint(kind, v, p, x, y));
}

class _GaugePaint extends CustomPainter {
  const _GaugePaint(this.kind, this.v, this.p, this.x, this.y);
  final _GaugeKind kind;
  final double? v, x, y;
  final Param? p;

  double _t(double? val, [double? lo, double? hi]) {
    final a = lo ?? p?.min, b = hi ?? p?.max;
    if (val == null || a == null || b == null || b == a) return 0;
    return ((val - a) / (b - a)).clamp(0.0, 1.0);
  }

  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero);
    final dim = Paint()..color = Grey.g26;
    final lit = Paint()..color = Grey.g91;
    final line = Paint()
      ..color = Grey.g56
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    switch (kind) {
      case _GaugeKind.pie:
        final r = Rect.fromCircle(center: o, radius: 8);
        c.drawCircle(o, 8, dim);
        final t = (v ?? 0) / ((p?.max ?? 100) == 0 ? 100 : (p!.max!.abs() > 100 ? p!.max!.abs() : 100));
        c.drawArc(r, -math.pi / 2, 2 * math.pi * t.clamp(-1.0, 1.0), true, lit);
        if (t.abs() > 1) c.drawCircle(o, 8, line..color = Grey.g91);
      case _GaugeKind.up || _GaugeKind.down:
        final box = Rect.fromCenter(center: o, width: 8, height: 18);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(2)), dim);
        final h = box.height * _t(v);
        final fill = kind == _GaugeKind.up
            ? Rect.fromLTRB(box.left, box.bottom - h, box.right, box.bottom)
            : Rect.fromLTRB(box.left, box.top, box.right, box.top + h);
        c.drawRRect(RRect.fromRectAndRadius(fill, const Radius.circular(2)), lit);
        final tip = kind == _GaugeKind.up ? box.top - 1 : box.bottom + 1;
        c.drawLine(Offset(o.dx - 2.5, tip + (kind == _GaugeKind.up ? 2.5 : -2.5)), Offset(o.dx, tip), line);
        c.drawLine(Offset(o.dx + 2.5, tip + (kind == _GaugeKind.up ? 2.5 : -2.5)), Offset(o.dx, tip), line);
      case _GaugeKind.across:
        final box = Rect.fromCenter(center: o, width: 18, height: 8);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(2)), dim);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(box.left, box.top, box.width * _t(v), box.height), const Radius.circular(2)), lit);
      case _GaugeKind.centre:
        final box = Rect.fromCenter(center: o, width: 18, height: 8);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(2)), dim);
        final z = box.left + box.width * _t(0), at = box.left + box.width * _t(v);
        c.drawRect(Rect.fromLTRB(math.min(z, at), box.top, math.max(z, at), box.bottom), lit);
        c.drawLine(Offset(z, box.top - 2), Offset(z, box.bottom + 2), line);
      case _GaugeKind.ramp:
        final t = _t(v);
        for (var i = 0; i < 5; i++) {
          final h = 4.0 + i * 3;
          final r = Rect.fromLTWH(o.dx - 9 + i * 3.8, o.dy + 8 - h, 2.6, h);
          c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(1)), (i + .5) / 5 <= t ? lit : dim);
        }
      case _GaugeKind.free:
        c.drawLine(Offset(o.dx - 7, o.dy), Offset(o.dx + 7, o.dy), line);
        for (final d in const [-1.0, 1.0]) {
          final tip = Offset(o.dx + d * 8, o.dy);
          c.drawLine(tip, tip + Offset(-d * 3, -3), line);
          c.drawLine(tip, tip + Offset(-d * 3, 3), line);
        }
      case _GaugeKind.count:
        final n = ((p?.max ?? 1) - (p?.min ?? 0)).round() + 1, k = ((v ?? 0) - (p?.min ?? 0)).round();
        final cols = math.min(n, 4), rows = (n / cols).ceil().clamp(1, 3);
        for (var i = 0; i < math.min(n, cols * rows); i++) {
          final cx = o.dx + (i % cols - (cols - 1) / 2) * 4.6, cy = o.dy + (i ~/ cols - (rows - 1) / 2) * 4.6;
          c.drawCircle(Offset(cx, cy), 1.6, i <= k ? lit : dim);
        }
      case _GaugeKind.point:
        final box = Rect.fromCenter(center: o, width: 18, height: 18);
        c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(3)), dim);
        final px = box.left + box.width * _t(x), py = box.top + box.height * _t(y);
        c.drawLine(Offset(px, box.top), Offset(px, box.bottom), line);
        c.drawLine(Offset(box.left, py), Offset(box.right, py), line);
        c.drawCircle(Offset(px, py), 2.6, lit);
      case _GaugeKind.box:
        final mx = math.max(x ?? 1, y ?? 1).abs();
        final w = mx == 0 ? 0.0 : 18 * (x ?? 0).abs() / mx, h = mx == 0 ? 0.0 : 18 * (y ?? 0).abs() / mx;
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: o, width: math.max(w, 1.5), height: math.max(h, 1.5)), const Radius.circular(2)), lit);
    }
  }

  @override
  bool shouldRepaint(_GaugePaint o) => o.kind != kind || o.v != v || o.x != x || o.y != y || o.p != p;
}

/// A dial for an angle: turn it by moving around it (the farther out, the finer), or scrub the number beside it.
/// contract: one turn of the pointer is one turn of the value, so the dial never lies about direction; one undo per drag.
class _AngleDial extends StatefulWidget {
  const _AngleDial({required this.id, required this.p});
  final String id;
  final Param p;
  @override
  State<_AngleDial> createState() => _AngleDialState();
}

class _AngleDialState extends State<_AngleDial> {
  double? _last;
  bool _hot = false;
  double _angle(Offset local) => math.atan2(local.dy - 11, local.dx - 11);
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, v = doc.get(widget.id) ?? 0;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    return MouseRegion(
      cursor: on ? SystemMouseCursors.grab : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => _hot = true),
      onExit: (_) => setState(() => _hot = false),
      child: Listener(
        onPointerDown: on
            ? (e) {
                _last = _angle(e.localPosition);
                doc.beginGesture();
              }
            : null,
        onPointerMove: (e) {
          final last = _last;
          if (last == null) return;
          final a = _angle(e.localPosition);
          var d = a - last;
          if (d > math.pi) d -= 2 * math.pi;
          if (d < -math.pi) d += 2 * math.pi;
          _last = a;
          final lo = widget.p.min ?? double.negativeInfinity, hi = widget.p.max ?? double.infinity;
          doc.set(widget.id, ((doc.get(widget.id) ?? 0) + d * 180 / math.pi).clamp(lo, hi).roundToDouble());
        },
        onPointerUp: (_) {
          if (_last != null) doc.endGesture();
          _last = null;
        },
        onPointerCancel: (_) {
          if (_last != null) doc.endGesture();
          _last = null;
        },
        child: CustomPaint(size: const Size(22, 22), painter: _DialPaint(v, _hot || _last != null)),
      ),
    );
  }
}

class _DialPaint extends CustomPainter {
  const _DialPaint(this.deg, this.hot);
  final double deg;
  final bool hot;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), r = 9.0;
    c.drawCircle(o, r, Paint()..color = hot ? Grey.g38 : Grey.g26);
    final a = deg * math.pi / 180;
    c.drawArc(
      Rect.fromCircle(center: o, radius: r - 1.5),
      -math.pi / 2,
      a.clamp(-2 * math.pi, 2 * math.pi),
      false,
      Paint()
        ..color = Grey.g63
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5,
    );
    final tip = o + Offset(math.sin(a), -math.cos(a)) * (r - 1);
    c.drawLine(
      o,
      tip,
      Paint()
        ..color = Grey.g95
        ..strokeWidth = 2
        ..strokeCap = StrokeCap.round,
    );
    c.drawCircle(o, 1.8, Paint()..color = Grey.g95);
  }

  @override
  bool shouldRepaint(_DialPaint o) => o.deg != deg || o.hot != hot;
}

/// A 2D value's handle: drag it like a trackpad, sideways moves X (or width), up and down moves Y (or height); the picture shows where it is.
/// contract: one drag is one undo; Y follows the canvas (down is more) for a point, and up is more for a height.
class _PadGlyph extends StatefulWidget {
  const _PadGlyph({required this.id, required this.p, required this.size});
  final String id;
  final Param p;
  final bool size;
  @override
  State<_PadGlyph> createState() => _PadGlyphState();
}

class _PadGlyphState extends State<_PadGlyph> {
  bool _drag = false, _hot = false;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, p = widget.p;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    final ix = '${widget.id}.x', iy = '${widget.id}.y';
    final lo = p.min ?? double.negativeInfinity, hi = p.max ?? double.infinity;
    double step(double v, double d) => double.parse((v + d * p.perPx).clamp(lo, hi).toStringAsFixed(math.max(p.dec, 2)));
    return MouseRegion(
      cursor: on ? SystemMouseCursors.move : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => _hot = true),
      onExit: (_) => setState(() => _hot = false),
      child: Listener(
        onPointerDown: on
            ? (_) {
                doc.beginGesture();
                setState(() => _drag = true);
              }
            : null,
        onPointerMove: (e) {
          if (!_drag) return;
          doc.setMany({ix: step(doc.get(ix) ?? 0, e.delta.dx), iy: step(doc.get(iy) ?? 0, widget.size ? -e.delta.dy : e.delta.dy)});
        },
        onPointerUp: (_) {
          if (_drag) doc.endGesture();
          setState(() => _drag = false);
        },
        onPointerCancel: (_) {
          if (_drag) doc.endGesture();
          setState(() => _drag = false);
        },
        child: Container(
          width: Pop.tile,
          height: Pop.tile,
          decoration: BoxDecoration(color: _drag || _hot ? Grey.g20 : Pop.well, borderRadius: BorderRadius.circular(6)),
          child: FittedBox(
            child: _Gauge(widget.size ? _GaugeKind.box : _GaugeKind.point, null, p, x: doc.get(ix), y: doc.get(iy)),
          ),
        ),
      ),
    );
  }
}

/// A small whole number's plus and minus, stacked to stay one block tall; the count itself is the field beside them.
class _Steps extends StatelessWidget {
  const _Steps({required this.id, required this.p});
  final String id;
  final Param p;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, v = doc.get(id) ?? p.def;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    Widget btn(String s, double by) {
      final ok = on && (by < 0 ? v > (p.min ?? -1e9) : v < (p.max ?? 1e9));
      return Hov(
        onTap: ok ? () => doc.set(id, (v + by).clamp(p.min ?? -1e9, p.max ?? 1e9)) : null,
        builder: (_, h) => Container(
          width: Pop.glyph,
          height: (Pop.tile - 2 - 8 - 2) / 2,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: h && ok ? Grey.g38 : Grey.g20, borderRadius: BorderRadius.circular(4)),
          child: Text(s, style: T.label(ok ? Grey.g91 : Grey.g44).copyWith(fontWeight: FontWeight.w700, height: 1)),
        ),
      );
    }

    return Column(mainAxisSize: MainAxisSize.min, children: [btn('+', 1), const SizedBox(height: 2), btn('−', -1)]);
  }
}

/// A seed's re-roll: click the die for another random look; the number stays editable beside it.
class _Dice extends StatelessWidget {
  const _Dice({required this.id, required this.p});
  final String id;
  final Param p;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    return Hov(
      onTap: on
          ? () {
              final v = doc.get(id) ?? 0, hi = p.max ?? 9999, lo = p.min ?? 0;
              doc.set(id, (lo + (v * 7919 + 104729) % (hi - lo + 1)).roundToDouble());
            }
          : null,
      builder: (_, h) => CustomPaint(size: const Size(22, 22), painter: _DicePaint(h && on, (doc.get(id) ?? 0).round() % 6 + 1)),
    );
  }
}

class _DicePaint extends CustomPainter {
  const _DicePaint(this.hot, this.face);
  final bool hot;
  final int face;
  @override
  void paint(Canvas c, Size s) {
    final r = Rect.fromCenter(center: s.center(Offset.zero), width: 16, height: 16);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), Paint()..color = hot ? Grey.g95 : Grey.g76);
    const pips = {
      1: [(0, 0)],
      2: [(-1, -1), (1, 1)],
      3: [(-1, -1), (0, 0), (1, 1)],
      4: [(-1, -1), (1, -1), (-1, 1), (1, 1)],
      5: [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)],
      6: [(-1, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (1, 1)],
    };
    for (final (dx, dy) in pips[face]!) {
      c.drawCircle(r.center + Offset(dx * 4.0, dy * 4.0), 1.5, Paint()..color = Grey.g10);
    }
  }

  @override
  bool shouldRepaint(_DicePaint o) => o.hot != hot || o.face != face;
}

/// A long option list for an effect: the same floating chooser as everywhere, storing the index as the value.
class _ChoiceField extends StatelessWidget {
  const _ChoiceField({required this.id, required this.options});
  final String id;
  final List<String> options;
  @override
  Widget build(BuildContext context) => _ChoiceSync(
    id: id,
    options: options,
    child: Chooser(id: id, options: options, word: 'Modes'),
  );
}

/// Keeps a chooser's picked name and the numeric index the shader reads in step.
class _ChoiceSync extends StatelessWidget {
  const _ChoiceSync({required this.id, required this.options, required this.child});
  final String id;
  final List<String> options;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    final i = options.indexOf(doc.s2[id] ?? options.first);
    if (i >= 0 && doc.get(id) != i) {
      WidgetsBinding.instance.addPostFrameCallback((_) => doc.set(id, i.toDouble()));
    }
    return child;
  }
}
