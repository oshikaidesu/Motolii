import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls/scale.dart';
import '../lib/live_hf/keys.dart';
import '../lib/live_hf/ui_scale.dart';
import '../lib/session/editor_session.dart';

/// The UI's size is one number: whole percent steps, a safe range, kept in the settings, applied at the root (so a menu
/// or sheet in the overlay scales with the rest), and never applied to the work.
void main() {
  final written = <Map>[];
  var settings = <String, dynamic>{};
  setUp(() {
    written.clear();
    settings = {};
    LiveUiScale.instance.reset();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      if (call.method == 'readSettings') return Map<String, dynamic>.from(settings);
      if (call.method == 'writeSettings') {
        settings = Map<String, dynamic>.from(call.arguments as Map);
        written.add(settings);
      }
      return <String, dynamic>{};
    });
  });

  test('whole percent steps inside 70-130 %, and a saved value comes back', () {
    final ui = LiveUiScale.instance;
    expect(ui.smaller(), isTrue);
    expect(ui.percent.value, .99);
    expect(ui.factor.value, closeTo(LiveUiScale.base * .99, 1e-9));
    ui.set(.2);
    expect(ui.percent.value, LiveUiScale.min);
    ui.set(9);
    expect(ui.percent.value, LiveUiScale.max);
    ui.set(.8345);
    expect(ui.percent.value, .83, reason: 'kept to a whole percent');
    ui.restore('nonsense');
    expect(ui.percent.value, 1.0);
    ui.restore(.9);
    expect(ui.percent.value, .9);
  });

  testWidgets('Cmd+Option+minus / plus / 0 change the size a percent at a time and keep it; Cmd+minus stays the Stage zoom', (tester) async {
    final c = EditorSession()..document.value = {'layers': [], 'durationFrames': 90, 'capabilities': ['stageView']};
    final node = FocusNode();
    late BuildContext ctx;
    final keys = LiveKeys(c, () => ctx);
    await tester.pumpWidget(Builder(builder: (context) {
      ctx = context;
      return Focus(focusNode: node, onKeyEvent: keys.handle, child: const SizedBox());
    }));
    node.requestFocus();
    await tester.pump();
    Future<void> chord(LogicalKeyboardKey k, {bool alt = true}) async {
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      if (alt) await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
      await tester.sendKeyEvent(k);
      if (alt) await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await tester.pumpAndSettle();
    }

    await chord(LogicalKeyboardKey.minus);
    await chord(LogicalKeyboardKey.minus);
    expect(LiveUiScale.instance.percent.value, .98);
    expect(settings[LiveUiScale.settingsKey], .98, reason: 'the size is kept in the settings');
    await chord(LogicalKeyboardKey.equal);
    expect(LiveUiScale.instance.percent.value, .99);
    await chord(LogicalKeyboardKey.digit0);
    expect(LiveUiScale.instance.percent.value, 1.0);
    // without Option, minus is the Stage's zoom and leaves the UI's size alone
    await chord(LogicalKeyboardKey.minus, alt: false);
    expect(LiveUiScale.instance.percent.value, 1.0);
    c.dispose();
  });

  testWidgets('the root viewport lays the UI out at 1/scale and paints it at scale; EditorScale tells the Stage the factor', (tester) async {
    tester.view.physicalSize = const Size(1000, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    LiveUiScale.instance.set(.8);
    Size? laidOut;
    double? told;
    await tester.pumpWidget(Directionality(
      textDirection: TextDirection.ltr,
      child: EditorScale(
        notifier: LiveUiScale.instance.factor,
        child: EditorScaledViewport(
          scale: LiveUiScale.instance.factor.value,
          child: LayoutBuilder(builder: (context, box) {
            laidOut = box.biggest;
            told = EditorScale.of(context)?.value;
            return const SizedBox.expand();
          }),
        ),
      ),
    ));
    expect(laidOut!.width, closeTo(1000 / (LiveUiScale.base * .8), .01), reason: 'more room in the same window at a smaller size');
    expect(told, closeTo(LiveUiScale.base * .8, 1e-9));
  });
}
