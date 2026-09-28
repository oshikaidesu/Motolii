import 'dart:convert';

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/hf/shell/place.dart' show H;
import '../lib/hf/shell/timeline.dart';
import '../lib/live_hf/adapters/timeline.dart';
import '../lib/session/editor_session.dart';

void main() {
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String)
            sent.add(jsonDecode(args['command'] as String) as Map);
          return <String, dynamic>{};
        });
  });

  testWidgets(
    'a body dragged is retimed from where the drag began; a name selects its layer',
    (tester) async {
      tester.view.physicalSize = const Size(1178, 291);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession()
        ..document.value = {
          'fps': 30,
          'durationFrames': 300,
          'capabilities': [
            'moveKeys',
            'setTiming',
            'setTimings',
            'previewTimings',
            'commitPreview',
            'moveLayers',
            'setAttrs',
            'toggleKey',
            'clip',
          ],
          'layers': [
            {
              'id': 5,
              'name': 'Title',
              'kind': 'Text',
              'start': 30,
              'duration': 60,
              'sourceIn': 0,
              'properties': [],
            },
          ],
        };
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 1178,
              height: 291,
              child: LiveTimeline(c: c),
            ),
          ),
        ),
      );

      // the body runs from 1 s to 3 s: grab it at 2 s and move it one second (one major tick) right
      final y = tlTop + tlPitch * 0 + tlRowH / 2 - 703;
      final from = Offset(tlX(2) - 344, y);
      final g = await tester.startGesture(from);
      await g.moveBy(const Offset(tlUnit / 2, 0));
      await tester.pump();
      // the host answers the preview with the moved start; the next preview must not add to it
      c.document.value = {
        ...c.state,
        'layers': [
          {...EditorSession.maps(c.state['layers']).first, 'start': 45},
        ],
      };
      await tester.pump();
      await g.moveBy(const Offset(tlUnit / 2, 0));
      await g.up();
      for (var i = 0; i < 5; i++) {
        await tester.pump();
      }
      final retimes = [
        for (final m in sent)
          if (m['op'] == 'previewTimings' || m['op'] == 'setTimings') m,
      ];
      expect(sent.last['op'], 'commitPreview');
      expect((retimes.last['changes'] as List).single, {
        'layer': 5,
        'start': 60,
        'duration': 60,
        'sourceIn': 0,
      });
      expect(
        sent.any((m) => m['op'] == 'select' && (m['ids'] as List).contains(5)),
        isTrue,
      );
    },
  );

  testWidgets(
    'HF Timeline opens shared property and effect lanes from folded key summaries',
    (tester) async {
      tester.view.physicalSize = const Size(1178, 291);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession()
        ..document.value = {
          'fps': 30,
          'durationFrames': 90,
          'layers': [
            {
              'id': 9,
              'name': 'Shape',
              'kind': 'Shape',
              'start': 0,
              'duration': 90,
              'properties': [
                {
                  'id': 'opacity',
                  'label': 'Opacity',
                  'keys': [
                    {'frame': 10},
                    {'frame': 20},
                  ],
                },
              ],
              'effects': [
                {
                  'id': 4,
                  'name': 'Blur',
                  'params': [
                    {
                      'id': 'effect.4.param.radius',
                      'label': 'Radius',
                      'keys': [
                        {'frame': 15},
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        };
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 1178,
              height: 291,
              child: LiveTimeline(c: c),
            ),
          ),
        ),
      );
      final dynamic state = tester.state(find.byType(LiveTimeline));
      expect(state.tracks.first.summaryFrames, [10, 15, 20]);
      expect(state.tracks, hasLength(1));
      await tester.tapAt(const Offset(134, 83));
      await tester.pump();
      expect(
        state.tracks.map((row) => row.property?['id']).whereType<String>(),
        ['opacity', 'effect.4.param.radius'],
      );
      // the lanes carry a key mark, not the layer's switches
      expect(find.text('M'), findsOneWidget);
    },
  );

  testWidgets(
    'the HF M control toggles layer state through the shared Timeline input',
    (tester) async {
      tester.view.physicalSize = const Size(1178, 291);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final c = EditorSession()
        ..document.value = {
          'fps': 30,
          'durationFrames': 90,
          'capabilities': ['setAttrs'],
          'layers': [
            {
              'id': 5,
              'name': 'Title',
              'kind': 'Text',
              'start': 0,
              'duration': 90,
              'properties': [],
            },
          ],
        };
      await tester.pumpWidget(
        Directionality(
          textDirection: TextDirection.ltr,
          child: Align(
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: 1178,
              height: 291,
              child: LiveTimeline(c: c),
            ),
          ),
        ),
      );
      await tester.tapAt(const Offset(151, 83));
      await tester.pump();
      expect(sent.single, {
        'op': 'setAttrs',
        'layers': [5],
        'patch': {'hidden': true},
      });
    },
  );

  Future<EditorSession> mountRows(WidgetTester tester, int count, {int frames = 300}) async {
    tester.view.physicalSize = const Size(1178, 291);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = EditorSession()
      ..document.value = {
        'fps': 30,
        'durationFrames': frames,
        'capabilities': ['seek', 'select'],
        'layers': [
          for (var n = 0; n < count; n++)
            {'id': n + 1, 'name': 'L$n', 'kind': 'Shape', 'start': 0, 'duration': 90, 'properties': []},
        ],
      };
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: Overlay(
          initialEntries: [
            OverlayEntry(
              builder: (_) =>
                  Align(alignment: Alignment.topLeft, child: SizedBox(width: 1178, height: 291, child: LiveTimeline(c: c))),
            ),
          ],
        ),
      ),
    );
    return c;
  }

  String firstRuler(WidgetTester tester) =>
      tester.widgetList<Text>(find.byType(Text)).map((t) => t.data ?? '').firstWhere((t) => RegExp(r'^\d\d:\d\d$').hasMatch(t));

  testWidgets('a trackpad pan moves time the way Classic does (core navigation)', (tester) async {
    await mountRows(tester, 3, frames: 3000); // 100 s: far more than the visible span
    final before = firstRuler(tester);
    final pad = TestPointer(1, PointerDeviceKind.trackpad);
    final at = Offset(tlX(4) - 344, tlTop + tlPitch - 703);
    await tester.sendEventToBinding(pad.panZoomStart(at));
    for (var i = 1; i <= 8; i++) {
      await tester.sendEventToBinding(pad.panZoomUpdate(at, pan: Offset(-40.0 * i, 0)));
      await tester.pump(const Duration(milliseconds: 16));
    }
    await tester.sendEventToBinding(pad.panZoomEnd());
    await tester.pumpAndSettle();
    expect(firstRuler(tester), isNot(before), reason: 'dragging two fingers left must move later time into view');
  });

  testWidgets('Cmd+wheel zooms around the pointer, not the middle', (tester) async {
    await mountRows(tester, 3);
    final at = Offset(tlX(1) - 344, tlTop + tlPitch - 703);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    final mouse = TestPointer(2, PointerDeviceKind.mouse);
    await tester.sendEventToBinding(mouse.hover(at));
    await tester.sendEventToBinding(mouse.scroll(const Offset(0, -200)));
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await tester.pump();
    await tester.pump();
    // the zoom tells the session how many frames are in view (new layers take that span)
    final c = (tester.state(find.byType(LiveTimeline)) as dynamic).c as EditorSession;
    final dynamic st = tester.state(find.byType(LiveTimeline));
    expect(c.visibleFrames.value, ((1177 - 215) / (st.pixelsPerFrame as double)).round());
    // the frame under the pointer (1 s) stays under it: pressing there seeks to ~frame 30
    sent.clear();
    await tester.tapAt(Offset(tlX(1) - 344, 760 - 703));
    await tester.pump(const Duration(milliseconds: 50));
    final seek = sent.lastWhere((m) => m['op'] == 'seek');
    expect((seek['frame'] as num).toDouble(), closeTo(30, 3));
  });

  testWidgets('rows below the first are hit where they are drawn (row pitch 23)', (tester) async {
    await mountRows(tester, 8);
    sent.clear();
    // the 8th row's bar, 2 px above the row's bottom edge: with the old 22 px hit pitch the lanes drifted a pixel per
    // row and this landed below the last row
    final y = tlTop + tlPitch * 7 + tlRowH - 2 - 703;
    await tester.tapAt(Offset(tlX(1.5) - 344, y));
    await tester.pump();
    final select = sent.lastWhere((m) => m['op'] == 'select');
    expect(select['ids'], [8]);
  });

  testWidgets('Option+arrows pass through the Timeline to the window (nudge), plain arrows step the frame', (tester) async {
    await mountRows(tester, 3);
    await tester.tapAt(Offset(tlX(4) - 344, tlTop + tlPitch - 703)); // an empty lane: the Timeline takes focus
    await tester.pump(const Duration(milliseconds: 50));
    sent.clear();
    await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
    await tester.pump(const Duration(milliseconds: 50));
    expect(sent.where((m) => m['op'] == 'seek'), isEmpty);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pump(const Duration(milliseconds: 50));
    expect(sent.where((m) => m['op'] == 'seek'), isNotEmpty);
  });
  TimelineModel drawn(WidgetTester tester) => (tester
          .widgetList<CustomPaint>(find.byType(CustomPaint))
          .map((p) => p.painter)
          .firstWhere((p) => p.runtimeType.toString() == '_TlPaint') as dynamic)
      .m as TimelineModel;

  testWidgets('a drag over empty tracks shows the marquee while it lasts', (tester) async {
    await mountRows(tester, 1);
    final boxes = find.byWidgetPredicate((w) =>
        w is DecoratedBox &&
        (w.decoration as BoxDecoration).color == EditorTheme.of(tester.element(find.byType(LiveTimeline))).accent.withValues(alpha: .12));
    final g = await tester.startGesture(Offset(tlX(4) - 344, tlTop + tlPitch * 3 - 703));
    await g.moveBy(const Offset(60, 20));
    await tester.pump();
    expect(drawn(tester).marquee, isNotNull);
    expect(boxes, findsOneWidget);
    await g.up();
    await tester.pump();
    expect(boxes, findsNothing);
  });

  testWidgets('a dragged bar is drawn where the pointer took it before the host replies', (tester) async {
    final c = await mountRows(tester, 1);
    c.document.value = {...c.state, 'capabilities': ['seek', 'select', 'setTiming', 'previewTimings']};
    await tester.pump();
    final before = drawn(tester).rows.first.body!.$1;
    final g = await tester.startGesture(Offset(tlX(1.5) - 344, tlTop + tlRowH / 2 - 703));
    await g.moveBy(Offset(tlUnit, 0)); // one second later
    await tester.pump();
    expect(drawn(tester).rows.first.body!.$1, closeTo(before + tlUnit, 4));
    await g.up();
    await tester.pump(const Duration(milliseconds: 50));
  });
  testWidgets('right-click on a row picks it and offers the Timeline menu (Freeze, lanes, edit)', (tester) async {
    final c = await mountRows(tester, 3);
    c.document.value = {...c.state, 'capabilities': ['seek', 'select', 'freeze', 'duplicate']};
    await tester.pump();
    // the second row's bar
    sent.clear();
    await tester.tapAt(Offset(tlX(1.5) - 344, tlTop + tlPitch + tlRowH / 2 - 703), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    expect(sent.firstWhere((m) => m['op'] == 'select')['ids'], [2]);
    for (final label in ['Freeze', 'Show animated properties', 'Show all properties', 'Hide properties', 'Duplicate', 'Group'])
      expect(find.text(label), findsOneWidget, reason: label);
    // each edit line names its key, as the editor menu does
    for (final key in ['⌘D', '⌘G', '⇧⌘G', '⌘K']) expect(find.text(key), findsOneWidget, reason: key);
    // Escape closes it without a choice, and the selection it was about stays
    // the keys alone choose: ↓ walks to the first line, Enter takes it
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pumpAndSettle();
    expect(sent.last, {'op': 'freeze', 'layer': 2, 'enabled': true});
    await tester.tapAt(Offset(tlX(1.5) - 344, tlTop + tlPitch + tlRowH / 2 - 703), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    final sentBefore = sent.length;
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(find.text('Freeze'), findsNothing);
    expect(sent.length, sentBefore);
    await tester.tapAt(Offset(tlX(1.5) - 344, tlTop + tlPitch + tlRowH / 2 - 703), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Freeze'));
    await tester.pumpAndSettle();
    expect(sent.last, {'op': 'freeze', 'layer': 2, 'enabled': true});
    // a disabled line does nothing
    await tester.tapAt(Offset(150, tlTop + tlPitch + tlRowH / 2 - 703), buttons: kSecondaryButton);
    await tester.pumpAndSettle();
    final before = sent.length;
    await tester.tap(find.text('Group'));
    await tester.pump();
    expect(sent.length, before);
  });
  testWidgets('a Media asset carried onto a row is placed there, at the frame under the pointer', (tester) async {
    final c = await mountRows(tester, 3);
    c.document.value = {...c.state, 'capabilities': ['seek', 'select', 'placeAsset']};
    await tester.pumpWidget(
      Directionality(
        textDirection: TextDirection.ltr,
        child: Overlay(
          key: const ValueKey('drop'),
          initialEntries: [
            OverlayEntry(
              builder: (_) => Stack(children: [
                Align(alignment: Alignment.topLeft, child: SizedBox(width: 1178, height: 291, child: LiveTimeline(c: c))),
                Positioned(
                  left: 0,
                  top: 0,
                  width: 20,
                  height: 20,
                  child: Draggable<Map<String, dynamic>>(
                    data: const {'asset': '12', 'name': 'clip'},
                    dragAnchorStrategy: pointerDragAnchorStrategy,
                    feedback: const SizedBox(width: 8, height: 8),
                    child: const ColoredBox(color: Color(0xFF000000)),
                  ),
                ),
              ]),
            ),
          ],
        ),
      ),
    );
    sent.clear();
    final g = await tester.startGesture(const Offset(10, 10));
    await tester.pump();
    final target = Offset(tlX(2) - 344, tlTop + tlPitch + tlRowH / 2 - 703); // the second row's bar, 2 s in
    await g.moveTo(target - const Offset(40, 0));
    await tester.pump();
    await g.moveTo(target);
    await tester.pump();
    expect(drawn(tester).dropGuide, isNotNull);
    await g.up();
    await tester.pump();
    final place = sent.lastWhere((m) => m['op'] == 'placeAsset');
    expect(place['id'], '12');
    expect(place['target'], 2);
    expect((place['start'] as num).toDouble(), closeTo(60, 2));
    expect(drawn(tester).dropGuide, isNull);
  });
  testWidgets('small downward wheel steps add up to scrolling a row', (tester) async {
    await mountRows(tester, 12);
    expect(find.text('L0'), findsOneWidget);
    final mouse = TestPointer(3, PointerDeviceKind.mouse);
    final at = Offset(tlX(4) - 344, tlTop + tlPitch * 2 - 703);
    await tester.sendEventToBinding(mouse.hover(at));
    for (var k = 0; k < 3; k++) {
      await tester.sendEventToBinding(mouse.scroll(const Offset(0, 10)));
      await tester.pump(const Duration(milliseconds: 16));
    }
    await tester.pumpAndSettle();
    expect(find.text('L0'), findsNothing, reason: '30 px of wheel is more than one 23 px row');
  });
  testWidgets('a hidden layer is grey, a ghost shows past its bar, a lane draws its spans', (tester) async {
    final c = await mountRows(tester, 2);
    c.document.value = {
      ...c.state,
      'layers': [
        {'id': 1, 'name': 'L0', 'kind': 'Shape', 'start': 0, 'duration': 30, 'hidden': true, 'ghost': 6, 'properties': []},
        {
          'id': 2,
          'name': 'L1',
          'kind': 'Shape',
          'start': 0,
          'duration': 90,
          'properties': [
            {
              'id': 'opacity',
              'label': 'Opacity',
              'keys': [
                {'frame': 0, 'interp': {'kind': 'Linear'}},
                {'frame': 30, 'interp': {'kind': 'EasyEase'}},
                {'frame': 60},
              ],
            },
          ],
        },
      ],
    };
    await tester.pump();
    await tester.tapAt(const Offset(134, 106)); // the second row's properties toggle
    await tester.pump();
    final rows = drawn(tester).rows;
    expect(rows, hasLength(3), reason: 'the opened lane is drawn at once');
    expect(rows[0].body!.$3, H.raisedHi);
    expect(rows[0].ghost, isNotNull);
    expect(rows[0].ghost!.$1, closeTo(rows[0].body!.$2, .01));
    final lane = rows.firstWhere((r) => r.lane);
    expect(lane.spans.map((s) => s.$3), [true, false], reason: 'linear, then shaped');
  });
  testWidgets('a marker dragged along the ruler lands on the frame under it', (tester) async {
    final c = await mountRows(tester, 1);
    c.document.value = {
      ...c.state,
      'capabilities': ['seek', 'select', 'setMarker'],
      'markers': [
        {'id': '30/1', 'frame': 30},
      ],
    };
    await tester.pump();
    final at = Offset(tlX(1) - 344, 758 - 703); // the marker's handle at 1 s
    final g = await tester.startGesture(at);
    for (var k = 1; k <= 5; k++) {
      await g.moveTo(at + Offset(tlUnit * k / 5, 0));
      await tester.pump();
    }
    await g.up();
    await tester.pump();
    final set = sent.lastWhere((m) => m['op'] == 'setMarker');
    expect(set['id'], '30/1');
    expect((set['frame'] as num).toDouble(), closeTo(60, 2));
  });
}
