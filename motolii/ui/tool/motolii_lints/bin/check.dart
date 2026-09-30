/// Command-line twin of the `raw_dimension` plugin rule, for
/// `motolii-ui.sh test` and CI (batch `flutter analyze` does not run plugins).
///
///   dart run bin/check.dart <dir>          list raw measurements, exit 1 if any
///   dart run bin/check.dart <dir> --fix    replace those matching an
///                                          Surface/Step token, then list the rest
import 'dart:io';

import 'package:analyzer/dart/analysis/analysis_context_collection.dart';
import 'package:analyzer/dart/analysis/results.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:motolii_lints/src/card_housing.dart';
import 'package:motolii_lints/src/material_import.dart';
import 'package:motolii_lints/src/ratchet.dart';
import 'package:motolii_lints/src/raw_color.dart';
import 'package:motolii_lints/src/raw_dimension.dart';
import 'package:path/path.dart' as p;

Future<void> main(List<String> args) async {
  final fix = args.contains('--fix');
  String? flag(String name) => args.where((a) => a.startsWith('--$name=')).map((a) => a.substring(name.length + 3)).firstOrNull;
  final ratchetFile = flag('ratchet'), writeBaseline = flag('write-baseline');
  final dirs = args.where((a) => !a.startsWith('--')).toList();
  if (dirs.isEmpty) dirs.add('lib');
  final roots = dirs.map((d) => p.normalize(p.absolute(d))).toList();
  final collection = AnalysisContextCollection(includedPaths: roots);
  var left = 0, fixed = 0, colours = 0, foreign = 0, cards = 0;
  final perFile = <String, int>{};
  final lines = <String, List<String>>{};
  void emit(String path, String line) {
    final unix = path.replaceAll('\\', '/');
    final key = unix.contains('/lib/') ? unix.substring(unix.lastIndexOf('/lib/') + 1) : unix;
    perFile[key] = (perFile[key] ?? 0) + 1;
    (lines[key] ??= []).add(line);
  }
  for (final context in collection.contexts) {
    for (final path in context.contextRoot.analyzedFiles()) {
      final unix = path.replaceAll('\\', '/');
      if (!unix.endsWith('.dart') ||
          unix.contains('/test/') ||
          scaleFiles.any(unix.endsWith)) {
        continue;
      }
      var result = await context.currentSession.getResolvedUnit(path);
      if (result is! ResolvedUnitResult) continue;
      final content = result.content;
      if (excusedFile(content)) continue;
      bool keep(AstNode n) => !excused(content, n.offset);
      var hits = _collect(result.unit).where(keep).toList();
      if (fix && hits.isNotEmpty) {
        final scale = await _scale(context.currentSession.getLibraryByUri, scaleFor(path));
        if (scale.path == null) continue;
        final done = _rewrite(File(path), result, hits, scale.tokens, scale.path!, scaleFor(path).cls);
        if (done > 0) {
          fixed += done;
          context.changeFile(path);
          await context.applyPendingFileChanges();
          result = await context.currentSession.getResolvedUnit(path);
          if (result is! ResolvedUnitResult) continue;
          hits = _collect(result.unit).where(keep).toList();
        }
      }
      final unit = result;
      String at(AstNode hit) {
        final l = unit.lineInfo.getLocation(hit.offset);
        return '${p.relative(path)}:${l.lineNumber}:${l.columnNumber}: ${hit.toSource()}';
      }

      final paletteOk = paletteFiles.any(unix.endsWith);
      for (final hit in hits) {
        emit(path, at(hit));
        left++;
      }
      if (!paletteOk) {
        for (final hit in _colours(result.unit).where(keep)) {
          emit(path, at(hit));
          colours++;
        }
      }
      for (final directive in result.unit.directives) {
        if (directive is! ImportDirective) continue;
        final hit = materialImport(directive);
        if (hit == null) continue;
        emit(path, at(hit));
        foreign++;
      }
      for (final hit in _cards(result.unit).where(keep)) {
        emit(path, '${at(hit)}  [card_housing]');
        cards++;
      }
    }
  }
  if (writeBaseline != null) {
    File(writeBaseline).writeAsStringSync(formatBaseline(perFile));
    stderr.writeln('baseline written: ${perFile.values.fold<int>(0, (a, b) => a + b)} in ${perFile.length} files -> $writeBaseline');
    return;
  }
  if (fixed > 0) stderr.writeln('raw_dimension: replaced $fixed with tokens');
  if (ratchetFile != null) {
    final result = ratchet(perFile, parseBaseline(File(ratchetFile).readAsStringSync()));
    for (final e in result.fresh.entries) {
      stdout.writeln('NEW  ${e.key}: ${e.value} raw styling in a file the baseline does not know (must be 0)');
      lines[e.key]!.forEach(stdout.writeln);
    }
    for (final e in result.over.entries) {
      stdout.writeln('OVER ${e.key}: ${e.value.$1} raw styling, baseline allows ${e.value.$2}');
      lines[e.key]!.forEach(stdout.writeln);
    }
    if (result.better.isNotEmpty) {
      stderr.writeln('surface ratchet: ${result.better.length} files hold less than their baseline; lower it with --write-baseline=$ratchetFile');
    }
    final total = perFile.values.fold<int>(0, (a, b) => a + b);
    stderr.writeln(result.ok
        ? 'surface ratchet: ok ($total raw styling in ${perFile.length} files, none more than its baseline)'
        : 'surface ratchet: FAILED (${result.over.length} files over, ${result.fresh.length} new). Use a Surface/Dn token, or // surface: <why a token cannot say this>');
    exit(result.ok ? 0 : 1);
  }
  for (final key in lines.keys) {
    lines[key]!.forEach(stdout.writeln);
  }
  stderr.writeln(left == 0 ? 'raw_dimension: clean' : 'raw_dimension: $left raw measurements');
  stderr.writeln(colours == 0 ? 'raw_color: clean' : 'raw_color: $colours raw colours');
  stderr.writeln(foreign == 0 ? 'material_import: clean' : 'material_import: $foreign Material/Cupertino imports');
  stderr.writeln(cards == 0 ? 'card_housing: clean' : 'card_housing: $cards cards / rounded outlined boxes');
  exit(left == 0 && colours == 0 && foreign == 0 && cards == 0 ? 0 : 1);
}

List<AstNode> _cards(CompilationUnit unit) {
  final hits = <AstNode>[];
  unit.accept(_CardCollector(hits));
  return hits;
}

class _CardCollector extends RecursiveAstVisitor<void> {
  _CardCollector(this.hits);
  final List<AstNode> hits;

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    final target = cardHousing(node);
    if (target != null) hits.add(target);
    super.visitInstanceCreationExpression(node);
  }
}

List<AstNode> _colours(CompilationUnit unit) {
  final hits = <AstNode>[];
  unit.accept(_ColourCollector(hits));
  return hits;
}

class _ColourCollector extends RecursiveAstVisitor<void> {
  _ColourCollector(this.hits);
  final List<AstNode> hits;

  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    final target = rawColor(node);
    if (target != null) hits.add(target);
    super.visitInstanceCreationExpression(node);
  }
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
  ({String uri, String cls}) which,
) async {
  final result = await library(which.uri);
  final out = <double, String>{};
  if (result is! LibraryElementResult) return (tokens: out, path: null);
  final path = result.element.firstFragment.source.fullName;
  final scale = result.element.getClass(which.cls);
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
  String metricsClass,
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
    if (d.uri.stringValue?.endsWith('metrics.dart') == true) {
      return text;
    }
  }
  final rel = p.relative(metrics, from: p.dirname(path)).replaceAll('\\', '/');
  final line = "import '$rel';\n";
  final last = directives.isEmpty ? null : directives.last;
  if (last == null) return line + text;
  return text.replaceRange(last.end, last.end, '\n$line');
}
