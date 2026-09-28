import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/bp/effects.dart' show EffectScene;
import '../lib/live_hf/adapters/console.dart';
import '../lib/live_hf/adapters/relations.dart';
import '../lib/session/console_log.dart';
import '../lib/live_hf/adapters/web.dart';
import '../lib/live_hf/workspace.dart';
import '../lib/session/editor_session.dart';

void main() {
  // The Inspector's "Relation…" and "Show relation" ask for the Relations panel by name (placePanel); the live
  // shell activates whatever the workspace defines under that name. Web is offered by the Dock's Open menu.
  testWidgets('the live workspace has the Relations, Web and Console panels the Inspector, Dock and Timeline seat ask for', (tester) async {
    final scene = (await tester.runAsync(EffectScene.build))!;
    final c = EditorSession()
      ..document.value = {
        'layers': [
          {'id': 1, 'name': 'Dot', 'kind': 'Shape', 'properties': []},
        ],
        'capabilities': ['relate'],
      };
    final log = ConsoleLog(c, c.slice('notice', const [], derived: () => effectsNotice(c.state)), effectsNotice);
    final ws = LiveWorkspace(c: c, scene: scene, console: log);
    for (final (name, type) in [('Relations', RelationsPanel), ('Web', NewWeb), ('Console', LiveConsole)]) {
      final def = ws.dock.defs[name];
      expect(def, isNotNull, reason: name);
      await tester.pumpWidget(Directionality(
        textDirection: TextDirection.ltr,
        child: Overlay(
          key: ValueKey(name),
          initialEntries: [OverlayEntry(builder: (_) => SizedBox(width: 400, height: 500, child: def!.build()))],
        ),
      ));
      await tester.pump();
      expect(find.byType(type), findsOneWidget, reason: name);
    }
    // the Console is the session's messages: an operation error lands there; Clear empties it
    expect(find.text('No messages'), findsOneWidget);
    c.error.value = 'Asset is still in use';
    await tester.pump();
    expect(find.text('Asset is still in use'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('console-clear')));
    await tester.pump();
    expect(find.text('No messages'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    log.dispose();
    c.dispose();
  });
}
