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
  if (kind == 'vec2' || row['value'] is List) return _Kind.vec2;
  if (row['min'] != null && row['max'] != null) return _Kind.bounded;
  return _Kind.scalar;
}

String? _unitOf(Map<String, dynamic> row) {
  final id = '${row['id']}';
  if (_kindOf(row) == _Kind.angle) return '°';
  if (id == 'opacity') return '%';
  if (id.startsWith('position') || id == 'anchor' || id == 'camera.center') {
    return 'px';
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
    double width = EditorMetrics.field,
    bool fill = false,
  }) {
    final v = row['value'];
    final value = v is List && axis < v.length
        ? v[axis]
        : axis == 0 && v is num
        ? v
        : null;
    if (value is! num) return SizedBox(width: width);
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
        width: width,
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
          unit: unit,
          decimals: percent
              ? 0
              : id.startsWith('position')
              ? 1
              : 2,
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
    width: EditorMetrics.s60,
    child: Text(
      s,
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      style: const TextStyle(fontSize: EditorMetrics.font),
    ),
  );

  Widget _gap() => const SizedBox(width: EditorMetrics.s4);

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
          _well(layer, position, 0, label: 'X'),
          _gap(),
          _well(layer, position, 1, label: 'Y'),
          _gap(),
          if (z != null) _well(layer, z, 0, label: 'Z'),
        ]),
      if (scale != null)
        _line([
          _glyph(Icons.aspect_ratio, 'Scale'),
          _word('Scale'),
          if (_scaleLocked && scaleEven)
            _well(layer, scale, 0, label: 'Scale')
          else ...[
            _well(layer, scale, 0, label: 'X'),
            _gap(),
            _well(layer, scale, 1, label: 'Y'),
          ],
          _gap(),
          EditorSwitch(
            on: _scaleLocked && scaleEven,
            glyph: Icons.link,
            label: 'Keep the shape: one number scales both axes',
            onChanged: (on) async {
              if (on && sv is List && sv.length >= 2 && !scaleEven) {
                await _write(layer, scale, [sv[0], sv[0]], preview: false);
              }
              setState(() => _scaleLocked = on);
            },
          ),
        ]),
      if (rotation != null)
        _line([
          _glyph(Icons.rotate_right, 'Rotation'),
          _word('Rotation'),
          EditorDial(
            degrees: (rotation['value'] as num? ?? 0).toDouble(),
            enabled: _canEdit(layer),
            onBegin: () {},
            onPreview: (d) => _write(layer, rotation, d, preview: true),
            onFinish: () => _finish(false),
            onCancel: () => _finish(true),
          ),
          _gap(),
          _well(layer, rotation, 0, label: 'Rotation'),
          _gap(),
          if (rx != null)
            _well(layer, rx, 0, label: 'Tilt X', width: EditorMetrics.s44),
          _gap(),
          if (ry != null)
            _well(layer, ry, 0, label: 'Tilt Y', width: EditorMetrics.s44),
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
        const Spacer(),
        if (opacity != null) ...[
          _glyph(Icons.opacity, 'Opacity'),
          _well(
            layer,
            opacity,
            0,
            label: 'Opacity',
            fill: true,
            width: EditorMetrics.s70,
          ),
        ],
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
          const SizedBox(width: EditorMetrics.s12),
        ],
        if (layer['ghostable'] == true) ...[
          EditorSwitch(
            on: ghost != null,
            glyph: Icons.blur_on,
            label: 'Ghost: the same layer seen later by a delay',
            onChanged: layer['locked'] != true && panelCan(c, 'ghost')
                ? (on) => c.command('ghost', {'enabled': on})
                : null,
          ),
          const SizedBox(width: EditorMetrics.s12),
        ],
        EditorSwitch(
          on: layer['clipToBelow'] == true,
          glyph: Icons.subdirectory_arrow_right,
          label: 'Clip to the layer below',
          onChanged: layer['locked'] != true && panelCan(c, 'clip')
              ? (_) => c.command('clip', {'layer': layer['id']})
              : null,
        ),
      ]),
    ];
  }

  // ---- Effects: one sheet per effect, controls from the declaration -------

  Widget _effect(Map<String, dynamic> layer, Map<String, dynamic> effect) {
    final params = panelRows(effect['params']);
    final controls = <Widget>[];
    String? section;
    for (final row in params) {
      final here = row['section'] as String?;
      if (here != null && here != section) {
        controls.add(
          Padding(
            padding: const EdgeInsets.only(
              top: EditorMetrics.s4,
              bottom: EditorMetrics.s2,
            ),
            child: Text(
              here,
              style: const TextStyle(
                fontSize: EditorMetrics.dense,
                color: EditorTheme.muted,
              ),
            ),
          ),
        );
      }
      section = here;
      controls.add(_control(layer, row));
    }
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
        Wrap(
          spacing: EditorMetrics.s6,
          runSpacing: EditorMetrics.s4,
          children: controls,
        ),
      ],
    );
  }

  /// A labelled control for one declared param: the word above, the control
  /// under it, sized by its kind.
  Widget _control(Map<String, dynamic> layer, Map<String, dynamic> row) {
    final kind = _kindOf(row);
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
        body = _well(layer, row, 0, fill: true, width: EditorMetrics.s70);
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
        body = _well(layer, row, 0);
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          label,
          style: const TextStyle(
            fontSize: EditorMetrics.dense,
            color: EditorTheme.muted,
          ),
        ),
        const SizedBox(height: EditorMetrics.s2),
        body,
      ],
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
}
