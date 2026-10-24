// The window's controls, headless (React Aria's and Radix's manner): the window says what a control is and does (its
// words, its value, what a press does) and a skin's Forms says how it looks and how it is put together, as Dart. What a
// hand does to it (press, hover, focus, pick) is naked_ui's, so a form draws and never re-makes behaviour.
// A skin names its forms in theme.xml (<Forms Value="poster"/>); a new form is a file in lib/forms and a line in [forms].
// Saved while the window runs through live.sh, it shows at once.
import 'package:flutter/widgets.dart';

import '../theme.dart';
import '../tokens.dart';
import 'lighttable.dart';
import 'number_paint.dart';
import 'own.dart';
import 'poster.dart';
import 'sheet.dart';
import 'usual.dart';

/// Every form the window knows, by the name a skin says.
const forms = <String, Forms>{'own': OwnForms(), 'poster': PosterForms(), 'lighttable': LightTableForms(), 'sheet': SheetForms()};

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

/// One of a few small words (Fit · 50% · 100%, Layer · Group). Unlike [ChoiceSpec] in a cell, the row sizes to its words.
class FlagSpec {
  const FlagSpec({required this.label, required this.on, required this.onTap, this.mark});
  final String label;
  final bool on;
  final VoidCallback onTap;

  /// The window's picture of the flag, in the ink the form chooses.
  final Widget Function(Color ink)? mark;
}

/// A shelf's filter chips: a word, an optional count, which one is on.
class ChipsSpec {
  const ChipsSpec({required this.chips, required this.active, required this.onPick});
  final List<(String, int?)> chips;
  final String active;
  final ValueChanged<String> onPick;
}

/// One timeline name. The kind and the three toggles are the window's, because their behaviour is not the form's.
class TrackSpec {
  const TrackSpec({
    required this.index,
    required this.name,
    required this.selected,
    required this.dim,
    required this.hot,
    required this.onSelect,
    required this.kind,
    required this.eye,
    required this.solo,
    required this.lock,
  });
  final int index;
  final String name;
  final bool selected, dim, hot;
  final VoidCallback onSelect;
  final Widget kind, eye, solo, lock;
}

/// How a skin draws the window's controls.
///
/// Dragging a number, typing it, and carrying a timeline row stay in the window. A form is given the pieces and returns the picture.
abstract class Forms {
  const Forms();

  /// The forms the skin worn names (none, or one the window does not know: its own).
  static Forms get worn => _numbersBound ? (forms[ColorTheme.word('Forms', 'own')] ?? const OwnForms()) : const OwnForms();

  Widget action(BuildContext context, ActionSpec a);
  Widget choice(BuildContext context, ChoiceSpec c);

  /// Where one is (Edit · Animate · Export): words only, one lit.
  Widget tabs(BuildContext context, ChoiceSpec c);
  Widget onOff(BuildContext context, SwitchSpec s);
  Widget card(BuildContext context, CardSpec c);

  /// Fit, Layer/Group, and the other small one-of-a-few. The default is the window's own well.
  Widget segment(BuildContext context, ChoiceSpec c) => Usual.segment(context, c);

  /// Snap, Safe, Grid, and the other small on/off chips.
  Widget flag(BuildContext context, FlagSpec f) => Usual.flag(context, f);

  /// A shelf's row of filter chips.
  Widget chips(BuildContext context, ChipsSpec c) => Usual.chips(context, c);

  /// One layer's name in the timeline.
  Widget track(BuildContext context, TrackSpec t) => Usual.track(context, t);

  /// A number's picture. The field calls [paintNumber], which was bound to this when the forms loaded.
  Widget number(BuildContext context, NumberSpec n) => usualNumber(context, n);

  /// The bar of a layer in the timeline: its fill, how tall and how round it is, and the band behind its row.
  Color trackFill(Color kind, {required bool dim, required bool selected}) => dim ? Grey.g26 : (selected ? Color.lerp(kind, const Color(0xFFFFFFFF), .18)! : kind);   // (its colour as it is, as Ableton's clips; picked, a little lighter)
  double get trackRadius => 4;
  double trackBar({required bool selected}) => Shape.z(18);
  Color trackBand(int index, bool selected) => selected ? Grey.g20 : (index.isEven ? Grey.bandA : Grey.bandB);

  /// The mark at the front of a pane's tab. The window's own mark, unless a form puts a shape there.
  Widget paneMark(BuildContext context, {required bool selected, required Widget mark}) => mark;

  /// The tab strip keeps the package's cards. A form that returns true clears them, so the mark is the tab.
  bool get plainTabs => false;
}

/// Installed as this library loads, so a number field can ask the worn form without importing it back.
final bool _numbersBound = () {
  bindNumber((context, spec) => Forms.worn.number(context, spec));
  return true;
}();
