// The window's colour theme in Ableton Live's manner (its .ask files): a flat list of roles, each a colour,
//   <Motolii><Theme>
//     <ChosenDefault Value="#b3c66b"/>
//     <Grey07 Value="#131313"/>
//   </Theme></Motolii>
// read from ~/.motolii/theme.xml; a role the file does not say is the window's own (VS Code's manner). The meaning
// roles keep Ableton's names (ChosenDefault, ChosenPlay, ChosenRecord, Alert, SelectionBackground, ControlOnForeground,
// Clip1-16), the key mark Blender's (Keyframe); the grey ramp keeps the window's own step names (Grey00 ... Grey100),
// each step a role in both shades (ground, body, card, well, raised, lines, quiet text, text, strong text).
// A role may also be a number or a word (AviUtl2's style.conf, Obsidian's variables): the skin's shapes and type,
//     <Shade Value="light"/>  <Corner Value="3"/>  <TextScale Value="1.1"/>  <FontSans Value="Space Grotesk"/>
//     <FontSansFile Value="~/.motolii/fonts/SpaceGrotesk.ttf"/>
// a file named for a font is loaded under that name when the theme is read.
import 'dart:async';
import 'dart:io';
import 'dart:typed_data' show ByteData;

import 'package:flutter/services.dart' show FontLoader;
import 'package:flutter/widgets.dart';

abstract final class ColorTheme {
  /// The roles the theme file says (none: the window's own colours). The window is built again when they change.
  static final roles = ValueNotifier<Map<String, Color>>(const {});

  /// The roles that are numbers or words (the skin's shapes and type): every `<Name Value="..."/>` that is not a colour.
  static final words = ValueNotifier<Map<String, String>>(const {});

  /// The theme file the window reads unless told another (none: the home folder's; a test's window sets one of its own, so
  /// what the person who runs it wears does not move the test).
  static String? at;

  /// A role's colour: the theme's, else [usual].
  static Color of(String role, Color usual) => roles.value[role] ?? usual;

  /// A role's number: the theme's, else [usual].
  static double number(String role, double usual) => double.tryParse(words.value[role] ?? '') ?? usual;

  /// A role's word: the theme's, else [usual].
  static String word(String role, String usual) => words.value[role] ?? usual;

  /// The theme file read (not there, or not readable: the window's own colours and shapes).
  static void load([String? path]) {
    try {
      final file = File(path ?? at ?? '${Platform.environment['HOME']}/.motolii/theme.xml');
      folder = file.parent.path;
      final xml = file.readAsStringSync();
      roles.value = parse(xml);
      words.value = {
        for (final m in RegExp(r'<(\w+)\s+Value="([^"#][^"]*)"\s*/>').allMatches(xml)) m.group(1)!: m.group(2)!,
      };
    } catch (_) {
      roles.value = const {};
      words.value = const {};
    }
    _fonts();
    shadeSaid.value = switch (words.value['Shade']) { 'light' => true, 'dark' => false, _ => null };
  }

  /// The folder of the theme file read last: a place a role names is read from here unless it is whole (`/...`) or from home (`~/...`).
  static String folder = '';
  static String place(String w) => w.isEmpty || w.startsWith('/') ? w : (w.startsWith('~/') ? '${Platform.environment['HOME']}${w.substring(1)}' : '$folder/$w');

  /// The shade the theme asks for (null: it says none; the window's own switch decides).
  static final shadeSaid = ValueNotifier<bool?>(null);

  /// Each `Font...File` role: the font file loaded under the family its `Font...` role names.
  static void _fonts() {
    for (final e in words.value.entries.where((e) => e.key.startsWith('Font') && e.key.endsWith('File'))) {
      final family = words.value[e.key.substring(0, e.key.length - 4)];
      final path = place(e.value);
      if (family == null || !File(path).existsSync()) continue;
      FontLoader(family)
        ..addFont(File(path).readAsBytes().then((b) => ByteData.sublistView(b)))
        ..load();
    }
  }

  /// The theme file read now, and again whenever it is written ([changed]: the window built again in place).
  static StreamSubscription<FileSystemEvent>? _watching;
  static void watch(void Function() changed, [String? path]) {
    final file = File(path ?? at ?? '${Platform.environment['HOME']}/.motolii/theme.xml');
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
