import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/panels/desk.dart';
import '../lib/panels/panel_settings.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets('Desk presents auxiliary tools while Settings owns placement', (
    tester,
  ) async {
    final c = EditorSession();
    final requests = <Map<String, dynamic>>[];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'placePanel') {
            final args = Map<String, dynamic>.from(call.arguments as Map);
            requests.add(args);
            c.panePlaces.value = {
              ...c.panePlaces.value,
              args['name']: args['placement'],
            };
          }
          return {};
        });
    c.document.value = {'selectedIds': [], 'selectedKeys': [], 'layers': []};
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Row(
            children: [
              SizedBox(
                width: 280,
                child: DeskPanel(
                  controller: c,
                  panelBuilder: (name) => Text('Body $name'),
                ),
              ),
              Expanded(child: PanelSettings(controller: c)),
            ],
          ),
        ),
      ),
    );
    Finder inDesk(String text) =>
        find.descendant(of: find.byType(DeskPanel), matching: find.text(text));
    expect(inDesk('Depth'), findsOneWidget);
    expect(inDesk('Ease'), findsOneWidget);
    expect(
      find.descendant(
        of: find.byType(PanelSettings),
        matching: find.byType(DropdownButton<String>),
      ),
      findsNothing,
    );
    for (final name in ['Stage', 'Timeline', 'Inspector', 'Notes', 'Web']) {
      expect(inDesk(name), findsNothing);
    }
    expect(
      find.descendant(
        of: find.byType(PanelSettings),
        matching: find.text('Stage'),
      ),
      findsOneWidget,
    );
    final pin = find.byKey(const ValueKey('placement:Depth:tab'));
    await tester.ensureVisible(pin);
    await tester.tap(pin);
    await tester.pumpAndSettle();
    expect(requests.single, {'name': 'Depth', 'placement': 'tab'});
    expect(inDesk('Depth'), findsNothing);
    final drawer = find.byKey(const ValueKey('placement:Depth:drawer'));
    await tester.tap(drawer);
    await tester.pumpAndSettle();
    expect(inDesk('Depth'), findsOneWidget);
    expect(c.deskDrawer.value, isNull);
    expect(c.deskDefault.value, 'Tools');
    await tester.tap(find.byTooltip('Use Depth when idle'));
    await tester.pumpAndSettle();
    expect(c.deskDefault.value, 'Depth');
    expect(inDesk('Body Depth'), findsOneWidget);
    expect(requests.length, 2);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
