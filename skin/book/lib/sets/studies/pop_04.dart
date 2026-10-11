// Pop 04: three sheets of 20 pop panels each (Easing, Spring, Wiggle), one screenshot per sheet. Each panel is a different picture of
// why you touch the parameter while making a work; drag (or the panel's own gesture) changes it, double-click resets it.
import 'package:widgetbook/widgetbook.dart';

import 'pop_04_ease.dart';
import 'pop_04_kit.dart';
import 'pop_04_spring.dart';
import 'pop_04_wiggle.dart';

WidgetbookComponent pop04Set() => WidgetbookComponent(name: 'Pop 04', useCases: [
      WidgetbookUseCase(name: 'Easing x20', builder: (c) => PopSheet(concept: 'Easing', panels: easePanels)),
      WidgetbookUseCase(name: 'Spring x20', builder: (c) => PopSheet(concept: 'Spring', panels: springPanels)),
      WidgetbookUseCase(name: 'Wiggle x20', builder: (c) => PopSheet(concept: 'Wiggle', panels: wigglePanels)),
    ]);
