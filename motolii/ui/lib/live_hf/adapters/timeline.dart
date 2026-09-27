import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../hf/shell/place.dart';
import '../../hf/shell/timeline.dart';
import '../../session/editor_session.dart';

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
final _families = [(H.neutralN, H.neutralT), (H.scatter.n, H.scatter.t), (H.stagger.n, H.stagger.t), (H.along.n, H.along.t), (H.face.n, H.face.t), (H.attach.n, H.attach.t)];

/// Seconds one major tick spans.
const _spans = [0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0];

class _LiveTimelineState extends State<LiveTimeline> {
  static const _watched = ['layers', 'fps', 'durationFrames', 'waveforms', 'selectedIds', 'selectedKeys'];
  EditorSession get c => widget.c;
  double fps = 30;
  int span = 2;
  double start = 0;
  int rowStart = 0;

  /// The layers shown, in row order, with each key's frame and properties at that frame.
  List<Map<String, dynamic>> _layers = const [];
  List<int> _ids = const [];

  List<TlRow> _shown = const [];

  @override
  void initState() {
    super.initState();
    _shown = _rows();
    c.slice('liveTimeline', _watched).addListener(_changed);
  }

  @override
  void dispose() {
    c.slice('liveTimeline', _watched).removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() => _shown = _rows());

  double get _unit => _spans[span];
  double _x(num frame) => tlX((frame / fps - start) / _unit);
  double _frameAt(double x) => ((x - tlX0 - .5) / tlUnit * _unit + start) * fps;
  int _frames(double dx) => (dx / tlUnit * _unit * fps).round();

  /// Frame -> the properties keyed there, for one layer.
  Map<int, List<String>> _keysOf(Map<String, dynamic> l) {
    final out = <int, List<String>>{};
    for (final p in EditorSession.maps(l['properties'])) {
      for (final k in EditorSession.maps(p['keys'])) {
        (out[(k['frame'] as num).round()] ??= []).add('${p['id']}');
      }
    }
    return out;
  }

  List<TlRow> _rows() {
    fps = (c.state['fps'] as num? ?? 30).toDouble();
    final selected = c.selectedIds.toSet();
    final picked = <int, Set<int>>{};
    for (final k in EditorSession.maps(c.state['selectedKeys'])) {
      (picked[k['layer'] as int] ??= {}).add((k['frame'] as num).round());
    }
    final waves = {for (final w in EditorSession.maps(c.state['waveforms'])) w['layer']: EditorSession.maps(w['columns'])};
    final all = EditorSession.maps(c.state['layers']);
    final audio = all.where((l) => l['kind'] == 'Audio').firstOrNull;
    final listed = [for (final l in all) if (l['kind'] != 'Audio') l];
    rowStart = rowStart.clamp(0, math.max(0, listed.length - 8));
    final shown = listed.skip(rowStart).take(8).toList();
    final out = <TlRow>[];
    _layers = [...shown, if (audio != null) audio];
    _ids = [for (final l in _layers) l['id'] as int];
    for (final (n, l) in listed.indexed) {
      if (n < rowStart || n >= rowStart + 8) continue;
      final id = l['id'] as int, name = '${l['name'] ?? l['kind']}';
      final keys = _keysOf(l).keys.toList();
      final xs = [for (final f in keys) _x(f)];
      final pickedXs = [for (final f in keys) if (picked[id]?.contains(f) ?? false) _x(f)];
      switch (l['kind']) {
        case 'Group':
          out.add(TlRow(name, TlKind.group, open: true, selected: selected.contains(id)));
        case 'Camera':
          out.add(TlRow(name, TlKind.camera, open: false, keys: xs, pickedKeys: pickedXs, selected: selected.contains(id)));
        default:
          final (chip, body) = _families[n % _families.length];
          final s = (l['start'] as num? ?? 0), d = (l['duration'] as num? ?? 0);
          out.add(TlRow(name, TlKind.item, chip: chip, body: (_x(s), _x(s + d), body), keys: xs, pickedKeys: pickedXs, selected: selected.contains(id)));
      }
    }
    if (audio != null) out.add(TlRow('${audio['name'] ?? 'Audio'}', TlKind.audio, wave: _wave(waves[audio['id']] ?? const []), selected: selected.contains(audio['id'])));
    return out;
  }

  /// The floor's half-heights, one per 2 px from x 596, from the host's per-frame min/max columns.
  List<double> _wave(List<Map<String, dynamic>> columns) {
    if (columns.isEmpty) return const [];
    final byFrame = {for (final col in columns) (col['frame'] as num).toInt(): ((col['max'] as num) - (col['min'] as num)).toDouble() / 2};
    return [for (var x = 596.0; x < 1488; x += 2) (byFrame[_frameAt(x).round()] ?? 0) * 11];
  }

  String _label(int i) {
    final t = start + i * _unit;
    final m = t ~/ 60, sec = (t % 60).floor(), ff = ((t - t.floorToDouble()) * fps).round();
    return _unit < 1 ? '${sec.toString().padLeft(2, '0')}:${ff.toString().padLeft(2, '0')}' : '${m.toString().padLeft(2, '0')}:${sec.toString().padLeft(2, '0')}';
  }

  void _pickKey(int row, double x, bool add) {
    final l = _layers[row], id = l['id'] as int;
    final keys = _keysOf(l);
    final frame = keys.keys.reduce((a, b) => (_x(a) - x).abs() <= (_x(b) - x).abs() ? a : b);
    final these = [for (final p in keys[frame]!) {'layer': id, 'property': p, 'frame': frame}];
    final held = add ? EditorSession.maps(c.state['selectedKeys']) : const <Map<String, dynamic>>[];
    c.command('select', {
      'ids': add ? {...c.selectedIds, id}.toList() : [id],
      'keys': [...held, ...these],
    });
  }

  /// The layer's timing when the current drag began: every preview is measured from it, not from the last preview.
  (int, int, int, int)? _base;

  Map<String, dynamic> _retimed(int row, TlGrip grip, double dx) {
    final l = _layers[row], d = _frames(dx);
    final b = _base ??= (l['id'] as int, (l['start'] as num? ?? 0).round(), (l['duration'] as num? ?? 0).round(), (l['sourceIn'] as num? ?? 0).round());
    final (_, s, len, src) = b;
    return switch (grip) {
      TlGrip.body => {'layer': b.$1, 'start': s + d, 'duration': len, 'sourceIn': src},
      TlGrip.start => {'layer': b.$1, 'start': s + d.clamp(-s, len - 1), 'duration': len - d.clamp(-s, len - 1), 'sourceIn': src + d.clamp(-s, len - 1)},
      TlGrip.end => {'layer': b.$1, 'start': s, 'duration': math.max(1, len + d), 'sourceIn': src},
    };
  }

  @override
  Widget build(BuildContext context) => ValueListenableBuilder<int>(
        valueListenable: c.frame,
        builder: (context, frame, _) {
          final duration = (c.state['durationFrames'] as num? ?? 1).toInt();
          return RF(timeline(TimelineModel(
            rows: _shown,
            ruler: [for (var i = 0; i <= 10; i++) _label(i)],
            playhead: _x(frame),
            onSeek: (x) => c.seek(_frameAt(x).round().clamp(0, duration > 0 ? duration - 1 : 0)),
            onRow: (i) => c.command('select', {'ids': [_ids[i]]}),
            onKey: _pickKey,
            onKeysDrag: (dx, done) {
              if (done == true && _frames(dx) != 0) c.command('moveKeys', {'deltaFrames': _frames(dx)});
            },
            onGrip: (row, grip, dx, done) {
              if (done == null) {
                _base = null;
                c.cancelPreview();
                return;
              }
              final change = _retimed(row, grip, dx);
              if (done) _base = null;
              c.command(done ? 'setTimings' : 'previewTimings', {'changes': [change]});
            },
            onScroll: (dx, dy, zoom) => setState(() {
              if (zoom) {
                span = (span + (dy > 0 ? 1 : -1)).clamp(0, _spans.length - 1);
              } else if (dy.abs() > dx.abs() && _layers.length >= 8) {
                rowStart = math.max(0, rowStart + (dy > 0 ? 1 : -1));
              } else {
                start = math.max(0, start + (dx == 0 ? dy : dx) / tlUnit * _unit);
              }
              _shown = _rows();
            }),
          )), ox: 344, oy: 703);
        },
      );
}
