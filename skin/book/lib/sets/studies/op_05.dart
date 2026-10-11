// set: OP 05. Three sheets of 20 OP-1-language panels each (Time remap, Loop, Camera); one screenshot shows a whole sheet.
import 'package:widgetbook/widgetbook.dart';

import 'op_05_camera.dart';
import 'op_05_kit.dart';
import 'op_05_loop.dart';
import 'op_05_remap.dart';

WidgetbookComponent op05Set() => WidgetbookComponent(name: 'OP 05', useCases: [
      WidgetbookUseCase(name: 'Time remap x20', builder: (_) => OpSheet(concept: 'Time remap', panels: remapPanels(), rate: spd)),
      WidgetbookUseCase(name: 'Loop x20', builder: (_) => OpSheet(concept: 'Loop', panels: loopPanels())),
      WidgetbookUseCase(name: 'Camera x20', builder: (_) => OpSheet(concept: 'Camera', panels: cameraPanels())),
    ]);
