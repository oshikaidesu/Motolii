import 'dart:ui';

import 'package:flutter/widgets.dart';

import 'session/editor_session.dart';

import 'app/editor_app.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  void report(Object error, StackTrace? stack) => EditorSession.reportError(error, stack);

  final previous = FlutterError.onError;
  FlutterError.onError = (details) {
    previous?.call(details);
    report(details.exception, details.stack);
  };
  final previousAsync = PlatformDispatcher.instance.onError;
  PlatformDispatcher.instance.onError = (error, stack) {
    report(error, stack);
    return previousAsync?.call(error, stack) ?? false;
  };
  runApp(const EditorApp());
}
