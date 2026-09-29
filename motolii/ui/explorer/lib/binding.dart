// Production's EditorSession talks to its host over the `motolii/probe` channel, which the macOS runner answers.
// Here the explorer answers it, forwarding to the real host (host.dart). Production code is not touched: this binding
// only sits between the channel and the platform.
import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import 'host.dart';

class ExplorerBinding extends WidgetsFlutterBinding {
  static ExplorerBinding? _instance;
  static ExplorerBinding ensure() => _instance ??= ExplorerBinding();

  /// The host the story on screen talks to.
  RealHost? host;

  @override
  BinaryMessenger createBinaryMessenger() => _ProbeMessenger(super.createBinaryMessenger(), this);
}

class _ProbeMessenger extends BinaryMessenger {
  _ProbeMessenger(this._platform, this._binding);
  final BinaryMessenger _platform;
  final ExplorerBinding _binding;
  static const _codec = StandardMethodCodec();

  @override
  Future<void> handlePlatformMessage(String channel, ByteData? data, PlatformMessageResponseCallback? callback) =>
      _platform.handlePlatformMessage(channel, data, callback);

  @override
  void setMessageHandler(String channel, MessageHandler? handler) => _platform.setMessageHandler(channel, handler);

  @override
  Future<ByteData?>? send(String channel, ByteData? message) {
    if (channel != 'motolii/probe') return _platform.send(channel, message);
    final call = _codec.decodeMethodCall(message);
    try {
      return Future.value(_codec.encodeSuccessEnvelope(_answer(call.method, (call.arguments as Map?)?.cast<String, dynamic>() ?? const {})));
    } catch (e) {
      return Future.value(_codec.encodeErrorEnvelope(code: 'probe', message: '$e'));
    }
  }

  Map<String, dynamic> _status(RealHost host) => host.request({'op': 'status'});

  Object? _answer(String method, Map<String, dynamic> args) {
    final host = _binding.host;
    switch (method) {
      case 'windowInfo':
        return {'id': 'explorer', 'main': true, 'panels': const [], 'paneState': const {}, 'panelWindows': 0};
      case 'readSettings':
        return const <String, dynamic>{};
      case 'writeSettings' || 'flushEditors' || 'detach' || 'ensureSurfaces' || 'placePanel':
        return true;
    }
    if (host == null) throw StateError('Open a document first');
    switch (method) {
      case 'attach' || 'open':
        return {'status': _status(host)};
      case 'render':
        if (args['playing'] == true) host.request({'op': 'tick', 'quiet': true});
        return {'status': _status(host), 'frameReady': true};
      case 'request':
        final command = (jsonDecode(args['command'] as String) as Map).cast<String, dynamic>();
        for (final key in const ['deferSnapshot', 'knownSnapshotId', 'knownReferenceId']) {
          if (args[key] != null) command[key] = args[key];
        }
        // as the macOS runner does: the reply as it is (a failure is the status's own `error`)
        return host.request(command);
      case 'easeModel':
        return host.request({...args, 'op': 'easeModel'});
    }
    // pickers, reveal, clipboard, other windows: nothing to do in a story
    return null;
  }
}
