import 'package:flutter/services.dart';

import 'protocol.dart';

class NativeBridge {
  static const channel = MethodChannel('motolii/probe');

  Future<dynamic> invoke(
    String method, [
    Map<String, dynamic> arguments = const {},
  ]) => channel.invokeMethod(method, arguments);

  Future<dynamic> request(
    DocumentOperation operation, [
    Map<String, dynamic> arguments = const {},
  ]) => invoke('request', {'command': operation.encode(arguments)});

  void listen(Future<dynamic> Function(MethodCall)? handler) =>
      channel.setMethodCallHandler(handler);
}
