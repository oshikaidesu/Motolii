import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../foundation/metrics.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import 'inspector.dart' show EditorColorRow;
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

/// One hue per family, so a glance sorts the numbers before a word is read.
Color? _tintOf(Map<String, dynamic> row) => switch (_characterOf(row)) {
  _Character.place ||
  _Character.size ||
  _Character.ratio ||
  _Character.soft => EditorTheme.spatial,
  _Character.amount ||
  _Character.opacity ||
  _Character.level => EditorTheme.amount,
  _Character.time || _Character.delay => EditorTheme.time,
  _Character.count || _Character.detail => EditorTheme.count,
  _Character.seed => EditorTheme.seed,
  _Character.angle => EditorTheme.angle,
  _ => null,
};

/// The track's grammar by family: a threshold lights its far side, a count
/// shows whole steps, a length sits on a ruler, an amount fills.
TrackStyle _trackOf(Map<String, dynamic> row) => switch (_characterOf(row)) {
  _Character.level => TrackStyle.level,
  _Character.count ||
  _Character.detail when _span(row) <= 12 => TrackStyle.steps,
  _Character.size || _Character.soft => TrackStyle.ruler,
  _ => TrackStyle.fill,
};

double _span(Map<String, dynamic> row) =>
    ((row['max'] as num?)?.toDouble() ?? 0) -
    ((row['min'] as num?)?.toDouble() ?? 0);

/// A range that is a real reach (opacity 0..1, octaves 1..8), not a guard
/// (amount 0..100000): only a real reach earns a track.
bool _tight(Map<String, dynamic> row) {
  final v = (row['value'] as num?)?.toDouble() ?? 0;
  final d = (row['default'] as num?)?.toDouble() ?? v;
  return _span(row) <= 20 * math.max(d.abs(), 1);
}

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
  /// Every selected, unlocked layer; the shown layer alone when it is not
  /// part of the selection. A drag moves them all by the same amount, a
  /// typed number sets them all.
  List<Map<String, dynamic>> _targets(Map<String, dynamic> layer) {
    final live = c
        .liveLayers()
        .where((v) => c.selectedIds.contains(v['id']) && v['locked'] != true)
        .toList();
    return live.any((v) => v['id'] == layer['id']) ? live : [layer];
  }

  bool get _multiple => c.selectedIds.length > 1;

  /// One value into a property of every target: during a drag the change is
  /// relative (each layer keeps its own offset), a typed number is absolute.
  Future<void> _write(
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
    dynamic next, {
    required bool preview,
  }) async {
    final id = '${row['id']}';
    final edits = <Map<String, dynamic>>[];
    for (final target in _targets(layer)) {
      final own = _property(target, id);
      if (own == null) continue;
      dynamic value = next;
      if (preview && target['id'] != layer['id']) {
        final base = row['value'], mine = own['value'];
        if (next is num && base is num && mine is num) {
          value = mine + (next - base);
        } else if (next is List && base is List && mine is List) {
          value = [
            for (var i = 0; i < next.length; i++)
              i < base.length && i < mine.length && next[i] is num
                  ? (mine[i] as num) + (next[i] as num) - (base[i] as num)
                  : next[i],
          ];
        }
      }
      edits.add({'layer': target['id'], 'property': id, 'value': value});
    }
    if (edits.isEmpty) return;
    await c.command('previewProperties', {'edits': edits});
    if (!preview) await c.command('commitPreview');
  }

  Future<void> _writeMany(
    Map<String, dynamic> layer,
    Map<String, double> values, {
    required bool preview,
  }) async {
    final edits = [
      for (final target in _targets(layer))
        for (final e in values.entries)
          if (_property(target, e.key) != null)
            {'layer': target['id'], 'property': e.key, 'value': e.value},
    ];
    if (edits.isEmpty) return;
    await c.command('previewProperties', {'edits': edits});
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
    final mixed = _targets(layer).any((t) {
      final other = _property(t, id)?['value'];
      final n = other is List && axis < other.length ? other[axis] : other;
      return n is num && n != value;
    });
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
    return Builder(
      builder: (context) => GestureDetector(
        onSecondaryTapDown: (d) =>
            _cellMenu(context, d.globalPosition, layer, row),
        child: EditorLamp(
          state: keyLampOf(row),
          onTap: panelCan(c, 'toggleKey') && _canEdit(layer)
              ? () => c.command('toggleKey', {
                  'layer': layer['id'],
                  'property': row['id'],
                })
              : null,
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
              mixed: mixed,
              fill: fill,
              unit: unit ?? '',
              // A narrow well keeps its digits whole rather than clipping them.
              decimals: percent || slotWidth < EditorMetrics.field ? 0 : 2,
              defaultValue: _restOf(row, axis, shownScale),
              tint: _tintOf(row),
              track: _trackOf(row),
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
        ),
      ),
    );
  }

  /// The value a row rests at: declared on the row, or by meaning for the
  /// layer's own transform (scale 1, rotation 0, opacity 1).
  double? _restOf(Map<String, dynamic> row, int axis, double scale) {
    final id = '${row['id']}';
    final d = row['default'];
    if (d is num) return d.toDouble() * scale;
    if (d is List && axis < d.length && d[axis] is num) {
      return (d[axis] as num).toDouble() * scale;
    }
    if (id == 'scale') return 1.0;
    if (id == 'opacity') return scale;
    if (id.startsWith('rotation')) return 0.0;
    return null;
  }

  /// Every bounded number of an effect nudged by chance — within a fifth of
  /// its reach around where it is, so a throw stays playable — seeds fully,
  /// counts whole, as one edit (the Ableton dice).
  Future<void> _roll(
    Map<String, dynamic> layer,
    Map<String, dynamic> effect,
  ) async {
    final rnd = math.Random();
    final values = <String, double>{};
    for (final row in panelRows(effect['params'])) {
      final id = '${row['id']}';
      if (row['value'] is! num || row['choices'] is List) continue;
      final min = (row['min'] as num?)?.toDouble(),
          max = (row['max'] as num?)?.toDouble();
      if (id.endsWith('.seed')) {
        values[id] = rnd.nextInt(10000).toDouble();
      } else if (min != null && max != null) {
        final reach = (max - min) * .2;
        var next =
            ((row['value'] as num).toDouble() +
                    (rnd.nextDouble() * 2 - 1) * reach)
                .clamp(min, max)
                .toDouble();
        if (_characterOf(row) == _Character.count) next = next.roundToDouble();
        values[id] = next;
      }
    }
    if (values.isNotEmpty) await _writeMany(layer, values, preview: false);
  }

  /// Every number of an effect back to where it rests, as one edit.
  Future<void> _rest(
    Map<String, dynamic> layer,
    Map<String, dynamic> effect,
  ) async {
    final values = <String, double>{};
    for (final row in panelRows(effect['params'])) {
      final d = row['default'];
      if (row['value'] is num && d is num && row['choices'] is! List) {
        values['${row['id']}'] = d.toDouble();
      }
    }
    if (values.isNotEmpty) await _writeMany(layer, values, preview: false);
  }

  Widget _headGlyph(IconData icon, String tip, VoidCallback? onTap) => Tooltip(
    message: tip,
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.only(left: EditorMetrics.s6),
        child: Icon(
          icon,
          size: EditorMetrics.s12,
          color: onTap == null ? EditorTheme.disabledInk : EditorTheme.muted,
        ),
      ),
    ),
  );

  /// Two wells side by side in one cell, and the wells beside a pad.
  static const _half = (EditorMetrics.cell - EditorMetrics.s4) / 2;
  static const _beside =
      EditorMetrics.cell - EditorMetrics.s60 - EditorMetrics.s4;

  Widget _glyph(IconData icon, String tip) => Tooltip(
    message: tip,
    child: SizedBox(
      width: EditorMetrics.s18,
      child: Icon(icon, size: EditorMetrics.s14, color: EditorTheme.muted),
    ),
  );

  Widget _word(String s) => SizedBox(
    width: _wordWidth,
    child: _wordWidth == 0
        ? null
        : Text(
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

  /// The word column; 0 when the panel is too narrow for three wells beside
  /// it — the glyph then carries the meaning and the tooltip keeps the word.
  double _wordWidth = EditorMetrics.s48;
  void _fit(double panelWidth) {
    final fixed =
        EditorMetrics.s12 * 2 +
        EditorMetrics.s18 +
        EditorMetrics.s4 * 3 +
        EditorMetrics.s22;
    var free = panelWidth - fixed - EditorMetrics.s48;
    _wordWidth = EditorMetrics.s48;
    if (free / 3 < EditorMetrics.s44) {
      _wordWidth = 0;
      free = panelWidth - fixed;
    }
    _wellWidth = (free / 3).clamp(EditorMetrics.s32, EditorMetrics.s70);
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

  /// A camera layer authors Center, Zoom and Roll instead of a transform.
  List<Widget> _camera(Map<String, dynamic> layer) {
    final center = _property(layer, 'camera.center');
    final zoom = _property(layer, 'camera.zoom');
    final roll = _property(layer, 'camera.roll');
    return [
      if (center != null)
        _line([
          _glyph(Icons.center_focus_strong, 'Center'),
          _word('Center'),
          _slot(_well(layer, center, 0, label: 'X')),
          _gap(),
          _slot(_well(layer, center, 1, label: 'Y')),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      if (zoom != null)
        _line([
          _glyph(Icons.zoom_in, 'Zoom'),
          _word('Zoom'),
          _slot(_well(layer, zoom, 0, label: 'Zoom')),
          _gap(),
          _slot(),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      if (roll != null)
        _line([
          _glyph(Icons.rotate_right, 'Roll'),
          _word('Roll'),
          _slot(_well(layer, roll, 0, label: 'Roll')),
          _gap(),
          _slot(),
          _gap(),
          _slot(),
          _gap(),
          _tail(
            EditorDial(
              degrees: (roll['value'] as num? ?? 0).toDouble(),
              enabled: _canEdit(layer),
              tint: EditorTheme.angle,
              onBegin: () {},
              onPreview: (d) => _write(layer, roll, d, preview: true),
              onFinish: () => _finish(false),
              onCancel: () => _finish(true),
            ),
          ),
        ]),
    ];
  }

  /// Property ids the Transform card already shows.
  static const _transformIds = {
    'position',
    'position.z',
    'scale',
    'rotation',
    'rotation.x',
    'rotation.y',
    'opacity',
    'anchor',
    'camera.center',
    'camera.zoom',
    'camera.roll',
    'content',
  };

  bool _isTextProperty(Map<String, dynamic> r) =>
      '${r['id']}'.startsWith('text') ||
      const ['Size', 'Line height', 'Tracking'].contains(r['label']);

  /// A text layer: what it says, then how it is set.
  List<Widget> _text(Map<String, dynamic> layer, Map<String, dynamic> text) {
    final content = _property(layer, 'content');
    final rows = panelRows(layer['properties']).where(_isTextProperty).toList();
    return [
      EditorLamp(
        state: keyLampOf(
          content ??
              {
                'keys': panelRows(layer['contentKeys']),
                'keyedNow': panelRows(layer['contentKeys'])
                    .any((k) => k['frame'] == c.frame.value),
              },
        ),
        child: EditorDraftField(
          key: ValueKey('test:content:${layer['id']}'),
          value: '${text['content'] ?? ''}',
          label: 'Content',
          multiline: true,
          enabled: panelCan(c, 'setText'),
          onCommit: (v) =>
              c.command('setText', {'layer': layer['id'], 'content': v}),
        ),
      ),
      if (rows.isNotEmpty) ...[
        const SizedBox(height: EditorMetrics.s6),
        _cells([for (final r in rows) _Cell(_control(layer, r))]),
      ],
    ];
  }

  /// The layer's colours: a swatch that opens the Colors desk, and the hex.
  List<Widget> _colors(Map<String, dynamic> layer) {
    final colors = panelRows(layer['colors']);
    return [
      if (layer['kind'] == 'Shape')
        _line([
          _glyph(Icons.format_color_fill, 'Fill'),
          _word('Fill'),
          for (final gradient in [false, true])
            Padding(
              padding: const EdgeInsets.only(right: EditorMetrics.s2),
              child: EditorButton(
                gradient ? 'Gradient' : 'Solid',
                panelCan(c, 'setFillMode') && colors.isNotEmpty
                    ? () => c.command('setFillMode', {
                        'layer': layer['id'],
                        'slot': colors.first['slot'],
                        'gradient': gradient,
                      })
                    : null,
                selected: (colors.length > 1) == gradient,
              ),
            ),
        ]),
      for (final color in colors)
        Padding(
          padding: const EdgeInsets.only(bottom: EditorMetrics.s4),
          child: EditorColorRow(controller: c, layer: layer, color: color),
        ),
    ];
  }

  /// The old matte, still shown while a layer carries one and does not clip.
  List<Widget> _matte(Map<String, dynamic> layer, Map<String, dynamic> matte) {
    final others = c.layers.where((v) => v['id'] != layer['id']);
    return [
      _line([
        _glyph(Icons.layers_outlined, 'Source'),
        _word('Source'),
        Expanded(
          child: EditorChoice<dynamic>(
            value: matte['source'],
            choices: [
              for (final v in others) MapEntry(v['id'], '${v['name']}'),
            ],
            onChanged: panelCan(c, 'setMatte')
                ? (v) => c.command('setMatte', {
                    'layer': layer['id'],
                    'source': v,
                    'mode': matte['mode'],
                  })
                : null,
          ),
        ),
      ]),
      _line([
        _glyph(Icons.contrast, 'Mode'),
        _word('Mode'),
        Expanded(
          child: EditorChoice<dynamic>(
            value: matte['mode'],
            choices: const [
              MapEntry('Alpha', 'Alpha'),
              MapEntry('AlphaInverted', 'Alpha inverted'),
              MapEntry('Luma', 'Luma'),
              MapEntry('LumaInverted', 'Luma inverted'),
            ],
            onChanged: panelCan(c, 'setMatte')
                ? (v) => c.command('setMatte', {
                    'layer': layer['id'],
                    'source': matte['source'],
                    'mode': v,
                  })
                : null,
          ),
        ),
      ]),
    ];
  }

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
          onHover: (x, y, inside) =>
              c.anchorPreview.value = inside ? [x, y] : null,
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
        if (layer['kind'] == 'Group') ...[
          _slot(
            EditorSwitch(
              on: layer['frozen'] == true,
              glyph: Icons.ac_unit,
              label: 'Freeze: the group keeps its picture as it is',
              onChanged: panelCan(c, 'freeze')
                  ? (on) => c.command('freeze', {
                      'layer': layer['id'],
                      'enabled': on,
                    })
                  : null,
            ),
            true,
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

  Widget _effect(
    Map<String, dynamic> layer,
    Map<String, dynamic> effect, {
    int index = 0,
    int count = 1,
  }) {
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
    // Heroes: declared, else the first four that are not advanced (the OP-1
    // rule: four knobs in front, the rest behind shift).
    final plain = params
        .where((r) => !advancedIds.contains('${r['id']}'))
        .toList();
    final declaredHeroes = plain.where((r) => r['hero'] == true).toList();
    final heroIds = <String>{
      for (final r
          in declaredHeroes.isNotEmpty
              ? declaredHeroes
              : plain.length > 4
              ? plain.take(4)
              : const <Map<String, dynamic>>[])
        '${r['id']}',
    };
    final heroes = <_Cell>[];
    final controls = <_Cell>[];
    final advanced = <_Cell>[];
    String? section;
    final byId = {for (final r in params) '${r['id']}': r};
    final folded = <String>{};
    for (final row in params) {
      final id = '${row['id']}';
      if (folded.contains(id)) continue;
      final hero = heroIds.contains(id);
      final into = advancedIds.contains(id)
          ? advanced
          : hero
          ? heroes
          : controls;
      final here = row['section'] as String?;
      if (here != null && here != section && !advancedIds.contains(id)) {
        controls.add(_Cell(_SectionLabel(here), wide: true));
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
        into.add(_Cell(_pointControl(layer, row, y, hero: hero), tall: true));
        continue;
      }
      into.add(_Cell(_control(layer, row, hero: hero)));
    }
    final key = '${effect['id']}';
    final open = _advancedOpen.contains(key);
    return EditorCard(
      title: '${effect['name']}',
      glyph: Icons.auto_fix_high_outlined,
      dim: effect['enabled'] == false,
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _headGlyph(
            effect['enabled'] == false
                ? Icons.visibility_off_outlined
                : Icons.visibility_outlined,
            effect['enabled'] == false
                ? 'Off — press to apply'
                : 'Applied — press to bypass',
            panelCan(c, 'enableEffect')
                ? () => c.command('enableEffect', {
                    'layer': layer['id'],
                    'id': effect['id'],
                    'enabled': effect['enabled'] == false,
                  })
                : null,
          ),
          _headGlyph(
            Icons.arrow_upward,
            'Apply earlier',
            panelCan(c, 'moveEffect') && index > 0
                ? () => c.command('moveEffect', {
                    'layer': layer['id'],
                    'id': effect['id'],
                    'to': index - 1,
                  })
                : null,
          ),
          _headGlyph(
            Icons.arrow_downward,
            'Apply later',
            panelCan(c, 'moveEffect') && index < count - 1
                ? () => c.command('moveEffect', {
                    'layer': layer['id'],
                    'id': effect['id'],
                    'to': index + 1,
                  })
                : null,
          ),
          _headGlyph(
            Icons.casino_outlined,
            'Throw every number within its reach',
            _canEdit(layer) ? () => _roll(layer, effect) : null,
          ),
          _headGlyph(
            Icons.restart_alt,
            'Back to where the numbers rest',
            _canEdit(layer) ? () => _rest(layer, effect) : null,
          ),
          if (effect['placement'] == true)
            _headGlyph(
              Icons.unfold_more,
              'Expand copies into layers',
              panelCan(c, 'expandEffect')
                  ? () => c.command('expandEffect', {
                      'layer': layer['id'],
                      'id': effect['id'],
                    })
                  : null,
            ),
          _headGlyph(
            Icons.close,
            'Remove effect',
            panelCan(c, 'removeEffect')
                ? () => c.command('removeEffect', {
                    'layer': layer['id'],
                    'id': effect['id'],
                  })
                : null,
          ),
        ],
      ),
      children: [
        if (heroes.isNotEmpty) ...[
          _cells(heroes),
          if (controls.isNotEmpty) ...[
            const SizedBox(height: EditorMetrics.s6),
            const Divider(height: 1),
            const SizedBox(height: EditorMetrics.s6),
          ],
        ],
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

  /// Controls in two equal columns on fixed rows (a word, then a control),
  /// so every cell sits on the same grid; a section label spans both.
  Widget _cells(List<_Cell> cells) => LayoutBuilder(
    builder: (context, box) {
      // Cells keep one width and pack from the left, so the gap between
      // items never grows; a wider panel only adds columns.
      const gap = EditorMetrics.s6;
      final columns = math.max(
        1,
        ((box.maxWidth + gap) / (EditorMetrics.cell + gap)).floor(),
      );
      // One column takes the whole width; more columns keep the cell's own.
      final cell = columns == 1
          ? box.maxWidth
          : math.min(
              EditorMetrics.cell,
              (box.maxWidth - gap * (columns - 1)) / columns,
            );
      return Wrap(
        spacing: EditorMetrics.s6,
        runSpacing: EditorMetrics.s6,
        children: [
          for (final c in cells)
            SizedBox(
              width: c.wide ? box.maxWidth : cell,
              height: c.wide
                  ? EditorMetrics.s16
                  : c.tall
                  ? EditorMetrics.s76
                  : EditorMetrics.s36,
              child: c.child,
            ),
        ],
      );
    },
  );

  /// A pair of params as one point: drag the dot, or type either number.
  Widget _pointControl(
    Map<String, dynamic> layer,
    Map<String, dynamic> xRow,
    Map<String, dynamic> yRow, {
    bool hero = false,
  }) {
    final stem = '${xRow['label'] ?? xRow['id']}';
    final label = stem.endsWith(' X')
        ? stem.substring(0, stem.length - 2)
        : stem;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        _cellLabel(label, Icons.open_with, hero, EditorTheme.spatial),
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            EditorPad(
              x: (xRow['value'] as num).toDouble(),
              y: (yRow['value'] as num).toDouble(),
              enabled: _canEdit(layer),
              tint: EditorTheme.spatial,
              size: EditorMetrics.s60,
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
                _well(layer, xRow, 0, label: '$label X', width: _beside),
                const SizedBox(height: EditorMetrics.s4),
                _well(layer, yRow, 0, label: '$label Y', width: _beside),
              ],
            ),
          ],
        ),
      ],
    );
  }

  /// The cell's own menu: back to its rest value, and key / unkey.
  Future<void> _cellMenu(
    BuildContext context,
    Offset at,
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
  ) async {
    final rest = row['default'];
    final chosen = await showMenu<String>(
      context: context,
      position: RelativeRect.fromLTRB(at.dx, at.dy, at.dx, at.dy),
      items: [
        EditorMenuItem<String>(
          value: 'rest',
          enabled: rest is num && row['value'] is num && _canEdit(layer),
          child: const Text('Reset'),
        ),
        EditorMenuItem<String>(
          value: 'key',
          enabled: panelCan(c, 'toggleKey') && _canEdit(layer),
          child: Text(
            row['keyedNow'] == true ? 'Remove key' : 'Key this frame',
          ),
        ),
      ],
    );
    if (chosen == 'rest' && rest is num) {
      await _write(layer, row, rest.toDouble(), preview: false);
    } else if (chosen == 'key') {
      await c.command('toggleKey', {
        'layer': layer['id'],
        'property': row['id'],
      });
    }
  }

  Widget _cellLabel(
    String label, [
    IconData? glyph,
    bool hero = false,
    Color? tint,
  ]) => Padding(
    padding: const EdgeInsets.only(bottom: EditorMetrics.s2),
    child: Row(
      children: [
        SizedBox(
          width: EditorMetrics.s14,
          child: glyph == null
              ? null
              : Icon(
                  glyph,
                  size: EditorMetrics.s12,
                  color: tint ?? EditorTheme.muted,
                ),
        ),
        const SizedBox(width: EditorMetrics.s2),
        Expanded(
          child: Text(
            label,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontSize: EditorMetrics.dense,
              color: hero ? EditorTheme.ink : EditorTheme.muted,
            ),
          ),
        ),
      ],
    ),
  );

  /// A labelled control for one declared param: the word above, the control
  /// under it, sized by its kind.
  Widget _control(
    Map<String, dynamic> layer,
    Map<String, dynamic> row, {
    bool hero = false,
  }) {
    // A seed is a number to roll, whatever range it declares.
    final kind = '${row['id']}'.endsWith('.seed') ? _Kind.scalar : _kindOf(row);
    final label = '${row['label'] ?? row['id']}';
    Widget body;
    switch (kind) {
      case _Kind.choice:
        final choices = row['choices'];
        body = SizedBox(
          width: EditorMetrics.cell,
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
            _well(layer, row, 0, label: '$label X', width: _half),
            _gap(),
            _well(layer, row, 1, label: '$label Y', width: _half),
          ],
        );
      case _Kind.bounded:
        body = _well(
          layer,
          row,
          0,
          fill: _tight(row),
          width: EditorMetrics.cell,
        );
      case _Kind.angle:
        body = Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            EditorDial(
              degrees: (row['value'] as num? ?? 0).toDouble(),
              enabled: _canEdit(layer),
              tint: EditorTheme.angle,
              onBegin: () {},
              onPreview: (d) => _write(layer, row, d, preview: true),
              onFinish: () => _finish(false),
              onCancel: () => _finish(true),
            ),
            _gap(),
            _well(
              layer,
              row,
              0,
              width: EditorMetrics.cell - EditorMetrics.s22 - EditorMetrics.s4,
            ),
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
            _well(
              layer,
              row,
              0,
              width: seed
                  ? EditorMetrics.cell - EditorMetrics.s16 - EditorMetrics.s4
                  : EditorMetrics.cell,
            ),
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
    return Builder(
      builder: (context) => GestureDetector(
        onSecondaryTapDown: (d) =>
            _cellMenu(context, d.globalPosition, layer, row),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            _cellLabel(label, _glyphOf(row), hero, _tintOf(row)),
            body,
          ],
        ),
      ),
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
              _multiple ? '${c.selectedIds.length} layers' : '${layer['name']}',
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
      final text = panelMap(layer['text']);
      final matte = panelMap(layer['matte']);
      final rest = panelRows(layer['properties'])
          .where(
            (r) =>
                !_transformIds.contains('${r['id']}') &&
                !_isTextProperty(r) &&
                !'${r['id']}'.startsWith('effect') &&
                (r['value'] is num || r['value'] is List),
          )
          .toList();
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
                      if (layer['kind'] == 'Camera')
                        EditorCard(
                          title: 'Camera',
                          glyph: Icons.videocam_outlined,
                          children: _camera(layer),
                        )
                      else
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
                      if (!_multiple && text.isNotEmpty)
                        EditorCard(
                          title: 'Text',
                          glyph: Icons.text_fields,
                          children: _text(layer, text),
                        ),
                      if (!_multiple && panelRows(layer['colors']).isNotEmpty)
                        EditorCard(
                          title: 'Color',
                          glyph: Icons.palette_outlined,
                          children: _colors(layer),
                        ),
                      if (!_multiple &&
                          matte.isNotEmpty &&
                          layer['clipToBelow'] != true)
                        EditorCard(
                          title: 'Matte',
                          glyph: Icons.contrast,
                          children: _matte(layer, matte),
                        ),
                      if (rest.isNotEmpty)
                        EditorCard(
                          title: 'Properties',
                          glyph: Icons.tune,
                          children: [
                            _cells([
                              for (final r in rest) _Cell(_control(layer, r)),
                            ]),
                          ],
                        ),
                      if (!_multiple)
                        for (var i = 0; i < effects.length; i++)
                          _effect(
                            layer,
                            effects[i],
                            index: i,
                            count: effects.length,
                          ),
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

/// A section inside a card: the same kicker as the card's title, on a rule.
class _SectionLabel extends StatelessWidget {
  const _SectionLabel(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    alignment: Alignment.bottomLeft,
    decoration: const BoxDecoration(
      border: Border(bottom: BorderSide(color: EditorTheme.line)),
    ),
    child: Text(
      text.toUpperCase(),
      style: const TextStyle(
        fontSize: EditorMetrics.micro,
        letterSpacing: 1,
        color: EditorTheme.muted,
      ),
    ),
  );
}

/// One cell of a card's grid: a control, a tall one (a pad), or a wide
/// section label.
class _Cell {
  const _Cell(this.child, {this.tall = false, this.wide = false});
  final Widget child;
  final bool tall, wide;
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
