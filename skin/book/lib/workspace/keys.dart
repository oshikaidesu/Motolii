// The window's commands by name and the keys that say them, in VS Code's manner (keybindings.json): the window's own
// table, then the maker's file over it (~/.motolii/keybindings.json, a list of {"key": "cmd+shift+z", "command":
// "edit.redo"}; a command begun with '-' takes that key from the command). Flutter's own Shortcuts and Actions carry
// them; the menus show the same keys. While words are being typed, a key with no cmd, ctrl or alt is the words'.
import 'dart:convert';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

/// The window's own keys (After Effects' where it has one: Space plays, Home and End go to the ends).
const defaultKeys = <(String, String)>[
  ('cmd+n', 'file.new'),
  ('cmd+r', 'file.render'),
  ('cmd+z', 'edit.undo'),
  ('cmd+shift+z', 'edit.redo'),
  ('cmd+x', 'edit.cut'),
  ('cmd+c', 'edit.copy'),
  ('cmd+v', 'edit.paste'),
  ('cmd+a', 'edit.selectAll'),
  ('backspace', 'thing.remove'),
  ('delete', 'thing.remove'),
  ('space', 'play.toggle'),
  ('left', 'frame.previous'),
  ('right', 'frame.next'),
  ('shift+left', 'frame.back10'),
  ('shift+right', 'frame.forward10'),
  ('home', 'frame.first'),
  ('end', 'frame.last'),
];

/// The keys in use: the window's own, the maker's file over them.
class Keymap {
  Keymap(this.pairs);
  final List<(SingleActivator, String)> pairs;

  static Keymap load([String? file]) {
    final all = [for (final (k, c) in defaultKeys) if (activatorOf(k) case final a?) (a, c)];
    final path = file ?? '${Platform.environment['HOME']}/.motolii/keybindings.json';
    try {
      final told = jsonDecode(File(path).readAsStringSync()) as List;
      for (final e in told.cast<Map>()) {
        final a = activatorOf('${e['key']}'), c = '${e['command']}';
        if (a == null) continue;
        all.removeWhere((p) => _same(p.$1, a));   // (one key, one command: the maker's)
        if (!c.startsWith('-')) all.add((a, c));
      }
    } catch (_) {}   // (no file, or not a list: the window's own keys)
    return Keymap(all);
  }

  /// The first key that says a command (the menus show it), if any.
  SingleActivator? keyOf(String command) => pairs.where((p) => p.$2 == command).firstOrNull?.$1;

  static bool _same(SingleActivator a, SingleActivator b) => a.trigger == b.trigger && a.meta == b.meta && a.shift == b.shift && a.control == b.control && a.alt == b.alt;
}

/// A key as written ("cmd+shift+z", "space", "left"...) as Flutter's own activator; null if not one it knows.
SingleActivator? activatorOf(String said) {
  final parts = said.toLowerCase().split('+').map((p) => p.trim()).toList();
  final key = _keys[parts.last] ?? (parts.last.length == 1 ? LogicalKeyboardKey.findKeyByKeyId(parts.last.codeUnitAt(0)) : null);
  if (key == null) return null;
  final mods = parts.sublist(0, parts.length - 1).toSet();
  return SingleActivator(key, meta: mods.contains('cmd') || mods.contains('meta'), shift: mods.contains('shift'), control: mods.contains('ctrl') || mods.contains('control'), alt: mods.contains('alt') || mods.contains('option'));
}

const _keys = {
  'space': LogicalKeyboardKey.space, 'left': LogicalKeyboardKey.arrowLeft, 'right': LogicalKeyboardKey.arrowRight, 'up': LogicalKeyboardKey.arrowUp,
  'down': LogicalKeyboardKey.arrowDown, 'home': LogicalKeyboardKey.home, 'end': LogicalKeyboardKey.end, 'pageup': LogicalKeyboardKey.pageUp,
  'pagedown': LogicalKeyboardKey.pageDown, 'backspace': LogicalKeyboardKey.backspace, 'delete': LogicalKeyboardKey.delete, 'enter': LogicalKeyboardKey.enter,
  'escape': LogicalKeyboardKey.escape, 'tab': LogicalKeyboardKey.tab, 'f1': LogicalKeyboardKey.f1, 'f2': LogicalKeyboardKey.f2, 'f3': LogicalKeyboardKey.f3,
  'f4': LogicalKeyboardKey.f4, 'f5': LogicalKeyboardKey.f5, 'f6': LogicalKeyboardKey.f6, 'f7': LogicalKeyboardKey.f7, 'f8': LogicalKeyboardKey.f8,
  'f9': LogicalKeyboardKey.f9, 'f10': LogicalKeyboardKey.f10, 'f11': LogicalKeyboardKey.f11, 'f12': LogicalKeyboardKey.f12,
};

/// A command said by a key.
class CommandIntent extends Intent {
  const CommandIntent(this.name, this.plain);
  final String name;
  final bool plain;   // (a key with no cmd, ctrl or alt: the words' while words are typed)
}

/// The keys over the whole window, each saying its command ([commands]: what each does).
class WsKeys extends StatelessWidget {
  const WsKeys({super.key, required this.keymap, required this.commands, required this.child});
  final Keymap keymap;
  final Map<String, VoidCallback> commands;
  final Widget child;
  @override
  Widget build(BuildContext context) => Shortcuts(
    shortcuts: {for (final (a, c) in keymap.pairs) a: CommandIntent(c, !a.meta && !a.control && !a.alt)},
    child: Actions(actions: {CommandIntent: _Command(commands)}, child: child),
  );
}

class _Command extends Action<CommandIntent> {
  _Command(this.commands);
  final Map<String, VoidCallback> commands;
  static bool get _typing => FocusManager.instance.primaryFocus?.context?.findAncestorStateOfType<EditableTextState>() != null;
  @override
  bool isEnabled(CommandIntent intent) => commands.containsKey(intent.name) && !(intent.plain && _typing);
  @override
  Object? invoke(CommandIntent intent) {
    commands[intent.name]?.call();
    return null;
  }
}
