import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../hf/shell/place.dart';
import '../../hf/shell/menu.dart' show showHfMenu;
import '../../hf/shell/timeline.dart';
import '../../session/editor_session.dart';
import '../../timeline_core/frame.dart';
import '../../timeline_core/geometry.dart';
import '../../timeline_core/grip.dart';
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
        TimelineView<LiveTimeline> {
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

  @override
  void navigateView(double scale, double x, double y) {
    final nextScale = scale.clamp(.1, 40.0);
    final visible = math.max(1.0, viewportWidth - labelWidth);
    final maxOffset = math.max(0.0, overviewExtent * nextScale - visible);
    setState(() {
      pixelsPerFrame = nextScale;
      _viewportOffset = x.clamp(0.0, maxOffset).toDouble();
      _shown = _rows();
    });
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
    final waves = {
      for (final w in EditorSession.maps(c.state['waveforms']))
        w['layer']: EditorSession.maps(w['columns']),
    };
    rowStart = rowStart.clamp(0, math.max(0, tracks.length - 8));
    final shown = tracks.skip(rowStart).take(8).toList();
    final out = <TlRow>[];
    for (final (n, row) in shown.indexed) {
      final layer = row.layer;
      final id = row.id;
      final name = row.property == null
          ? '${layer['name'] ?? layer['kind']}'
          : '${row.property!['label'] ?? row.property!['id']}';
      final keyRows = row.property == null ? row.allKeys : row.keys;
      final frames =
          keyRows.map((key) => (key['frame'] as num).round()).toSet().toList()
            ..sort();
      final xs = [for (final frame in frames) _x(frame)];
      final pickedFrames = {
        for (final key in selectedKeys)
          if (key['layer'] == id &&
              (row.property == null || key['property'] == row.property!['id']))
            (key['frame'] as num).round(),
      };
      final pickedXs = [
        for (final frame in frames)
          if (pickedFrames.contains(frame)) _x(frame),
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
        final (chip, bodyColor) = _families[(n + row.depth) % _families.length];
        final body = row.property == null
            ? (
                _x(layer['start'] as num? ?? 0),
                _x(
                  (layer['start'] as num? ?? 0) +
                      (layer['duration'] as num? ?? 0),
                ),
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
  Widget build(BuildContext context) => ValueListenableBuilder<int>(
    valueListenable: c.frame,
    builder: (context, frame, _) {
      final duration = (c.state['durationFrames'] as num? ?? 1).toInt();
      return Focus(
        focusNode: focus,
        onKeyEvent: key,
        child: RF(
          timeline(
            TimelineModel(
              rows: _shown,
              ruler: [for (var i = 0; i <= 10; i++) _label(i)],
              playhead: _x(frame),
              markers: [
                for (final marker in EditorSession.maps(c.state['markers']))
                  (_x(marker['frame'] as num), '${marker['id']}'),
              ],
              onAddMarker: c.supports('addMarker')
                  ? () => c.command('addMarker')
                  : null,
              onSplit: c.supports('split') ? () => c.command('split') : null,
              onMarkerContext: (id, at) => _markerContext(id, at, context),
              onSeek: (x) => c.seek(
                _frameAt(x).round().clamp(0, duration > 0 ? duration - 1 : 0),
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
              onScroll: (dx, dy, zoom) {
                if (zoom) {
                  zoomAt(
                    math.exp(-dy / 500),
                    labelWidth + (viewportWidth - labelWidth) / 2,
                  );
                } else if (dy.abs() > dx.abs() && tracks.length > 8) {
                  setState(() {
                    rowStart = (rowStart + (dy > 0 ? 1 : -1)).clamp(
                      0,
                      math.max(0, tracks.length - 8),
                    );
                    _shown = _rows();
                  });
                } else {
                  navigateView(
                    pixelsPerFrame,
                    math.max(0, offset + (dx == 0 ? dy : dx)),
                    0,
                  );
                }
              },
            ),
          ),
          ox: 344,
          oy: 703,
        ),
      );
    },
  );
}
