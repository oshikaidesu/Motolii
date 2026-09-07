import 'dart:convert';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/browser.dart';
import '../lib/panels/stage.dart';
import '../lib/foundation/theme.dart';

Future<Uint8List> twoTonePng() async {
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);
  canvas.drawRect(
    const Rect.fromLTWH(0, 0, 16, 32),
    Paint()..color = const Color(0xffff0000),
  );
  canvas.drawRect(
    const Rect.fromLTWH(16, 0, 16, 32),
    Paint()..color = const Color(0xff0000ff),
  );
  final image = await recorder.endRecording().toImage(32, 32);
  final data = await image.toByteData(format: ui.ImageByteFormat.png);
  return data!.buffer.asUint8List();
}

void main() {
  testWidgets('A picture becomes its main colours', (tester) async {
    late List<List<double>> colors;
    await tester.runAsync(() async {
      colors = await paletteOf(await twoTonePng(), count: 4);
    });
    expect(colors.length, 2);
    expect(colors.first[0], closeTo(1, .02));
    expect(colors.last[2], closeTo(1, .02));
  });

  testWidgets('The line under the picker drags the wheel size', (tester) async {
    tester.view.physicalSize = const Size(400, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
    final c = EditorSession();
    c.document.value = {
      'capabilities': ['applyPalette'],
      'palette': [],
    };
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: BrowserPanel(controller: c, fixedTab: 'Colors'),
        ),
      ),
    );
    final grip = find.byKey(const ValueKey('browser:picker-grip'));
    final before = tester.getTopLeft(grip).dy;
    await tester.drag(grip, const Offset(0, -100));
    await tester.pump();
    expect(tester.getTopLeft(grip).dy, lessThan(before));
    expect(c.deskWork.value['browserWheel'], 128.0);
  });

  testWidgets('Eyedropper: one Stage click reads a pixel and applies it', (
    tester,
  ) async {
    final commands = <Map<String, dynamic>>[];
    const picked = [.1, .2, .3, 1.0];
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(EditorSession.channel, (call) async {
          final args = call.arguments;
          if (args is Map && args['command'] is String) {
            final command = Map<String, dynamic>.from(
              jsonDecode(args['command'] as String) as Map,
            );
            commands.add(command);
            return {
              'status': {
                'width': 640,
                'height': 480,
                'layers': [],
                'selectedIds': [],
                'capabilities': [
                  'pickColor',
                  'applyPalette',
                  'stageGesture',
                  'select',
                ],
                if (command['op'] == 'pickColor') 'pickedColor': picked,
              },
            };
          }
          return <String, dynamic>{};
        });
    final c = EditorSession();
    c.document.value = {
      'width': 640,
      'height': 480,
      'layers': [],
      'selectedIds': [],
      'capabilities': ['pickColor', 'applyPalette', 'stageGesture', 'select'],
    };
    await tester.pumpWidget(
      MaterialApp(
        theme: EditorTheme.data,
        home: Scaffold(
          body: SizedBox(
            width: 600,
            height: 400,
            child: StagePanel(controller: c),
          ),
        ),
      ),
    );
    await tester.pump();
    c.eyedropper.value = true;
    await tester.tapAt(tester.getCenter(find.byType(StagePanel)));
    await tester.pump(const Duration(seconds: 1));
    expect(c.eyedropper.value, isFalse);
    final ops = commands.map((m) => m['op']).toList();
    expect(ops.indexOf('pickColor'), greaterThanOrEqualTo(0));
    expect(ops.indexOf('applyPalette'), greaterThan(ops.indexOf('pickColor')));
    expect(
      commands.firstWhere((m) => m['op'] == 'applyPalette')['rgba'],
      picked,
    );
    await tester.pumpWidget(const SizedBox());
  });
}
