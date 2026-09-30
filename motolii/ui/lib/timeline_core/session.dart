import 'dart:math' as math;

import 'package:flutter/foundation.dart';

import '../session/editor_session.dart';
import 'geometry.dart';
import 'layout.dart';
import 'semantics.dart';

/// The Timeline as a person works it, with no screen in it: which rows show (lanes opened, groups folded), which span
/// of time and which rows are in view, and the gesture in flight (what was pressed, how far it has moved, where a row
/// or an asset would land, the marquee) — in frames and rows. Any skin reads it and draws it; a skin decides what is
/// under its pointer and says so here in the same terms. One per document session, so a skin can be thrown away,
/// detached or swapped and the Timeline stays where it was.
class TimelineSession extends ChangeNotifier {
  TimelineSession._(this.c) {
    c.slice('timelineSession', _watched).addListener(relane);
    relane();
  }

  static final _all = Expando<TimelineSession>();
  static TimelineSession of(EditorSession c) => _all[c] ??= TimelineSession._(c);

  final EditorSession c;
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'selectedKeys', 'durationFrames', 'fps', 'fpsNum', 'fpsDen', 'markers', 'capabilities'];

  bool has(String op) => c.supports(op);
  int get fps => (c.state['fps'] as num? ?? 30).round().clamp(1, 1000);
  int get duration => (c.state['durationFrames'] as num? ?? 1).toInt();
  List<Map<String, dynamic>> get selectedKeys => EditorSession.maps(c.state['selectedKeys']);

  // ---- rows ------------------------------------------------------------------------------------------------------
  final expanded = <int>{}, allProperties = <int>{}, collapsed = <int>{};

  /// The rows in order: layers (groups with their children when open), each layer's property lanes when open.
  List<TrackRow> rows = [];
  LaneLayout _layout = LaneLayout([], {}, {}, {});

  /// Rows are laid out in row units (one row = 1), so a row's top is its index.
  static const _unit = TimelineGeometry(rowHeight: 1, rulerHeight: 0, indentStep: 1, disclosureWidth: 0, nameWidth: 0, nameWidthMin: 0, nameWidthMax: 1e9);

  void relane() {
    final live = c.layers.map((l) => (l['id'] as num).toInt()).toSet();
    expanded.retainAll(live);
    allProperties.retainAll(live);
    collapsed.retainAll(live);
    _layout = LaneLayout(c.layers, expanded, allProperties, collapsed, geometry: _unit);
    rows = _layout.rows;
    _clampView();
    notifyListeners();
  }

  void toggleLanes(int row) {
    final r = rows[row];
    allProperties.remove(r.id);
    expanded.contains(r.id) ? expanded.remove(r.id) : expanded.add(r.id);
    relane();
  }

  void showLanes(int row, {required bool all}) {
    final id = rows[row].id;
    expanded.add(id);
    all ? allProperties.add(id) : allProperties.remove(id);
    relane();
  }

  void hideLanes(int row) {
    final id = rows[row].id;
    expanded.remove(id);
    allProperties.remove(id);
    relane();
  }

  void toggleFold(int row) {
    final id = rows[row].id;
    collapsed.contains(id) ? collapsed.remove(id) : collapsed.add(id);
    relane();
  }

  /// A layer switch: hidden, solo, locked (document attributes) or clipToBelow (the clip op).
  void toggleSwitch(int row, String field) {
    final r = rows[row];
    if (field == 'clipToBelow') {
      if (has('clip')) c.command('clip', {'layer': r.id});
    } else if (has('toggle')) {
      c.command('toggle', {'layer': r.id, 'flag': field});
    }
  }

  /// A lane's key at the playhead, added or removed.
  void toggleKeyHere(int row) {
    final r = rows[row];
    if (r.property != null && has('toggleKey')) c.command('toggleKey', {'layer': r.id, 'property': r.property!['id']});
  }

  // ---- view ------------------------------------------------------------------------------------------------------
  /// How close time is (pixels a frame), where the view starts in time, and the first row shown (fractional while
  /// scrolling). The skin's width in frames and height in rows bound them.
  double pixelsPerFrame = 3;
  double startFrame = 0, firstRow = 0;
  double _viewFrames = 300, _viewRows = 8;

  /// The skin says how much it shows; the session keeps the view inside the work and tells the document session how
  /// many frames are visible (its zoom-to-fit reads it).
  void viewport({required double frames, required double rows}) {
    if (frames == _viewFrames && rows == _viewRows) return;
    _viewFrames = frames;
    _viewRows = rows;
    final visible = frames.round();
    if (c.visibleFrames.value != visible) c.visibleFrames.value = visible;
    _clampView();
  }

  /// Where time stops being worth scrolling to: the farthest of the length, the playhead and any layer's end (ghost
  /// included), plus a margin.
  int get extent {
    var far = math.max(duration, c.frame.value + 1);
    for (final l in c.layers) {
      far = math.max(far, (l['start'] as num? ?? 0).toInt() + (l['duration'] as num? ?? 0).toInt() + math.max(0, (l['ghost'] as num? ?? 0).toInt()));
    }
    return far + math.max(fps, far ~/ 8);
  }

  void setView({double? pixelsPerFrame, double? startFrame, double? firstRow}) {
    this.pixelsPerFrame = (pixelsPerFrame ?? this.pixelsPerFrame).clamp(.1, 40.0);
    this.startFrame = startFrame ?? this.startFrame;
    this.firstRow = firstRow ?? this.firstRow;
    _clampView();
    notifyListeners();
  }

  /// Zoom by [factor] keeping [atFrame] where it is on screen ([anchor] frames from the view's start).
  void zoomAt(double factor, double atFrame) {
    final anchor = atFrame - startFrame;
    final next = (pixelsPerFrame * factor).clamp(.1, 40.0);
    final frames = _viewFrames * pixelsPerFrame / next;
    setView(pixelsPerFrame: next, startFrame: atFrame - anchor * (frames / _viewFrames));
  }

  void _clampView() {
    startFrame = startFrame.clamp(0.0, math.max(0.0, extent - _viewFrames)).toDouble();
    firstRow = firstRow.clamp(0.0, math.max(0.0, rows.length - _viewRows + 1)).toDouble();
  }

  // ---- time ------------------------------------------------------------------------------------------------------
  /// While scrubbing, the head is drawn where it was asked to be before the host's reply lands.
  final scrub = ValueNotifier<int?>(null);
  Future<void>? _seeking;
  void seek(int frame) {
    frame = math.max(0, frame);
    scrub.value = frame;
    final flight = _seeking = c.commandDirect('seek', {'frame': frame}, 'timeline-seek');
    flight.whenComplete(() {
      if (identical(_seeking, flight)) scrub.value = null;
    });
  }

  /// A marker carried along the ruler (its frame while carried), landing on release.
  (String, double)? markerDrag;
  void dragMarker(String id, double frame) {
    markerDrag = (id, math.max(0, frame));
    notifyListeners();
  }

  void dropMarker() {
    final d = markerDrag;
    markerDrag = null;
    notifyListeners();
    if (d != null && has('setMarker')) c.command('setMarker', {'id': d.$1, 'frame': d.$2.round()});
  }

  void deleteMarker(String id) {
    if (has('deleteMarker')) c.command('deleteMarker', {'id': id});
  }

  // ---- the gesture in flight -------------------------------------------------------------------------------------
  TlGesture? gesture;
  int? _anchor;
  TrackRow? _pressed;
  List<int> _initialIds = [], _rowDragIds = [];
  List<Map<String, dynamic>> initialKeys = [];
  List<TrackRow> _timingRows = [];
  double _fromFrame = 0;
  int deltaFrames = 0;
  bool _additive = false;

  /// The marquee, in frames and rows.
  ({double f0, double r0, double f1, double r1})? marquee;

  /// Where a carried row or asset would land, and whether inside a group.
  TlDrop? drop;

  /// Keys moved on release stay drawn where they went until the document carries them.
  List<Map<String, dynamic>> settlingKeys = const [];
  int settlingDelta = 0;

  bool get gripping => gesture != null || settlingKeys.isNotEmpty;

  void _choose(int id, TlMods mods) {
    var ids = c.selectedIds.toList();
    if (mods.shift && _anchor != null) {
      final order = c.layers.map((l) => (l['id'] as num).toInt()).toList();
      final a = order.indexOf(_anchor!), b = order.indexOf(id);
      if (a >= 0 && b >= 0) {
        final range = order.sublist(math.min(a, b), math.max(a, b) + 1);
        ids = mods.primary ? {...ids, ...range}.toList() : range;
      }
    } else if (mods.primary) {
      _anchor = id;
      c.command('select', {'toggle': id});
      return;
    } else {
      ids = [id];
    }
    _anchor = id;
    c.command('select', {'ids': ids, 'keys': []});
  }

  void _pickKeys(List<Map<String, dynamic>> found) {
    final chosen = tlPick(selectedKeys, found, additive: _additive);
    c.command('select', {'ids': chosen.map((k) => k['layer']).toSet().toList(), 'keys': chosen});
    initialKeys = chosen;
  }

  /// A press: on [target], at [frame] (the time under the pointer) and [row] (the row position under it).
  void press(TlTarget target, {required double frame, required double row, required TlMods mods}) {
    _additive = mods.primary || mods.shift;
    _fromFrame = frame;
    deltaFrames = 0;
    initialKeys = selectedKeys;
    _initialIds = c.selectedIds.toList();
    switch (target) {
      case TlRowName(:final row):
        final r = rows[row];
        _pressed = r;
        if (r.property == null) {
          _rowDragIds = _initialIds.contains(r.id) ? _initialIds.toList() : [r.id];
          if (!_initialIds.contains(r.id)) _choose(r.id, mods);
          gesture = TlGesture.rowPending;
        } else {
          _choose(r.id, mods);
          gesture = null;
        }
      case TlLayerKey(:final row, frame: final f):
        final r = rows[row];
        _pickKeys(r.allKeys.where((k) => k['frame'] == f).toList());
        gesture = has('moveKeys') ? TlGesture.keys : null;
      case TlLaneKey(:final row, frame: final f):
        final r = rows[row];
        _pickKeys([for (final k in r.keys) if (k['frame'] == f) tlKeyOf(r, k)]);
        gesture = has('moveKeys') ? TlGesture.keys : null;
      case TlSpan(:final row, :final from, :final to):
        final r = rows[row];
        _pickKeys([for (final k in r.keys) if (k['frame'] == from || k['frame'] == to) tlKeyOf(r, k)]);
        gesture = null;
      case TlBar(:final row, :final part):
        final r = rows[row];
        _pressed = r;
        if (!_initialIds.contains(r.id)) _choose(r.id, mods);
        _timingRows = _initialIds.contains(r.id)
            ? [for (final l in c.layers) if (_initialIds.contains(l['id']) && l['locked'] != true) TrackRow(l)]
            : [r];
        if (r.layer['locked'] == true || (!has('setTiming') && !has('setTimings')) || (_timingRows.length > 1 && !has('setTimings'))) {
          gesture = null;
        } else {
          gesture = mods.alt ? TlGesture.slip : switch (part) { TlBarPart.start => TlGesture.trimIn, TlBarPart.end => TlGesture.trimOut, TlBarPart.body => TlGesture.move };
        }
      case TlEmpty():
        gesture = TlGesture.marquee;
        marquee = (f0: frame, r0: row, f1: frame, r1: row);
    }
    notifyListeners();
  }

  /// The pointer moved (past the skin's own slop) with the press held.
  void drag({required double frame, required double row}) {
    switch (gesture) {
      case null:
        return;
      case TlGesture.rowPending || TlGesture.rowDrag:
        gesture = TlGesture.rowDrag;
        drop = _dropAt(row, _rowDragIds);
      case TlGesture.marquee:
        final m = marquee!;
        marquee = (f0: m.f0, r0: m.r0, f1: frame, r1: row);
      case TlGesture.keys:
        // keys stop at frame 0 as the bars do, and the host clamps the same way, so what is drawn is what lands
        deltaFrames = (frame - _fromFrame).round();
        if (initialKeys.isNotEmpty) deltaFrames = math.max(deltaFrames, -initialKeys.map((k) => (k['frame'] as num).toInt()).reduce(math.min));
      case TlGesture.move || TlGesture.trimIn || TlGesture.trimOut || TlGesture.slip:
        // what the gesture did and by how many frames; the host works out the timings and their limits, once, and
        // the Timeline and the Stage both show its preview while the bar is held
        deltaFrames = (frame - _fromFrame).round();
        if (has('previewTimings')) {
          _previewUsed = true;
          c.commandDirect('previewTimings', {'changes': _retime(gesture!)}, 'timeline-timings');
        }
    }
    notifyListeners();
  }

  List<Map<String, dynamic>> _retime(TlGesture g) => [
        for (final r in _timingRows)
          {'layer': r.id, 'mode': const {TlGesture.trimIn: 'trimStart', TlGesture.trimOut: 'trimEnd', TlGesture.slip: 'slip'}[g] ?? 'move', 'delta': deltaFrames},
      ];

  void release() {
    final g = gesture;
    if (g == null) return;
    switch (g) {
      case TlGesture.rowDrag when drop != null:
        if (has('moveLayers')) c.command('moveLayers', drop!.command);
      case TlGesture.rowPending when _pressed != null:
        _choose(_pressed!.id, const TlMods());
      case TlGesture.keys when deltaFrames != 0:
        settlingKeys = initialKeys;
        settlingDelta = deltaFrames;
        c.command('moveKeys', {'deltaFrames': deltaFrames}).whenComplete(() {
          settlingKeys = const [];
          settlingDelta = 0;
          notifyListeners();
        });
      case TlGesture.move || TlGesture.trimIn || TlGesture.trimOut || TlGesture.slip when deltaFrames != 0:
        _finishTiming(_retime(g));
      case TlGesture.marquee:
        final m = marquee!;
        final picked = tlMarquee(rows, frames: (math.min(m.f0, m.f1), math.max(m.f0, m.f1)), rows_: (math.min(m.r0, m.r1), math.max(m.r0, m.r1)), keys: _additive ? initialKeys : const [], ids: _additive ? _initialIds : const []);
        c.command('select', {'ids': picked.ids, 'keys': picked.keys});
      default:
        break;
    }
    if (_previewUsed && !const {TlGesture.move, TlGesture.trimIn, TlGesture.trimOut, TlGesture.slip}.contains(g)) {
      _previewUsed = false;
      c.cancelPreview();
    }
    _reset();
  }

  void cancel() {
    if (_previewUsed) {
      _previewUsed = false;
      c.cancelPreview();
    }
    _reset();
  }

  void _reset() {
    gesture = null;
    drop = null;
    marquee = null;
    _rowDragIds = [];
    _pressed = null;
    _timingRows = [];
    deltaFrames = 0;
    notifyListeners();
  }

  // the host's preview of timings while a bar is held goes through commandDirect: the newest wins, the commit keeps its place
  bool _previewUsed = false;

  Future<void> _finishTiming(List<Map<String, dynamic>> changes) async {
    final used = _previewUsed;
    _previewUsed = false;
    if (used) {
      await c.command('commitPreview');
    } else if (has('setTimings')) {
      await c.command('setTimings', {'changes': changes});
    }
  }

  // ---- dropping --------------------------------------------------------------------------------------------------
  TlDrop? _dropAt(double row, List<int> moving) {
    if (row >= rows.length) return TlDrop(moving, null, 'rootEnd', depth: 0, at: rows.length.toDouble());
    if (row < 0) return null;
    final i = row.floor();
    final target = rows[i];
    if (moving.contains(target.id) || target.ancestors.any(moving.contains)) return null;
    final f = row - i;
    final inside = target.isGroup && target.property == null && f > .23 && f < .77;
    final placement = inside ? 'inside' : (f < .5 ? 'before' : 'after');
    final container = _layout.container(target.id)!;
    return TlDrop(moving, target.id, placement,
        depth: target.depth, at: inside ? i.toDouble() : (placement == 'before' ? container.bounds.top : container.bounds.bottom), inside: inside);
  }

  /// A Media asset carried over the rows, at [row] and, over the lanes, [frame].
  double? assetFrame;
  void aimAsset({required double row, double? frame}) {
    drop = _dropAt(row, const []);
    assetFrame = frame;
    notifyListeners();
  }

  void leaveAsset() {
    drop = null;
    assetFrame = null;
    notifyListeners();
  }

  /// [catalog]: the id is a Media Catalog asset's (the work admits it as it is placed), not an asset the work already owns.
  void acceptAsset(Object asset, {bool catalog = false}) {
    final d = drop, f = assetFrame;
    leaveAsset();
    if (d == null) return;
    c.command(catalog ? 'placeCatalogAsset' : 'placeAsset', {'id': asset, 'target': d.target, 'placement': d.placement, if (f != null) 'start': math.max(0, f.round())});
  }

  // ---- keys ------------------------------------------------------------------------------------------------------
  /// Arrows move picked keys (Shift: by ten), else step the playhead; Escape drops the gesture.
  bool key({required bool left, required bool right, required bool escape, required bool shift}) {
    if (escape) {
      cancel();
      c.cancelPreview();
      return true;
    }
    if (!left && !right) return false;
    final delta = (left ? -1 : 1) * (shift ? 10 : 1);
    if (selectedKeys.isNotEmpty && has('moveKeys')) {
      c.command('moveKeys', {'deltaFrames': delta});
    } else {
      seek(c.frame.value + delta);
    }
    return true;
  }

  // ---- the row menu ----------------------------------------------------------------------------------------------
  /// The right-click menu of [row] (null: off the rows): the row is picked first unless its selection holds it.
  Future<List<TimelineMenuLine>> menu(int? row) async {
    final id = row == null ? null : rows[row].id;
    if (id != null && !c.selectedIds.contains(id)) await c.command('select', {'ids': [id], 'keys': []});
    final frozen = id != null && rows[row!].layer['frozen'] == true;
    return [
      if (id != null) ...[
        (value: frozen ? 'freeze:off' : 'freeze:on', label: frozen ? 'Unfreeze' : 'Freeze', enabled: has('freeze'), shortcut: '', groupEnd: true),
        (value: 'lanes:keyed', label: 'Show animated properties', enabled: true, shortcut: '', groupEnd: false),
        (value: 'lanes:all', label: 'Show all properties', enabled: true, shortcut: '', groupEnd: false),
        (value: 'lanes:hide', label: 'Hide properties', enabled: true, shortcut: '', groupEnd: true),
      ],
      for (final (value, label, shortcut) in const [('copy', 'Copy', '⌘C'), ('cut', 'Cut', '⌘X'), ('paste', 'Paste', '⌘V'), ('duplicate', 'Duplicate', '⌘D'), ('delete', 'Delete', '⌫'), ('group', 'Group', '⌘G'), ('ungroup', 'Ungroup', '⇧⌘G'), ('split', 'Split', '⌘K')])
        (value: value, label: label, enabled: has(value), shortcut: shortcut, groupEnd: false),
    ];
  }

  void runMenu(int? row, String value) {
    final id = row == null ? null : rows[row].id;
    if (value == 'lanes:hide' && row != null) {
      hideLanes(row);
    } else if (value.startsWith('lanes:') && row != null) {
      showLanes(row, all: value == 'lanes:all');
    } else if (value.startsWith('freeze:') && id != null) {
      c.command('freeze', {'layer': id, 'enabled': value == 'freeze:on'});
    } else if (value == 'delete' && id != null) {
      c.command('delete', {'layers': true});
    } else if (!value.contains(':')) {
      c.command(value);
    }
  }
}

typedef TimelineMenuLine = ({String value, String label, bool enabled, String shortcut, bool groupEnd});
