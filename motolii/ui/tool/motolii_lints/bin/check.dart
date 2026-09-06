/// Command-line twin of the `raw_dimension` plugin rule, for
/// `motolii-ui.sh test` and CI (batch `flutter analyze` does not run plugins).
///
///   dart run bin/check.dart <dir>          list raw measurements, exit 1 if any
///   dart run bin/check.dart <dir> --fix    replace those matching an
///                                          EditorMetrics token, then list the rest
import 'dart:io';

import 'package:analyzer/dart/analysis/analysis_context_collection.dart';
import 'package:analyzer/dart/analysis/results.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:motolii_lints/src/raw_dimension.dart';
import 'package:motolii_lints/src/use_metric.dart';
import 'package:path/path.dart' as p;

Future<void> main(List<String> args) async {
  final fix = args.contains('--fix');
  final dirs = args.where((a) => !a.startsWith('--')).toList();
  if (dirs.isEmpty) dirs.add('lib');
  final roots = dirs.map((d) => p.normalize(p.absolute(d))).toList();
  final collection = AnalysisContextCollection(includedPaths: roots);
  var left = 0, fixed = 0;
  for (final context in collection.contexts) {
    Map<double, String>? tokens;
    String? metricsPath;
    for (final path in context.contextRoot.analyzedFiles()) {
      final unix = path.replaceAll('\\', '/');
      if (!unix.endsWith('.dart') ||
          unix.contains('/test/') ||
          scaleFiles.any(unix.endsWith)) {
        continue;
      }
      var result = await context.currentSession.getResolvedUnit(path);
      if (result is! ResolvedUnitResult) continue;
      var hits = _collect(result.unit);
      if (fix && hits.isNotEmpty) {
        if (tokens == null) {
          final scale = await _scale(context.currentSession.getLibraryByUri);
          tokens = scale.tokens;
          metricsPath = scale.path;
        }
        if (metricsPath == null) break;
        final done = _rewrite(File(path), result, hits, tokens, metricsPath);
        if (done > 0) {
          fixed += done;
          context.changeFile(path);
          await context.applyPendingFileChanges();
          result = await context.currentSession.getResolvedUnit(path);
          if (result is! ResolvedUnitResult) continue;
          hits = _collect(result.unit);
        }
      }
      for (final hit in hits) {
        final at = result.lineInfo.getLocation(hit.offset);
        stdout.writeln(
          '${p.relative(path)}:${at.lineNumber}:${at.columnNumber}: '
          '${hit.toSource()}',
        );
        left++;
      }
    }
  }
  if (fixed > 0) stderr.writeln('raw_dimension: replaced $fixed with tokens');
  stderr.writeln(
    left == 0 ? 'raw_dimension: clean' : 'raw_dimension: $left raw measurements',
  );
  exit(left == 0 ? 0 : 1);
}

List<AstNode> _collect(CompilationUnit unit) {
  final hits = <AstNode>[];
  unit.accept(_Collector(hits));
  return hits;
}

class _Collector extends RecursiveAstVisitor<void> {
  _Collector(this.hits);
  final List<AstNode> hits;

  void _check(Literal node, num? value) {
    final target = rawMeasurement(node, value);
    if (target != null) hits.add(target);
  }

  @override
  void visitIntegerLiteral(IntegerLiteral node) => _check(node, node.value);

  @override
  void visitDoubleLiteral(DoubleLiteral node) => _check(node, node.value);
}

/// value → first token name with that value, read from the scale class, and
/// where that class lives so the import can point at it.
Future<({Map<double, String> tokens, String? path})> _scale(
  Future<SomeLibraryElementResult> Function(String) library,
) async {
  final result = await library(metricsUri);
  final out = <double, String>{};
  if (result is! LibraryElementResult) return (tokens: out, path: null);
  final path = result.element.firstFragment.source.fullName;
  final scale = result.element.getClass(metricsClass);
  if (scale == null) return (tokens: out, path: path);
  for (final field in scale.fields) {
    if (!field.isStatic || !field.isConst) continue;
    final constant = field.computeConstantValue();
    final px = constant?.toDoubleValue() ?? constant?.toIntValue()?.toDouble();
    final name = field.name;
    if (px != null && name != null) out.putIfAbsent(px, () => name);
  }
  return (tokens: out, path: path);
}

int _rewrite(
  File file,
  ResolvedUnitResult result,
  List<AstNode> hits,
  Map<double, String> tokens,
  String metricsPath,
) {
  var text = result.content;
  var done = 0;
  for (final hit in hits.reversed) {
    var literal = hit;
    var sign = '';
    if (literal is PrefixExpression) {
      sign = '-';
      literal = literal.operand;
    }
    final value = switch (literal) {
      IntegerLiteral(:final value?) => value.toDouble(),
      DoubleLiteral(:final value) => value,
      _ => null,
    };
    final name = value == null ? null : tokens[value];
    if (name == null) continue;
    text = text.replaceRange(hit.offset, hit.end, '$sign$metricsClass.$name');
    done++;
  }
  if (done == 0) return 0;
  text = _ensureImport(text, result, file.path, metricsPath);
  file.writeAsStringSync(text);
  return done;
}

String _ensureImport(
  String text,
  ResolvedUnitResult result,
  String path,
  String metrics,
) {
  final directives = result.unit.directives.whereType<ImportDirective>();
  for (final d in directives) {
    if (d.uri.stringValue?.endsWith('foundation/metrics.dart') == true ||
        d.uri.stringValue == 'metrics.dart') {
      return text;
    }
  }
  final rel = p.relative(metrics, from: p.dirname(path)).replaceAll('\\', '/');
  final line = "import '$rel';\n";
  final last = directives.isEmpty ? null : directives.last;
  if (last == null) return line + text;
  return text.replaceRange(last.end, last.end, '\n$line');
}
