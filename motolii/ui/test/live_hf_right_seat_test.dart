import 'dart:convert';

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/right_seat.dart';
import '../lib/session/editor_session.dart';

void main() {
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      return <String, dynamic>{};
    });
  });

  Future<EditorSession> seat(WidgetTester t, Map<String, dynamic> layer) async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': ['setProperty', 'toggleKey', 'enableEffect', 'removeEffect', 'moveEffect', 'previewProperties', 'commitPreview'],
        'layers': [
          layer,
          {'id': 3, 'name': 'Child', 'kind': 'Shape', 'parent': 1},
        ],
        'selectedId': layer['id'],
        'selectedIds': [layer['id']],
      };
    await t.pumpWidget(Directionality(
      textDirection: TextDirection.ltr,
      child: Center(child: SizedBox(width: 320, height: 700, child: RightSeat(c: c))),
    ));
    await t.pump();
    return c;
  }

  testWidgets('a plain layer shows Transform alone', (t) async {
    await seat(t, {'id': 1, 'name': 'Box', 'kind': 'Shape'});
    expect(find.byKey(const ValueKey('tf-title')), findsOneWidget);
    expect(find.byKey(const ValueKey('seat-scroll')), findsNothing);
  });

  testWidgets('a group shows its Layout card under Transform', (t) async {
    await seat(t, {'id': 1, 'name': 'Row', 'kind': 'Group'});
    expect(find.byKey(const ValueKey('tf-title')), findsOneWidget);
    expect(find.byKey(const ValueKey('layout-title')), findsOneWidget);
  });

  testWidgets('a layer\'s effects are its cards; the power toggles through enableEffect', (t) async {
    await seat(t, {
      'id': 1,
      'name': 'Box',
      'kind': 'Shape',
      'effects': [
        {'id': 'e1', 'name': 'Blur', 'enabled': true, 'params': []},
      ],
    });
    expect(find.byKey(const ValueKey('effect-card:e1')), findsOneWidget);
    await t.ensureVisible(find.byKey(const ValueKey('effect-toggle:e1')));
    await t.tap(find.byKey(const ValueKey('effect-toggle:e1')));
    await t.pump();
    expect(sent.where((m) => m['op'] == 'enableEffect').single, {'op': 'enableEffect', 'layer': 1, 'id': 'e1', 'enabled': false});
  });
}
