import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../foundation/metrics.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';
import '../session/read_model.dart';
import 'gradient_inspector.dart';
import 'rich_text_editor.dart';

part 'inspector_property_style.dart';
part 'inspector_color.dart';

/// The Inspector as controls, not rows. Every property row the snapshot sends
/// is turned into one control by [_kindOf]; nothing here is laid out by hand
/// per effect, so a new Vism with declared params gets its sheet for free.
///
/// Routes are the Inspector's own: previewProperties / commitPreview for
/// values, setAttrs / anchor / ghost / clip for the layer, focusEditing for
/// Blend (the Desk owns the picker). Nothing is re-implemented.
class InspectorPanel extends StatefulWidget {
  const InspectorPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> {
  EditorSession get c => widget.controller;
  final _nodes = <String, FocusNode>{};
  bool _scaleLocked = true;
  List<Map<String, dynamic>>? _gestureLayers;
  void _begin() => _gestureLayers = c.liveLayers().toList();
  final _rows = <String, GlobalKey>{};
  final _scroll = ScrollController();

  /// The shown layer and its rows by id, worked out once per status. The
  /// panel asks for the same rows dozens of times while it builds, and
  /// `liveLayers` copies the document every time it is called.
  Map<String, dynamic>? _fromState, _fromRendered, _shown;
  Map<String, Map<String, dynamic>> _byId = const {};
  List<Map<String, dynamic>> _live = const [];

  /// One listenable per row: a status update pushes the rows that moved, so a
  /// number that changes rebuilds its own well and nothing else. The frame of
  /// the panel is rebuilt only when [_stampShape] moves.
  final _pulse = <String, ValueNotifier<Object?>>{};
  Object? _shape;
  int? _shownId;

  static const _watched = [
    'layers',
    'selectedId',
    'selectedIds',
    'animate',
    'capabilities',
    'contentRevision',
    'documentRevision',
  ];

  @override
  void initState() {
    super.initState();
    c.focusProperty.addListener(_reveal);
    c.slice('inspector', _watched).addListener(_absorb);
    c.rendered.addListener(_absorb);
    _shape = _stampShape();
    _shownId = _shown?['id'] as int?;
  }

  /// Take in one status: hand every row that moved to its own listeners, and
  /// rebuild the frame only when the shape of the panel is not what it was.
  void _absorb() {
    _fromState = null;
    _read();
    for (final entry in _pulse.entries) {
      final now = _stampRow(entry.key);
      if (!sameValue(entry.value.value, now)) entry.value.value = now;
    }
    final shape = _stampShape();
    if (sameValue(_shape, shape)) return;
    _shape = shape;
    final id = _shown?['id'] as int?;
    if (id != _shownId) {
      _shownId = id;
      if (_scroll.hasClients) _scroll.jumpTo(0);
    }
    if (mounted) setState(() {});
  }

  void _read() {
    if (identical(c.state, _fromState) &&
        identical(c.rendered.value, _fromRendered))
      return;
    _fromState = c.state;
    _fromRendered = c.rendered.value;
    _live = c.liveLayers();
    final ids = c.selectedIds;
    Map<String, dynamic>? shown;
    if (ids.isNotEmpty) {
      for (final layer in _live) {
        if (layer['id'] == ids.last) shown = layer;
      }
    }
    _shown = shown ??= c.activeLayer;
    _byId = shown == null
        ? const {}
        : {
            for (final row in [
              ...panelRows(shown['properties']),
              for (final effect in panelRows(shown['effects']))
                ...panelRows(effect['params']),
            ])
              '${row['id']}': row,
          };
  }

  /// The row a control shows, as of the last status.
  Map<String, dynamic>? _row(String id) {
    _read();
    return _byId[id];
  }

  /// What one control watches: its rows, and the same rows on the other
  /// selected layers (a well says when they disagree).
  Object? _stampRow(String name) {
    _read();
    return [
      for (final id in name.split('+')) ...[
        _byId[id],
        [
          for (final layer in _live)
            if (layer['id'] != _shown?['id'] &&
                c.selectedIds.contains(layer['id']))
              _property(layer, id)?['value'],
        ],
      ],
    ];
  }

  Widget _live2(List<String> ids, Widget Function() body) {
    final name = ids.join('+');
    return _Live(
      pulse: _pulse.putIfAbsent(name, () => ValueNotifier(_stampRow(name))),
      body: body,
    );
  }

  /// What the frame of the panel is made of. The numbers are left out on
  /// purpose: each row watches its own, so a value that moves rebuilds one
  /// row and leaves the cards, the words and the glyphs where they are.
  Object? _stampShape() {
    _read();
    final layer = _shown;
    if (layer == null) return null;
    return [
      c.selectedIds,
      c.state['animate'],
      c.state['capabilities'],
      for (final key in const [
        'id',
        'kind',
        'name',
        'locked',
        'projection',
        'parent',
        'blendMode',
        'ghostable',
        'environment',
        'frozen',
        'clipToBelow',
        'anchorFraction',
        'colors',
        'fill',
        'matte',
        'text',
      ])
        layer[key],
      layer['ghost'] == null,
      [
        for (final other in _live) [other['id'], other['name']],
      ],
      _scaleEven(),
      [for (final row in panelRows(layer['properties'])) _stampDeclared(row)],
      [
        for (final effect in panelRows(layer['effects']))
          [
            effect['id'],
            effect['name'],
            effect['enabled'],
            effect['placement'],
            effect['layout'],
            [
              for (final row in panelRows(effect['params']))
                _stampDeclared(row),
            ],
          ],
      ],
    ];
  }

  /// A row as the panel's shape reads it: what it is, never what it says.
  static List<Object?> _stampDeclared(Map<String, dynamic> row) => [
    row['id'],
    row['label'],
    row['kind'],
    row['subtype'],
    row['unit'],
    row['min'],
    row['max'],
    row['default'],
    row['choices'],
    row['hero'],
    row['advanced'],
    row['section'],
    row['group'],
    switch (row['value']) {
      final List v => 'list ${v.length}',
      num() => 'number',
      String() => 'text',
      _ => 'other',
    },
  ];

  /// Whether the scale line shows one well or two.
  bool _scaleEven() {
    final v = _byId['scale']?['value'];
    return v is List && v.length >= 2 && (v[0] as num) == (v[1] as num);
  }

  @override
  void didUpdateWidget(covariant InspectorPanel oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != c) {
      oldWidget.controller.focusProperty.removeListener(_reveal);
      oldWidget.controller.slice('inspector', _watched).removeListener(_absorb);
      oldWidget.controller.rendered.removeListener(_absorb);
      c.focusProperty.addListener(_reveal);
      c.slice('inspector', _watched).addListener(_absorb);
      c.rendered.addListener(_absorb);
      _fromState = null;
      _absorb();
    }
  }

  void _reveal() {
    final id = c.focusProperty.value;
    if (id == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final target = _rows[id]?.currentContext;
      if (target != null) Scrollable.ensureVisible(target);
      _nodes['$id:0']?.requestFocus();
    });
    WidgetsBinding.instance.ensureVisualUpdate();
  }

  dynamic _axisValue(Map<String, dynamic> row, int axis, double value) {
    row = _gestureLayers == null
        ? row
        : _property(
                _gestureLayers!.firstWhere(
                  (v) => v['id'] == _active?['id'],
                  orElse: () => <String, dynamic>{},
                ),
                '${row['id']}',
              ) ??
              row;
    final current = row['value'];
    if (row['id'] == 'scale' && _scaleLocked && current is List) {
      final base = (current[axis] as num).toDouble();
      return [
        for (final n in current) base == 0 ? value : (n as num) * value / base,
      ];
    }
    return _withAxis(current, axis, value);
  }

  Map<String, dynamic>? get _active {
    _read();
    return _shown;
  }

  @override
  void dispose() {
    c.focusProperty.removeListener(_reveal);
    c.slice('inspector', _watched).removeListener(_absorb);
    c.rendered.removeListener(_absorb);
    for (final n in _nodes.values) {
      n.dispose();
    }
    for (final p in _pulse.values) {
      p.dispose();
    }
    _scroll.dispose();
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
    final live = (_gestureLayers ?? c.liveLayers())
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
    if (preview && _gestureLayers != null) {
      final base = _gestureLayers!
          .where((v) => v['id'] == layer['id'])
          .firstOrNull;
      if (base != null) row = _property(base, id) ?? row;
    }
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
            {
              'layer': target['id'],
              'property': e.key,
              'value': preview && target['id'] != layer['id']
                  ? (_property(target, e.key)!['value'] as num) +
                        e.value -
                        (_property(
                              (_gestureLayers ?? [layer]).firstWhere(
                                (v) => v['id'] == layer['id'],
                              ),
                              e.key,
                            )!['value']
                            as num)
                  : e.value,
            },
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

  Future<void> _finish(bool cancel) async {
    try {
      await c.command(cancel ? 'cancelPreview' : 'commitPreview');
    } finally {
      _gestureLayers = null;
    }
  }

  /// The numeric well: one axis of a row, watching that row alone.
  Widget _well(
    Map<String, dynamic> layer,
    String id,
    int axis, {
    String? label,
    double? width,
    bool fill = false,
  }) {
    final slot = width ?? _wellWidth;
    return _live2([id], () {
      final row = _row(id);
      return row == null
          ? SizedBox(width: slot)
          : _wellBody(layer, row, axis, label: label, width: slot, fill: fill);
    });
  }

  Widget _wellBody(
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
    int axis, {
    String? label,
    required double width,
    bool fill = false,
  }) {
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
    final isScale = id == 'scale' || id == 'scale.z';
    final unit = isScale ? '%' : _unitOf(row);
    final percent = isScale || unit == '%' && max == 1;
    final speed = isScale
        ? .01
        : min != null && max != null
        ? (max - min) / 300
        : id.startsWith('scale')
        ? .005
        : _kindOf(row) == _Kind.angle
        ? .5
        : 1.0;
    final shownScale = percent ? 100.0 : 1.0;
    return Builder(
      key: axis == 0 ? _rows.putIfAbsent(id, () => GlobalKey()) : null,
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
              key: ValueKey('inspector:$id:$axis'),
              owner: layer['id'],
              value: value.toDouble() * shownScale,
              idleFocus: _nodes.putIfAbsent(
                '$id:$axis',
                () => FocusNode(debugLabel: 'Inspector $id $axis'),
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
              onBegin: _begin,
              onPreview: (n) => _write(
                layer,
                row,
                _axisValue(row, axis, n / shownScale),
                preview: true,
              ),
              onCommit: (n) => _write(
                layer,
                row,
                _axisValue(row, axis, n / shownScale),
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

  Widget _headGlyph(IconData icon, String tip, VoidCallback? onTap) =>
      EditorTooltip(
        message: tip,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: onTap,
          child: Padding(
            padding: const EdgeInsets.only(left: EditorMetrics.s6),
            child: Icon(
              icon,
              size: EditorMetrics.s12,
              color: onTap == null
                  ? EditorTheme.disabledInk
                  : EditorTheme.muted,
            ),
          ),
        ),
      );

  /// Two wells side by side in one cell, and the wells beside a pad.
  double get _half => (_cellWidth - EditorMetrics.s4) / 2;
  double get _beside => _cellWidth - EditorMetrics.s60 - EditorMetrics.s4;

  /// One name per row, never two marks for one meaning: the word while it
  /// fits, the glyph when the panel is too narrow for words. The name is a
  /// step quieter than the value it names.
  Widget _name([IconData? icon, String? label]) => SizedBox(
    width: EditorMetrics.s18 + _wordWidth,
    child: label == null
        ? null
        : _wordWidth == 0
        ? EditorTooltip(
            message: label,
            child: Icon(
              icon,
              size: EditorMetrics.s14,
              color: EditorTheme.muted,
            ),
          )
        : Text(
            label,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(
              fontSize: EditorMetrics.dense,
              color: EditorTheme.muted,
            ),
          ),
  );

  Widget _gap() => const SizedBox(width: EditorMetrics.s4);

  /// One column of the grid every row shares: the name, three wells, and a
  /// fixed tail for the row's own extra (dial, link). Empty slots keep their
  /// width so the columns never move. The name and tail columns are fixed;
  /// the three well columns share what is left, because the wells are what
  /// the panel is for.
  double _wellWidth = EditorMetrics.field;

  /// The word part of the name column; 0 when the panel is too narrow for
  /// three wells beside it — the name falls back to its glyph and tooltip.
  double _wordWidth = EditorMetrics.s48;

  /// The room a card's contents get: the panel less the card's margin and
  /// padding. Measured once for the panel instead of once per row of cells.
  double _cardWidth = EditorMetrics.cell;
  void _fit(double panelWidth) {
    _cardWidth = math.max(1, panelWidth - EditorMetrics.s6 * 4);
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
    return [
      if (_row('camera.center') != null)
        _line([
          _name(Icons.center_focus_strong, 'Center'),
          _slot(_well(layer, 'camera.center', 0, label: 'X')),
          _gap(),
          _slot(_well(layer, 'camera.center', 1, label: 'Y')),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      if (_row('camera.target.z') != null)
        _line([
          _name(Icons.center_focus_weak, 'Target Z'),
          _slot(_well(layer, 'camera.target.z', 0, label: 'Z')),
          _gap(),
          _slot(),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      if (_row('camera.target') != null)
        _line([
          _name(Icons.gps_fixed, 'Target'),
          Expanded(
            child: _live2(['camera.target'], () {
              final current = _row('camera.target')?['value'];
              return EditorChoice<dynamic>(
                value: current is num ? current.toInt() : 0,
                choices: [
                  const MapEntry(0, 'None'),
                  ...c.layers
                      .where((v) => v['id'] != layer['id'])
                      .map((v) => MapEntry(v['id'], '${v['name']}')),
                ],
                onChanged: (v) => c.command('setProperty', {
                  'layer': layer['id'],
                  'property': 'camera.target',
                  'value': v,
                }),
              );
            }),
          ),
        ]),
      if (_row('camera.orbit') != null)
        _line([
          _name(Icons.threesixty, 'Orbit'),
          _slot(_well(layer, 'camera.orbit', 0, label: 'Pitch')),
          _gap(),
          _slot(_well(layer, 'camera.orbit', 1, label: 'Yaw')),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      if (_row('camera.distance') != null)
        _line([
          _name(Icons.straighten, 'Distance'),
          _slot(_well(layer, 'camera.distance', 0, label: 'Scale')),
          _gap(),
          _slot(),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      if (_row('camera.zoom') != null)
        _line([
          _name(Icons.zoom_in, 'Zoom'),
          _slot(_well(layer, 'camera.zoom', 0, label: 'Zoom')),
          _gap(),
          _slot(),
          _gap(),
          _slot(),
          _gap(),
          _tail(),
        ]),
      if (_row('camera.roll') != null)
        _line([
          _name(Icons.rotate_right, 'Roll'),
          _slot(_well(layer, 'camera.roll', 0, label: 'Roll')),
          _gap(),
          _slot(),
          _gap(),
          _slot(),
          _gap(),
          _tail(_dial(layer, 'camera.roll', EditorTheme.angle)),
        ]),
    ];
  }

  /// The angle wheel of one row, watching that row alone.
  Widget _dial(Map<String, dynamic> layer, String id, [Color? tint]) =>
      _live2([id], () {
        final row = _row(id);
        return row == null
            ? const SizedBox.shrink()
            : EditorDial(
                degrees: (row['value'] as num? ?? 0).toDouble(),
                enabled: _canEdit(layer),
                tint: tint,
                onBegin: _begin,
                onPreview: (d) => _write(layer, row, d, preview: true),
                onFinish: () => _finish(false),
                onCancel: () => _finish(true),
              );
      });

  /// Property ids the Transform card already shows.
  static const _transformIds = {
    'position',
    'position.z',
    'scale',
    'scale.z',
    'rotation',
    'rotation.x',
    'rotation.y',
    'opacity',
    'anchor',
    'camera.center',
    'camera.target.z',
    'camera.target',
    'camera.orbit',
    'camera.distance',
    'camera.zoom',
    'camera.roll',
    'content',
  };

  bool _isTextProperty(Map<String, dynamic> r) =>
      '${r['id']}'.startsWith('text') ||
      const ['Size', 'Line height', 'Tracking'].contains(r['label']);

  /// A text layer: what it says, then how it is set.
  List<Widget> _text(Map<String, dynamic> layer, Map<String, dynamic> text) {
    final rows = panelRows(layer['properties'])
        .where(_isTextProperty)
        .where((r) => r['id'] != 'text_justify' && r['label'] != 'Size')
        .toList();
    final align = _row('text_justify');
    return [
      RichTextEditor(
        key: ValueKey('rich:${layer['id']}'),
        controller: c,
        layer: layer,
        text: text,
      ),
      if (align != null)
        Padding(
          padding: const EdgeInsets.symmetric(vertical: EditorMetrics.s6),
          child: Row(
            children: [
              for (final entry in const {
                0: Icons.format_align_left,
                2: Icons.format_align_center,
                1: Icons.format_align_right,
              }.entries)
                Expanded(
                  child: IconButton(
                    tooltip: const {
                      0: 'Align left',
                      2: 'Align center',
                      1: 'Align right',
                    }[entry.key],
                    isSelected: align['value'] == entry.key,
                    icon: Icon(entry.value, size: EditorMetrics.s16),
                    color: align['value'] == entry.key
                        ? EditorTheme.accent
                        : EditorTheme.muted,
                    onPressed: _canEdit(layer)
                        ? () => _write(layer, align, entry.key, preview: false)
                        : null,
                  ),
                ),
            ],
          ),
        ),
      if (rows.isNotEmpty) ...[
        const SizedBox(height: EditorMetrics.s6),
        _cells([for (final r in rows) _Cell(_control(layer, '${r['id']}'))]),
      ],
    ];
  }

  /// The layer's colours: a swatch that opens the Colors desk, and the hex.
  List<Widget> _colors(Map<String, dynamic> layer) {
    final colors = panelRows(layer['colors']);
    return [
      if (layer['kind'] == 'Shape' && layer['fill'] is Map)
        GradientInspector(
          key: ValueKey('fill:${layer['id']}'),
          controller: c,
          layer: layer,
          fill: panelMap(layer['fill']),
        )
      else
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
        _name(Icons.layers_outlined, 'Source'),
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
        _name(Icons.contrast, 'Mode'),
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
    final scaleEven = _scaleEven();
    final anchor = layer['anchorFraction'];
    return [
      if (_row('position') != null)
        _line([
          _name(Icons.open_with, 'Position'),
          _slot(_well(layer, 'position', 0, label: 'X')),
          _gap(),
          _slot(_well(layer, 'position', 1, label: 'Y')),
          _gap(),
          _slot(
            _row('position.z') == null
                ? null
                : _well(layer, 'position.z', 0, label: 'Z'),
          ),
          _gap(),
          _tail(),
        ]),
      if (_row('scale') != null)
        _line([
          _name(Icons.aspect_ratio, 'Scale'),
          _slot(
            _well(
              layer,
              'scale',
              0,
              label: _scaleLocked && scaleEven ? 'Scale' : 'X',
            ),
          ),
          _gap(),
          _slot(
            _scaleLocked && scaleEven
                ? null
                : _well(layer, 'scale', 1, label: 'Y'),
          ),
          _gap(),
          _slot(
            _row('scale.z') == null
                ? null
                : _well(layer, 'scale.z', 0, label: 'Z'),
          ),
          _gap(),
          _tail(
            EditorSwitch(
              on: _scaleLocked,
              glyph: Icons.link,
              compact: true,
              label: 'Keep the shape: one number scales both axes',
              onChanged: _canEdit(layer)
                  ? (on) => setState(() => _scaleLocked = on)
                  : null,
            ),
          ),
        ]),
      if (_row('rotation') != null)
        _line([
          _name(Icons.rotate_right, 'Rotation'),
          _slot(_well(layer, 'rotation', 0, label: 'Rotation')),
          _gap(),
          _slot(
            _row('rotation.x') == null
                ? null
                : _well(layer, 'rotation.x', 0, label: 'Tilt X'),
          ),
          _gap(),
          _slot(
            _row('rotation.y') == null
                ? null
                : _well(layer, 'rotation.y', 0, label: 'Tilt Y'),
          ),
          _gap(),
          _tail(_dial(layer, 'rotation')),
        ]),
      if (_row('opacity') != null)
        _line([
          _name(Icons.opacity, 'Opacity'),
          SizedBox(
            width: _wellWidth * 2 + EditorMetrics.s4,
            height: EditorMetrics.s22,
            child: _well(
              layer,
              'opacity',
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
        _name(Icons.center_focus_weak, 'Anchor'),
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
        _name(Icons.view_in_ar_outlined, 'Space'),
        for (final p in ['2D', '2.5D', '3D'])
          Padding(
            padding: const EdgeInsets.only(right: EditorMetrics.s2),
            child: EditorTooltip(
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
        _name(Icons.account_tree_outlined, 'Parent'),
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
        _name(Icons.layers_outlined, 'Blend'),
        Expanded(
          child: EditorButton(
            '${layer['blendMode'] ?? 'Normal'}',
            () => c.focusEditing(layer['id'] as int, 'blendMode'),
            tooltip: 'Blend mode (opens the Blend desk)',
          ),
        ),
      ]),
      _line([
        _name(),
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

  /// Everything an effect can be told to do, in one list: the head keeps a
  /// single mark instead of a row of equal glyphs.
  Future<void> _effectMenu(
    BuildContext context,
    Map<String, dynamic> layer,
    Map<String, dynamic> effect,
    int index,
    int count,
  ) async {
    final box = context.findRenderObject() as RenderBox?;
    final at = box == null
        ? Offset.zero
        : box.localToGlobal(box.size.bottomLeft(Offset.zero));
    final chosen = await showMenu<String>(
      context: context,
      position: RelativeRect.fromLTRB(at.dx, at.dy, at.dx, at.dy),
      items: [
        EditorMenuItem<String>(
          value: 'earlier',
          enabled: panelCan(c, 'moveEffect') && index > 0,
          child: const Text('Apply earlier'),
        ),
        EditorMenuItem<String>(
          value: 'later',
          enabled: panelCan(c, 'moveEffect') && index < count - 1,
          child: const Text('Apply later'),
        ),
        EditorMenuItem<String>(
          value: 'roll',
          enabled: _canEdit(layer),
          child: const Text('Throw every number within its reach'),
        ),
        EditorMenuItem<String>(
          value: 'rest',
          enabled: _canEdit(layer),
          child: const Text('Back to where the numbers rest'),
        ),
        if (effect['placement'] == true)
          EditorMenuItem<String>(
            value: 'expand',
            enabled: panelCan(c, 'expandEffect'),
            child: const Text('Expand copies into layers'),
          ),
        EditorMenuItem<String>(
          value: 'remove',
          enabled: panelCan(c, 'removeEffect'),
          child: const Text('Remove effect'),
        ),
      ],
    );
    switch (chosen) {
      case 'earlier' || 'later':
        await c.command('moveEffect', {
          'layer': layer['id'],
          'id': effect['id'],
          'to': chosen == 'earlier' ? index - 1 : index + 1,
        });
      case 'roll':
        await _roll(layer, effect);
      case 'rest':
        await _rest(layer, effect);
      case 'expand':
        await c.command('expandEffect', {
          'layer': layer['id'],
          'id': effect['id'],
        });
      case 'remove':
        await c.command('removeEffect', {
          'layer': layer['id'],
          'id': effect['id'],
        });
    }
  }

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
        into.add(_Cell(_pointControl(layer, id, yId, hero: hero), tall: true));
        continue;
      }
      into.add(_Cell(_control(layer, id, hero: hero)));
    }
    final key = '${effect['id']}';
    final open = _advancedOpen.contains(key);
    return EditorCard(
      key: ValueKey('effect:$key'),
      title: '${effect['name']}',
      dim: effect['enabled'] == false,
      // The order is the pipeline: grab the head to move the effect up or
      // down it; the menu keeps the same move for one step at a time.
      leading: panelCan(c, 'moveEffect')
          ? ReorderableDragStartListener(
              index: index,
              child: EditorTooltip(
                message: 'Drag to reorder',
                child: MouseRegion(
                  cursor: SystemMouseCursors.grab,
                  child: Padding(
                    padding: const EdgeInsets.only(right: EditorMetrics.s4),
                    child: Icon(
                      Icons.drag_indicator,
                      size: EditorMetrics.s12,
                      color: EditorTheme.muted,
                    ),
                  ),
                ),
              ),
            )
          : null,
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          // Only the state a glance needs stays on the head; the seven
          // same-sized glyphs that used to sit here now live behind one.
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
          Builder(
            builder: (context) => _headGlyph(
              Icons.more_horiz,
              'Effect actions',
              () => _effectMenu(context, layer, effect, index, count),
            ),
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

  /// The cell width the user set at the panel's foot (the zoom strip);
  /// EditorMetrics.cell until touched.
  double get _cellWidth =>
      (c.deskWork.value['inspectorCell'] as num? ?? EditorMetrics.cell)
          .toDouble()
          .clamp(InspectorCell.min, InspectorCell.max);

  /// Controls in equal columns on fixed rows (a word, then a control), so
  /// every cell sits on the same grid; a section label spans them all.
  Widget _cells(List<_Cell> cells) {
    // Cells keep one width and pack from the left, so the gap between items
    // never grows; a wider panel or a smaller cell only adds columns.
    const gap = EditorMetrics.s6;
    final room = _cardWidth;
    final wanted = _cellWidth;
    final columns = math.max(1, ((room + gap) / (wanted + gap)).floor());
    // One column takes the whole width; more columns keep the cell's own.
    final cell = columns == 1
        ? room
        : math.min(wanted, (room - gap * (columns - 1)) / columns);
    return Wrap(
      spacing: EditorMetrics.s6,
      runSpacing: EditorMetrics.s6,
      children: [
        for (final c in cells)
          SizedBox(width: c.wide ? room : cell, child: c.child),
      ],
    );
  }

  /// A pair of params as one point: drag the dot, or type either number.
  Widget _pointControl(
    Map<String, dynamic> layer,
    String xId,
    String yId, {
    bool hero = false,
  }) => _live2([xId, yId], () {
    final xRow = _row(xId), yRow = _row(yId);
    return xRow == null || yRow == null
        ? const SizedBox.shrink()
        : _pointBody(layer, xRow, yRow, hero: hero);
  });

  Widget _pointBody(
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
        _cellLabel(label, hero),
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
              onBegin: _begin,
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
                _wellBody(layer, xRow, 0, label: '$label X', width: _beside),
                const SizedBox(height: EditorMetrics.s4),
                _wellBody(layer, yRow, 0, label: '$label Y', width: _beside),
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

  /// The word over a cell's control: the smallest type in the panel, so the
  /// value under it is what the eye lands on. A hero's word is ink, the rest
  /// stay muted; the family's hue rides the value, not the word.
  Widget _cellLabel(String label, [bool hero = false]) => Padding(
    padding: const EdgeInsets.only(bottom: EditorMetrics.s2),
    child: Text(
      label,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: TextStyle(
        fontSize: EditorMetrics.micro,
        color: hero ? EditorTheme.ink : EditorTheme.muted,
      ),
    ),
  );

  /// A labelled control for one declared param: the word above, the control
  /// under it, sized by its kind.
  Widget _control(Map<String, dynamic> layer, String id, {bool hero = false}) =>
      _live2([id], () {
        final row = _row(id);
        return row == null
            ? const SizedBox.shrink()
            : _controlBody(layer, row, hero: hero);
      });

  Widget _controlBody(
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
          width: _cellWidth,
          child: EditorChoice<dynamic>(
            value: (row['value'] as num?)?.round(),
            choices: [
              if (choices is List)
                for (var i = 0; i < choices.length; i++)
                  MapEntry(i, '${choices[i]}'),
            ],
            onChanged: _canEdit(layer)
                ? (v) => _write(layer, row, v, preview: false)
                : null,
          ),
        );
      case _Kind.vec2:
        body = Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _wellBody(layer, row, 0, label: '$label X', width: _half),
            _gap(),
            _wellBody(layer, row, 1, label: '$label Y', width: _half),
          ],
        );
      case _Kind.bounded:
        body = _wellBody(layer, row, 0, fill: _tight(row), width: _cellWidth);
      case _Kind.angle:
        body = Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            EditorDial(
              degrees: (row['value'] as num? ?? 0).toDouble(),
              enabled: _canEdit(layer),
              tint: EditorTheme.angle,
              onBegin: _begin,
              onPreview: (d) => _write(layer, row, d, preview: true),
              onFinish: () => _finish(false),
              onCancel: () => _finish(true),
            ),
            _gap(),
            _wellBody(
              layer,
              row,
              0,
              width: _cellWidth - EditorMetrics.s22 - EditorMetrics.s4,
            ),
          ],
        );
      case _Kind.text:
        body = SizedBox(
          width: _cellWidth,
          child: EditorDraftField(
            key: ValueKey('${layer['id']}:${row['id']}'),
            value: '${row['value'] ?? ''}',
            label: label,
            enabled: _canEdit(layer),
            onCommit: (v) => _write(layer, row, v, preview: false),
          ),
        );
      case _Kind.color:
        final rgba = (row['value'] as List).cast<num>();
        final hex = rgba
            .map(
              (v) => (v.clamp(0, 1) * 255)
                  .round()
                  .toRadixString(16)
                  .padLeft(2, '0'),
            )
            .join();
        body = SizedBox(
          width: _cellWidth,
          child: EditorDraftField(
            key: ValueKey('${layer['id']}:${row['id']}'),
            value: '#$hex',
            label: label,
            enabled: _canEdit(layer),
            validator: (v) =>
                _parsePropertyColor(v) == null ? 'Use RGB or RGBA hex' : null,
            onCommit: (v) {
              final next = _parsePropertyColor(v)!;
              return _write(
                layer,
                row,
                rgba.length == 3 ? next.take(3).toList() : next,
                preview: false,
              );
            },
          ),
        );
      case _Kind.scale:
      case _Kind.scalar:
        final seed = '${row['id']}'.endsWith('.seed');
        body = Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _wellBody(
              layer,
              row,
              0,
              width: seed
                  ? _cellWidth - EditorMetrics.s16 - EditorMetrics.s4
                  : _cellWidth,
            ),
            if (seed) ...[
              _gap(),
              EditorTooltip(
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
          children: [_cellLabel(label, hero), body],
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
          child: EditorTooltip(
            message: '${layer['name']}',
            child: Text(
              _multiple ? '${c.selectedIds.length} layers' : '${layer['name']}',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                fontSize: EditorMetrics.title,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
        ),
        EditorSwitch(
          on: c.animating,
          glyph: Icons.diamond_outlined,
          tint: EditorTheme.keyAccent,
          label: c.animateFrom
              ? 'Animate (A): values you touch become keys at this frame, '
                    'and at the frame Animate was turned on'
              : 'Animate (A): values you touch become keys at this frame',
          onChanged: panelCan(c, 'animate') ? c.setAnimate : null,
        ),
      ],
    ),
  );

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: c.deskWork,
    builder: (context, _) {
      final layer = _active;
      if (layer == null) {
        return const ColoredBox(
          color: EditorTheme.app,
          child: Center(
            child: Text(
              'Select a layer',
              style: TextStyle(
                fontSize: EditorMetrics.title,
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
                (r['value'] is num ||
                    r['value'] is List ||
                    r['value'] is String),
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
                  child: CustomScrollView(
                    controller: _scroll,
                    slivers: [
                      SliverList.list(
                        children: [
                          if (layer['kind'] == 'Camera')
                            EditorCard(
                              title: 'Camera',
                              children: _camera(layer),
                            )
                          else
                            EditorCard(
                              title: 'Transform',
                              children: _transform(layer),
                            ),
                          if (layer['kind'] != 'Camera')
                            EditorCard(title: 'World', children: _world(layer)),
                          if (!_multiple && text.isNotEmpty)
                            EditorCard(
                              title: 'Text',
                              children: _text(layer, text),
                            ),
                          if (!_multiple &&
                              panelRows(layer['colors']).isNotEmpty)
                            EditorCard(
                              title: 'Color',
                              children: _colors(layer),
                            ),
                          if (!_multiple &&
                              matte.isNotEmpty &&
                              layer['clipToBelow'] != true)
                            EditorCard(
                              title: 'Matte',
                              children: _matte(layer, matte),
                            ),
                          if (rest.isNotEmpty)
                            EditorCard(
                              title: 'Properties',
                              children: [
                                _cells([
                                  for (final r in rest)
                                    _Cell(_control(layer, '${r['id']}')),
                                ]),
                              ],
                            ),
                        ],
                      ),
                      if (!_multiple)
                        SliverReorderableList(
                          itemCount: effects.length,
                          itemBuilder: (context, i) => _effect(
                            layer,
                            effects[i],
                            index: i,
                            count: effects.length,
                          ),
                          onReorderItem: (from, to) => c.command('moveEffect', {
                            'layer': layer['id'],
                            'id': effects[from]['id'],
                            'to': to,
                          }),
                        ),
                      const SliverPadding(
                        padding: EdgeInsets.only(bottom: EditorMetrics.s6),
                      ),
                    ],
                  ),
                ),
                EditorZoomBar(
                  base: EditorMetrics.cell,
                  value: _cellWidth,
                  min: InspectorCell.min,
                  max: InspectorCell.max,
                  keyPrefix: 'inspector:cell',
                  onChanged: (v) => c.storeDesk('inspectorCell', v),
                ),
              ],
            ),
          );
        },
      );
    },
  );
}

/// One reading of the Inspector: it rebuilds when the rows it names move and
/// stays as it is while the rest of the panel is rebuilt around it.
class _Live extends StatelessWidget {
  const _Live({required this.pulse, required this.body});
  final ValueNotifier<Object?> pulse;
  final Widget Function() body;
  @override
  Widget build(BuildContext context) => ValueListenableBuilder<Object?>(
    valueListenable: pulse,
    builder: (_, __, ___) => body(),
  );
}

/// How wide a card cell may be: narrow packs three columns into a dock,
/// wide gives one number a whole row.
abstract final class InspectorCell {
  static const double min = 88, max = 200;
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
  Widget build(BuildContext context) => EditorTooltip(
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

List<double>? _parseHex(String input) {
  var text = input.trim().replaceFirst(RegExp(r'^#+'), '');
  if (text.length == 3) text = text.split('').map((c) => '$c$c').join();
  if (!RegExp(r'^[a-fA-F0-9]{6}$').hasMatch(text)) return null;
  return [
    for (var i = 0; i < 3; i++)
      int.parse(text.substring(i * 2, i * 2 + 2), radix: 16) / 255,
  ];
}

List<double>? _parsePropertyColor(String input) {
  final text = input.trim().replaceFirst(RegExp(r'^#'), '');
  if (RegExp(r'^[a-fA-F0-9]{8}$').hasMatch(text)) {
    return [
      for (var i = 0; i < 4; i++)
        int.parse(text.substring(i * 2, i * 2 + 2), radix: 16) / 255,
    ];
  }
  final rgb = _parseHex(input);
  return rgb == null ? null : [...rgb, 1.0];
}
