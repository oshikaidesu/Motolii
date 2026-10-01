part of '../inspector.dart';

/// The Layout card: the parent's grid, gap, alignment and sizing, the
/// child's own lines, and the rest of the layout rows behind Advanced.
mixin _InspectorLayoutCard on _InspectorControls, _InspectorFolds {
  // ---- Layout ------------------------------------------------------------

  /// The one place that says which layer authors the parent lines (grid,
  /// gap, alignment, sizing, transition): a Group, the source that lays
  /// its children out. A split Text becomes one later and follows the same
  /// rule.
  static bool _isGroup(Map<String, dynamic> layer) => layer['kind'] == 'Group';

  /// A layer native hands the item rows to: it sits in a laid-out group.
  bool _isLaidOut(Map<String, dynamic> layer) =>
      _property(layer, 'layout.position_type') != null;

  /// The Layout card shows for a Group or a laid-out child, nothing else.
  bool _hasLayout(Map<String, dynamic> layer) =>
      !_multiple && (_isGroup(layer) || _isLaidOut(layer));

  int _choice(String id) => ((_row(id)?['value'] as num?) ?? 0).round();

  /// The 3×3 alignment box as the pad's snaps.
  static const _nineSnaps = [
    Offset(0, 0),
    Offset(.5, 0),
    Offset(1, 0),
    Offset(0, .5),
    Offset(.5, .5),
    Offset(1, .5),
    Offset(0, 1),
    Offset(.5, 1),
    Offset(1, 1),
  ];

  /// The rows whose value changes the shape of the Layout card (which lines
  /// show, which glyph is lit); the wells watch their own.
  static const _layoutShape = [
    'layout.display',
    'layout.justify_content',
    'layout.align_items',
    'layout.position_type',
    'layout.horizontal_sizing',
    'layout.vertical_sizing',
  ];

  Widget _layoutLine(
    String name,
    Widget label,
    List<_RowCell> cells, {
    Widget? trailing,
    int divisions = 3,
  }) => KeyedSubtree(
    key: ValueKey('layout:$name'),
    child: _line(label, cells, trailing: trailing, divisions: divisions),
  );

  /// The parent's lines, the child's, and the rest of the layout rows
  /// behind the Advanced fold, unchanged (a Flex document keeps its rows
  /// readable there).
  List<Widget> _layout(Map<String, dynamic> layer) => [
    _live2(_layoutShape, () {
      final used = <String>{};
      final lines = <Widget>[
        if (_isGroup(layer)) ..._layoutParent(layer, used),
        if (_isLaidOut(layer)) ..._layoutChild(layer, used),
      ];
      final rest = [
        for (final row in panelRows(layer['properties']))
          if ('${row['id']}'.startsWith('layout.') &&
              !used.contains('${row['id']}'))
            row,
      ];
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          ...lines,
          if (rest.isNotEmpty)
            _AdvancedFold(
              opened: _advancedOpen,
              id: 'layout:${layer['id']}',
              builder: () => _cells([
                for (final row in rest) _Cell(_control(layer, '${row['id']}')),
              ]),
            ),
        ],
      );
    }),
  ];

  /// Laying out is always a grid: columns × rows (one row = a line across,
  /// one column = a line down). The switch on the mark is Display itself.
  List<Widget> _layoutParent(Map<String, dynamic> layer, Set<String> used) {
    final can = _canEdit(layer);
    final display = _choice('layout.display');
    used.addAll(['layout.display', 'layout.grid_columns', 'layout.grid_rows']);
    final lines = <Widget>[
      _layoutLine(
        'grid',
        Align(
          alignment: Alignment.centerLeft,
          child: EditorSwitch(
            compact: true,
            on: display != 0,
            glyph: Glyph.grid_on,
            label: 'display: grid',
            onChanged: can
                ? (on) => _writeChoices(layer, {
                    'layout.display': on ? 2 : 0,
                  }, preview: false)
                : null,
          ),
        ),
        [
          _RowCell(
            _well(
              layer,
              'layout.grid_columns',
              0,
              label: 'Columns',
              unit: 'cols',
              decimals: 0,
            ),
          ),
          _RowCell(
            _well(
              layer,
              'layout.grid_rows',
              0,
              label: 'Rows',
              unit: 'rows',
              decimals: 0,
              zeroWord: 'auto',
            ),
          ),
          _RowCell(null),
        ],
      ),
    ];
    if (display == 0) return lines;
    used.addAll(['layout.gap', 'layout.padding']);
    lines.add(
      _layoutLine(
        'gap',
        _mark(Glyph.horizontal_distribute, 'gap'),
        [
          _RowCell(
            _well(
              layer,
              'layout.gap',
              0,
              label: 'Gap',
              unit: 'px',
              decimals: 0,
            ),
          ),
          _RowCell(
            _well(
              layer,
              'layout.padding',
              0,
              label: 'Padding X',
              unit: 'px',
              decimals: 0,
            ),
          ),
          _RowCell(
            _well(
              layer,
              'layout.padding',
              1,
              label: 'Padding Y',
              unit: 'px',
              decimals: 0,
            ),
          ),
        ],
        trailing: Icon(
          Glyph.padding,
          size: EditorMetrics.s14,
          color: EditorTheme.of(context).muted,
        ),
      ),
    );
    lines.add(_alignLine(layer, used));
    lines.addAll(_sizingLines(layer, used));
    if (_row('layout.transition_duration') != null) {
      used.addAll(['layout.transition_duration', 'layout.transition_easing']);
      lines.add(
        _layoutLine('transition', _mark(Glyph.timer, 'transition'), [
          _RowCell(
            _well(
              layer,
              'layout.transition_duration',
              0,
              label: 'Duration',
              unit: 's',
            ),
          ),
          _RowCell(
            _pick(layer, 'layout.transition_easing', width: _columns.span(2)),
            span: 2,
          ),
        ]),
      );
    }
    return lines;
  }

  /// Justify Content (across) and Align Items (down) as one 3×3 box on the
  /// pad; the sent value is the nearest of the nine, the dot follows the
  /// pointer until let go.
  Widget _alignLine(Map<String, dynamic> layer, Set<String> used) {
    const justifyId = 'layout.justify_content', alignId = 'layout.align_items';
    used.addAll([justifyId, alignId]);
    final justify = _choice(justifyId), align = _choice(alignId);
    // Start / End / Center / Between / Around / Evenly across.
    const acrossOf = [0.0, 1.0, 0.5, 0.0, 0.5, 1.0];
    // Stretch (the centre) / Start / End / Center down.
    const downOf = [0.5, 0.0, 1.0, 0.5];
    Offset? sent;
    return _layoutLine(
      'align',
      _mark(Glyph.center_focus_weak, 'justify-content · align-items'),
      [
        _RowCell(
          EditorPad(
            x: acrossOf[justify.clamp(0, 5)],
            y: downOf[align.clamp(0, 3)],
            unit: true,
            snaps: _nineSnaps,
            enabled: _canEdit(layer),
            tint: EditorTheme.of(context).spatial,
            // Two rows tall, not three: the nine places read at this size and
            // the card keeps its density (the craft ledger's 24 px floor, 28
            // base — a 44 px box holds both with room for the marks).
            size: EditorMetrics.s44,
            onBegin: () {
              sent = null;
              _begin();
            },
            onPreview: (x, y) async {
              final at = Offset(x, y);
              if (at == sent) return;
              sent = at;
              await _writeChoices(
                layer,
                {
                  justifyId: [0, 2, 1][(x * 2).round()],
                  alignId: [1, 3, 2][(y * 2).round()],
                },
                preview: true,
                always: true,
              );
            },
            onFinish: () => _finish(false),
            onCancel: () => _finish(true),
          ),
          span: 3,
          tall: true,
        ),
      ],
    );
  }

  /// Hug / Fill / Fixed as three glyphs (arrows in, arrows out, a lock), and
  /// the number, which counts under Fixed alone and greys out otherwise.
  List<Widget> _sizingLines(Map<String, dynamic> layer, Set<String> used) {
    Widget line(String key, bool across, String sizingId, String sizeId) {
      used.addAll([sizingId, sizeId]);
      final sizing = _choice(sizingId);
      final fixed = sizing == 2;
      return _layoutLine(key, _letter(across ? 'W' : 'H', key), [
        _RowCell(_pick(layer, sizingId)),
        _RowCell(
          IgnorePointer(
            ignoring: !fixed,
            child: Opacity(
              opacity: fixed ? 1 : .45,
              child: _well(layer, sizeId, 0, unit: 'px', decimals: 0),
            ),
          ),
        ),
        _RowCell(null),
      ]);
    }

    return [
      if (_row('layout.horizontal_sizing') != null)
        line('width', true, 'layout.horizontal_sizing', 'layout.width'),
      if (_row('layout.vertical_sizing') != null)
        line('height', false, 'layout.vertical_sizing', 'layout.height'),
    ];
  }

  /// The child's lines: out of the flow or not, its size, and its cell.
  List<Widget> _layoutChild(Map<String, dynamic> layer, Set<String> used) {
    final can = _canEdit(layer);
    used.add('layout.position_type');
    final lines = <Widget>[
      _layoutLine(
        'ignore',
        _mark(Glyph.filter_center_focus, 'position: absolute'),
        [
          _RowCell(
            EditorSwitch(
              compact: _wellWidth < EditorSwitch.minExpandedWidth,
              on: _choice('layout.position_type') == 1,
              glyph: Glyph.filter_center_focus,
              label: 'Ignore layout (position: absolute)',
              onChanged: can
                  ? (on) => _writeChoices(layer, {
                      'layout.position_type': on ? 1 : 0,
                    }, preview: false)
                  : null,
            ),
            center: true,
          ),
          _RowCell(null),
          _RowCell(null),
        ],
      ),
      ..._sizingLines(layer, used),
    ];
    if (_row('layout.column_start') != null) {
      const cell = [
        ('layout.column_start', 'grid-column-start'),
        ('layout.row_start', 'grid-row-start'),
        ('layout.column_span', 'grid-column span'),
        ('layout.row_span', 'grid-row span'),
      ];
      used.addAll([for (final (id, _) in cell) id]);
      lines.add(
        _layoutLine('cell', _mark(Glyph.grid_on, 'grid-area'), [
          for (final (id, label) in cell)
            _RowCell(
              _well(
                layer,
                id,
                0,
                label: label,
                width: _columns.division(4),
                decimals: 0,
              ),
            ),
        ], divisions: 4),
      );
    }
    return lines;
  }
}
