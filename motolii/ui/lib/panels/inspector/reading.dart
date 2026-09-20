part of '../inspector.dart';

/// What the snapshot says, and watching it move: the shown layer, its rows
/// by id, and one pulse per row so a number that changes rebuilds its own
/// well and nothing else.
mixin _InspectorReading on State<InspectorPanel> {
  EditorSession get c => widget.controller;

  final _scroll = ScrollController();

  /// The shown layer and its rows by id, worked out once per status. The
  /// panel asks for the same rows dozens of times while it builds, and
  /// `liveLayers` copies the document every time it is called.
  Map<String, dynamic>? _fromState, _fromRendered, _shown;

  Map<String, Map<String, dynamic>> _byId = const {};

  List<Map<String, dynamic>> _live = const [];

  /// The selected layers other than the shown one, as of the last status; a
  /// well reads these to say when the selection disagrees.
  List<Map<String, dynamic>> _others = const [];

  /// One listenable per row: a status update pushes the rows that moved, so a
  /// number that changes rebuilds its own well and nothing else. The frame of
  /// the panel is rebuilt only when [_stampShape] moves.
  final _pulse = <String, ValueNotifier<Object?>>{};

  Object? _shape;

  int? _shownId;

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
    _others = [
      for (final layer in _live)
        if (layer['id'] != shown?['id'] && ids.contains(layer['id'])) layer,
    ];
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
        [for (final layer in _others) _property(layer, id)?['value']],
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
        'blocksLight',
        'frozen',
        'clipToBelow',
        'anchorFraction',
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

  Map<String, dynamic>? get _active {
    _read();
    return _shown;
  }

  Map<String, dynamic>? _property(Map<String, dynamic> layer, String id) {
    for (final row in _rowsOf[layer] ??= [
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

  /// A layer's rows, flattened once per layer object rather than per well.
  static final _rowsOf = Expando<List<Map<String, dynamic>>>();

  bool get _multiple => c.selectedIds.length > 1;
}
