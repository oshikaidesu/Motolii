// The window's colour theme in Ableton Live's manner (its .ask files): a flat list of roles, each a colour,
//   <Motolii><Theme>
//     <ChosenDefault Value="#b3c66b"/>
//     <Grey07 Value="#131313"/>
//   </Theme></Motolii>
// read from ~/.motolii/theme.xml; a role the file does not say is the window's own (VS Code's manner). The meaning
// roles keep Ableton's names (ChosenDefault, ChosenPlay, ChosenRecord, Alert, SelectionBackground, ControlOnForeground,
// Clip1-16), the key mark Blender's (Keyframe); the grey ramp keeps the window's own step names (Grey00 ... Grey100),
// each step a role in both shades (ground, body, card, well, raised, lines, quiet text, text, strong text).
import 'dart:async';
import 'dart:io';

import 'package:flutter/widgets.dart';

abstract final class ColorTheme {
  /// The roles the theme file says (none: the window's own colours). The window is built again when they change.
  static final roles = ValueNotifier<Map<String, Color>>(const {});

  /// A role's colour: the theme's, else [usual].
  static Color of(String role, Color usual) => roles.value[role] ?? usual;

  /// The theme file read (not there, or not readable: the window's own colours).
  static void load([String? path]) {
    try {
      roles.value = parse(File(path ?? '${Platform.environment['HOME']}/.motolii/theme.xml').readAsStringSync());
    } catch (_) {
      roles.value = const {};
    }
  }

  /// The theme file read now, and again whenever it is written ([changed]: the window built again in place).
  static StreamSubscription<FileSystemEvent>? _watching;
  static void watch(void Function() changed, [String? path]) {
    final file = File(path ?? '${Platform.environment['HOME']}/.motolii/theme.xml');
    load(file.path);
    _watching?.cancel();
    try {
      file.parent.createSync(recursive: true);
      _watching = file.parent.watch().where((e) => e.path == file.path).listen((_) {
        load(file.path);
        changed();
      });
    } catch (_) {}   // (a folder that cannot be watched: read once)
  }

  /// The roles of a theme file: every `<Name Value="#rrggbb"/>` or `"#rrggbbaa"`.
  static Map<String, Color> parse(String xml) => {
    for (final m in RegExp(r'<(\w+)\s+Value="#([0-9a-fA-F]{6})([0-9a-fA-F]{2})?"\s*/>').allMatches(xml))
      m.group(1)!: Color(int.parse('${m.group(3) ?? 'ff'}${m.group(2)}', radix: 16)),
  };

  /// A theme file of these roles (the form at the top of this file), as the window writes its own (a maker copies one and changes the colours).
  static String write(Map<String, Color> all) {
    String hex(Color c) => [c.r, c.g, c.b, c.a].map((v) => (v * 255).round().toRadixString(16).padLeft(2, '0')).join();
    return '<?xml version="1.0" encoding="UTF-8"?>\n<Motolii>\n\t<Theme>\n${[for (final e in all.entries) '\t\t<${e.key} Value="#${hex(e.value)}"/>'].join('\n')}\n\t</Theme>\n</Motolii>\n';
  }
}
