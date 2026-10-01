import 'package:flutter/widgets.dart';

import '../../hf/desk/ease.dart';
import '../../session/editor_session.dart';

/// The Ease desk's curves over the session: the host's ease kinds (`easeKinds`, each drawn from its own samples), the
/// intervals of the picked keys (`easeIntervals`), and `ease` for a changed curve — for one interval its first key is
/// picked first, since `ease` shapes the intervals that start at the picked keys.
class LiveEaseHost extends ChangeNotifier implements EaseHost {
  LiveEaseHost(this.c) {
    c.slice('liveEase', _watched).addListener(_read);
    _read();
  }
  static const _watched = ['easeKinds', 'easeIntervals'];
  final EditorSession c;

  List<Preset> _kinds = const [];
  List<Map<String, dynamic>> _rows = const [];

  static Shape _sampled(List samples) {
    final pts = [for (final p in samples) if (p is List && p.length >= 2) ((p[0] as num).toDouble(), (p[1] as num).toDouble())];
    if (pts.length < 2) return (t) => t;
    return (t) {
      for (var i = 1; i < pts.length; i++) {
        final (u1, v1) = pts[i];
        if (t <= u1) {
          final (u0, v0) = pts[i - 1];
          return u1 == u0 ? v1 : v0 + (v1 - v0) * (t - u0) / (u1 - u0);
        }
      }
      return pts.last.$2;
    };
  }

  static Preset _preset(Map<String, dynamic> k) {
    final name = '${k['kind']}';
    final bez = name == 'Bezier' ? [for (final n in ['x1', 'y1', 'x2', 'y2']) (k[n] as num? ?? 0).toDouble()] : null;
    return Preset(name, bez, bez == null ? _sampled(k['samples'] as List? ?? const []) : null);
  }

  void _read() {
    final kinds = EditorSession.maps(c.state['easeKinds']);
    if (kinds.isNotEmpty) _kinds = [for (final k in kinds) _preset(k)];
    _rows = EditorSession.maps(c.state['easeIntervals']);
    notifyListeners();
  }

  @override
  List<Preset> get kinds => _kinds.isEmpty ? const [Preset('Linear', [0, 0, 1, 1], null)] : _kinds;

  @override
  List<Seg> get intervals => [
        for (final r in _rows)
          () {
            final shape = EditorSession.map(r['shape']);
            final i = kinds.indexWhere((k) => k.name == shape['kind']);
            final seg = Seg(i < 0 ? 0 : i, ((r['end'] as num? ?? 1) - (r['frame'] as num? ?? 0)).round().clamp(1, 1 << 30), kinds);
            if (shape['kind'] == 'Bezier') seg.setValues([for (final n in ['x1', 'y1', 'x2', 'y2']) (shape[n] as num? ?? 0).toDouble()]);
            return seg;
          }(),
      ];

  @override
  Future<void> apply(int? interval, Seg curve) async {
    final picked = {'ids': [...c.selectedIds], 'keys': [...EditorSession.maps(c.state['selectedKeys'])]};
    final narrowed = interval != null && interval < _rows.length;
    if (narrowed) {
      final r = _rows[interval];
      await c.command('select', {
        'ids': [r['layer']],
        'keys': [
          {'layer': r['layer'], 'property': r['property'], 'frame': r['frame']},
        ],
      });
    }
    final kind = kinds[curve.p].name;
    await c.command('ease', {
      'kind': kind,
      if (curve.bez) ...{'x1': curve.x1, 'y1': curve.y1, 'x2': curve.x2, 'y2': curve.y2},
    });
    if (narrowed) await c.command('select', picked);
  }

  @override
  void dispose() {
    c.slice('liveEase', _watched).removeListener(_read);
    super.dispose();
  }
}

/// The Ease desk on the session.
class LiveEase extends StatefulWidget {
  const LiveEase({super.key, required this.c});
  final EditorSession c;
  @override
  State<LiveEase> createState() => _LiveEaseState();
}

class _LiveEaseState extends State<LiveEase> {
  late final host = LiveEaseHost(widget.c);
  @override
  void dispose() {
    host.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => EaseDesk(host: host);
}
