// Static check for thing descriptors. Exit code 1 on any issue, so CI can gate on it.
//   dart run tool/validate_things.dart [dir] [set ...]
// Default: lib/proto_hf/data/things with the sets builtin and stress.
import 'dart:io';
import '../lib/hf/bp/catalog_io.dart';

void main(List<String> args) {
  final dir = args.isNotEmpty ? args.first : 'lib/proto_hf/data/things';
  final sets = args.length > 1 ? args.sublist(1) : const ['builtin', 'stress'];
  final catalog = loadCatalog(dir, sets: sets);
  final issues = catalog.validate();
  for (final i in issues) {
    stdout.writeln(i);
  }
  stdout.writeln('${catalog.files.length} descriptors, ${catalog.registry.families.length} families, ${issues.length} issues');
  exit(issues.isEmpty ? 0 : 1);
}
