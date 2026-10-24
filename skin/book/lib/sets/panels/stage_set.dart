import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'stage_parts.dart';
part 'stage_set_2.dart';
part 'stage_set_3.dart';

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
