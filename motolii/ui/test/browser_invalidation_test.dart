// A Stage preview must wake only the UI whose visible meaning changed: a Position scrub rewrites `layers` and
// `documentRevision` on every tick, and the Browser (and the Colors instrument) must stay still through it.
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_ui/browser/session.dart';
import 'package:motolii_ui/inspector/session.dart';
import 'package:motolii_ui/session/editor_session.dart';

Map<String, dynamic> doc({double x = 0, String family = 'Inter', List fill = const [0.5, 0.5, 0.5, 1], int rev = 1}) => {
      'documentRevision': rev,
      'selectedIds': [1],
      'selectedId': 1,
      'fontFamilies': ['Inter'],
      'capabilities': ['setColor'],
      'layers': [
        {
          'id': 1,
          'name': 'Title',
          'kind': 'Text',
          'text': {'fontFamily': family},
          'fill': {
            'stops': [
              {'rgba': fill}
            ]
          },
          'properties': [
            {'id': 'position.x', 'value': x}
          ],
        }
      ],
    };

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('Browser is woken by what it shows, not by a Position preview', () {
    final c = EditorSession();
    c.document.value = doc();
    final browser = BrowserSession.of(c);
    var woken = 0;
    browser.addListener(() => woken++);

    for (var i = 1; i <= 5; i++) {
      c.document.value = doc(x: i * 10.0, rev: 1 + i);
    }
    expect(woken, 0, reason: 'Position ticks change nothing a shelf draws');

    c.document.value = doc(x: 50, fill: const [1, 0, 0, 1], rev: 20);
    expect(woken, 1, reason: 'the fill colour is the Colors shelf current colour');

    c.document.value = doc(x: 50, fill: const [1, 0, 0, 1], family: 'Mono', rev: 21);
    expect(woken, 2, reason: 'the font a text layer uses is shown as chosen and as used');
    c.dispose();
  });

  test('Inspector seat shape ignores property values, sees structure', () {
    final c = EditorSession();
    c.document.value = doc();
    final seat = InspectorSession.of(c);
    final before = seat.shape;
    c.document.value = doc(x: 99, rev: 9);
    expect(sameValue(seat.shape, before), isTrue, reason: 'a Position preview is not a change of what the seat draws');
    final renamed = doc();
    (renamed['layers'] as List).first['name'] = 'Other';
    c.document.value = renamed;
    expect(sameValue(seat.shape, before), isFalse);
    c.dispose();
  });
}
