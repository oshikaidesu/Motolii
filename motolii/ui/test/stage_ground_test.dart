import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/stage.dart';
import '../lib/session/editor_session.dart';

/// 地は comp の値、市松はその見せ方。枠の事実の隣の 1 つの札で往復し、
/// 透明のあいだだけ Stage が市松を敷く(Photoshop・AE と同じ描き方)。
class GroundSession extends EditorSession {
  final commands = <(String, Map<String, dynamic>)>[];
  @override
  Future<void> refreshPreview() async {}
  @override
  Future<void> command(
    String op, [
    Map<String, dynamic> args = const {},
  ]) async {
    commands.add((op, args));
  }

  @override
  Future<dynamic> native(
    String method, [
    Map<String, dynamic> args = const {},
  ]) async => <String, dynamic>{};
}

void main() {
  Future<GroundSession> mount(
    WidgetTester tester,
    List<double> background,
  ) async {
    final c = GroundSession();
    c.document.value = {
      'width': 400,
      'height': 400,
      'frame': 0,
      'documentRevision': 'r1',
      'stageView': 'Camera',
      'background': background,
      'capabilities': ['composition'],
      'layers': [],
      'selectedIds': [],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: SizedBox(
            width: 464,
            height: 480,
            child: StagePanel(controller: c),
          ),
        ),
      ),
    );
    await tester.pump();
    return c;
  }

  bool checkerShown(WidgetTester tester) => tester
      .widgetList<CustomPaint>(find.byType(CustomPaint))
      .any((paint) => paint.painter is CheckerPainter);

  testWidgets('the ground switch drops the alpha and the checker follows it', (
    tester,
  ) async {
    final solid = await mount(tester, const [0.0, 0.0, 0.0, 1.0]);
    expect(
      checkerShown(tester),
      isFalse,
      reason: 'a solid ground draws no grid',
    );
    await tester.tap(find.byKey(const ValueKey('stage:transparentGround')));
    await tester.pump();
    expect(solid.commands.last.$1, 'composition');
    expect(solid.commands.last.$2, {
      'background': [0.0, 0.0, 0.0, 0.0],
    });
    await tester.pumpWidget(const SizedBox());

    // 透明のあいだは市松。押し戻すと同じ色で alpha だけ戻る。
    final clear = await mount(tester, const [0.2, 0.2, 0.2, 0.0]);
    expect(checkerShown(tester), isTrue);
    await tester.tap(find.byKey(const ValueKey('stage:transparentGround')));
    await tester.pump();
    expect(clear.commands.last.$2, {
      'background': [0.2, 0.2, 0.2, 1.0],
    });
    await tester.pumpWidget(const SizedBox());
  });
}
