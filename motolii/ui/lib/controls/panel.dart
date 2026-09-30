/// The catalogue of panel parts: what a panel presses, scrubs, turns, toggles
/// and frames its rows with, on `flutter/widgets` and the [leaves] alone.
///
/// One family to a file, so the shelf reads at a glance:
/// [panel_controls/numeric.dart] the number well and its ladder,
/// [panel_controls/drag.dart] the drag-preview-commit route they share,
/// [panel_controls/dials.dart] the ring and the square,
/// [panel_controls/toggles.dart] on / off and the lamps,
/// [panel_controls/fields.dart] the box typing happens in,
/// [panel_controls/frames.dart] the bars, cards and heads that hold rows,
/// [panel_controls/scale.dart] the editor's scale and what sets it.
library;

export 'leaves.dart' show EditorChoice;
export 'panel/dials.dart';
export 'panel/drag.dart';
export 'panel/fields.dart';
export 'panel/frames.dart';
export 'panel/numeric.dart';
export 'panel/scale.dart';
export 'panel/toggles.dart';
