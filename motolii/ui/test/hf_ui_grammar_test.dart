import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/metrics.dart';
import '../lib/hf/shell/dialog.dart';
import '../lib/hf/shell/place.dart';
import '../lib/hf/shell/sheet.dart';

/// Ordinary desktop grammar: a selection never looks like the action, an action's weight says primary / secondary /
/// destructive, a fact has no box, a short task opens under its control and answers Enter and Esc.
void main() {
  Widget app(Widget Function(BuildContext) home) => WidgetsApp(
        color: const Color(0xFF000000),
        pageRouteBuilder: <T>(s, b) => PageRouteBuilder<T>(settings: s, pageBuilder: (context, _, __) => b(context)),
        home: Builder(builder: home),
      );

  Color groundOf(WidgetTester t, String label) {
    final box = t.widget<Container>(find.ancestor(of: find.text(label), matching: find.byType(Container)).first);
    return (box.decoration as BoxDecoration).color!;
  }

  testWidgets('a chosen segment is a selection, never the primary action\'s colour', (t) async {
    var v = 0;
    await t.pumpWidget(app((_) => StatefulBuilder(
          builder: (context, set) => Center(
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              HfChoice<int>(options: const [(0, 'Whole', true), (1, 'Markers', true)], value: v, onChanged: (n) => set(() => v = n)),
              HfAction('Export…', kind: HfActionKind.primary, onTap: () {}),
            ]),
          ),
        )));
    expect(groundOf(t, 'Whole'), H.selHi);
    expect(groundOf(t, 'Export…'), H.mode);
    expect(groundOf(t, 'Whole'), isNot(H.mode));
    await t.tap(find.text('Markers'));
    await t.pump();
    expect(v, 1);
    expect(t.getSize(find.byType(HfChoice<int>)).height, UiMetrics.control);
  });

  testWidgets('the unsaved-work question sets Don\'t Save apart as destructive; Save is the primary', (t) async {
    late BuildContext ctx;
    await t.pumpWidget(app((c) {
      ctx = c;
      return const SizedBox.expand();
    }));
    final answer = showHfDialog<String>(ctx, title: 'Save changes?', body: 'Save the current document before closing it.', answers: const [('cancel', 'Cancel'), ('discard', "Don't Save"), ('save', 'Save')], destructive: const {'discard'});
    await t.pump();
    await t.pump();
    final discard = t.widget<Text>(find.text("Don't Save"));
    expect(discard.style!.color, H.record);
    expect(groundOf(t, 'Save'), H.mode);
    expect(t.getTopLeft(find.text("Don't Save")).dx, lessThan(t.getTopLeft(find.text('Cancel')).dx), reason: 'apart, on the left');
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await t.pump();
    expect(await answer, 'save', reason: 'Enter is the primary');
  });

  testWidgets('a short task opens under its control, dims nothing, runs its primary on Enter, closes on Esc', (t) async {
    late BuildContext ctx;
    var ran = 0;
    await t.pumpWidget(app((c) {
      ctx = c;
      return const SizedBox.expand();
    }));
    const anchor = Rect.fromLTWH(600, 10, 88, 30);
    var closed = false;
    showHfPopover(ctx, anchor: anchor, title: 'Export', primary: () => () => ran++, body: (_, close) => const HfFormRow('Output', HfFact('1920 × 1080'))).then((_) => closed = true);
    await t.pumpAndSettle();
    final pop = t.getRect(find.byKey(const ValueKey('hf-popover')));
    expect(pop.top, greaterThanOrEqualTo(anchor.bottom), reason: 'under the control');
    expect(pop.right, closeTo(anchor.right, 1), reason: 'its right edge on the control\'s');
    expect(find.byWidgetPredicate((w) => w is ColoredBox && w.color == const Color(0x88000000)), findsNothing, reason: 'no dimmed backdrop');
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    expect(ran, 1);
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pumpAndSettle();
    expect(closed, isTrue);
  });
}
