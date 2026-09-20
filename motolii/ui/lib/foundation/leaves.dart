/// The leaves the window presses, reads and scrolls with, on
/// `flutter/widgets` alone. Each reproduces what the Material leaf it
/// replaced drew under the editor's theme — the same lifts, sizes and inks —
/// so a panel swapping one for the other looks the same. What Material also
/// did (ripples, tap-target padding, its own metrics) never showed here.
///
/// One family to a file, so the catalogue reads as a shelf:
/// [leaves/press.dart] what a pointer presses, [leaves/track.dart] what runs
/// along a line, [leaves/field.dart] what takes typing,
/// [leaves/floating.dart] what floats in the overlay, [leaves/dialog.dart]
/// what comes in front and holds, [leaves/choice.dart] what picks one value.
library;

export 'leaves/choice.dart';
export 'leaves/dialog.dart';
export 'leaves/field.dart';
export 'leaves/floating.dart';
export 'leaves/press.dart';
export 'leaves/track.dart';
