import 'package:flutter/scheduler.dart';

/// Where the time goes between a hand and a pixel: marks placed at the boundaries an input crosses (pointer, command sent,
/// reply, render asked, render answered, next frame), each with microseconds and the frame it fell in. Off unless a test or a
/// probe run turns it on; a mark then costs one list add.
class LatencyProbe {
  static bool on = false;
  static final events = <({String name, int us, int frame})>[];
  static final _clock = Stopwatch()..start();
  static int frame = 0;
  static bool _hooked = false;

  static void enable() {
    on = true;
    events.clear();
    if (_hooked) return;
    _hooked = true;
    SchedulerBinding.instance.addPersistentFrameCallback((_) => frame++);
  }

  static void mark(String name) {
    if (!on) return;
    events.add((name: name, us: _clock.elapsedMicroseconds, frame: frame));
  }

  /// The next frame that is built after now.
  static void markNextFrame(String name) {
    if (!on) return;
    SchedulerBinding.instance.addPostFrameCallback((_) => mark(name));
    SchedulerBinding.instance.ensureVisualUpdate();
  }
}
