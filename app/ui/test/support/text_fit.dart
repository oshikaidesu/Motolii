import 'dart:async';

import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

/// Text that was given less height than one of its own lines needs: laid out
/// in a box the font does not fit, so its glyphs are cut at the edge. Flutter
/// ships guidelines for target size and contrast but none for this
/// (design-craft-ledger: a field's height is its border, its padding and one
/// line, in that order).
class TextFitsGuideline extends AccessibilityGuideline {
  const TextFitsGuideline();

  @override
  String get description => 'text keeps the height its lines need';

  @override
  FutureOr<Evaluation> evaluate(WidgetTester tester) {
    var result = const Evaluation.pass();
    void visit(RenderObject node) {
      if (node is RenderOffstage && node.offstage) return;
      if (node is RenderParagraph && node.hasSize) {
        final needed = node.getMinIntrinsicHeight(node.size.width);
        if (node.size.height + .5 < needed) {
          result += Evaluation.fail(
            '"${node.text.toPlainText()}" was given ${_px(node.size.height)} '
            'and needs ${_px(needed)}',
          );
        }
      } else if (node is RenderEditable && node.hasSize) {
        final needed = node.preferredLineHeight;
        if (node.size.height + .5 < needed) {
          result += Evaluation.fail(
            'a field was given ${_px(node.size.height)} '
            'and its line needs ${_px(needed)}',
          );
        }
      }
      node.visitChildren(visit);
    }

    for (final view in tester.binding.renderViews) {
      visit(view);
    }
    return result;
  }

  static String _px(double v) => '${v.toStringAsFixed(1)} px';
}
