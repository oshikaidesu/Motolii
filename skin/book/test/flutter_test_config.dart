// The tests joined to the inside run in real time (Flutter's LiveTestWidgetsFlutterBinding), as the inside answers in
// real time: a change the window asks of it is done when its notice comes (motolii.dart answered), not when the test's
// own clock moves. The lab's sets, with no inside, keep the usual binding: test/lab/flutter_test_config.dart.
import 'dart:async';
import 'package:flutter_test/flutter_test.dart';

import 'mac_fonts.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  LiveTestWidgetsFlutterBinding.ensureInitialized();
  await loadMacFonts();   // (letters as the Mac draws them: mac_fonts.dart)
  await testMain();
}
