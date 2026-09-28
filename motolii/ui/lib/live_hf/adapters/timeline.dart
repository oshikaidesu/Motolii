import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../hf/shell/place.dart';
import '../../hf/shell/menu.dart' show showHfMenu;
import '../../hf/shell/timeline.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import '../../timeline_core/frame.dart';
import '../../timeline_core/geometry.dart';
import '../../timeline_core/grip.dart';
import '../../timeline_core/menu.dart';
import '../../timeline_core/view.dart';

/// The Timeline face over the session: one row per layer (a group, a camera, an audio floor, or an item with its
/// body from `start` to `start + duration` and a diamond at every key), the playhead at `frame`.
/// Pressing the tracks seeks; a name or a body selects its layer; a key picks the layer's keys at that frame (Shift
/// adds); a picked key dragged moves the picked keys (`moveKeys`); a body or one of its ends dragged retimes the layer
/// (`previewTimings`, then `setTimings`). Scrolling pans time and rows; with Cmd/Ctrl it changes the seconds a major
/// tick spans.
class LiveTimeline extends StatefulWidget {
  const LiveTimeline({super.key, required this.c});
  final EditorSession c;
  @override
  State<LiveTimeline> createState() => _LiveTimelineState();
}

/// Row identity colours, given out in layer order: presentation only, nothing is stored.
final _families = [
  (H.neutralN, H.neutralT),
  (H.scatter.n, H.scatter.t),
  (H.stagger.n, H.stagger.t),
  (H.along.n, H.along.t),
  (H.face.n, H.face.t),
  (H.attach.n, H.attach.t),
];

class _LiveTimelineState extends State<LiveTimeline>
    with
        TimelineFrame<LiveTimeline>,
        TimelineGrip<LiveTimeline>,
        TimelineView<LiveTimeline>,
        TimelineMenu<LiveTimeline> {
  static const _watched = [
    'layers',
    'fps',
    'durationFrames',
    'waveforms',
    'selectedIds',
    'selectedKeys',
    'markers',
  ];
  EditorSession get c => widget.c;
  @override
  EditorSession get timelineSession => c;
  @override
  double baseNameWidth = TimelineGeometry.hf.nameWidth;
  @override
  TimelineGeometry get timelineGeometry => TimelineGeometry.hf;
  double fps = 30;
  double _viewportOffset = 0;
  int rowStart = 0;

  @override
  double get offset => _viewportOffset;

  List<TlRow> _shown = const [];

  @override
  void initState() {
    super.initState();
    viewportWidth = tlRight - 344;
    fps = (c.state['fps'] as num? ?? 30).toDouble();
    pixelsPerFrame = tlUnit / fps;
    relane();
    _shown = _rows();
    c.slice('liveTimeline', _watched).addListener(_changed);
  }

  @override
  void dispose() {
    c.slice('liveTimeline', _watched).removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() => _shown = _rows());

  double get _unit => tlUnit / (fps * pixelsPerFrame);
  double _x(num frame) => tlX0 + .5 + frame * pixelsPerFrame - offset;
  double _frameAt(double x) => (x - tlX0 - .5 + offset) / pixelsPerFrame;

  // Rows scroll in whole rows, but the offset keeps the part of a row a small wheel step moved, so steps add up.
  double _rowOffset = 0;

  @override
  double get verticalOffset => _rowOffset;

  /// Whenever the core lays the rows out again (a lane or group opened from the rows, the document), the face's
  /// rows follow at once.
  @override
  void relane() {
    super.relane();
    _shown = _rows();
  }

  @override
  void applyNavigation(double scale, double x, double y) {
    final maxRow = math.max(0, tracks.length - 8);
    setState(() {
      pixelsPerFrame = scale;
      _viewportOffset = x;
      _rowOffset = y.clamp(0.0, maxRow * timelineGeometry.rowHeight);
      rowStart = (_rowOffset / timelineGeometry.rowHeight).floor();
      _shown = _rows();
    });
    reportVisible(); // new layers take the span in view (the session's visible frames)
  }

  List<TlRow> _rows() {
    final nextFps = (c.state['fps'] as num? ?? 30).toDouble();
    if (fps != nextFps) {
      final unit = _unit;
      fps = nextFps;
      pixelsPerFrame = tlUnit / (fps * unit);
    }
    final selected = c.selectedIds.toSet();
    final selectedKeys = EditorSession.maps(c.state['selectedKeys']);
    // A grip draws from its press-time keys and timings plus the delta, as Classic's does: the document's rows only
    // catch up when the preview reply lands.
    final gripped = gesture == 'keys' ? initialKeys : settlingKeys;
    final keyShift = gesture == 'keys' ? deltaFrames : settlingDelta;
    final timings =
        const {'move', 'trimIn', 'trimOut', 'slip'}.contains(gesture)
        ? {for (final r in timingRows) r.id: timing(r)}
        : const <Object, Map<String, dynamic>>{};
    final waves = {
      for (final w in EditorSession.maps(c.state['waveforms']))
        w['layer']: EditorSession.maps(w['columns']),
    };
    final maxRow = math.max(0, tracks.length - 8);
    if (rowStart > maxRow) {
      rowStart = maxRow;
      _rowOffset = maxRow * timelineGeometry.rowHeight;
    }
    final shown = tracks.skip(rowStart).take(8).toList();
    final out = <TlRow>[];
    for (final (n, row) in shown.indexed) {
      final layer = row.layer;
      final id = row.id;
      final name = row.property == null
          ? '${layer['name'] ?? layer['kind']}'
          : '${row.property!['label'] ?? row.property!['id']}';
      final keyRows = row.property == null ? row.allKeys : row.keys;
      final moving = {
        for (final key in gripped)
          if (key['layer'] == id &&
              (row.property == null || key['property'] == row.property!['id']))
            (key['frame'] as num).round(),
      };
      double keyX(int frame) =>
          _x(moving.contains(frame) ? frame + keyShift : frame);
      final frames =
          keyRows.map((key) => (key['frame'] as num).round()).toSet().toList()
            ..sort();
      final xs = [for (final frame in frames) keyX(frame)];
      final pickedFrames = {
        for (final key in selectedKeys)
          if (key['layer'] == id &&
              (row.property == null || key['property'] == row.property!['id']))
            (key['frame'] as num).round(),
      };
      final pickedXs = [
        for (final frame in frames)
          if (pickedFrames.contains(frame)) keyX(frame),
      ];
      final selectedRow = selected.contains(id);
      final indent = row.depth;
      if (row.property == null && row.isGroup) {
        out.add(
          TlRow(
            name,
            TlKind.group,
            indent: indent,
            nameW: 100,
            open: row.groupOpen,
            propertiesOpen: row.lanesOpen,
            selected: selectedRow,
            hidden: layer['hidden'] == true,
            solo: layer['solo'] == true,
            locked: layer['locked'] == true,
            clipToBelow: layer['clipToBelow'] == true,
          ),
        );
      } else if (row.property == null && layer['kind'] == 'Camera') {
        out.add(
          TlRow(
            name,
            TlKind.camera,
            indent: indent,
            nameW: 80,
            propertiesOpen: row.lanesOpen,
            keys: xs,
            pickedKeys: pickedXs,
            selected: selectedRow,
            hidden: layer['hidden'] == true,
            solo: layer['solo'] == true,
            locked: layer['locked'] == true,
            clipToBelow: layer['clipToBelow'] == true,
          ),
        );
      } else {
        final (chip, family) = _families[(n + row.depth) % _families.length];
        // a hidden (muted) layer's bar is grey (Classic TL-105)
        final bodyColor = layer['hidden'] == true ? H.raisedHi : family;
        final t = timings[id] ?? layer;
        final body = row.property == null
            ? (
                _x(t['start'] as num? ?? 0),
                _x((t['start'] as num? ?? 0) + (t['duration'] as num? ?? 0)),
                bodyColor,
              )
            : null;
        final wave = layer['kind'] == 'Audio'
            ? _wave(waves[id] ?? const [])
            : null;
        out.add(
          TlRow(
            name,
            TlKind.item,
            indent: indent,
            nameW: 80,
            chip: chip,
            body: body,
            keys: xs,
            pickedKeys: pickedXs,
            propertiesOpen: row.property == null ? row.lanesOpen : null,
            lane: row.property != null,
            ghost: row.property == null ? _ghost(layer, t) : null,
            spans: row.property == null ? const [] : _spans(row.keys, keyX, pickedFrames),
            wave: wave,
            selected: selectedRow,
            hidden: layer['hidden'] == true,
            solo: layer['solo'] == true,
            locked: layer['locked'] == true,
            clipToBelow: layer['clipToBelow'] == true,
          ),
        );
      }
    }
    return out;
  }

  /// Where only the ghost plays, as Classic draws it: past the bar's end for a delay, before its start (never before
  /// frame 0, where a notch says it starts earlier) for an advance.
  (double, double, double?)? _ghost(Map<String, dynamic> layer, Map<String, dynamic> t) {
    final d = layer['ghost'];
    if (d is! num || d == 0) return null;
    final start = (t['start'] as num? ?? 0).toDouble(), end = start + (t['duration'] as num? ?? 0).toDouble();
    final zero = _x(0);
    if (d > 0) return (_x(end), _x(end + d), null);
    final from = _x(start + d);
    return (math.max(zero, from), _x(start), from < zero ? zero : null);
  }

  List<(double, double, bool, bool)> _spans(List<Map<String, dynamic>> keys, double Function(int) keyX, Set<int> picked) {
    final ordered = [...keys]..sort((a, b) => (a['frame'] as num).compareTo(b['frame'] as num));
    return [
      for (var i = 0; i + 1 < ordered.length; i++)
        if (keyX((ordered[i]['frame'] as num).round()) + 5 < keyX((ordered[i + 1]['frame'] as num).round()) - 5)
          (
            keyX((ordered[i]['frame'] as num).round()) + 5,
            keyX((ordered[i + 1]['frame'] as num).round()) - 5,
            EditorSession.map(ordered[i]['interp'])['kind'] == 'Linear',
            picked.contains((ordered[i]['frame'] as num).round()) && picked.contains((ordered[i + 1]['frame'] as num).round()),
          ),
    ];
  }

  /// The floor's half-heights, one per 2 px from x 596, from the host's per-frame min/max columns.
  List<double> _wave(List<Map<String, dynamic>> columns) {
    if (columns.isEmpty) return const [];
    final byFrame = {
      for (final col in columns)
        (col['frame'] as num).toInt():
            ((col['max'] as num) - (col['min'] as num)).toDouble() / 2,
    };
    return [
      for (var x = 596.0; x < 1488; x += 2)
        (byFrame[_frameAt(x).round()] ?? 0) * 11,
    ];
  }

  String _label(int i) {
    final t = (_frameAt(tlX0 + i * tlUnit) / fps).clamp(0, double.infinity);
    final m = t ~/ 60,
        sec = (t % 60).floor(),
        ff = ((t - t.floorToDouble()) * fps).round();
    return _unit < 1
        ? '${sec.toString().padLeft(2, '0')}:${ff.toString().padLeft(2, '0')}'
        : '${m.toString().padLeft(2, '0')}:${sec.toString().padLeft(2, '0')}';
  }

  Offset _corePosition(Offset p) =>
      Offset(p.dx + tlX0 - 344, p.dy + rowStart * timelineGeometry.rowHeight);

  Offset _coreLabelPosition(int visibleIndex, Offset p) => Offset(
    p.dx + 1,
    p.dy + (rowStart + visibleIndex) * timelineGeometry.rowHeight,
  );

  void _foldRow(int visibleIndex) {
    final index = rowStart + visibleIndex;
    if (index < 0 || index >= tracks.length) return;
    final row = tracks[index];
    if (!row.isGroup) return;
    setState(() {
      row.groupOpen
          ? collapsedGroups.add(row.id)
          : collapsedGroups.remove(row.id);
      relane();
      _shown = _rows();
    });
  }

  void _togglePropertyRows(int visibleIndex) {
    final index = rowStart + visibleIndex;
    if (index < 0 || index >= tracks.length) return;
    final row = tracks[index];
    if (row.property != null) return;
    setState(() {
      allProperties.remove(row.id);
      expanded.contains(row.id)
          ? expanded.remove(row.id)
          : expanded.add(row.id);
      relane();
      _shown = _rows();
    });
  }

  (String, double)? _markerDrag;

  Future<void> _markerContext(
    String id,
    Offset at,
    BuildContext context,
  ) async {
    if (!c.supports('deleteMarker')) return;
    final action = await showHfMenu<String>(
      context,
      Rect.fromLTWH(at.dx, at.dy, 180, 0),
      const [('delete', 'Delete marker')],
    );
    if (action == 'delete') c.command('deleteMarker', {'id': id});
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: Listenable.merge([c.frame, scrubFrame]),
    builder: (context, _) {
      // A scrub draws the head where it was asked to be at once; the host's reply catches up (TimelineFrame).
      final frame = scrubFrame.value ?? c.frame.value;
      final gripping = gesture != null || settlingKeys.isNotEmpty;
      final theme = EditorTheme.of(context);
      // grip coordinates are the tracks' own: x from the panel's left edge, y from the first row, scrolled rows above
      final toFace = Offset(344, tlTop - rowStart * timelineGeometry.rowHeight);
      final duration = (c.state['durationFrames'] as num? ?? 1).toInt();
      // A Media tile carried here lands before, after or inside the row under it, at the frame under it (the core's
      // asset drop, the same as Classic's).
      Offset core(Offset global, BuildContext box) =>
          (box.findRenderObject() as RenderBox).globalToLocal(global) +
          const Offset(344, 703) -
          toFace;
      return DragTarget<Map<String, dynamic>>(
        onWillAcceptWithDetails: (d) =>
            d.data['asset'] != null && c.supports('placeAsset'),
        onMove: (d) => aimAssetAt(core(d.offset, context)),
        onLeave: (_) => leaveAsset(),
        onAcceptWithDetails: acceptAsset,
        builder: (context, _, __) => Focus(
          focusNode: focus,
          onKeyEvent: key,
          // Trackpad pan, pinch and ruler scrub-zoom with momentum, and the wheel (pointer-anchored zoom with Cmd or
          // over the ruler): the core's own navigation, the same one Classic's Timeline uses.
          child: navigation(
            RF(
              timeline(
                TimelineModel(
                  tabs: false,
                  rows: gripping ? _rows() : _shown,
                  marquee:
                      gesture == 'marquee' && start != null && current != null
                      ? Rect.fromPoints(start!, current!).shift(toFace)
                      : null,
                  dropGuide: rowDropGuide?.shift(toFace),
                  dropInside: rowDropInside,
                  marqueeInk: theme.accent,
                  guideInk: theme.select,
                  ruler: [for (var i = 0; i <= 10; i++) _label(i)],
                  playhead: _x(frame),
                  markers: [
                    for (final marker in EditorSession.maps(c.state['markers']))
                      (
                        _markerDrag?.$1 == '${marker['id']}'
                            ? _markerDrag!.$2
                            : _x(marker['frame'] as num),
                        '${marker['id']}',
                      ),
                  ],
                  // a marker dragged along the ruler lands on the frame under it (Classic TL-031, setMarker{frame})
                  onMarkerDrag: c.supports('setMarker')
                      ? (id, dx, done) {
                          if (!done) {
                            final from = _markerDrag?.$1 == id
                                ? _markerDrag!.$2
                                : _x(
                                    EditorSession.maps(c.state['markers'])
                                            .firstWhere((m) => '${m['id']}' == id)['frame']
                                        as num,
                                  );
                            setState(() => _markerDrag = (id, from + dx));
                            return;
                          }
                          final drag = _markerDrag;
                          setState(() => _markerDrag = null);
                          if (drag == null) return;
                          c.command('setMarker', {
                            'id': id,
                            'frame': _frameAt(drag.$2)
                                .round()
                                .clamp(0, duration > 0 ? duration - 1 : 0),
                          });
                        }
                      : null,
                  onAddMarker: c.supports('addMarker')
                      ? () => c.command('addMarker')
                      : null,
                  onSplit: c.supports('split')
                      ? () => c.command('split')
                      : null,
                  onMarkerContext: (id, at) => _markerContext(id, at, context),
                  onContext: (at, global) => menuAt(
                    at - toFace,
                    (items) => showHfMenu<String>(
                      context,
                      Rect.fromLTWH(global.dx, global.dy, 220, 0),
                      [for (final item in items) (item.value, item.label)],
                      disabled: {
                        for (final item in items)
                          if (!item.enabled) item.value,
                      },
                      shortcuts: {
                        for (final item in items)
                          if (item.shortcut.isNotEmpty)
                            item.value: item.shortcut,
                      },
                      dividers: {
                        for (final item in items)
                          if (item.groupEnd) item.value,
                      },
                    ),
                  ),
                  onSeek: (x) => requestSeek(
                    _frameAt(x)
                        .round()
                        .clamp(0, duration > 0 ? duration - 1 : 0),
                  ),
                  onRow: (i) {
                    final index = rowStart + i;
                    if (index >= 0 && index < tracks.length)
                      chooseLayer(tracks[index].id);
                  },
                  onFold: _foldRow,
                  onProperties: _togglePropertyRows,
                  onCoreDown: (event, p) => beginAt(event, _corePosition(p)),
                  onCoreMove: (event, p) => moveAt(event, _corePosition(p)),
                  onCoreUp: (event, p) => endAt(event, _corePosition(p)),
                  onCoreCancel: (_, __) => cancel(),
                  onCoreLabelDown: (event, index, p) =>
                      beginAt(event, _coreLabelPosition(index, p)),
                  onCoreLabelMove: (event, index, p) =>
                      moveAt(event, _coreLabelPosition(index, p)),
                  onCoreLabelUp: (event, index, p) =>
                      endAt(event, _coreLabelPosition(index, p)),
                  onCoreLabelCancel: (_, __, ___) => cancel(),
                ),
              ),
              ox: 344,
              oy: 703,
            ),
          ),
        ),
      );
    },
  );
}
