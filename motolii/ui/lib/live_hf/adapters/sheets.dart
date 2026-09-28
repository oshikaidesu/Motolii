import 'dart:async';

import 'package:flutter/widgets.dart';

import '../../hf/insp/panel.dart';
import '../../hf/insp/rows.dart';
import '../../hf/shell/place.dart';
import '../../hf/shell/sheet.dart';
import '../../session/editor_session.dart';
import '../../session/export_actions.dart';

/// The Export sheet: what will be written, which range (the whole document, or marker to marker around the
/// playhead), Export (a save panel, then the host's export job), its progress and Cancel while it runs.
Future<void> showExportSheet(BuildContext context, EditorSession c) => showHfSheet(context, title: 'Export', body: (_, close) => _Export(c: c));

class _Export extends StatefulWidget {
  const _Export({required this.c});
  final EditorSession c;
  @override
  State<_Export> createState() => _ExportState();
}

class _ExportState extends State<_Export> {
  EditorSession get c => widget.c;
  bool markers = false;
  Timer? poll;

  @override
  void initState() {
    super.initState();
    c.slice('liveExport', const ['export', 'markers', 'width', 'height', 'fps', 'durationFrames']).addListener(_changed);
  }

  @override
  void dispose() {
    c.slice('liveExport', const ['export', 'markers', 'width', 'height', 'fps', 'durationFrames']).removeListener(_changed);
    poll?.cancel();
    super.dispose();
  }

  void _changed() => setState(() {});

  @override
  Widget build(BuildContext context) {
    final s = c.state, job = EditorSession.map(s['export']);
    final running = const ['running', 'cancelling'].contains(job['phase']);
    final (start, end) = exportRange(c, s, markers: markers);
    final fps = (s['fps'] as num? ?? 30).toDouble();
    Widget line(String t, {Color color = H.text2}) => Padding(padding: const EdgeInsets.only(bottom: 8), child: Text(t, style: H.m(12, color: color)));
    return Padding(
      padding: const EdgeInsets.fromLTRB(18, 14, 18, 16),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        line('${s['width']} × ${s['height']} · ${fps.toStringAsFixed(2)} fps · MP4'),
        line('Frames $start – $end  (${((end - start) / fps).toStringAsFixed(2)} s)'),
        const SizedBox(height: 4),
        Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Padding(padding: const EdgeInsets.only(top: 9), child: Text('RANGE', style: H.s(10, w: FontWeight.w600, ls: 1.2, color: H.text3))),
          Expanded(
            child: Wrap(alignment: WrapAlignment.end, runSpacing: 6, children: [
              HfKey('Whole', on: !markers, onTap: running ? null : () => setState(() => markers = false)),
              HfKey('Marker to marker', on: markers, onTap: running || EditorSession.maps(s['markers']).isEmpty ? null : () => setState(() => markers = true)),
            ]),
          ),
        ]),
        const SizedBox(height: 14),
        if (job['phase'] != null && job['phase'] != 'idle')
          line(switch (job['phase']) {
            'running' => 'Writing ${job['done'] ?? 0} / ${job['total'] ?? end - start}',
            'cancelling' => 'Stopping…',
            'done' => 'Written ${job['path'] ?? ''}',
            'failed' || 'error' => 'Failed: ${job['error'] ?? ''}',
            _ => '${job['phase']}',
          }, color: job['phase'] == 'failed' || job['phase'] == 'error' ? H.record : H.text2),
        Wrap(alignment: WrapAlignment.end, runSpacing: 6, children: [
          if (running) HfKey('Cancel', onTap: c.supports('cancelExport') ? () => c.command('cancelExport') : null),
          HfKey('Export…', main: !running, onTap: running || !c.supports('export')
              ? null
              : () async {
                  if (await startExport(c, start, end)) {
                    poll?.cancel();
                    poll = pollExport(c);
                  }
                }),
        ]),
      ]),
    );
  }
}

/// The Composition sheet: the frame's size, rate and length as the Inspector's own rows, each change written
/// through `composition`.
Future<void> showCompositionSheet(BuildContext context, EditorSession c) {
  final store = CompositionStore(c);
  return showHfSheet(context, title: 'Composition', width: 360, body: (_, close) => SizedBox(height: 300, child: InspectorBody(store: store, subject: 'Composition'))).whenComplete(store.dispose);
}

const _rates = [(24000, 1001), (24, 1), (25, 1), (30000, 1001), (30, 1), (50, 1), (60000, 1001), (60, 1)];

class CompositionStore extends ParamStore {
  CompositionStore(this.c) : super(_rows(c));
  final EditorSession c;

  static List<Map<String, dynamic>> _rows(EditorSession c) {
    final s = c.state;
    final rate = (s['fpsNum'] as num? ?? 30, s['fpsDen'] as num? ?? 1);
    final i = _rates.indexWhere((r) => r.$1 == rate.$1 && r.$2 == rate.$2);
    return [
      {'id': 'width', 'label': 'Width', 'kind': 'u32', 'value': s['width'] ?? 1920, 'min': 16, 'max': 8192, 'unit': 'px', 'hero': true},
      {'id': 'height', 'label': 'Height', 'kind': 'u32', 'value': s['height'] ?? 1080, 'min': 16, 'max': 8192, 'unit': 'px', 'hero': true},
      {'id': 'fps', 'label': 'Frame rate', 'kind': 'enum', 'choices': [for (final r in _rates) (r.$1 / r.$2).toStringAsFixed(r.$2 == 1 ? 0 : 3)], 'value': i < 0 ? 4 : i},
      {'id': 'durationFrames', 'label': 'Length', 'kind': 'u32', 'value': s['durationFrames'] ?? 300, 'min': 1, 'unit': 'f'},
      // AE's Composition Settings > Background Color: the ground where no environment layer is (Classic's presets)
      {'id': 'background', 'label': 'Background', 'kind': 'enum', 'choices': [for (final g in _greys) g.$1], 'value': _grey(s['background'])},
    ];
  }

  static const _greys = [('Black', 0.0), ('Dark', 0.12), ('Grey', 0.5), ('White', 1.0)];

  static int _grey(Object? raw) {
    final v = raw is List && raw.length >= 3 ? [for (final x in raw.take(3)) (x as num).toDouble()] : null;
    if (v == null) return -1;
    return _greys.indexWhere((g) => v.every((x) => (x - g.$2).abs() < .002));
  }

  void _write(String id) {
    final v = row(id)['value'];
    final args = switch (id) {
      'fps' => {'fpsNum': _rates[(v as num).toInt()].$1, 'fpsDen': _rates[v.toInt()].$2},
      'background' => {'background': [for (var i = 0; i < 3; i++) _greys[(v as num).toInt()].$2, 1.0]},
      _ => {id: (v as num).round()},
    };
    c.command('composition', args);
  }

  @override
  void commit(String id) {
    super.commit(id);
    _write(id);
  }
}
