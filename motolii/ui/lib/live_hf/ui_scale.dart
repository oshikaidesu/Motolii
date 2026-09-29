import 'dart:async';

import 'package:flutter/widgets.dart';
import '../hf/neutral.dart';

/// The live UI's size: one number scales every piece of chrome together (menus and sheets too, because the root
/// viewport scales the overlay they live in), and nothing of the work: the document, the Stage's picture (its native
/// window is asked for at the device's ratio times this factor, so it stays sharp), the camera, transforms, time zoom
/// and export are untouched. Classic's UI Scale (foundation/panel_controls/scale.dart: EditorScale and
/// EditorScaledViewport) is the mechanism; this owns live's number.
class LiveUiScale {
  LiveUiScale._() {
    percent.addListener(() => factor.value = base * percent.value);
  }
  static final instance = LiveUiScale._();

  /// The user's size, in whole percent steps of the new 100 %.
  static const min = .70, max = 1.30, step = .01;

  /// The settings key (Classic's own `scale` is a factor of Classic's sizes, a different number).
  static const settingsKey = 'hfScale';

  /// The new 100 %: the part of the reference-frame geometry (the 1536x1024 Product Home frame the hf faces are drawn
  /// in) the UI is drawn at. Changing Motolii's density as a whole is this one number.
  static const base = 1.0;

  /// What the user chose: 1.0 is 100 %.
  final percent = ValueNotifier<double>(1.0);

  /// What the root viewport scales by: the base times the user's choice.
  final factor = ValueNotifier<double>(base);

  /// Where a chosen size is kept (the shell gives the settings store once); every way of changing the size keeps it.
  void Function(double percent)? persist;

  /// Sets the user's size (clamped, whole percent). True when it changed.
  bool set(double next, {bool keep = true}) {
    final v = (next.clamp(min, max) * 100).round() / 100;
    if (v == percent.value) return false;
    percent.value = v;
    if (keep) persist?.call(v);
    return true;
  }

  bool bigger() => set(percent.value + step);
  bool smaller() => set(percent.value - step);
  bool reset() => set(1.0);

  /// A saved value read back at start (anything unreadable is 100 %).
  void restore(Object? saved) => set(saved is num && saved.isFinite ? saved.toDouble() : 1.0, keep: false);
}

/// The size just chosen, said for a moment where the eye is (top centre), then gone: the change is otherwise silent.
class UiScaleReadout extends StatefulWidget {
  const UiScaleReadout({super.key});
  @override
  State<UiScaleReadout> createState() => _UiScaleReadoutState();
}

class _UiScaleReadoutState extends State<UiScaleReadout> {
  final ui = LiveUiScale.instance;
  bool _shown = false;
  Timer? _hide;

  @override
  void initState() {
    super.initState();
    ui.percent.addListener(_changed);
  }

  void _changed() {
    _hide?.cancel();
    setState(() => _shown = true);
    _hide = Timer(const Duration(milliseconds: 1100), () {
      if (mounted) setState(() => _shown = false);
    });
  }

  @override
  void dispose() {
    _hide?.cancel();
    ui.percent.removeListener(_changed);
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
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
              decoration: BoxDecoration(color: N.veilHi, borderRadius: BorderRadius.circular(5), border: Border.all(color: N.g26)),
              child: Text('UI ${(ui.percent.value * 100).round()}%', style: const TextStyle(fontFamily: 'Menlo', fontSize: 12, color: N.g95, decoration: TextDecoration.none)),
            ),
          ),
        ),
      );
}
