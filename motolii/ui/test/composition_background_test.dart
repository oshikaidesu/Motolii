import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/composition_controls.dart';
import '../lib/foundation/color_field.dart';
import 'support/editor_test_theme.dart';

import '../lib/foundation/leaves.dart';

class RecordingSession extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    commands.add((op, args));
  }
}

/// AE の Composition Settings → Background Color: 地の色は Composition の値で、preset と hex の 2 口。
void main() {
  testWidgets(
    'background presets write the colour; the swatch focuses the wheel',
    (tester) async {
      final c = RecordingSession();
      c.document.value = {
        'width': 1920,
        'height': 1080,
        'durationFrames': 300,
        'fps': 30.0,
        'background': [0.0, 0.0, 0.0, 1.0],
        'capabilities': ['focusColor'],
      };
      await tester.pumpWidget(
        MaterialApp(
          theme: editorTestTheme,
          home: Scaffold(
            body: SizedBox(
              width: 600,
              child: SingleChildScrollView(
                child: CompositionControls(controller: c),
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      await tester.tap(find.text('Grey'));
      await tester.pump();
      expect(c.commands.last.$1, 'composition');
      expect(c.commands.last.$2, {
        'background': [0.5, 0.5, 0.5, 1.0],
      });
      // The background is a colour atom: no hex on the sheet. Its swatch hands
      // the focus to the Browser's wheel as the composition's background.
      expect(
        tester
            .widgetList<EditorTextField>(find.byType(EditorTextField))
            .every((f) => !(f.controller?.text ?? '').startsWith('#')),
        isTrue,
        reason: 'no hex on the sheet',
      );
      await tester.tap(find.byType(EditorColorField));
      await tester.pump();
      expect(c.commands.last.$1, 'focusColor');
      expect(c.commands.last.$2, {'slot': 'Background'});
      await tester.pumpWidget(const SizedBox());
    },
  );
}
