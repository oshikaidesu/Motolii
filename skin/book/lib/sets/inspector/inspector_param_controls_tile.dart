// One parameter as a block: the name on top, the number large under it, a picture of the value in the top-right corner.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:lab_book/parts/controls.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_angle_dial.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_gauge.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

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
    return RowInfo(
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
    Widget gauge(GaugeKind k, {double? vx, double? vy}) => SizedBox.square(
      dimension: Pop.glyph,
      child: FittedBox(
        child: Gauge(k, v, p, x: vx, y: vy),
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
                    child: PopSwitch(on: b, onChanged: (_) {}),
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
      case ParamLook.colour when p.unit == 'numbers':
        // numbers, one for each in their order (a bar chart's heights): bars a hand drags up and down, said when let go
        return shell(_Heights(id: id, on: on));
      case ParamLook.colour:
        final all = (doc.s2[id] ?? '—').split(' ').where((c) => hexControls(c) != null).toList();
        return shell(
          glyph: all.length > 1 ? _Ramp(all) : Swatch(doc.s2[id]),
          Padding(
            padding: const EdgeInsets.only(bottom: 3),
            child: LinkLine(to: LinkTo.colors, value: all.length > 1 ? '${all.length}' : doc.s2[id] ?? '—', id: id), // (colours in their order: how many, the strip shows which)
          ),
        );
      case ParamLook.angle:
        return _num(
          context,
          glyph: AngleDial(id: id, p: p),
          owns: true,
        );
      case ParamLook.percent:
        return _num(context, glyph: gauge(GaugeKind.pie));
      case ParamLook.vertical:
        return _num(context, glyph: gauge(p.upIncreases ? GaugeKind.up : GaugeKind.down));
      case ParamLook.horizontal:
        return _num(context, glyph: gauge(GaugeKind.across));
      case ParamLook.bipolar:
        return _num(context, glyph: gauge(GaugeKind.centre));
      case ParamLook.bar:
        return _num(context, glyph: gauge(GaugeKind.ramp));
      case ParamLook.scrub:
        return _num(context, glyph: gauge(GaugeKind.free));
      case ParamLook.seed:
        return _num(
          context,
          glyph: SizedBox.square(
            dimension: Pop.glyph,
            child: FittedBox(
              child: Dice(id: id, p: p),
            ),
          ),
          owns: true,
        );
      case ParamLook.stepper:
        return _num(
          context,
          glyph: Steps(id: id, p: p),
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

/// Colours in their order, as a swatch: a strip from the first to the last.
class _Ramp extends StatelessWidget {
  const _Ramp(this.hexes);
  final List<String> hexes;
  @override
  Widget build(BuildContext context) => Center(
    child: Container(
      width: 16,
      height: 16,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(3),
        border: Border.all(color: Grey.g44, width: .5),
        gradient: LinearGradient(colors: [for (final h in hexes) hexControls(h)!]),
      ),
    ),
  );
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
            child: Gauge(widget.size ? GaugeKind.box : GaugeKind.point, null, p, x: doc.get(ix), y: doc.get(iy)),
          ),
        ),
      ),
    );
  }
}

/// Numbers, one for each in their order, as bars: a hand drags a bar up or down; the row is said once it lets go.
class _Heights extends StatefulWidget {
  const _Heights({required this.id, required this.on});
  final String id;
  final bool on;
  @override
  State<_Heights> createState() => _HeightsState();
}

class _HeightsState extends State<_Heights> {
  List<double>? _held; // (while a hand is on it: the numbers as it moves them)
  double _top = 1;

  List<double> _said(Doc doc) {
    final w = [for (final s in (doc.s2[widget.id] ?? '').split(' ')) ?double.tryParse(s)];
    return w.isEmpty ? List.filled(8, 1) : w; // (none said yet: eight alike, to be dragged)
  }

  void _at(Offset p, Size box) {
    final v = _held!, i = (p.dx / box.width * v.length).floor().clamp(0, v.length - 1);
    setState(() => v[i] = (((1 - p.dy / box.height) * _top) * 10).roundToDouble().clamp(0, _top * 10) / 10);
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, v = _held ?? _said(doc);
    return SizedBox(
      height: 24,
      child: LayoutBuilder(
        builder: (_, box) {
          final size = Size(box.maxWidth, 24);
          void start(Offset at) {
            _held = [..._said(doc)];
            _top = math.max(1, _held!.reduce(math.max) * 1.25);
            _at(at, size);
          }

          void end() {
            doc.str(widget.id, [for (final x in _held!) x == x.roundToDouble() ? '${x.round()}' : '$x'].join(' '));
            setState(() => _held = null);
          }

          // (up and down, as the bars go: the panel's own scrolling does not take the hand away; a tap sets a bar too)
          return GestureDetector(
            behavior: HitTestBehavior.opaque,
            onVerticalDragStart: widget.on ? (d) => start(d.localPosition) : null,
            onVerticalDragUpdate: widget.on ? (d) => _at(d.localPosition, size) : null,
            onVerticalDragEnd: widget.on ? (_) => end() : null,
            onTapUp: widget.on
                ? (d) {
                    start(d.localPosition);
                    end();
                  }
                : null,
            child: CustomPaint(size: size, painter: _BarsPaint(v, _held == null ? math.max(1, v.reduce(math.max) * 1.25) : _top, widget.on)),
          );
        },
      ),
    );
  }
}

class _BarsPaint extends CustomPainter {
  _BarsPaint(this.v, this.top, this.on);
  final List<double> v;
  final double top;
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    final w = s.width / v.length, paint = Paint()..color = on ? Grey.g76 : Grey.g44;
    for (var i = 0; i < v.length; i++) {
      final h = (v[i] / top).clamp(0.0, 1.0) * s.height;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(i * w + w * .15, s.height - h, w * .7, h), const Radius.circular(1.5)), paint);
    }
  }

  @override
  bool shouldRepaint(_BarsPaint o) => o.v != v || o.top != top || o.on != on;
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
    child: Chooser(id: id, options: options),
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
