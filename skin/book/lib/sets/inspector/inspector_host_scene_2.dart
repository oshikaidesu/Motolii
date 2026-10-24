// Part of inspector_host_scene.dart (cut there by size, as it was).
part of 'inspector_host_scene.dart';

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
              RowInfo(
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
                  glyph: CustomPaint(size: Size.square(Pop.glyph), painter: _ScSolidPaint(d, on)),
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
              RowInfo(
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
                  glyph: CustomPaint(size: Size.square(Pop.glyph), painter: _ScBevelPaint(r, chamfer ? 1 : n, on)),
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
          glyph: Swatch(doc.s2[id]),
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
