/// Command-line twin of the lint rules, for `motolii-ui.sh test` and CI (batch `flutter analyze` does not run plugins): "Motolii UI Grammar".
///
///   dart run bin/check.dart <dir>                       the report; exit 1 if any violation
///   dart run bin/check.dart <dir> --ratchet=baseline    the same, but only what rises above the baseline fails
///   dart run bin/check.dart <dir> --write-baseline=F    regenerate the baseline (refuses when any count would rise)
///   dart run bin/check.dart <dir> --fix [--steps-only]  the codemod: raw dimension -> canonical Surface/Dn token by value and role,
///                                                       else Surface.px(n); a painter's own pixels get `// surface-block:`
import 'dart:io';

import 'package:analyzer/dart/analysis/analysis_context_collection.dart';
import 'package:analyzer/dart/analysis/results.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:motolii_lints/src/canon.dart';
import 'package:motolii_lints/src/card_housing.dart';
import 'package:motolii_lints/src/dimension_use.dart';
import 'package:motolii_lints/src/material_import.dart';
import 'package:motolii_lints/src/ratchet.dart';
import 'package:motolii_lints/src/raw_color.dart';
import 'package:motolii_lints/src/raw_dimension.dart';
import 'package:motolii_lints/src/surface_scale.dart';
import 'package:path/path.dart' as p;

/// One finding: where, which area of the report, and what rule says so.
class Finding {
  Finding(this.path, this.line, this.area, this.what);
  final String path;
  final int line;
  final String area;
  final String what;
}

const _areas = ['typography', 'colours', 'spacing', 'geometry', 'radius', 'scaling'];

const _hint = {
  'typography': 'use canonical Surface type family: Dn.nameSize / labelSize / microSize / numericSize (Dn.name(), Dn.label(), ...)',
  'colours': 'use canonical Surface colour family: Surface.base / raised / ink / muted, N.gNN, or H (identity.dart)',
  'spacing': 'use canonical Surface spacing family: Surface.labelGap / inlineGap / sectionGap / panelInset',
  'geometry': 'use canonical Surface density family: Surface.workRow / control / chromeRow / hit / glyph, or Surface.px(n) with `// surface: <reason>`',
  'radius': 'use canonical Surface shape family: Surface.controlRadius / faceRadius',
  'scaling': 'use the UI Scale (lib/app/ui_scale.dart): tokens are derived values, a subtree is never transformed',
  'housing': 'no Material/Cupertino import; no card: a line or a level change separates sections',
};

class FileReport {
  final findings = <Finding>[];
  var pxTotal = 0, pxPlain = 0, tokens = 0, fixedExcused = 0;
  int get raw => findings.length;
}

Future<void> main(List<String> args) async {
  final fix = args.contains('--fix'), stepsOnly = args.contains('--steps-only');
  String? flag(String name) => args.where((a) => a.startsWith('--$name=')).map((a) => a.substring(name.length + 3)).firstOrNull;
  final ratchetFile = flag('ratchet'), writeBaseline = flag('write-baseline');
  final dirs = args.where((a) => !a.startsWith('--')).toList();
  if (dirs.isEmpty) dirs.add('lib');
  final roots = dirs.map((d) => p.normalize(p.absolute(d))).toList();
  final collection = AnalysisContextCollection(includedPaths: roots);
  final reports = <String, FileReport>{};
  var tokenised = 0, pxAll = 0, rawDims = 0, excusedDims = 0;
  final migrated = <String, (int tokens, int px, int blocks)>{};

  for (final context in collection.contexts) {
    for (final path in context.contextRoot.analyzedFiles()) {
      final unix = path.replaceAll('\\', '/');
      if (!unix.endsWith('.dart') || unix.contains('/test/')) continue;
      final key = unix.contains('/lib/') ? unix.substring(unix.lastIndexOf('/lib/') + 1) : unix;
      var result = await context.currentSession.getResolvedUnit(path);
      if (result is! ResolvedUnitResult) continue;
      final content = result.content;
      if (excusedFile(content)) continue;
      final isCanon = unix.endsWith('lib/theme/metrics.dart');
      if (fix && !isCanon) {
        final done = _fix(File(path), result, scaleFiles.any(unix.endsWith) || stepsOnly);
        if (done != null) {
          migrated[key] = done;
          context.changeFile(path);
          await context.applyPendingFileChanges();
          result = await context.currentSession.getResolvedUnit(path);
          if (result is! ResolvedUnitResult) continue;
        }
      }
      if (isCanon) continue;
      final unit = result;
      final report = reports.putIfAbsent(key, FileReport.new);
      void add(AstNode node, String area, String what) {
        final l = unit.lineInfo.getLocation(node.offset);
        report.findings.add(Finding(key, l.lineNumber, area, '$what: ${_short(node)}'));
      }

      final canonFile = scaleFiles.any(unix.endsWith);
      if (!canonFile) {
        for (final hit in _collect(unit.unit)) {
          if (excusedNode(unit.content, hit)) {
            report.fixedExcused++;
            excusedDims++;
            continue;
          }
          final use = dimensionUse(hit);
          add(hit, use.area, 'raw ${use.kind.name == 'other' ? 'dimension' : use.kind.name} ${use.name.isEmpty ? '' : '(${use.name}) '}in a measurement');
          rawDims++;
        }
        if (!paletteFiles.any(unix.endsWith)) {
          for (final hit in _colours(unit.unit).where((n) => !excused(unit.content, n.offset))) {
            add(hit, 'colours', 'raw colour');
          }
        }
        for (final directive in unit.unit.directives) {
          if (directive is! ImportDirective) continue;
          final hit = materialImport(directive);
          if (hit != null) add(hit, 'housing', 'Material/Cupertino import');
        }
        for (final hit in _cards(unit.unit).where((n) => !excused(unit.content, n.offset))) {
          add(hit, 'housing', 'card / rounded outlined box [card_housing]');
        }
        if (!unix.endsWith(scaleMechanismFile)) {
          for (final hit in scalingViolations(unit.unit).where((n) => !excused(unit.content, n.offset))) {
            add(hit, 'scaling', 'arbitrary visual scaling');
          }
        }
      }
      final counter = _Counter();
      unit.unit.accept(counter);
      report.tokens = counter.tokens;
      tokenised += counter.tokens;
      for (final call in counter.px) {
        report.pxTotal++;
        pxAll++;
        if (!excused(unit.content, call.offset)) report.pxPlain++;
      }
    }
  }

  if (fix) {
    var t = 0, px = 0, b = 0;
    for (final e in migrated.entries) {
      t += e.value.$1;
      px += e.value.$2;
      b += e.value.$3;
    }
    stderr.writeln('codemod: $t literals -> canonical token, $px -> Surface.px, $b painter/canvas scopes marked `// surface-block:` in ${migrated.length} files');
    return;
  }

  final raw = {for (final e in reports.entries) e.key: e.value.raw};
  final px = {for (final e in reports.entries) e.key: (e.value.pxTotal, e.value.pxPlain)};

  if (writeBaseline != null) {
    final file = File(writeBaseline);
    if (file.existsSync()) {
      final old = parseBaseline(file.readAsStringSync());
      final up = risers(raw, px, old, hadPx: old.px.isNotEmpty || file.readAsStringSync().contains('Surface.px'));
      if (up.isNotEmpty) {
        stderr.writeln('baseline NOT written: a count would rise (a baseline may only go down):');
        up.forEach(stderr.writeln);
        exit(1);
      }
    }
    file.writeAsStringSync(formatBaseline(raw, px));
    stderr.writeln('baseline written: ${raw.values.fold<int>(0, (a, b) => a + b)} raw in ${raw.values.where((v) => v > 0).length} files, '
        '${px.values.fold<int>(0, (a, b) => a + b.$1)} Surface.px in ${px.values.where((v) => v.$1 > 0).length} files -> $writeBaseline');
    return;
  }

  final baseline = ratchetFile == null ? null : parseBaseline(File(ratchetFile).readAsStringSync());
  final result = baseline == null ? null : ratchet(raw, baseline, px: px);
  // what is shown: everything (plain report) or what breaks the ratchet
  final shown = <String>{
    if (result == null) ...reports.keys.where((k) => reports[k]!.raw > 0) else ...[...result.fresh.keys, ...result.over.keys],
  };
  final all = [for (final k in shown) ...reports[k]!.findings];
  final counts = {for (final a in [..._areas, 'housing']) a: all.where((f) => f.area == a).length};
  final allCounts = {for (final a in [..._areas, 'housing']) a: [for (final r in reports.values) ...r.findings].where((f) => f.area == a).length};

  stdout.writeln('Motolii UI Grammar');
  for (final a in [..._areas, 'housing']) {
    final shownHere = counts[a]!, total = allCounts[a]!;
    final mark = shownHere == 0 ? '✓' : '✗';
    final note = result == null ? '$total' : '$shownHere above baseline ($total held)';
    stdout.writeln('  $mark ${a.padRight(11)} $note');
  }
  stdout.writeln('');
  for (final f in all..sort((a, b) => a.path == b.path ? a.line.compareTo(b.line) : a.path.compareTo(b.path))) {
    stdout.writeln('✗ ${f.path}:${f.line}');
    stdout.writeln('    ${f.what}');
    stdout.writeln('    ${_hint[f.area]}');
  }
  if (result != null) {
    for (final e in result.pxOver.entries) {
      stdout.writeln('✗ ${e.key}');
      stdout.writeln('    Surface.px exceptions above the baseline: ${e.value}');
      stdout.writeln('    a new exception says why: `// surface: <reason of at least 8 characters>`, or names a canonical token');
    }
    for (final e in result.pxFresh.entries) {
      stdout.writeln('✗ ${e.key}');
      stdout.writeln('    ${e.value} Surface.px in a file the baseline does not know (a new file holds none)');
      stdout.writeln('    use a canonical Surface token; a custom dimension needs an existing file and `// surface: <reason>`');
    }
  }
  final rawTotal = raw.values.fold<int>(0, (a, b) => a + b);
  final pxPlain = px.values.fold<int>(0, (a, b) => a + b.$2);
  final all100 = tokenised + pxAll + rawDims;
  final coverage = all100 == 0 ? 100.0 : 100.0 * tokenised / all100;
  stdout.writeln('');
  stdout.writeln('Totals: $rawTotal violations held in ${raw.values.where((v) => v > 0).length} files'
      '${result == null ? '' : ', ${all.length} above the baseline'}');
  stdout.writeln('Exceptions: $pxAll Surface.px in ${px.values.where((v) => v.$1 > 0).length} files ($pxPlain without a reason), $excusedDims painter/fixed geometry excused');
  stdout.writeln('Canonical coverage: ${coverage.toStringAsFixed(1)} % ($tokenised tokens of $all100 UI dimension occurrences; painters and fixed excluded)');

  if (result != null) {
    if (result.better.isNotEmpty || result.pxBetter.isNotEmpty) {
      stderr.writeln('surface ratchet: ${result.better.length + result.pxBetter.length} files hold less than their baseline; lower it with --write-baseline=$ratchetFile');
    }
    stderr.writeln(result.ok
        ? 'surface ratchet: ok'
        : 'surface ratchet: FAILED (${result.over.length + result.pxOver.length} files over, ${result.fresh.length + result.pxFresh.length} new)');
    exit(result.ok ? 0 : 1);
  }
  exit(rawTotal == 0 ? 0 : 1);
}

String _short(AstNode n) {
  final s = n.toSource().replaceAll(RegExp(r'\s+'), ' ');
  return s.length > 60 ? '${s.substring(0, 57)}...' : s;
}

/// Token references and `Surface.px` calls of a unit (the coverage).
class _Counter extends RecursiveAstVisitor<void> {
  int tokens = 0;
  final px = <MethodInvocation>[];

  @override
  void visitPrefixedIdentifier(PrefixedIdentifier node) {
    if (isDimensionToken(node)) tokens++;
    super.visitPrefixedIdentifier(node);
  }

  @override
  void visitMethodInvocation(MethodInvocation node) {
    if (isPxException(node)) px.add(node);
    super.visitMethodInvocation(node);
  }
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

/// The retired `Step.sN` rungs: a reference is the value it named.
class _StepCollector extends RecursiveAstVisitor<void> {
  final hits = <PrefixedIdentifier>[];
  @override
  void visitPrefixedIdentifier(PrefixedIdentifier node) {
    if (node.prefix.name == 'Step' && stepValue(node.identifier.name) != null) hits.add(node);
    super.visitPrefixedIdentifier(node);
  }
}

/// The codemod over one file: every raw dimension (and `Step.sN`) at its exact position becomes the canonical token of its value and
/// role, else `Surface.px(value)`; a literal inside a painter or a canvas function is left as it is and the scope says so once.
/// Returns (tokens, px, blocks) or null when nothing changed.
(int, int, int)? _fix(File file, ResolvedUnitResult result, bool canonFile) {
  final edits = <(int, int, String)>[];
  final blocks = <int, String>{};
  var tokens = 0, px = 0;
  final face = faceFile(file.path);

  void replace(AstNode node, double value, {required bool step}) {
    final use = dimensionUse(node);
    final scope = step ? null : painterScope(node);
    if (scope != null) {
      final name = scope is ClassDeclaration ? 'class ${scope.namePart.typeName.lexeme}' : (scope is MethodDeclaration ? 'method ${scope.name.lexeme}' : 'function');
      blocks[scope.beginToken.offset] = "// surface-block: painter geometry: drawn in the canvas's own pixels ($name)";
      return;
    }
    final negative = node is PrefixExpression;
    final source = step ? _num(value) : (negative ? node.operand.toSource() : node.toSource());
    final token = negative ? null : canonicalToken(use, value, face: face);
    if (token != null) {
      tokens++;
      edits.add((node.offset, node.end, token));
    } else {
      px++;
      edits.add((node.offset, node.end, '${negative ? '-' : ''}Surface.px($source)'));
    }
  }

  if (!canonFile) {
    for (final hit in _collect(result.unit)) {
      if (excusedNode(result.content, hit)) continue;
      var literal = hit;
      if (literal is PrefixExpression) literal = literal.operand;
      final value = switch (literal) {
        IntegerLiteral(:final value?) => value.toDouble(),
        DoubleLiteral(:final value) => value,
        _ => null,
      };
      if (value == null) continue;
      replace(hit, value, step: false);
    }
  }
  final steps = _StepCollector();
  result.unit.accept(steps);
  for (final s in steps.hits) {
    replace(s, stepValue(s.identifier.name)!, step: true);
  }
  if (edits.isEmpty && blocks.isEmpty) return null;

  var text = result.content;
  final all = <(int, int, String)>[
    ...edits,
    for (final e in blocks.entries) (_lineStart(text, e.key), _lineStart(text, e.key), '${_indentAt(text, e.key)}${e.value}\n'),
  ]..sort((a, b) => b.$1.compareTo(a.$1));
  for (final e in all) {
    text = text.replaceRange(e.$1, e.$2, e.$3);
  }
  if (tokens + px > 0) text = _ensureImport(text, result, file.path);
  file.writeAsStringSync(text);
  return (tokens, px, blocks.length);
}

String _num(double v) => v == v.roundToDouble() ? v.toStringAsFixed(0) : '$v';

int _lineStart(String text, int offset) => text.lastIndexOf('\n', offset == 0 ? 0 : offset - 1) + 1;

String _indentAt(String text, int offset) {
  final start = _lineStart(text, offset);
  return text.substring(start, offset).replaceAll(RegExp(r'\S'), ' ');
}

String _ensureImport(String text, ResolvedUnitResult result, String path) {
  if (result.unit.directives.any((d) => d is PartOfDirective)) return text; // the library that owns the part imports it
  final directives = result.unit.directives.whereType<ImportDirective>();
  for (final d in directives) {
    if (d.uri.stringValue?.endsWith('metrics.dart') != true) continue;
    final shown = d.combinators.whereType<ShowCombinator>().expand((c) => c.shownNames).map((n) => n.name);
    if (shown.isEmpty || shown.contains('Surface')) return text;
    final at = d.combinators.whereType<ShowCombinator>().last.end; // `show Dn;` -> `show Dn, Surface;`
    return text.replaceRange(at, at, ', Surface');
  }
  final abs = p.absolute(path);
  final rel = p.relative(p.join(abs.split('/lib/').first, 'lib', 'theme', 'metrics.dart'), from: p.dirname(abs)).replaceAll('\\', '/');
  final line = "import '$rel';";
  final last = directives.isEmpty ? null : directives.last;
  if (last == null) return '$line\n$text';
  return text.replaceRange(last.end, last.end, '\n$line');
}
