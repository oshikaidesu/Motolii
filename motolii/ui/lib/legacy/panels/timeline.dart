import 'dart:math' as math;

import 'package:flutter/foundation.dart' show ValueListenable;
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

import '../../theme/editor_metrics.dart';
import '../../controls/panel.dart';
import '../../theme/editor_theme.dart';
import '../../controls/leaves.dart';
import '../../session/editor_session.dart';
import '../timeline_core/frame.dart';
import '../timeline_core/grip.dart';
import 'timeline/layout.dart';
import 'timeline/menu.dart';
import 'timeline/overview.dart';
import 'timeline/paint.dart';
import '../timeline_core/view.dart';

/// What a host's own toolbar needs from the Timeline: play/pause and the zoom operations Classic's own bar's
/// buttons call. Nothing about the ruler, the tracks, the keyframes or the overview strip below them — those,
/// and the editing they carry (selection, scrub, key move, resize, group behaviour, undo/redo, preview/commit),
/// stay exactly as they are, whichever buttons sit above them.
abstract class TimelineToolbarApi {
  ValueListenable<bool> get playing;
  void togglePlayback();
  double get zoomPercent;
  void setZoomPercent(double percent);
  void zoomOut();
  void zoomIn();
  void fit();
}

class TimelinePanel extends StatefulWidget {
  const TimelinePanel({
    super.key,
    required this.controller,
    this.topBarButtons,
  });
  final EditorSession controller;

  /// A host's own play/zoom controls (New's, in hf presentation), left of the overview strip, which stays
  /// Classic's own. Null keeps Classic's own buttons there too.
  final Widget Function(BuildContext context, TimelineToolbarApi api)?
  topBarButtons;
  @override
  State<TimelinePanel> createState() => _TimelinePanelState();
}

/// 板の組み立てと再生時刻。責任は四つの mixin が持ち、ここはそれを組むだけ。
class _TimelinePanelState extends State<TimelinePanel>
    with
        TimelineFrame<TimelinePanel>,
        TimelineGrip<TimelinePanel>,
        TimelineView<TimelinePanel>,
        TimelineMenu<TimelinePanel>,
        TimelineEditorMenu<TimelinePanel>
    implements TimelineToolbarApi {
  @override
  EditorSession get timelineSession => widget.controller;
  // ---- TimelineToolbarApi: exactly what the bar's own buttons call, nothing about the ruler/tracks/keys ---------
  @override
  ValueListenable<bool> get playing => widget.controller.playing;
  @override
  void togglePlayback() => widget.controller.togglePlayback();
  @override
  double get zoomPercent => pixelsPerFrame / 4 * 100;
  @override
  void setZoomPercent(double percent) => zoom(percent * .04 / pixelsPerFrame);
  @override
  void zoomOut() => zoom(
    ((pixelsPerFrame / 4 * 100).round() - 1).clamp(3, 1000) *
        .04 /
        pixelsPerFrame,
  );
  @override
  void zoomIn() =>
      zoom(((pixelsPerFrame / 4 * 100).round() + 1) * .04 / pixelsPerFrame);
  @override
  void fit() => setState(
    () => pixelsPerFrame = math.max(
      .1,
      (_lastBounds.maxWidth - labelWidth) /
          math.max(1, overviewExtentWithoutFrame),
    ),
  );
  BoxConstraints _lastBounds = BoxConstraints.tight(Size.zero);

  @override
  Widget build(BuildContext context) => Focus(
    key: const ValueKey('timeline-bounded-layout'),
    focusNode: focus,
    onKeyEvent: key,
    child: ColoredBox(
      color: EditorTheme.of(context).panel,
      child: LayoutBuilder(
        builder: (context, bounds) {
          viewportWidth = bounds.maxWidth;
          _lastBounds = bounds;
          reportVisible();
          return navigation(
            ListenableBuilder(
              listenable: timelineSlice,
              child: Column(
                children: [
                  _bar(bounds),
                  GestureDetector(
                    supportedDevices: const {
                      PointerDeviceKind.mouse,
                      PointerDeviceKind.touch,
                      PointerDeviceKind.stylus,
                    },
                    onTapDown: (e) {
                      if (e.localPosition.dx >= labelWidth)
                        requestSeek(frameAt(e.localPosition.dx));
                    },
                    onHorizontalDragUpdate: (e) {
                      if (e.localPosition.dx >= labelWidth)
                        requestSeek(frameAt(e.localPosition.dx));
                    },
                    child: SizedBox(
                      height: timelineRulerHeight,
                      width: double.infinity,
                      child: RepaintBoundary(
                        child: ListenableBuilder(
                          listenable: Listenable.merge([
                            widget.controller.frame,
                            scrubFrame,
                            timelineSlice,
                            scrolled,
                          ]),
                          builder: (context, _) => CustomPaint(
                            painter: TimelinePainter(
                              colors: EditorTheme.of(context),
                              ink: EditorInk.of(context),
                              labelWidth: labelWidth,
                              rows: const [],
                              selected: const [],
                              keys: const [],
                              frame:
                                  scrubFrame.value ??
                                  widget.controller.frame.value,
                              scale: pixelsPerFrame,
                              offset: offset,
                              duration: duration,
                              fps:
                                  (widget.controller.state['fps'] as num? ?? 30)
                                      .toDouble(),
                              markers: EditorSession.maps(
                                widget.controller.state['markers'],
                              ),
                              ruler: true,
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  Expanded(
                    child: EditorScrollbar(
                      controller: vertical,
                      child: SingleChildScrollView(
                        controller: vertical,
                        physics: const ClampingScrollPhysics(
                          parent: NeverScrollableScrollPhysics(),
                        ),
                        child: DragTarget<Map<String, dynamic>>(
                          onWillAcceptWithDetails: (d) =>
                              d.data['asset'] != null && has('placeAsset'),
                          onMove: aimAsset,
                          onLeave: (_) => leaveAsset(),
                          onAcceptWithDetails: acceptAsset,
                          builder: (_, __, ___) => GestureDetector(
                            onSecondaryTapDown: menu,
                            child: Listener(
                              onPointerDown: begin,
                              onPointerMove: move,
                              onPointerUp: end,
                              onPointerCancel: (_) => cancel(),
                              child: ListenableBuilder(
                                listenable: Listenable.merge([
                                  widget.controller.frame,
                                  scrubFrame,
                                  timelineSlice,
                                  scrolled,
                                ]),
                                builder: (context, _) => SizedBox(
                                  key: rowsKey,
                                  width: bounds.maxWidth,
                                  height: math.max(
                                    layout.height,
                                    bounds.maxHeight - 66,
                                  ),
                                  child: RepaintBoundary(
                                    child: CustomPaint(
                                      key: const ValueKey('timeline-lanes'),
                                      painter: TimelinePainter(
                                        colors: EditorTheme.of(context),
                                        ink: EditorInk.of(context),
                                        labelWidth: labelWidth,
                                        rows: tracks,
                                        rowDropGuide: rowDropGuide,
                                        rowDropInside: rowDropInside,
                                        containers: layout.roots,
                                        activeLane: activeLane,
                                        selected: widget.controller.selectedIds,
                                        keys: selectedKeys,
                                        frame:
                                            scrubFrame.value ??
                                            widget.controller.frame.value,
                                        scale: pixelsPerFrame,
                                        offset: offset,
                                        duration: duration,
                                        fps:
                                            (widget.controller.state['fps']
                                                        as num? ??
                                                    30)
                                                .toDouble(),
                                        markers: EditorSession.maps(
                                          widget.controller.state['markers'],
                                        ),
                                        marquee:
                                            gesture == 'marquee' &&
                                                start != null &&
                                                current != null
                                            ? Rect.fromPoints(start!, current!)
                                            : null,
                                        dragKeys: gesture == 'keys'
                                            ? initialKeys
                                            : settlingKeys,
                                        delta: gesture == 'keys'
                                            ? deltaFrames
                                            : settlingDelta,
                                        // Every bar in the grip is drawn from
                                        // its press-time timing plus the delta:
                                        // the document's rows only catch up
                                        // when the preview reply lands, and a
                                        // bar that waits for that trails the
                                        // one under the pointer.
                                        dragLayers:
                                            [
                                              'move',
                                              'trimIn',
                                              'trimOut',
                                              'slip',
                                            ].contains(gesture)
                                            ? {
                                                for (final r in timingRows)
                                                  r.id: drawnTiming(r),
                                              }
                                            : const {},
                                      ),
                                    ),
                                  ),
                                ),
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                  ),
                  _TimelineScrollStrip(
                    timeline: timelineSlice,
                    horizontal: horizontal,
                    viewportWidth: bounds.maxWidth,
                    measure: () => (labelWidth, contentWidth),
                  ),
                ],
              ),
              builder: (context, column) => Stack(
                children: [
                  column!,
                  _NameColumnGrip(
                    left: layout.nameWidth - EditorMetrics.s3,
                    onDragStart: (x) {
                      resizeStart = layout.nameWidth - layout.indentation;
                      resizePointerStart = x;
                    },
                    onDragUpdate: (x) => setState(() {
                      baseNameWidth = (resizeStart + x - resizePointerStart)
                          .clamp(114.0 - layout.indentation, 162.0);
                      relane();
                    }),
                    onDragCancel: () => setState(() {
                      baseNameWidth = resizeStart;
                      relane();
                    }),
                    onReset: () => setState(() {
                      baseNameWidth = 138;
                      relane();
                    }),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    ),
  );

  /// 帯は寸法だけで建つ。書類を見る二つ — 全体図と Marker — は自分で
  /// 書類を購読するので、帯そのものは建て直さなくてよい。
  Widget _bar(BoxConstraints bounds) => SizedBox(
    height: EditorMetrics.s22,
    child: Row(
      children: [
        if (widget.topBarButtons != null)
          widget.topBarButtons!(context, this)
        else ...[
          const SizedBox(width: EditorMetrics.s6),
          ValueListenableBuilder<bool>(
            valueListenable: widget.controller.playing,
            builder: (_, playing, __) => EditorButton(
              playing ? 'Ⅱ' : '▶',
              widget.controller.togglePlayback,
              selected: playing,
              tooltip: 'Play / Pause · Space',
            ),
          ),
          EditorButton(
            '−',
            () => zoom(
              ((pixelsPerFrame / 4 * 100).round() - 1).clamp(3, 1000) *
                  .04 /
                  pixelsPerFrame,
            ),
          ),
          EditorPercentField(
            value: pixelsPerFrame / 4 * 100,
            min: 3,
            max: 1000,
            label: 'Timeline zoom',
            onChanged: (v) => zoom(v * .04 / pixelsPerFrame),
          ),
          EditorButton(
            '+',
            () => zoom(
              ((pixelsPerFrame / 4 * 100).round() + 1) * .04 / pixelsPerFrame,
            ),
          ),
          EditorButton('Fit', () {
            setState(
              () => pixelsPerFrame = math.max(
                .1,
                (bounds.maxWidth - labelWidth) /
                    math.max(1, overviewExtentWithoutFrame),
              ),
            );
          }),
        ],
        Expanded(
          child: SizedBox(
            height: EditorMetrics.s18,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s8),
              child: LayoutBuilder(
                builder: (context, overview) {
                  void navigate(double x) {
                    if (!horizontal.hasClients) return;
                    final target =
                        x /
                            math.max(1, overview.maxWidth) *
                            overviewExtent *
                            pixelsPerFrame -
                        (bounds.maxWidth - labelWidth) / 2;
                    horizontal.jumpTo(
                      target.clamp(
                        0,
                        horizontal.positions.last.maxScrollExtent,
                      ),
                    );
                  }

                  // The jump is on the press, below the arena: beside the
                  // double-tap, onTapDown would wait out the press deadline.
                  return Listener(
                    onPointerDown: (e) {
                      if (e.buttons == kPrimaryButton)
                        navigate(e.localPosition.dx);
                    },
                    child: GestureDetector(
                      supportedDevices: const {
                        PointerDeviceKind.mouse,
                        PointerDeviceKind.touch,
                        PointerDeviceKind.stylus,
                      },
                      onHorizontalDragUpdate: (e) =>
                          navigate(e.localPosition.dx),
                      onDoubleTap: () {
                        setState(
                          () => pixelsPerFrame = math.max(
                            .1,
                            (bounds.maxWidth - labelWidth) /
                                math.max(1, overviewExtentWithoutFrame),
                          ),
                        );
                      },
                      child: ListenableBuilder(
                        listenable: Listenable.merge([
                          widget.controller.frame,
                          scrubFrame,
                          timelineSlice,
                          scrolled,
                        ]),
                        builder: (context, _) => RepaintBoundary(
                          child: CustomPaint(
                            size: Size(overview.maxWidth, EditorMetrics.s18),
                            painter: ArrangementOverview(
                              colors: EditorTheme.of(context),
                              layers: widget.controller.layers,
                              duration: duration,
                              extent: overviewExtent,
                              frame:
                                  scrubFrame.value ??
                                  widget.controller.frame.value,
                              offset: offset,
                              scale: pixelsPerFrame,
                              viewportWidth: bounds.maxWidth - labelWidth,
                            ),
                          ),
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
          ),
        ),
        ListenableBuilder(
          listenable: timelineSlice,
          builder: (context, _) => EditorButton(
            'Marker',
            has('addMarker')
                ? () => widget.controller.command('addMarker')
                : null,
          ),
        ),
      ],
    ),
  );
}

/// The horizontal scrollbar under the lanes; sized from the timeline's
/// measures, so it listens to that signal alone.
class _TimelineScrollStrip extends StatelessWidget {
  const _TimelineScrollStrip({
    required this.timeline,
    required this.horizontal,
    required this.viewportWidth,
    required this.measure,
  });
  final Listenable timeline;
  final ScrollController horizontal;
  final double viewportWidth;

  /// (label column width, content width) as the timeline now measures them.
  final (double, double) Function() measure;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: timeline,
    builder: (context, _) {
      final (labelWidth, contentWidth) = measure();
      return Padding(
        padding: EdgeInsets.only(left: labelWidth),
        child: SizedBox(
          height: EditorMetrics.s12,
          child: EditorScrollbar(
            controller: horizontal,
            thumbVisibility: true,
            child: SingleChildScrollView(
              controller: horizontal,
              physics: const ClampingScrollPhysics(
                parent: NeverScrollableScrollPhysics(),
              ),
              scrollDirection: Axis.horizontal,
              child: SizedBox(
                width: math.max(viewportWidth - labelWidth, contentWidth),
                height: EditorMetrics.s12,
              ),
            ),
          ),
        ),
      );
    },
  );
}

/// The grip between the names and the lanes: drag to widen the name column,
/// double-click to put it back.
class _NameColumnGrip extends StatelessWidget {
  const _NameColumnGrip({
    required this.left,
    required this.onDragStart,
    required this.onDragUpdate,
    required this.onDragCancel,
    required this.onReset,
  });
  final double left;
  final ValueChanged<double> onDragStart, onDragUpdate;
  final VoidCallback onDragCancel, onReset;
  @override
  Widget build(BuildContext context) => Positioned(
    left: left,
    top: EditorMetrics.s22,
    bottom: EditorMetrics.s12,
    width: EditorMetrics.s6,
    child: MouseRegion(
      cursor: SystemMouseCursors.resizeColumn,
      child: GestureDetector(
        supportedDevices: const {
          PointerDeviceKind.mouse,
          PointerDeviceKind.touch,
          PointerDeviceKind.stylus,
        },
        behavior: HitTestBehavior.opaque,
        onHorizontalDragStart: (e) => onDragStart(e.globalPosition.dx),
        onHorizontalDragUpdate: (e) => onDragUpdate(e.globalPosition.dx),
        onHorizontalDragCancel: onDragCancel,
        onDoubleTap: onReset,
        child: const SizedBox.expand(),
      ),
    ),
  );
}
