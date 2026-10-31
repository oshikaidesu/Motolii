// The document's tempo: Live's Tempo field and Tap, its grid on the ruler, keys and handles held to it. Alight Motion has none.
import 'package:flutter/gestures.dart' show PointerDeviceKind, kSecondaryButton;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_ease_graph.dart';
import 'package:lab_book/workspace/desk_ease_handles.dart';
import 'package:lab_book/workspace/timeline.dart' show wtTimeOf;
import 'package:lab_book/workspace/timeline_paint.dart';
import 'package:lab_book/workspace/ws.dart';
import 'door.dart';

void main() {
  test('beats: every so many frames from where a bar begins, four to a bar', () {
    final b = WsBeats.of(120, 1, 60)!;
    expect((b.first, b.every), (60, 30));
    expect(b.between(55, 125).toList(), [(60.0, true), (90.0, false), (120.0, false)]);
    expect(b.between(170, 185).toList(), [(180.0, true)]);
    expect(b.nearest(74), 60);
    expect(b.nearest(76), 90);
    expect(b.nearest(10), 0, reason: 'before where the bar was said to begin: the grid goes on');
    expect(WsBeats.of(0, 1, 60), isNull, reason: 'no tempo, no beats');
    expect(wsNumber('１２８．５'), 128.5, reason: 'a Japanese input method\'s digits are digits');
    expect(wsNumber(' 90 '), 90);
    expect(wsNumber('－１'), -1);
    expect(wsNumber('速い'), isNull);
    expect(wtTimeOf('１：３０', 0, 60), wtTimeOf('1:30', 0, 60), reason: 'a time as well');
  });

  Future<(Ws, int)> opened(WidgetTester t) async {
    final ws = await openWorkspace(t, size: const Size(1600, 960)), m = ws.inside!;
    final cube = m.add('solid', 'cube');
    expect(m.key(cube, 'x', 0, 0) && m.key(cube, 'x', 2, 1), isTrue, reason: m.trouble);
    ws.reread();
    await t.pumpAndSettle();
    return (ws, cube);
  }

  Future<void> tempo(WidgetTester t, Ws ws, String said) async {
    await t.tap(find.byKey(const ValueKey('ws-tl-bpm')));
    await t.pump();
    await t.enterText(find.byKey(const ValueKey('ws-tl-bpm-words')), said);
    await t.testTextInput.receiveAction(TextInputAction.done);
    await settled(t, ws);
    await t.pumpAndSettle();
  }

  testWidgets('written beside the frame rate it is the document\'s, one step, and the ruler goes by it', (t) async {
    final (ws, _) = await opened(t);
    WtRulerPainter ruler() => find.byWidgetPredicate((w) => w is CustomPaint && w.painter is WtRulerPainter).evaluate().map((e) => (e.widget as CustomPaint).painter as WtRulerPainter).single;
    expect(ruler().tempo, isNull, reason: 'none until it is said');
    await tempo(t, ws, '120');
    expect(ws.bpm, 120, reason: ws.inside!.trouble);
    expect(ruler().tempo?.every, 30);
    expect(find.text('120 BPM'), findsOneWidget);
    expect(ws.inside!.undo(), isTrue);
    ws.reread();
    await t.pumpAndSettle();
    expect(ws.bpm, 0, reason: 'one step');
    expect(find.text('BPM'), findsOneWidget);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('left, the field says what is in it, as a marker\'s name does; Esc leaves it as it was', (t) async {
    final (ws, _) = await opened(t);
    Future<void> write(String said) async {
      await t.tap(find.byKey(const ValueKey('ws-tl-bpm')));
      await t.pump();
      await t.enterText(find.byKey(const ValueKey('ws-tl-bpm-words')), said);
      await t.pump();
    }

    await write('128');
    FocusManager.instance.primaryFocus?.unfocus();
    await t.pump();
    await settled(t, ws);
    await t.pumpAndSettle();
    expect(ws.bpm, 128, reason: ws.inside!.trouble);
    await write('90');
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pump();
    await settled(t, ws);
    await t.pumpAndSettle();
    expect(ws.bpm, 128, reason: 'as it was');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('Tap: one press a beat, the tempo follows', (t) async {
    final (ws, _) = await opened(t);
    await t.tap(find.byKey(const ValueKey('ws-tl-bpm')));
    await t.pump();
    for (var i = 0; i < 4; i++) {
      await t.tap(find.byKey(const ValueKey('ws-tl-bpm-tap')));
      await t.pump();
      await t.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
    }
    await settled(t, ws);
    expect(ws.bpm, inInclusiveRange(60, 200.5), reason: 'three beats of 300 ms at the least, and what a loaded machine adds to them');
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('a key carried near a beat stays on it; held with Cmd it does not; Set 1.1.1 Here moves the grid', (t) async {
    final (ws, cube) = await opened(t);
    await tempo(t, ws, '120');
    final m = ws.inside!;
    final tracks = find.byWidgetPredicate((w) => w is CustomPaint && w.painter is WtTracksPainter);
    WtTracksPainter painter() => t.widget<CustomPaint>(tracks).painter! as WtTracksPainter;
    Future<void> carry(int from, double to) async {
      final r = t.getRect(tracks), y = r.top + WtM.row / 2;
      final hand = await t.startGesture(Offset(r.left + painter().view.x(from), y), kind: PointerDeviceKind.mouse);
      await hand.moveBy(const Offset(4, 0));
      await t.pump();
      await hand.moveTo(Offset(r.left + to, y));
      await t.pump();
      await hand.up();
      await settled(t, ws);
      await hand.removePointer();
      await t.pumpAndSettle();
    }

    final view = painter().view, near = view.x(150) + 3;   // (three pixels from the beat at frame 150, 2.5 s)
    await carry(120, near);
    expect(m.keyMoments(cube, 'x'), [0, 2.5], reason: 'on the beat: ${m.trouble}');
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await carry(150, view.x(180) + 3);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    expect(m.keyMoments(cube, 'x').last, greaterThan(3), reason: 'with Cmd it goes where the hand does: Live\'s 6.10');
    await t.tapAt(t.getRect(find.byKey(const ValueKey('ws-tl-ruler'))).center, buttons: kSecondaryButton);
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ws-menu-Set 1.1.1 Here')));
    await settled(t, ws);
    expect(ws.beats!.first, closeTo(ws.inside!.value(0, 'bpm_from', 0)! * Ws.fps, 1e-6));
    expect(ws.beats!.first, greaterThan(0), reason: 'a bar begins where the menu was asked');
    expect(t.takeException(), isNull);
    await closeWorkspace(t);
  }, skip: !insideBuilt);

  testWidgets('the Ease graph shows the beats of its stretch, and a handle stays on one: as many waves a beat as the hand says', (t) async {
    final (ws, cube) = await opened(t);
    await tempo(t, ws, '120');
    final m = ws.inside!;
    ws
      ..pickKey((layer: '$cube', frame: 0))
      ..front('Ease');
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('ease.Cyclic')));
    await settled(t, ws);
    final graph = find.byType(EaseGraph);
    EaseGraph g() => t.widget<EaseGraph>(graph);
    expect(g().beats, [.25, .5, .75], reason: 'a stretch of 2 s, a beat each half second');
    final r = t.getRect(graph), box = easeBox(r.size).shift(r.topLeft), e = g();
    final (lo, hi) = easeFit(e.ease, e.overshoot, e.ends, e.onion);
    final h = easeHandles(e.ease).first;
    final from = Offset(box.left + h.x * box.width, box.bottom - (h.y - lo) / (hi - lo) * box.height);
    final hand = await t.startGesture(from, kind: PointerDeviceKind.mouse);
    await hand.moveBy(const Offset(3, 0));
    await t.pump();
    await hand.moveTo(Offset(box.left + .25 * box.width + 3, from.dy));
    await t.pump();
    await hand.up();
    await settled(t, ws);
    await hand.removePointer();
    expect(double.parse(m.wayOf(cube, 'x', 0).split(' ')[1]), closeTo(.25, 1e-6), reason: 'one wave a beat: ${m.wayOf(cube, 'x', 0)}');
    await closeWorkspace(t);
  }, skip: !insideBuilt);
}
