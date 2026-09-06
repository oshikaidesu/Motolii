import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/stage.dart';
import '../lib/foundation/theme.dart';

class StageSession extends EditorSession {
  int refreshes = 0;
  @override
  Future<void> refreshPreview() async {
    refreshes++;
  }
}

void main() {
  testWidgets(
    'Stage requests its frame after mount and updates fit without editing',
    (tester) async {
      final c = StageSession();
      c.document.value = {
        'width': 640,
        'height': 480,
        'frame': 27,
        'documentRevision': 'r1',
        'layers': [],
        'selectedIds': [],
      };
      c.frame.value = 27;
      final document = c.document.value;
      Widget app(double width) => MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: width,
              height: 400,
              child: StagePanel(controller: c),
            ),
          ),
        ),
      );
      await tester.pumpWidget(app(600));
      await tester.pump();
      expect(c.refreshes, 1);
      expect(find.text('0%'), findsNothing);
      await tester.pumpWidget(app(500));
      await tester.pump();
      expect(c.refreshes, 1);
      expect(c.frame.value, 27);
      expect(identical(c.document.value, document), isTrue);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
