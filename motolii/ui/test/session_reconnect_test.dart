import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';

void main() {
  testWidgets(
    'reattaching during playback resumes render cadence without restarting the clock',
    (tester) async {
      var frame = 40, ticks = 0;
      var playing = true;
      final requests = <String>[];
      Map<String, dynamic> status() => {
        'frame': frame,
        'playing': playing,
        'durationFrames': 300,
        'capabilities': ['play', 'pause'],
        'layers': [],
        'selectedIds': [],
      };
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(EditorSession.channel, (call) async {
            switch (call.method) {
              case 'windowInfo':
                return {'main': true};
              case 'attach':
                return {'textureId': 1, 'status': status()};
              case 'render':
                if ((call.arguments as Map?)?['playing'] == true) {
                  ticks++;
                  frame++;
                }
                return {'textureId': 1, 'frameReady': true, 'status': status()};
              case 'request':
                final op =
                    jsonDecode(
                          (call.arguments as Map)['command'] as String,
                        )['op']
                        as String;
                requests.add(op);
                if (op == 'pause') playing = false;
                return status();
              case 'close':
                return {};
            }
            throw StateError('Unexpected method ${call.method}');
          });
      final c = EditorSession();
      await c.initialize();
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 16));
      await tester.pump(const Duration(milliseconds: 16));
      expect(ticks, greaterThanOrEqualTo(2));
      expect(c.frame.value, greaterThan(40));
      expect(requests, isNot(contains('play')));
      await c.command('pause');
      await tester.pump();
      final stopped = frame;
      await tester.pump(const Duration(milliseconds: 100));
      expect(frame, stopped);
      expect(c.playing.value, isFalse);
      c.dispose();
      await tester.pump();
      expect(tester.takeException(), isNull);
    },
  );
}
