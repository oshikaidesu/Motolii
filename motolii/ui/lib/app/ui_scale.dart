import 'dart:async';

import 'package:flutter/rendering.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../theme/metrics.dart';
import '../theme/neutral.dart';

/// UI Scale as a user setting: a whole percent, one percent a step, for every project. It is an application preference kept next to
/// the window's other saved settings (never in a project file); the tokens read it through [UiScale], and [UiScaleScope] makes the
/// window follow it. Nothing of the work scales: the document, the Stage's picture (its native view is asked for at the logical size
/// times the device's ratio), the camera, transforms, time zoom and export are untouched.
class LiveUiScale {
  LiveUiScale._();
  static final instance = LiveUiScale._();

  /// The settings key: an integer percent.
  static const settingsKey = 'uiScalePercent';

  /// What an earlier window kept instead: a fraction of the same 100 % (`1.0` is 100 %). Read once when the new key is missing.
  static const legacyKey = 'hfScale';

  /// The user's size, in whole percent (the provisional range is [UiScale.minPercent]..[UiScale.maxPercent]).
  int get percent => UiScale.percent;

  /// Where a chosen size is kept (the shell gives the settings store once); every way of changing the size keeps it.
  void Function(int percent)? persist;

  /// Sets the user's size (clamped, whole). True when it changed.
  bool set(int next, {bool keep = true}) {
    final changed = UiScale.setPercent(next);
    if (changed && keep) persist?.call(UiScale.percent);
    return changed;
  }

  bool bigger() => set(percent + 1);
  bool smaller() => set(percent - 1);
  bool reset() => set(UiScale.resetPercent);

  /// The saved settings read back at start: the percent, else the old fraction (migrated: the new key is written once),
  /// else 100 %. Anything unreadable is 100 %.
  void restore(Map<String, dynamic> settings) {
    final saved = settings[settingsKey], legacy = settings[legacyKey];
    if (saved is num && saved.isFinite) {
      set(saved.round(), keep: false);
    } else if (legacy is num && legacy.isFinite) {
      set((legacy * 100).round());
    } else {
      set(UiScale.resetPercent, keep: false);
    }
  }
}

/// The root of the window's UI Scale. Tokens are derived values read at build time, so when the scale (or the display's pixel ratio)
/// changes every element below is marked dirty, like a hot reload does: each widget builds again with the new values and keeps its
/// State (scroll positions, focus, text being typed), and every render object lays out and paints again. Menus, sheets and
/// overlays live below this (they build from tokens too). No subtree is transformed and none is re-keyed.
class UiScaleScope extends StatefulWidget {
  const UiScaleScope({super.key, required this.child});
  final Widget child;
  @override
  State<UiScaleScope> createState() => _UiScaleScopeState();
}

class _UiScaleScopeState extends State<UiScaleScope> {
  bool _queued = false;

  @override
  void initState() {
    super.initState();
    UiScale.changes.addListener(_changed);
  }

  @override
  void dispose() {
    UiScale.changes.removeListener(_changed);
    super.dispose();
  }

  void _changed() {
    if (_queued) return;
    if (SchedulerBinding.instance.schedulerPhase == SchedulerPhase.idle) {
      _rebuildAll();
      return;
    }
    _queued = true;
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _queued = false;
      if (mounted) _rebuildAll();
    });
  }

  void _rebuildAll() {
    void dirty(Element e) {
      e.markNeedsBuild();
      e.visitChildren(dirty);
    }

    (context as Element).visitChildren(dirty);
    for (final view in RendererBinding.instance.renderViews) {
      view.reassemble();
    }
    SchedulerBinding.instance.ensureVisualUpdate();
  }

  @override
  Widget build(BuildContext context) {
    // the display this window is on: a SNAP token reads it, so a window dragged to another screen rebuilds once
    if (UiScale.adoptDevicePixelRatio(MediaQuery.devicePixelRatioOf(context))) {
      SchedulerBinding.instance.addPostFrameCallback((_) {
        if (mounted) _rebuildAll();
      });
    }
    return widget.child;
  }
}

/// The size just chosen, said for a moment where the eye is (top centre), then gone: the change is otherwise silent.
class UiScaleReadout extends StatefulWidget {
  const UiScaleReadout({super.key});
  @override
  State<UiScaleReadout> createState() => _UiScaleReadoutState();
}

class _UiScaleReadoutState extends State<UiScaleReadout> {
  bool _shown = false;
  int _last = UiScale.percent;
  Timer? _hide;

  @override
  void initState() {
    super.initState();
    UiScale.changes.addListener(_changed);
  }

  void _changed() {
    if (_last == UiScale.percent) return; // a change of display, not of the size
    _last = UiScale.percent;
    _hide?.cancel();
    setState(() => _shown = true);
    _hide = Timer(const Duration(milliseconds: 1100), () {
      if (mounted) setState(() => _shown = false);
    });
  }

  @override
  void dispose() {
    _hide?.cancel();
    UiScale.changes.removeListener(_changed);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => IgnorePointer(
        child: AnimatedOpacity(
          opacity: _shown ? 1 : 0,
          duration: const Duration(milliseconds: 120),
          child: Align(
            alignment: const Alignment(0, -.82),
            child: Container(
              key: const ValueKey('ui-scale-readout'),
              padding: EdgeInsets.symmetric(horizontal: 10, vertical: 5),
              decoration: BoxDecoration(color: N.veilHi, borderRadius: BorderRadius.circular(5), border: Border.all(color: N.g26)),
              child: Text('UI ${UiScale.percent}%', style: TextStyle(fontFamily: 'Menlo', fontSize: 12, color: N.g95, decoration: TextDecoration.none)),
            ),
          ),
        ),
      );
}
