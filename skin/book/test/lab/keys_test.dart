// The keys' table: the window's own, a maker's file over it (one key, one command; '-' takes a key away).
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/workspace/keys.dart';

void main() {
  test('a maker\'s keys over the window\'s own', () {
    final f = File('${Directory.systemTemp.createTempSync('keys').path}/keybindings.json')
      ..writeAsStringSync('[{"key": "k", "command": "play.toggle"}, {"key": "cmd+shift+z", "command": "-edit.redo"}, {"key": "cmd+y", "command": "edit.redo"}, {"key": "nonsense+++", "command": "x"}]');
    final k = Keymap.load(f.path);
    expect(k.keyOf('play.toggle')?.trigger, LogicalKeyboardKey.space);   // (the window's own Space still says it)
    expect(k.pairs.any((p) => p.$1.trigger == LogicalKeyboardKey.keyK && p.$2 == 'play.toggle'), true);
    expect(k.keyOf('edit.redo')?.trigger, LogicalKeyboardKey.keyY);   // (⇧⌘Z taken away, ⌘Y given)
    expect(k.keyOf('edit.redo')?.meta, true);
    expect(activatorOf('cmd+shift+left')?.shift, true);
    expect(activatorOf('nonsense'), null);
  });
  test('no file: the window\'s own', () {
    final k = Keymap.load('/no/such/file.json');
    expect(k.pairs.length, defaultKeys.length);
  });
}
