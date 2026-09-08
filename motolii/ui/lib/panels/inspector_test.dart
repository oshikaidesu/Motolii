import 'package:flutter/material.dart';

import '../foundation/metrics.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';
import '../session/read_model.dart';

/// The Inspector as controls, not rows. Every property row the snapshot sends
/// is turned into one control by [_kindOf]; nothing here is laid out by hand
/// per effect, so a new Vism with declared params gets its sheet for free.
///
/// Routes are the Inspector's own: previewProperties / commitPreview for
/// values, setAttrs / anchor / ghost / clip for the layer, focusEditing for
/// Blend (the Desk owns the picker). Nothing is re-implemented.
class InspectorTestPanel extends StatefulWidget {
  const InspectorTestPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<InspectorTestPanel> createState() => _InspectorTestPanelState();
}

/// What a control is, decided from the declaration, never from the label.
enum _Kind { bounded, scalar, angle, vec2, scale, color, choice, text }

_Kind _kindOf(Map<String, dynamic> row) {
  final id = '${row['id']}';
  final kind = '${row['kind']}';
  if (row['choices'] is List || kind == 'enum') return _Kind.choice;
  if (kind == 'color') return _Kind.color;
  if (kind == 'text') return _Kind.text;
  if (id == 'scale') return _Kind.scale;
  if (id.startsWith('rotation') || id == 'camera.roll') return _Kind.angle;
  if (id.contains('.param.') &&
      row['value'] is num &&
      _characterOf(row) == _Character.angle) {
    return _Kind.angle;
  }
  if (kind == 'vec2' || row['value'] is List) return _Kind.vec2;
  if (row['min'] != null && row['max'] != null) return _Kind.bounded;
  return _Kind.scalar;
}

/// What a parameter is *for*, read off its declared name. One table, so a
/// new Vism gets its glyph, unit and control the moment it is declared.
enum _Character {
  amount,
  size,
  ratio,
  angle,
  place,
  seed,
  time,
  count,
  color,
  level,
  soft,
  opacity,
  direction,
  choice,
  delay,
  detail,
  none,
}

const _characterWords = <_Character, List<String>>{
  _Character.seed: ['seed'],
  _Character.angle: ['angle', 'rotation', 'roll', 'tilt', 'spin'],
  _Character.place: ['position', 'offset', 'center', 'centre', 'origin'],
  _Character.size: [
    'size',
    'radius',
    'width',
    'height',
    'thickness',
    'scale',
    'length',
    'distance',
  ],
  _Character.opacity: ['opacity', 'alpha', 'transparency'],
  _Character.soft: ['softness', 'blur', 'feather', 'spread', 'smooth'],
  _Character.time: ['evolution', 'time', 'phase', 'speed', 'rate', 'frequency'],
  _Character.delay: ['delay', 'lag'],
  _Character.count: ['count', 'columns', 'rows', 'copies', 'number', 'steps'],
  _Character.detail: [
    'complexity',
    'octaves',
    'detail',
    'iterations',
    'quality',
  ],
  _Character.color: ['color', 'colour', 'tint', 'hue'],
  _Character.level: ['threshold', 'level', 'gamma', 'contrast', 'brightness'],
  _Character.direction: ['along', 'direction', 'axis', 'side'],
  _Character.amount: [
    'amount',
    'strength',
    'intensity',
    'mix',
    'gain',
    'weight',
    'density',
    'power',
  ],
};

const _characterGlyphs = <_Character, IconData>{
  _Character.amount: Icons.tune,
  _Character.size: Icons.straighten,
  _Character.ratio: Icons.aspect_ratio,
  _Character.angle: Icons.rotate_right,
  _Character.place: Icons.open_with,
  _Character.seed: Icons.casino_outlined,
  _Character.time: Icons.timelapse,
  _Character.count: Icons.grid_on,
  _Character.color: Icons.palette_outlined,
  _Character.level: Icons.linear_scale,
  _Character.soft: Icons.blur_on,
  _Character.opacity: Icons.opacity,
  _Character.direction: Icons.alt_route,
  _Character.choice: Icons.category_outlined,
  _Character.delay: Icons.history_toggle_off,
  _Character.detail: Icons.grain,
};

/// The declared or analysed subtype (Blender vocabulary) wins over words.
_Character? _declaredCharacter(Map<String, dynamic> row) =>
    switch ('${row['subtype'] ?? ''}') {
      'SEED' => _Character.seed,
      'ANGLE' => _Character.angle,
      'OPACITY' => _Character.opacity,
      'DISTANCE' || 'PIXEL' => _Character.size,
      'TRANSLATION' => _Character.place,
      'TIME' => _Character.time,
      'LEVEL' => _Character.level,
      'FACTOR' || 'PERCENTAGE' => _Character.amount,
      'COUNT' => _Character.count,
      _ => null,
    };

_Character _characterOf(Map<String, dynamic> row) {
  final declared = _declaredCharacter(row);
  if (declared != null) return declared;
  final id = '${row['id']}'.split('.param.').last;
  final words = <String>{
    ...id.toLowerCase().split(RegExp('[^a-z]+')),
    ...'${row['label'] ?? ''}'.toLowerCase().split(RegExp('[^a-z]+')),
  }..remove('');
  for (final entry in _characterWords.entries) {
    if (entry.value.any(words.contains)) return entry.key;
  }
  if (row['choices'] is List) return _Character.choice;
  return _Character.none;
}

IconData? _glyphOf(Map<String, dynamic> row) =>
    _characterGlyphs[_characterOf(row)];

String? _unitOf(Map<String, dynamic> row) {
  final id = '${row['id']}';
  if (_kindOf(row) == _Kind.angle) return '°';
  if (id == 'opacity') return '%';
  if (id.startsWith('position') || id == 'anchor' || id == 'camera.center') {
    return 'px';
  }
  if (row['unit'] is String) return row['unit'] as String;
  if (id.contains('.param.')) {
    final max = row['max'] as num?;
    return switch (_characterOf(row)) {
      _Character.angle => '°',
      _Character.place || _Character.size || _Character.soft => 'px',
      _Character.opacity ||
      _Character.amount when max == 1 || max == 100 => '%',
      _ => null,
    };
  }
  return null;
}

class _InspectorTestPanelState extends State<InspectorTestPanel> {
  EditorSession get c => widget.controller;
  final _nodes = <String, FocusNode>{};
  bool _scaleLocked = true;

  Map<String, dynamic>? get _active {
    for (final layer in c.liveLayers()) {
      if (c.selectedIds.isNotEmpty && layer['id'] == c.selectedIds.last)
        return layer;
    }
    return c.activeLayer;
  }

  @override
  void dispose() {
    for (final n in _nodes.values) {
      n.dispose();
    }
    super.dispose();
  }

  Map<String, dynamic>? _property(Map<String, dynamic> layer, String id) {
    for (final row in [
      ...panelRows(layer['properties']),
      for (final effect in panelRows(layer['effects']))
        ...panelRows(effect['params']),
    ]) {
      if (row['id'] == id) return row;
    }
    return null;
  }

  bool _canEdit(Map<String, dynamic> layer) =>
      layer['locked'] != true &&
      panelCan(c, 'previewProperties') &&
      panelCan(c, 'commitPreview');

  /// One absolute value into one axis of a property: preview during a drag,
  /// preview + commit for a typed number.
  Future<void> _write(
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
    dynamic next, {
    required bool preview,
  }) async {
    await c.command('previewProperties', {
      'edits': [
        {'layer': layer['id'], 'property': row['id'], 'value': next},
      ],
    });
    if (!preview) await c.command('commitPreview');
  }

  Future<void> _writeMany(
    Map<String, dynamic> layer,
    Map<String, double> values, {
    required bool preview,
  }) async {
    await c.command('previewProperties', {
      'edits': [
        for (final e in values.entries)
          {'layer': layer['id'], 'property': e.key, 'value': e.value},
      ],
    });
    if (!preview) await c.command('commitPreview');
  }

  dynamic _withAxis(dynamic current, int axis, double v) {
    if (current is List) {
      final out = List<dynamic>.from(current);
      if (axis < out.length) out[axis] = v;
      return out;
    }
    return v;
  }

  Future<void> _finish(bool cancel) =>
      c.command(cancel ? 'cancelPreview' : 'commitPreview');

  /// The numeric well: one axis of a row. Bounded rows paint their amount.
  Widget _well(
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
    int axis, {
    String? label,
    double? width,
    bool fill = false,
  }) {
    width ??= _wellWidth;
    final v = row['value'];
    final value = v is List && axis < v.length
        ? v[axis]
        : axis == 0 && v is num
        ? v
        : null;
    if (value is! num) return SizedBox(width: width);
    final slotWidth = width;
    final id = '${row['id']}';
    // Opacity is declared without a range; it is 0..1 by meaning.
    final min =
            (row['min'] as num?)?.toDouble() ?? (id == 'opacity' ? 0 : null),
        max = (row['max'] as num?)?.toDouble() ?? (id == 'opacity' ? 1 : null);
    final unit = _unitOf(row);
    final percent = unit == '%' && max == 1;
    final speed = min != null && max != null
        ? (max - min) / 300
        : id == 'scale'
        ? .005
        : _kindOf(row) == _Kind.angle
        ? .5
        : 1.0;
    final shownScale = percent ? 100.0 : 1.0;
    return EditorLamp(
      state: keyLampOf(row),
      child: SizedBox(
        width: slotWidth,
        child: EditorNumericField(
          key: ValueKey('test:${layer['id']}:$id:$axis'),
          value: value.toDouble() * shownScale,
          idleFocus: _nodes.putIfAbsent(
            '$id:$axis',
            () => FocusNode(debugLabel: 'Test $id $axis'),
          ),
          label: label ?? '${row['label']}',
          min: min == null ? null : min * shownScale,
          max: max == null ? null : max * shownScale,
          speed: speed * shownScale,
          fill: fill,
          unit: unit ?? '',
          decimals: percent ? 0 : 2,
          enabled: _canEdit(layer),
          onPreview: (n) => _write(
            layer,
            row,
            _withAxis(row['value'], axis, n / shownScale),
            preview: true,
          ),
          onCommit: (n) => _write(
            layer,
            row,
            _withAxis(row['value'], axis, n / shownScale),
            preview: false,
          ),
          onFinish: () => _finish(false),
          onCancel: () => _finish(true),
        ),
      ),
    );
  }

  Widget _glyph(IconData icon, String tip) => Tooltip(
    message: tip,
    child: SizedBox(
      width: EditorMetrics.s18,
      child: Icon(icon, size: EditorMetrics.s14, color: EditorTheme.muted),
    ),
  );

  Widget _word(String s) => SizedBox(
    width: EditorMetrics.s48,
    child: Text(
      s,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: const TextStyle(fontSize: EditorMetrics.font),
    ),
  );

  Widget _gap() => const SizedBox(width: EditorMetrics.s4);

  /// One column of the grid every row shares: glyph, word, three wells, and
  /// a fixed tail for the row's own extra (dial, link). Empty slots keep
  /// their width so the columns never move.
  /// Well width for the current panel width: the glyph, word and tail
  /// columns are fixed, the three well columns share what is left.
  double _wellWidth = EditorMetrics.field;
  void _fit(double panelWidth) {
    final free =
        panelWidth -
        EditorMetrics.s12 * 2 -
        EditorMetrics.s18 -
        EditorMetrics.s48 -
        EditorMetrics.s4 * 3 -
        EditorMetrics.s22;
    _wellWidth = (free / 3).clamp(EditorMetrics.s36, EditorMetrics.s70);
  }

  Widget _slot([Widget? child, bool center = false]) => SizedBox(
    width: _wellWidth,
    height: EditorMetrics.s22,
    child: child == null
        ? null
        : center
        ? Center(child: child)
        : child,
  );
  Widget _tail([Widget? child]) => SizedBox(
    width: EditorMetrics.s22,
    height: EditorMetrics.s22,
    child: child == null ? null : Center(child: child),
  );

  Widget _line(List<Widget> children) => Padding(
    padding: const EdgeInsets.only(bottom: EditorMetrics.s4),
    child: Row(children: children),
  );

  // ---- Transform ---------------------------------------------------------

  List<Widget> _transform(Map<String, dynamic> layer) {
    final position = _property(layer, 'position');
    final z = _property(layer, 'position.z');
    final scale = _property(layer, 'scale');
    final rotation = _property(layer, 'rotation');
    final rx = _property(layer, 'rotation.x');
    final ry = _property(layer, 'rotation.y');
    final opacity = _property(layer, 'opacity');
    final sv = scale?['value'];
    final scaleEven =
        sv is List && sv.length >= 2 && (sv[0] as num) == (sv[1] as num);
    final anchor = layer['anchorFraction'];
    return [
      if (position != null)
        _line([
          _glyph(Icons.open_with, 'Position'),
          _word('Position'),
          _slot(_well(layer, position, 0, label: 'X')),
          _gap(),
          _slot(_well(layer, position, 1, label: 'Y')),
          _gap(),
          _slot(z == null ? null : _well(layer, z, 0, label: 'Z')),
          _gap(),
          _tail(),
        ]),
      if (scale != null)
        _line([
          _glyph(Icons.aspect_ratio, 'Scale'),
          _word('Scale'),
          _slot(
            _well(
              layer,
              scale,
              0,
              label: _scaleLocked && scaleEven ? 'Scale' : 'X',
            ),
          ),
          _gap(),
          _slot(
            _scaleLocked && scaleEven
                ? null
                : _well(layer, scale, 1, label: 'Y'),
          ),
          _gap(),
          _slot(),
          _gap(),
          _tail(
            EditorSwitch(
              on: _scaleLocked && scaleEven,
              glyph: Icons.link,
              compact: true,
              label: 'Keep the shape: one number scales both axes',
              onChanged: (on) async {
                if (on && sv is List && sv.length >= 2 && !scaleEven) {
                  await _write(layer, scale, [sv[0], sv[0]], preview: false);
                }
                setState(() => _scaleLocked = on);
              },
            ),
          ),
        ]),
      if (rotation != null)
        _line([
          _glyph(Icons.rotate_right, 'Rotation'),
          _word('Rotation'),
          _slot(_well(layer, rotation, 0, label: 'Rotation')),
          _gap(),
          _slot(rx == null ? null : _well(layer, rx, 0, label: 'Tilt X')),
          _gap(),
          _slot(ry == null ? null : _well(layer, ry, 0, label: 'Tilt Y')),
          _gap(),
          _tail(
            EditorDial(
              degrees: (rotation['value'] as num? ?? 0).toDouble(),
              enabled: _canEdit(layer),
              onBegin: () {},
              onPreview: (d) => _write(layer, rotation, d, preview: true),
              onFinish: () => _finish(false),
              onCancel: () => _finish(true),
            ),
          ),
        ]),
      if (opacity != null)
        _line([
          _glyph(Icons.opacity, 'Opacity'),
          _word('Opacity'),
          SizedBox(
            width: _wellWidth * 2 + EditorMetrics.s4,
            height: EditorMetrics.s22,
            child: _well(
              layer,
              opacity,
              0,
              label: 'Opacity',
              fill: true,
              width: _wellWidth * 2 + EditorMetrics.s4,
            ),
          ),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      _line([
        _glyph(Icons.center_focus_weak, 'Anchor'),
        _word('Anchor'),
        EditorAnchorGrid(
          fraction: anchor is List
              ? [(anchor[0] as num).toDouble(), (anchor[1] as num).toDouble()]
              : null,
          onPick: panelCan(c, 'anchor') && layer['locked'] != true
              ? (x, y) => c.command('anchor', {
                  'layer': layer['id'],
                  'xFraction': x,
                  'yFraction': y,
                })
              : null,
        ),
      ]),
    ];
  }

  // ---- World -------------------------------------------------------------

  List<Widget> _world(Map<String, dynamic> layer) {
    final can = panelCan(c, 'setAttrs') && layer['locked'] != true;
    final ghost = (layer['ghost'] as num?)?.toInt();
    return [
      _line([
        _glyph(Icons.view_in_ar_outlined, 'Space'),
        _word('Space'),
        for (final p in ['2D', '2.5D', '3D'])
          Padding(
            padding: const EdgeInsets.only(right: EditorMetrics.s2),
            child: Tooltip(
              message: p,
              child: EditorButton(
                p,
                can
                    ? () => c.command('setAttrs', {
                        'layers': c.selectedIds,
                        'patch': {'projection': p},
                      })
                    : null,
                selected: layer['projection'] == p,
              ),
            ),
          ),
      ]),
      _line([
        _glyph(Icons.account_tree_outlined, 'Parent'),
        _word('Parent'),
        Expanded(
          child: EditorChoice<dynamic>(
            value: layer['parent'] ?? -1,
            choices: [
              const MapEntry(-1, 'None'),
              ...c.layers
                  .where((v) => v['id'] != layer['id'])
                  .map((v) => MapEntry(v['id'], '${v['name']}')),
            ],
            onChanged: can
                ? (v) => c.command('setAttrs', {
                    'layers': [layer['id']],
                    'patch': {'parent': v == -1 ? null : v},
                  })
                : null,
          ),
        ),
      ]),
      _line([
        _glyph(Icons.layers_outlined, 'Blend'),
        _word('Blend'),
        Expanded(
          child: EditorButton(
            '${layer['blendMode'] ?? 'Normal'}',
            () => c.focusEditing(layer['id'] as int, 'blendMode'),
            tooltip: 'Blend mode (opens the Blend desk)',
          ),
        ),
      ]),
      _line([
        const SizedBox(width: EditorMetrics.s18),
        _word(''),
        if (layer['kind'] == 'Image') ...[
          _slot(
            EditorSwitch(
              on: layer['environment'] == true,
              glyph: Icons.wb_sunny_outlined,
              label: 'Environment: this image lights and surrounds the scene',
              onChanged: can
                  ? (on) => c.command('setAttrs', {
                      'layers': [layer['id']],
                      'patch': {'environment': on},
                    })
                  : null,
            ),
          ),
          _gap(),
        ],
        if (layer['ghostable'] == true) ...[
          _slot(
            EditorSwitch(
              on: ghost != null,
              glyph: Icons.blur_on,
              label: 'Ghost: the same layer seen later by a delay',
              onChanged: layer['locked'] != true && panelCan(c, 'ghost')
                  ? (on) => c.command('ghost', {'enabled': on})
                  : null,
            ),
          ),
          _gap(),
        ],
        _slot(
          EditorSwitch(
            on: layer['clipToBelow'] == true,
            glyph: Icons.subdirectory_arrow_right,
            label: 'Clip to the layer below',
            onChanged: layer['locked'] != true && panelCan(c, 'clip')
                ? (_) => c.command('clip', {'layer': layer['id']})
                : null,
          ),
        ),
      ]),
    ];
  }

  // ---- Effects: one sheet per effect, controls from the declaration -------

  /// Effects whose advanced fold is open, by effect id.
  final _advancedOpen = <String>{};

  Widget _effect(Map<String, dynamic> layer, Map<String, dynamic> effect) {
    final params = panelRows(effect['params']);
    // Advanced: declared on the row (ADVANCED in the manifest) or, for a
    // placement, on the grid rows the layout marks.
    final advancedIds = <String>{
      for (final r in params)
        if (r['advanced'] == true) '${r['id']}',
      for (final g in panelRows(panelMap(effect['layout'])['rows']))
        if (g['advanced'] == true) ...[
          if (g['each'] is String) '${g['each']}',
          if (g['random'] is String) '${g['random']}',
        ],
    };
    final controls = <Widget>[];
    final advanced = <Widget>[];
    String? section;
    final byId = {for (final r in params) '${r['id']}': r};
    final folded = <String>{};
    for (final row in params) {
      final id = '${row['id']}';
      if (folded.contains(id)) continue;
      final into = advancedIds.contains(id) ? advanced : controls;
      final here = row['section'] as String?;
      if (here != null && here != section && !advancedIds.contains(id)) {
        controls.add(_SectionLabel(here));
      }
      section = here;
      // A pair the shader adds to a coordinate together (group), or declared
      // as `name_x` + `name_y`, is one point: a pad and two wells.
      final group = row['group'];
      final partner = group is String && group == id
          ? params
                .where((r) => r['group'] == group && r['id'] != id)
                .firstOrNull
          : null;
      final yId = partner != null
          ? '${partner['id']}'
          : id.endsWith('_x')
          ? '${id.substring(0, id.length - 2)}_y'
          : null;
      final y = yId == null ? null : byId[yId];
      if (y != null && row['value'] is num && y['value'] is num) {
        folded.add(yId!);
        into.add(_pointControl(layer, row, y));
        continue;
      }
      into.add(_control(layer, row));
    }
    final key = '${effect['id']}';
    final open = _advancedOpen.contains(key);
    return EditorCard(
      title: '${effect['name']}',
      glyph: Icons.auto_fix_high_outlined,
      trailing: Tooltip(
        message: 'Remove effect',
        child: GestureDetector(
          onTap: panelCan(c, 'removeEffect')
              ? () => c.command('removeEffect', {
                  'layer': layer['id'],
                  'effect': effect['id'],
                })
              : null,
          child: const Icon(
            Icons.close,
            size: EditorMetrics.s12,
            color: EditorTheme.muted,
          ),
        ),
      ),
      children: [
        _cells(controls),
        if (advanced.isNotEmpty) ...[
          const SizedBox(height: EditorMetrics.s4),
          _Fold(
            open: open,
            onTap: () => setState(() {
              open ? _advancedOpen.remove(key) : _advancedOpen.add(key);
            }),
          ),
          if (open) ...[
            const SizedBox(height: EditorMetrics.s4),
            _cells(advanced),
          ],
        ],
      ],
    );
  }

  /// Controls in two equal columns; a section label spans both.
  Widget _cells(List<Widget> controls) => LayoutBuilder(
    builder: (context, box) {
      final cell = (box.maxWidth - EditorMetrics.s6) / 2;
      return Wrap(
        spacing: EditorMetrics.s6,
        runSpacing: EditorMetrics.s4,
        children: [
          for (final w in controls)
            w is _SectionLabel
                ? SizedBox(width: box.maxWidth, child: w)
                : SizedBox(width: cell, child: w),
        ],
      );
    },
  );

  /// A pair of params as one point: drag the dot, or type either number.
  Widget _pointControl(
    Map<String, dynamic> layer,
    Map<String, dynamic> xRow,
    Map<String, dynamic> yRow,
  ) {
    final stem = '${xRow['label'] ?? xRow['id']}';
    final label = stem.endsWith(' X')
        ? stem.substring(0, stem.length - 2)
        : stem;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        _cellLabel(label, Icons.open_with),
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            EditorPad(
              x: (xRow['value'] as num).toDouble(),
              y: (yRow['value'] as num).toDouble(),
              enabled: _canEdit(layer),
              onBegin: () {},
              onPreview: (x, y) => _writeMany(layer, {
                '${xRow['id']}': x,
                '${yRow['id']}': y,
              }, preview: true),
              onFinish: () => _finish(false),
              onCancel: () => _finish(true),
            ),
            _gap(),
            Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                _well(layer, xRow, 0, label: '$label X'),
                const SizedBox(height: EditorMetrics.s4),
                _well(layer, yRow, 0, label: '$label Y'),
              ],
            ),
          ],
        ),
      ],
    );
  }

  Widget _cellLabel(String label, [IconData? glyph]) => Padding(
    padding: const EdgeInsets.only(bottom: EditorMetrics.s2),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: EditorMetrics.s14,
          child: glyph == null
              ? null
              : Icon(glyph, size: EditorMetrics.s12, color: EditorTheme.muted),
        ),
        const SizedBox(width: EditorMetrics.s2),
        Text(
          label,
          style: const TextStyle(
            fontSize: EditorMetrics.dense,
            color: EditorTheme.muted,
          ),
        ),
      ],
    ),
  );

  /// A labelled control for one declared param: the word above, the control
  /// under it, sized by its kind.
  Widget _control(Map<String, dynamic> layer, Map<String, dynamic> row) {
    // A seed is a number to roll, whatever range it declares.
    final kind = '${row['id']}'.endsWith('.seed') ? _Kind.scalar : _kindOf(row);
    final label = '${row['label'] ?? row['id']}';
    Widget body;
    switch (kind) {
      case _Kind.choice:
        final choices = row['choices'];
        body = SizedBox(
          width: EditorMetrics.s96,
          child: EditorChoice<dynamic>(
            value: (row['value'] as num?)?.round(),
            choices: [
              if (choices is List)
                for (var i = 0; i < choices.length; i++)
                  MapEntry(i, '${choices[i]}'),
            ],
            onChanged: _canEdit(layer)
                ? (v) => c.command('setProperty', {
                    'layer': layer['id'],
                    'property': row['id'],
                    'value': v,
                  })
                : null,
          ),
        );
      case _Kind.vec2:
        body = Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _well(layer, row, 0, label: '$label X'),
            _gap(),
            _well(layer, row, 1, label: '$label Y'),
          ],
        );
      case _Kind.bounded:
        body = _well(layer, row, 0, fill: true, width: EditorMetrics.s96);
      case _Kind.angle:
        body = Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            EditorDial(
              degrees: (row['value'] as num? ?? 0).toDouble(),
              enabled: _canEdit(layer),
              onBegin: () {},
              onPreview: (d) => _write(layer, row, d, preview: true),
              onFinish: () => _finish(false),
              onCancel: () => _finish(true),
            ),
            _gap(),
            _well(layer, row, 0),
          ],
        );
      case _Kind.color:
      case _Kind.text:
      case _Kind.scale:
      case _Kind.scalar:
        final seed = '${row['id']}'.endsWith('.seed');
        body = Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _well(layer, row, 0),
            if (seed) ...[
              _gap(),
              Tooltip(
                message: 'Roll a new seed',
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: _canEdit(layer)
                      ? () {
                          final max = (row['max'] as num?)?.toDouble() ?? 9999;
                          final n =
                              (DateTime.now().microsecondsSinceEpoch %
                                      max.round().clamp(1, 1 << 30))
                                  .toDouble();
                          _write(layer, row, n, preview: false);
                        }
                      : null,
                  child: const Icon(
                    Icons.casino_outlined,
                    size: EditorMetrics.s16,
                    color: EditorTheme.muted,
                  ),
                ),
              ),
            ],
          ],
        );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [_cellLabel(label, _glyphOf(row)), body],
    );
  }

  // ---- Identity ----------------------------------------------------------

  Widget _identity(Map<String, dynamic> layer) => Container(
    height: EditorMetrics.bar,
    padding: const EdgeInsets.only(right: EditorMetrics.s6),
    decoration: BoxDecoration(
      color: EditorTheme.panel,
      border: Border(
        left: BorderSide(
          color: EditorTheme.layerColor(layer['id']),
          width: EditorMetrics.s3,
        ),
        bottom: const BorderSide(color: EditorTheme.line),
      ),
    ),
    child: Row(
      children: [
        const SizedBox(width: EditorMetrics.s6),
        Icon(
          switch ('${layer['kind']}') {
            'Text' => Icons.text_fields,
            'Shape' => Icons.pentagon_outlined,
            'Camera' => Icons.videocam_outlined,
            'Video' => Icons.movie_outlined,
            'Audio' => Icons.graphic_eq,
            'Group' => Icons.folder_outlined,
            _ => Icons.image_outlined,
          },
          size: EditorMetrics.s14,
          color: EditorTheme.kindColor('${layer['kind']}'),
        ),
        const SizedBox(width: EditorMetrics.s6),
        Expanded(
          child: Tooltip(
            message: '${layer['name']}',
            child: Text(
              '${layer['name']}',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                fontSize: EditorMetrics.font,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ),
        EditorSwitch(
          on: c.document.value['animate'] == true,
          glyph: Icons.diamond_outlined,
          label: 'Animate: values you touch become keys at this frame',
          onChanged: panelCan(c, 'animate')
              ? (on) => c.command('animate', {'enabled': on})
              : null,
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: Listenable.merge([c.document, c.frame]),
    builder: (context, _) {
      final layer = _active;
      if (layer == null) {
        return const ColoredBox(
          color: EditorTheme.app,
          child: Center(
            child: Text(
              'Select a layer',
              style: TextStyle(
                fontSize: EditorMetrics.font,
                color: EditorTheme.muted,
              ),
            ),
          ),
        );
      }
      final effects = panelRows(layer['effects']);
      return LayoutBuilder(
        builder: (context, box) {
          _fit(box.maxWidth);
          return ColoredBox(
            color: EditorTheme.app,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _identity(layer),
                Expanded(
                  child: ListView(
                    padding: const EdgeInsets.only(bottom: EditorMetrics.s6),
                    children: [
                      EditorCard(
                        title: 'Transform',
                        glyph: Icons.open_with,
                        children: _transform(layer),
                      ),
                      if (layer['kind'] != 'Camera')
                        EditorCard(
                          title: 'World',
                          glyph: Icons.public,
                          children: _world(layer),
                        ),
                      for (final effect in effects) _effect(layer, effect),
                    ],
                  ),
                ),
              ],
            ),
          );
        },
      );
    },
  );
}

class _SectionLabel extends StatelessWidget {
  const _SectionLabel(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(
      top: EditorMetrics.s4,
      bottom: EditorMetrics.s2,
    ),
    child: Text(
      text,
      style: const TextStyle(
        fontSize: EditorMetrics.dense,
        color: EditorTheme.muted,
      ),
    ),
  );
}

/// The fold of the seldom-used controls: a chevron and the word, one line.
class _Fold extends StatelessWidget {
  const _Fold({required this.open, required this.onTap});
  final bool open;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => Tooltip(
    message: open ? 'Hide the advanced controls' : 'Show the advanced controls',
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Row(
        children: [
          Icon(
            open ? Icons.expand_more : Icons.chevron_right,
            size: EditorMetrics.s14,
            color: EditorTheme.muted,
          ),
          const SizedBox(width: EditorMetrics.s2),
          const Text(
            'Advanced',
            style: TextStyle(
              fontSize: EditorMetrics.dense,
              color: EditorTheme.muted,
            ),
          ),
          const SizedBox(width: EditorMetrics.s6),
          const Expanded(child: Divider(height: 1)),
        ],
      ),
    ),
  );
}
