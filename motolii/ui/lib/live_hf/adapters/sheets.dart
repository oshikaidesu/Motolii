import 'dart:async';

import 'package:flutter/widgets.dart';

import '../../hf/insp/panel.dart';
import '../../hf/insp/rows.dart';
import '../../hf/shell/place.dart';
import '../../hf/shell/sheet.dart';
import '../../session/editor_session.dart';
import '../../session/export_actions.dart';

/// Export as a short task under its control: what will be written (read-only facts: the composition's size, rate and
/// the format), which range (a choice: the whole document, or marker to marker around the playhead, with the frames
/// and seconds that makes), then one action — Export… (a save panel, then the host's export job) — with Cancel beside
/// it (it stops a running job, else closes). Enter exports, Esc closes. [anchor] is the asking control, in global
/// coordinates; the task opens under it and leaves the Stage in view.
Future<void> showExportSheet(BuildContext context, EditorSession c, {Rect? anchor}) {
  final a = anchor ?? _topRight(context);
  final key = GlobalKey<_ExportState>();
  return showHfPopover(context, anchor: a, title: 'Export', width: 320, primary: () => key.currentState?.exportAction, body: (_, close) => _Export(key: key, c: c, close: close));
}

Rect _topRight(BuildContext context) {
  final box = context.findRenderObject() as RenderBox?;
  final size = box?.size ?? const Size(1280, 44);
  final at = box?.localToGlobal(Offset(size.width - 12, 0)) ?? Offset.zero;
  return Rect.fromLTWH(at.dx - 88, at.dy, 88, 40);
}

class _Export extends StatefulWidget {
  const _Export({super.key, required this.c, required this.close});
  final EditorSession c;
  final VoidCallback close;
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

  bool get _running => const ['running', 'cancelling'].contains(EditorSession.map(c.state['export'])['phase']);

  /// The primary action while it can run (Enter uses it too).
  VoidCallback? get exportAction {
    if (_running || !c.supports('export')) return null;
    final (start, end) = exportRange(c, c.state, markers: markers);
    return () async {
      if (await startExport(c, start, end)) {
        poll?.cancel();
        poll = pollExport(c);
      }
    };
  }

  @override
  Widget build(BuildContext context) {
    final s = c.state, job = EditorSession.map(s['export']);
    final running = _running;
    final (start, end) = exportRange(c, s, markers: markers);
    final fps = (s['fps'] as num? ?? 30).toDouble();
    final rate = fps == fps.roundToDouble() ? fps.toStringAsFixed(0) : fps.toStringAsFixed(2);
    final hasMarkers = EditorSession.maps(s['markers']).isNotEmpty;
    final phase = job['phase'];
    final failed = phase == 'failed' || phase == 'error';
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 10, 12, 12),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        HfFormRow('Output', HfFact('${s['width']} × ${s['height']} · $rate fps · MP4')),
        HfFormRow(
          'Range',
          HfChoice<bool>(
            options: [(false, 'Whole', true), (true, 'Markers', hasMarkers)],
            value: markers,
            onChanged: running ? null : (v) => setState(() => markers = v),
          ),
        ),
        HfFormRow('', HfFact('$start – $end · ${((end - start) / fps).toStringAsFixed(2)} s', color: H.text3)),
        if (phase != null && phase != 'idle')
          HfFormRow(
            'Status',
            HfFact(switch (phase) {
              'running' => 'Writing ${job['done'] ?? 0} / ${job['total'] ?? end - start}',
              'cancelling' => 'Stopping…',
              'done' || 'complete' => 'Written ${job['path'] ?? ''}',
              _ when failed => 'Failed: ${job['error'] ?? ''}',
              _ => '$phase',
            }, color: failed ? H.record : H.text2),
          ),
        const SizedBox(height: 4),
        Row(mainAxisAlignment: MainAxisAlignment.end, children: [
          HfAction(running ? 'Stop' : 'Cancel', onTap: running ? (c.supports('cancelExport') ? () => c.command('cancelExport') : null) : widget.close),
          HfAction('Export…', kind: HfActionKind.primary, onTap: exportAction),
        ]),
      ]),
    );
  }
}

/// The Composition settings as a short task under the top bar's right side: the frame's size, rate and length as the
/// Inspector's own rows (no second title, count or filter), each change written through `composition`. The Stage stays
/// in view (the settings change what it shows) and Colors, where Background colour hands the ground, is not dimmed.
Future<void> showCompositionSheet(BuildContext context, EditorSession c, {Rect? anchor}) {
  final store = CompositionStore(c);
  return showHfPopover(context, anchor: anchor ?? _topRight(context), title: 'Composition', width: 300, body: (_, close) => Padding(
        padding: const EdgeInsets.fromLTRB(12, 4, 12, 10),
        child: ParamSheet(store, thingId: 'composition'),
      )).whenComplete(store.dispose);
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
      // Classic's size presets; a size none names highlights none
      {'id': 'size', 'label': 'Size', 'kind': 'enum', 'choices': [for (final p in _sizes) p.$1], 'value': _sizes.indexWhere((p) => p.$2 == s['width'] && p.$3 == s['height'])},
      {'id': 'width', 'label': 'Width', 'kind': 'u32', 'value': s['width'] ?? 1920, 'min': 16, 'max': 8192, 'unit': 'px', 'hero': true},
      {'id': 'height', 'label': 'Height', 'kind': 'u32', 'value': s['height'] ?? 1080, 'min': 16, 'max': 8192, 'unit': 'px', 'hero': true},
      {'id': 'fps', 'label': 'Frame rate', 'kind': 'enum', 'choices': [for (final r in _rates) (r.$1 / r.$2).toStringAsFixed(r.$2 == 1 ? 0 : 3)], 'value': i < 0 ? 4 : i},
      {'id': 'durationFrames', 'label': 'Length', 'kind': 'u32', 'value': s['durationFrames'] ?? 300, 'min': 1, 'unit': 'f'},
      // AE's Composition Settings > Background Color: the ground where no environment layer is (Classic's presets)
      {'id': 'background', 'label': 'Background', 'kind': 'enum', 'choices': [for (final g in _greys) g.$1], 'value': _grey(s['background'])},
      // any colour: the Colors wheel edits the ground (Classic's swatch, focusColor Background)
      {'id': 'backgroundColor', 'label': 'Background colour', 'kind': 'color', 'route': 'Colors', 'value': _hex(s['background'])},
    ];
  }

  static const _sizes = [('16:9', 1920, 1080), ('9:16', 1080, 1920), ('1:1', 1080, 1080), ('4K', 3840, 2160)];

  static String _hex(Object? raw) {
    final v = raw is List && raw.length >= 3 ? [for (final x in raw.take(3)) ((x as num).toDouble().clamp(0, 1) * 255).round()] : [0, 0, 0];
    return '#${v.map((c) => c.toRadixString(16).padLeft(2, '0')).join().toUpperCase()}';
  }

  @override
  void route(String to, [String? from]) {
    super.route(to, from);
    if (from == 'backgroundColor' && c.supports('focusColor')) c.focusColor({'slot': 'Background'});
  }

  static const _greys = [('Black', 0.0), ('Dark', 0.12), ('Grey', 0.5), ('White', 1.0)];

  static double _alpha(Object? raw) => raw is List && raw.length >= 4 ? (raw[3] as num).toDouble() : 1.0;

  static int _grey(Object? raw) {
    final v = raw is List && raw.length >= 3 ? [for (final x in raw.take(3)) (x as num).toDouble()] : null;
    if (v == null) return -1;
    return _greys.indexWhere((g) => v.every((x) => (x - g.$2).abs() < .002));
  }

  void _write(String id) {
    final v = row(id)['value'];
    final args = switch (id) {
      'fps' => {'fpsNum': _rates[(v as num).toInt()].$1, 'fpsDen': _rates[v.toInt()].$2},
      'size' => {'width': _sizes[(v as num).toInt()].$2, 'height': _sizes[v.toInt()].$3},
      // a preset changes the grey and keeps the ground's alpha (a transparent ground stays transparent)
      'background' => {'background': [for (var i = 0; i < 3; i++) _greys[(v as num).toInt()].$2, _alpha(c.state['background'])]},
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
