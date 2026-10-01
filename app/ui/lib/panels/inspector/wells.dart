part of '../inspector.dart';

/// One row of the panel: the numeric well, the name column beside it, and
/// the small controls a line can carry (dial, choice, layer picker, menu).
mixin _InspectorWells on _InspectorWriting, _InspectorGrid {
  final _nodes = <String, FocusNode>{};

  final _rows = <String, GlobalKey>{};

  /// The numeric well: one axis of a row, watching that row alone.
  Widget _well(
    Map<String, dynamic> layer,
    String id,
    int axis, {
    String? label,
    double? width,
    bool fill = false,
    String? unit,
    int? decimals,
    String? zeroWord,
  }) {
    final slot = width ?? _wellWidth;
    return _live2([id], () {
      final row = _row(id);
      return row == null
          ? SizedBox(width: slot)
          : _wellBody(
              layer,
              row,
              axis,
              label: label,
              width: slot,
              fill: fill,
              unit: unit,
              decimals: decimals,
              zeroWord: zeroWord,
            );
    });
  }

  Widget _wellBody(
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
    int axis, {
    String? label,
    required double width,
    bool fill = false,
    String? unit,
    int? decimals,
    String? zeroWord,
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
    final mixed =
        _multiple &&
        _live.any((t) {
          if (t['id'] == layer['id'] || !c.selectedIds.contains(t['id'])) {
            return false;
          }
          final other = _property(t, id)?['value'];
          final n = other is List && axis < other.length ? other[axis] : other;
          return n is num && n != value;
        });
    // Opacity is declared without a range; it is 0..1 by meaning.
    final min =
            (row['min'] as num?)?.toDouble() ?? (id == 'opacity' ? 0 : null),
        max = (row['max'] as num?)?.toDouble() ?? (id == 'opacity' ? 1 : null);
    final isScale = id == 'scale' || id == 'scale.z';
    // The row's own rider, unless the line asked for a plain word of its
    // own (cols, rows, px, s) to name the field from the inside.
    final rowUnit = unit ?? (isScale ? '%' : _unitOf(row));
    final percent = isScale || rowUnit == '%' && max == 1;
    // A declared range sets the drag speed; a range open on one side
    // (max = f64::MAX on the native side) is unbounded and drags 1 px = 1.
    final speed = isScale
        ? .01
        : min != null && max != null && (max - min) < 1e9
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
              unit: rowUnit ?? '',
              zeroWord: zeroWord,
              // A narrow well keeps its digits whole rather than clipping them.
              decimals:
                  decimals ??
                  (percent || slotWidth < EditorMetrics.field ? 0 : 2),
              defaultValue: _restOf(row, axis, shownScale),
              tint: _tintOf(row, EditorTheme.of(context)),
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

  Widget _headGlyph(
    IconData icon,
    String tip,
    VoidCallback? onTap, {
    Color? ink,
  }) => _HeadGlyph(icon: icon, tip: tip, onTap: onTap, ink: ink);

  /// One name per row, never two marks for one meaning: the word while it
  /// fits, the glyph when the panel is too narrow for words. The name is a
  /// step quieter than the value it names.
  Widget _named(IconData icon, String id) => _name(icon, panelName(c, id));

  Widget _name([IconData? icon, String? label]) => label == null
      ? const SizedBox.shrink()
      : _wordWidth == 0
      ? EditorTooltip(
          message: label,
          child: Icon(
            icon,
            size: EditorMetrics.s14,
            color: EditorTheme.of(context).muted,
          ),
        )
      : Text(
          label,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: TextStyle(
            fontSize: EditorMetrics.dense,
            color: EditorTheme.of(context).muted,
          ),
        );

  Widget _gap() => const SizedBox(width: _InspectorColumns.gap);

  Widget _line(
    Widget label,
    List<_RowCell> cells, {
    Widget? trailing,
    int divisions = 3,
  }) {
    final used = cells.fold(0, (int sum, cell) => sum + cell.span);
    assert(used <= divisions);
    return Row(
      children: [
        SizedBox(width: _columns.label, child: label),
        for (final (index, cell) in cells.indexed) ...[
          if (index > 0) _gap(),
          SizedBox(
            width: _columns.span(cell.span, divisions: divisions),
            height: cell.tall ? null : _InspectorColumns.rowHeight,
            child: cell.child == null
                ? null
                : cell.tall
                ? Align(
                    alignment: Alignment.centerLeft,
                    heightFactor: 1,
                    child: cell.child,
                  )
                : cell.center
                ? Center(child: cell.child)
                : cell.child,
          ),
        ],
        if (used < divisions) ...[
          if (used > 0) _gap(),
          SizedBox(
            width: _columns.span(divisions - used, divisions: divisions),
          ),
        ],
        _gap(),
        SizedBox(
          width: _InspectorColumns.accessory,
          height: _InspectorColumns.rowHeight,
          child: trailing == null ? null : Center(child: trailing),
        ),
      ],
    );
  }

  /// A choice row as a compact menu, watching that row alone.
  Widget _pick(Map<String, dynamic> layer, String id, {double? width}) =>
      _live2([id], () {
        final row = _row(id);
        final choices = row?['choices'];
        return row == null
            ? SizedBox(width: width)
            : SizedBox(
                width: width,
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
      });

  /// The name column of a Layout line: a glyph, never a word. The CSS word
  /// stays in the tooltip and in the row the well writes.
  Widget _mark(IconData icon, String css) => Align(
    alignment: Alignment.centerLeft,
    child: EditorTooltip(
      message: css,
      child: Icon(
        icon,
        size: EditorMetrics.s14,
        color: EditorTheme.of(context).muted,
      ),
    ),
  );

  /// A one-letter leader where a glyph says nothing: W and H. The CSS word
  /// stays in the tooltip, as the glyph marks do.
  Widget _letter(String letter, String css) => Align(
    alignment: Alignment.centerLeft,
    child: EditorTooltip(
      message: css,
      child: Text(
        letter,
        style: TextStyle(
          fontSize: EditorMetrics.font,
          color: EditorTheme.of(context).muted,
        ),
      ),
    ),
  );

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

  Widget _cells(List<_Cell> cells) => Wrap(
    spacing: _InspectorColumns.gap,
    runSpacing: _InspectorColumns.gap,
    children: [
      for (final cell in cells)
        SizedBox(width: cell.wide ? _cardWidth : _cellWidth, child: cell.child),
    ],
  );

  /// A property that points at another layer (the camera's target, an
  /// effect's matte): one list of the other layers, `None` on top.
  Widget _layerPicker(Map<String, dynamic> layer, String id) =>
      _live2([id], () {
        final current = _row(id)?['value'];
        return EditorChoice<dynamic>(
          value: current is num ? current.toInt() : 0,
          choices: [
            const MapEntry(0, 'None'),
            for (final v
                in (c.state['layers'] as List? ?? const []).whereType<Map>())
              if (v['id'] != layer['id']) MapEntry(v['id'], '${v['name']}'),
          ],
          onChanged: _canEdit(layer)
              ? (v) => c.command('setProperty', {
                  'layer': layer['id'],
                  'property': id,
                  'value': v,
                })
              : null,
        );
      });

  /// The cell's own menu: back to its rest value, and key / unkey.
  Future<void> _cellMenu(
    BuildContext context,
    Offset at,
    Map<String, dynamic> layer,
    Map<String, dynamic> row,
  ) async {
    final rest = row['default'];
    final chosen = await showEditorMenu<String>(context, at, [
      EditorMenuItem<String>(
        value: 'rest',
        enabled: rest is num && row['value'] is num && _canEdit(layer),
        child: const Text('Reset'),
      ),
      EditorMenuItem<String>(
        value: 'key',
        enabled: panelCan(c, 'toggleKey') && _canEdit(layer),
        child: Text(row['keyedNow'] == true ? 'Remove key' : 'Key this frame'),
      ),
    ]);
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
}
