// Density: the same controls as Motolii draws them now, as Flutter's Material draws them by default, and as the very same standard
// widgets draw under a desktop-compact Theme (no widget of ours). A fixture for looking, and for the contract in test/density_compact_theme_test.dart.
import 'package:flutter/material.dart';
import 'package:motolii_stage5/foundation/theme.dart' show EditorButton;
import 'package:motolii_stage5/foundation/leaves/track.dart' show EditorSlider;
import 'package:motolii_stage5/hf/metrics.dart';

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
      padding: const EdgeInsets.all(UiMetrics.pad),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        const Padding(padding: EdgeInsets.only(bottom: 6), child: Text('Rounded Rectangle', style: TextStyle(fontSize: 11, color: Color(0xFFDDDDDD)))),
        SizedBox(height: UiMetrics.rowStd, child: Row(children: const [Text('Position', style: TextStyle(fontSize: 11, color: Color(0xFFDDDDDD))), Spacer(), Text('960.00', style: TextStyle(fontSize: 11, color: Color(0xFFDDDDDD)))])),
        const SizedBox(height: 4),
        SizedBox(height: UiMetrics.control, child: EditorSlider(value: .5, onChanged: (_) {})),
        const SizedBox(height: 6),
        Row(children: [EditorButton('Apply', () {}), const SizedBox(width: 6), EditorButton('Reset', () {})]),
      ]),
    );

final densityStories = <Story>[
  Story('Density 1 Motolii now', const Scene('night-sky.rrd'), (c) => motoliiColumn(), width: 300, height: 320),
  Story('Density 2 Material default', const Scene('night-sky.rrd'), (c) => standardColumn(ThemeData(useMaterial3: true, brightness: Brightness.dark)), width: 300, height: 520),
  Story('Density 3 Material, compact Theme only', const Scene('night-sky.rrd'), (c) => standardColumn(compactMaterialTheme()), width: 300, height: 320),
];
