// Density: the same controls as Motolii draws them now, as Flutter's Material draws them by default, and as the very same standard
// widgets draw under a desktop-compact Theme (no widget of ours). A fixture for looking, and for the contract in test/density_compact_theme_test.dart.
import 'package:flutter/material.dart';
import 'package:motolii_stage5/foundation/theme.dart' show EditorButton;
import 'package:motolii_stage5/foundation/leaves/track.dart' show EditorSlider;
import 'package:motolii_stage5/hf/metrics.dart';
import 'package:motolii_stage5/hf/neutral.dart' show N;

import '../story.dart';

/// Standard Material widgets brought to a desktop tool's density by Theme alone: VisualDensity and MaterialTapTargetSize shrink what they
/// can (note that `VisualDensity` also subtracts from `minimumSize`, so buttons and fields take their height from an explicit minimum, and
/// the controls that have no height setting take the density floor).
ThemeData compactMaterialTheme({double row = 20}) {
  const text = TextStyle(fontSize: 11, height: 1.0);
  const tight = VisualDensity(horizontal: -4, vertical: -4);
  final field = InputDecorationTheme(isDense: true, contentPadding: const EdgeInsets.symmetric(horizontal: 6), border: const OutlineInputBorder(borderSide: BorderSide.none), filled: true, constraints: BoxConstraints(minHeight: row, maxHeight: row));
  ButtonStyle button(double px) => ButtonStyle(visualDensity: VisualDensity.standard, tapTargetSize: MaterialTapTargetSize.shrinkWrap, minimumSize: WidgetStatePropertyAll(Size(0, row)), padding: WidgetStatePropertyAll(EdgeInsets.symmetric(horizontal: px)), textStyle: const WidgetStatePropertyAll(text));
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    visualDensity: VisualDensity.standard,
    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
    textTheme: const TextTheme(bodyMedium: text, bodyLarge: text, labelLarge: text, titleMedium: text, bodySmall: text, labelMedium: text),
    inputDecorationTheme: field,
    filledButtonTheme: FilledButtonThemeData(style: button(8)),
    textButtonTheme: TextButtonThemeData(style: button(6)),
    sliderTheme: const SliderThemeData(trackHeight: 2, thumbShape: RoundSliderThumbShape(enabledThumbRadius: 5), overlayShape: RoundSliderOverlayShape(overlayRadius: 10)),
    checkboxTheme: const CheckboxThemeData(materialTapTargetSize: MaterialTapTargetSize.shrinkWrap, visualDensity: tight),
    listTileTheme: ListTileThemeData(dense: true, minVerticalPadding: 0, minTileHeight: row, visualDensity: VisualDensity.standard, titleTextStyle: text),
    dropdownMenuTheme: DropdownMenuThemeData(textStyle: text, inputDecorationTheme: field),
    segmentedButtonTheme: const SegmentedButtonThemeData(style: ButtonStyle(visualDensity: VisualDensity(horizontal: -2, vertical: -4), tapTargetSize: MaterialTapTargetSize.shrinkWrap, padding: WidgetStatePropertyAll(EdgeInsets.symmetric(horizontal: 6)), textStyle: WidgetStatePropertyAll(text))),
    switchTheme: const SwitchThemeData(materialTapTargetSize: MaterialTapTargetSize.shrinkWrap),
  );
}

/// Motolii's two marks that Material has no slot for, carried by Theme itself: the lamp of a keyed property and the colour of an
/// identity (a layer, an accent). They are read as `Theme.of(context).extension<MotoliiMarks>()`.
@immutable
class MotoliiMarks extends ThemeExtension<MotoliiMarks> {
  const MotoliiMarks({required this.keyed, required this.identity});
  final Color keyed, identity;
  @override
  MotoliiMarks copyWith({Color? keyed, Color? identity}) => MotoliiMarks(keyed: keyed ?? this.keyed, identity: identity ?? this.identity);
  @override
  MotoliiMarks lerp(MotoliiMarks? other, double t) => other == null ? this : MotoliiMarks(keyed: Color.lerp(keyed, other.keyed, t)!, identity: Color.lerp(identity, other.identity, t)!);
}

/// The Motolii skin as a Theme over the compact standard widgets: colours for normal / hover / focus / selected / disabled, the
/// section boundary as DividerTheme, and the marks as a ThemeExtension. No widget is ours.
ThemeData motoliiMaterialTheme({double row = 20}) {
  final base = compactMaterialTheme(row: row);
  const accent = Color(0xFF6FA8FF);
  OutlineInputBorder edge(Color c) => OutlineInputBorder(borderRadius: BorderRadius.circular(2), borderSide: BorderSide(color: c));
  final field = InputDecorationTheme(
    isDense: true,
    contentPadding: const EdgeInsets.symmetric(horizontal: 6),
    filled: true,
    fillColor: N.g07,
    constraints: BoxConstraints(minHeight: row, maxHeight: row),
    border: edge(N.clear),
    enabledBorder: edge(N.clear),
    focusedBorder: edge(accent),
    disabledBorder: edge(N.clear),
    hintStyle: const TextStyle(color: N.g44),
  );
  Color? fill(Set<WidgetState> s) => s.contains(WidgetState.disabled) ? N.g13 : s.contains(WidgetState.pressed) ? N.g26 : s.contains(WidgetState.hovered) ? N.g20 : N.g15;
  Color? ink(Set<WidgetState> s) => s.contains(WidgetState.disabled) ? N.g44 : N.g95;
  ButtonStyle skin(ButtonStyle b) => b.copyWith(
        backgroundColor: WidgetStateProperty.resolveWith(fill),
        foregroundColor: WidgetStateProperty.resolveWith(ink),
        overlayColor: const WidgetStatePropertyAll(N.clear),
        shape: WidgetStatePropertyAll(RoundedRectangleBorder(borderRadius: BorderRadius.circular(2))),
        side: WidgetStateProperty.resolveWith((s) => s.contains(WidgetState.focused) ? const BorderSide(color: accent) : BorderSide.none),
      );
  return base.copyWith(
    scaffoldBackgroundColor: N.g10,
    hoverColor: N.g15,
    colorScheme: ColorScheme.dark(primary: accent, surface: N.g10, onSurface: N.g95, secondary: accent),
    inputDecorationTheme: field,
    dropdownMenuTheme: DropdownMenuThemeData(textStyle: base.textTheme.bodyMedium, inputDecorationTheme: field),
    filledButtonTheme: FilledButtonThemeData(style: skin(base.filledButtonTheme.style!)),
    textButtonTheme: TextButtonThemeData(style: skin(base.textButtonTheme.style!)),
    segmentedButtonTheme: SegmentedButtonThemeData(style: skin(base.segmentedButtonTheme.style!).copyWith(backgroundColor: WidgetStateProperty.resolveWith((s) => s.contains(WidgetState.selected) ? N.g26 : fill(s)))),
    checkboxTheme: base.checkboxTheme.copyWith(fillColor: WidgetStateProperty.resolveWith((s) => s.contains(WidgetState.selected) ? accent : N.g20), side: const BorderSide(color: N.g44)),
    sliderTheme: base.sliderTheme.copyWith(activeTrackColor: accent, inactiveTrackColor: N.g26, thumbColor: N.g95),
    listTileTheme: base.listTileTheme.copyWith(selectedTileColor: N.g20, selectedColor: N.g95, textColor: N.g82, tileColor: N.clear),
    dividerTheme: const DividerThemeData(color: N.g20, thickness: 1, space: 1),
    extensions: const [MotoliiMarks(keyed: Color(0xFFE6B450), identity: Color(0xFFE0609A))],
  );
}

/// Each state of each control, forced (WidgetStatesController) so a still shows what a pointer would: one row per control.
Widget skinStates(ThemeData theme) {
  final marks = theme.extension<MotoliiMarks>()!;
  WidgetStatesController at(Set<WidgetState> s) => WidgetStatesController(s);
  Widget cell(String name, Widget w) => Padding(padding: const EdgeInsets.only(right: 6), child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [Text(name, style: const TextStyle(fontSize: 9, color: N.g51)), const SizedBox(height: 2), SizedBox(width: 92, child: w)]));
  final states = <String, Set<WidgetState>>{'normal': {}, 'hover': {WidgetState.hovered}, 'pressed': {WidgetState.pressed}, 'focus': {WidgetState.focused}, 'disabled': {WidgetState.disabled}};
  return MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: theme,
    home: Material(
      color: N.g10,
      child: Padding(
        padding: const EdgeInsets.all(9),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          Wrap(runSpacing: 6, children: [
            for (final e in states.entries) cell('button ${e.key}', FilledButton(statesController: at(e.value), onPressed: e.key == 'disabled' ? null : () {}, child: const Text('Apply'))),
            cell('text focus', TextField(focusNode: FocusNode()..requestFocus(), decoration: const InputDecoration(hintText: 'Name'))),
            cell('text disabled', const TextField(enabled: false, decoration: InputDecoration(hintText: 'Name'))),
            cell('checkbox on', Checkbox(value: true, onChanged: (_) {})),
            cell('checkbox off', Checkbox(value: false, onChanged: (_) {})),
            cell('checkbox disabled', const Checkbox(value: true, onChanged: null)),
            cell('segmented selected', SegmentedButton<int>(showSelectedIcon: false, segments: const [ButtonSegment(value: 1, label: Text('2D')), ButtonSegment(value: 2, label: Text('3D'))], selected: const {2}, onSelectionChanged: (_) {})),
            cell('row normal', const ListTile(title: Text('Position'))),
            cell('row selected', const ListTile(title: Text('Position'), selected: true)),
            cell('keyed lamp + identity', Row(children: [Container(width: 8, height: 8, color: marks.keyed), const SizedBox(width: 6), Container(width: 8, height: 8, color: marks.identity), const SizedBox(width: 6), const Text('Opacity', style: TextStyle(fontSize: 11))])),
          ]),
          const Padding(padding: EdgeInsets.symmetric(vertical: 6), child: Divider()),
          const Text('section boundary: DividerTheme', style: TextStyle(fontSize: 9, color: N.g51)),
        ]),
      ),
    ),
  );
}

/// One inspector-like column of standard widgets.
Widget standardColumn(ThemeData theme) => MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: theme,
      home: Material(
        color: const Color(0xFF1A1A1A),
        child: Padding(
          padding: const EdgeInsets.all(9),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            const Padding(padding: EdgeInsets.only(bottom: 6), child: Text('Rounded Rectangle')),
            ListTile(title: const Text('Position'), trailing: const Text('960.00')),
            const TextField(decoration: InputDecoration(hintText: 'Name')),
            DropdownMenu<int>(dropdownMenuEntries: const [DropdownMenuEntry(value: 1, label: 'Normal')], initialSelection: 1, expandedInsets: EdgeInsets.zero),
            Slider(value: .5, onChanged: (_) {}),
            Row(children: [Checkbox(value: true, onChanged: (_) {}), const Text('Ghost'), const Spacer(), Checkbox(value: false, onChanged: (_) {}), const Text('Clip to below')]),
            SegmentedButton<int>(segments: const [ButtonSegment(value: 1, label: Text('2D')), ButtonSegment(value: 2, label: Text('2.5D')), ButtonSegment(value: 3, label: Text('3D'))], selected: const {2}, onSelectionChanged: (_) {}),
            const SizedBox(height: 6),
            Row(children: [FilledButton(onPressed: () {}, child: const Text('Apply')), const SizedBox(width: 6), TextButton(onPressed: () {}, child: const Text('Reset'))]),
          ]),
        ),
      ),
    );

/// The controls Motolii draws now (the foundation's own widgets), for the eye.
Widget motoliiColumn() => Container(
      color: const Color(0xFF1A1A1A),
      padding: const EdgeInsets.all(Surface.panelInset),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        const Padding(padding: EdgeInsets.only(bottom: 6), child: Text('Rounded Rectangle', style: TextStyle(fontSize: 11, color: Color(0xFFDDDDDD)))),
        SizedBox(height: Surface.workRow, child: Row(children: const [Text('Position', style: TextStyle(fontSize: 11, color: Color(0xFFDDDDDD))), Spacer(), Text('960.00', style: TextStyle(fontSize: 11, color: Color(0xFFDDDDDD)))])),
        const SizedBox(height: 4),
        SizedBox(height: Surface.control, child: EditorSlider(value: .5, onChanged: (_) {})),
        const SizedBox(height: 6),
        Row(children: [EditorButton('Apply', () {}), const SizedBox(width: 6), EditorButton('Reset', () {})]),
      ]),
    );

final densityStories = <Story>[
  Story('Density 1 Motolii now', const Scene('night-sky.rrd'), (c) => motoliiColumn(), width: 300, height: 320),
  Story('Density 2 Material default', const Scene('night-sky.rrd'), (c) => standardColumn(ThemeData(useMaterial3: true, brightness: Brightness.dark)), width: 300, height: 520),
  Story('Density 3 Material, compact Theme only', const Scene('night-sky.rrd'), (c) => standardColumn(compactMaterialTheme()), width: 300, height: 320),
  Story('Density 4 Motolii skin by Theme, every state', const Scene('night-sky.rrd'), (c) => skinStates(motoliiMaterialTheme()), width: 640, height: 260),
];
