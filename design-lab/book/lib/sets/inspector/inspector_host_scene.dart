part of 'inspector_parts.dart';

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
      const g = Pop.tileGap;
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
            for (var i = 0; i < cs.length; i++) ...[if (i > 0) const SizedBox(width: g), Expanded(flex: cs[i].$1, child: cs[i].$2)],
            if (last && filled < cols) ...[const SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: EdgeInsets.only(top: top),
        child: Column(
          children: [
            for (var i = 0; i < rows.length; i++) ...[if (i > 0) const SizedBox(height: g), row(rows[i], i == rows.length - 1)],
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
            padding: EdgeInsets.fromLTRB(Pop.tilePad, 6, glyph != null ? Pop.glyph + Pop.tilePad : Pop.tilePad, 3),
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
            if (id != null) OnOff(on: (doc.get(id) ?? 0) > 0, onChanged: (v) => doc.set(id, v ? 1 : 0)),
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
                builder: (_, h) => CustomPaint(size: const Size.square(Pop.glyph), painter: _ScProjPaint(m, h)),
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
        _RowInfo(
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
        _ScShell('Target', const Chooser(id: 'hc_target', options: _hcTargets, word: 'Layers', searchable: true)),
        const SizedBox(height: Pop.tileGap),
        SizedBox(
          height: Pop.tile * 2 + Pop.tileGap,
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const _ScOrbit(),
              const SizedBox(width: Pop.tileGap),
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
              glyph: CustomPaint(size: const Size.square(Pop.glyph), painter: _ScFovPaint(doc.get('hc_fov') ?? _hcBaseFov)),
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
  static const _w = Pop.tile * 2 + Pop.tileGap;

  void _put(Doc doc, Offset p) {
    final d = p - const Offset(_w / 2, _w / 2), r = (d.distance / (_w / 2 - 8)).clamp(.05, 1.0);
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

class _ScFovPaint extends CustomPainter {
  const _ScFovPaint(this.fov);
  final double fov;
  @override
  void paint(Canvas c, Size s) {
    final o = Offset(s.width / 2, s.height - 3), half = (fov / 2).clamp(1.0, 85.0) * math.pi / 180, r = 16.0;
    final rect = Rect.fromCircle(center: o, radius: r);
    c.drawArc(rect, -math.pi / 2 - half, half * 2, true, Paint()..color = Grey.g26);
    c.drawArc(
      rect,
      -math.pi / 2 - half,
      half * 2,
      true,
      Paint()
        ..color = Grey.g91
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.2,
    );
  }

  @override
  bool shouldRepaint(_ScFovPaint o) => o.fov != fov;
}

// ---- Extrude / Bevel --------------------------------------------------------------------------------------------------
const _bvProfiles = ['Round', 'Chamfer'];
final _bvRadius = _scN('radius', 'Radius', 8, lo: 0, unit: 'px', dec: 1, look: ParamLook.scrub),
    _bvSegments = Param(id: 'segments', label: 'Segments', look: ParamLook.stepper, type: WType.i32, min: 1, max: 32, def: 6),
    _bvProfile = _scCh('profile', 'Profile', _bvProfiles);

/// A solid card's head: the On switch of an effect; off dims the card and its blocks.
Widget _scOn(Doc doc, String id) => OnOff(on: doc.b[id] ?? true, onChanged: (v) => doc.flag(id, v));

class ExtrudeCard extends StatelessWidget {
  const ExtrudeCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed({'ex_depth': 20});
    final on = doc.b['ex.on'] ?? true, d = doc.get('ex_depth') ?? 20;
    return Sect(
      title: 'Extrude',
      tone: Pop.toneSolid,
      mark: CardMark.solid,
      dim: !on,
      keyIds: const ['ex_depth'],
      brief: ['Depth ${brief(doc.get('ex_depth'))} px'],
      trailing: _scOn(doc, 'ex.on'),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: _ScGrid(top: 0, [
            (
              99,
              _RowInfo(
                unit: null,
                keys: doc.keys['ex_depth'] ?? KeyS.off,
                child: NumField(
                  id: 'ex_depth',
                  caption: 'Depth',
                  unit: 'px',
                  decimals: 1,
                  min: 0,
                  perPx: .5,
                  enabled: on,
                  glyph: CustomPaint(size: const Size.square(Pop.glyph), painter: _ScSolidPaint(d, on)),
                ),
              ),
            ),
          ]),
        ),
      ],
    );
  }
}

/// The outline pushed back by Depth, seen at an angle; deep values ease toward a fixed length so the picture never leaves its box.
class _ScSolidPaint extends CustomPainter {
  const _ScSolidPaint(this.depth, this.on);
  final double depth;
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    final k = 8 * (1 - math.exp(-depth / 40)), f = const Rect.fromLTWH(3, 9, 10, 10), b = f.translate(k, -k);
    final side = Paint()..color = on ? Grey.g44 : Grey.g26;
    c.drawPath(
      Path()
        ..moveTo(f.left, f.top)
        ..lineTo(b.left, b.top)
        ..lineTo(b.right, b.top)
        ..lineTo(b.right, b.bottom)
        ..lineTo(f.right, f.bottom)
        ..lineTo(f.right, f.top)
        ..close(),
      side,
    );
    c.drawRect(f, Paint()..color = on ? Grey.g91 : Grey.g56);
  }

  @override
  bool shouldRepaint(_ScSolidPaint o) => o.depth != depth || o.on != on;
}

/// contract: Chamfer is one flat cut, so Segments only shows for Round.
class BevelCard extends StatelessWidget {
  const BevelCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(_scSeeds('bv', [_bvRadius, _bvSegments, _bvProfile]));
    final on = doc.b['bv.on'] ?? true, chamfer = (doc.get('bv_profile') ?? 0).round() == 1;
    final r = doc.get('bv_radius') ?? 8, n = (doc.get('bv_segments') ?? 6).round().clamp(1, 32);
    return Sect(
      title: 'Bevel',
      tone: Pop.toneSolid,
      mark: CardMark.solid,
      dim: !on,
      keyIds: const ['bv_radius', 'bv_segments'],
      brief: [_bvProfiles[chamfer ? 1 : 0], 'Radius ${brief(r)} px', if (!chamfer) '$n seg'],
      trailing: _scOn(doc, 'bv.on'),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: _ScGrid(top: 0, [
            (
              1,
              _RowInfo(
                unit: null,
                keys: doc.keys['bv_radius'] ?? KeyS.off,
                child: NumField(
                  id: 'bv_radius',
                  caption: 'Radius',
                  unit: 'px',
                  decimals: 1,
                  min: 0,
                  perPx: .25,
                  enabled: on,
                  glyph: CustomPaint(size: const Size.square(Pop.glyph), painter: _ScBevelPaint(r, chamfer ? 1 : n, on)),
                ),
              ),
            ),
            if (!chamfer) _scT('bv', _bvSegments, 1),
            _scT('bv', _bvProfile, 1),
          ]),
        ),
      ],
    );
  }
}

/// A corner of the cap in section: rounded in [steps] facets (one for a chamfer).
class _ScBevelPaint extends CustomPainter {
  const _ScBevelPaint(this.radius, this.steps, this.on);
  final double radius;
  final int steps;
  final bool on;
  @override
  void paint(Canvas c, Size s) {
    const l = 3.0, t = 3.0, r = 19.0, b = 19.0;
    final rr = 14 * (1 - math.exp(-radius / 12));
    final p = Path()
      ..moveTo(l, b)
      ..lineTo(l, t)
      ..lineTo(r - rr, t);
    for (var i = 1; i <= steps; i++) {
      final a = i / steps * math.pi / 2;
      p.lineTo(r - rr + rr * math.sin(a), t + rr - rr * math.cos(a));
    }
    p
      ..lineTo(r, b)
      ..close();
    c.drawPath(p, Paint()..color = on ? Grey.g91 : Grey.g56);
  }

  @override
  bool shouldRepaint(_ScBevelPaint o) => o.radius != radius || o.steps != steps || o.on != on;
}

// ---- Overlay ----------------------------------------------------------------------------------------------------------
const _ovKinds = ['Track Overlay', 'Found Grid', 'Push Trace', 'Physics Trace'], _ovShort = ['Track', 'Grid', 'Push', 'Physics'];
const _ovMethods = ['Motion Detection', 'Key Color', 'Layers'];
final _ovGreen = _scHex(.62, 1, .24);

/// TRACK_OVERLAY_PARAMS in display units (shares as %); colours live in s2 as #RRGGBB.
final _ovParams = <Param>[
  _scCh('method', 'Detection Method', _ovMethods),
  _scN('threshold', 'Threshold', 10, lo: 0, hi: 100),
  _scN('blur', 'Blur Strength', 0, lo: 0, hi: 100),
  _scN('min_region', 'Min Region', 200, lo: 0, unit: 'px', look: ParamLook.scrub),
  _scCol('key_color', 'Key Color'),
  _scSw('show_mask', 'Show Mask'),
  _scN('detail', 'Detail', 960, lo: 120, hi: 3840, unit: 'px'),
  _scN('separation', 'Separation', 2, lo: 0, hi: 200, unit: 'px'),
  _scN('max_region', 'Max Region', 1e9, lo: 0, hi: 1e9, unit: 'px', look: ParamLook.scrub),
  _scSw('keep_ids', 'Keep IDs', true),
  _scSw('links', 'Contact Lines'),
  _scSw('contacts', 'Contact Points'),
  _scSw('velocity', 'Velocity'),
  _scCol('velocity_color', 'Velocity Color'),
  _scSw('hull', 'Outline'),
  _scCol('links_color', 'Line Color'),
  _scN('links_width', 'Line Width', 2, lo: 0, hi: 100, unit: 'px', dec: 1),
  _scSw('well', 'Field Reach'),
  _scCol('well_color', 'Field Color'),
  _scSw('box', 'Box', true),
  _scCh('box_shape', 'Shape', const ['Rectangle', 'Square', 'Ellipse', 'Circle']),
  _scSw('custom_size', 'Custom Size'),
  _scN('custom_radius', 'Custom Radius', 32, lo: 0, unit: 'px', look: ParamLook.scrub),
  _scSw('box_stroke', 'Stroke', true),
  _scCol('box_stroke_color', 'Stroke Color'),
  _scPct('box_stroke_opacity', 'Stroke Opacity', 100),
  _scN('box_stroke_width', 'Stroke Width', 2, lo: 0, hi: 200, unit: 'px', dec: 1),
  _scSw('box_gap', 'Gap'),
  _scPct('box_gap_size', 'Gap Size', 30),
  _scSw('box_fill', 'Fill'),
  _scPct('box_fill_opacity', 'Fill Opacity', 25),
  _scCol('box_fill_color', 'Fill Color'),
  _scSw('marker', 'Marker'),
  _scCh('marker_type', 'Type', const ['Dot', 'Plus', 'Cross', 'Polygon'], 2),
  _scCol('marker_color', 'Color'),
  _scPct('marker_opacity', 'Opacity', 100),
  _scN('marker_size', 'Size', 16, lo: 0, unit: 'px', look: ParamLook.scrub),
  _scN('marker_thickness', 'Thickness', 3, lo: 0, hi: 200, unit: 'px', dec: 1),
  _scN('marker_rotation', 'Rotation', 0, unit: '°', look: ParamLook.angle),
  Param(id: 'polygon_sides', label: 'Sides', look: ParamLook.stepper, type: WType.i32, min: 3, max: 12, def: 3),
  _scSw('polygon_fill', 'Polygon Fill'),
  _scSw('grid', 'Grid'),
  _scCh('grid_mode', 'View Mode', const ['Edge', 'Cartesian']),
  _scN('grid_columns', 'Columns', 12, lo: 1, hi: 400),
  _scN('grid_rows', 'Rows', 8, lo: 1, hi: 400),
  _scCol('grid_color', 'Color'),
  _scPct('grid_opacity', 'Opacity', 85),
  _scN('grid_thickness', 'Thickness', 1, lo: 0, hi: 200, unit: 'px', dec: 1),
  _scN('grid_merge', 'Merge Distance', 12, lo: 0, unit: 'px', look: ParamLook.scrub),
  _scPct('grid_snap', 'Snap Strength', 0),
  _scSw('push', 'Push'),
  _scCol('push_color', 'Color'),
  _scPct('push_opacity', 'Opacity', 100),
  _scN('push_thickness', 'Thickness', 2, lo: 0, hi: 200, unit: 'px', dec: 1),
  _scSw('push_dash', 'Dash', true),
  _scSw('label', 'Labels'),
  _scCh('label_mode', 'Display Mode', const ['Coordinates', 'Dimensions', 'Push', 'Speed']),
  _scCol('label_color', 'Color'),
  _scPct('label_opacity', 'Opacity', 85),
  _scN('font_size', 'Font Size', 18, lo: 1, unit: 'px', look: ParamLook.scrub),
  _scN('label_offset_x', 'Offset X', 0, unit: 'px', look: ParamLook.scrub),
  _scN('label_offset_y', 'Offset Y', 10, unit: 'px', look: ParamLook.scrub),
];
final _ovP = {for (final p in _ovParams) p.id: p};
final _ovColours = <String, String>{
  'key_color': _scHex(0, 1, 0),
  'velocity_color': _scHex(.55, .95, 1),
  'links_color': _scHex(1, .85, .2),
  'well_color': _scHex(.4, .9, 1),
  for (final k in ['box_stroke_color', 'box_fill_color', 'marker_color', 'grid_color', 'label_color']) k: _ovGreen,
  'push_color': _scHex(.88, .23, .18),
};

/// Per-kind default overrides (overlay.rs FOUND_GRID / PUSH_TRACE / PHYSICS_TRACE_DEFAULTS), numbers in display units.
final _ovOverN = <Map<String, double>>[
  {},
  {'method': 2, 'grid': 1, 'grid_opacity': 100, 'grid_thickness': 1.2, 'grid_merge': 40, 'box_stroke_width': 1, 'box_gap': 1, 'box_gap_size': 70},
  {
    'method': 2,
    'box': 1,
    'box_stroke_opacity': 60,
    'box_stroke_width': 1,
    'box_gap': 1,
    'box_gap_size': 80,
    'push': 1,
    'label': 1,
    'label_mode': 2,
    'label_opacity': 100,
  },
  {
    'method': 2,
    'box': 0,
    'hull': 1,
    'contacts': 1,
    'links_width': 1,
    'velocity': 1,
    'well': 1,
    'marker': 1,
    'marker_type': 1,
    'marker_size': 5,
    'marker_thickness': 1,
    'marker_opacity': 90,
    'label': 1,
    'label_mode': 3,
    'label_opacity': 75,
    'font_size': 10,
    'label_offset_x': 0,
    'label_offset_y': 6,
  },
];
final _ovPale = _scHex(.93, .98, .55);
final _ovOverC = <Map<String, String>>[
  {},
  {'grid_color': _scHex(.85, .33, .23), 'box_stroke_color': _scHex(.08, .08, .08)},
  {'box_stroke_color': _scHex(.95, .93, .89), 'label_color': _scHex(.88, .23, .18)},
  {
    for (final k in ['box_stroke_color', 'links_color', 'velocity_color', 'well_color', 'marker_color', 'label_color']) k: _ovPale,
  },
];

/// What a colour chooser offers besides its kind's default: every colour the four kinds ship with, plus white.

Map<String, double> _ovNums(int k) => {
  for (final p in _ovParams)
    if (p.look != ParamLook.colour) 'ov_${p.id}': p.def,
  for (final e in _ovOverN[k].entries) 'ov_${e.key}': e.value,
};
Map<String, String> _ovStrs(int k) => {for (final e in _ovColours.entries) 'ov_${e.key}': e.value, for (final e in _ovOverC[k].entries) 'ov_${e.key}': e.value};

/// One card for the four overlay effects: they share one table of rows and differ only in defaults.
/// contract: picking a kind rewrites every row to that kind's defaults (and Reset goes to them); a part's switch shows its blocks.
class OverlayCard extends StatelessWidget {
  const OverlayCard({super.key});

  static void apply(Doc doc, int k) {
    final nums = {..._ovNums(k), 'ov_kind': k.toDouble()}, strs = _ovStrs(k);
    doc.d.addAll(nums);
    doc.s2.addAll(strs);
    doc.setMany(nums);
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed({..._ovNums(0), 'ov_kind': 0}, strs: _ovStrs(0));
    final k = (doc.get('ov_kind') ?? 0).round().clamp(0, 3), on = doc.b['ov.on'] ?? true, open = doc.b['ov.open'] ?? true;
    double v(String id) => doc.get('ov_$id') ?? 0;
    bool sw(String id) => v(id) > 0;
    final method = v('method').round().clamp(0, 2), layers = method == 2;
    final strs = _ovStrs(k);
    (int, Widget) colour(Param p) {
      final id = 'ov_${p.id}', def = strs[id]!;
      return (
        1,
        _ScShell(
          p.label,
          LinkLine(to: LinkTo.colors, value: doc.s2[id] ?? def, id: id),
          glyph: _Swatch(doc.s2[id]),
        ),
      );
    }

    List<(int, Widget)> ts(List<String> ids) => [
      for (final id in ids)
        if (_ovP[id]!.look == ParamLook.colour) colour(_ovP[id]!) else _scT('ov', _ovP[id]!),
    ];
    Widget grid(List<String> ids) => _ScGrid(ts(ids));
    const parts = [('box', 'Box'), ('marker', 'Marker'), ('grid', 'Grid'), ('push', 'Push'), ('label', 'Labels')];
    final physics = [('links', 'Lines'), ('contacts', 'Contacts'), ('velocity', 'Velocity'), ('hull', 'Outline'), ('well', 'Field')];
    return Sect(
      title: _ovKinds[k],
      sub: true,
      open: open,
      onToggle: () => doc.flag('ov.open', !open),
      tone: Pop.toneOutput,
      mark: CardMark.overlay,
      dim: !on,
      keyIds: [
        for (final p in _ovParams)
          if (p.look != ParamLook.toggle && p.look != ParamLook.choice && p.look != ParamLook.colour) 'ov_${p.id}',
      ],
      brief: [
        _ovMethods[method],
        for (final (id, name) in parts)
          if (sw(id) && (id != 'push' || layers)) name,
        for (final (id, name) in physics)
          if (sw(id)) name,
      ],
      trailing: _scOn(doc, 'ov.on'),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _ScShell(
                'Kind',
                Dis(
                  child: Segmented(items: _ovShort, index: k, expand: true, onChanged: (i) => apply(doc, i)),
                ),
              ),
              const _ScPart('Keying'),
              _ScGrid([
                _scT('ov', _ovP['method']!, 99),
                if (!layers) ...ts(['threshold', 'blur']),
                ...ts(['min_region']),
                if (method == 1) ...ts(['key_color']),
              ]),
              _ScAdvanced('ov.adv', layers ? 4 : 5, [
                grid(['detail', 'separation', 'max_region', 'keep_ids', if (!layers) 'show_mask']),
              ]),
              const _ScPart('Physics'),
              _ScGrid([
                for (final (id, _) in physics) _scT('ov', _ovP[id]!),
                if (sw('links')) ...ts(['links_color', 'links_width']),
                if (sw('velocity')) ...ts(['velocity_color']),
                if (sw('well')) ...ts(['well_color']),
              ]),
              const _ScPart('Box', sw: 'ov_box'),
              if (sw('box'))
                _ScGrid([
                  _scT('ov', _ovP['box_shape']!, 99),
                  ...ts(['custom_size', if (sw('custom_size')) 'custom_radius', 'box_stroke']),
                  if (sw('box_stroke')) ...ts(['box_stroke_color', 'box_stroke_opacity', 'box_stroke_width']),
                  ...ts(['box_gap', if (sw('box_gap')) 'box_gap_size', 'box_fill']),
                  if (sw('box_fill')) ...ts(['box_fill_opacity', 'box_fill_color']),
                ]),
              const _ScPart('Marker', sw: 'ov_marker'),
              if (sw('marker'))
                _ScGrid([
                  _scT('ov', _ovP['marker_type']!, 99),
                  ...ts(['marker_color', 'marker_opacity', 'marker_size', 'marker_thickness', 'marker_rotation']),
                  if (v('marker_type').round() == 3) ...ts(['polygon_sides', 'polygon_fill']),
                ]),
              const _ScPart('Grid', sw: 'ov_grid'),
              if (sw('grid'))
                _ScGrid([
                  ...ts(['grid_mode']),
                  if (v('grid_mode').round() == 1) ...ts(['grid_columns', 'grid_rows']) else ...ts(['grid_merge', if (layers) 'grid_snap']),
                  ...ts(['grid_color', 'grid_opacity', 'grid_thickness']),
                ]),
              if (layers) ...[
                const _ScPart('Push', sw: 'ov_push'),
                if (sw('push')) grid(['push_color', 'push_opacity', 'push_thickness', 'push_dash']),
              ],
              const _ScPart('Labels', sw: 'ov_label'),
              if (sw('label'))
                _ScGrid([
                  _scT('ov', _ovP['label_mode']!, 99),
                  ...ts(['label_color', 'label_opacity', 'font_size', 'label_offset_x', 'label_offset_y']),
                ]),
            ],
          ),
        ),
      ],
    );
  }
}
