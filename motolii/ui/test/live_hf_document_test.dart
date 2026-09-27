import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/document.dart';
import '../lib/session/editor_session.dart';

void main() {
  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async => <String, dynamic>{});
  });

  Future<Future<bool>> ask(WidgetTester tester, EditorSession c) async {
    late BuildContext ctx;
    await tester.pumpWidget(WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(s, b) => PageRouteBuilder<T>(settings: s, pageBuilder: (context, _, __) => b(context)),
      home: Builder(builder: (context) {
        ctx = context;
        return const SizedBox.expand();
      }),
    ));
    final answer = mayReplace(ctx, c);
    await tester.pump();
    return answer;
  }

  testWidgets('a saved document closes without a question', (tester) async {
    final c = EditorSession()..document.value = {'dirty': false};
    final answer = await ask(tester, c);
    expect(find.text('Save changes?'), findsNothing);
    expect(await answer, isTrue);
  });

  testWidgets('an edited document asks; Cancel keeps it, Don\'t Save lets it go', (tester) async {
    final c = EditorSession()..document.value = {'dirty': true};
    var answer = await ask(tester, c);
    expect(find.text('Save changes?'), findsOneWidget);
    await tester.tap(find.text('Cancel'));
    await tester.pump();
    expect(await answer, isFalse);

    answer = await ask(tester, c);
    await tester.tap(find.text("Don't Save"));
    await tester.pump();
    expect(await answer, isTrue);
  });
}
