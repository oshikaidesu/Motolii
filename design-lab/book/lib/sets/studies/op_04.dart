// OP 04: three sheets of 20 panels each (Easing, Spring, Wiggle) in the OP-1 screen language: black ground, monoline drawings,
// encoder colours (blue, green, red) for the three values, thin stroke numerals. Each panel is a different picture of why you
// touch the parameter; the drawing is the control (drag, or the panel's own gesture), double-tap resets.
import 'package:widgetbook/widgetbook.dart';

import 'op_04_ease.dart';
import 'op_04_kit.dart';
import 'op_04_spring.dart';
import 'op_04_wiggle.dart';

WidgetbookComponent op04Set() => WidgetbookComponent(name: 'OP 04', useCases: [
      WidgetbookUseCase(name: 'Easing x20', builder: (c) => OpSheet(concept: 'Easing', params: const ['in', 'out', 'overshoot'], panels: easePanels)),
      WidgetbookUseCase(name: 'Spring x20', builder: (c) => OpSheet(concept: 'Spring', params: const ['stiffness', 'damping', 'mass'], panels: springPanels)),
      WidgetbookUseCase(name: 'Wiggle x20', builder: (c) => OpSheet(concept: 'Wiggle', params: const ['frequency', 'amplitude', 'smoothness'], panels: wigglePanels)),
    ]);
