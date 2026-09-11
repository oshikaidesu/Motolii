import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/native_visual_sample.dart';

/// An effect's tile is the shelf's one sample with the effect on it. Holding
/// the pointer shows the bare sample; a difference-only effect splits the
/// large tile, bare left and dressed right.
void main() {
  Future<EditorSession> mount(WidgetTester tester, {required int view}) async {
    tester.view.physicalSize = const Size(520, 640);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['applyEffect'],
      'selectedIds': [1],
      'layers': [
        {'id': 1, 'name': 'Clip', 'kind': 'Media'},
      ],
      'catalog': [
        {'id': 'motolii.blur', 'name': 'Blur', 'stage': 'Pass', 'split': false, 'generation': 3},
        {'id': 'motolii.gain', 'name': 'Gain', 'stage': 'Pass', 'split': true, 'generation': 3},
      ],
    };
    c.deskWork.value = {'browserView': view};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Effects'),
        ),
      ),
    );
    return c;
  }

  Finder sample(String id, String side) =>
      find.byKey(ValueKey('browser:effect:$id:$side'));
  Map<String, dynamic> request(WidgetTester tester, Finder f) =>
      tester.widget<NativeVisualSample>(f).request;

  testWidgets('every tile asks for the shelf sample dressed in its effect', (
    tester,
  ) async {
    await mount(tester, view: 0);
    expect(sample('motolii.blur', 'after'), findsOneWidget);
    expect(sample('motolii.blur', 'before'), findsNothing);
    expect(request(tester, sample('motolii.blur', 'after')), {
      'kind': 'effect',
      'id': 'motolii.blur',
      'before': false,
      'generation': 3,
    });
    // The small grid never splits, even for a difference-only effect.
    expect(sample('motolii.gain', 'before'), findsNothing);
  });

  testWidgets('holding the pointer shows the bare sample', (tester) async {
    await mount(tester, view: 0);
    final gesture = await tester.startGesture(
      tester.getCenter(sample('motolii.blur', 'after')),
    );
    await tester.pump();
    expect(sample('motolii.blur', 'before'), findsOneWidget);
    expect(sample('motolii.blur', 'after'), findsNothing);
    expect(request(tester, sample('motolii.blur', 'before'))['before'], true);
    await gesture.up();
    await tester.pump();
    expect(sample('motolii.blur', 'after'), findsOneWidget);
    expect(sample('motolii.blur', 'before'), findsNothing);
    // The tile's double-tap window must close before the tree goes away.
    await tester.pump(const Duration(seconds: 1));
  });

  testWidgets('a difference-only effect splits the large tile', (tester) async {
    await mount(tester, view: 2);
    expect(sample('motolii.gain', 'after'), findsOneWidget);
    expect(sample('motolii.gain', 'before'), findsOneWidget);
    expect(
      find.descendant(
        of: find.byKey(const ValueKey('browser:Effects:motolii.gain')),
        matching: find.byKey(const ValueKey('browser:effect:divider')),
      ),
      findsOneWidget,
    );
    final tile = tester.getRect(find.byKey(const ValueKey('browser:Effects:motolii.gain')));
    final divider = tester.getRect(find.byKey(const ValueKey('browser:effect:divider')));
    expect(divider.center.dx, closeTo(tile.center.dx, 1));
    expect(sample('motolii.blur', 'before'), findsNothing);
  });
}
