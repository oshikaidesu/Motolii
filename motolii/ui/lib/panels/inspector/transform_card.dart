part of '../inspector.dart';

/// Where the layer is and how it meets the scene: Transform, the Camera
/// card that replaces it, and World (space, parent, blend, the flags).
mixin _InspectorTransformCard on _InspectorControls {
  // ---- Transform ---------------------------------------------------------

  List<Widget> _transform(Map<String, dynamic> layer) {
    final scaleEven = _scaleEven();
    final anchor = layer['anchorFraction'];
    return [
      if (_row('position') != null)
        _line(_named(Glyph.open_with, 'position'), [
          _RowCell(_well(layer, 'position', 0, label: 'X')),
          _RowCell(_well(layer, 'position', 1, label: 'Y')),
          _RowCell(
            _row('position.z') == null
                ? null
                : _well(layer, 'position.z', 0, label: 'Z'),
          ),
        ]),
      if (_row('scale') != null)
        _line(
          _named(Glyph.aspect_ratio, 'scale'),
          [
            _RowCell(
              _well(
                layer,
                'scale',
                0,
                label: _scaleLocked && scaleEven ? 'Scale' : 'X',
              ),
            ),
            _RowCell(
              _scaleLocked && scaleEven
                  ? null
                  : _well(layer, 'scale', 1, label: 'Y'),
            ),
            _RowCell(
              _row('scale.z') == null
                  ? null
                  : _well(layer, 'scale.z', 0, label: 'Z'),
            ),
          ],
          trailing: EditorSwitch(
            on: _scaleLocked,
            glyph: Glyph.link,
            compact: true,
            label: 'Keep the shape: one number scales both axes',
            onChanged: _canEdit(layer)
                ? (on) => setState(() => _scaleLocked = on)
                : null,
          ),
        ),
      if (_row('rotation') != null)
        _line(_named(Glyph.rotate_right, 'rotation'), [
          _RowCell(_well(layer, 'rotation', 0, label: 'Rotation')),
          _RowCell(
            _row('rotation.x') == null ? null : _well(layer, 'rotation.x', 0),
          ),
          _RowCell(
            _row('rotation.y') == null ? null : _well(layer, 'rotation.y', 0),
          ),
        ], trailing: _dial(layer, 'rotation')),
      // Depth turns the flat picture into a body, so it only means something
      // once the layer has left 2D; scale Z then has something to scale.
      if (_row('depth') != null && layer['projection'] != '2D')
        _line(_named(Glyph.view_in_ar, 'depth'), [
          _RowCell(_well(layer, 'depth', 0, label: 'Depth')),
          _RowCell(null),
          _RowCell(null),
        ]),
      if (_row('opacity') != null)
        _line(_named(Glyph.opacity, 'opacity'), [
          _RowCell(
            _well(
              layer,
              'opacity',
              0,
              label: 'Opacity',
              fill: true,
              width: _columns.span(2),
            ),
            span: 2,
          ),
          _RowCell(null),
        ]),
      _line(_named(Glyph.center_focus_weak, 'anchor'), [
        _RowCell(
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
          span: 3,
          tall: true,
        ),
      ]),
    ];
  }

  /// A camera layer authors Center, Zoom and Roll instead of a transform.
  List<Widget> _camera(Map<String, dynamic> layer) {
    return [
      if (_row('camera.center') != null)
        _line(_named(Glyph.center_focus_strong, 'camera.center'), [
          _RowCell(_well(layer, 'camera.center', 0, label: 'X')),
          _RowCell(_well(layer, 'camera.center', 1, label: 'Y')),
          _RowCell(null),
        ]),
      if (_row('camera.target.z') != null)
        _line(_named(Glyph.center_focus_weak, 'camera.target.z'), [
          _RowCell(_well(layer, 'camera.target.z', 0, label: 'Z')),
          _RowCell(null),
          _RowCell(null),
        ]),
      if (_row('camera.target') != null)
        _line(_named(Glyph.gps_fixed, 'camera.target'), [
          _RowCell(_layerPicker(layer, 'camera.target'), span: 3),
        ]),
      if (_row('camera.orbit') != null)
        _line(_named(Glyph.threesixty, 'camera.orbit'), [
          _RowCell(_well(layer, 'camera.orbit', 0, label: 'Pitch')),
          _RowCell(_well(layer, 'camera.orbit', 1, label: 'Yaw')),
          _RowCell(null),
        ]),
      if (_row('camera.distance') != null)
        _line(_named(Glyph.straighten, 'camera.distance'), [
          _RowCell(_well(layer, 'camera.distance', 0, label: 'Scale')),
          _RowCell(null),
          _RowCell(null),
        ]),
      if (_row('camera.zoom') != null)
        _line(_named(Glyph.zoom_in, 'camera.zoom'), [
          _RowCell(_well(layer, 'camera.zoom', 0, label: 'Zoom')),
          _RowCell(null),
          _RowCell(null),
        ]),
      if (_row('camera.roll') != null)
        _line(
          _named(Glyph.rotate_right, 'camera.roll'),
          [
            _RowCell(_well(layer, 'camera.roll', 0, label: 'Roll')),
            _RowCell(null),
            _RowCell(null),
          ],
          trailing: _dial(layer, 'camera.roll', EditorTheme.of(context).angle),
        ),
    ];
  }

  // ---- World -------------------------------------------------------------

  List<Widget> _world(Map<String, dynamic> layer) {
    final can = panelCan(c, 'setAttrs') && layer['locked'] != true;
    final ghost = (layer['ghost'] as num?)?.toInt();
    final flagColumns = layer['kind'] == 'Image' && layer['ghostable'] == true
        ? 4
        : 3;
    return [
      _line(_name(Glyph.view_in_ar_outlined, 'Space'), [
        _RowCell(
          Row(
            children: [
              for (final p in ['2D', '2.5D', '3D'])
                _SpaceChoice(
                  projection: p,
                  selected: layer['projection'] == p,
                  onPick: can
                      ? () => c.command('setAttrs', {
                          'layers': c.selectedIds,
                          'patch': {'projection': p},
                        })
                      : null,
                ),
            ],
          ),
          span: 3,
        ),
      ]),
      _line(_name(Glyph.account_tree_outlined, 'Parent'), [
        _RowCell(
          EditorChoice<dynamic>(
            key: const ValueKey('inspector:parent'),
            value: layer['parent'] ?? -1,
            choices: [
              const MapEntry(-1, 'None'),
              for (final v
                  in (c.state['layers'] as List? ?? const []).whereType<Map>())
                if (v['id'] != layer['id']) MapEntry(v['id'], '${v['name']}'),
            ],
            onChanged: can
                ? (v) => c.command('setAttrs', {
                    'layers': [layer['id']],
                    'patch': {'parent': v == -1 ? null : v},
                  })
                : null,
          ),
          span: 3,
        ),
      ]),
      _line(_name(Glyph.layers_outlined, 'Blend'), [
        _RowCell(
          EditorButton(
            '${layer['blendMode'] ?? 'Normal'}',
            () => c.focusEditing(layer['id'] as int, 'blendMode'),
            key: const ValueKey('inspector:blend'),
            tooltip: 'Blend mode (opens the Blend desk)',
          ),
          span: 3,
        ),
      ]),
      _line(_name(), [
        _RowCell(
          EditorSwitch(
            compact:
                _columns.division(flagColumns) < EditorSwitch.minExpandedWidth,
            on: layer['blocksLight'] == true,
            glyph: Glyph.wb_shade,
            label: 'Blocks light: casts this layer\'s shadow and colored light',
            onChanged: can
                ? (on) => c.command('setAttrs', {
                    'layers': [layer['id']],
                    'patch': {'blocksLight': on},
                  })
                : null,
          ),
        ),
        if (layer['kind'] == 'Image') ...[
          _RowCell(
            EditorSwitch(
              compact:
                  _columns.division(flagColumns) <
                  EditorSwitch.minExpandedWidth,
              on: layer['environment'] == true,
              glyph: Glyph.wb_sunny_outlined,
              label: 'Environment: this image lights and surrounds the scene',
              onChanged: can
                  ? (on) => c.command('setAttrs', {
                      'layers': [layer['id']],
                      'patch': {'environment': on},
                    })
                  : null,
            ),
          ),
        ],
        if (layer['ghostable'] == true) ...[
          _RowCell(
            EditorSwitch(
              compact:
                  _columns.division(flagColumns) <
                  EditorSwitch.minExpandedWidth,
              on: ghost != null,
              glyph: Glyph.blur_on,
              label: 'Ghost: the same layer seen later by a delay',
              onChanged: layer['locked'] != true && panelCan(c, 'ghost')
                  ? (on) => c.command('ghost', {'enabled': on})
                  : null,
            ),
          ),
        ],
        _RowCell(
          EditorSwitch(
            compact:
                _columns.division(flagColumns) < EditorSwitch.minExpandedWidth,
            on: layer['clipToBelow'] == true,
            glyph: Glyph.subdirectory_arrow_right,
            label: 'Clip to the layer below',
            onChanged: layer['locked'] != true && panelCan(c, 'clip')
                ? (_) => c.command('clip', {'layer': layer['id']})
                : null,
          ),
        ),
      ], divisions: flagColumns),
      // Freeze は旗ではなく状態(DAW の Freeze Track): 自分の行。docs/freeze-and-flatten.md
      if (layer['kind'] != 'Camera')
        _line(_name(Glyph.ac_unit, 'Freeze'), [
          _RowCell(
            EditorSwitch(
              compact: _wellWidth < EditorSwitch.minExpandedWidth,
              on: layer['frozen'] == true,
              glyph: Glyph.ac_unit,
              label: 'Freeze: bake the picture; source and effects stay as they are until unfrozen',
              onChanged: panelCan(c, 'freeze')
                  ? (on) => c.command('freeze', {
                      'layer': layer['id'],
                      'enabled': on,
                    })
                  : null,
            ),
            center: true,
          ),
          _RowCell(null),
          _RowCell(null),
        ]),
    ];
  }
}
