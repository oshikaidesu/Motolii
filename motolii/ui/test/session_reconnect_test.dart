import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';

void main() {
  testWidgets(
    'reattaching during playback receives native frames without restarting the clock',
    (tester) async {
      var frame = 40;
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
      await TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .handlePlatformMessage(
            EditorSession.channel.name,
            const StandardMethodCodec().encodeMethodCall(
              MethodCall('playbackFrame', {'frame': 41}),
            ),
            null,
          );
      await tester.pump();
      expect(c.frame.value, 41);
      expect(requests, isEmpty);
      expect(requests, isNot(contains('play')));
      expect(requests, isNot(contains('tick')));
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
