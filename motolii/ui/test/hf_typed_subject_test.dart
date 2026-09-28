import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/insp/rows.dart';
import '../lib/hf/insp/toys.dart';

class _Store extends ParamStore {
  _Store() : super([{'id': 'x', 'label': 'X', 'kind': 'f64', 'value': 10.0}]);
  Object? who = 1;
  @override
  Object? get subject => who;
}

void main() {
  Future<_Store> mount(WidgetTester tester) async {
    final s = _Store();
    await tester.pumpWidget(Directionality(
      textDirection: TextDirection.ltr,
      child: Center(child: SizedBox(width: 200, height: 40, child: ValueToy(Slot(s, 'x')))),
    ));
    await tester.tap(find.byType(ValueToy));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText), '42');
    return s;
  }

  testWidgets('a number typed for one layer is written when the typing ends on it', (tester) async {
    final s = await mount(tester);
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(s.row('x')['value'], 42);
  });

  testWidgets('a number typed for one layer is dropped when the shown layer changed meanwhile', (tester) async {
    final s = await mount(tester);
    s.who = 2; // another layer is now the one shown
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pumpAndSettle();
    expect(s.row('x')['value'], 10.0);
  });
}
