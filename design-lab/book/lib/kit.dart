// What every set uses to put a widget on the book's ground. Not a design decision; a convenience.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'parts/controls.dart';
import 'tokens.dart';

/// A widget centred on the lab's ground (g07), at an optional fixed size.
Widget onGround(Widget child, {double? width, double? height, Color color = N.g07, EdgeInsets padding = const EdgeInsets.all(24)}) =>
    ColoredBox(color: color, child: Padding(padding: padding, child: Center(child: SizedBox(width: width, height: height, child: child))));

/// The "Look" knob: an open design question is shown in three finishes and the owner picks in the real window.
Look lookKnob(BuildContext c) => c.knobs.object.dropdown<Look>(label: 'Look', options: Look.values, initialOption: Look.quiet, labelBuilder: (l) => l.name);

/// One use case with a ready-made ground.
WidgetbookUseCase uc(String name, Widget Function(BuildContext) build, {double? width, double? height}) =>
    WidgetbookUseCase(name: name, builder: (c) => onGround(build(c), width: width, height: height));

/// The global Dose knob (0 = today's tone, 1-3 = more seasoning).
Dose doseKnob(BuildContext c) => Dose.at(c.knobs.int.slider(label: 'Dose', initialValue: 1, min: 0, max: 3, divisions: 3));
