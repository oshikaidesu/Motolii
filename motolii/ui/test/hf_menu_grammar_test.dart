import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/shell/menu.dart';
import '../lib/hf/shell/place.dart';

/// A menu lights the line under the pointer (not only the one the keys walked to), marks the current value with a
/// check instead of the same light, tells titles and facts as information, and is never narrower than a line.
void main() {
  testWidgets('pointer lights a line; info lines are information; current has a check; a point gets a width', (t) async {
    late BuildContext ctx;
    await t.pumpWidget(WidgetsApp(
      color: const Color(0xFF000000),
      pageRouteBuilder: <T>(s, b) => PageRouteBuilder<T>(settings: s, pageBuilder: (context, _, __) => b(context)),
      home: Builder(builder: (c) {
        ctx = c;
        return const SizedBox.expand();
      }),
    ));
    final picked = showHfMenu<String>(ctx, const Rect.fromLTWH(100, 100, 0, 0), const [
      ('title', 'Dusk_gradient'),
      ('fact:0', '1600 × 900'),
      ('apply', 'Place'),
      ('remove', 'Remove'),
    ], info: const {'title', 'fact:0'}, selected: 'apply');
    await t.pumpAndSettle();
    // never a zero-width menu
    final place = t.getRect(find.text('Place'));
    expect(place.width, greaterThan(20));
    expect(find.text('✓'), findsOneWidget, reason: 'the current value carries a check');
    // hover lights the line under the pointer
    final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
    await mouse.addPointer(location: t.getCenter(find.text('Remove')));
    await t.pump();
    Container lineOf(String label) => t.widget<Container>(find.ancestor(of: find.text(label), matching: find.byType(Container)).first);
    expect(lineOf('Remove').color, H.selHi);
    expect(lineOf('Place').color, isNull, reason: 'current is a check, not the light');
    // the title is information: not greyed like an action that cannot run
    expect(t.widget<Text>(find.text('Dusk_gradient')).style!.color, isNot(const Color(0xFF6A6A6C)));
    // the keys walk only lines that do something
    await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await t.pumpAndSettle();
    expect(await picked, isIn(['apply', 'remove']));
    await mouse.removePointer();
  });
}
