import 'package:flutter/widgets.dart';

import '../../hf/desk/ease.dart';
import '../../session/editor_session.dart';

/// The Ease desk's curves over the session: the host's ease kinds (`easeKinds`, each drawn from its own samples), the
/// intervals of the picked keys (`easeIntervals`), and `ease` for a changed curve — for one interval its first key is
/// picked first, since `ease` shapes the intervals that start at the picked keys.
class LiveEaseHost extends ChangeNotifier implements EaseHost {
  LiveEaseHost(this.c) {
    c.slice('liveEase', _watched).addListener(_read);
    c.deskWork.addListener(_read);
    _read();
  }
  static const _watched = ['easeKinds', 'easeIntervals', 'selectedIds', 'selectedKeys', 'layers'];
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
    _layers = _sequenceLayers();
    notifyListeners();
  }

  /// The ghost mode (2026-09-07 ruling, Classic's Ease desk): no keys picked and two or more picked layers that can
  /// carry a ghost — the curve spreads their delays, in the order they were picked.
  List<Map<String, dynamic>> _layers = const [];
  List<Map<String, dynamic>> _sequenceLayers() {
    if (EditorSession.maps(c.state['selectedKeys']).isNotEmpty) return const [];
    final byId = {for (final l in c.layers) l['id']: l};
    final picked = [
      for (final id in c.selectedIds)
        if (byId[id] case final l? when l['ghostable'] == true) l,
    ];
    return picked.length > 1 ? picked : const [];
  }

  @override
  int get sequence => _layers.length;

  bool get _canSequence => c.supports('sequence') && !_layers.any((l) => l['locked'] == true);

  /// The curve the ghost mode keeps between uses (the desk's own `ease` setting, as Classic keeps it).
  Map<String, dynamic> get _stored => EditorSession.map(c.deskWork.value['ease']);

  Map<String, dynamic> _shape(Seg curve) => {
        'kind': kinds[curve.p].name,
        if (curve.bez) ...{'x1': curve.x1, 'y1': curve.y1, 'x2': curve.x2, 'y2': curve.y2},
      };

  Map<String, dynamic> _payload(Seg curve) => {
        'layers': [for (final l in _layers) l['id']],
        'shape': _shape(curve),
      };

  Map<String, dynamic>? _pending;
  Future<void>? _flight;
  bool _previewed = false;

  @override
  void preview(int? interval, Seg curve) {
    if (_layers.isEmpty || !_canSequence || !c.supports('previewSequence')) return;
    _pending = _payload(curve);
    _previewed = true;
    _flight ??= () async {
      try {
        while (_pending != null) {
          final next = _pending!;
          _pending = null;
          await c.command('previewSequence', next);
        }
      } finally {
        _flight = null;
      }
    }();
  }

  @override
  void cancel() {
    _pending = null;
    if (_previewed) c.cancelPreview();
    _previewed = false;
  }

  @override
  List<Preset> get kinds => _kinds.isEmpty ? const [Preset('Linear', [0, 0, 1, 1], null)] : _kinds;

  @override
  List<Seg> get intervals => _layers.isNotEmpty
      ? [
          () {
            final shape = _stored;
            final i = kinds.indexWhere((k) => k.name == shape['kind']);
            final seg = Seg(i < 0 ? 0 : i, 1, kinds);
            if (shape['kind'] == 'Bezier') seg.setValues([for (final n in ['x1', 'y1', 'x2', 'y2']) (shape[n] as num? ?? 0).toDouble()]);
            return seg;
          }(),
        ]
      : [
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
    if (_layers.isNotEmpty) {
      // One step on release (one undo); the preview in flight lands first.
      _pending = null;
      await _flight;
      _previewed = false;
      final payload = _payload(curve);
      await c.storeDesk('ease', payload['shape'] as Map<String, dynamic>);
      if (_canSequence) await c.command('sequence', payload);
      return;
    }
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
    c.deskWork.removeListener(_read);
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
