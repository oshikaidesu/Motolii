// The window's controls, headless (React Aria's and Radix's manner): the window says what a control is and does (its
// words, its value, what a press does) and a skin's Forms says how it looks and how it is put together, as Dart. What a
// hand does to it (press, hover, focus, pick) is naked_ui's, so a form draws and never re-makes behaviour.
// A skin names its forms in theme.xml (<Forms Value="poster"/>); a new form is a file in lib/forms and a line in [forms].
// Saved while the window runs through live.sh, it shows at once.
import 'package:flutter/widgets.dart';

import '../theme.dart';
import 'own.dart';
import 'poster.dart';

/// Every form the window knows, by the name a skin says.
const forms = <String, Forms>{'own': OwnForms(), 'poster': PosterForms()};

/// One loud action (Render).
class ActionSpec {
  const ActionSpec(this.label, this.onPressed);
  final String label;
  final VoidCallback? onPressed;
}

/// One of a few (2D · 2.5D · 3D): each option a word and, if it has one, a glyph drawn in a 16 x 14 box.
class ChoiceSpec {
  const ChoiceSpec({required this.labels, required this.selected, required this.onSelect, this.glyphs = const [], this.enabled = true});
  final List<String> labels;
  final int selected;
  final ValueChanged<int> onSelect;
  final List<void Function(Canvas c, Paint stroke)> glyphs;
  final bool enabled;
}

/// On or off.
class SwitchSpec {
  const SwitchSpec({required this.on, required this.onChanged, this.enabled = true});
  final bool on, enabled;
  final ValueChanged<bool> onChanged;
}

/// A card of the inspector: its title and kind's colour, whether it is open and what opens it, what sits at the end of
/// its head, its rows; and the window's own head and the whole of it as the window puts them together.
class CardSpec {
  const CardSpec({this.title, this.tone, required this.open, this.onToggle, this.trailing, required this.body, this.head, required this.own});
  final String? title;
  final Color? tone;
  final bool open;
  final VoidCallback? onToggle;
  final Widget? trailing, head;
  final List<Widget> body;
  final Widget own;
}

/// How a skin draws the window's controls.
abstract class Forms {
  const Forms();

  /// The forms the skin worn names (none, or one the window does not know: its own).
  static Forms get worn => forms[ColorTheme.word('Forms', 'own')] ?? const OwnForms();

  Widget action(BuildContext context, ActionSpec a);
  Widget choice(BuildContext context, ChoiceSpec c);

  /// Where one is (Edit · Animate · Export): words only, one lit.
  Widget tabs(BuildContext context, ChoiceSpec c);
  Widget onOff(BuildContext context, SwitchSpec s);
  Widget card(BuildContext context, CardSpec c);
}
