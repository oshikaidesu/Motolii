// The tests run in real time (Flutter's LiveTestWidgetsFlutterBinding), as the inside answers in real time: a change
// the window asks of it is done when its notice comes (motolii.dart answered), not when the test's own clock moves.
import 'dart:async';
import 'package:flutter_test/flutter_test.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  LiveTestWidgetsFlutterBinding.ensureInitialized();
  await testMain();
}
