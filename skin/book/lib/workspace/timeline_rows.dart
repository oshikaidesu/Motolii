// timeline_rows.dart: each row's state as the document has it (hidden, solo, locked, a relation's colours) and the column of their names.
part of 'timeline.dart';

extension _WsTimelineRows on _WsTimelineState {
  // A relation's row: the label colours of the things it moves, in its order (none: a thing's row, one colour).
  List<Color>? _mixOf(Ws ws, String id) {
    final m = ws.inside, t = int.tryParse(id);
    if (m == null || t == null || m.word(t, 'kind') != 'behaviour' || !wsRelations.contains(m.word(t, 'file'))) return null;
    return [for (final s in m.word(t, 'members').split(' ')) if (s.isNotEmpty && s != '_') wsLabel(s)];
  }

  // The eye of a row. Joined: the thing is put aside in the document (visible 0) and is not on the stage; one step.
  bool _isHidden(Ws ws, String id) => ws.layers.any((l) => l.id == id && l.hidden);   // (as last read: no call to the inside while a row is built)

  void _eye(Ws ws, String id) {
    ws.after(ws.inside!.later('set', thing: int.parse(id), name: 'visible', number: _isHidden(ws, id) ? 1 : 0));
  }

  // Joined: solo and lock are the document's (After Effects keeps both in its project): while any thing is solo only
  // those are seen; a locked thing is not changed (the inside refuses, and says so). Else the sample's own sets.
  bool _isSolo(Ws ws, String id) => ws.layers.any((l) => l.id == id && l.solo);

  bool _isLocked(Ws ws, String id) => ws.layers.any((l) => l.id == id && l.locked);

  bool _anySolo(Ws ws) => ws.layers.any((l) => _isSolo(ws, l.id));

  Widget _labels(Ws ws, List<WtRow> rows) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _LabelHeader(count: ws.layers.length, solo: _anySolo(ws)),
      Expanded(
        child: Listener(
          onPointerSignal: _wheel,
          child: WsTwoFingers(
          onPan: (by, _) => _scrollBy(Offset(0, -by.dy)),   // (the labels go up and down with the tracks' rows)
          child: ClipRect(
            child: LayoutBuilder(
              builder: (context, box) {
                final top = _clampV(rows.length);
                return Stack(
                  children: [
                    for (var i = 0; i < rows.length; i++)
                      Positioned(
                        key: ValueKey('ws-tl-label-${ws.layers[i].id}'),
                        left: 0,
                        right: 0,
                        top: i * WtM.row - top,
                        height: WtM.row,
                        // Joined: a row carried up or down is the thing brought nearer or further back, by as many rows as it was carried.
                        child: GestureDetector(
                          supportedDevices: wsPressing,
                          dragStartBehavior: DragStartBehavior.down, // (all of the way it was carried counts, from where the hand came down)
                          onVerticalDragStart: (_) => _lift = 0,
                          onVerticalDragUpdate: (d) => _lift += d.delta.dy,
                          onVerticalDragEnd: (_) {
                                  final by = (_lift / WtM.row).round();
                                  if (by != 0) ws.after(ws.inside!.later('raise', thing: int.parse(ws.layers[i].id), number: -by.toDouble()));
                                },
                          child: _LabelRow(
                            index: i,
                            row: rows[i],
                            hidden: _isHidden(ws, ws.layers[i].id),
                            solo: _isSolo(ws, ws.layers[i].id),
                            onSelect: () => HardwareKeyboard.instance.isShiftPressed
                                ? wsPickRange(ws, ws.layers[i].id)   // (Shift: every row from the last picked to this, After Effects' manner)
                                : HardwareKeyboard.instance.isMetaPressed ? wsPickAlso(ws, ws.layers[i].id) : ws.select(ws.layers[i].id),   // (Cmd: one more, or one less)
                            onEye: () => _eye(ws, ws.layers[i].id),
                            onSolo: () => _flag(ws, 'solo', ws.layers[i].id),
                            onLock: () => _flag(ws, 'lock', ws.layers[i].id),
                          ),
                        ),
                      ),
                  ],
                );
              },
            ),
          ),
          ),
        ),
      ),
    ],
  );
}
