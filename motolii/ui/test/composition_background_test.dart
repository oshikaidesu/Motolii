import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/composition_controls.dart';
import '../lib/foundation/theme.dart';

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
  testWidgets('background presets and hex both write the composition colour', (
    tester,
  ) async {
    final c = RecordingSession();
    c.document.value = {
      'width': 1920,
      'height': 1080,
      'durationFrames': 300,
      'fps': 30.0,
      'background': [0.0, 0.0, 0.0, 1.0],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
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
    await tester.enterText(find.byType(TextFormField).last, '#ff8000');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();
    final rgba = c.commands.last.$2['background'] as List;
    expect(rgba[0], 1.0);
    expect((rgba[1] as double) * 255, closeTo(128, .5));
    expect(rgba[2], 0.0);
    await tester.enterText(find.byType(TextFormField).last, 'zzz');
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();
    expect(c.error.value, isNotNull);
    await tester.pumpWidget(const SizedBox());
  });
}
