import 'package:analysis_server_plugin/edit/dart/correction_producer.dart';
import 'package:analysis_server_plugin/edit/dart/dart_fix_kind_priority.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer_plugin/utilities/change_builder/change_builder_core.dart';
import 'package:analyzer_plugin/utilities/fixes/fixes.dart';
import 'package:analyzer_plugin/utilities/range_factory.dart';

import 'canon.dart';
import 'dimension_use.dart';
import 'raw_dimension.dart' show scaleFor;

/// Replaces a raw measurement with the canonical Surface / Dn token of its value and role (the same table as `bin/check.dart --fix`),
/// else with `Surface.px(n)`: a SCALE-policy custom dimension, an exception the lint counts (say why with `// surface: <reason>`).
class UseMetric extends ResolvedCorrectionProducer {
  static const _kind = FixKind(
    'motolii.fix.useMetric',
    DartFixKindPriority.standard,
    'Replace with the canonical Surface token (or Surface.px)',
  );

  UseMetric({required super.context});

  @override
  CorrectionApplicability get applicability =>
      CorrectionApplicability.singleLocation;

  @override
  FixKind get fixKind => _kind;

  @override
  Future<void> compute(ChangeBuilder builder) async {
    final target = node;
    var negative = false;
    Expression? literal = target is Expression ? target : null;
    if (literal is PrefixExpression) {
      negative = true;
      literal = literal.operand;
    }
    final value = switch (literal) {
      IntegerLiteral(:final value?) => value.toDouble(),
      DoubleLiteral(:final value) => value,
      _ => null,
    };
    if (value == null) return;
    final token = negative ? null : canonicalToken(dimensionUse(target), value, face: faceFile(file));
    final text = token ?? '${negative ? '-' : ''}Surface.px(${literal!.toSource()})';
    await builder.addDartFileEdit(file, (b) {
      b.importLibrary(Uri.parse(scaleFor(file).uri));
      b.addSimpleReplacement(range.node(target), text);
    });
  }
}
