/// The Surface lint's ratchet: the product window already holds raw styling that predates the grammar, so the rule is not "zero
/// everywhere" but "never more". A file may not exceed what the baseline says it had; a file the baseline does not know must be clean.
/// Fixing a file lowers its line (run with --write-baseline); nothing can raise one.
Map<String, int> parseBaseline(String text) {
  final out = <String, int>{};
  for (final line in text.split('\n')) {
    final t = line.trim();
    if (t.isEmpty || t.startsWith('#')) continue;
    final at = t.lastIndexOf(' ');
    if (at < 0) continue;
    final n = int.tryParse(t.substring(at + 1));
    if (n != null) out[t.substring(0, at)] = n;
  }
  return out;
}

String formatBaseline(Map<String, int> counts) {
  final keys = counts.keys.where((k) => counts[k]! > 0).toList()..sort();
  return '# Raw styling that predates the Surface Grammar, per file (raw_dimension + raw_color + material_import + card_housing).\n'
      '# A file may not exceed its line; a file not listed must be clean. Lower a line by fixing the file, then\n'
      '#   dart run bin/check.dart ../../lib --write-baseline=baseline.txt\n'
      '${[for (final k in keys) '$k ${counts[k]}'].join('\n')}\n';
}

class RatchetResult {
  RatchetResult(this.over, this.fresh, this.better);

  /// Files that now hold more than their baseline.
  final Map<String, (int now, int allowed)> over;

  /// Files the baseline does not know that hold any.
  final Map<String, int> fresh;

  /// Files that hold fewer than their baseline (the baseline can come down).
  final Map<String, (int now, int allowed)> better;
  bool get ok => over.isEmpty && fresh.isEmpty;
}

RatchetResult ratchet(Map<String, int> now, Map<String, int> baseline) {
  final over = <String, (int, int)>{}, fresh = <String, int>{}, better = <String, (int, int)>{};
  for (final e in now.entries) {
    if (e.value == 0) continue;
    final allowed = baseline[e.key];
    if (allowed == null) {
      fresh[e.key] = e.value;
    } else if (e.value > allowed) {
      over[e.key] = (e.value, allowed);
    } else if (e.value < allowed) {
      better[e.key] = (e.value, allowed);
    }
  }
  for (final e in baseline.entries) {
    if (!now.containsKey(e.key) || now[e.key] == 0) better[e.key] = (0, e.value);
  }
  return RatchetResult(over, fresh, better);
}
