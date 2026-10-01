import 'dart:convert';
import 'dart:ui' show ViewFocusEvent, ViewFocusState;
import 'dart:math' as math;

import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';
import '../foundation/glyphs.dart';
import '../foundation/leaves.dart';

part 'ease_desk/values.dart';
part 'ease_desk/state.dart';
part 'ease_desk/parts.dart';
part 'ease_desk/painters.dart';

class EaseDesk extends StatefulWidget {
  const EaseDesk({super.key, required this.controller, this.leading});
  final EditorSession controller;
  final Widget? leading;
  @override
  State<EaseDesk> createState() => _EaseDeskState();
}

class _EaseDeskState extends State<EaseDesk>
    with
        WidgetsBindingObserver,
        SingleTickerProviderStateMixin,
        _EaseDeskLogic {
  @override
  Widget build(BuildContext context) {
    final segments = _segments;
    final first = _interval;
    final mixed = _mixed;
    final saved = EditorSession.maps(c.deskWork.value['easePresets']);
    final clip = EditorSession.map(c.deskWork.value['curveClip']);
    final presets = [
      ...EditorSession.maps(c.state['easeKinds']),
      if (clip.isNotEmpty) clip,
      ...saved,
    ];
    // Which preset is the shape in force: each object is encoded once, and
    // kept while the status hands back the same object.
    final shapeKey = _keyOf(_shape);
    final presetKeys = [for (final p in presets) _keyOf(p)];
    final sequence = _ghostMode ? _sequence : const <Map<String, dynamic>>[];
    final target = sequence.isNotEmpty
        ? 'Sequence · ${sequence.length} layers · ghosts${!_canApply ? ' · Read only' : ''}'
        : first == null
        ? 'No interval · Workspace'
        : '${first['name']} · ${first['property']} · ${first['frame']}–${first['end']}${segments.length > 1 ? ' · ${segments.length} intervals' : ''}${mixed ? ' · Mixed' : ''}${!_canApply ? ' · Read only' : ''}';
    double? playhead() => first == null
        ? null
        : (c.frame.value - (first['frame'] as num)) /
              ((first['end'] as num) - (first['frame'] as num));
    final railActive = first == null
        ? -1
        : segments.indexWhere(
            (s) =>
                s['layer'] == first['layer'] &&
                s['property'] == first['property'] &&
                s['frame'] == first['frame'],
          );
    return Focus(
      focusNode: _focus,
      onKeyEvent: (_, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.escape &&
            _original != null) {
          _cancel();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: LayoutBuilder(
        builder: (context, viewport) {
          final narrow = viewport.maxWidth < EditorMetrics.s280;
          final compact = viewport.maxHeight < EditorMetrics.sheetWide;
          final shown = _audition ?? _shape;
          final kind = '${_shape['kind']}';
          final previewKind = '${shown['kind']}';
          final contentWidth = viewport.maxWidth - EditorMetrics.s16;
          final side = math.min(
            EditorMetrics.cell - EditorMetrics.s16,
            math.min(
              contentWidth * .5,
              math.max(
                EditorMetrics.s85,
                viewport.maxHeight -
                    EditorMetrics.row -
                    EditorMetrics.s14 -
                    EditorMetrics.bar -
                    EditorMetrics.s16 -
                    EditorMetrics.s8 -
                    EditorMetrics.s70,
              ),
            ),
          );
          final infoWidth = contentWidth - side - EditorMetrics.s8;
          final params = _shape.entries.where((e) => e.value is num).toList()
            ..sort((a, b) {
              if (kind == 'Bezier') {
                const order = ['x1', 'y1', 'x2', 'y2'];
                return order.indexOf(a.key).compareTo(order.indexOf(b.key));
              }
              return a.key.compareTo(b.key);
            });
          Widget action(String label, IconData icon, VoidCallback onPressed) =>
              _EaseIcon(tooltip: label, icon: icon, onPressed: onPressed);
          final graph = Container(
            width: side,
            height: side,
            decoration: BoxDecoration(
              color: EditorInk.dark.easePaper,
              borderRadius: BorderRadius.circular(EditorMetrics.s4),
            ),
            child: LayoutBuilder(
              builder: (context, box) {
                final painter = EaseCurvePainter(
                  colors: EditorTheme.of(context),
                  shape: _shape,
                  handles: true,
                  free: _free,
                  ghost: sequence.isNotEmpty,
                  marks: [
                    for (var i = 0; i < sequence.length; i++)
                      i / (sequence.length - 1),
                  ],
                );
                final size = Size(box.maxWidth, box.maxHeight);
                return Listener(
                  key: const ValueKey('ease-plot'),
                  behavior: HitTestBehavior.opaque,
                  onPointerDown: (e) {
                    if (_pointer != null || e.buttons != 1) return;
                    _endPeek();
                    final handles = _points(_shape['handles']);
                    final index = handles.indexWhere(
                      (p) =>
                          (painter.toPixel(p, size) - e.localPosition)
                              .distance <=
                          12,
                    );
                    if (index < 0) return;
                    _focus.requestFocus();
                    _pointer = e.pointer;
                    _handle = index;
                    _original = Map.of(_shape);
                  },
                  onPointerMove: (e) {
                    if (e.pointer != _pointer || _handle == null) return;
                    _handles.add((
                      _handle!,
                      painter.toCurve(e.localPosition, size),
                    ));
                    _pending = _handles.drained;
                  },
                  onPointerUp: (e) {
                    if (e.pointer != _pointer) return;
                    _pointer = null;
                    _handle = null;
                    _commit();
                  },
                  onPointerCancel: (e) {
                    if (e.pointer == _pointer) _cancel();
                  },
                  child: ListenableBuilder(
                    listenable: c.frame,
                    builder: (context, _) =>
                        _plot(_shape, handles: true, playhead: playhead()),
                  ),
                );
              },
            ),
          );
          final choices = LayoutBuilder(
            builder: (context, box) {
              final columns = math.max(
                1,
                math.min(
                  3,
                  ((box.maxWidth + EditorMetrics.s4) / EditorMetrics.s96)
                      .floor(),
                ),
              );
              final tileWidth =
                  (box.maxWidth - (columns - 1) * EditorMetrics.s4) / columns;
              final labelStyle = DefaultTextStyle.of(context).style
                  .copyWith(fontSize: EditorMetrics.font);
              final labelKey = (
                tileWidth,
                [for (final p in presets) p['kind']].join('|'),
                MediaQuery.textScalerOf(context),
              );
              if (labelKey != _labelKey) {
                _labelKey = labelKey;
                _labelHeight = presets.fold<double>(0, (height, preset) {
                  final label = TextPainter(
                    text: TextSpan(
                      text: _curveName('${preset['kind']}'),
                      style: labelStyle,
                    ),
                    maxLines: 2,
                    textDirection: Directionality.of(context),
                    textScaler: MediaQuery.textScalerOf(context),
                  )..layout(maxWidth: tileWidth - EditorMetrics.s12);
                  return math.max(height, label.height);
                });
              }
              final labelHeight = _labelHeight;
              final tileHeight = math.max(
                EditorMetrics.s70,
                EditorMetrics.s44 + EditorMetrics.s12 + labelHeight,
              );
              return Focus(
                focusNode: _presetFocus,
                onFocusChange: (focused) {
                  if (!focused) _endPeek();
                  _redraw();
                },
                onKeyEvent: (_, event) {
                  if (event is! KeyDownEvent || presets.isEmpty)
                    return KeyEventResult.ignored;
                  final key = event.logicalKey;
                  final delta = key == LogicalKeyboardKey.arrowRight
                      ? 1
                      : key == LogicalKeyboardKey.arrowLeft
                      ? -1
                      : key == LogicalKeyboardKey.arrowDown
                      ? columns
                      : key == LogicalKeyboardKey.arrowUp
                      ? -columns
                      : 0;
                  if (key == LogicalKeyboardKey.enter) {
                    _choose(presets[_focused.clamp(0, presets.length - 1)]);
                    return KeyEventResult.handled;
                  }
                  if (delta == 0 &&
                      key != LogicalKeyboardKey.home &&
                      key != LogicalKeyboardKey.end)
                    return KeyEventResult.ignored;
                  setState(
                    () => _focused =
                        (key == LogicalKeyboardKey.home
                                ? 0
                                : key == LogicalKeyboardKey.end
                                ? presets.length - 1
                                : _focused + delta)
                            .clamp(0, presets.length - 1),
                  );
                  _peek(presets[_focused], _focused);
                  return KeyEventResult.handled;
                },
                child: Semantics(
                  label: 'Easing presets',
                  child: Wrap(
                    spacing: EditorMetrics.s4,
                    runSpacing: EditorMetrics.s4,
                    children: [
                      for (var i = 0; i < presets.length; i++)
                        _PresetTile(
                          index: i,
                          preset: presets[i],
                          width: tileWidth,
                          height: tileHeight,
                          selected: presetKeys[i] == shapeKey,
                          hovered: _hover == i,
                          focused: _presetFocus.hasFocus && _focused == i,
                          free: _free,
                          onEnter: () => _peek(presets[i], i),
                          onExit: _endPeek,
                          onTap: () {
                            _focused = i;
                            _presetFocus.requestFocus();
                            _choose(presets[i]);
                          },
                        ),
                    ],
                  ),
                ),
              );
            },
          );

          final fields = <Widget>[
            for (final param in params)
              _EaseParamField(
                kind: '${_shape['kind']}',
                width:
                    ((compact ? contentWidth : infoWidth) - EditorMetrics.s8) /
                    2,
                name: param.key,
                value: (param.value as num).toDouble(),
                onPreview: (v) async {
                  _original ??= Map.of(_shape);
                  _pending = _model({..._shape, param.key: v});
                  await _pending;
                },
                onCommit: (v) async {
                  _pending = _model({..._shape, param.key: v});
                  await _commit();
                },
                onFinish: () async {
                  if (_original != null) await _commit();
                },
                onCancel: () async => _cancel(),
              ),
          ];
          final info = _EaseInfo(
            width: infoWidth,
            leading: widget.leading,
            kind: kind,
            previewKind: _audition == null ? null : previewKind,
            shown: shown,
            free: _free,
            motion: _motion,
            onPlay: _runMotion,
            fields: compact ? const [] : fields,
          );
          final savedActions = <Widget>[
            action('Copy curve', Glyph.copy_outlined, () async {
              await c.storeDesk('curveClip', Map.of(_shape));
              if (mounted) setState(() => _notice = 'Curve copied');
            }),
            action('Save preset', Glyph.bookmark_add_outlined, () async {
              await c.storeDesk('easePresets', [...saved, Map.of(_shape)]);
              if (mounted) setState(() => _notice = 'Preset saved');
            }),
            action(
              'Use for new keys (now ${_curveName('${c.newKeyShape['kind']}')})',
              Glyph.fiber_new_outlined,
              () async {
                await c.storeDesk('newKeyShape', _payload(_shape));
                if (c.animating) await c.setAnimate(true);
                if (mounted)
                  setState(
                    () => _notice =
                        'New keys: ${_curveName('${_shape['kind']}')}',
                  );
              },
            ),
            if (saved.isNotEmpty)
              action(
                'Clear saved presets',
                Glyph.delete_sweep_outlined,
                () async {
                  await c.storeDesk('easePresets', []);
                  if (mounted)
                    setState(() => _notice = 'Saved presets cleared');
                },
              ),
            Expanded(
              child: EditorTooltip(
                message: target,
                child: Text(
                  _notice ??
                      (sequence.isNotEmpty
                          ? '${sequence.length} layers'
                          : first == null
                          ? 'Workspace'
                          : '${EditorSession.maps(c.state['selectedKeys']).length} keys selected'),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: EditorMetrics.dense,
                    color: EditorTheme.of(context).muted,
                  ),
                ),
              ),
            ),
          ];
          final applyActions = <Widget>[
            _OvershootToggle(
              free: _free,
              narrow: narrow,
              onPressed: () => setState(() => _free = !_free),
            ),
            const SizedBox(width: EditorMetrics.s4),
            _ApplyButton(
              message: _canApply ? 'Apply to selected intervals' : target,
              onPressed: _canApply ? _commit : null,
            ),
          ];
          final footer = SizedBox(
            height: EditorMetrics.bar,
            child: Row(children: [...savedActions, ...applyActions]),
          );
          return ColoredBox(
            color: EditorTheme.of(context).app,
            child: Padding(
              padding: const EdgeInsets.all(EditorMetrics.s8),
              child: Column(
                children: [
                  _EaseTargetRow(
                    target: target,
                    title: first == null
                        ? (sequence.isNotEmpty
                              ? 'Sequence · ${sequence.length} layers'
                              : 'Workspace · no key interval')
                        : '${first['name']} · ${first['property']}  ${first['frame']}–${first['end']} f',
                    frame: c.frame,
                    playhead: playhead,
                  ),
                  const SizedBox(height: EditorMetrics.s2),
                  _EaseRail(
                    target: target,
                    label: first == null
                        ? (sequence.isNotEmpty
                              ? 'Sequence of ${sequence.length} layers'
                              : 'No key interval')
                        : 'Curve runs ${first['frame']} to ${first['end']} f',
                    hasInterval: first != null,
                    frame: c.frame,
                    segments: segments,
                    active: railActive,
                  ),
                  const SizedBox(height: EditorMetrics.s2),
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      graph,
                      const SizedBox(width: EditorMetrics.s8),
                      Expanded(child: info),
                    ],
                  ),
                  const SizedBox(height: EditorMetrics.s4),
                  Expanded(
                    child: ListView(
                      key: const ValueKey('ease-choices-scroll'),
                      padding: EdgeInsets.zero,
                      children: [
                        choices,
                        if (compact && fields.isNotEmpty) ...[
                          const SizedBox(height: EditorMetrics.s8),
                          Wrap(spacing: EditorMetrics.s8, children: fields),
                        ],
                      ],
                    ),
                  ),
                  const SizedBox(height: EditorMetrics.s4),
                  footer,
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}
