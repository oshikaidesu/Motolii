import 'dart:async';
import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/session/editor_session.dart';
import '../lib/panels/stage.dart';
import '../lib/foundation/theme.dart';

class AttachmentHost {
  AttachmentHost({this.main = true});
  final bool main;
  int sequence = 0;
  int runtimeEpoch = 1;
  Map<String, dynamic>? stageWindow;
  int? current;
  bool playing = false;
  bool attaching = false;
  Completer<void>? pendingAttach;
  final calls = <String>[];
  final detached = <int>[];
  final projections = <String>[];

  Map<String, dynamic> status([int frame = 40]) => {
    'frame': frame,
    'width': 640,
    'height': 360,
    'documentRevision': 'same-content',
    'contentRevision': 'same-content',
    'stageWindow': stageWindow,
    'playing': playing,
    'durationFrames': 300,
    'capabilities': ['play', 'pause', 'stageWindow'],
    'layers': [],
    'selectedIds': [],
  };

  Future<dynamic> handle(MethodCall call) async {
    calls.add(call.method);
    final args = Map<String, dynamic>.from(call.arguments as Map? ?? {});
    switch (call.method) {
      case 'windowInfo':
        return {'id': main ? 'main' : 'panel', 'main': main};
      case 'attach':
        final expected = args['attachmentId'];
        if (expected != null && expected != current) {
          throw PlatformException(code: 'stale');
        }
        final lease = expected as int? ?? ++sequence;
        current = lease;
        attaching = true;
        await pendingAttach?.future;
        return {
          'attachmentId': lease,
          'runtimeEpoch': runtimeEpoch,
          'textureId': 1,
          'status': status(),
        };
      case 'open':
        runtimeEpoch++;
        stageWindow = null;
        return {
          'attachmentId': current,
          'runtimeEpoch': runtimeEpoch,
          'textureId': 1,
          'status': status(),
        };
      case 'detach':
        final lease = args['attachmentId'] as int;
        detached.add(lease);
        if (current == lease) current = null;
        return {'detached': current == null};
      case 'render':
        return {'textureId': 1, 'status': status()};
      case 'request':
        final request =
            jsonDecode(args['command'] as String) as Map<String, dynamic>;
        final op = request['op'];
        calls.add('request:$op');
        if (op == 'pause') playing = false;
        if (op == 'play') playing = true;
        if (op == 'preferences')
          projections.add(request['flatProjection'] as String);
        if (op == 'stageWindow') stageWindow = Map.of(request)..remove('op');
        return status();
      default:
        throw StateError('Unexpected method ${call.method}');
    }
  }
}

void main() {
  testWidgets('new runtimes receive the restored creation preference', (
    tester,
  ) async {
    final host = AttachmentHost();
    final messenger =
        TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
    messenger.setMockMethodCallHandler(EditorSession.channel, host.handle);
    final session = EditorSession();
    await session.initialize();
    session.restoreDeskWork({'flatProjection': '3D'});
    await tester.pump();
    expect(host.projections.last, '3D');
    host.projections.clear();
    session.deskWork.value = {...session.deskWork.value, 'easePresets': []};
    await tester.pump();
    expect(host.projections, isEmpty);
    await session.open('replacement.rrd');
    await tester.pump();
    expect(host.projections, ['3D']);
    session.dispose();
    await tester.pump();
    messenger.setMockMethodCallHandler(EditorSession.channel, null);
  });
  testWidgets(
    'Stage reconnects when a runtime is replaced at identical document dimensions',
    (tester) async {
      final host = AttachmentHost();
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(EditorSession.channel, host.handle);
      final session = EditorSession();
      addTearDown(() {
        session.dispose();
        messenger.setMockMethodCallHandler(EditorSession.channel, null);
      });
      await session.initialize();
      await tester.pumpWidget(
        WidgetsApp(
          color: EditorTheme.chromatic.app,
          textStyle: EditorTheme.chromatic.text,
          pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(
            settings: settings,
            pageBuilder: (context, _, __) => builder(context),
          ),
          home: Center(
            child: SizedBox(
              width: 600,
              height: 400,
              child: StagePanel(controller: session),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      final configured = host.calls
          .where((call) => call == 'request:stageWindow')
          .length;
      expect(configured, greaterThan(0));
      expect(session.runtimeEpoch.value, 1);
      await session.open('same-sized-document.rrd');
      await tester.pumpAndSettle();
      expect(session.runtimeEpoch.value, 2);
      expect(
        host.calls.where((call) => call == 'request:stageWindow').length,
        greaterThan(configured),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      session.dispose();
      await tester.pump();
    },
  );
  testWidgets(
    'the main viewer does not render playback started by another window',
    (tester) async {
      final host = AttachmentHost();
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(EditorSession.channel, host.handle);
      final session = EditorSession();
      addTearDown(() {
        session.dispose();
        messenger.setMockMethodCallHandler(EditorSession.channel, null);
      });
      await session.initialize();
      await tester.pump();
      await tester.pump();
      final before = host.calls.where((call) => call == 'render').length;
      host.playing = true;
      await messenger.handlePlatformMessage(
        EditorSession.channel.name,
        const StandardMethodCodec().encodeMethodCall(
          MethodCall('documentChanged', {'status': host.status()}),
        ),
        null,
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 16));
      expect(host.calls.where((call) => call == 'render').length, before);
      expect(host.calls, isNot(contains('request:play')));
      session.dispose();
      await tester.pump();
    },
  );

  testWidgets(
    'a panel can request playback without owning the shared render cadence',
    (tester) async {
      final host = AttachmentHost(main: false);
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(EditorSession.channel, host.handle);
      final session = EditorSession();
      addTearDown(() {
        session.dispose();
        messenger.setMockMethodCallHandler(EditorSession.channel, null);
      });
      await session.initialize();
      await session.command('play');
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 32));
      expect(host.calls, contains('request:play'));
      expect(host.calls, isNot(contains('render')));
      session.dispose();
      await tester.pump();
      expect(host.playing, isTrue);
      expect(host.calls, isNot(contains('request:pause')));
    },
  );
  testWidgets(
    'disposing a UI detaches its lease without closing the work or pausing playback',
    (tester) async {
      final host = AttachmentHost()..playing = true;
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(EditorSession.channel, host.handle);
      final session = EditorSession();
      addTearDown(() {
        session.dispose();
        messenger.setMockMethodCallHandler(EditorSession.channel, null);
      });
      await session.initialize();
      session.dispose();
      await tester.pump();
      expect(host.detached, [1]);
      expect(host.calls, isNot(contains('close')));
      expect(host.calls, isNot(contains('request:pause')));
      expect(host.playing, isTrue);
      expect(tester.takeException(), isNull);
    },
  );

  testWidgets(
    'late disposal cannot detach the successor or erase its event listener',
    (tester) async {
      final host = AttachmentHost();
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(EditorSession.channel, host.handle);
      final old = EditorSession();
      await old.initialize();
      final next = EditorSession();
      addTearDown(() {
        old.dispose();
        next.dispose();
        messenger.setMockMethodCallHandler(EditorSession.channel, null);
      });
      await next.initialize();
      await next.attachView('User');
      expect(
        host.current,
        2,
        reason: 'adding another texture reuses the same UI lease',
      );
      old.dispose();
      await tester.pump();
      expect(host.current, 2);
      await messenger.handlePlatformMessage(
        EditorSession.channel.name,
        const StandardMethodCodec().encodeMethodCall(
          MethodCall('documentChanged', {'status': host.status(123)}),
        ),
        null,
      );
      expect(next.state['frame'], 123);
      next.dispose();
      await tester.pump();
      expect(host.detached, [1, 2]);
      expect(host.current, isNull);
      expect(host.calls, isNot(contains('close')));
    },
  );

  testWidgets(
    'an attachment arriving after disposal is retired without rendering or opening a document',
    (tester) async {
      final pending = Completer<void>();
      final host = AttachmentHost()..pendingAttach = pending;
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(EditorSession.channel, host.handle);
      final session = EditorSession();
      addTearDown(() {
        session.dispose();
        messenger.setMockMethodCallHandler(EditorSession.channel, null);
      });
      final initialized = session.initialize();
      await tester.pump();
      expect(host.attaching, isTrue);
      session.dispose();
      pending.complete();
      await initialized;
      await tester.pump();
      expect(host.detached, [1]);
      expect(
        host.calls.where(
          (call) => ['render', 'open', 'close', 'request:pause'].contains(call),
        ),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
    },
  );
}
