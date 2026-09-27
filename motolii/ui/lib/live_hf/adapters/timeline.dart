import 'package:flutter/widgets.dart';

import '../../hf/shell/place.dart';
import '../../hf/shell/timeline.dart';
import '../../session/editor_session.dart';

/// The Timeline face over the session: one row per layer (a group, a camera, an audio floor, or an item with its
/// body from `start` to `start + duration` and a diamond at every key), a second per major tick, the playhead at
/// `frame`. Pressing the tracks seeks; pressing a name selects that layer.
class LiveTimeline extends StatefulWidget {
  const LiveTimeline({super.key, required this.c});
  final EditorSession c;
  @override
  State<LiveTimeline> createState() => _LiveTimelineState();
}

/// Row identity colours, given out in layer order: presentation only, nothing is stored.
final _families = [(H.neutralN, H.neutralT), (H.scatter.n, H.scatter.t), (H.stagger.n, H.stagger.t), (H.along.n, H.along.t), (H.face.n, H.face.t), (H.attach.n, H.attach.t)];

class _LiveTimelineState extends State<LiveTimeline> {
  static const _watched = ['layers', 'fps', 'durationFrames', 'waveforms'];
  EditorSession get c => widget.c;
  late List<TlRow> rows;
  late List<int> ids;
  late double fps;

  @override
  void initState() {
    super.initState();
    _read();
    c.slice('liveTimeline', _watched).addListener(_changed);
  }

  @override
  void dispose() {
    c.slice('liveTimeline', _watched).removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(_read);

  double _x(num frame) => tlX(frame / fps);

  void _read() {
    fps = (c.state['fps'] as num? ?? 30).toDouble();
    final waves = {for (final w in EditorSession.maps(c.state['waveforms'])) w['layer']: EditorSession.maps(w['columns'])};
    final out = <TlRow>[], outIds = <int>[];
    TlRow? audio;
    int? audioId;
    var n = 0;
    for (final l in EditorSession.maps(c.state['layers'])) {
      final id = l['id'] as int, kind = '${l['kind']}', name = '${l['name'] ?? kind}';
      final keys = <double>{
        for (final p in EditorSession.maps(l['properties'])) for (final k in EditorSession.maps(p['keys'])) _x(k['frame'] as num),
        for (final e in EditorSession.maps(l['effects'])) for (final p in EditorSession.maps(e['params'])) for (final k in EditorSession.maps(p['keys'])) _x(k['frame'] as num),
      }.toList();
      if (kind == 'Audio') {
        audio ??= TlRow(name, TlKind.audio, wave: _wave(waves[id] ?? const []));
        audioId ??= id;
        continue;
      }
      if (out.length == 8) continue;
      outIds.add(id);
      switch (kind) {
        case 'Group':
          out.add(TlRow(name, TlKind.group, open: true));
        case 'Camera':
          out.add(TlRow(name, TlKind.camera, open: false, keys: keys));
        default:
          final (chip, body) = _families[n++ % _families.length];
          final start = (l['start'] as num? ?? 0), duration = (l['duration'] as num? ?? 0);
          out.add(TlRow(name, TlKind.item, chip: chip, body: (_x(start), _x(start + duration), body), keys: keys));
      }
    }
    if (audio != null) {
      out.add(audio);
      outIds.add(audioId!);
    }
    rows = out;
    ids = outIds;
  }

  /// The floor's half-heights, one per 2 px from x 596, from the host's per-frame min/max columns.
  List<double> _wave(List<Map<String, dynamic>> columns) {
    if (columns.isEmpty) return const [];
    final byFrame = {for (final col in columns) (col['frame'] as num).toInt(): ((col['max'] as num) - (col['min'] as num)).toDouble() / 2};
    return [
      for (var x = 596.0; x < 1488; x += 2) (byFrame[((x - tlX0 - .5) / tlUnit * fps).round()] ?? 0) * 11,
    ];
  }

  @override
  Widget build(BuildContext context) => ValueListenableBuilder<int>(
        valueListenable: c.frame,
        builder: (context, frame, _) {
          final duration = (c.state['durationFrames'] as num? ?? 1).toInt();
          return RF(timeline(TimelineModel(
            rows: rows,
            ruler: [for (var i = 0; i <= 10; i++) '00:${i.toString().padLeft(2, '0')}'],
            playhead: _x(frame),
            onSeek: (x) => c.seek(((x - tlX0 - .5) / tlUnit * fps).round().clamp(0, duration > 0 ? duration - 1 : 0)),
            onRow: (i) => c.command('select', {'ids': [ids[i]]}),
          )), ox: 344, oy: 703);
        },
      );
}
