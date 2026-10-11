// set: dose. research/pop-magazine.md section 10: eight widgets, one knob (Dose 0..3). Seasoning numbers live in `Dose` (tokens.dart).
// Each widget has its own use case with the Dose knob; "All doses side by side" shows every widget at 0, 1, 2 and 3.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../tokens.dart';
import 'dose_parts.dart';
import 'dose_spatial.dart';

int _badges(BuildContext c) => c.knobs.int.slider(label: 'Badges (0-3)', initialValue: 3, min: 0, max: 3, divisions: 3);

/// One column of the side-by-side grid: a dose label over the widget.
Widget _cell(Dose d, Widget child) => Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text('DOSE ${d.level}', style: T.micro(N.g76)),
      const SizedBox(height: Dose.gapL),
      child,
    ]);

Widget _band(String title, Widget Function(Dose) build) => Padding(
      padding: const EdgeInsets.only(bottom: Dose.gapXL * 2),
      child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text(title, style: T.title()),
        const SizedBox(height: Dose.gapXL),
        Wrap(spacing: Dose.gapXL * 2, runSpacing: Dose.gapXL, children: [for (final d in Dose.all) _cell(d, build(d))]),
      ]),
    );

Widget _scroll(Widget child) => ColoredBox(color: N.g07, child: SingleChildScrollView(padding: const EdgeInsets.all(Dose.gapXL * 2), child: child));

WidgetbookComponent doseSet() => WidgetbookComponent(name: 'Dose', useCases: [
      uc('1 Relation chips', (c) => RelationChips(d: doseKnob(c))),
      uc('2 Mode plates', (c) => ModePlates(d: doseKnob(c))),
      uc('3 Easing thumbnails', (c) => EasingThumbs(d: doseKnob(c))),
      uc('4 Browser tile', (c) => TileRow(key: ValueKey(_badges(c)), d: doseKnob(c), badges: _badges(c))),
      uc('5 Panel header', (c) => PanelHeader(d: doseKnob(c))),
      uc('6 Spatial pad', (c) => SpatialPad(d: doseKnob(c))),
      uc('7 Plain controls (same at every dose)', (c) => const PlainControls()),
      uc('8 Too-far meter', (c) => TooFarMeter(key: ValueKey(doseKnob(c).level), d: doseKnob(c))),
      WidgetbookUseCase(name: 'All doses side by side', builder: (c) {
        final b = _badges(c);
        return _scroll(Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
          _band('1  Relation chips', (d) => RelationChips(d: d)),
          _band('2  Mode plates', (d) => ModePlates(d: d)),
          _band('3  Easing thumbnails', (d) => EasingThumbs(d: d)),
          _band('4  Browser tile', (d) => TileRow(key: ValueKey(b), d: d, badges: b)),
          _band('5  Panel header strip', (d) => PanelHeader(d: d)),
          _band('6  Spatial pad', (d) => SpatialPad(d: d)),
          _band('7  Plain controls (no dose input)', (d) => const PlainControls()),
          _band('8  Too-far meter', (d) => TooFarMeter(d: d)),
        ]));
      }),
    ]);
