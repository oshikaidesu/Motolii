import 'dart:convert';

import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/live_hf/adapters/right_seat.dart';
import '../lib/session/editor_session.dart';

void main() {
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String)
            sent.add(jsonDecode(args['command'] as String) as Map);
          return <String, dynamic>{};
        });
  });

  Future<EditorSession> seat(WidgetTester t, Map<String, dynamic> layer) async {
    final c = EditorSession()
      ..document.value = {
        'capabilities': [
          'setProperty',
          'toggleKey',
          'enableEffect',
          'removeEffect',
          'moveEffect',
          'previewProperties',
          'commitPreview',
        ],
        'layers': [
          layer,
          {'id': 3, 'name': 'Child', 'kind': 'Shape', 'parent': 1},
        ],
        'selectedId': layer['id'],
        'selectedIds': [layer['id']],
      };
    await t.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: Localizations(
          locale: const Locale('en'),
          delegates: const [DefaultWidgetsLocalizations.delegate],
          child: Overlay(
          initialEntries: [
            OverlayEntry(
              builder: (_) => Center(
                child: SizedBox(width: 320, height: 700, child: RightSeat(c: c)),
              ),
            ),
          ],
        ),
        ),
      ),
    );
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

  testWidgets(
    'a layer\'s effects are its cards; the power toggles through enableEffect',
    (t) async {
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
      expect(sent.where((m) => m['op'] == 'enableEffect').single, {
        'op': 'enableEffect',
        'layer': 1,
        'id': 'e1',
        'enabled': false,
      });
    },
  );
  testWidgets('dragging an effect card by its header below the next applies it later (moveEffect)', (t) async {
    await seat(t, {
      'id': 1,
      'name': 'Box',
      'kind': 'Shape',
      'effects': [
        {'id': 'e1', 'name': 'Blur', 'enabled': true, 'params': []},
        {'id': 'e2', 'name': 'Glow', 'enabled': true, 'params': []},
      ],
    });
    final first = find.byKey(const ValueKey('effect-card:e1'));
    final second = find.byKey(const ValueKey('effect-card:e2'));
    await t.ensureVisible(first);
    final from = t.getRect(first).topCenter + const Offset(0, 12);
    final g = await t.startGesture(from);
    await t.pump();
    final to = t.getRect(second).bottomCenter + const Offset(0, 10);
    for (var k = 1; k <= 10; k++) {
      await g.moveTo(Offset.lerp(from, to, k / 10)!);
      await t.pump(const Duration(milliseconds: 30));
    }
    await t.pump(const Duration(milliseconds: 300));
    await g.up();
    await t.pumpAndSettle();
    expect(sent.where((m) => m['op'] == 'moveEffect').single, {'op': 'moveEffect', 'layer': 1, 'id': 'e1', 'to': 1});
  });
  testWidgets('a frozen layer shows its effects baked: no toggle, no menu, the reason said', (t) async {
    await seat(t, {
      'id': 1,
      'name': 'Box',
      'kind': 'Shape',
      'frozen': true,
      'effects': [
        {'id': 'e1', 'name': 'Blur', 'enabled': true, 'params': []},
      ],
    });
    expect(find.byKey(const ValueKey('effect-frozen:e1')), findsOneWidget);
    await t.ensureVisible(find.byKey(const ValueKey('effect-toggle:e1')));
    await t.tap(find.byKey(const ValueKey('effect-toggle:e1')));
    await t.tap(find.byKey(const ValueKey('effect-menu:e1')));
    await t.pumpAndSettle();
    expect(sent.where((m) => m['op'] == 'enableEffect'), isEmpty);
    expect(find.text('Remove effect'), findsNothing);
  });
  testWidgets('nothing picked says so; a camera header names the layer and offers Animate', (t) async {
    final c = await seat(t, {'id': 1, 'name': 'Box', 'kind': 'Shape'});
    c.document.value = {...c.state, 'selectedId': null, 'selectedIds': <int>[]};
    await t.pump();
    expect(find.text('Select a layer'), findsOneWidget);
    c.document.value = {
      ...c.state,
      'capabilities': [...(c.state['capabilities'] as List), 'animate'],
      'layers': [
        {'id': 5, 'name': 'Dolly', 'kind': 'Camera', 'properties': []},
      ],
      'selectedId': 5,
      'selectedIds': [5],
    };
    await t.pump();
    expect(find.text('Dolly'), findsOneWidget);
    await t.tap(find.byKey(const ValueKey('camera-animate')));
    await t.pump();
    expect(sent.where((m) => m['op'] == 'animate'), isNotEmpty);
  });
}
