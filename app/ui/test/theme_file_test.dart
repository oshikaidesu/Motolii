import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/theme_settings.dart';
import '../lib/foundation/leaves.dart';
import '../lib/foundation/theme.dart';
import '../lib/session/editor_session.dart';

void main() {
  testWidgets(
    'window theme broadcasts preserve document and local desk state',
    (tester) async {
      final c = EditorSession();
      addTearDown(c.dispose);
      c.document.value = {
        'selectedIds': [7],
        'frame': 135,
        'layers': [],
      };
      final document = c.document.value;
      c.deskWork.value = {'browserTile': 72};
      var changes = 0;
      c.deskWork.addListener(() => changes++);
      final message = {
        'places': {'Create': 'tab'},
        'drawer': null,
        'theme': {
          'path': 'third-party.json',
          'data': EditorTheme.chromatic.toJson(),
        },
      };
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      for (var i = 0; i < 2; i++) {
        await messenger.handlePlatformMessage(
          EditorSession.channel.name,
          const StandardMethodCodec().encodeMethodCall(
            MethodCall('paneState', message),
          ),
          (_) {},
        );
      }
      expect(c.document.value, same(document));
      expect(c.deskWork.value['browserTile'], 72);
      expect(changes, 1);
      expect(
        EditorTheme.fromJson((c.deskWork.value['theme'] as Map)['data']).name,
        EditorTheme.chromatic.name,
      );
    },
  );
  test('published schema and example cover the actual theme interface', () {
    final schema =
        jsonDecode(File('themes/theme.schema.json').readAsStringSync()) as Map;
    final fields = schema['properties'] as Map;
    expect(
      (fields['colors']['properties'] as Map).keys.toSet(),
      EditorTheme.chromatic.colors.keys.toSet(),
    );
    expect(
      (fields['drawing']['properties'] as Map).keys.toSet(),
      EditorTheme.chromatic.drawing.namedColors.keys.toSet(),
    );
    final sample = EditorTheme.fromJson(
      jsonDecode(File('themes/velvet.json').readAsStringSync()),
    );
    expect(sample.name, 'Velvet');
    expect(sample.panel, isNot(EditorTheme.chromatic.panel));
  });
  test('theme export is a complete round trip including alpha and drawing', () {
    final parsed = EditorTheme.fromJson(jsonDecode(EditorTheme.chromatic.json));
    expect(parsed.toJson(), EditorTheme.chromatic.toJson());
    const codec = StandardMessageCodec();
    final saved = codec.decodeMessage(codec.encodeMessage(parsed.toJson()));
    expect(EditorTheme.fromJson(saved).toJson(), parsed.toJson());
    final custom = EditorTheme.fromJson({
      'schemaVersion': 1,
      'name': 'Custom',
      'colors': {'accent': '#12345680'},
      'drawing': {'grid': '#abcdef'},
      'identityColors': ['#123456', '#abcdef'],
    });
    expect(custom.accent.toARGB32(), 0x80123456);
    expect(custom.drawing.grid.toARGB32(), 0xffabcdef);
    expect(custom.layerColor(3).toARGB32(), 0xffabcdef);
    expect(custom.panel, EditorTheme.chromatic.panel);
    expect(EditorTheme.fromJson(custom.toJson()).toJson(), custom.toJson());
    expect(() => custom.colors['accent'] = Colors.red, throwsUnsupportedError);
  });

  test('bad keys, versions, values and empty palettes are rejected', () {
    for (final patch in <Map<String, dynamic>>[
      {'schemaVersion': 2},
      {
        'colors': {'accnet': '#abcdef'},
      },
      {
        'colors': {'accent': 'red'},
      },
      {
        'drawing': {'notAColor': '#abcdef'},
      },
      {'identityColors': []},
      {
        'collectionColors': ['#abcdef'],
      },
      {'script': 'do not execute'},
      {'name': ''},
    ]) {
      expect(
        () =>
            EditorTheme.fromJson({'schemaVersion': 1, 'name': 'Bad', ...patch}),
        throwsFormatException,
        reason: '$patch',
      );
    }
  });

  testWidgets(
    'switch recolors const controls and painters without losing a draft',
    (tester) async {
      final appearance = ValueNotifier(EditorTheme.chromatic);
      final draft = TextEditingController(text: 'Keep this draft');
      addTearDown(appearance.dispose);
      addTearDown(draft.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: ValueListenableBuilder(
            valueListenable: appearance,
            builder: (context, theme, _) => theme.wrap(
              Column(
                children: [
                  const EditorRule(key: ValueKey('rule'), thickness: 1),
                  const EditorSlider(
                    value: .5,
                    min: 0,
                    max: 1,
                    onChanged: null,
                  ),
                  EditorTextField(
                    key: const ValueKey('draft'),
                    controller: draft,
                  ),
                ],
              ),
            ),
          ),
        ),
      );
      final state = tester.state(find.byKey(const ValueKey('draft')));
      final oldPainters = tester
          .widgetList<CustomPaint>(find.byType(CustomPaint))
          .map((w) => w.painter)
          .whereType<CustomPainter>()
          .toList();
      final custom = EditorTheme.fromJson({
        'schemaVersion': 1,
        'name': 'Switch',
        'colors': {'line': '#eeddcc', 'caret': '#66ddaa', 'ink': '#ffeeaa'},
      });
      appearance.value = custom;
      await tester.pump();
      expect(
        identical(tester.state(find.byKey(const ValueKey('draft'))), state),
        isTrue,
      );
      expect(draft.text, 'Keep this draft');
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).cursorColor,
        custom.caret,
      );
      final rule = tester.widget<Container>(
        find.descendant(
          of: find.byKey(const ValueKey('rule')),
          matching: find.byType(Container),
        ),
      );
      expect(
        ((rule.decoration as BoxDecoration).border as Border).bottom.color,
        custom.line,
      );
      final newPainters = tester
          .widgetList<CustomPaint>(find.byType(CustomPaint))
          .map((w) => w.painter)
          .whereType<CustomPainter>()
          .toList();
      expect(newPainters.length, oldPainters.length);
      expect([
        for (var i = 0; i < newPainters.length; i++)
          newPainters[i].shouldRepaint(oldPainters[i]),
      ], contains(true));
      await tester.pumpWidget(const SizedBox());
    },
  );

  testWidgets('load saves a valid snapshot; failed reload keeps it intact', (
    tester,
  ) async {
    final folder = Directory.systemTemp.createTempSync('motolii-theme-test-');
    final file = File('${folder.path}/theme.json');
    file.writeAsStringSync(
      jsonEncode({
        'schemaVersion': 1,
        'name': 'Third-party',
        'colors': {'tab': '#eeccee'},
      }),
    );
    final c = (await tester.runAsync(() async => EditorSession()))!;
    Map<String, dynamic> settings = {};
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          if (call.method == 'pickImport') return [file.path];
          if (call.method == 'readSettings') return settings;
          if (call.method == 'writeSettings') {
            settings = Map.from(call.arguments as Map);
            return true;
          }
          return <String, dynamic>{};
        });
    addTearDown(() async {
      c.dispose();
      folder.deleteSync(recursive: true);
    });
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(body: ThemeSettings(controller: c)),
      ),
    );
    await tester.runAsync(() async {
      await tester.tap(find.byKey(const ValueKey('theme:load')));
      for (var i = 0; i < 100 && settings['deskWork'] == null; i++) {
        await Future<void>.delayed(const Duration(milliseconds: 10));
      }
    });
    await tester.pumpAndSettle();
    final saved = c.deskWork.value['theme'];
    expect((saved as Map)['data']['name'], 'Third-party');
    expect(settings['deskWork']['theme'], saved);
    await tester.runAsync(() async {
      file.writeAsStringSync('{bad JSON');
      await tester.tap(find.byKey(const ValueKey('theme:reload')));
      await Future<void>.delayed(const Duration(milliseconds: 100));
    });
    await tester.pumpAndSettle();
    expect(c.deskWork.value['theme'], same(saved));
    expect(find.textContaining('Theme unchanged:'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('theme:default')));
    await tester.pumpAndSettle();
    expect(c.deskWork.value['theme'], isNull);
    await tester.pumpWidget(const SizedBox());
  });
}
