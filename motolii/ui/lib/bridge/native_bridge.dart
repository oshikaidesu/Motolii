import 'dart:convert';

import 'package:flutter/services.dart';

import 'protocol.dart';

class NativeBridge {
  static const channel = MethodChannel('motolii/probe');
  static NativeBridge? _listenerOwner;

  /// The host hands a status over as the JSON bytes Rust wrote (`statusJson` in an envelope, or the reply itself when it is
  /// a status): it never rebuilds them as a tree. They are decoded here, once, into the same map a tree would have been.
  static dynamic unpack(dynamic reply) {
    if (reply is Uint8List) return json.fuse(utf8).decode(reply);
    if (reply is Map && reply['statusJson'] is Uint8List) {
      return {
        for (final e in reply.entries)
          if (e.key != 'statusJson') e.key: e.value,
        'status': json.fuse(utf8).decode(reply['statusJson'] as Uint8List),
      };
    }
    return reply;
  }

  Future<dynamic> invoke(
    String method, [
    Map<String, dynamic> arguments = const {},
  ]) async => unpack(await channel.invokeMethod(method, arguments));

  Future<dynamic> request(
    DocumentOperation operation, [
    Map<String, dynamic> arguments = const {},
    Map<String, dynamic> snapshot = const {},
  ]) =>
      invoke('request', {'command': operation.encode(arguments), ...snapshot});

  void listen(Future<dynamic> Function(MethodCall)? handler) {
    if (handler != null) {
      _listenerOwner = this;
      channel.setMethodCallHandler(
        (call) => handler(MethodCall(call.method, unpack(call.arguments))),
      );
    } else if (identical(_listenerOwner, this)) {
      _listenerOwner = null;
      channel.setMethodCallHandler(null);
    }
  }
}
