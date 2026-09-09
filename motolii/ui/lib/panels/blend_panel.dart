import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';
import 'native_visual_sample.dart';

class BlendPanel extends StatefulWidget {
  const BlendPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<BlendPanel> createState() => BlendPanelState();
}

class BlendPanelState extends State<BlendPanel> with WidgetsBindingObserver {
  /// Families in the W3C Compositing order: darken, lighten, contrast,
  /// inversion, component. A gap between families is the only grouping cue.
  static const families = [
    ['Normal'],
    ['Darken', 'Multiply', 'ColorBurn'],
    ['Lighten', 'Screen', 'ColorDodge', 'Add'],
    ['Overlay', 'SoftLight', 'HardLight'],
    ['Difference', 'Exclusion'],
    ['Hue', 'Saturation', 'Color', 'Luminosity'],
  ];
  EditorSession get c => widget.controller;
  late final String _interaction = 'blend-desk:${identityHashCode(this)}';
  int _previewEpoch = 0;
  String _source = '';
  String? _hover;
  Timer? _hoverTimer;
  int? _owner;
  bool _applying = false;
  Map<String, dynamic>? _desired;
  Future<void>? _flight;
  List<Map<String, dynamic>> get _targets => c.layers
      .where(
        (l) =>
            c.selectedIds.contains(l['id']) &&
            l['locked'] != true &&
            !['Camera', 'Audio', 'Null', 'Stage'].contains(l['kind']),
      )
      .toList();
  String get _selection => jsonEncode([c.state['path'], c.selectedIds]);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    c.document.addListener(_sync);
    c.playing.addListener(_sync);
    _sync();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) _leave();
  }

  @override
  void dispose() {
    _hoverTimer?.cancel();
    c.document.removeListener(_sync);
    c.playing.removeListener(_sync);
    WidgetsBinding.instance.removeObserver(this);
    _desired = null;
    _previewEpoch++;
    final controller = c;
    final flight = _flight ?? Future<void>.value();
    unawaited(
      flight.then((_) async {
        if (_owner case final owner?)
          await controller.command('cancelPreview', {'owner': owner});
      }),
    );
    super.dispose();
  }

  void _sync() {
    if (!mounted) return;
    if (_source != _selection) {
      _hoverTimer?.cancel();
      _source = _selection;
      _hover = null;
      _queue(null);
    }
    setState(() {});
  }

  void _queue(Map<String, dynamic>? desired) {
    _desired = desired;
    _previewEpoch++;
    _flight ??= _pump();
  }

  Future<void> _pump() async {
    // Defer until _flight has been assigned, including the empty cancellation case.
    await Future<void>.value();
    try {
      var done = -1;
      while (mounted && done != _previewEpoch) {
        done = _previewEpoch;
        final desired = _desired;
        if (desired == null) {
          final owner = _owner;
          _owner = null;
          if (owner != null) await c.command('cancelPreview', {'owner': owner});
        } else {
          final source = _selection;
          final args = {...desired};
          final op = args.remove('op') as String;
          await c.command(op, {...args, 'interaction': _interaction});
          _owner = c.state['previewInteraction'] == _interaction
              ? c.state['previewOwner'] as int?
              : null;
          if (source != _selection) {
            _desired = null;
            _previewEpoch++;
          }
        }
      }
    } finally {
      _flight = null;
    }
  }

  void _inspect(String mode) {
    if (_applying ||
        _targets.isEmpty ||
        !c.supports('previewBlend'))
      return;
    setState(() => _hover = mode);
    _hoverTimer?.cancel();
    _hoverTimer = Timer(const Duration(milliseconds: 100), () {
      if (!mounted || _hover != mode) return;
      _queue({
        'op': 'previewBlend',
        'layers': _targets.map((l) => l['id']).toList(),
        'mode': mode,
      });
    });
  }

  void _leave() {
    _hoverTimer?.cancel();
    if (!mounted) return;
    setState(() {
      _hover = null;
    });
    _queue(null);
  }

  Future<void> _apply(String mode) async {
    _hoverTimer?.cancel();
    if (_applying || !c.supports('setAttrs')) return;
    final source = _selection;
    final ids = _targets
        .where((l) => (l['blendMode'] ?? 'Normal') != mode)
        .map((l) => l['id'])
        .toList();
    setState(() {
      _applying = true;
      _hover = null;
    });
    _queue(null);
    await _flight;
    try {
      if (mounted && source == _selection && ids.isNotEmpty) {
        await c.command('setAttrs', {
          'layers': ids,
          'patch': {'blendMode': mode},
        });
      }
    } finally {
      if (mounted) {
        _applying = false;
        _sync();
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final targets = _targets;
    final values = targets.map((l) => '${l['blendMode'] ?? 'Normal'}').toSet();
    final mode = values.length == 1 ? values.single : null;
    final enabled = targets.isNotEmpty && !_applying;
    return Focus(
      onKeyEvent: (_, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.escape) {
          _leave();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      onFocusChange: (focused) {
        if (!focused) _leave();
      },
      child: Opacity(
        opacity: targets.isEmpty ? .4 : 1,
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(EditorMetrics.s8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              for (final family in families) ...[
                if (family != families.first)
                  const SizedBox(height: EditorMetrics.s16),
                Wrap(
                  spacing: EditorMetrics.s6,
                  runSpacing: EditorMetrics.s6,
                  children: [
                    for (final item in family)
                      _tile(item, current: mode == item, enabled: enabled),
                  ],
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _tile(String item, {required bool current, required bool enabled}) {
    final ring = current
        ? EditorTheme.accent
        : _hover == item
        ? EditorTheme.select
        : Colors.transparent;
    return Semantics(
      button: true,
      selected: current,
      label: item,
      child: MouseRegion(
        onEnter: (_) => _inspect(item),
        onExit: (_) {
          if (_hover == item) _leave();
        },
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          width: EditorMetrics.thumb,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(EditorMetrics.s8),
            border: Border.all(color: ring, width: EditorMetrics.s2),
          ),
          child: ClipRRect(
            borderRadius: BorderRadius.circular(EditorMetrics.s6),
            child: AspectRatio(
              aspectRatio: 1.6,
              child: Material(
                color: EditorTheme.line,
                child: InkWell(
                  key: ValueKey('blend:$item'),
                  onTap: enabled ? () => _apply(item) : null,
                  onFocusChange: (focused) {
                    if (focused)
                      _inspect(item);
                    else if (_hover == item)
                      _leave();
                  },
                  child: NativeVisualSample(
                    controller: c,
                    request: {'kind': 'blendBehavior', 'mode': item},
                    fit: BoxFit.cover,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
