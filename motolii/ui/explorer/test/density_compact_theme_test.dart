// Flutter's own widgets, themed and not rebuilt, reach the density of a desktop creative tool (Motolii's chrome is 18-24 px): the
// general controls need no widget of ours. Material's default (48 px and up) is what this replaces.
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_explorer/stories/density.dart';

void main() {
  Future<Map<String, double>> heights(WidgetTester t, ThemeData theme) async {
    await t.pumpWidget(MaterialApp(
      theme: theme,
      home: Scaffold(
        body: Align(
          alignment: Alignment.topLeft,
          child: SizedBox(
            width: 300,
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              const TextField(key: Key('field')),
              DropdownMenu<int>(key: const Key('dropdown'), dropdownMenuEntries: const [DropdownMenuEntry(value: 1, label: 'Normal')], initialSelection: 1, width: 300),
              FilledButton(key: const Key('button'), onPressed: () {}, child: const Text('Apply')),
              TextButton(key: const Key('textbutton'), onPressed: () {}, child: const Text('Reset')),
              Slider(key: const Key('slider'), value: .5, onChanged: (_) {}),
              Checkbox(key: const Key('checkbox'), value: true, onChanged: (_) {}),
              const ListTile(key: Key('tile'), title: Text('Rounded Rectangle')),
              SegmentedButton<int>(key: const Key('segmented'), segments: const [ButtonSegment(value: 1, label: Text('2D')), ButtonSegment(value: 2, label: Text('2.5D'))], selected: const {2}, onSelectionChanged: (_) {}),
            ]),
          ),
        ),
      ),
    ));
    return {for (final k in ['field', 'dropdown', 'button', 'textbutton', 'slider', 'checkbox', 'tile', 'segmented']) k: t.getSize(find.byKey(Key(k))).height};
  }

  testWidgets('Material by default is a touch UI: 48 px and up', (t) async {
    final h = await heights(t, ThemeData(useMaterial3: true));
    for (final k in ['field', 'dropdown', 'button', 'slider', 'checkbox', 'tile', 'segmented']) {
      expect(h[k], greaterThanOrEqualTo(40), reason: k);
    }
  });

  testWidgets('the same standard widgets under a compact Theme are a desktop tool: 20-24 px, with no widget of ours', (t) async {
    final h = await heights(t, compactMaterialTheme());
    for (final k in ['field', 'dropdown', 'button', 'textbutton', 'slider', 'tile']) {
      expect(h[k], lessThanOrEqualTo(22), reason: '$k is ${h[k]}');
    }
    for (final k in ['checkbox', 'segmented']) {
      expect(h[k], lessThanOrEqualTo(26), reason: '$k is ${h[k]} (the density floor of that widget)');
    }
  });
}
