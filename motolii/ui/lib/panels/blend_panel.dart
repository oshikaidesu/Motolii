import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';
import '../session/read_model.dart';

/// Blend desk. One tile per mode; the tile *is* the sample — the selected
/// layer's own colour laid over the beds of `blend_preview.rs` (black → white
/// ramp, then blue and orange). Point at a tile to see it on Stage, click to
/// keep it. Nothing else: no cards, no title, no presets.
class BlendPanel extends StatefulWidget {
  const BlendPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<BlendPanel> createState() => BlendPanelState();
}

class BlendPanelState extends State<BlendPanel> {
  /// W3C Compositing order: normal, darken, lighten, contrast, inversion,
  /// component. Neighbours share a character, so the strip reads as a family
  /// without a heading over it.
  static const modes = [
    'Normal',
    'Darken',
    'Multiply',
    'ColorBurn',
    'Lighten',
    'Screen',
    'ColorDodge',
    'Add',
    'Overlay',
    'SoftLight',
    'HardLight',
    'Difference',
    'Exclusion',
    'Hue',
    'Saturation',
    'Color',
    'Luminosity',
  ];

  /// Kept between snapshots so the tiles never blink to empty while the
  /// selection or the frame moves.
  Map<String, dynamic> _samples = const {};
  String _selection = '';
  String? _hover;
  String? _previewing;
  Timer? _hoverTimer;
  bool _applying = false;
  String? _wanted;
  Future<void>? _flight;

  EditorSession get c => widget.controller;

  List<Map<String, dynamic>> get _targets => c.layers
      .where(
        (l) =>
            c.selectedIds.contains(l['id']) &&
            l['locked'] != true &&
            l['kind'] != 'Camera',
      )
      .toList();

  String get _mark => jsonEncode([c.state['path'], c.selectedIds]);

  @override
  void initState() {
    super.initState();
    _selection = _mark;
    c.document.addListener(_sync);
  }

  @override
  void dispose() {
    _hoverTimer?.cancel();
    c.document.removeListener(_sync);
    if (_previewing != null) c.command('cancelPreview');
    super.dispose();
  }

  /// A new selection or a new document drops the preview this desk owns; the
  /// tiles under the pointer must not keep showing someone else's layer.
  void _sync() {
    if (!mounted) return;
    if (_selection != _mark) {
      _selection = _mark;
      _hoverTimer?.cancel();
      _hover = null;
      _want(null);
    }
    setState(() {});
  }

  /// Latest wish wins: hovering across the grid must not queue one round trip
  /// per tile passed over.
  void _want(String? mode) {
    _wanted = mode;
    _flight ??= _pump();
  }

  Future<void> _pump() async {
    await Future<void>.value();
    try {
      while (mounted && _wanted != _previewing) {
        // A selection that lost its editable layers can only be cancelled.
        final targets = _targets;
        if (targets.isEmpty) _wanted = null;
        final mode = _wanted;
        if (mode == null) {
          _previewing = null;
          await c.command('cancelPreview');
        } else {
          _previewing = mode;
          await c.command('previewBlend', {
            'layer': targets.last['id'],
            'mode': mode,
          });
        }
      }
    } finally {
      _flight = null;
    }
  }

  /// Aim at a tile, or at nothing. Both ends wait out the same beat, so
  /// sweeping the pointer across the grid sends one preview, not one per tile
  /// entered and one cancel per tile left.
  void _aim(String? mode) {
    if (mode != null &&
        (_applying || _targets.isEmpty || !panelCan(c, 'previewBlend')))
      return;
    setState(() => _hover = mode);
    _hoverTimer?.cancel();
    _hoverTimer = Timer(const Duration(milliseconds: 90), () {
      if (mounted && _hover == mode) _want(mode);
    });
  }

  /// Give the document back now: the pointer is gone, not merely moving.
  void _leave() {
    _hoverTimer?.cancel();
    if (!mounted) return;
    setState(() => _hover = null);
    _want(null);
  }

  /// One click, one undo step: `setAttrs` cancels the running preview in the
  /// port before it applies, so the preview never lands in history.
  Future<void> _apply(String mode) async {
    _hoverTimer?.cancel();
    if (_applying || !panelCan(c, 'setAttrs')) return;
    final mark = _mark;
    final ids = _targets
        .where((l) => '${l['blendMode'] ?? 'Normal'}' != mode)
        .map((l) => l['id'])
        .toList();
    setState(() {
      _applying = true;
      _hover = null;
    });
    try {
      if (ids.isEmpty) {
        // Already this mode: nothing to record, but the preview must go.
        _want(null);
        await _flight;
      } else {
        // The port folds the running preview away before it applies, so the
        // desk must not send a cancel of its own — that would be a second step.
        _wanted = null;
        _previewing = null;
        await _flight;
        if (mounted && mark == _mark)
          await c.command('setAttrs', {
            'layers': ids,
            'patch': {'blendMode': mode},
          });
      }
    } finally {
      if (mounted) setState(() => _applying = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final targets = _targets;
    // The samples belong to the layer the hover previews on.
    final fresh = panelMap(
      (targets.isEmpty ? c.activeLayer : targets.last)?['blendPreviews'],
    );
    if (fresh.isNotEmpty) _samples = fresh;
    final values = targets.map((l) => '${l['blendMode'] ?? 'Normal'}').toSet();
    final current = values.length == 1 ? values.single : null;
    final live = targets.isNotEmpty && !_applying;
    return Focus(
      onKeyEvent: (_, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.escape &&
            _previewing != null) {
          _leave();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      onFocusChange: (focused) {
        if (!focused) _leave();
      },
      child: Opacity(
        opacity: targets.isEmpty ? 0.45 : 1,
        child: LayoutBuilder(
          builder: (context, box) {
            const gap = EditorMetrics.s3;
            final room = box.maxWidth - EditorMetrics.s6 * 2;
            final columns = math.max(
              2,
              (room + gap) ~/ (EditorMetrics.s70 + gap),
            );
            final width = (room - gap * (columns - 1)) / columns;
            return SingleChildScrollView(
              padding: const EdgeInsets.all(EditorMetrics.s6),
              child: Wrap(
                spacing: gap,
                runSpacing: gap,
                children: [
                  for (final mode in modes)
                    _tile(
                      mode,
                      width: width,
                      current: mode == current,
                      live: live,
                    ),
                ],
              ),
            );
          },
        ),
      ),
    );
  }

  Widget _tile(
    String mode, {
    required double width,
    required bool current,
    required bool live,
  }) {
    final beds = _samples[mode] as List? ?? const [];
    final ring = current
        ? EditorTheme.accent
        : _hover == mode
        ? EditorTheme.select
        : Colors.transparent;
    return SizedBox(
      width: width,
      height: EditorMetrics.s36,
      child: Semantics(
        button: true,
        selected: current,
        label: mode,
        child: MouseRegion(
          onEnter: (_) => _aim(mode),
          onExit: (_) {
            if (_hover == mode) _aim(null);
          },
          child: Material(
            color: EditorTheme.panel,
            child: InkWell(
              key: ValueKey('blend:$mode'),
              onTap: live ? () => _apply(mode) : null,
              onFocusChange: (focused) {
                if (focused)
                  _aim(mode);
                else if (_hover == mode)
                  _aim(null);
              },
              child: Stack(
                fit: StackFit.expand,
                children: [
                  Column(
                    children: [
                      SizedBox(
                        height: EditorMetrics.s22,
                        child: Row(
                          children: [
                            for (final rgb in beds)
                              Expanded(
                                child: ColoredBox(
                                  color: Color.fromARGB(
                                    255,
                                    ((rgb[0] as num).clamp(0, 1) * 255).round(),
                                    ((rgb[1] as num).clamp(0, 1) * 255).round(),
                                    ((rgb[2] as num).clamp(0, 1) * 255).round(),
                                  ),
                                  child: const SizedBox.expand(),
                                ),
                              ),
                          ],
                        ),
                      ),
                      Expanded(
                        child: Align(
                          child: Text(
                            mode,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(
                              fontSize: EditorMetrics.dense,
                              height: 1,
                              color: current
                                  ? EditorTheme.accent
                                  : EditorTheme.muted,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                  IgnorePointer(
                    child: DecoratedBox(
                      decoration: BoxDecoration(
                        border: Border.all(
                          color: ring,
                          width: EditorMetrics.s2,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
