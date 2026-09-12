import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
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

  /// One tile, one look. The seventeen skeletons are built once; a snapshot
  /// only moves the looks, and a [ValueNotifier] wakes the tiles whose look
  /// actually differs — a new selection with the same modes wakes none.
  final _looks = {for (final mode in modes) mode: ValueNotifier(const _Look())};

  /// Whether the desk has anything to blend: the one thing outside the tiles
  /// that a snapshot can change.
  final _idle = ValueNotifier(true);

  EditorSession get c => widget.controller;

  List<Map<String, dynamic>> get _targets => blendTargets(c);
  DocumentSlice get _slice => c.slice('blend', const [
    'path',
    'capabilities',
  ], derived: () => blendReading(c));

  String get _mark => jsonEncode([c.state['path'], c.selectedIds]);

  @override
  void initState() {
    super.initState();
    _selection = _mark;
    _slice.addListener(_sync);
    _look();
  }

  @override
  void dispose() {
    _hoverTimer?.cancel();
    _slice.removeListener(_sync);
    for (final look in _looks.values) look.dispose();
    _idle.dispose();
    if (_previewing != null) c.command('cancelPreview');
    super.dispose();
  }

  /// Read the document once and hand each tile its own look.
  void _look() {
    final targets = _targets;
    // The samples belong to the layer the hover previews on.
    final fresh = panelMap(
      (targets.isEmpty ? c.activeLayer : targets.last)?['blendPreviews'],
    );
    if (fresh.isNotEmpty) _samples = fresh;
    final values = targets.map((l) => '${l['blendMode'] ?? 'Normal'}').toSet();
    final current = values.length == 1 ? values.single : null;
    final live = targets.isNotEmpty && !_applying;
    _idle.value = targets.isEmpty;
    for (final mode in modes)
      _looks[mode]!.value = _Look(
        beds: [
          for (final rgb in _samples[mode] as List? ?? const [])
            Color.fromARGB(
              255,
              ((rgb[0] as num).clamp(0, 1) * 255).round(),
              ((rgb[1] as num).clamp(0, 1) * 255).round(),
              ((rgb[2] as num).clamp(0, 1) * 255).round(),
            ),
        ],
        current: mode == current,
        hovered: _hover == mode,
        live: live,
      );
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
    _look();
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
    _hover = mode;
    _look();
    _hoverTimer?.cancel();
    _hoverTimer = Timer(const Duration(milliseconds: 90), () {
      if (mounted && _hover == mode) _want(mode);
    });
  }

  /// Give the document back now: the pointer is gone, not merely moving.
  void _leave() {
    _hoverTimer?.cancel();
    if (!mounted) return;
    _hover = null;
    _look();
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
    _applying = true;
    _hover = null;
    _look();
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
      if (mounted) {
        _applying = false;
        _look();
      }
    }
  }

  @override
  Widget build(BuildContext context) => Focus(
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
    child: ValueListenableBuilder<bool>(
      valueListenable: _idle,
      builder: (context, idle, grid) =>
          Opacity(opacity: idle ? 0.45 : 1, child: grid),
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
                  _BlendTile(
                    key: ValueKey('blend:$mode'),
                    mode: mode,
                    width: width,
                    look: _looks[mode]!,
                    desk: this,
                  ),
              ],
            ),
          );
        },
      ),
    ),
  );
}

/// What a snapshot can change about one tile: the beds painted across its
/// top, whether the layer already wears the mode, whether the pointer is on
/// it, and whether it can be clicked.
@immutable
class _Look {
  const _Look({
    this.beds = const [],
    this.current = false,
    this.hovered = false,
    this.live = false,
  });
  final List<Color> beds;
  final bool current, hovered, live;
  @override
  bool operator ==(Object other) =>
      other is _Look &&
      current == other.current &&
      hovered == other.hovered &&
      live == other.live &&
      listEquals(beds, other.beds);
  @override
  int get hashCode => Object.hash(Object.hashAll(beds), current, hovered, live);
}

/// One mode. The skeleton stands for the life of the desk; only the parts
/// under the [ValueListenableBuilder] follow the document.
class _BlendTile extends StatelessWidget {
  const _BlendTile({
    super.key,
    required this.mode,
    required this.width,
    required this.look,
    required this.desk,
  });
  final String mode;
  final double width;
  final ValueListenable<_Look> look;
  final BlendPanelState desk;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: width,
    height: EditorMetrics.s36,
    child: MouseRegion(
      onEnter: (_) => desk._aim(mode),
      onExit: (_) {
        if (desk._hover == mode) desk._aim(null);
      },
      child: Material(
        color: EditorTheme.panel,
        child: ValueListenableBuilder<_Look>(
          valueListenable: look,
          builder: (context, look, _) => Semantics(
            button: true,
            selected: look.current,
            label: mode,
            child: InkWell(
              onTap: look.live ? () => desk._apply(mode) : null,
              onFocusChange: (focused) {
                if (focused)
                  desk._aim(mode);
                else if (desk._hover == mode)
                  desk._aim(null);
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
                            for (final bed in look.beds)
                              Expanded(
                                child: ColoredBox(
                                  color: bed,
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
                              color: look.current
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
                          color: look.current
                              ? EditorTheme.accent
                              : look.hovered
                              ? EditorTheme.select
                              : Colors.transparent,
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
    ),
  );
}

/// The unlocked, non-camera layers a blend applies to.
List<Map<String, dynamic>> blendTargets(EditorSession c) {
  final ids = c.selectedIds;
  return [
    for (final l in (c.state['layers'] as List? ?? const []).whereType<Map>())
      if (ids.contains(l['id']) && l['locked'] != true && l['kind'] != 'Camera')
        Map<String, dynamic>.from(l),
  ];
}

/// Everything the tiles show: the mode in force and the specimens for it.
Object blendReading(EditorSession c) => [
  c.selectedIds,
  for (final l in blendTargets(c))
    [l['id'], l['blendMode'], l['blendPreviews']],
  c.activeLayer?['blendPreviews'],
];
