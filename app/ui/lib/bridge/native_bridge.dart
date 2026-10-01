import 'package:flutter/services.dart';

import 'protocol.dart';

class NativeBridge {
  static const channel = MethodChannel('motolii/probe');
  static NativeBridge? _listenerOwner;

  Future<dynamic> invoke(
    String method, [
    Map<String, dynamic> arguments = const {},
  ]) => channel.invokeMethod(method, arguments);

  Future<dynamic> request(
    DocumentOperation operation, [
    Map<String, dynamic> arguments = const {},
    Map<String, dynamic> snapshot = const {},
  ]) =>
      invoke('request', {'command': operation.encode(arguments), ...snapshot});

  void listen(Future<dynamic> Function(MethodCall)? handler) {
    if (handler != null) {
      _listenerOwner = this;
      channel.setMethodCallHandler(handler);
    } else if (identical(_listenerOwner, this)) {
      _listenerOwner = null;
      channel.setMethodCallHandler(null);
    }
  }
}
