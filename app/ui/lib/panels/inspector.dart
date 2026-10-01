import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../foundation/metrics.dart';
import '../foundation/color_field.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';
import '../session/read_model.dart';
import 'rich_text_editor.dart';
import 'gradient_inspector.dart';
import '../foundation/glyphs.dart';
import '../foundation/leaves.dart';

part 'inspector/property_style.dart';
part 'inspector/parts.dart';
part 'inspector/grid.dart';
part 'inspector/reading.dart';
part 'inspector/folds.dart';
part 'inspector/writing.dart';
part 'inspector/wells.dart';
part 'inspector/controls.dart';
part 'inspector/layout_card.dart';
part 'inspector/transform_card.dart';
part 'inspector/content_cards.dart';
part 'inspector/effects_card.dart';

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

class _InspectorPanelState extends State<InspectorPanel>
    with
        _InspectorReading,
        _InspectorGrid,
        _InspectorFolds,
        _InspectorWriting,
        _InspectorWells,
        _InspectorControls,
        _InspectorLayoutCard,
        _InspectorTransformCard,
        _InspectorContentCards,
        _InspectorEffectsCard {
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
    c.deskWork.addListener(_deskMoved);
    _deskSeen = (_preferredCellWidth, c.animateFrom);
    _shape = _stampShape();
    _shownId = _shown?['id'] as int?;
  }

  /// The desk feeds this panel its cell width and the Animate default; a
  /// write to any other desk key leaves it still.
  (double, bool)? _deskSeen;

  void _deskMoved() {
    final now = (_preferredCellWidth, c.animateFrom);
    if (now == _deskSeen || !mounted) return;
    _deskSeen = now;
    setState(() {});
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
    final layer = _active;
    if (layer != null) {
      setState(() {
        _closed.remove(
          _transformIds.contains(id)
              ? layer['kind'] == 'Camera'
                    ? 'Camera'
                    : 'Transform'
              : 'Stage',
        );
        for (final effect in panelRows(layer['effects'])) {
          if (panelRows(effect['params']).any((row) => row['id'] == id)) {
            _closed.remove(_effectSection(layer, effect['id']));
            _advancedOpen.value = {..._advancedOpen.value, '${effect['id']}'};
          }
        }
      });
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final target = _rows[id]?.currentContext;
      if (target != null) Scrollable.ensureVisible(target);
      _nodes['$id:0']?.requestFocus();
    });
    WidgetsBinding.instance.ensureVisualUpdate();
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
    c.deskWork.removeListener(_deskMoved);
    _advancedOpen.dispose();
    super.dispose();
  }

  /// Property ids the Transform card already shows.
  static const _transformIds = {
    'position',
    'position.z',
    'scale',
    'scale.z',
    'rotation',
    'rotation.x',
    'rotation.y',
    'depth',
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

  // ---- Identity ----------------------------------------------------------

  Widget _identity(Map<String, dynamic> layer) => Container(
    height: EditorMetrics.bar,
    padding: const EdgeInsets.only(right: EditorMetrics.s6),
    // The layer's own colour as a flat block, as its bar wears it in the
    // Timeline: what the panel edits is told by the panel's head.
    decoration: BoxDecoration(
      color: EditorTheme.of(context).layerColor(layer['id']),
      border: Border(bottom: BorderSide(color: EditorTheme.of(context).line)),
    ),
    child: Row(
      children: [
        const SizedBox(width: EditorMetrics.s6),
        Icon(
          switch ('${layer['kind']}') {
            'Text' => Glyph.text_fields,
            'Shape' => Glyph.pentagon_outlined,
            'Camera' => Glyph.videocam_outlined,
            'Video' => Glyph.movie_outlined,
            'Audio' => Glyph.graphic_eq,
            'Group' => Glyph.folder_outlined,
            _ => Glyph.image_outlined,
          },
          size: EditorMetrics.s14,
          color: EditorTheme.of(context).tabInk,
        ),
        const SizedBox(width: EditorMetrics.s6),
        Expanded(
          child: EditorTooltip(
            message: '${layer['name']}',
            child: Text(
              _multiple ? '${c.selectedIds.length} layers' : '${layer['name']}',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: EditorMetrics.title,
                fontWeight: FontWeight.w600,
                color: EditorTheme.of(context).tabInk,
              ),
            ),
          ),
        ),
        if (!_multiple && panelRows(layer['effects']).isNotEmpty)
          _headGlyph(
            _effectsClosed(layer) ? Glyph.unfold_more : Glyph.unfold_less,
            _effectsClosed(layer) ? 'Expand effects' : 'Collapse effects',
            () {
              final closed = _effectsClosed(layer);
              setState(() {
                for (final effect in panelRows(layer['effects'])) {
                  final id = _effectSection(layer, effect['id']);
                  if (closed) {
                    _closed.remove(id);
                  } else {
                    _closed.add(id);
                  }
                }
              });
            },
            ink: EditorTheme.of(context).tabInk,
          ),
        EditorSwitch(
          on: c.animating,
          glyph: Glyph.diamond_outlined,
          tint: EditorTheme.of(context).keyAccent,
          ink: EditorTheme.of(context).tabInk,
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
  Widget build(BuildContext context) {
    final layer = _active;
    if (layer == null) {
      return ColoredBox(
        color: EditorTheme.of(context).app,
        child: Center(
          child: Text(
            'Select a layer',
            style: TextStyle(
              fontSize: EditorMetrics.title,
              color: EditorTheme.of(context).muted,
            ),
          ),
        ),
      );
    }
    final effects = panelRows(layer['effects']);
    final text = panelMap(layer['text']);
    final matte = panelMap(layer['matte']);
    return LayoutBuilder(
      builder: (context, box) {
        _fit(box.maxWidth);
        return ColoredBox(
          color: EditorTheme.of(context).app,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _identity(layer),
              Expanded(
                child: CustomScrollView(
                  controller: _scroll,
                  slivers: [
                    SliverList.list(
                      // The shared cards keep their seats — Transform first,
                      // World under it — whatever is selected. What a kind
                      // owns (text, fill, matte) comes after, so a change of
                      // selection never pushes Position up or down.
                      children: [
                        if (layer['kind'] == 'Camera')
                          _card(title: 'Camera', children: _camera(layer))
                        else if (layer['kind'] == 'Stage')
                          _card(
                            title: 'Stage',
                            children: [
                              _cells([
                                for (final row in panelRows(
                                  layer['properties'],
                                ))
                                  _Cell(_control(layer, '${row['id']}')),
                              ]),
                            ],
                          )
                        else
                          _card(
                            title: 'Transform',
                            children: _transform(layer),
                          ),
                        if (layer['kind'] != 'Camera')
                          _card(title: 'World', children: _world(layer)),
                        if (_hasLayout(layer))
                          _card(title: 'Layout', children: _layout(layer)),
                        if (!_multiple && text.isNotEmpty)
                          _card(title: 'Text', children: _text(layer, text)),
                        if (!_multiple &&
                            layer['kind'] == 'Shape' &&
                            layer['fill'] is Map)
                          _card(title: 'Fill', children: _colors(layer)),
                        if (!_multiple &&
                            matte.isNotEmpty &&
                            layer['clipToBelow'] != true)
                          _card(title: 'Matte', children: _matte(layer, matte)),
                      ],
                    ),
                    // 凍った層: 効果は焼かれている。灰色にして触れない(DAW の凍った device)。
                    if (!_multiple &&
                        layer['frozen'] == true &&
                        effects.isNotEmpty)
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: EditorMetrics.s6,
                            vertical: EditorMetrics.s4,
                          ),
                          child: Text(
                            'Frozen — effects are baked. Unfreeze to edit.',
                            style: TextStyle(
                              color: EditorTheme.of(context).muted,
                            ),
                          ),
                        ),
                      ),
                    if (!_multiple)
                      SliverOpacity(
                        opacity: layer['frozen'] == true ? 0.45 : 1.0,
                        sliver: SliverIgnorePointer(
                          ignoring: layer['frozen'] == true,
                          sliver: SliverReorderableList(
                            itemCount: effects.length,
                            itemBuilder: (context, i) => _effect(
                              layer,
                              effects[i],
                              index: i,
                              count: effects.length,
                            ),
                            onReorderItem: (from, to) =>
                                c.command('moveEffect', {
                                  'layer': layer['id'],
                                  'id': effects[from]['id'],
                                  'to': to,
                                }),
                          ),
                        ),
                      ),
                    const SliverPadding(
                      padding: EdgeInsets.only(bottom: EditorMetrics.s6),
                    ),
                  ],
                ),
              ),
              EditorZoomBar(
                base: EditorMetrics.cell,
                value: _preferredCellWidth,
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
  }
}
