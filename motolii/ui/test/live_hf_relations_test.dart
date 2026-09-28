import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/bp/effects.dart' show EffectScene;
import '../lib/live_hf/adapters/relations.dart';
import '../lib/live_hf/adapters/web.dart';
import '../lib/live_hf/workspace.dart';
import '../lib/session/editor_session.dart';

void main() {
  // The Inspector's "Relation…" and "Show relation" ask for the Relations panel by name (placePanel); the live
  // shell activates whatever the workspace defines under that name. Web is offered by the Dock's Open menu.
  testWidgets('the live workspace has the Relations and Web panels the Inspector and Dock ask for', (tester) async {
    final scene = (await tester.runAsync(EffectScene.build))!;
    final c = EditorSession()
      ..document.value = {
        'layers': [
          {'id': 1, 'name': 'Dot', 'kind': 'Shape', 'properties': []},
        ],
        'capabilities': ['relate'],
      };
    final ws = LiveWorkspace(c: c, scene: scene);
    for (final (name, type) in [('Relations', RelationsPanel), ('Web', NewWeb)]) {
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
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });
}
