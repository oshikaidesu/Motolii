// Test tool of the lab: the global 'Words' addon (settings panel, "Words"). Full = today; Hidden = every T.* text is transparent, layout unchanged.
// Use it to look at any panel and ask: can I tell what this does without reading? (owner: universal = understandable by touch, without words).
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'tokens.dart';

enum WordsMode {
  full('Full (today)', false),
  hidden('Hidden (text invisible, layout kept)', true);

  const WordsMode(this.label, this.off);
  final String label;
  final bool off;
}

class WordsAddon extends WidgetbookAddon<WordsMode> {
  WordsAddon() : super(name: 'Words');

  @override
  List<Field> get fields => [
        ObjectDropdownField<WordsMode>(name: 'mode', values: WordsMode.values, initialValue: WordsMode.full, labelBuilder: (m) => m.label),
      ];

  @override
  WordsMode valueFromQueryGroup(Map<String, String> group) => valueOf<WordsMode>('mode', group)!;

  @override
  Widget buildUseCase(BuildContext context, Widget child, WordsMode setting) {
    // Same as the Colour addon: set the flag before the build, remount so const subtrees re-read it.
    T.hidden.value = setting.off;
    return KeyedSubtree(key: ValueKey(setting), child: child);
  }
}
