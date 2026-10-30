// The tests joined to the inside run in real time (Flutter's LiveTestWidgetsFlutterBinding), as the inside answers in
// real time: a change the window asks of it is done when its notice comes (motolii.dart answered), not when the test's
// own clock moves. The lab's sets, with no inside, keep the usual binding: test/lab/flutter_test_config.dart.
import 'dart:async';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';

import 'package:lab_book/workspace/create_preview.dart' show RecipeMotions;
import 'package:lab_book/workspace/dock_make.dart' show coloursAt;

import 'mac_fonts.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  LiveTestWidgetsFlutterBinding.ensureInitialized();
  await loadMacFonts();   // (letters as the Mac draws them: mac_fonts.dart)
  tearDownAll(RecipeMotions.stop);   // (a file of tests done: the Create shelf's previews not left making documents as the program ends)
  setUp(() => coloursAt(File('${Directory.systemTemp.createTempSync('colours').path}/colors.json')));   // (each from no colours kept, as on a new machine, and the maker's own left alone: a test's state is its own, Meszaros's Fresh Fixture)
  await testMain();
}
