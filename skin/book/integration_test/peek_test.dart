// The inspector follows a hand on the stage while it is still down: the window peeks at the inside after each picture
// of the stage it is given (Chromium's BeginFrame), so this needs the real window and the stage's real picture, not a
// unit test's (test/workspace_peek_test.dart has the rest).
//   flutter test integration_test/peek_test.dart -d macos
import 'dart:io';

import 'package:flutter/rendering.dart' show RenderBox;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart' show Ctx, NumField;
import 'package:lab_book/workspace/timeline.dart';
import 'package:lab_book/workspace/workspace.dart' show Workspace;
import 'package:lab_book/workspace/ws.dart';

/// What the inspector shows of a value now (the number its field is given).
double? _shown(WidgetTester t, String id) => Ctx.read(t.element(find.byWidgetPredicate((w) => w is NumField && w.id == id))).doc.get(id);

/// Where the Transform card's drawing has the thing's middle now.
Offset _gizmo(WidgetTester t) => (t.widgetList<CustomPaint>(find.byType(CustomPaint)).firstWhere((p) => p.painter.runtimeType.toString() == '_CanvasPaint').painter as dynamic).g.pivot as Offset;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive; // (the stage's picture is the inside's, drawn as the window goes)

  testWidgets('a thing carried on the stage: the inspector\'s numbers follow the hand while it is still down', (t) async {
    t.binding.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.menu, (_) async => null);
    final doc = '${Directory.systemTemp.createTempSync('peek').path}/peek.motolii';
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => Workspace(document: doc)));
    await t.pumpAndSettle();
    final ws = WsScope.read(t.element(find.byType(WsTimeline))), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    ws..reread()..select('$cube');
    await t.pumpAndSettle();
    final was = _shown(t, 'v_x'), drawn = _gizmo(t);
    final canvas = (t.renderObject(find.byWidgetPredicate((w) => w is CustomPaint && w.painter.runtimeType.toString() == '_CanvasPaint')) as RenderBox).size;   // (its own size, in the units it draws in: the window may be drawn smaller as a whole)
    final stage = t.getRect(find.byKey(const ValueKey('ws-stage-comp')));
    final g = await t.startGesture(stage.center);
    for (var i = 1; i <= 10; ++i) {
      await g.moveBy(Offset(stage.width * .02, 0));
      await t.pump(const Duration(milliseconds: 16));
    }
    await t.pump(const Duration(milliseconds: 100));   // (pictures of the stage come and the window peeks after each)
    final carried = _shown(t, 'v_x'), went = _gizmo(t);
    await g.up();
    await t.pumpAndSettle();
    expect(carried, isNot(was), reason: 'X moved while the hand was down ($was -> $carried)');
    expect(went.dx, greaterThan(drawn.dx + 2), reason: 'the inspector\'s drawing moved right with it ($drawn -> $went)');
    expect(drawn.dx, closeTo(canvas.width / 2, 1), reason: 'drawn from the middle when opened');
    await t.pumpWidget(const SizedBox());
  });
}
