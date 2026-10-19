// What the window keeps for the next time (its zoom, the skin it wears), in ~/.motolii/window.xml in the theme's form:
//   <Motolii><Window><Zoom Value="1.17"/><Skin Value="labs"/></Window></Motolii>
import 'dart:io';

abstract final class WindowFile {
  static File get _file => File('${Platform.environment['HOME']}/.motolii/window.xml');

  static Map<String, String> _all() {
    try {
      return {for (final m in RegExp(r'<(\w+)\s+Value="([^"]*)"\s*/>').allMatches(_file.readAsStringSync())) m.group(1)!: m.group(2)!};
    } catch (_) {
      return {};
    }
  }

  /// A value kept (none: not kept).
  static String? get(String name) => _all()[name];

  /// A value kept, the others as they were (null: no longer kept).
  static void set(String name, String? value) {
    final all = _all();
    value == null ? all.remove(name) : all[name] = value;
    try {
      _file.parent.createSync(recursive: true);
      _file.writeAsStringSync('<?xml version="1.0" encoding="UTF-8"?>\n<Motolii>\n\t<Window>\n${[for (final e in all.entries) '\t\t<${e.key} Value="${e.value}"/>'].join('\n')}\n\t</Window>\n</Motolii>\n');
    } catch (_) {}
  }
}
