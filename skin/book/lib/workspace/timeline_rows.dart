// timeline_rows.dart: each row's state as the document has it (hidden, solo, locked, a relation's colours) and the column of their names.
part of 'timeline.dart';

extension _WsTimelineRows on _WsTimelineState {
  // A relation's row: the label colours of the things it moves, in its order (none: a thing's row, one colour).
  List<Color>? _mixOf(Ws ws, String id) {
    final m = ws.inside, t = int.tryParse(id);
    if (m == null || t == null || m.word(t, 'kind') != 'behaviour' || !wsRelations.contains(m.word(t, 'file'))) return null;
    return [for (final s in m.word(t, 'members').split(' ')) if (s.isNotEmpty && s != '_') wsLabel(s)];
  }

  // A thing by its row's number and name, as After Effects' Parent column says it (many may share the name).
  String _numbered(Ws ws, String id) {
    final i = ws.layers.indexWhere((l) => l.id == id);
    return i < 0 ? id : '${i + 1} ${ws.layers[i].name}';
  }

  // The eye of a row. Joined: the thing is put aside in the document (visible 0) and is not on the stage; one step.
  bool _isHidden(Ws ws, String id) => ws.layers.any((l) => l.id == id && l.hidden);   // (as last read: no call to the inside while a row is built)

  // How many things a row carried [by] rows goes past: a shut group's row is all that it holds.
  int _places(Ws ws, WsLayer l, int by) {
    final away = ws.foldedAway, seen = [for (final o in _order) if (o.value == null) ws.layers[o.layer].id], at = seen.indexOf(l.id);
    final past = by > 0 ? seen.sublist(at + 1, math.min(at + 1 + by, seen.length)) : seen.sublist(math.max(0, at + by), at);
    return past.fold(0, (n, id) => n + 1 + away.values.where((g) => g == id).length);
  }

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
        child: WsMenu(
        key: _labelMenu,
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
                      if (_order[i].aim case (final said, final other)?)
                        Positioned(
                          key: ValueKey('ws-tl-aim-${ws.layers[_order[i].layer].id}-$said-$other'),
                          left: Shape.s(34),
                          right: Shape.s(8),
                          top: i * WtM.row - top,
                          height: WtM.row,
                          child: GestureDetector(
                            behavior: HitTestBehavior.opaque,
                            onTap: () => ws.select(other),   // (a press goes there: Cavalry's connection list)
                            child: Row(
                              children: [
                                Text(said, maxLines: 1, style: T.label(Grey.g56)),
                                SizedBox(width: Shape.s(5)),
                                Flexible(child: Text(_numbered(ws, other), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g91))),
                              ],
                            ),
                          ),
                        )
                      else if (_order[i].value case final n?)   // (a value's lane: its name, under its layer's, as Ableton names a lane by its control)
                        Positioned(
                          key: ValueKey('ws-tl-lane-${ws.layers[_order[i].layer].id}-$n'),
                          left: Shape.s(34),
                          right: Shape.s(8),
                          top: i * WtM.row - top,
                          height: WtM.row,
                          // (what it aims at, by the card's name then its own: Ableton's Device and Control choosers, Logic's plug-in and parameter; an effect's knob Threshold, not js_am_rays_threshold)
                          child: Row(
                            children: [
                              if (_laneCard(ws, ws.layers[_order[i].layer], n) case final card?) ...[
                                Flexible(child: Text(card, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g56))),
                                SizedBox(width: Shape.s(5)),
                              ],
                              Text(ws.inside?.shown(n)?.label ?? n, maxLines: 1, style: T.label(Grey.g76)),
                            ],
                          ),
                        )
                      else if (ws.layers[_order[i].layer] case final l)
                      Positioned(
                        key: ValueKey('ws-tl-label-${l.id}'),
                        left: 0,
                        right: 0,
                        top: i * WtM.row - top,
                        height: WtM.row,
                        // Joined: a row carried up or down is the thing brought nearer or further back, by as many rows as it was carried.
                        child: GestureDetector(
                          supportedDevices: wsPressing,
                          onSecondaryTapDown: (d) => _labelRight(l, d.globalPosition),
                          dragStartBehavior: DragStartBehavior.down, // (all of the way it was carried counts, from where the hand came down)
                          onVerticalDragStart: (_) => _lift = 0,
                          onVerticalDragUpdate: (d) => _lift += d.delta.dy,
                          onVerticalDragEnd: (_) {
                                  final by = (_lift / WtM.row).round();
                                  if (by != 0) ws.after(ws.inside!.later('raise', thing: int.parse(l.id), number: (by > 0 ? -1 : 1) * _places(ws, l, by).toDouble()));
                                },
                          child: _LabelRow(
                            index: _order[i].layer,   // (its layer's number, not its row's: After Effects' # counts layers, a value's lane none; golf (61))
                            depth: ws.depth(l),
                            row: rows[i],
                            hidden: _isHidden(ws, l.id),
                            solo: _isSolo(ws, l.id),
                            lanes: l.kind == WsKind.group ? ws.groupOpen(l.id) : l.keys.isEmpty && wsTargets(ws, l.id).isEmpty && !l.lined ? null : ws.lanesOpen(l.id),
                            onLanes: () => l.kind == WsKind.group ? ws.turnGroup(l.id) : ws.twirl([l.id]),
                            onSelect: () => HardwareKeyboard.instance.isShiftPressed
                                ? wsPickRange(ws, l.id)   // (Shift: every row from the last picked to this, After Effects' manner)
                                : HardwareKeyboard.instance.isMetaPressed ? wsPickAlso(ws, l.id) : ws.select(l.id),   // (Cmd: one more, or one less)
                            onEye: () => _eye(ws, l.id),
                            onSolo: () => _flag(ws, 'solo', l.id),
                            onLock: () => _flag(ws, 'lock', l.id),
                          ),
                        ),
                      ),
                    for (var i = 0; i < rows.length; i++)
                      if (_order[i].value == null)
                        if (_renameField(i, ws.layers[_order[i].layer].id, i * WtM.row - top) case final f?) f,   // (its name written over it: Cmd+R)
                  ],
                );
              },
            ),
          ),
          ),
        ),
        ),
      ),
    ],
  );
}
