import 'package:analysis_server_plugin/edit/dart/correction_producer.dart';
import 'package:analysis_server_plugin/edit/dart/dart_fix_kind_priority.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer_plugin/utilities/change_builder/change_builder_core.dart';
import 'package:analyzer_plugin/utilities/fixes/fixes.dart';
import 'package:analyzer_plugin/utilities/range_factory.dart';

/// Where the scale lives. The fix reads the class, so a new token needs no
/// change here.
const metricsUri = 'package:motolii_stage5/foundation/metrics.dart';
const metricsClass = 'EditorMetrics';

/// Replaces a raw measurement with the token of exactly the same value.
class UseMetric extends ResolvedCorrectionProducer {
  static const _kind = FixKind(
    'motolii.fix.useMetric',
    DartFixKindPriority.standard,
    'Replace with the matching EditorMetrics token',
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
    final scale = await sessionHelper.getClass(metricsUri, metricsClass);
    if (scale == null) return;
    for (final field in scale.fields) {
      if (!field.isStatic || !field.isConst) continue;
      final constant = field.computeConstantValue();
      final px = constant?.toDoubleValue() ?? constant?.toIntValue()?.toDouble();
      final name = field.name;
      if (px != value || name == null) continue;
      await builder.addDartFileEdit(file, (b) {
        b.importLibrary(Uri.parse(metricsUri));
        b.addSimpleReplacement(
          range.node(target),
          '${negative ? '-' : ''}$metricsClass.$name',
        );
      });
      return;
    }
  }
}
