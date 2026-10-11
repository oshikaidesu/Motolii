// set: Pop 05. Three sheets of 20 pop panels each (Time remap, Loop, Camera); one screenshot shows a whole sheet.
import 'package:widgetbook/widgetbook.dart';

import 'pop_05_camera.dart';
import 'pop_05_kit.dart';
import 'pop_05_loop.dart';
import 'pop_05_remap.dart';

WidgetbookComponent pop05Set() => WidgetbookComponent(name: 'Pop 05', useCases: [
      WidgetbookUseCase(name: 'Time remap x20', builder: (_) => PopSheet(concept: 'Time remap', panels: remapPanels(), rate: sp)),
      WidgetbookUseCase(name: 'Loop x20', builder: (_) => PopSheet(concept: 'Loop', panels: loopPanels())),
      WidgetbookUseCase(name: 'Camera x20', builder: (_) => PopSheet(concept: 'Camera', panels: cameraPanels())),
    ]);
