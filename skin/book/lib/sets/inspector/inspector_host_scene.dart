import 'dart:math' as math;
import 'inspector_studies.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_tile.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';
part 'inspector_host_scene_2.dart';

// ---- host: scene (projection, camera, solids, overlays) -----------------------------------------------------------------
/// contract: ids are `sc_` scene, `hc_` camera, `ex_` extrude, `bv_` bevel, `ov_` overlay; a block's id is `<prefix>_<param>` as in [ParamTile].

Param _scN(String id, String label, double def, {double? lo, double? hi, String unit = '', int dec = 0, ParamLook look = ParamLook.bar}) =>
    Param(id: id, label: label, look: look, type: WType.f32, min: lo, max: hi, def: def, unit: unit, dec: dec);
Param _scPct(String id, String label, double def) => _scN(id, label, def, lo: 0, hi: 100, unit: '%', look: ParamLook.percent);
Param _scSw(String id, String label, [bool on = false]) =>
    Param(id: id, label: label, look: ParamLook.toggle, type: WType.u32, min: 0, max: 1, def: on ? 1 : 0);
Param _scCh(String id, String label, List<String> o, [double def = 0]) =>
    Param(id: id, label: label, look: ParamLook.choice, type: WType.u32, min: 0, max: o.length - 1.0, def: def, options: o);
Param _scCol(String id, String label) => Param(id: id, label: label, look: ParamLook.colour, type: WType.vec4);

String _scHex(double r, double g, double b) =>
    '#${[r, g, b].map((c) => (c * 255).round().clamp(0, 255).toRadixString(16).padLeft(2, '0')).join().toUpperCase()}';

Map<String, double> _scSeeds(String fx, List<Param> ps) => {for (final p in ps) ...paramDefaults(fx, p)};

/// Blocks that flex to the panel's width like [ParamGrid], for cells that are not all [ParamTile]s; a span of 99 takes the whole row.
class _ScGrid extends StatelessWidget {
  const _ScGrid(this.cells, {this.top = 4});
  final List<(int, Widget)> cells;
  final double top;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      final g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <List<(int, Widget)>>[[]];
      var used = 0;
      for (final (s0, w) in cells) {
        final s = math.min(s0, cols);
        if (used + s > cols && rows.last.isNotEmpty) {
          rows.add([]);
          used = 0;
        }
        rows.last.add((s, w));
        used += s;
      }
      Widget row(List<(int, Widget)> cs, bool last) {
        final filled = cs.fold<int>(0, (a, c) => a + c.$1);
        return Row(
          children: [
            for (var i = 0; i < cs.length; i++) ...[if (i > 0) SizedBox(width: g), Expanded(flex: cs[i].$1, child: cs[i].$2)],
            if (last && filled < cols) ...[SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: EdgeInsets.only(top: top),
        child: Column(
          children: [
            for (var i = 0; i < rows.length; i++) ...[if (i > 0) SizedBox(height: g), row(rows[i], i == rows.length - 1)],
          ],
        ),
      );
    },
  );
}

(int, Widget) _scT(String fx, Param p, [int? span]) => (span ?? ParamGrid.span(p), ParamTile(fx, p));

/// A block whose bottom is a control (segments, a chooser) instead of a number: same skeleton as [ParamTile].
class _ScShell extends StatelessWidget {
  const _ScShell(this.caption, this.child, {this.glyph});
  final String caption;
  final Widget child;
  final Widget? glyph;
  @override
  Widget build(BuildContext context) {
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return Container(
      height: Pop.tile,
      decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6)),
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
            padding: EdgeInsets.fromLTRB(Pop.tilePad, 4, glyph != null ? Pop.glyph + Pop.tilePad : Pop.tilePad, 3),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(caption, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                SizedBox(height: 22, child: child),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// A named part inside a card; with [sw] its switch shows or hides the part's blocks.
class _ScPart extends StatelessWidget {
  const _ScPart(this.label, {this.sw});
  final String label;
  final String? sw;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, id = sw;
    return Padding(
      padding: const EdgeInsets.only(top: 6),
      child: SizedBox(
        height: 18,
        child: Row(
          children: [
            Expanded(
              child: Text(label, style: T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700)),
            ),
            if (id != null) ListenableBuilder(listenable: doc.watch(id), builder: (context, _) => OnOff(on: (doc.get(id) ?? 0) > 0, onChanged: (v) => doc.set(id, v ? 1 : 0))),   // (a number shown as a switch: drawn again when it is told anew)
          ],
        ),
      ),
    );
  }
}

/// The Advanced fold, as on an effect card.
class _ScAdvanced extends StatelessWidget {
  const _ScAdvanced(this.flagId, this.count, this.children);
  final String flagId;
  final int count;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), adv = doc.b[flagId] ?? false;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Hov(
            onTap: () => doc.flag(flagId, !adv),
            builder: (_, h) => Container(
              height: 24,
              decoration: BoxDecoration(
                border: Border(top: BorderSide(color: ed.rule)),
              ),
              child: Row(
                children: [
                  Text('Advanced', style: ed.labStyle(h || adv ? Grey.g95 : Grey.g63)),
                  const Spacer(),
                  Text(adv ? 'Fold' : '$count more', style: T.label(Grey.g56)),
                ],
              ),
            ),
          ),
        ),
        if (adv) ...children,
      ],
    );
  }
}

// ---- Scene ------------------------------------------------------------------------------------------------------------
const _scProj = ['2D', '2.5D', '3D'];
final _scToggles = [_scSw('flatten', 'Flatten'), _scSw('environment', 'Environment'), _scSw('auto_orient', 'Auto-orient')];

/// contract: projection defaults to 3D and Flatten defaults to off (the reverse of AE); the glyph cycles 2D → 2.5D → 3D like the Stage's toggle.
class SceneCard extends StatelessWidget {
  const SceneCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed({'sc_projection': 2, ..._scSeeds('sc', _scToggles)});
    final m = (doc.get('sc_projection') ?? 2).round().clamp(0, 2);
    bool on(String id) => (doc.get('sc_$id') ?? 0) > 0;
    return Sect(
      title: 'Scene',
      tone: Pop.toneCamera,
      mark: CardMark.layer,
      brief: [_scProj[m], if (on('flatten')) 'Flatten', if (on('environment')) 'Environment', if (on('auto_orient')) 'Auto-orient'],
      children: [
        _ScGrid(top: 0, [
          (
            99,
            _ScShell(
              'Projection',
              glyph: Hov(
                onTap: Ctx.of(context).cfg.locked ? null : () => doc.set('sc_projection', ((m + 1) % 3).toDouble()),
                builder: (_, h) => CustomPaint(size: Size.square(Pop.glyph), painter: _ScProjPaint(m, h)),
              ),
              Dis(
                child: Segmented(items: _scProj, index: m, expand: true, onChanged: (i) => doc.set('sc_projection', i.toDouble())),
              ),
            ),
          ),
          for (final p in _scToggles) _scT('sc', p),
        ]),
      ],
    );
  }
}

/// 2D: a card on the frame; 2.5D: front-facing cards at two depths; 3D: a card turned in space.
class _ScProjPaint extends CustomPainter {
  const _ScProjPaint(this.m, this.hot);
  final int m;
  final bool hot;
  @override
  void paint(Canvas c, Size s) {
    final lit = Paint()..color = hot ? Grey.g100 : Grey.g91, dim = Paint()..color = Grey.g44;
    final line = Paint()
      ..color = Grey.g56
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    switch (m) {
      case 0:
        c.drawRect(const Rect.fromLTWH(2, 4, 18, 14), line);
        c.drawRect(const Rect.fromLTWH(7, 8, 8, 6), lit);
      case 1:
        c.drawRect(const Rect.fromLTWH(12, 4, 7, 6), dim);
        c.drawRect(const Rect.fromLTWH(3, 9, 11, 9), lit);
      default:
        c.drawPath(
          Path()
            ..moveTo(4, 6)
            ..lineTo(17, 3)
            ..lineTo(17, 19)
            ..lineTo(4, 16)
            ..close(),
          lit,
        );
        c.drawLine(const Offset(4, 11), const Offset(17, 11), Paint()..color = Grey.g44);
    }
  }

  @override
  bool shouldRepaint(_ScProjPaint o) => o.m != m || o.hot != hot;
}

// ---- Camera -----------------------------------------------------------------------------------------------------------
const _hcTargets = ['Point', 'Jewel', 'Hero title', 'Jewel Field', 'Background'];
const _hcBaseFov = 55.0;
double _hcFov(double zoom) => 2 * math.atan(math.tan(_hcBaseFov / 2 * math.pi / 180) / math.max(zoom, 1e-3)) * 180 / math.pi;
double _hcZoom(double fov) => math.tan(_hcBaseFov / 2 * math.pi / 180) / math.tan(fov.clamp(1, 170) / 2 * math.pi / 180);

final _hcPan = _scN('pan', 'Pan', 0, unit: '°', look: ParamLook.angle),
    _hcTilt = _scN('tilt', 'Tilt', 0, lo: -90, hi: 90, unit: '°', look: ParamLook.angle),
    _hcRoll = _scN('roll', 'Roll', 0, unit: '°', look: ParamLook.angle),
    _hcDist = _scN('distance', 'Distance', 100, lo: 1, unit: '%', look: ParamLook.scrub),
    _hcCenter = Param(id: 'center', label: 'Target', look: ParamLook.point, type: WType.vec2, min: -1000, max: 1000, unit: 'px'),
    _hcZ = _scN('target_z', 'Target Z', 0, unit: 'px', look: ParamLook.scrub),
    _hcFade = _scN('near_fade', 'Near fade', 0, lo: 0, unit: 'px', look: ParamLook.scrub);
final _hcAll = [_hcPan, _hcTilt, _hcRoll, _hcDist, _hcCenter, _hcZ, _hcFade];

/// The camera of `ResolvedCamera`: a target (a point, or a layer's centre), orbit around it, distance, zoom and roll.
/// contract: Zoom and Field of view are one value seen two ways (55° at zoom 1); editing either writes both. The real camera has no depth of field — Near fade is its only depth control.
class HostCameraCard extends StatelessWidget {
  const HostCameraCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed({..._scSeeds('hc', _hcAll), 'hc_zoom': 1, 'hc_fov': _hcBaseFov}, strs: {'hc_target': _hcTargets.first});
    final target = doc.s2['hc_target'] ?? _hcTargets.first, free = target == _hcTargets.first;
    void zoom(double z) => doc.setMany({'hc_zoom': z, 'hc_fov': double.parse(_hcFov(z).toStringAsFixed(1))});
    void fov(double f) => doc.setMany({'hc_fov': f, 'hc_zoom': double.parse(_hcZoom(f).toStringAsFixed(3))});
    Widget num(String id, String cap, String unit, int dec, double lo, double hi, double perPx, ValueChanged<double> set, double def, {Widget? glyph}) =>
        RowInfo(
          unit: null,
          keys: doc.keys[id] ?? KeyS.off,
          child: NumField(
            id: id,
            caption: cap,
            unit: unit,
            decimals: dec,
            min: lo,
            max: hi,
            perPx: perPx,
            step: dec > 0 ? .1 : 1,
            glyph: glyph,
            onSet: set,
            onReset: () => set(def),
          ),
        );
    return Sect(
      title: 'Camera',
      tone: Pop.toneCamera,
      mark: CardMark.camera,
      keyIds: const ['hc_pan', 'hc_tilt', 'hc_roll', 'hc_distance', 'hc_zoom', 'hc_center', 'hc_target_z', 'hc_near_fade'],
      brief: [
        free ? 'Point' : '→ $target',
        'Zoom ${brief(doc.get('hc_zoom'), 2)}×',
        'Dist ${brief(doc.get('hc_distance'), 0)}%',
        'Orbit ${brief(doc.get('hc_pan'), 0)}° ${brief(doc.get('hc_tilt'), 0)}°',
      ],
      children: [
        _ScShell('Target', const Chooser(id: 'hc_target', options: _hcTargets, searchable: true)),
        SizedBox(height: Pop.tileGap),
        SizedBox(
          height: Pop.tile * 2 + Pop.tileGap,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const _ScOrbit(),
              SizedBox(width: Pop.tileGap),
              Expanded(
                child: _ScGrid(top: 0, [
                  for (final p in [_hcPan, _hcTilt, _hcDist, _hcRoll]) _scT('hc', p),
                ]),
              ),
            ],
          ),
        ),
        _ScGrid([
          (1, num('hc_zoom', 'Zoom', '×', 2, .05, 20, .01, zoom, 1)),
          (
            1,
            num(
              'hc_fov',
              'Field of view',
              '°',
              1,
              1,
              170,
              .25,
              fov,
              _hcBaseFov,
              glyph: CustomPaint(size: Size.square(Pop.glyph), painter: _ScFovPaint(doc.get('hc_fov') ?? _hcBaseFov)),
            ),
          ),
          if (free) ...[_scT('hc', _hcCenter), _scT('hc', _hcZ)],
        ]),
        if (!free) Cap('Aims at the centre of $target. Pick Point to place the target by hand.'),
        _ScAdvanced('hc.adv', 1, [
          _ScGrid([_scT('hc', _hcFade)]),
          Cap('Things nearer than Near fade thin out and vanish at a third of it. 0 is off.'),
        ]),
      ],
    );
  }
}

/// Top-down orbit: the target in the middle, the camera around it. Drag the camera: around turns Pan, in and out sets Distance.
/// contract: one drag is one undo; the ring is 100 % distance and the edge 400 % (typed values beyond it pin to the edge).
class _ScOrbit extends StatefulWidget {
  const _ScOrbit();
  @override
  State<_ScOrbit> createState() => _ScOrbitState();
}

class _ScOrbitState extends State<_ScOrbit> {
  bool _drag = false, _hot = false;
  static final _w = Pop.tile * 2 + Pop.tileGap;

  void _put(Doc doc, Offset p) {
    final d = p - Offset(_w / 2, _w / 2), r = (d.distance / (_w / 2 - 8)).clamp(.05, 1.0);
    if (d.distance < 2) return;
    final cur = doc.get('hc_pan') ?? 0, th = math.atan2(-d.dx, d.dy) * 180 / math.pi;
    var dd = (th - cur) % 360;
    if (dd > 180) dd -= 360;
    doc.setMany({'hc_pan': (cur + dd).roundToDouble(), 'hc_distance': (400 * r * r).clamp(1.0, 400.0).roundToDouble()});
  }

  void _end(Doc doc) {
    if (_drag) doc.endGesture();
    setState(() => _drag = false);
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final on = !x.cfg.locked && !RowOff.offOf(context);
    return MouseRegion(
      cursor: on ? SystemMouseCursors.move : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => _hot = true),
      onExit: (_) => setState(() => _hot = false),
      child: Listener(
        onPointerDown: on
            ? (e) {
                doc.beginGesture();
                setState(() => _drag = true);
                _put(doc, e.localPosition);
              }
            : null,
        onPointerMove: (e) {
          if (_drag) _put(doc, e.localPosition);
        },
        onPointerUp: (_) => _end(doc),
        onPointerCancel: (_) => _end(doc),
        child: Container(
          key: const ValueKey('hc-orbit'),
          width: _w,
          decoration: BoxDecoration(color: _drag || _hot ? Grey.g20 : Pop.well, borderRadius: BorderRadius.circular(6)),
          child: Stack(
            children: [
              Positioned.fill(
                child: CustomPaint(painter: _ScOrbitPaint(doc.get('hc_pan') ?? 0, doc.get('hc_distance') ?? 100, doc.get('hc_fov') ?? _hcBaseFov, on)),
              ),
              Positioned(
                left: Pop.tilePad,
                top: 6,
                child: Text('Orbit', style: T.micro(on ? Grey.g63 : Grey.g44)),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ScOrbitPaint extends CustomPainter {
  const _ScOrbitPaint(this.pan, this.dist, this.fov, this.on);
  final double pan, dist, fov;
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), big = s.shortestSide / 2 - 8;
    final ring = Paint()
      ..color = Grey.g26
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    c.drawCircle(o, big * .5, ring);
    c.drawCircle(o, big, ring..color = Grey.g20);
    final a = pan * math.pi / 180, dir = Offset(-math.sin(a), math.cos(a));
    final cam = o + dir * big * math.sqrt((dist / 400).clamp(0.0, 1.0));
    final half = (fov / 2).clamp(1.0, 80.0) * math.pi / 180, back = -dir, len = math.max((cam - o).distance + 10, 18.0);
    Offset ray(double t) => cam + Offset(back.dx * math.cos(t) - back.dy * math.sin(t), back.dx * math.sin(t) + back.dy * math.cos(t)) * len;
    c.drawPath(
      Path()
        ..moveTo(cam.dx, cam.dy)
        ..lineTo(ray(-half).dx, ray(-half).dy)
        ..lineTo(ray(half).dx, ray(half).dy)
        ..close(),
      Paint()..color = on ? Grey.g26 : Grey.g20,
    );
    final ink = on ? Grey.g95 : Grey.g56;
    final tick = Paint()
      ..color = ink
      ..strokeWidth = 1.2;
    c.drawLine(o - const Offset(4, 0), o + const Offset(4, 0), tick);
    c.drawLine(o - const Offset(0, 4), o + const Offset(0, 4), tick);
    c.save();
    c.translate(cam.dx, cam.dy);
    c.rotate(a);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 9, height: 6), const Radius.circular(1.5)), Paint()..color = ink);
    c.restore();
  }

  @override
  bool shouldRepaint(_ScOrbitPaint o) => o.pan != pan || o.dist != dist || o.fov != fov || o.on != on;
}
