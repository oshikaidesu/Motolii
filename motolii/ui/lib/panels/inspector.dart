import '../session/read_model.dart';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';
import '../foundation/metrics.dart';

class InspectorPanel extends StatefulWidget {
  const InspectorPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> {
  EditorSession get controller => widget.controller;
  final _rows = <String, GlobalKey>{};
  final _nodes = <String, FocusNode>{};
  _AxisDrag? _axisDrag;
  List<Map<String, dynamic>> get _layers => controller.liveLayers();

  Map<String, dynamic>? get _active {
    for (final layer in _layers) {
      if (controller.selectedIds.isNotEmpty &&
          layer['id'] == controller.selectedIds.last)
        return layer;
    }
    return controller.activeLayer;
  }

  @override
  void initState() {
    super.initState();
    controller.focusProperty.addListener(_reveal);
  }

  void _reveal() {
    final id = controller.focusProperty.value;
    if (id == null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final target = _rows[id]?.currentContext;
      if (target != null)
        Scrollable.ensureVisible(
          target,
          duration: const Duration(milliseconds: 120),
        );
      _nodes['$id:0']?.requestFocus();
    });
  }

  @override
  void dispose() {
    controller.focusProperty.removeListener(_reveal);
    for (final node in _nodes.values) {
      node.dispose();
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

  void _beginAxis(
    Map<String, dynamic> layer,
    Map<String, dynamic> property,
    int axis,
  ) {
    final id = '${property['id']}';
    final values = <int, dynamic>{};
    final layers = _layers;
    for (final target in layers.where(
      (candidate) => controller.selectedIds.contains(candidate['id']),
    )) {
      final value = _property(target, id)?['value'];
      if (value is List) {
        values[target['id'] as int] = List<dynamic>.from(value);
      } else if (value is num) {
        values[target['id'] as int] = value.toDouble();
      }
    }
    final primary = values[layer['id'] as int];
    final base = primary is List && axis < primary.length
        ? primary[axis]
        : primary;
    if (base is num) {
      _axisDrag = _AxisDrag(id, axis, base.toDouble(), values);
    }
  }

  Future<void> _finishAxis(bool cancel) {
    _axisDrag = null;
    return controller.command(cancel ? 'cancelPreview' : 'commitPreview');
  }

  Future<void> _axis(
    Map<String, dynamic> layer,
    Map<String, dynamic> property,
    int axis,
    double value,
    bool preview,
  ) async {
    final id = property['id'];
    final edits = <Map<String, dynamic>>[];
    final current = property['value'];
    final drag = preview && _axisDrag?.property == id && _axisDrag?.axis == axis
        ? _axisDrag
        : null;
    final base =
        drag?.primaryBase ??
        (current is List
            ? (current[axis] as num).toDouble()
            : (current as num).toDouble());
    for (final target in _layers.where(
      (v) => controller.selectedIds.contains(v['id']),
    )) {
      if (target['locked'] == true) continue;
      final row = _property(target, '$id');
      if (row == null) continue;
      dynamic next = drag?.values[target['id']] ?? row['value'];
      if (next is List) {
        final values = List<dynamic>.from(next);
        if (axis >= values.length || values[axis] is! num) continue;
        values[axis] = preview
            ? (values[axis] as num).toDouble() + value - base
            : value;
        next = values;
      } else if (next is num) {
        next = preview ? next.toDouble() + value - base : value;
      } else {
        continue;
      }
      edits.add({'layer': target['id'], 'property': id, 'value': next});
    }
    if (edits.isEmpty) return;
    await controller.command('previewProperties', {'edits': edits});
    if (!preview) await controller.command('commitPreview');
  }

  Widget _editingTarget(
    Map<String, dynamic> layer,
    String property,
    Widget child,
  ) => Listener(
    onPointerDown: (_) => controller.focusEditing(layer['id'] as int, property),
    child: Focus(
      canRequestFocus: false,
      onFocusChange: (focused) {
        if (focused) controller.focusEditing(layer['id'] as int, property);
      },
      child: child,
    ),
  );

  Widget _cell(
    Map<String, dynamic> layer,
    Map<String, dynamic>? row,
    int axis,
  ) {
    if (row == null) return const SizedBox(width: EditorMetrics.field);
    final v = row['value'];
    final value = v is List && axis < v.length
        ? v[axis]
        : axis == 0 && v is num
        ? v
        : null;
    if (value is! num) return const SizedBox(width: EditorMetrics.field);
    final id = '${row['id']}';
    final others = _layers.where(
      (v) => controller.selectedIds.contains(v['id']),
    );
    final mixed = others.any((layer) {
      final other = _property(layer, id)?['value'];
      final n = other is List && axis < other.length ? other[axis] : other;
      return n != value;
    });
    final min = (row['min'] as num?)?.toDouble(),
        max = (row['max'] as num?)?.toDouble();
    final speed = min != null && max != null
        ? (max - min) / 300
        : id.startsWith('scale') || id == 'opacity'
        ? .005
        : id.startsWith('rotation')
        ? .5
        : 1.0;
    return _editingTarget(
      layer,
      id,
      SizedBox(
        width: EditorMetrics.field,
        child: EditorNumericField(
          key: ValueKey('${layer['id']}:$id:$axis'),
          value: value.toDouble(),
          idleFocus: _nodes.putIfAbsent(
            '$id:$axis',
            () => FocusNode(debugLabel: 'Inspector $id $axis'),
          ),
          label: '${row['label']} ${['X', 'Y', 'Z'][axis.clamp(0, 2)]}',
          mixed: mixed,
          min: min,
          max: max,
          speed: speed,
          enabled:
              layer['locked'] != true &&
              panelCan(controller, 'previewProperties') &&
              panelCan(controller, 'commitPreview'),
          onPreview: (n) => _axis(layer, row, axis, n, true),
          onCommit: (n) => _axis(layer, row, axis, n, false),
          onBegin: () => _beginAxis(layer, row, axis),
          onFinish: () => _finishAxis(false),
          onCancel: () => _finishAxis(true),
        ),
      ),
    );
  }

  /// Advanced folds opened by the user, per layer and effect.
  final _advancedOpen = <String>{};

  /// A placement effect declares its own grid: a head (count, shape), then
  /// one row per attribute with an Each and a Random cell.
  List<Widget> _placementGrid(
    Map<String, dynamic> layer,
    Map<String, dynamic> effect,
  ) {
    final layout = panelMap(effect['layout']);
    final params = panelRows(effect['params']);
    Map<String, dynamic>? row(dynamic id) =>
        id == null ? null : params.where((r) => r['id'] == id).firstOrNull;
    Widget unit(String text) => SizedBox(
      width: EditorMetrics.s16,
      child: Text(
        text,
        style: const TextStyle(
          fontSize: EditorMetrics.dense,
          color: EditorTheme.muted,
        ),
      ),
    );
    Widget mark(List<Map<String, dynamic>?> rows) {
      final live = rows.whereType<Map<String, dynamic>>();
      final now = live.any((r) => r['keyedNow'] == true);
      final any = live.any((r) => (r['keys'] as List? ?? const []).isNotEmpty);
      return SizedBox(
        width: EditorMetrics.row,
        child: Center(
          child: Text(
            now
                ? '◆'
                : any
                ? '◇'
                : '',
            style: const TextStyle(
              fontSize: EditorMetrics.dense,
              color: EditorTheme.muted,
            ),
          ),
        ),
      );
    }

    final canSet =
        panelCan(controller, 'setProperty') && layer['locked'] != true;
    Widget gridLine(Map<String, dynamic> grid) => _line('${grid['label']}', [
      for (final id in [grid['each'], grid['random']])
        if (row(id) case final r?)
          _cell(layer, r, (grid['axis'] as num?)?.toInt() ?? 0)
        else
          const SizedBox(width: EditorMetrics.field),
      if (grid['random'] != null && grid['random'] == layout['seed'])
        panelButton(
          '↻',
          canSet
              ? () => controller.command('setProperty', {
                  'layer': layer['id'],
                  'property': grid['random'],
                  'value':
                      (((row(grid['random'])?['value'] as num?) ?? 0) + 1) %
                      10000,
                })
              : null,
          tooltip: 'Next seed: same spread, different scatter',
        )
      else
        unit('${grid['unit'] ?? ''}'),
      mark([row(grid['each']), row(grid['random'])]),
    ]);
    final materials = panelRows(layout['materials']);
    final shares = [
      for (final m in materials)
        ((row(m['id'])?['value'] as num?) ?? 0).toDouble(),
    ];
    final shareTotal = shares.fold<double>(0, (a, b) => a + b);
    final pick = row(layout['pick']);
    final pickChoices = pick?['choices'];
    final subject = row(layout['subject']);
    final subjectChoices = subject?['choices'];
    final wholeGroup = (subject?['value'] as num?)?.round() == 1;
    Widget choiceLine(String label, Map<String, dynamic> r, List choices) =>
        _line(label, [
          Expanded(
            child: _dropdown(
              (r['value'] as num?)?.round(),
              [
                for (var i = 0; i < choices.length; i++)
                  MapEntry(i, '${choices[i]}'),
              ],
              canSet
                  ? (v) => controller.command('setProperty', {
                      'layer': layer['id'],
                      'property': r['id'],
                      'value': v,
                    })
                  : null,
            ),
          ),
        ]);
    final advancedRows = panelRows(layout['rows'])
        .where((g) => g['advanced'] == true)
        .toList();
    final key = '${layer['id']}:${effect['id']}';
    final open = _advancedOpen.contains(key);
    bool touched = false;
    for (final grid in advancedRows)
      for (final id in [grid['each'], grid['random']]) {
        final r = row(id);
        if (r == null) continue;
        final v = r['value'];
        final zero = v is num ? v == 0 : v is List && v.every((c) => c == 0);
        if (!zero || (r['keys'] as List? ?? const []).isNotEmpty)
          touched = true;
      }
    final count = row(layout['count']);
    final along = row(layout['along']);
    final choices = along?['choices'];
    return [
      _line('Count', [
        if (count != null) _cell(layer, count, 0),
        const SizedBox(width: EditorMetrics.s8),
        if (along != null && choices is List)
          Expanded(
            child: _dropdown(
              (along['value'] as num?)?.round(),
              [
                for (var i = 0; i < choices.length; i++)
                  MapEntry(i, '${choices[i]}'),
              ],
              panelCan(controller, 'setProperty') && layer['locked'] != true
                  ? (v) => controller.command('setProperty', {
                      'layer': layer['id'],
                      'property': along['id'],
                      'value': v,
                    })
                  : null,
            ),
          ),
      ]),
      if (materials.isNotEmpty) ...[
        if (subject != null && subjectChoices is List)
          choiceLine('Copies', subject, subjectChoices),
      ],
      if (materials.isNotEmpty && !wholeGroup) ...[
        if (pick != null && pickChoices is List)
          choiceLine('Pick', pick, pickChoices),
        _line('Materials', [
          SizedBox(
            width: EditorMetrics.field,
            child: Text(
              'Share',
              textAlign: TextAlign.right,
              style: const TextStyle(fontSize: EditorMetrics.dense),
            ),
          ),
        ]),
        for (var i = 0; i < materials.length; i++)
          if (row(materials[i]['id']) case final r?)
            _line('${materials[i]['label']}', [
              const SizedBox(width: EditorMetrics.field),
              _cell(layer, r, 0),
              SizedBox(
                width: EditorMetrics.s32,
                child: Text(
                  shareTotal > 0
                      ? '${(shares[i] / shareTotal * 100).round()}%'
                      : '',
                  style: const TextStyle(
                    fontSize: EditorMetrics.dense,
                    color: EditorTheme.muted,
                  ),
                ),
              ),
              mark([r]),
            ]),
      ],
      for (final field in panelRows(layout['shape']))
        if (row(field['id']) case final r?)
          _line('${field['label']}', [
            const SizedBox(width: EditorMetrics.field),
            _cell(layer, r, 0),
            unit('${field['unit'] ?? ''}'),
            mark([r]),
          ]),
      _line('', [
        for (final column
            in panelRows(layout['columns']).isEmpty
                ? (layout['columns'] as List? ?? const ['Each', 'Random'])
                : const ['Each', 'Random'])
          SizedBox(
            width: EditorMetrics.field,
            child: Text(
              '$column',
              textAlign: TextAlign.right,
              style: const TextStyle(fontSize: EditorMetrics.dense),
            ),
          ),
      ]),
      for (final grid in panelRows(layout['rows']))
        if (grid['advanced'] != true) gridLine(grid),
      if (advancedRows.isNotEmpty) ...[
        _line('Advanced', [
          panelButton(
            open ? '▾' : '▸',
            () => setState(() {
              open ? _advancedOpen.remove(key) : _advancedOpen.add(key);
            }),
            tooltip: 'Values that do not change the picture by themselves',
          ),
          if (!open && touched)
            const Text('•', style: TextStyle(color: EditorTheme.muted)),
        ]),
        if (open)
          for (final grid in advancedRows) gridLine(grid),
      ],
    ];
  }

  /// Effect rows carry their own section and, for a shape toggle, choices.
  List<Widget> _effectRows(
    Map<String, dynamic> layer,
    List<Map<String, dynamic>> rows,
  ) {
    final out = <Widget>[];
    String? section;
    for (final row in rows) {
      final here = row['section'] as String?;
      if (here != null && here != section && here != 'Shape') {
        out.add(_line(here, []));
      }
      section = here;
      final choices = row['choices'];
      if (choices is List) {
        out.add(
          _choice(
            '${row['label'] ?? row['id']}',
            (row['value'] as num?)?.round(),
            [
              for (var i = 0; i < choices.length; i++)
                MapEntry(i, '${choices[i]}'),
            ],
            panelCan(controller, 'setProperty') && layer['locked'] != true
                ? (v) => controller.command('setProperty', {
                    'layer': layer['id'],
                    'property': row['id'],
                    'value': v,
                  })
                : null,
          ),
        );
      } else {
        out.add(_row(layer, row));
      }
    }
    return out;
  }

  /// ゴースト: この層を遅れ(f)だけ後に見た姿が 1 枚。行は増えず、Stage では掴めない。
  /// 複製が要るなら Delay の効果(Repeater)。
  List<Widget> _ghostLines(Map<String, dynamic> layer) {
    final ghost = (layer['ghost'] as num?)?.toInt();
    final can = layer['locked'] != true;
    Future<void> write(int? next) => controller.command('setAttrs', {
      'layers': [layer['id']],
      'patch': {'ghost': next},
    });
    return [
      _line('Ghost', [
        if (ghost != null)
          SizedBox(
            width: EditorMetrics.field,
            child: EditorNumericField(
              key: ValueKey('${layer['id']}:ghost'),
              value: ghost.toDouble(),
              label: 'Ghost delay',
              speed: .25,
              enabled: can && panelCan(controller, 'setAttrs'),
              onPreview: (_) async {},
              // 0 は「無し」。負は先へずらす。
              onCommit: (n) => write(
                n.round() == 0 ? null : n.round().clamp(-(1 << 20), 1 << 20),
              ),
              onFinish: () async {},
              onCancel: () async {},
            ),
          ),
        panelButton(
          ghost == null ? 'Off' : 'On',
          can && panelCan(controller, 'ghost')
              ? () => controller.command('ghost', {'enabled': ghost == null})
              : null,
          selected: ghost != null,
          tooltip: 'The same layer, seen later by a delay',
        ),
      ]),
    ];
  }

  Widget _row(Map<String, dynamic> layer, Map<String, dynamic> row) {
    final value = row['value'];
    return KeyedSubtree(
      key: _rows.putIfAbsent('${row['id']}', () => GlobalKey()),
      child: _line('${row['label'] ?? row['id']}', [
        if (value is List)
          ...List.generate(3, (i) => _cell(layer, row, i))
        else ...[
          const SizedBox(width: EditorMetrics.field),
          const SizedBox(width: EditorMetrics.field),
          _cell(layer, row, 0),
        ],
        _key(layer, row),
      ]),
    );
  }

  /// Key state only. Keys are made with Animate on; moved and removed in the Timeline.
  Widget _key(Map<String, dynamic> layer, Map<String, dynamic>? row) {
    final keys = row == null ? const [] : (row['keys'] as List? ?? const []);
    final mark = row == null
        ? ''
        : row['keyedNow'] == true
        ? '◆'
        : keys.isNotEmpty
        ? '◇'
        : '';
    return SizedBox(
      width: EditorMetrics.row,
      child: Center(
        child: Text(
          mark,
          style: const TextStyle(
            fontSize: EditorMetrics.dense,
            color: EditorTheme.muted,
          ),
        ),
      ),
    );
  }

  Widget _line(String label, List<Widget> children) => Container(
    height: EditorMetrics.row,
    padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
    decoration: BoxDecoration(
      border: Border(bottom: BorderSide(color: EditorTheme.line)),
    ),
    child: Row(
      children: [
        Expanded(
          child: Tooltip(
            message: label,
            child: Text(
              label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: EditorMetrics.font),
            ),
          ),
        ),
        ...children,
      ],
    ),
  );
  Widget _choice(
    String label,
    dynamic value,
    List<MapEntry<dynamic, String>> choices,
    ValueChanged<dynamic>? changed,
  ) => _line(label, [Expanded(child: _dropdown(value, choices, changed))]);

  Widget _dropdown(
    dynamic value,
    List<MapEntry<dynamic, String>> choices,
    ValueChanged<dynamic>? changed,
  ) =>
      EditorChoice<dynamic>(value: value, choices: choices, onChanged: changed);
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) => SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: SizedBox(
        width: box.maxWidth < 280 ? EditorMetrics.s280 : box.maxWidth,
        height: box.maxHeight,
        child: _content(context),
      ),
    ),
  );
  Widget _content(BuildContext context) => AnimatedBuilder(
    animation: Listenable.merge([controller.document, controller.rendered]),
    builder: (context, _) {
      final layer = _active;
      if (layer == null)
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [panelTitle('No selection')],
        );
      final multiple = controller.selectedIds.length > 1;
      final all = panelRows(layer['properties']);
      final used = <String>{};
      Widget transform(String label, List<String> ids) {
        final base = _property(layer, ids[0]);
        used.addAll(ids);
        if (base == null) return const SizedBox.shrink();
        if (label == 'Position')
          return KeyedSubtree(
            key: _rows.putIfAbsent('position', () => GlobalKey()),
            child: _line(label, [
              _cell(layer, base, 0),
              _cell(layer, base, 1),
              _cell(layer, _property(layer, 'position.z'), 0),
              _key(layer, base),
            ]),
          );
        if (label == 'Scale')
          return KeyedSubtree(
            key: _rows.putIfAbsent('scale', () => GlobalKey()),
            child: _line(label, [
              _cell(layer, base, 0),
              _cell(layer, base, 1),
              _cell(layer, _property(layer, 'scale.z'), 0),
              _key(layer, base),
            ]),
          );
        if (label == 'Rotation')
          return KeyedSubtree(
            key: _rows.putIfAbsent('rotation', () => GlobalKey()),
            child: _line(label, [
              _cell(layer, _property(layer, 'rotation.x'), 0),
              _cell(layer, _property(layer, 'rotation.y'), 0),
              _cell(layer, base, 0),
              _key(layer, base),
            ]),
          );
        return _row(layer, base);
      }

      final transformWidgets = layer['kind'] == 'Camera'
          ? [
              transform('Center', ['camera.center']),
              transform('Zoom', ['camera.zoom']),
              transform('Roll', ['camera.roll']),
            ]
          : [
              transform('Position', ['position', 'position.z']),
              transform('Scale', ['scale', 'scale.z']),
              transform('Rotation', ['rotation', 'rotation.x', 'rotation.y']),
              transform('Opacity', ['opacity']),
              transform('Anchor', ['anchor']),
            ];
      final text = panelMap(layer['text']);
      final textProperties = all
          .where(
            (r) =>
                !used.contains(r['id']) &&
                ('${r['id']}'.startsWith('text') ||
                    ['Size', 'Line height', 'Tracking'].contains(r['label'])),
          )
          .toList();
      final remaining = all
          .where(
            (r) =>
                !used.contains(r['id']) &&
                !textProperties.contains(r) &&
                !'${r['id']}'.startsWith('effect'),
          )
          .toList();
      final matte = panelMap(layer['matte']);
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            decoration: BoxDecoration(
              border: Border(
                left: BorderSide(
                  color: EditorTheme.layerColor(layer['id']),
                  width: EditorMetrics.s3,
                ),
                bottom: const BorderSide(color: EditorTheme.line),
              ),
            ),
            padding: const EdgeInsets.symmetric(
              horizontal: EditorMetrics.s6,
              vertical: EditorMetrics.s4,
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  multiple
                      ? '${controller.selectedIds.length} layers'
                      : '${layer['name']}',
                  style: const TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: EditorMetrics.font,
                  ),
                ),
                Text(
                  '${layer['kind']}',
                  style: const TextStyle(
                    fontSize: EditorMetrics.dense,
                    color: EditorTheme.muted,
                  ),
                ),
              ],
            ),
          ),
          _line('Animate', [
            panelButton(
              controller.document.value['animate'] == true ? 'On' : 'Off',
              panelCan(controller, 'animate')
                  ? () => controller.command('animate', {
                      'enabled': controller.document.value['animate'] != true,
                    })
                  : null,
              tooltip:
                  'While on, values you touch become keys at the current time',
              selected: controller.document.value['animate'] == true,
            ),
          ]),
          _line('Property', [
            for (final label in ['X', 'Y', 'Z'])
              SizedBox(
                width: EditorMetrics.field,
                child: Text(
                  label,
                  textAlign: TextAlign.right,
                  style: const TextStyle(fontSize: EditorMetrics.dense),
                ),
              ),
            const SizedBox(
              width: EditorMetrics.row,
              child: Text(
                'Key',
                style: TextStyle(fontSize: EditorMetrics.dense),
              ),
            ),
          ]),
          Expanded(
            child: ListView(
              children: [
                panelTitle(layer['kind'] == 'Camera' ? 'Camera' : 'Transform'),
                ...transformWidgets,
                if (!multiple && layer['kind'] != 'Camera')
                  _line('Anchor', [
                    SizedBox(
                      width: EditorMetrics.s60,
                      height: EditorMetrics.row,
                      child: Row(
                        children: [
                          for (final x in [0.0, .5, 1.0])
                            Expanded(
                              child: Column(
                                children: [
                                  for (final y in [0.0, .5, 1.0])
                                    Expanded(
                                      child: GestureDetector(
                                        onTap: panelCan(controller, 'anchor')
                                            ? () =>
                                                  controller.command('anchor', {
                                                    'layer': layer['id'],
                                                    'xFraction': x,
                                                    'yFraction': y,
                                                  })
                                            : null,
                                        child: Tooltip(
                                          message:
                                              'Anchor $x, $y${panelCan(controller, 'anchor') ? '' : ' · unavailable'}',
                                          child: Container(
                                            margin: const EdgeInsets.all(.5),
                                            color:
                                                panelCan(controller, 'anchor')
                                                ? EditorTheme.ink
                                                : EditorTheme.raised,
                                          ),
                                        ),
                                      ),
                                    ),
                                ],
                              ),
                            ),
                        ],
                      ),
                    ),
                  ]),
                if (layer['kind'] != 'Camera') ...[
                  panelTitle('Layer'),
                  _line('Space', [
                    for (final p in ['2D', '2.5D', '3D'])
                      panelButton(
                        p,
                        panelCan(controller, 'setAttrs')
                            ? () => controller.command('setAttrs', {
                                'layers': controller.selectedIds,
                                'patch': {'projection': p},
                              })
                            : null,
                        selected: layer['projection'] == p,
                      ),
                  ]),
                  _line('Blend', [
                    Expanded(
                      child: panelButton(
                        '${layer['blendMode'] ?? 'Normal'}',
                        () => controller.focusEditing(
                          layer['id'] as int,
                          'blendMode',
                        ),
                      ),
                    ),
                  ]),
                  _choice(
                    'Parent',
                    layer['parent'] ?? -1,
                    [
                      const MapEntry(-1, 'None'),
                      ...controller.layers
                          .where((v) => v['id'] != layer['id'])
                          .map((v) => MapEntry(v['id'], '${v['name']}')),
                    ],
                    panelCan(controller, 'setAttrs')
                        ? (v) => controller.command('setAttrs', {
                            'layers': [layer['id']],
                            'patch': {'parent': v == -1 ? null : v},
                          })
                        : null,
                  ),
                  if (layer['kind'] == 'Group')
                    _line('Group', [
                      panelButton(
                        layer['frozen'] == true ? 'Unfreeze' : 'Freeze',
                        panelCan(controller, 'freeze')
                            ? () => controller.command('freeze', {
                                'layer': layer['id'],
                                'enabled': layer['frozen'] != true,
                              })
                            : null,
                      ),
                    ]),
                  if (layer['kind'] == 'Image')
                    _line('Environment', [
                      panelButton(
                        layer['environment'] == true ? 'On' : 'Off',
                        panelCan(controller, 'setAttrs')
                            ? () => controller.command('setAttrs', {
                                'layers': [layer['id']],
                                'patch': {
                                  'environment': layer['environment'] != true,
                                },
                              })
                            : null,
                        selected: layer['environment'] == true,
                      ),
                    ]),
                  if (layer['ghostable'] == true) ..._ghostLines(layer),
                  _line('Clip to below', [
                    panelButton(
                      layer['clipToBelow'] == true ? 'On' : 'Off',
                      panelCan(controller, 'clip')
                          ? () => controller.command('clip', {
                              'layer': layer['id'],
                            })
                          : null,
                      selected: layer['clipToBelow'] == true,
                    ),
                  ]),
                  if (matte.isNotEmpty && layer['clipToBelow'] != true) ...[
                    panelTitle('Legacy matte'),
                    _choice(
                      'Source',
                      matte['source'],
                      controller.layers
                          .where((v) => v['id'] != layer['id'])
                          .map((v) => MapEntry(v['id'], '${v['name']}'))
                          .toList(),
                      panelCan(controller, 'setMatte')
                          ? (v) => controller.command('setMatte', {
                              'layer': layer['id'],
                              'source': v,
                              'mode': matte['mode'],
                            })
                          : null,
                    ),
                    _choice(
                      'Mode',
                      matte['mode'],
                      const [
                        'Alpha',
                        'AlphaInverted',
                        'Luma',
                        'LumaInverted',
                      ].map((v) => MapEntry(v, v)).toList(),
                      panelCan(controller, 'setMatte')
                          ? (v) => controller.command('setMatte', {
                              'layer': layer['id'],
                              'source': matte['source'],
                              'mode': v,
                            })
                          : null,
                    ),
                  ],
                ],
                if (!multiple && text.isNotEmpty) ...[
                  panelTitle('Text'),
                  _line('Content', [
                    _key(
                      layer,
                      _property(layer, 'content') ??
                          {
                            'id': 'content',
                            'label': 'Content',
                            'keyedNow': panelRows(
                              layer['contentKeys'],
                            ).any((k) => k['frame'] == controller.frame.value),
                          },
                    ),
                  ]),
                  _editingTarget(
                    layer,
                    'content',
                    EditorDraftField(
                      key: ValueKey('content:${layer['id']}'),
                      value: '${text['content'] ?? ''}',
                      label: 'Content',
                      multiline: true,
                      enabled: panelCan(controller, 'setText'),
                      onCommit: (v) => controller.command('setText', {
                        'layer': layer['id'],
                        'content': v,
                      }),
                    ),
                  ),
                  ...textProperties.map((r) => _row(layer, r)),
                ],
                if (!multiple && panelRows(layer['colors']).isNotEmpty) ...[
                  panelTitle('Color'),
                  if (layer['kind'] == 'Shape')
                    _line('Fill', [
                      for (final gradient in [false, true])
                        panelButton(
                          gradient ? 'Gradient' : 'Solid',
                          panelCan(controller, 'setFillMode')
                              ? () => controller.command('setFillMode', {
                                  'layer': layer['id'],
                                  'slot': panelRows(layer['colors'])
                                      .first['slot'],
                                  'gradient': gradient,
                                })
                              : null,
                        ),
                    ]),
                  for (final color in panelRows(layer['colors']))
                    EditorColorRow(
                      controller: controller,
                      layer: layer,
                      color: color,
                    ),
                ],
                for (final row in remaining)
                  if (row['value'] is num || row['value'] is List)
                    _row(layer, row),
                panelTitle('Effects'),
                if (panelRows(layer['effects']).isEmpty)
                  _line(multiple ? 'Multiple layers' : 'No effects', []),
                if (!multiple)
                  for (final effect in panelRows(layer['effects'])) ...[
                    _line('${effect['name'] ?? effect['pluginId']}', [
                      if (effect['placement'] == true)
                        panelButton(
                          'Expand',
                          panelCan(controller, 'expandEffect')
                              ? () => controller.command('expandEffect', {
                                  'layer': layer['id'],
                                  'id': effect['id'],
                                })
                              : null,
                          tooltip: 'Expand copies into layers',
                        ),
                      panelButton(
                        '×',
                        panelCan(controller, 'removeEffect')
                            ? () => controller.command('removeEffect', {
                                'layer': layer['id'],
                                'id': effect['id'],
                              })
                            : null,
                        tooltip: 'Remove effect',
                      ),
                    ]),
                    if (effect['layout'] is Map)
                      ..._placementGrid(layer, effect)
                    else
                      ..._effectRows(layer, panelRows(effect['params'])),
                  ],
              ],
            ),
          ),
        ],
      );
    },
  );
}

class _AxisDrag {
  const _AxisDrag(this.property, this.axis, this.primaryBase, this.values);
  final String property;
  final int axis;
  final double primaryBase;
  final Map<int, dynamic> values;
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

/// A colour of a layer: the swatch opens the Colors desk on it, the hex sets it.
class EditorColorRow extends StatelessWidget {
  const EditorColorRow({
    required this.controller,
    required this.layer,
    required this.color,
  });
  final EditorSession controller;
  final Map<String, dynamic> layer, color;
  @override
  Widget build(BuildContext context) {
    final rgba = (color['rgba'] as List? ?? [0, 0, 0, 1])
        .map((v) => (v as num).toDouble())
        .toList();
    final hex = rgba
        .take(3)
        .map(
          (v) =>
              (v.clamp(0, 1) * 255).round().toRadixString(16).padLeft(2, '0'),
        )
        .join();
    return Row(
      children: [
        SizedBox(
          width: EditorMetrics.s70,
          child: Text(
            '${color['label']}',
            style: const TextStyle(fontSize: EditorMetrics.font),
          ),
        ),
        GestureDetector(
          onTap: panelCan(controller, 'focusColor')
              ? () {
                  controller.browserTab.value = 'Colors';
                  controller.command('focusColor', {
                    'layer': layer['id'],
                    'slot': color['slot'],
                  });
                }
              : null,
          child: Container(
            width: EditorMetrics.row,
            height: EditorMetrics.s16,
            color: Color.fromARGB(
              (rgba[3].clamp(0, 1) * 255).round(),
              (rgba[0].clamp(0, 1) * 255).round(),
              (rgba[1].clamp(0, 1) * 255).round(),
              (rgba[2].clamp(0, 1) * 255).round(),
            ),
          ),
        ),
        Expanded(
          child: EditorDraftField(
            value: '#$hex',
            label: '${color['label']} hex',
            enabled: panelCan(controller, 'setColor'),
            validator: (v) =>
                _parseHex(v) == null ? 'Use three or six hex digits' : null,
            onCommit: (v) => controller.command('setColor', {
              'layer': layer['id'],
              'slot': color['slot'],
              'rgba': [..._parseHex(v)!, rgba[3]],
            }),
          ),
        ),
      ],
    );
  }
}
