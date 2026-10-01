import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter/scheduler.dart';

/// Maps input displacement and its release motion through the same transform.
enum ViewportGestureMode { pan, scrubZoom }

class ViewportMotion {
  ViewportMotion(this.apply);
  final void Function(double scale, double x, double y) apply;
  Ticker? _ticker;
  final _clock = Stopwatch();
  VelocityTracker? _panVelocity, _zoomVelocity;
  Offset _pan = Offset.zero;
  double _zoom = 0, _scale = 1, _frame = 0, _anchor = 0, _y = 0;
  ViewportGestureMode _mode = ViewportGestureMode.pan;
  Duration? _lastTime;
  bool _active = false;

  void begin({
    required ViewportGestureMode mode,
    required double scale,
    required double frame,
    required double anchor,
    required double y,
  }) {
    stop();
    _mode = mode;
    _scale = scale;
    _frame = frame;
    _anchor = anchor;
    _y = y;
    _pan = Offset.zero;
    _zoom = 0;
    _lastTime = null;
    _active = true;
    _clock
      ..reset()
      ..start();
    _panVelocity = VelocityTracker.withKind(PointerDeviceKind.trackpad);
    _zoomVelocity = VelocityTracker.withKind(PointerDeviceKind.trackpad);
  }

  void update(Offset pan, double scale, Duration? time) {
    if (!_active) return;
    _pan = pan;
    _zoom = math.log(math.max(.0001, scale)) * 200;
    final stamp = time ?? _clock.elapsed;
    if (_lastTime == null || stamp > _lastTime!) {
      _panVelocity!.addPosition(stamp, pan);
      _zoomVelocity!.addPosition(stamp, Offset(_zoom, 0));
      _lastTime = stamp;
    }
    _project(pan, _zoom);
  }

  void _project(Offset pan, double zoom) {
    final (logScale, y) = switch (_mode) {
      ViewportGestureMode.pan => (zoom / 200, _y - pan.dy),
      ViewportGestureMode.scrubZoom => (zoom / 200 - pan.dy * .006, _y),
    };
    final scale = (_scale * math.exp(logScale.clamp(-20.0, 20.0))).clamp(
      .1,
      40.0,
    );
    apply(scale, _frame * scale - (_anchor + pan.dx), y);
  }

  void end() {
    if (!_active) return;
    _active = false;
    final panVelocity = _panVelocity!.getVelocity().pixelsPerSecond;
    final zoomVelocity = _zoomVelocity!.getVelocity().pixelsPerSecond.dx;
    final origin = [_pan.dx, _pan.dy, _zoom];
    final speed = [panVelocity.dx, panVelocity.dy, zoomVelocity];
    final simulations = List<Simulation?>.generate(
      3,
      (i) => speed[i].abs() < 50
          ? null
          : ClampingScrollSimulation(
              position: origin[i],
              velocity: speed[i].clamp(-8000.0, 8000.0),
            ),
    );
    if (simulations.every((s) => s == null)) return;
    _ticker = Ticker((elapsed) {
      final t = elapsed.inMicroseconds / 1000000;
      final value = List<double>.generate(
        3,
        (i) => simulations[i]?.x(t) ?? origin[i],
      );
      _project(Offset(value[0], value[1]), value[2]);
      if (simulations.every((s) => s == null || s.isDone(t))) stop();
    })..start();
  }

  void stop() {
    _active = false;
    _ticker?.dispose();
    _ticker = null;
    _clock.stop();
  }

  void dispose() => stop();
}
