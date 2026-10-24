// The lab's own sets, with no inside: Flutter's usual test binding, its own clock (the nearest flutter_test_config.dart
// is the one a test runs under: test/flutter_test_config.dart's real time is for the tests joined to the inside).
import 'dart:async';

Future<void> testExecutable(FutureOr<void> Function() testMain) async => testMain();
