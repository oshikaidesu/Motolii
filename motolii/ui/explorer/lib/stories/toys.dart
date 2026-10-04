// Toys: the Inspector's parameters as objects to hold, drafted on paper beside the product's own panels. Not wired to anything,
// so no document or host is opened: a toy is a widget with a value of its own.
import 'package:flutter/widgets.dart';
import 'package:motolii_ui/theme/neutral.dart';
import 'package:widgetbook/widgetbook.dart';

import '../paper/toys.dart';

WidgetbookUseCase _sheet(String name, Widget Function() sheet, double width, double height) => WidgetbookUseCase(
      name: name,
      builder: (context) {
        final w = context.knobs.double.slider(label: 'Width', initialValue: width, min: 160, max: 1200, divisions: 104, precision: 0);
        final h = context.knobs.double.slider(label: 'Height', initialValue: height, min: 120, max: 1200, divisions: 108, precision: 0);
        return ColoredBox(color: N.g00, child: Center(child: SizedBox(width: w, height: h, child: sheet())));
      },
    );

final toyComponent = WidgetbookComponent(name: 'Toys', useCases: [
  _sheet('Shelf all', () => const ToyShelf(), 820, 820),
  for (final g in toyGroups.keys) _sheet('Shelf $g', () => ToyShelf(only: g), 760, 360),
  for (final e in toyRowSets.entries) _sheet('Inspector rows ${e.key}', () => ToyRows(e.value), 300, 640),
]);
