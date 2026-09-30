/// The Surface lint's ratchet: the product window already holds raw styling that predates the grammar, so the rule is not "zero
/// everywhere" but "never more". A file may not exceed what the baseline says it had; a file the baseline does not know must be clean.
/// Fixing a file lowers its line (run with --write-baseline); nothing can raise one.
///
/// Two sections: `path N` is raw styling (a violation); `px path total plain` is the file's `Surface.px(...)` exceptions: `total`
/// all of them, `plain` the ones without a `// surface: <reason>`. Both may only go down; a file the baseline does not know may hold none.
class Baseline {
  Baseline(this.raw, this.px);
  final Map<String, int> raw;
  final Map<String, (int total, int plain)> px;
}

Baseline parseBaseline(String text) {
  final raw = <String, int>{};
  final px = <String, (int, int)>{};
  for (final line in text.split('\n')) {
    final t = line.trim();
    if (t.isEmpty || t.startsWith('#')) continue;
    final parts = t.split(' ');
    if (parts.first == 'px' && parts.length == 4) {
      final total = int.tryParse(parts[2]), plain = int.tryParse(parts[3]);
      if (total != null && plain != null) px[parts[1]] = (total, plain);
      continue;
    }
    final at = t.lastIndexOf(' ');
    if (at < 0) continue;
    final n = int.tryParse(t.substring(at + 1));
    if (n != null) raw[t.substring(0, at)] = n;
  }
  return Baseline(raw, px);
}

String formatBaseline(Map<String, int> raw, Map<String, (int, int)> px) {
  final keys = raw.keys.where((k) => raw[k]! > 0).toList()..sort();
  final pxKeys = px.keys.where((k) => px[k]!.$1 > 0).toList()..sort();
  return '# Raw styling that predates the Surface Grammar, per file (raw_dimension + raw_color + material_import + card_housing + scaling).\n'
      '# A file may not exceed its line; a file not listed must be clean. Lower a line by fixing the file, then\n'
      '#   dart run bin/check.dart ../../lib --write-baseline=baseline.txt      (refuses when any count rises)\n'
      '${[for (final k in keys) '$k ${raw[k]}'].join('\n')}\n'
      '# Surface.px(...) exceptions per file: `px <path> <total> <without a // surface: reason>`. Ratcheted down; a new file holds none.\n'
      '${[for (final k in pxKeys) 'px $k ${px[k]!.$1} ${px[k]!.$2}'].join('\n')}\n';
}

class RatchetResult {
  RatchetResult(this.over, this.fresh, this.better, this.pxOver, this.pxFresh, this.pxBetter);

  /// Files that now hold more than their baseline.
  final Map<String, (int now, int allowed)> over;

  /// Files the baseline does not know that hold any.
  final Map<String, int> fresh;

  /// Files that hold fewer than their baseline (the baseline can come down).
  final Map<String, (int now, int allowed)> better;

  /// The same for the `Surface.px` exceptions: total, and those without a reason.
  final Map<String, String> pxOver;
  final Map<String, int> pxFresh;
  final Map<String, (int now, int allowed)> pxBetter;
  bool get ok => over.isEmpty && fresh.isEmpty && pxOver.isEmpty && pxFresh.isEmpty;
}

RatchetResult ratchet(Map<String, int> now, Baseline baseline, {Map<String, (int, int)> px = const {}}) {
  final over = <String, (int, int)>{}, fresh = <String, int>{}, better = <String, (int, int)>{};
  for (final e in now.entries) {
    if (e.value == 0) continue;
    final allowed = baseline.raw[e.key];
    if (allowed == null) {
      fresh[e.key] = e.value;
    } else if (e.value > allowed) {
      over[e.key] = (e.value, allowed);
    } else if (e.value < allowed) {
      better[e.key] = (e.value, allowed);
    }
  }
  for (final e in baseline.raw.entries) {
    if (!now.containsKey(e.key) || now[e.key] == 0) better[e.key] = (0, e.value);
  }
  final pxOver = <String, String>{}, pxFresh = <String, int>{}, pxBetter = <String, (int, int)>{};
  for (final e in px.entries) {
    if (e.value.$1 == 0) continue;
    final allowed = baseline.px[e.key];
    if (allowed == null) {
      pxFresh[e.key] = e.value.$1;
    } else if (e.value.$1 > allowed.$1) {
      pxOver[e.key] = '${e.value.$1} Surface.px, baseline allows ${allowed.$1}';
    } else if (e.value.$2 > allowed.$2) {
      pxOver[e.key] = '${e.value.$2} Surface.px without a // surface: reason, baseline allows ${allowed.$2}';
    } else if (e.value.$1 < allowed.$1) {
      pxBetter[e.key] = (e.value.$1, allowed.$1);
    }
  }
  for (final e in baseline.px.entries) {
    if ((px[e.key]?.$1 ?? 0) == 0) pxBetter[e.key] = (0, e.value.$1);
  }
  return RatchetResult(over, fresh, better, pxOver, pxFresh, pxBetter);
}

/// What a regenerated baseline would raise above the old one (it may only lower).
List<String> risers(Map<String, int> raw, Map<String, (int, int)> px, Baseline old, {required bool hadPx}) {
  final out = <String>[];
  for (final e in raw.entries) {
    if (e.value > (old.raw[e.key] ?? 0)) out.add('${e.key}: ${e.value} raw styling, baseline ${old.raw[e.key] ?? 0}');
  }
  if (hadPx) {
    for (final e in px.entries) {
      final was = old.px[e.key] ?? (0, 0);
      if (e.value.$1 > was.$1 || e.value.$2 > was.$2) out.add('${e.key}: ${e.value.$1}/${e.value.$2} Surface.px, baseline ${was.$1}/${was.$2}');
    }
  }
  return out;
}
