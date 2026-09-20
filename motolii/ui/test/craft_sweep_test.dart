import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/app/editor_app.dart';
import '../lib/panels/registry.dart';
import '../lib/session/editor_session.dart';
import 'support/editor_test_theme.dart';
import 'support/text_fit.dart';
import 'support/window_fixture.dart';

/// Every panel, at the pixels the default dock gives it, held against the two
/// guidelines Flutter ships (docs/reviews/2026-09-19-design-craft-ledger.md
/// 4-4 and 3-1): a target of at least 24 px, and text at 4.5:1.
///
/// All three are held at none. Every panel is judged before any is reported,
/// so one run names them all.
void main() {
  const tapTarget24 = MinimumTapTargetGuideline(
    size: Size(24, 24),
    link:
        'https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html',
  );
  final guidelines = <String, AccessibilityGuideline>{
    'target 24': tapTarget24,
    'contrast 4.5': textContrastGuideline,
    'text fits': const TextFitsGuideline(),
  };

  testWidgets('every panel meets the tap-target and contrast guidelines', (
    tester,
  ) async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
          EditorSession.channel,
          (call) async => <String, dynamic>{},
        );
    tester.view.physicalSize = const Size(1280, 796);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final semantics = tester.ensureSemantics();
    final found = <String>[];
    for (final entry in dockPanels.entries) {
      final c = EditorSession();
      addTearDown(c.dispose);
      c.document.value = windowStatus(3, 0);
      await tester.pumpWidget(
        MaterialApp(
          theme: editorTestTheme,
          builder: EditorApp.noHover,
          home: Scaffold(
            body: Align(
              alignment: Alignment.topLeft,
              child: SizedBox.fromSize(
                size: entry.value,
                child: buildPanel(entry.key, c, const ValueKey('panel')),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      for (final rule in guidelines.entries) {
        final result = await rule.value.evaluate(tester);
        if (!result.passed) {
          found.add('${entry.key} · ${rule.key}\n${_summary(result.reason)}');
        }
      }
      await tester.pumpWidget(const SizedBox());
    }
    // Released here: the harness checks for live handles before teardown runs.
    semantics.dispose();
    debugPrint('CRAFT SWEEP: ${found.length} findings\n${found.join('\n\n')}');
    expect(found, isEmpty);
  });
}

/// A reason is one paragraph per failing node; what a reader needs is how many
/// there are and which sizes recur.
String _summary(String? reason) {
  final nodes = reason
      ?.split('SemanticsNode#')
      .where((v) => v.trim().isNotEmpty)
      .toList();
  if (nodes == null || nodes.isEmpty) return '$reason';
  final sizes = <String, int>{};
  final size = RegExp(r'found Size\(([\d.]+), ([\d.]+)\)');
  for (final node in nodes) {
    final m = size.firstMatch(node);
    if (m == null) continue;
    final key = '${double.parse(m[1]!).round()}x${double.parse(m[2]!).round()}';
    sizes[key] = (sizes[key] ?? 0) + 1;
  }
  final top = sizes.entries.toList()..sort((a, b) => b.value - a.value);
  return '${nodes.length} nodes  '
      '${top.take(6).map((e) => '${e.key} x${e.value}').join('  ')}'
      '${sizes.isEmpty ? nodes.first.split('\n').take(2).join(' ') : ''}';
}
