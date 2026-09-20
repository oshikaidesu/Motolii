part of '../inspector.dart';

/// One declared param as one labelled control, picked by its kind — and a
/// declared pair as one point: a pad with the two numbers beside it.
mixin _InspectorControls on _InspectorWells {
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
        _CellLabel(label, hero: hero),
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            EditorPad(
              x: (xRow['value'] as num).toDouble(),
              y: (yRow['value'] as num).toDouble(),
              enabled: _canEdit(layer),
              tint: EditorTheme.of(context).spatial,
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
      case _Kind.layer:
        body = SizedBox(
          width: _cellWidth,
          child: _layerPicker(layer, '${row['id']}'),
        );
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
              tint: EditorTheme.of(context).angle,
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
        body = SizedBox(
          width: EditorMetrics.row,
          child: EditorColorField(
            key: ValueKey('color:${layer['id']}:${row['id']}'),
            value: Color.from(
              red: rgba[0].toDouble(),
              green: rgba[1].toDouble(),
              blue: rgba[2].toDouble(),
              alpha: rgba.length > 3 ? rgba[3].toDouble() : 1,
            ),
            label: label,
            enabled: _canEdit(layer),
            onFocus: !c.supports('focusColor')
                ? null
                : () => c.focusColor({
                    'layer': layer['id'],
                    'property': row['id'],
                  }),
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
              _SeedRoll(
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
            _CellLabel(label, hero: hero),
            body,
          ],
        ),
      ),
    );
  }
}
