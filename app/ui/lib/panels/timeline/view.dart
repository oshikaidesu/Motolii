import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../input/viewport_motion.dart';
import '../timeline.dart';
import 'frame.dart';
import 'grip.dart';

/// Timeline の見る位置 — 流す・寄る・惰性を止める、そして鍵盤。
/// 書類は一切動かさない。escape だけは掴み手へ返すので、そこへ寄りかかる。
mixin TimelineView on State<TimelinePanel>, TimelineFrame, TimelineGrip {
  Offset navigationOrigin = Offset.zero;
  int navigationRevision = 0;
  bool navigating = false;
  ViewportMotion? _motion;
  ViewportMotion get motion => _motion ??= ViewportMotion(navigateView);

  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    final k = event.logicalKey;
    if (k == LogicalKeyboardKey.escape) {
      cancel();
      widget.controller.cancelPreview();
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowLeft ||
        k == LogicalKeyboardKey.arrowRight) {
      final delta =
          (k == LogicalKeyboardKey.arrowLeft ? -1 : 1) *
          (HardwareKeyboard.instance.isShiftPressed ? 10 : 1);
      if (selectedKeys.isNotEmpty && has('moveKeys'))
        widget.controller.command('moveKeys', {'deltaFrames': delta});
      else
        requestSeek(math.max(0, widget.controller.frame.value + delta));
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void navigateView(double scale, double x, double y) {
    final nextScale = scale.clamp(.1, 40.0);
    final visible = math.max(1.0, viewportWidth - labelWidth);
    final targetX = x.clamp(
      0.0,
      math.max(0.0, overviewExtent * nextScale - visible),
    );
    final revision = ++navigationRevision;
    if (nextScale != pixelsPerFrame) setState(() => pixelsPerFrame = nextScale);
    void apply() {
      if (!mounted || revision != navigationRevision) return;
      if (horizontal.hasClients) {
        final position = horizontal.positions.last;
        if (position.hasContentDimensions)
          position.jumpTo(
            targetX
                .clamp(position.minScrollExtent, position.maxScrollExtent)
                .toDouble(),
          );
      }
      if (vertical.hasClients) {
        final position = vertical.positions.last;
        if (position.hasContentDimensions)
          position.jumpTo(
            y.clamp(position.minScrollExtent, position.maxScrollExtent),
          );
      }
    }

    apply();
    WidgetsBinding.instance.addPostFrameCallback((_) => apply());
  }

  double get verticalOffset =>
      vertical.hasClients ? vertical.positions.last.pixels : 0;
  void zoomAt(double factor, double x) {
    final anchorX = (x - labelWidth).clamp(
      0.0,
      math.max(1.0, viewportWidth - labelWidth),
    );
    final at = (offset + anchorX) / pixelsPerFrame;
    final scale = (pixelsPerFrame * factor).clamp(.1, 40.0);
    navigateView(scale, at * scale - anchorX, verticalOffset);
  }

  void zoom(double factor) => zoomAt(
    factor,
    labelWidth + math.max(1.0, viewportWidth - labelWidth) / 2,
  );
  void stopMomentum() {
    _motion?.stop();
    navigationRevision++;
    for (final controller in [horizontal, vertical]) {
      if (controller.hasClients) {
        final position = controller.positions.last;
        if (position is ScrollPositionWithSingleContext) position.goIdle();
      }
    }
  }

  Widget navigation(Widget child) => GestureDetector(
    supportedDevices: const {PointerDeviceKind.trackpad},
    onScaleStart: (e) {
      stopMomentum();
      navigating = true;
      navigationOrigin = e.localFocalPoint;
      final anchor = math.max(0.0, e.localFocalPoint.dx - labelWidth);
      motion.begin(
        mode: overTimeRuler(e.localFocalPoint)
            ? ViewportGestureMode.scrubZoom
            : ViewportGestureMode.pan,
        scale: pixelsPerFrame,
        frame: (offset + anchor) / pixelsPerFrame,
        anchor: anchor,
        y: verticalOffset,
      );
    },
    onScaleUpdate: (e) => motion.update(
      e.localFocalPoint - navigationOrigin,
      e.scale,
      e.sourceTimeStamp,
    ),
    onScaleEnd: (_) {
      navigating = false;
      navigationRevision++;
      motion.end();
    },
    child: Listener(
      onPointerDown: (_) => stopMomentum(),
      onPointerSignal: (event) {
        if (event is! PointerScrollEvent || navigating) return;
        GestureBinding.instance.pointerSignalResolver.register(event, (_) {
          stopMomentum();
          if (primary || overTimeRuler(event.localPosition))
            zoomAt(
              math.exp(-event.scrollDelta.dy * .002),
              event.localPosition.dx,
            );
          else if (HardwareKeyboard.instance.isShiftPressed)
            navigateView(
              pixelsPerFrame,
              offset + event.scrollDelta.dy + event.scrollDelta.dx,
              verticalOffset,
            );
          else
            navigateView(
              pixelsPerFrame,
              offset + event.scrollDelta.dx,
              verticalOffset + event.scrollDelta.dy,
            );
        });
      },
      child: child,
    ),
  );

  @override
  void dispose() {
    _motion?.dispose();
    super.dispose();
  }
}
