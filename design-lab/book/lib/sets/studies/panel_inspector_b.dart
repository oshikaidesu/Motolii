// Panel drafts, Inspector set B: long property lists (I7), links (I9), copy one aspect (I11), effect stack (I8), bulk / stagger (I12), and the cross-cutting X problems that live in an Inspector.
// One use case per OPTION of research/options-inspector.md; each is a PANEL at real size with real-looking content, light local interaction only.
// Nothing here talks to another panel: every use case owns its state. Knobs switch states and variants.
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';
import '../inspector_gui_set.dart' show GDoc, Spec, XyzStack;
import '../inspector/inspector_parts.dart' show Hov;
import '../panels/media_parts.dart' show MiniSlider, Pill, QuietButton, StarButton, hexOf;

part 'panel_inspector_b_data.dart';
part 'panel_inspector_b_parts.dart';
part 'panel_inspector_b_i7.dart';
part 'panel_inspector_b_i9.dart';
part 'panel_inspector_b_i11.dart';
part 'panel_inspector_b_i8.dart';
part 'panel_inspector_b_i12.dart';
part 'panel_inspector_b_x.dart';
part 'panel_inspector_b_sheets.dart';

List<WidgetbookUseCase> _starting(List<WidgetbookUseCase> all, List<String> prefixes) => [for (final u in all) if (prefixes.any(u.name.startsWith)) u];

/// Release order = list order: each component opens with its Parts @200% sheet, then the existing drafts. The orchestrator registers the list.
List<WidgetbookComponent> panelInspectorBSets() {
  final i7 = _i7Cases(), i8 = _i8Cases(), i9 = _i9Cases(), i11 = _i11Cases(), i12 = _i12Cases(), x = _xCases();
  WidgetbookComponent c(String name, WidgetbookUseCase parts, List<WidgetbookUseCase> cases) => WidgetbookComponent(name: name, useCases: [parts, ...cases]);
  return [
    c('Inspector I7 Fold', _sheetFold(), _starting(i7, ['I7-a'])),
    c('Inspector I7 Find', _sheetFind(), _starting(i7, ['I7-b', 'I7-ab'])),
    c('Inspector I7 Pin Recent', _sheetPin(), _starting(i7, ['I7-c', 'I7-d'])),
    c('Inspector X Controls', _sheetControls(), _starting(x, ['X2', 'X3', 'X4'])),
    c('Inspector X States', _sheetStates(), _starting(x, ['X5', 'X6', 'X7', 'X8'])),
    c('Inspector I8 Stack', _sheetStack(), i8),
    c('Inspector I11 Clipboard', _sheetClip(), _starting(i11, ['I11-a', 'I11-b'])),
    c('Inspector I11 Eyedrop Shelf', _sheetPickShelf(), _starting(i11, ['I11-c', 'I11-d'])),
    c('Inspector I9 Whip', _sheetWhip(), _starting(i9, ['I9-a'])),
    c('Inspector I9 Lasso', _sheetLasso(), _starting(i9, ['I9-b'])),
    c('Inspector I9 Macro Inline', _sheetMacro(), _starting(i9, ['I9-c', 'I9-d'])),
    c('Inspector I12 Mixed', _sheetMixed(), _starting(i12, ['I12-a'])),
    c('Inspector I12 Stagger Grab', _sheetStagger(), _starting(i12, ['I12-b', 'I12-c', 'I12-d'])),
  ];
}
