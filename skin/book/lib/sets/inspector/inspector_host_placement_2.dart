// Part of inspector_host_placement.dart (cut there by size, as it was).
part of 'inspector_host_placement.dart';

class RepeaterCard extends StatelessWidget {
  const RepeaterCard({super.key});

  /// placement.rs default_for: a Circle's Position Each defaults to 0 so the ring closes round the layer; an untouched Each follows the shape.
  static void _pickMode(Doc doc, int k) {
    final old = (doc.get('rep_mode') ?? 0).round();
    final nx = k == 1 ? 0.0 : 100.0, ox = old == 1 ? 0.0 : 100.0;
    final untouched = doc.get('rep_pos_each.x') == ox && doc.get('rep_pos_each.y') == 0;
    doc.d['rep_pos_each.x'] = nx;
    doc.setMany({'rep_mode': k.toDouble(), if (untouched) 'rep_pos_each.x': nx});
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc..seed(_repDefaults, flags: {'rep.on': true});
    final on = doc.b['rep.on'] ?? true, mode = (doc.get('rep_mode') ?? 0).round().clamp(0, 2);
    double g(String id) => doc.get(id) ?? 0;
    final copies = _repCopies(doc), total = g('rep_count').round().clamp(1, 1000);
    final random = _repRows.any((r) => g(r.rand) != 0) || g('rep_delay_rand') != 0;
    return Sect(
      title: 'Repeater',
      tone: Pop.tonePlace,
      mark: CardMark.repeater,
      dim: !on,
      keyIds: _repDefaults.keys.toList(),
      brief: [
        '$total × ${_plShapes[mode]}',
        if (mode == 1) 'R ${brief(g('rep_radius'), 0)} px',
        if (mode == 2) '${brief(g('rep_columns'), 0)} cols',
        'Each ${brief(g('rep_pos_each.x'), 0)}, ${brief(g('rep_pos_each.y'), 0)} px',
        if (random) 'Random',
      ],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('rep.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              PlStrip(painter: RepPaint(copies, total)),
              _PlGrid([
                [
                  (
                    1,
                    _plNum(
                      context,
                      'rep_count',
                      'Count',
                      min: 1,
                      max: 1000,
                      perPx: .2,
                      whole: true,
                      tick: false,
                      glyph: Steps(id: 'rep_count', p: _plCount(1, 1000)),
                      owns: true,
                      tall: true,
                    ),
                  ),
                  (2, _PlChoice('rep_mode', 'Along', _plShapes, onPick: (k) => _pickMode(doc, k))),
                ],
                [
                  if (mode == 2)
                    (
                      1,
                      _plNum(
                        context,
                        'rep_columns',
                        'Columns',
                        min: 1,
                        max: 1000,
                        perPx: .1,
                        whole: true,
                        tick: false,
                        glyph: Steps(id: 'rep_columns', p: _plCount(1, 1000)),
                        owns: true,
                        tall: true,
                      ),
                    ),
                  if (mode == 1) ...[
                    (1, _plNum(context, 'rep_radius', 'Radius', unit: 'px', min: 0, tick: false)),
                    (
                      1,
                      _plNum(
                        context,
                        'rep_start',
                        'Start',
                        unit: '°',
                        glyph: AngleDial(
                          id: 'rep_start',
                          p: const Param(id: 'start', label: '', look: ParamLook.angle, type: WType.f32),
                        ),
                        owns: true,
                      ),
                    ),
                    (1, _plNum(context, 'rep_sweep', 'Sweep', unit: '°', min: 0, max: 360, perPx: 1.8, glyph: _PlArc(g('rep_start'), g('rep_sweep')))),
                  ],
                  (1, _PlChoice('rep_pick', 'Pick', _plPicks)),
                  (1, _PlChoice('rep_transform', 'Transform', _plTransforms)),
                ],
              ]),
              _RepMatrix(_repRows, enabled: on),
              AdvancedFold('rep.adv', summary: '${_repAdvanced.length} more', [_RepMatrix(_repAdvanced, header: false, enabled: on)]),
            ],
          ),
        ),
      ],
    );
  }
}

// ---- Mirror -----------------------------------------------------------------------------------------------------------
/// A small flag and its reflections about the centre, as placement.rs mirrors() places them: x ↦ R(θ) S (x − c) + c.
class _MirPaint extends CustomPainter {
  const _MirPaint(this.axis, this.segments, this.cx, this.cy);
  final int axis, segments;
  final double cx, cy;
  static const k = .25;
  static const flag = [Offset(20, -60), Offset(90, -40), Offset(30, -12)];
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), cc = Offset(cx, cy);
    final copies = switch (axis) {
      0 => [(0.0, 1.0, 1.0), (0.0, -1.0, 1.0)],
      1 => [(0.0, 1.0, 1.0), (0.0, 1.0, -1.0)],
      2 => [(0.0, 1.0, 1.0), (0.0, -1.0, 1.0), (0.0, 1.0, -1.0), (0.0, -1.0, -1.0)],
      _ => [for (var i = 0; i < segments; i++) (2 * math.pi * i / segments, i.isOdd ? -1.0 : 1.0, 1.0)],
    };
    final guide = Paint()
      ..color = Grey.g38
      ..strokeWidth = 1;
    final at = o + cc * k;
    if (axis == 0 || axis == 2) c.drawLine(Offset(at.dx, 0), Offset(at.dx, s.height), guide);
    if (axis == 1 || axis == 2) c.drawLine(Offset(0, at.dy), Offset(s.width, at.dy), guide);
    if (axis == 3) {
      for (var i = 0; i < segments; i++) {
        final a = 2 * math.pi * (i + .5) / segments;
        c.drawLine(at, at + Offset(math.cos(a), math.sin(a)) * s.width, guide);
      }
    }
    for (final (i, (th, sx, sy)) in copies.indexed) {
      Offset tf(Offset p) {
        final d = Offset((p.dx - cc.dx) * sx, (p.dy - cc.dy) * sy);
        return o + (Offset(d.dx * math.cos(th) - d.dy * math.sin(th), d.dx * math.sin(th) + d.dy * math.cos(th)) + cc) * k;
      }

      final path = Path()..addPolygon([for (final p in flag) tf(p)], true);
      c.drawPath(path, Paint()..color = i == 0 ? Pop.tonePlace : Grey.g76.withValues(alpha: .55));
    }
    c.drawCircle(at, 2.5, Paint()..color = Grey.g95);
  }

  @override
  bool shouldRepaint(_MirPaint o) => o.axis != axis || o.segments != segments || o.cx != cx || o.cy != cy;
}

class MirrorCard extends StatelessWidget {
  const MirrorCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc..seed({'mir_mode': 0, 'mir_segments': 6, 'mir_centre.x': 0, 'mir_centre.y': 0}, flags: {'mir.on': true});
    final on = doc.b['mir.on'] ?? true, mode = (doc.get('mir_mode') ?? 0).round().clamp(0, 3);
    final seg = (doc.get('mir_segments') ?? 6).round().clamp(2, 64), cx = doc.get('mir_centre.x') ?? 0, cy = doc.get('mir_centre.y') ?? 0;
    return Sect(
      title: 'Mirror',
      tone: Pop.tonePlace,
      mark: CardMark.mirror,
      dim: !on,
      keyIds: const ['mir_segments', 'mir_centre.x', 'mir_centre.y'],
      brief: [mode == 3 ? 'Radial × $seg' : _plAxes[mode], 'Centre ${brief(cx, 0)}, ${brief(cy, 0)} px'],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('mir.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              PlStrip(
                height: 64,
                painter: _MirPaint(mode, seg, cx, cy),
                onDrag: (d) =>
                    doc.setMany({'mir_centre.x': (cx + d.dx / _MirPaint.k).roundToDouble(), 'mir_centre.y': (cy + d.dy / _MirPaint.k).roundToDouble()}),
              ),
              _PlGrid([
                [(3, const _PlChoice('mir_mode', 'Axis', _plAxes))],
                [
                  (1, _plNum(context, 'mir_centre.x', 'Centre X', unit: 'px')),
                  (1, _plNum(context, 'mir_centre.y', 'Y', unit: 'px')),
                  if (mode == 3)
                    (
                      1,
                      _plNum(
                        context,
                        'mir_segments',
                        'Segments',
                        min: 2,
                        max: 64,
                        perPx: .1,
                        whole: true,
                        glyph: Steps(id: 'mir_segments', p: _plCount(2, 64)),
                        owns: true,
                        tall: true,
                      ),
                    ),
                ],
              ]),
            ],
          ),
        ),
      ],
    );
  }
}

// ---- Blob Track -------------------------------------------------------------------------------------------------------
class BlobTrackCard extends StatelessWidget {
  const BlobTrackCard({super.key});
  static const _defaults = <String, double>{
    'blob_mode': 0, 'blob_threshold': .5, 'blob_invert': 0, 'blob_red': 1, 'blob_green': 0, 'blob_blue': 0, 'blob_tolerance': .25, //
    'blob_detail': 960, 'blob_min_area': 200, 'blob_max_area': 1e9, 'blob_separation': 2, 'blob_max_blobs': 100,
    'blob_persist': 1, 'blob_max_move': 40, 'blob_revive': 5, 'blob_fit': 1, 'blob_material.x': 100, 'blob_material.y': 100,
  };
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc..seed(_defaults, strs: {'blob_source': _plLayers.first}, flags: {'blob.on': true});
    double g(String id) => doc.get(id) ?? 0;
    final on = doc.b['blob.on'] ?? true, mode = g('blob_mode').round().clamp(0, 2), src = doc.s2['blob_source'] ?? _plLayers.first;
    String hex =
        '#${[g('blob_red'), g('blob_green'), g('blob_blue')].map((c) => (c.clamp(0, 1) * 255).round().toRadixString(16).padLeft(2, '0')).join().toUpperCase()}';
    Widget unit(String id, String label) => _plNum(context, id, label, dec: 2, min: 0, max: 1, perPx: .005, glyph: _plGauge(doc.get(id)));
    return Sect(
      title: 'Blob Track',
      tone: Pop.tonePlace,
      mark: CardMark.track,
      dim: !on,
      keyIds: _defaults.keys.toList(),
      brief: [src == _plLayers.first ? 'No layer' : src, '${_plSources[mode]} ${brief(g('blob_threshold'), 2)}', _plFits[g('blob_fit').round().clamp(0, 1)]],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('blob.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const _PlSub('Find'),
              _PlGrid([
                [
                  (
                    3,
                    const _PlShell(
                      label: 'Track Layer',
                      child: SizedBox(
                        height: 22,
                        child: Chooser(id: 'blob_source', options: _plLayers, searchable: true),
                      ),
                    ),
                  ),
                ],
                [
                  (2, const _PlChoice('blob_mode', 'Find By', _plSources)),
                  (1, unit('blob_threshold', 'Threshold')),
                  (1, _plToggle('blob', 'invert', 'Invert')),
                ],
              ]),
              if (mode == 2) ...[
                const _PlSub('Colour'),
                _PlGrid([
                  [
                    (1, unit('blob_red', 'Red')),
                    (1, unit('blob_green', 'Green')),
                    (1, unit('blob_blue', 'Blue')),
                    (1, _plNum(context, 'blob_tolerance', 'Tolerance', dec: 2, min: 0, max: 2, perPx: .01, glyph: Swatch(hex))),
                  ],
                ]),
              ],
              const _PlSub('Place'),
              _PlGrid([
                [
                  (1, const _PlChoice('blob_fit', 'Fit', _plFits)),
                  (1, _plNum(context, 'blob_material.x', 'Material W', unit: 'px', min: .001, tick: false)),
                  (1, _plNum(context, 'blob_material.y', 'H', unit: 'px', min: .001, tick: false)),
                ],
              ]),
              AdvancedFold('blob.adv', summary: '${g('blob_persist') > 0 ? 8 : 6} more', [
                const _PlSub('Filter'),
                _PlGrid([
                  [
                    (1, _plNum(context, 'blob_detail', 'Detail', unit: 'px', min: 120, max: 3840, perPx: 10, whole: true)),
                    (1, _plNum(context, 'blob_min_area', 'Min Size', unit: 'px²', min: 0, max: 1e9, perPx: 10, whole: true, tick: false)),
                    (1, _plNum(context, 'blob_max_area', 'Max Size', unit: 'px²', min: 0, max: 1e9, perPx: 10, whole: true, tick: false)),
                    (1, _plNum(context, 'blob_separation', 'Separation', unit: 'px', min: 0, max: 200, perPx: .5, dec: 1)),
                    (1, _plNum(context, 'blob_max_blobs', 'Max Blobs', min: 1, max: 1000, perPx: .5, whole: true, tick: false)),
                  ],
                ]),
                const _PlSub('Track'),
                _PlGrid([
                  [
                    (1, _plToggle('blob', 'persist', 'Keep IDs')),
                    if (g('blob_persist') > 0) ...[
                      (1, _plNum(context, 'blob_max_move', 'Max Move', unit: 'px', min: 0, max: 10000, whole: true, tick: false)),
                      (1, _plNum(context, 'blob_revive', 'Revive Frames', min: 0, max: 1000, perPx: .2, whole: true, tick: false)),
                    ],
                  ],
                ]),
              ]),
            ],
          ),
        ),
      ],
    );
  }
}

// ---- Motion Blur ------------------------------------------------------------------------------------------------------
/// A dot and its trail, the trail as long as Tune frames of travel.
class _MbPaint extends CustomPainter {
  const _MbPaint(this.tune);
  final double tune;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), w = 4.5 * tune.clamp(0, 4);
    for (var i = 5; i >= 1; i--) {
      c.drawCircle(o + Offset(-w * i / 5 + w / 2, 0), 3.2, Paint()..color = Grey.g91.withValues(alpha: .12 + .1 * (5 - i)));
    }
    c.drawCircle(o + Offset(w / 2, 0), 3.2, Paint()..color = Grey.g95);
  }

  @override
  bool shouldRepaint(_MbPaint o) => o.tune != tune;
}

class MotionBlurCard extends StatelessWidget {
  const MotionBlurCard({super.key});
  static const _parts = [('mb_position', 'Position'), ('mb_scale', 'Scale'), ('mb_angle', 'Angle')];
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc..seed({'mb_tune': 1, 'mb_position': 1, 'mb_scale': 1, 'mb_angle': 1}, flags: {'mb.on': true});
    final on = doc.b['mb.on'] ?? true, tune = doc.get('mb_tune') ?? 1;
    final picked = [
      for (final (id, name) in _parts)
        if ((doc.get(id) ?? 1) >= .5) name,
    ];
    final none = tune <= 0 || picked.isEmpty;
    final live = on && !x.cfg.locked;
    return Sect(
      title: 'Motion Blur',
      tone: Pop.tonePlace,
      mark: CardMark.motionBlur,
      dim: !on,
      keyIds: const ['mb_tune'],
      brief: none ? ['No blur'] : ['Tune ${brief(tune, 2)}', picked.join(' · ')],
      trailing: OnOff(on: on, onChanged: (v) => doc.flag('mb.on', v)),
      children: [
        RowOff(
          off: !on,
          narrow: false,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _PlGrid([
                [
                  (
                    1,
                    _plNum(
                      context,
                      'mb_tune',
                      'Tune',
                      dec: 2,
                      min: 0,
                      max: 4,
                      perPx: .02,
                      glyph: CustomPaint(size: const Size(22, 22), painter: _MbPaint(tune)),
                    ),
                  ),
                  (
                    2,
                    _PlShell(
                      label: 'Blur',
                      child: Row(
                        children: [
                          for (final (i, (id, name)) in _parts.indexed) ...[
                            if (i > 0) const SizedBox(width: 4),
                            Flexible(
                              child: FittedBox(
                                fit: BoxFit.scaleDown,
                                child: Chip(name, on: (doc.get(id) ?? 1) >= .5, enabled: live, onTap: () => doc.set(id, (doc.get(id) ?? 1) >= .5 ? 0 : 1)),
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
                  ),
                ],
              ]),
              if (none) Cap(tune <= 0 ? 'No blur while Tune is 0' : 'No blur: pick Position, Scale or Angle'),
            ],
          ),
        ),
      ],
    );
  }
}
