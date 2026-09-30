part of '../inspector.dart';

/// What the layer is made of: the Text card, a shape's Fill, and the old
/// Matte still shown while a layer carries one.
mixin _InspectorContentCards on _InspectorControls {
  bool _isTextProperty(Map<String, dynamic> r) =>
      '${r['id']}'.startsWith('text');

  /// A text layer: what it says, then how it is set.
  List<Widget> _text(Map<String, dynamic> layer, Map<String, dynamic> text) {
    // Numbers that take keys stay here; the face, the class being dressed,
    // the character size and the alignment are chosen on the Fonts shelf
    // (a size in px is not animated — Scale is). The font row is a value:
    // the family's name, and pressing it turns the shelf toward this layer.
    final rows = panelRows(layer['properties'])
        .where(_isTextProperty)
        .where((r) => r['id'] != 'text_justify')
        .where((r) => !'${r['id']}'.endsWith('.size'))
        .where((r) => r['kind'] != 'color')
        .toList();
    final family = '${text['fontFamily'] ?? ''}';
    return [
      RichTextEditor(
        key: ValueKey('rich:${layer['id']}'),
        controller: c,
        layer: layer,
        text: text,
      ),
      const SizedBox(height: EditorMetrics.s6),
      EditorTooltip(
        message: 'Font',
        child: EditorPress(
          key: const ValueKey('inspector:font'),
          onTap: _canEdit(layer) && c.supports('setFont')
              ? () => c.focusFont(layer)
              : null,
          child: Container(
            height: EditorMetrics.row,
            padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s6),
            decoration: BoxDecoration(
              color: EditorTheme.of(context).app,
              border: Border.all(color: EditorTheme.of(context).line),
            ),
            child: Row(
              children: [
                Text(
                  'Font',
                  style: TextStyle(color: EditorTheme.of(context).muted),
                ),
                const Spacer(),
                Flexible(
                  child: Text(
                    family,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(color: EditorTheme.of(context).ink),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
      if (rows.isNotEmpty) ...[
        const SizedBox(height: EditorMetrics.s6),
        _cells([for (final r in rows) _Cell(_control(layer, '${r['id']}'))]),
      ],
    ];
  }

  /// A shape's fill: the kind is a mode, the gradient's rows are values.
  /// The colours themselves are property rows and sit with the shape's other
  /// values; the stops' colours live in the gradient editor.
  List<Widget> _colors(Map<String, dynamic> layer) {
    final fillRows = panelRows(layer['properties'])
        .where(
          (r) =>
              '${r['id']}'.startsWith('fill.') &&
              !'${r['id']}'.startsWith('fill.stop.'),
        )
        .toList();
    return [
      GradientInspector(
        key: ValueKey('fill:${layer['id']}'),
        controller: c,
        layer: layer,
        fill: panelMap(layer['fill']),
      ),
      if (fillRows.isNotEmpty)
        _cells([
          for (final r in fillRows)
            _Cell(
              _control(layer, '${r['id']}'),
              wide: _kindOf(r) == _Kind.color,
            ),
        ]),
    ];
  }

  /// The old matte, still shown while a layer carries one and does not clip.
  List<Widget> _matte(Map<String, dynamic> layer, Map<String, dynamic> matte) {
    final others = c.layers.where((v) => v['id'] != layer['id']);
    return [
      _line(_name(Glyph.layers_outlined, 'Source'), [
        _RowCell(
          EditorChoice<dynamic>(
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
          span: 3,
        ),
      ]),
      _line(_name(Glyph.contrast, 'Mode'), [
        _RowCell(
          EditorChoice<dynamic>(
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
          span: 3,
        ),
      ]),
    ];
  }
}
