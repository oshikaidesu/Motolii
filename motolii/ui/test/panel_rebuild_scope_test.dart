import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/panel_controls.dart';
import '../lib/foundation/theme.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/font_browser.dart';
import '../lib/panels/history_records.dart';
import '../lib/panels/inspector.dart';
import '../lib/panels/timeline.dart';
import '../lib/session/editor_session.dart';

/// A panel rebuilds when the widget it hands back is a fresh object.
class _Rebuilds {
  _Rebuilds(this.name, this.finder);
  final String name;
  final Finder finder;
  Widget? _last;
  int count = 0;
  void sample(WidgetTester tester) {
    final found = finder.evaluate();
    if (found.isEmpty) throw StateError('no anchor for $name');
    final now = found.first.widget;
    if (_last != null && !identical(now, _last)) count++;
    _last = now;
  }
}

Finder _inside(Type panel, Finder what) =>
    find.descendant(of: find.byType(panel), matching: what);

void main() {
  testWidgets('a preview of one Inspector value rebuilds only what reads it', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1600, 1200);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final c = EditorSession();
    Map<String, dynamic> layer(double x) => {
      'id': 7,
      'kind': 'Text',
      'name': 'Title',
      'locked': false,
      'colors': const [],
      'text': {'content': 'Hello', 'fontFamily': 'Arial', 'styles': [], 'runs': []},
      'properties': [
        {'id': 'position', 'label': 'Position', 'kind': 'point', 'value': [x, 0.0]},
      ],
      'effects': const [],
    };
    c.document.value = {
      'layers': [layer(0)],
      'selectedIds': [7],
      'selectedKeys': const [],
      'assets': const [],
      'backgrounds': const [],
      'catalog': const [],
      'palette': const [],
      'fontFamilies': const ['Arial', 'Georgia'],
      'history': const {
        'head': 0,
        'entries': [
          {'head': 0, 'kind': 'edit', 'label': 'Open', 'detail': ''},
        ],
      },
      'capabilities': const ['setFont', 'preview', 'animate'],
      'durationFrames': 60,
      'fps': 30.0,
      'width': 640,
      'height': 480,
      'contentRevision': 'r0',
      'snapshotId': 1,
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: Column(
            children: [
              SizedBox(
                height: 300,
                child: Row(
                  children: [
                    SizedBox(width: 300, child: BrowserPanel(controller: c, fixedTab: 'Media', showTabs: false)),
                    SizedBox(width: 300, child: InspectorPanel(controller: c)),
                    SizedBox(width: 300, child: FontBrowser(controller: c)),
                    SizedBox(width: 300, child: HistoryRecords(controller: c)),
                  ],
                ),
              ),
              SizedBox(height: 300, child: TimelinePanel(controller: c)),
            ],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    final watched = [
      _Rebuilds('Timeline', _inside(TimelinePanel, find.byType(Focus))),
      _Rebuilds('Browser', _inside(BrowserPanel, find.byType(Focus))),
      _Rebuilds('Inspector', _inside(InspectorPanel, find.byType(LayoutBuilder))),
      _Rebuilds(
        'Position X',
        _inside(InspectorPanel, find.byType(EditorNumericField)),
      ),
      _Rebuilds('Fonts', _inside(FontBrowser, find.byType(ColoredBox))),
      _Rebuilds('History', _inside(HistoryRecords, find.byType(LayoutBuilder))),
    ];
    for (final w in watched) {
      w.sample(tester);
    }
    // Three drag frames of a preview: only the moved values and the render
    // bookkeeping come back.
    for (var step = 1; step <= 3; step++) {
      c.document.value = {
        ...c.state,
        'layers': [layer(step.toDouble())],
        'contentRevision': 'r$step',
        'renderMs': step.toDouble(),
        'snapshotId': 1 + step,
      };
      await tester.pumpAndSettle();
      for (final w in watched) {
        w.sample(tester);
      }
    }
    final counted = {for (final w in watched) w.name: w.count};
    debugPrint('PROBE room=rebuild-scope builds=$counted');
    expect(
      counted['Position X'],
      3,
      reason: 'the well that shows the value',
    );
    expect(
      counted['Inspector'],
      0,
      reason: 'the frame of the panel does not read the number',
    );
    expect(counted['Timeline'], 3, reason: 'the lanes show the same values');
    expect(counted['Browser'], 0);
    expect(counted['Fonts'], 0);
    expect(counted['History'], 0);
    await tester.pumpWidget(const SizedBox());
    c.dispose();
  });

  testWidgets('a status that repeats itself notifies nobody', (tester) async {
    final c = EditorSession();
    c.document.value = {
      'layers': [
        {'id': 1, 'name': 'One'},
      ],
      'selectedIds': [1],
    };
    var notified = 0;
    c.document.addListener(() => notified++);
    final held = c.state;
    c.absorb({
      'layers': [
        {'id': 1, 'name': 'One'},
      ],
      'selectedIds': [1],
    });
    expect(notified, 0);
    expect(identical(c.state, held), isTrue);
    c.absorb({'renderMs': 4.0});
    expect(notified, 1);
    expect(identical(c.state['layers'], held['layers']), isTrue);
    c.dispose();
  });
}
