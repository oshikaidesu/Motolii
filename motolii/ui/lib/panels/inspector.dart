import '../session/read_model.dart';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/panel_controls.dart';

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
  List<Map<String, dynamic>> get _layers {
    final rendered = controller.rendered.value;
    return rendered['frame'] == controller.frame.value &&
            rendered['documentRevision'] == controller.state['documentRevision']
        ? panelRows(rendered['layers'])
        : controller.layers;
  }

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
    final base = current is List
        ? (current[axis] as num).toDouble()
        : (current as num).toDouble();
    for (final target in _layers.where(
      (v) => controller.selectedIds.contains(v['id']),
    )) {
      if (target['locked'] == true) continue;
      final row = _property(target, '$id');
      if (row == null) continue;
      dynamic next = row['value'];
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
    if (row == null) return const SizedBox(width: 52);
    final v = row['value'];
    final value = v is List && axis < v.length
        ? v[axis]
        : axis == 0 && v is num
        ? v
        : null;
    if (value is! num) return const SizedBox(width: 52);
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
        : id == 'scale' || id == 'opacity'
        ? .005
        : id.startsWith('rotation')
        ? .5
        : 1.0;
    return _editingTarget(
      layer,
      id,
      SizedBox(
        width: 52,
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
          onFinish: () => controller.command('commitPreview'),
          onCancel: () => controller.command('cancelPreview'),
        ),
      ),
    );
  }

  Widget _row(Map<String, dynamic> layer, Map<String, dynamic> row) {
    final value = row['value'];
    return KeyedSubtree(
      key: _rows.putIfAbsent('${row['id']}', () => GlobalKey()),
      child: _line('${row['label'] ?? row['id']}', [
        if (value is List)
          ...List.generate(3, (i) => _cell(layer, row, i))
        else ...[
          const SizedBox(width: 52),
          const SizedBox(width: 52),
          _cell(layer, row, 0),
        ],
        _key(layer, row),
      ]),
    );
  }

  Widget _key(Map<String, dynamic> layer, Map<String, dynamic>? row) =>
      SizedBox(
        width: 20,
        child: row == null
            ? null
            : panelButton(
                row['keyedNow'] == true ? '◆' : '◇',
                panelCan(controller, 'toggleKey') && layer['locked'] != true
                    ? () => controller.command('toggleKey', {
                        'layer': layer['id'],
                        'property': row['id'],
                      })
                    : null,
                tooltip: 'Toggle ${row['label']} keyframe at current time',
              ),
      );
  Widget _line(String label, List<Widget> children) => Container(
    height: 20,
    padding: const EdgeInsets.symmetric(horizontal: 6),
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
              style: const TextStyle(fontSize: 11),
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
  ) => _line(label, [
    Expanded(
      child: SizedBox(
        height: 20,
        child: DropdownButtonHideUnderline(
          child: DropdownButton<dynamic>(
            value: choices.any((e) => e.key == value) ? value : null,
            isExpanded: true,
            isDense: true,
            style: const TextStyle(fontSize: 11, color: EditorTheme.ink),
            dropdownColor: EditorTheme.panel,
            items: choices
                .map(
                  (e) => DropdownMenuItem<dynamic>(
                    value: e.key,
                    child: Text(
                      e.value,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                )
                .toList(),
            onChanged: changed,
          ),
        ),
      ),
    ),
  ]);
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) => SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: SizedBox(
        width: box.maxWidth < 280 ? 280 : box.maxWidth,
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
              transform('Scale', ['scale']),
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
                  width: 3,
                ),
                bottom: const BorderSide(color: EditorTheme.line),
              ),
            ),
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  multiple
                      ? '${controller.selectedIds.length} layers'
                      : '${layer['name']}',
                  style: const TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 11,
                  ),
                ),
                Text(
                  '${layer['kind']}',
                  style: const TextStyle(
                    fontSize: 10,
                    color: EditorTheme.muted,
                  ),
                ),
              ],
            ),
          ),
          _line('Property', [
            for (final label in ['X', 'Y', 'Z'])
              SizedBox(
                width: 52,
                child: Text(
                  label,
                  textAlign: TextAlign.right,
                  style: const TextStyle(fontSize: 10),
                ),
              ),
            const SizedBox(
              width: 20,
              child: Text('Key', style: TextStyle(fontSize: 10)),
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
                      width: 60,
                      height: 20,
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
                    _ColorRow(
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
                    for (final row in panelRows(effect['params']))
                      _row(layer, row),
                  ],
              ],
            ),
          ),
        ],
      );
    },
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

class _ColorRow extends StatelessWidget {
  const _ColorRow({
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
          width: 70,
          child: Text(
            '${color['label']}',
            style: const TextStyle(fontSize: 11),
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
            width: 20,
            height: 16,
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
