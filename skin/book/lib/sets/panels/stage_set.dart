import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'stage_parts.dart';

// set: stage_set. The stage (viewport) chrome: it floats over the artwork and stays out of its way.
// Every use case is a 4px-rhythm, translucent-grey, 1px-hairline piece of chrome over a sample artwork.
// Axis colours are Fam.scatter (X), Fam.along (Y), Fam.stagger (Z) reused as axis marks, not as relations.

WidgetbookComponent stageSetSet() => WidgetbookComponent(name: 'stage_set', useCases: [
      uc('Tool column', (c) => _ToolColumnDemo(look: lookKnob(c), initial: c.knobs.int.slider(label: 'Active tool', initialValue: 0, min: 0, max: 5), keys: c.knobs.boolean(label: 'Show shortcuts', initialValue: true), size: c.knobs.int.slider(label: 'Button size', initialValue: 28, min: 24, max: 36).toDouble()), width: 280, height: 300),
      uc('Tool options bar', (c) => _OptionsBarDemo(look: lookKnob(c), tool: c.knobs.object.dropdown<_OptTool>(label: 'Tool', options: _OptTool.values, initialOption: _OptTool.shape, labelBuilder: (t) => t.name)), width: 560, height: 160),
      uc('Selection box', (c) => _SelectionDemo(look: lookKnob(c), rotHandle: c.knobs.boolean(label: 'Rotation handle', initialValue: true), snapAngle: c.knobs.boolean(label: 'Snap angle 15°', initialValue: false), readout: c.knobs.boolean(label: 'Readout while dragging', initialValue: true)), width: 520, height: 320),
      uc('Transform gizmo', (c) => _GizmoDemo(look: lookKnob(c), mode: c.knobs.object.dropdown<_GMode>(label: 'Mode', options: _GMode.values, initialOption: _GMode.move, labelBuilder: (m) => m.name), showZ: c.knobs.boolean(label: 'Show Z', initialValue: true), len: c.knobs.int.slider(label: 'Length', initialValue: 80, min: 48, max: 120).toDouble()), width: 520, height: 320),
      uc('Pivot / anchor marker', (c) => _PivotDemo(look: lookKnob(c), angle: c.knobs.int.slider(label: 'Preview rotation', initialValue: 0, min: -90, max: 90).toDouble(), snap: c.knobs.boolean(label: 'Snap to 9 points', initialValue: true), radius: c.knobs.int.slider(label: 'Marker radius', initialValue: 5, min: 4, max: 8).toDouble()), width: 520, height: 320),
      uc('Guides and rulers', (c) => _GuidesDemo(look: lookKnob(c), zoom: c.knobs.int.slider(label: 'Zoom %', initialValue: 25, min: 15, max: 40) / 100, pointerTick: c.knobs.boolean(label: 'Pointer tick', initialValue: true)), width: 560, height: 340),
      uc('Grid and safe area', (c) => _OverlaysDemo(look: lookKnob(c), grid: c.knobs.boolean(label: 'Grid', initialValue: true), density: c.knobs.int.slider(label: 'Density (0 coarse - 2 fine)', initialValue: 1, min: 0, max: 2), safe: c.knobs.boolean(label: 'Safe area', initialValue: true), mode: c.knobs.object.dropdown<_Safe>(label: 'Safe mode', options: _Safe.values, initialOption: _Safe.both, labelBuilder: (m) => m.name)), width: 520, height: 336),
      uc('Camera frustum (top view)', (c) => _FrustumDemo(look: lookKnob(c), fov: c.knobs.int.slider(label: 'FOV °', initialValue: 54, min: 20, max: 100).toDouble(), near: c.knobs.double.slider(label: 'Near', initialValue: 1, min: .5, max: 3), far: c.knobs.double.slider(label: 'Far', initialValue: 9, min: 5, max: 10)), width: 520, height: 340),
      uc('Zoom HUD and corner label', (c) => _HudDemo(look: lookKnob(c), zoom: c.knobs.int.slider(label: 'Zoom %', initialValue: 100, min: 12, max: 800), corner: c.knobs.object.dropdown<Alignment>(label: 'Label corner', options: const [Alignment.topLeft, Alignment.topRight, Alignment.bottomLeft, Alignment.bottomRight], initialOption: Alignment.topLeft, labelBuilder: (a) => a == Alignment.topLeft ? 'TL' : a == Alignment.topRight ? 'TR' : a == Alignment.bottomLeft ? 'BL' : 'BR'), view: c.knobs.object.dropdown<String>(label: 'View', options: _views, initialOption: 'Camera View')), width: 560, height: 300),
      uc('Cursor tooltip', (c) => _CursorDemo(look: lookKnob(c), crosshair: c.knobs.boolean(label: 'Crosshair', initialValue: true), centre: c.knobs.boolean(label: 'Origin at centre', initialValue: false), percent: c.knobs.boolean(label: 'Percent units', initialValue: false)), width: 520, height: 320),
      uc('Snapping indicator', (c) => _SnapDemo(look: lookKnob(c), on: c.knobs.boolean(label: 'Snap', initialValue: true), thr: c.knobs.int.slider(label: 'Threshold px', initialValue: 8, min: 2, max: 16).toDouble(), distance: c.knobs.boolean(label: 'Distance label', initialValue: true)), width: 520, height: 320),
      uc('Checkerboard backdrop', (c) => _CheckerDemo(look: lookKnob(c), cell: c.knobs.int.slider(label: 'Cell px', initialValue: 8, min: 4, max: 24).toDouble(), alpha: c.knobs.double.slider(label: 'Layer opacity', initialValue: .55, min: 0, max: 1)), width: 520, height: 280),
      uc('Minimap / navigator', (c) => _MiniMapDemo(look: lookKnob(c), zoom: c.knobs.int.slider(label: 'View zoom %', initialValue: 60, min: 25, max: 100) / 100, dim: c.knobs.boolean(label: 'Dim outside view', initialValue: true)), width: 520, height: 300),
      uc('Composition frame', (c) => _CompFrameDemo(look: lookKnob(c), dim: c.knobs.double.slider(label: 'Outside dim', initialValue: .6, min: 0, max: .9), aspect: c.knobs.object.dropdown<double>(label: 'Aspect', options: const [16 / 9, 1, 9 / 16], initialOption: 16 / 9, labelBuilder: (a) => a > 1 ? '16:9' : (a == 1 ? '1:1' : '9:16')), hide: c.knobs.boolean(label: 'Hide outside', initialValue: false), label: c.knobs.boolean(label: 'Label', initialValue: true)), width: 520, height: 320),
    ]);

const _views = ['Front', 'Top', 'Right', 'Perspective', 'Camera View'];

/// A stage: sample artwork, clipped, with children over it.
Widget _stage(List<Widget> children, {Offset shift = Offset.zero}) => ClipRRect(borderRadius: BorderRadius.circular(4), child: Stack(fit: StackFit.expand, children: [SampleArt(shift: shift), ...children]));

// ---------------------------------------------------------------- 1 tool column

const _tools = [(Glyph.select, 'Select', 'V'), (Glyph.move, 'Move', 'M'), (Glyph.shape, 'Rectangle', 'R'), (Glyph.ellipse, 'Ellipse', 'E'), (Glyph.pen, 'Pen', 'P'), (Glyph.text, 'Text', 'T')];

class _ToolColumn extends StatelessWidget {
  const _ToolColumn({required this.active, required this.onChanged, required this.look, required this.keys, required this.size});
  final int active;
  final ValueChanged<int> onChanged;
  final Look look;
  final bool keys;
  final double size;
  @override
  Widget build(BuildContext context) => Plate(
        look: look,
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < _tools.length; i++) ...[
            // [open] a hairline after Move and Pen groups tools (selection / drawing / crop); the grouping is not decided.
            if (i == 2 || i == 5) Container(width: size - 12, height: 1, margin: const EdgeInsets.symmetric(vertical: 4), color: hair(.10)),
            Padding(
              padding: EdgeInsets.only(top: i == 0 || i == 2 || i == 5 ? 0 : 2),
              child: ChromeButton(glyph: _tools[i].$1, on: i == active, look: look, size: size, tip: _tools[i].$2, keys: keys ? _tools[i].$3 : null, onTap: () => onChanged(i)),
            ),
          ],
        ]),
      );
}

class _ToolColumnDemo extends StatelessWidget {
  const _ToolColumnDemo({required this.look, required this.initial, required this.keys, required this.size});
  final Look look;
  final int initial;
  final bool keys;
  final double size;
  @override
  Widget build(BuildContext context) => SpLive<int>(
        initial: initial,
        builder: (context, v, set) => ClipRRect(borderRadius: BorderRadius.circular(4), child: Stack(fit: StackFit.expand, children: [
          const ColoredBox(color: N.g07),
          // the artwork starts 8px clear of the column's right edge, so nothing of it can run under the tools
          Positioned(left: size + 8 + 8 + 8, right: 0, top: 0, bottom: 0, child: const SampleArt()),
          Positioned(left: 8, top: 8, child: _ToolColumn(active: v, onChanged: set, look: look, keys: keys, size: size)),
        ])),
      );
}

// ---------------------------------------------------------------- 2 options bar

enum _OptTool { select, shape, text, pen }

class _OptionsBarDemo extends StatefulWidget {
  const _OptionsBarDemo({required this.look, required this.tool});
  final Look look;
  final _OptTool tool;
  @override
  State<_OptionsBarDemo> createState() => _OptionsBarDemoState();
}

class _OptionsBarDemoState extends State<_OptionsBarDemo> {
  bool snap = true, pivot = false, close = false;
  int level = 0, shapeKind = 0, align = 0, penMode = 0;
  double stroke = 2, radius = 4, size = 24;
  @override
  Widget build(BuildContext context) {
    final look = widget.look;
    final items = switch (widget.tool) {
      _OptTool.select => <Widget>[
          MiniSeg(items: const ['Layer', 'Group'], index: level, onChanged: (i) => setState(() => level = i)),
          const VDivider(),
          ChromeButton(glyph: Glyph.snap, on: snap, size: 24, look: look, tip: 'Snap to layers', keys: '⌘;', tipSide: AxisDirection.down, onTap: () => setState(() => snap = !snap)),
          ChromeButton(glyph: Glyph.pivot, on: pivot, size: 24, look: look, tip: 'Show pivots', tipSide: AxisDirection.down, onTap: () => setState(() => pivot = !pivot)),
        ],
      _OptTool.shape => <Widget>[
          Container(width: 20, height: 20, decoration: BoxDecoration(color: Fam.attach.c, borderRadius: BorderRadius.circular(4), border: Border.all(color: hair(.35)))),
          const SizedBox(width: 4),
          Scrub(label: 'STROKE', value: stroke, step: .1, max: 40, onChanged: (v) => setState(() => stroke = v)),
          Scrub(label: 'RADIUS', value: radius, max: 200, onChanged: (v) => setState(() => radius = v)),
          const VDivider(),
          MiniSeg(items: const ['Rect', 'Ellipse', 'Poly'], index: shapeKind, onChanged: (i) => setState(() => shapeKind = i)),
        ],
      _OptTool.text => <Widget>[
          Hover(builder: (context, h) => AnimatedContainer(duration: kFast, curve: Curves.easeOut, height: 24, padding: const EdgeInsets.only(left: 8, right: 4), decoration: BoxDecoration(color: h ? hair(.08) : shade(.28), borderRadius: BorderRadius.circular(4)), child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [Text('Inter', style: T.name(N.g95)), const SizedBox(width: 4), const GlyphIcon(Glyph.chevron, color: N.g56, size: 14)]))),
          Scrub(label: 'SIZE', value: size, max: 400, onChanged: (v) => setState(() => size = v)),
          const VDivider(),
          MiniSeg(items: const ['L', 'C', 'R'], index: align, onChanged: (i) => setState(() => align = i)),
        ],
      _OptTool.pen => <Widget>[
          MiniSeg(items: const ['Path', 'Mask'], index: penMode, onChanged: (i) => setState(() => penMode = i)),
          const VDivider(),
          GestureDetector(onTap: () => setState(() => close = !close), child: MiniSeg(items: const ['Close path'], index: close ? 0 : -1, onChanged: (_) => setState(() => close = !close))),
        ],
    };
    return _stage([
      Positioned(
        top: 12,
        left: 0,
        right: 0,
        child: Center(
          child: Plate(look: look, padding: const EdgeInsets.all(4), child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [for (var i = 0; i < items.length; i++) ...[if (i > 0) const SizedBox(width: 2), items[i]]])),
        ),
      ),
    ]);
  }
}

// ---------------------------------------------------------------- 3 selection box

class _SelectionDemo extends StatefulWidget {
  const _SelectionDemo({required this.look, required this.rotHandle, required this.snapAngle, required this.readout});
  final Look look;
  final bool rotHandle, snapAngle, readout;
  @override
  State<_SelectionDemo> createState() => _SelectionDemoState();
}

class _SelectionDemoState extends State<_SelectionDemo> {
  Offset c = const Offset(260, 170);
  Size sz = const Size(200, 120);
  double a = -.2, rawA = -.2;
  bool dragging = false;
  static const _hs = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)];
  static const _cur = [SystemMouseCursors.resizeUpLeft, SystemMouseCursors.resizeUp, SystemMouseCursors.resizeUpRight, SystemMouseCursors.resizeRight, SystemMouseCursors.resizeDownRight, SystemMouseCursors.resizeDown, SystemMouseCursors.resizeDownLeft, SystemMouseCursors.resizeLeft];
  static const _stem = 24.0; // [open] distance of the rotation handle from the top edge

  void resize(int hx, int hy, Offset delta) {
    final d = rot(delta, -a);
    final nw = math.max(24.0, sz.width + hx * d.dx), nh = math.max(24.0, sz.height + hy * d.dy);
    final dw = nw - sz.width, dh = nh - sz.height;
    setState(() {
      c += rot(Offset(hx * dw / 2, hy * dh / 2), a);
      sz = Size(nw, nh);
    });
  }

  @override
  Widget build(BuildContext context) {
    final look = widget.look;
    Offset at(double hx, double hy) => c + rot(Offset(hx * sz.width / 2, hy * sz.height / 2), a);
    final rotPos = c + rot(Offset(0, -sz.height / 2 - _stem), a);
    return _stage([
      Positioned(
        left: c.dx - sz.width / 2,
        top: c.dy - sz.height / 2,
        width: sz.width,
        height: sz.height,
        child: Transform.rotate(
          angle: a,
          child: MouseRegion(
            cursor: SystemMouseCursors.move,
            child: GestureDetector(
              onPanStart: (_) => setState(() => dragging = true),
              onPanUpdate: (d) => setState(() => c += d.delta),
              onPanEnd: (_) => setState(() => dragging = false),
              child: DecoratedBox(decoration: BoxDecoration(color: accent.withValues(alpha: .12), border: Border.all(color: accent.withValues(alpha: .6)))),
            ),
          ),
        ),
      ),
      Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _BoxPainter(c, sz, a, look, widget.rotHandle ? _stem : null)))),
      for (var i = 0; i < _hs.length; i++)
        RingHandle(
          at: at(_hs[i].$1.toDouble(), _hs[i].$2.toDouble()),
          look: look,
          cursor: _cur[i],
          onStart: () => setState(() => dragging = true),
          onEnd: () => setState(() => dragging = false),
          onUpdate: (d) => resize(_hs[i].$1, _hs[i].$2, d.delta),
        ),
      if (widget.rotHandle)
        RingHandle(
          at: rotPos,
          round: true,
          look: look,
          cursor: SystemMouseCursors.grab,
          onStart: () => setState(() {
            dragging = true;
            rawA = a;
          }),
          onEnd: () => setState(() => dragging = false),
          onUpdate: (d) {
            final p = c + rot(Offset(0, -sz.height / 2 - _stem), rawA) + d.delta - c;
            setState(() {
              rawA = math.atan2(p.dy, p.dx) + math.pi / 2;
              a = widget.snapAngle ? (rawA / (math.pi / 12)).round() * (math.pi / 12) : rawA;
            });
          },
        ),
      if (widget.readout)
        Positioned(
          left: 0,
          right: 0,
          bottom: 12,
          child: Center(
            child: IgnorePointer(
              child: AnimatedOpacity(
                opacity: dragging ? 1 : 0,
                duration: kFast,
                curve: Curves.easeOut,
                child: Readout([('W', sz.width.round().toString(), axisX), ('H', sz.height.round().toString(), axisY), ('∠', '${(a * 180 / math.pi).toStringAsFixed(1)}°', axisZ)], look: look),
              ),
            ),
          ),
        ),
    ]);
  }
}

class _BoxPainter extends CustomPainter {
  _BoxPainter(this.c, this.sz, this.a, this.look, this.stem);
  final Offset c;
  final Size sz;
  final double a;
  final Look look;
  final double? stem;
  @override
  void paint(Canvas cv, Size s) {
    Offset p(double x, double y) => c + rot(Offset(x * sz.width / 2, y * sz.height / 2), a);
    final box = Path()..moveTo(p(-1, -1).dx, p(-1, -1).dy)..lineTo(p(1, -1).dx, p(1, -1).dy)..lineTo(p(1, 1).dx, p(1, 1).dy)..lineTo(p(-1, 1).dx, p(-1, 1).dy)..close();
    final col = look == Look.quiet ? N.g100.withValues(alpha: .9) : accent;
    if (look == Look.glow) cv.drawPath(box, Paint()..style = PaintingStyle.stroke..strokeWidth = 5..color = accent.withValues(alpha: .35)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    legible(cv, box, col);
    if (stem != null) legibleLine(cv, p(0, -1), c + rot(Offset(0, -sz.height / 2 - stem!), a), col);
  }

  @override
  bool shouldRepaint(_BoxPainter o) => true;
}

// ---------------------------------------------------------------- 4 transform gizmo

enum _GMode { move, scale, rotate }

enum _Ax { x, y, z }

class _GizmoDemo extends StatefulWidget {
  const _GizmoDemo({required this.look, required this.mode, required this.showZ, required this.len});
  final Look look;
  final _GMode mode;
  final bool showZ;
  final double len;
  @override
  State<_GizmoDemo> createState() => _GizmoDemoState();
}

class _GizmoDemoState extends State<_GizmoDemo> {
  Offset c = const Offset(260, 170);
  _Ax? hot, active;
  double val = 0, startAng = 0;
  static Offset dir(_Ax a) => switch (a) { _Ax.x => const Offset(1, 0), _Ax.y => const Offset(0, -1), _Ax.z => const Offset(-.62, .62) };
  // [open] axis colours: family hues muted toward grey; the final axis palette is not decided.
  static Color col(_Ax a) => Color.lerp(switch (a) { _Ax.x => axisX, _Ax.y => axisY, _Ax.z => axisZ }, N.g63, .4)!;

  List<_Ax> get axes => widget.showZ ? _Ax.values : [_Ax.x, _Ax.y];

  _Ax? hit(Offset p) {
    final L = widget.len;
    _Ax? best;
    var bd = double.infinity;
    for (final ax in axes) {
      double d;
      if (widget.mode == _GMode.rotate) {
        final r = L * .8;
        final (rx, ry) = switch (ax) { _Ax.x => (r * .3, r), _Ax.y => (r, r * .3), _Ax.z => (r * .9, r * .9) };
        final q = math.sqrt(math.pow((p.dx - c.dx) / rx, 2) + math.pow((p.dy - c.dy) / ry, 2));
        d = (q - 1).abs() * math.min(rx, ry);
      } else {
        final v = dir(ax) * L, w = p - c;
        final t = ((w.dx * v.dx + w.dy * v.dy) / (v.dx * v.dx + v.dy * v.dy)).clamp(0.0, 1.0);
        d = (w - v * t).distance;
      }
      if (d < 7 && d < bd) {
        bd = d;
        best = ax;
      }
    }
    return best;
  }

  @override
  Widget build(BuildContext context) {
    final u = switch (widget.mode) { _GMode.move => 'px', _GMode.scale => '×', _GMode.rotate => '°' };
    final shown = active == null ? null : (widget.mode == _GMode.scale ? val.toStringAsFixed(2) : '${val >= 0 ? '+' : ''}${val.toStringAsFixed(1)}');
    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: MouseRegion(
        cursor: hot != null ? SystemMouseCursors.click : SystemMouseCursors.basic,
        onHover: (e) {
          final h = hit(e.localPosition);
          if (h != hot) setState(() => hot = h);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanDown: (d) {
            final h = hit(d.localPosition);
            setState(() {
              active = h;
              val = widget.mode == _GMode.scale ? 1 : 0;
              startAng = math.atan2(d.localPosition.dy - c.dy, d.localPosition.dx - c.dx);
            });
          },
          onPanUpdate: (d) {
            final ax = active;
            if (ax == null) return;
            final dr = dir(ax), p = d.localPosition;
            setState(() {
              switch (widget.mode) {
                case _GMode.move:
                  final pr = d.delta.dx * dr.dx + d.delta.dy * dr.dy;
                  c += dr * pr;
                  val += pr;
                case _GMode.scale:
                  val = math.max(.05, ((p.dx - c.dx) * dr.dx + (p.dy - c.dy) * dr.dy) / widget.len);
                case _GMode.rotate:
                  var g = (math.atan2(p.dy - c.dy, p.dx - c.dx) - startAng) * 180 / math.pi;
                  while (g > 180) { g -= 360; }
                  while (g < -180) { g += 360; }
                  val = g;
              }
            });
          },
          onPanEnd: (_) => setState(() => active = null),
          onPanCancel: () => setState(() => active = null),
          child: Stack(fit: StackFit.expand, children: [
            const SampleArt(),
            CustomPaint(painter: _GizmoPainter(c, widget.len, widget.mode, axes, hot, active, widget.look)),
            Positioned(
              left: 12,
              bottom: 12,
              child: IgnorePointer(
                child: AnimatedOpacity(
                  opacity: active == null ? 0 : 1,
                  duration: kFast,
                  curve: Curves.easeOut,
                  child: Readout([(active == null ? '' : active!.name.toUpperCase(), '${shown ?? ''} $u', active == null ? N.g56 : col(active!))], look: widget.look),
                ),
              ),
            ),
          ]),
        ),
      ),
    );
  }
}

class _GizmoPainter extends CustomPainter {
  _GizmoPainter(this.c, this.len, this.mode, this.axes, this.hot, this.active, this.look);
  final Offset c;
  final double len;
  final _GMode mode;
  final List<_Ax> axes;
  final _Ax? hot, active;
  final Look look;
  @override
  void paint(Canvas cv, Size s) {
    // z first, so y and x sit on top
    for (final ax in [...axes.reversed]) {
      final base = _GizmoDemoState.col(ax), d = _GizmoDemoState.dir(ax);
      final lit = ax == hot || ax == active;
      var color = lit ? Color.lerp(base, N.g100, .35)! : base.withValues(alpha: .92);
      if (active != null && ax != active) color = color.withValues(alpha: .35);
      final w = (look == Look.quiet ? 1.5 : 2.0) + (lit ? .5 : 0);
      if (look == Look.glow) {
        cv.drawCircle(c + d * len, lit ? 12 : 9, Paint()..color = base.withValues(alpha: lit ? .5 : .3)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6));
      }
      if (mode == _GMode.rotate) {
        final r = len * .8;
        final (rx, ry) = switch (ax) { _Ax.x => (r * .3, r), _Ax.y => (r, r * .3), _Ax.z => (r * .9, r * .9) };
        legible(cv, Path()..addOval(Rect.fromCenter(center: c, width: rx * 2, height: ry * 2)), color, w: w);
      } else {
        final tip = c + d * len;
        legibleLine(cv, c + d * 6, mode == _GMode.move ? c + d * (len - 6) : tip, color, w: w);
        if (mode == _GMode.move) {
          final n = Offset(-d.dy, d.dx);
          final head = Path()..moveTo(tip.dx, tip.dy)..lineTo((tip - d * 10 + n * 4.5).dx, (tip - d * 10 + n * 4.5).dy)..lineTo((tip - d * 10 - n * 4.5).dx, (tip - d * 10 - n * 4.5).dy)..close();
          cv.drawPath(head, Paint()..color = shade(.35)..style = PaintingStyle.stroke..strokeWidth = 2);
          cv.drawPath(head, Paint()..color = color);
        } else {
          final r = Rect.fromCenter(center: tip, width: 8, height: 8);
          cv.drawRect(r.inflate(1), Paint()..color = shade(.35));
          cv.drawRect(r, Paint()..color = color);
        }
        paintText(cv, ax.name.toUpperCase(), tip + d * 14, T.micro(color).copyWith(fontSize: 11), centerX: true, centerY: true);
      }
    }
    cv.drawRect(Rect.fromCenter(center: c, width: 8, height: 8), Paint()..color = N.g100);
    cv.drawRect(Rect.fromCenter(center: c, width: 8, height: 8), Paint()..style = PaintingStyle.stroke..color = shade(.5));
  }

  @override
  bool shouldRepaint(_GizmoPainter o) => true;
}

// ---------------------------------------------------------------- 5 pivot

class _PivotDemo extends StatefulWidget {
  const _PivotDemo({required this.look, required this.angle, required this.snap, required this.radius});
  final Look look;
  final double angle, radius;
  final bool snap;
  @override
  State<_PivotDemo> createState() => _PivotDemoState();
}

class _PivotDemoState extends State<_PivotDemo> {
  static const _w = 200.0, _h = 120.0;
  static const _o = Offset(160, 100); // layer top-left on the stage
  Offset pv = const Offset(_w / 2, _h / 2), raw = const Offset(_w / 2, _h / 2);
  bool dragging = false;
  bool snapped = false;

  void move(Offset d) {
    raw += d;
    var p = Offset(raw.dx.clamp(0.0, _w), raw.dy.clamp(0.0, _h));
    snapped = false;
    if (widget.snap) {
      for (final fx in [0.0, .5, 1.0]) {
        for (final fy in [0.0, .5, 1.0]) {
          final t = Offset(fx * _w, fy * _h);
          if ((p - t).distance < 8) {
            p = t;
            snapped = true;
          }
        }
      }
    }
    setState(() => pv = p);
  }

  @override
  Widget build(BuildContext context) {
    final world = _o + pv;
    return _stage([
      Positioned(
        left: _o.dx,
        top: _o.dy,
        width: _w,
        height: _h,
        child: Transform.rotate(
          angle: widget.angle * math.pi / 180,
          origin: pv - const Offset(_w / 2, _h / 2),
          child: Stack(fit: StackFit.expand, children: [
            DecoratedBox(decoration: BoxDecoration(border: Border.all(color: N.g95))),
            if (dragging) CustomPaint(painter: _SnapPointsPainter(pv)),
          ]),
        ),
      ),
      Positioned(
        left: world.dx - 14,
        top: world.dy - 14,
        width: 28,
        height: 28,
        child: MouseRegion(
          cursor: SystemMouseCursors.grab,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanStart: (_) => setState(() {
              dragging = true;
              raw = pv;
            }),
            onPanUpdate: (d) => move(d.delta),
            onPanEnd: (_) => setState(() => dragging = false),
            child: CustomPaint(painter: _PivotPainter(widget.look, widget.radius, dragging, snapped)),
          ),
        ),
      ),
      Positioned(left: 12, bottom: 12, child: Readout([('X', '${(pv.dx / _w * 100).round()}%', axisX), ('Y', '${(pv.dy / _h * 100).round()}%', axisY)], look: widget.look)),
    ]);
  }
}

class _SnapPointsPainter extends CustomPainter {
  _SnapPointsPainter(this.pv);
  final Offset pv;
  @override
  void paint(Canvas c, Size s) {
    for (final fx in [0.0, .5, 1.0]) {
      for (final fy in [0.0, .5, 1.0]) {
        c.drawRect(Rect.fromCenter(center: Offset(fx * s.width, fy * s.height), width: 3, height: 3), Paint()..color = N.g100.withValues(alpha: .7));
      }
    }
  }

  @override
  bool shouldRepaint(_SnapPointsPainter o) => false;
}

class _PivotPainter extends CustomPainter {
  _PivotPainter(this.look, this.r, this.hot, this.snapped);
  final Look look;
  final double r;
  final bool hot, snapped;
  @override
  void paint(Canvas c, Size s) {
    final m = s.center(Offset.zero);
    final color = snapped ? accent : N.g100;
    final arm = r + 7;
    if (look == Look.glow) c.drawCircle(m, r + 3, Paint()..color = accent.withValues(alpha: hot ? .5 : .3)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
    if (look != Look.quiet) {
      legible(c, Path()..addOval(Rect.fromCircle(center: m, radius: r)), color, w: 1.5);
      if (look == Look.glow) c.drawCircle(m, r, Paint()..color = accent.withValues(alpha: .35));
    }
    for (final d in const [Offset(1, 0), Offset(-1, 0), Offset(0, 1), Offset(0, -1)]) {
      legibleLine(c, m + d * (look == Look.quiet ? 2 : r + 1.5), m + d * arm, color);
    }
    c.drawCircle(m, 1.5, Paint()..color = color);
  }

  @override
  bool shouldRepaint(_PivotPainter o) => true;
}

// ---------------------------------------------------------------- 6 guides and rulers

class _GuidesDemo extends StatefulWidget {
  const _GuidesDemo({required this.look, required this.zoom, required this.pointerTick});
  final Look look;
  final double zoom;
  final bool pointerTick;
  @override
  State<_GuidesDemo> createState() => _GuidesDemoState();
}

class _GuidesDemoState extends State<_GuidesDemo> {
  static const ruler = 20.0;
  static const origin = Offset(36, 36);
  List<double> vg = [480, 1440], hg = [270];
  Offset? ptr;
  (bool, int)? drag; // (vertical guide?, index)

  double get z => widget.zoom;

  void setG(bool v, int i, double screen) => setState(() => (v ? vg : hg)[i] = ((screen - (v ? origin.dx : origin.dy)) / z));

  void end(bool v, int i, double screen) {
    if (screen < ruler) setState(() => (v ? vg : hg).removeAt(i));
    setState(() => drag = null);
  }

  Widget guide(bool v, int i, Size box) {
    final g = (v ? vg : hg)[i];
    final s = (v ? origin.dx : origin.dy) + g * z;
    final line = Stack(fit: StackFit.expand, children: [
      Center(child: Container(width: v ? 3 : null, height: v ? null : 3, color: shade(.30))),
      Center(child: Container(width: v ? 1 : null, height: v ? null : 1, color: widget.look == Look.quiet ? accent.withValues(alpha: .6) : accent)),
    ]);
    return Positioned(
      left: v ? s - 4 : ruler,
      top: v ? ruler : s - 4,
      width: v ? 9 : box.width - ruler,
      height: v ? box.height - ruler : 9,
      child: MouseRegion(
        cursor: v ? SystemMouseCursors.resizeColumn : SystemMouseCursors.resizeRow,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanStart: (_) => setState(() => drag = (v, i)),
          onPanUpdate: (d) => setG(v, i, s + (v ? d.delta.dx : d.delta.dy)),
          onPanEnd: (_) => end(v, i, s),
          child: line,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final sz = box.biggest;
          final tx = ptr != null && widget.pointerTick ? ptr!.dx : null, ty = ptr != null && widget.pointerTick ? ptr!.dy : null;
          return MouseRegion(
            onHover: (e) => setState(() => ptr = e.localPosition),
            onExit: (_) => setState(() => ptr = null),
            child: Stack(children: [
              Positioned.fill(child: ColoredBox(color: N.g07)),
              Positioned(left: origin.dx, top: origin.dy, width: 1920 * z, height: 1080 * z, child: const SampleArt()),
              for (var i = 0; i < vg.length; i++) guide(true, i, sz),
              for (var i = 0; i < hg.length; i++) guide(false, i, sz),
              Positioned(left: ruler, top: 0, right: 0, height: ruler, child: GestureDetector(
                onVerticalDragStart: (d) => setState(() { hg.add((d.localPosition.dy - origin.dy) / z); drag = (false, hg.length - 1); }),
                onVerticalDragUpdate: (d) => setG(false, hg.length - 1, d.localPosition.dy),
                onVerticalDragEnd: (_) => end(false, hg.length - 1, 0),
                child: CustomPaint(painter: _RulerPainter(true, z, origin.dx - ruler, tx == null ? null : tx - ruler)),
              )),
              Positioned(left: 0, top: ruler, bottom: 0, width: ruler, child: GestureDetector(
                onHorizontalDragStart: (d) => setState(() { vg.add((d.localPosition.dx - origin.dx) / z); drag = (true, vg.length - 1); }),
                onHorizontalDragUpdate: (d) => setG(true, vg.length - 1, d.localPosition.dx),
                onHorizontalDragEnd: (_) => end(true, vg.length - 1, 0),
                child: CustomPaint(painter: _RulerPainter(false, z, origin.dy - ruler, ty == null ? null : ty - ruler)),
              )),
              Positioned(left: 0, top: 0, width: ruler, height: ruler, child: ColoredBox(color: N.g10)),
              if (drag != null)
                Positioned(
                  left: drag!.$1 ? origin.dx + vg[drag!.$2] * z + 10 : 32,
                  top: drag!.$1 ? 28 : origin.dy + hg[drag!.$2] * z + 10,
                  child: IgnorePointer(child: Readout([(drag!.$1 ? 'X' : 'Y', (drag!.$1 ? vg[drag!.$2] : hg[drag!.$2]).round().toString(), drag!.$1 ? axisX : axisY)], look: widget.look)),
                ),
            ]),
          );
        }),
      );
}

class _RulerPainter extends CustomPainter {
  _RulerPainter(this.horizontal, this.zoom, this.origin, this.pointer);
  final bool horizontal;
  final double zoom, origin;
  final double? pointer;
  @override
  void paint(Canvas c, Size s) {
    final len = horizontal ? s.width : s.height, thick = horizontal ? s.height : s.width;
    c.drawRect(Offset.zero & s, Paint()..color = N.g10);
    final edge = Paint()..color = N.g20;
    if (horizontal) {
      c.drawRect(Rect.fromLTWH(0, thick - 1, len, 1), edge);
    } else {
      c.drawRect(Rect.fromLTWH(thick - 1, 0, 1, len), edge);
    }
    const steps = [10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0];
    final step = steps.firstWhere((v) => v * zoom >= 60, orElse: () => 1000);
    final minor = step / 5;
    final k0 = ((0 - origin) / zoom / minor).floor(), k1 = ((len - origin) / zoom / minor).ceil();
    final tick = Paint()..color = N.g44;
    for (var k = k0; k <= k1; k++) {
      final pos = origin + k * minor * zoom;
      if (pos < 0 || pos > len) continue;
      final major = k % 5 == 0;
      final l = major ? 6.0 : 3.0;
      if (horizontal) {
        c.drawRect(Rect.fromLTWH(pos, thick - 1 - l, 1, l), tick);
      } else {
        c.drawRect(Rect.fromLTWH(thick - 1 - l, pos, l, 1), tick);
      }
      if (major) {
        final label = (k * minor).round().toString();
        final st = T.value(N.g63).copyWith(fontSize: 9.5);
        final tw = (TextPainter(text: TextSpan(text: label, style: st), textDirection: TextDirection.ltr)..layout()).width;
        if (pos + 3 + tw > len - 8) continue; // keep 8px clear of the edge
        if (horizontal) {
          paintText(c, label, Offset(pos + 3, 3), st);
        } else {
          c.save();
          c.translate(3, pos - 3);
          c.rotate(-math.pi / 2);
          paintText(c, label, Offset.zero, st);
          c.restore();
        }
      }
    }
    if (pointer != null && pointer! >= 0 && pointer! <= len) {
      final p = Paint()..color = N.g100;
      if (horizontal) {
        c.drawRect(Rect.fromLTWH(pointer!, 0, 1, thick), p);
      } else {
        c.drawRect(Rect.fromLTWH(0, pointer!, thick, 1), p);
      }
    }
  }

  @override
  bool shouldRepaint(_RulerPainter o) => true;
}

// ---------------------------------------------------------------- 7 grid and safe area

enum _Safe { both, action, title }

class _OverlaysDemo extends StatelessWidget {
  const _OverlaysDemo({required this.look, required this.grid, required this.density, required this.safe, required this.mode});
  final Look look;
  final bool grid, safe;
  final int density;
  final _Safe mode;
  @override
  Widget build(BuildContext context) => SpLive<(bool, int, bool)>(
        initial: (grid, density, safe),
        builder: (context, v, set) => ClipRRect(borderRadius: BorderRadius.circular(4), child: Stack(fit: StackFit.expand, children: [
          const ColoredBox(color: N.g07),
          Positioned(left: 0, right: 0, top: 0, bottom: 44, child: ClipRect(child: Stack(fit: StackFit.expand, children: [
            const SampleArt(),
            if (v.$1) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _GridPainter([64.0, 32.0, 16.0][v.$2])))),
            if (v.$3) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _SafePainter(mode)))),
          ]))),
          Positioned(
            left: 8,
            bottom: 8,
            child: Plate(look: look, child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
              ChromeButton(glyph: Glyph.grid, size: 24, look: look, on: v.$1, tip: 'Grid', keys: "⌘'", tipSide: AxisDirection.up, onTap: () => set((!v.$1, v.$2, v.$3))),
              const SizedBox(width: 2),
              AnimatedOpacity(duration: kFast, curve: Curves.easeOut, opacity: v.$1 ? 1 : .4, child: MiniSeg(items: const ['64', '32', '16'], index: v.$2, onChanged: (i) => set((v.$1, i, v.$3)))),
              const VDivider(),
              ChromeButton(glyph: Glyph.safe, size: 24, look: look, on: v.$3, tip: 'Safe area', tipSide: AxisDirection.up, onTap: () => set((v.$1, v.$2, !v.$3))),
            ])),
          ),
        ])),
      );
}

class _GridPainter extends CustomPainter {
  _GridPainter(this.step);
  final double step;
  @override
  void paint(Canvas c, Size s) {
    // DESIGN.md: one line style, white 9% between and 16% at the major step; every 4th line is major.
    final minor = Paint()..color = N.g100.withValues(alpha: .09), major = Paint()..color = N.g100.withValues(alpha: .16);
    var i = 0;
    for (var x = 0.0; x < s.width; x += step, i++) {
      c.drawRect(Rect.fromLTWH(x, 0, 1, s.height), i % 4 == 0 ? major : minor);
    }
    i = 0;
    for (var y = 0.0; y < s.height; y += step, i++) {
      c.drawRect(Rect.fromLTWH(0, y, s.width, 1), i % 4 == 0 ? major : minor);
    }
  }

  @override
  bool shouldRepaint(_GridPainter o) => o.step != step;
}

class _SafePainter extends CustomPainter {
  _SafePainter(this.mode);
  final _Safe mode;
  @override
  void paint(Canvas c, Size s) {
    void frame(double inset, String label, {bool right = false}) {
      final r = Rect.fromLTRB(s.width * inset, s.height * inset, s.width * (1 - inset), s.height * (1 - inset));
      legible(c, Path()..addRect(r), N.g100.withValues(alpha: .55));
      // action label sits in the top-left corner, title label in the top-right, so they never meet
      final tp = TextPainter(text: TextSpan(text: label, style: T.micro(N.g91)), textDirection: TextDirection.ltr)..layout();
      paintText(c, label, right ? r.topRight + Offset(-8 - tp.width, 8) : r.topLeft + const Offset(8, 8), T.micro(N.g91));
    }

    if (mode != _Safe.title) frame(.035, 'ACTION 93%');
    if (mode != _Safe.action) frame(.05, 'TITLE 90%', right: true);
    final m = s.center(Offset.zero);
    legibleLine(c, m + const Offset(-5, 0), m + const Offset(5, 0), N.g100.withValues(alpha: .55));
    legibleLine(c, m + const Offset(0, -5), m + const Offset(0, 5), N.g100.withValues(alpha: .55));
  }

  @override
  bool shouldRepaint(_SafePainter o) => o.mode != mode;
}

// ---------------------------------------------------------------- 8 camera frustum

class _FrustumDemo extends StatefulWidget {
  const _FrustumDemo({required this.look, required this.fov, required this.near, required this.far});
  final Look look;
  final double fov, near, far;
  @override
  State<_FrustumDemo> createState() => _FrustumDemoState();
}

class _FrustumDemoState extends State<_FrustumDemo> {
  double camX = 0;
  int sel = 1;
  static const layers = [(-2.2, 3.0, 2.4), (1.5, 5.0, 1.8), (-.5, 7.0, 3.0), (3.5, 8.5, 1.5)];
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final sx = box.maxWidth / 10, sz = (box.maxHeight - 48) / 10;
          final cam = Offset(box.maxWidth / 2 + camX * sx, box.maxHeight - 24);
          final half = math.tan(widget.fov * math.pi / 360);
          return GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanUpdate: (d) => setState(() => camX = (camX + d.delta.dx / sx).clamp(-4.0, 4.0)),
            child: Stack(fit: StackFit.expand, children: [
              ColoredBox(color: N.g10),
              CustomPaint(painter: _FrustumPainter(cam, sx, sz, half, widget.near, widget.far, [for (var i = 0; i < layers.length; i++) (layers[i], i == sel)], widget.look)),
              for (var i = 0; i < layers.length; i++)
                Positioned(
                  left: box.maxWidth / 2 + (layers[i].$1 - layers[i].$3 / 2) * sx,
                  top: box.maxHeight - 24 - layers[i].$2 * sz - 8,
                  width: layers[i].$3 * sx,
                  height: 16,
                  child: MouseRegion(cursor: SystemMouseCursors.click, child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => setState(() => sel = i))),
                ),
              Positioned(left: 12, top: 12, child: Text('TOP', style: T.micro(N.g56).copyWith(letterSpacing: .8))),
              Positioned(left: 12, bottom: 12, child: Readout([('FOV', '${widget.fov.round()}°', N.g56), ('NEAR', widget.near.toStringAsFixed(1), axisZ), ('FAR', widget.far.toStringAsFixed(1), axisZ)], look: widget.look)),
            ]),
          );
        }),
      );
}

class _FrustumPainter extends CustomPainter {
  _FrustumPainter(this.cam, this.sx, this.sz, this.half, this.near, this.far, this.layers, this.look);
  final Offset cam;
  final double sx, sz, half, near, far;
  final List<((double, double, double), bool)> layers;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final base = s.height - 24;
    // depth ticks: one hairline kind, labels are values
    for (var z = 0; z <= 10; z++) {
      c.drawRect(Rect.fromLTWH(0, base - z * sz, s.width, 1), Paint()..color = N.g100.withValues(alpha: z % 5 == 0 ? .09 : .04));
      if (z % 2 == 0) paintText(c, '$z', Offset(s.width - 24, base - z * sz - 12), T.value(N.g63).copyWith(fontSize: 10));
    }
    Offset pt(double z, double sign) => Offset(cam.dx + sign * z * half * sx, base - z * sz);
    final poly = Path()..moveTo(pt(near, -1).dx, pt(near, -1).dy)..lineTo(pt(far, -1).dx, pt(far, -1).dy)..lineTo(pt(far, 1).dx, pt(far, 1).dy)..lineTo(pt(near, 1).dx, pt(near, 1).dy)..close();
    c.drawPath(poly, Paint()..color = look == Look.glow ? accent.withValues(alpha: .10) : N.g100.withValues(alpha: .05));
    final edge = look == Look.quiet ? N.g63.withValues(alpha: .6) : N.g76;
    c.drawPath(poly, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = edge);
    c.drawLine(cam, pt(near, -1), Paint()..color = edge.withValues(alpha: .4));
    c.drawLine(cam, pt(near, 1), Paint()..color = edge.withValues(alpha: .4));
    // layers: inside the frustum bright, outside dim, the selected one accent
    for (final l in layers) {
      final (x, z, w) = l.$1;
      final cx = s.width / 2 + x * sx;
      final inside = (cx - cam.dx).abs() <= z * half * sx + w * sx / 2 && z >= near && z <= far;
      final col = l.$2 ? accent : (inside ? N.g91 : N.g38);
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, base - z * sz), width: w * sx, height: 4), const Radius.circular(2)), Paint()..color = col);
    }
    // the camera: a body and a lens, pointing up
    final body = RRect.fromRectAndRadius(Rect.fromCenter(center: cam + const Offset(0, 3), width: 16, height: 10), const Radius.circular(2));
    final lens = Path()..moveTo(cam.dx - 4, cam.dy - 2)..lineTo(cam.dx - 6, cam.dy - 9)..lineTo(cam.dx + 6, cam.dy - 9)..lineTo(cam.dx + 4, cam.dy - 2);
    c.drawRRect(body, Paint()..color = N.g10);
    c.drawPath(lens, Paint()..color = N.g10);
    c.drawRRect(body, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeJoin = StrokeJoin.round..color = N.g95);
    c.drawPath(lens, Paint()..style = PaintingStyle.stroke..strokeWidth = 1.4..strokeJoin = StrokeJoin.round..color = N.g95);
    // axis key: X right, Z up
    const o = Offset(24, 0);
    final a0 = Offset(o.dx, s.height - 56 - 0);
    legibleLine(c, a0, a0 + const Offset(20, 0), axisX, w: 1.5);
    legibleLine(c, a0, a0 + const Offset(0, -20), axisZ, w: 1.5);
    paintText(c, 'X', a0 + const Offset(26, 0), T.micro(axisX), centerY: true);
    paintText(c, 'Z', a0 + const Offset(0, -28), T.micro(axisZ), centerX: true, centerY: true);
  }

  @override
  bool shouldRepaint(_FrustumPainter o) => true;
}

// ---------------------------------------------------------------- 9 zoom HUD and corner label

class _HudDemo extends StatelessWidget {
  const _HudDemo({required this.look, required this.zoom, required this.corner, required this.view});
  final Look look;
  final int zoom;
  final Alignment corner;
  final String view;
  @override
  Widget build(BuildContext context) => SpLive<int>(
        initial: zoom,
        builder: (context, z, set) => _stage([
          Positioned(left: 12, right: 12, top: 12, bottom: 12, child: Align(alignment: corner, child: _CornerLabel(key: ValueKey(view), initial: view, look: look, up: corner.y > 0, right: corner.x > 0))),
          Positioned(left: 0, right: 0, bottom: 12, child: Center(child: _ZoomPill(zoom: z, onChanged: set, look: look))),
        ]),
      );
}

const _zoomSteps = [12, 25, 50, 100, 200, 400, 800];
const _fitZoom = 64; // [open] the "fit" value depends on stage size; fixed here for the demo

class _ZoomPill extends StatelessWidget {
  const _ZoomPill({required this.zoom, required this.onChanged, required this.look});
  final int zoom;
  final ValueChanged<int> onChanged;
  final Look look;
  Widget chip(String t, bool on, VoidCallback f) => Hover(
        onTap: f,
        builder: (context, h) => AnimatedContainer(duration: kFast, curve: Curves.easeOut, height: 24, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.center, decoration: BoxDecoration(color: on ? hair(.14) : (h ? hair(.08) : hair(0)), borderRadius: BorderRadius.circular(4)), child: Text(t, style: T.label(on ? N.g100 : N.g76))),
      );
  @override
  Widget build(BuildContext context) => Plate(
        look: look,
        padding: const EdgeInsets.all(2),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          ChromeButton(glyph: Glyph.minus, size: 24, look: look, onTap: () => onChanged(_zoomSteps.lastWhere((s) => s < zoom, orElse: () => _zoomSteps.first))),
          Hover(
            onTap: () => onChanged(_zoomSteps.firstWhere((s) => s > zoom, orElse: () => _zoomSteps.first)),
            builder: (context, h) => SizedBox(width: 52, child: Center(child: Text('$zoom%', style: T.value(h ? N.g100 : N.g95)))),
          ),
          ChromeButton(glyph: Glyph.plus, size: 24, look: look, onTap: () => onChanged(_zoomSteps.firstWhere((s) => s > zoom, orElse: () => _zoomSteps.last))),
          const VDivider(),
          chip('Fit', zoom == _fitZoom, () => onChanged(_fitZoom)),
          chip('1:1', zoom == 100, () => onChanged(100)),
        ]),
      );
}

class _CornerLabel extends StatefulWidget {
  const _CornerLabel({super.key, required this.initial, required this.look, required this.up, required this.right});
  final String initial;
  final Look look;
  final bool up, right;
  @override
  State<_CornerLabel> createState() => _CornerLabelState();
}

class _CornerLabelState extends State<_CornerLabel> {
  late String v = widget.initial;
  bool open = false;
  @override
  Widget build(BuildContext context) {
    final head = Hover(
      onTap: () => setState(() => open = !open),
      builder: (context, h) => AnimatedContainer(
        duration: kFast,
        curve: Curves.easeOut,
        height: 24,
        padding: const EdgeInsets.only(left: 8, right: 4),
        decoration: BoxDecoration(color: N.g10.withValues(alpha: h || open ? .8 : .55), borderRadius: BorderRadius.circular(4), border: Border.all(color: hair(h || open ? .14 : .08))),
        child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Text(v, style: T.name(h || open ? N.g100 : N.g91)),
          const SizedBox(width: 2),
          GlyphIcon(Glyph.chevron, color: N.g56, size: 14),
        ]),
      ),
    );
    final list = AnimatedSize(
      duration: kFast,
      curve: Curves.easeOut,
      alignment: widget.up ? Alignment.bottomCenter : Alignment.topCenter,
      child: !open
          ? const SizedBox(width: 0, height: 0)
          : Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Plate(look: widget.look, padding: const EdgeInsets.all(2), child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                for (final s in _views)
                  Hover(
                    onTap: () => setState(() { v = s; open = false; }),
                    builder: (context, h) => Container(height: 24, width: 120, padding: const EdgeInsets.symmetric(horizontal: 8), alignment: Alignment.centerLeft, decoration: BoxDecoration(color: s == v ? hair(.14) : (h ? hair(.08) : hair(0)), borderRadius: BorderRadius.circular(3)), child: Text(s, style: T.name(s == v ? N.g100 : N.g76))),
                  ),
              ])),
            ),
    );
    return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: widget.right ? CrossAxisAlignment.end : CrossAxisAlignment.start, children: widget.up ? [list, head] : [head, list]);
  }
}

// ---------------------------------------------------------------- 10 cursor tooltip

class _CursorDemo extends StatefulWidget {
  const _CursorDemo({required this.look, required this.crosshair, required this.centre, required this.percent});
  final Look look;
  final bool crosshair, centre, percent;
  @override
  State<_CursorDemo> createState() => _CursorDemoState();
}

class _CursorDemoState extends State<_CursorDemo> {
  Offset? p, down;
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final W = box.maxWidth, H = box.maxHeight, k = 1920 / W;
          String fmt(double v, double axisLen) => widget.percent ? '${(v / axisLen * 100).toStringAsFixed(1)}%' : (v * k).round().toString();
          const w = 112.0;
          final h = down != null ? 44.0 : 24.0;
          Offset corner() {
            final q = p ?? Offset(W * .45, H * .4);
            return Offset(q.dx + 14 + w > W ? q.dx - 14 - w : q.dx + 14, q.dy + 16 + h > H ? q.dy - 16 - h : q.dy + 16);
          }

          final q = p ?? Offset(W * .45, H * .4); // a fixed sample until the pointer arrives
          final ox = widget.centre ? W / 2 : 0.0, oy = widget.centre ? H / 2 : 0.0;
          return Listener(
            onPointerHover: (e) => setState(() => p = e.localPosition),
            onPointerMove: (e) => setState(() => p = e.localPosition),
            onPointerDown: (e) => setState(() => down = e.localPosition),
            onPointerUp: (_) => setState(() => down = null),
            child: MouseRegion(
              cursor: SystemMouseCursors.precise,
              onExit: (_) => setState(() => p = null),
              child: Stack(fit: StackFit.expand, children: [
                const SampleArt(),
                if (widget.crosshair) IgnorePointer(child: CustomPaint(painter: _CrossPainter(q))),
                if (true)
                  Positioned(
                    left: corner().dx,
                    top: corner().dy,
                    width: w,
                    child: IgnorePointer(
                      child: Plate(look: widget.look, radius: 4, padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6), child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Flex(direction: Axis.horizontal, children: [Text('X', style: T.micro(axisX)), const SizedBox(width: 4), Text(fmt(q.dx - ox, W), style: T.value(N.g95)), const Spacer(), Text('Y', style: T.micro(axisY)), const SizedBox(width: 4), Text(fmt(q.dy - oy, H), style: T.value(N.g95))]),
                        if (down != null) ...[
                          const SizedBox(height: 8),
                          Flex(direction: Axis.horizontal, children: [Text('Δ', style: T.micro(N.g56)), const SizedBox(width: 4), Text('${fmt(q.dx - down!.dx, W)}, ${fmt(q.dy - down!.dy, H)}', style: T.value(accent))]),
                        ],
                      ])),
                    ),
                  ),
              ]),
            ),
          );
        }),
      );
}

class _CrossPainter extends CustomPainter {
  _CrossPainter(this.p);
  final Offset p;
  @override
  void paint(Canvas c, Size s) {
    final paint = Paint()..color = N.g100.withValues(alpha: .22);
    c.drawRect(Rect.fromLTWH(0, p.dy.floorToDouble(), s.width, 1), paint);
    c.drawRect(Rect.fromLTWH(p.dx.floorToDouble(), 0, 1, s.height), paint);
  }

  @override
  bool shouldRepaint(_CrossPainter o) => o.p != p;
}

// ---------------------------------------------------------------- 11 snapping indicator

class _SnapDemo extends StatefulWidget {
  const _SnapDemo({required this.look, required this.on, required this.thr, required this.distance});
  final Look look;
  final bool on, distance;
  final double thr;
  @override
  State<_SnapDemo> createState() => _SnapDemoState();
}

class _SnapDemoState extends State<_SnapDemo> {
  static const others = [Rect.fromLTWH(70, 50, 130, 80), Rect.fromLTWH(330, 170, 110, 90)];
  Offset raw = const Offset(85, 150); // starts snapped, so the guides show statically
  static const msz = Size(100, 64);
  bool dragging = false;

  // returns the snapped rect plus the matched (vertical line x, other) and (horizontal line y, other)
  (Rect, (double, Rect)?, (double, Rect)?) solve() {
    var r = raw & msz;
    if (!widget.on) return (r, null, null);
    double bx = double.infinity, by = double.infinity;
    (double, Rect)? mx, my;
    for (final o in others) {
      for (final ox in [o.left, o.center.dx, o.right]) {
        for (final mxv in [r.left, r.center.dx, r.right]) {
          final d = ox - mxv;
          if (d.abs() < widget.thr && d.abs() < bx.abs()) { bx = d; mx = (ox, o); }
        }
      }
      for (final oy in [o.top, o.center.dy, o.bottom]) {
        for (final myv in [r.top, r.center.dy, r.bottom]) {
          final d = oy - myv;
          if (d.abs() < widget.thr && d.abs() < by.abs()) { by = d; my = (oy, o); }
        }
      }
    }
    r = r.shift(Offset(mx == null ? 0 : bx, my == null ? 0 : by));
    return (r, mx, my);
  }

  @override
  Widget build(BuildContext context) {
    final (m, mx, my) = solve();
    final lines = <Widget>[];
    Widget pill(Offset at, String t) => Positioned(left: at.dx - 18, top: at.dy - 9, width: 36, height: 18, child: Container(alignment: Alignment.center, decoration: BoxDecoration(color: N.g10.withValues(alpha: .88), borderRadius: BorderRadius.circular(3), border: Border.all(color: hair(.10))), child: Text(t, style: T.value(N.g95))));
    var vx = -1.0, hy = -1.0;
    var vr = Rect.zero, hr = Rect.zero;
    if (mx != null) {
      vx = mx.$1;
      vr = mx.$2;
      if (widget.distance) {
        final gap = m.top >= vr.bottom ? m.top - vr.bottom : (vr.top >= m.bottom ? vr.top - m.bottom : 0.0);
        if (gap > 0) lines.add(pill(Offset(vx, m.top >= vr.bottom ? (vr.bottom + m.top) / 2 : (m.bottom + vr.top) / 2), gap.round().toString()));
      }
    }
    if (my != null) {
      hy = my.$1;
      hr = my.$2;
      if (widget.distance) {
        final gap = m.left >= hr.right ? m.left - hr.right : (hr.left >= m.right ? hr.left - m.right : 0.0);
        if (gap > 0) lines.add(pill(Offset(m.left >= hr.right ? (hr.right + m.left) / 2 : (m.right + hr.left) / 2, hy), gap.round().toString()));
      }
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: Stack(fit: StackFit.expand, children: [
        const SampleArt(),
        for (var i = 0; i < others.length; i++)
          Positioned.fromRect(rect: others[i], child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: N.g63)))),
        Positioned.fromRect(
          rect: m,
          child: MouseRegion(
            cursor: SystemMouseCursors.move,
            child: GestureDetector(
              onPanStart: (_) => setState(() => dragging = true),
              onPanUpdate: (d) => setState(() => raw += d.delta),
              onPanEnd: (_) => setState(() { raw = m.topLeft; dragging = false; }),
              child: DecoratedBox(decoration: BoxDecoration(color: shade(.25), border: Border.all(color: N.g95))),
            ),
          ),
        ),
        IgnorePointer(child: CustomPaint(painter: _SnapPainter(m, mx == null ? null : (vx, vr), my == null ? null : (hy, hr), widget.look))),
        ...lines,
        Positioned(left: 12, bottom: 12, child: IgnorePointer(child: AnimatedOpacity(opacity: dragging || mx != null || my != null ? 0 : 1, duration: kFast, curve: Curves.easeOut, child: Text('Drag the box', style: T.label(N.g76))))),
      ]),
    );
  }
}

class _SnapPainter extends CustomPainter {
  _SnapPainter(this.m, this.v, this.h, this.look);
  final Rect m;
  final (double, Rect)? v, h;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final col = look == Look.quiet ? magenta.withValues(alpha: .75) : magenta;
    void tick(Offset p, bool vertical) => legibleLine(c, p + (vertical ? const Offset(-3, 0) : const Offset(0, -3)), p + (vertical ? const Offset(3, 0) : const Offset(0, 3)), col);
    void line(Offset a, Offset b, bool vertical) {
      if (look == Look.glow) c.drawLine(a, b, Paint()..color = magenta.withValues(alpha: .5)..strokeWidth = 4..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3));
      legibleLine(c, a, b, col);
      if (look != Look.quiet) {
        tick(a, vertical);
        tick(b, vertical);
      }
    }

    if (v != null) line(Offset(v!.$1, math.min(m.top, v!.$2.top)), Offset(v!.$1, math.max(m.bottom, v!.$2.bottom)), true);
    if (h != null) line(Offset(math.min(m.left, h!.$2.left), h!.$1), Offset(math.max(m.right, h!.$2.right), h!.$1), false);
  }

  @override
  bool shouldRepaint(_SnapPainter o) => true;
}

// ---------------------------------------------------------------- 12 checkerboard backdrop

class _CheckerDemo extends StatelessWidget {
  const _CheckerDemo({required this.look, required this.cell, required this.alpha});
  final Look look;
  final double cell, alpha;
  @override
  Widget build(BuildContext context) {
    // concept: dark greys (quiet in the chrome); quiet: barely-there; glow: the light checker (reads alpha on dark art).
    final (a, b) = switch (look) { Look.concept => (N.g15, N.g20), Look.quiet => (N.g10, N.g13), Look.glow => (N.g63, N.g76) };
    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: Stack(fit: StackFit.expand, children: [
        CustomPaint(painter: _CheckerPainter(cell, a, b)),
        Center(child: Container(width: 220, height: 140, decoration: BoxDecoration(borderRadius: BorderRadius.circular(2), gradient: LinearGradient(colors: [Fam.scatter.c.withValues(alpha: alpha), Fam.attach.c.withValues(alpha: alpha)])))),
        Center(child: Container(width: 100, height: 100, decoration: BoxDecoration(shape: BoxShape.circle, color: Fam.along.c.withValues(alpha: alpha * .8)))),
      ]),
    );
  }
}

class _CheckerPainter extends CustomPainter {
  _CheckerPainter(this.cell, this.a, this.b);
  final double cell;
  final Color a, b;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = a);
    final p = Paint()..color = b;
    for (var y = 0; y * cell < s.height; y++) {
      for (var x = 0; x * cell < s.width; x++) {
        if ((x + y).isOdd) c.drawRect(Rect.fromLTWH(x * cell, y * cell, cell, cell), p);
      }
    }
  }

  @override
  bool shouldRepaint(_CheckerPainter o) => o.cell != cell || o.a != a || o.b != b;
}

// ---------------------------------------------------------------- 13 minimap

class _MiniMapDemo extends StatefulWidget {
  const _MiniMapDemo({required this.look, required this.zoom, required this.dim});
  final Look look;
  final double zoom;
  final bool dim;
  @override
  State<_MiniMapDemo> createState() => _MiniMapDemoState();
}

class _MiniMapDemoState extends State<_MiniMapDemo> {
  static const world = Size(1600, 900), mini = Size(160, 90), stageSz = Size(520, 300);
  Offset v = const Offset(300, 200); // view top-left in world px
  bool hot = false;
  Size get viewSz => Size(stageSz.width / widget.zoom, stageSz.height / widget.zoom);
  Offset clampV(Offset o) => Offset(o.dx.clamp(0.0, math.max(0.0, world.width - viewSz.width)), o.dy.clamp(0.0, math.max(0.0, world.height - viewSz.height)));

  @override
  Widget build(BuildContext context) {
    const m = 160 / 1600;
    final vv = clampV(v);
    final vr = Rect.fromLTWH(vv.dx * m, vv.dy * m, viewSz.width * m, viewSz.height * m);
    void center(Offset p) => setState(() => v = clampV(Offset(p.dx / m - viewSz.width / 2, p.dy / m - viewSz.height / 2)));
    return ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: Stack(fit: StackFit.expand, children: [
        ColoredBox(color: N.g07),
        OverflowBox(
          alignment: Alignment.topLeft,
          minWidth: 0,
          minHeight: 0,
          maxWidth: world.width * widget.zoom,
          maxHeight: world.height * widget.zoom,
          child: Transform.translate(offset: -vv * widget.zoom, child: SizedBox(width: world.width * widget.zoom, height: world.height * widget.zoom, child: const SampleArt())),
        ),
        Positioned(
          right: 12,
          bottom: 12,
          child: Container(
            width: mini.width + 8,
            height: mini.height + 8,
            padding: const EdgeInsets.all(4),
            decoration: plateDeco(widget.look),
            child: MouseRegion(
              onEnter: (_) => setState(() => hot = true),
              onExit: (_) => setState(() => hot = false),
              cursor: SystemMouseCursors.grab,
              child: GestureDetector(
                onPanDown: (d) => center(d.localPosition),
                onPanUpdate: (d) => center(d.localPosition),
                child: Stack(fit: StackFit.expand, children: [
                  const SampleArt(),
                  CustomPaint(painter: _MiniPainter(vr, widget.dim, hot, widget.look)),
                ]),
              ),
            ),
          ),
        ),
      ]),
    );
  }
}

class _MiniPainter extends CustomPainter {
  _MiniPainter(this.vr, this.dim, this.hot, this.look);
  final Rect vr;
  final bool dim, hot;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final all = Path()..addRect(Offset.zero & s);
    final view = Path()..addRect(vr.intersect(Offset.zero & s));
    if (dim) c.drawPath(Path.combine(PathOperation.difference, all, view), Paint()..color = shade(.55));
    c.drawRect(vr, Paint()..color = accent.withValues(alpha: hot ? .22 : .12));
    legible(c, Path()..addRect(vr.deflate(.5)), look == Look.quiet ? N.g100 : accent);
  }

  @override
  bool shouldRepaint(_MiniPainter o) => true;
}

// ---------------------------------------------------------------- 14 composition frame

class _CompFrameDemo extends StatefulWidget {
  const _CompFrameDemo({required this.look, required this.dim, required this.aspect, required this.hide, required this.label});
  final Look look;
  final double dim, aspect;
  final bool hide, label;
  @override
  State<_CompFrameDemo> createState() => _CompFrameDemoState();
}

class _CompFrameDemoState extends State<_CompFrameDemo> {
  Offset shift = Offset.zero;
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: LayoutBuilder(builder: (context, box) {
          final s = box.biggest;
          final maxW = s.width - 96, maxH = s.height - 72;
          final w = math.min(maxW, maxH * widget.aspect), h = w / widget.aspect;
          final f = Rect.fromCenter(center: Offset(s.width / 2, s.height / 2 + 8), width: w, height: h);
          final pxW = widget.aspect == 1 ? 1080 : (widget.aspect > 1 ? 1920 : 1080), pxH = widget.aspect == 1 ? 1080 : (widget.aspect > 1 ? 1080 : 1920);
          final art = Transform.translate(offset: shift, child: OverflowBox(maxWidth: s.width * 1.6, maxHeight: s.height * 1.6, child: SizedBox(width: s.width * 1.6, height: s.height * 1.6, child: const SampleArt())));
          return GestureDetector(
            onPanUpdate: (d) => setState(() => shift += d.delta),
            child: Stack(fit: StackFit.expand, children: [
              ColoredBox(color: N.g07),
              if (widget.hide) Positioned.fromRect(rect: f, child: ClipRect(child: Transform.translate(offset: Offset(-f.left, -f.top), child: OverflowBox(alignment: Alignment.topLeft, minWidth: 0, minHeight: 0, maxWidth: s.width, maxHeight: s.height, child: SizedBox(width: s.width, height: s.height, child: art))))) else Positioned.fill(child: art),
              if (!widget.hide) IgnorePointer(child: CustomPaint(painter: _DimPainter(f, widget.dim))),
              IgnorePointer(child: CustomPaint(painter: _FramePainter(f, widget.look))),
              if (widget.label) Positioned(left: f.left, top: f.top - 18, child: IgnorePointer(child: Text('Composition 1  ·  $pxW × $pxH', style: T.label(N.g63)))),
            ]),
          );
        }),
      );
}

class _DimPainter extends CustomPainter {
  _DimPainter(this.f, this.dim);
  final Rect f;
  final double dim;
  @override
  void paint(Canvas c, Size s) => c.drawPath(Path.combine(PathOperation.difference, Path()..addRect(Offset.zero & s), Path()..addRect(f)), Paint()..color = N.g07.withValues(alpha: dim));
  @override
  bool shouldRepaint(_DimPainter o) => o.f != f || o.dim != dim;
}

class _FramePainter extends CustomPainter {
  _FramePainter(this.f, this.look);
  final Rect f;
  final Look look;
  @override
  void paint(Canvas c, Size s) {
    final r = f.inflate(.5);
    switch (look) {
      case Look.concept:
        c.drawRect(r, Paint()..style = PaintingStyle.stroke..color = hair(.5));
      case Look.quiet:
        break;
      case Look.glow:
        c.drawRect(r, Paint()..style = PaintingStyle.stroke..strokeWidth = 4..color = accent.withValues(alpha: .35)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4));
        c.drawRect(r, Paint()..style = PaintingStyle.stroke..color = accent);
    }
  }

  @override
  bool shouldRepaint(_FramePainter o) => o.f != f || o.look != look;
}
