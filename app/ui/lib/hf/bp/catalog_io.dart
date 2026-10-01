// File loading for the descriptor catalog. Kept apart from things.dart so that model stays pure.
import 'dart:io';
import 'things.dart';

/// Reads `registry.json` and every `*.json` under each named set directory (for example builtin, stress).
Catalog loadCatalog(String dir, {List<String> sets = const ['builtin']}) {
  final files = <String, String>{};
  for (final s in sets) {
    final d = Directory('$dir/$s');
    if (!d.existsSync()) continue;
    for (final f in d.listSync().whereType<File>().where((f) => f.path.endsWith('.json'))) {
      files['$s/${f.uri.pathSegments.last}'] = f.readAsStringSync();
    }
  }
  return Catalog.parse(File('$dir/registry.json').readAsStringSync(), files);
}
