// The live app's colours for the widgets it keeps from the foundation (the Stage's chrome, menus): the same neutral
// ramp as the hf faces, through the foundation's own theme (EditorTheme), so those widgets follow the app.
import '../foundation/theme.dart';
import '../hf/neutral.dart';

final liveEditorTheme = EditorTheme.chromatic.copyWith(
  name: 'Live',
  colors: const {
    'app': N.g15, // the Stage's pasteboard: a step lighter than the work, so the frame's edge reads
    'panel': N.g10,
    'raised': N.g13,
    'hover': N.g15,
    'line': N.g07,
    'border': N.g20,
    'ink': N.g95,
    'muted': N.g63,
    'menu': N.g13,
    'menuEdge': N.g26,
    'disabledInk': N.g44,
  },
);
