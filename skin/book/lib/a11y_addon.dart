// Test tool of the lab: the global 'A11y' addon (settings panel, "A11y"). Off = today; Show = Flutter's SemanticsDebugger draws the semantic labels over the UI,
// i.e. what a screen reader would read. An interactive thing with no label here is invisible to a blind person.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

enum A11yMode {
  off('Off (today)'),
  labels('Show semantics labels');

  const A11yMode(this.label);
  final String label;
}

class A11yAddon extends WidgetbookAddon<A11yMode> {
  A11yAddon() : super(name: 'A11y');

  @override
  List<Field> get fields => [
        ObjectDropdownField<A11yMode>(name: 'mode', values: A11yMode.values, initialValue: A11yMode.off, labelBuilder: (m) => m.label),
      ];

  @override
  A11yMode valueFromQueryGroup(Map<String, String> group) => valueOf<A11yMode>('mode', group)!;

  @override
  Widget buildUseCase(BuildContext context, Widget child, A11yMode setting) =>
      setting == A11yMode.labels ? SemanticsDebugger(child: child) : child;
}
