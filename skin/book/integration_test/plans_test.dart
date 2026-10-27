// The inspector plans in a real window, each shot (reports/inspector_plans/<name>.png), B's hold-ring and D's macro used by hand.
//   flutter drive --profile -d macos --driver=test_driver/integration_test.dart --target=integration_test/plans_test.dart
import 'dart:io';
import 'dart:ui' show ImageByteFormat;

import 'package:flutter/rendering.dart' show OffsetLayer;
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:lab_book/motolii.dart' show motoliiDir;
import 'package:lab_book/sets/inspector/plans_set.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized().framePolicy = LiveTestWidgetsFlutterBindingFramePolicy.fullyLive;

  testWidgets('inspector plans, shot', (t) async {
    final out = Directory('$motoliiDir/../reports/inspector_plans')..createSync(recursive: true);
    t.view.physicalSize = const Size(2560, 1640);
    addTearDown(t.view.resetPhysicalSize);
    Future<void> shot(String name) async {
      await t.pump(const Duration(milliseconds: 300));
      final root = t.binding.renderViews.first.layer! as OffsetLayer;
      final png = await (await root.toImage(Offset.zero & t.view.physicalSize)).toByteData(format: ImageByteFormat.png);
      File('${out.path}/$name.png').writeAsBytesSync(png!.buffer.asUint8List());
    }

    final cases = inspectorPlansSet().useCases;
    for (final (i, name) in ['all', 'a', 'b', 'c', 'd'].indexed) {
      await t.pumpWidget(WidgetsApp(key: ValueKey(i), color: const Color(0xFF000000), builder: (c, _) => cases[i].builder(c)));
      await t.pump(const Duration(milliseconds: 500));
      await shot(name);
      if (name == 'b') {
        final g = await t.startGesture(const Offset(300, 400));
        await t.pump(const Duration(milliseconds: 700));
        await g.moveBy(const Offset(0, 60));
        await shot('b_hold_ring');
        await g.up();
        await shot('b_after_ring');
      }
      if (name == 'd') {
        await t.tap(find.text('うねり'));
        await t.drag(find.text('うねり'), const Offset(0, -110));
        await t.drag(find.text('まとまり'), const Offset(0, -90));
        await shot('d_turned');
      }
    }
  });
}
