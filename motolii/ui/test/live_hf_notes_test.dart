import 'dart:convert';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';

import '../lib/hf/desk/notes.dart';
import '../lib/live_hf/adapters/notes.dart';
import '../lib/session/editor_session.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final sent = <Map>[];
  setUp(() {
    sent.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async {
      final args = call.arguments;
      if (args is Map && args['command'] is String) sent.add(jsonDecode(args['command'] as String) as Map);
      return <String, dynamic>{};
    });
  });

  test('the notebook\'s blocks, in the desk\'s kinds', () {
    final c = EditorSession()
      ..document.value = {
        'notebook': {
          'pages': [
            {
              'id': 'a',
              'title': 'A',
              'blocks': [
                {'id': 't', 'x': 1, 'y': 2, 'width': 100, 'height': 60, 'kind': 'text', 'text': 'hello'},
                {'id': 'r', 'x': 5, 'y': 6, 'width': 90, 'height': 32, 'kind': 'reference', 'label': 'Title', 'layer': 4, 'start': 0, 'end': 30},
                {'id': 'z', 'x': 0, 'y': 0, 'width': 50, 'height': 40, 'kind': 'sketch'},
              ],
            },
          ],
        },
      };
    final host = LiveNotesHost(c);
    final b = host.blocks(0);
    expect([for (final x in b) x.kind], ['note', 'ref']);
    expect(b[0].text, 'hello');
    expect(b[1].text, 'Title');
    expect(b[0].id, 't');
    expect(host.can('hand'), isFalse);
    host.dispose();
  });

  test('the first note on an empty notebook makes its page; a patch keeps the host\'s bounds', () async {
    final c = EditorSession()..document.value = {'notebook': {'pages': []}};
    final host = LiveNotesHost(c);
    expect(host.pageCount, 1);
    await host.put(0, NBlock('note', const Offset(-10, 5), const Size(30, 20), 'New note'));
    expect([for (final m in sent) m['action']], ['addPage', 'putBlock']);
    expect(sent[1]['block'], containsPair('x', 0.0));
    expect(sent[1]['block'], containsPair('width', 40.0));
    expect(sent[1]['block'], containsPair('height', 32.0));
    expect(sent[1]['block'], containsPair('kind', 'text'));

    sent.clear();
    c.document.value = {'notebook': {'pages': [{'id': 'p1', 'title': 'Page 1', 'blocks': []}]}};
    await host.patch(0, NBlock('note', const Offset(12, -3), const Size(80, 60), 'x', 0, 'b1'), {'x': 12, 'y': -3});
    expect(sent.single['patch'], {'x': 12.0, 'y': 0.0});
    host.dispose();
  });
}
