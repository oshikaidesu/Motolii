// The live app's colours for the widgets it keeps from the foundation (the Stage's chrome, menus): the same neutral
// ramp as the hf faces, through the foundation's own theme (EditorTheme), so those widgets follow the app.
import 'editor_theme.dart';
import 'metrics.dart';
import 'neutral.dart';

final liveEditorTheme = EditorTheme.chromatic.copyWith(
  name: 'Live',
  colors: const {
    'app': N.g15, // the Stage's pasteboard: a step lighter than the work, so the frame's edge reads
    'panel': Surface.base,
    'raised': Surface.raised,
    'hover': Surface.hover,
    'line': Surface.well,
    'border': Surface.divider,
    'ink': Surface.ink,
    'muted': N.g63, // not Surface.muted (g56): the controls' muted is the tertiary text grey (decision queue)
    'menu': Surface.raised,
    'menuEdge': N.g26,
    'disabledInk': Surface.disabled,
  },
);
