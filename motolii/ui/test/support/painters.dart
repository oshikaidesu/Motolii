import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

/// A panel's painter is private to its own library, so a test cannot name its
/// type. Matching the name as a string instead makes the test break whenever a
/// file is split or a class renamed — the drawing never moved. Grab the picture
/// by the values its painter carries: the same values the test then reads.
Finder painterCarrying(bool Function(dynamic painter) values) =>
    find.byWidgetPredicate(
      (w) => w is CustomPaint && _carries(w.painter, values),
      description: 'CustomPaint whose painter carries the asked values',
    );

/// The painters behind [painterCarrying], in paint order.
Iterable<dynamic> paintersCarrying(
  WidgetTester tester,
  bool Function(dynamic painter) values,
) => tester
    .widgetList<CustomPaint>(painterCarrying(values))
    .map((w) => w.painter);

bool _carries(CustomPainter? painter, bool Function(dynamic) values) {
  if (painter == null) return false;
  try {
    return values(painter);
  } on NoSuchMethodError {
    return false;
  }
}
