// The lab's Widgetbook. Parts are in lib/parts and read lib/tokens.dart, which reads ../DESIGN.md.
// An open design question is a knob ("Look": concept / quiet / glow): the owner picks in the real window.
import 'package:flutter/material.dart' show ThemeMode, ThemeData, Brightness;
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'parts/browser.dart';
import 'parts/controls.dart';
import 'parts/inspector.dart';
import 'parts/relations.dart';
import 'parts/shell.dart';
import 'parts/timeline.dart';
import 'sets/feedback.dart';
import 'sets/inspector_set.dart';
import 'sets/inputs.dart';
import 'sets/media_set.dart';
import 'sets/navigation.dart';
import 'sets/relation_set.dart';
import 'sets/stage_set.dart';
import 'sets/timeline_parts.dart';
import 'sets/transport.dart';
import 'tokens.dart';

void main() => runApp(const LabBook());

Look _look(BuildContext c) => c.knobs.object.dropdown<Look>(label: 'Look', options: Look.values, initialOption: Look.quiet, labelBuilder: (l) => l.name);

Widget _onGround(Widget child, {double? width, double? height}) => ColoredBox(
      color: N.g07,
      child: Center(child: SizedBox(width: width, height: height, child: child)),
    );

class LabBook extends StatelessWidget {
  const LabBook({super.key});
  @override
  Widget build(BuildContext context) => Widgetbook.material(
        initialRoute: '/?path=inspector/inspector-shape-layer',
        themeMode: ThemeMode.dark,
        darkTheme: ThemeData(brightness: Brightness.dark, fontFamily: 'Inter'),
        directories: [
          inspectorSetSet(), transportSet(), inputsSet(), navigationSet(), timelinePartsSet(), relationSetSet(), feedbackSet(), mediaSetSet(), stageSetSet(),
          WidgetbookComponent(name: 'Shell', useCases: [
            WidgetbookUseCase(name: 'Window', builder: (c) => Shell(look: _look(c))),
            WidgetbookUseCase(name: 'Top bar', builder: (c) => _onGround(const TopBar(), width: 980, height: 48)),
          ]),
          WidgetbookComponent(name: 'Timeline', useCases: [
            WidgetbookUseCase(name: 'Six layers', builder: (c) => _onGround(
                  TimelinePanel(
                    selected: c.knobs.int.slider(label: 'Selected row', initialValue: 2, min: 0, max: 5),
                    head: c.knobs.int.slider(label: 'Playhead (frame)', initialValue: 60, min: 0, max: 120),
                    pixelsPerFrame: c.knobs.double.slider(label: 'Zoom (px / frame)', initialValue: 6, min: 2, max: 16),
                    tuning: TimelineTuning(
                      rowHeight: c.knobs.object.dropdown<double>(label: 'Row height (23 concept, 24 Spectrum, 20 Blender)', options: const [23, 24, 20], initialOption: 23, labelBuilder: (v) => '${v.toInt()} px'),
                      zebra: c.knobs.object.dropdown<int>(label: 'Zebra step (levels)', options: const [12, 8, 4], initialOption: 12, labelBuilder: (v) => '+$v'),
                      keyEdge: c.knobs.object.dropdown<KeyEdge>(label: 'Key edge', options: KeyEdge.values, initialOption: KeyEdge.white, labelBuilder: (v) => v.name),
                      grid: c.knobs.object.dropdown<GridTone>(label: 'Time grid', options: GridTone.values, initialOption: GridTone.light, labelBuilder: (v) => v.name),
                    ),
                  ),
                  width: 1000,
                  height: 220,
                )),
          ]),
          WidgetbookComponent(name: 'Browser', useCases: [
            WidgetbookUseCase(name: 'Objects, generators, relations', builder: (c) => _onGround(BrowserPanel(look: _look(c)))),
          ]),
          WidgetbookComponent(name: 'Inspector', useCases: [
            WidgetbookUseCase(name: 'Relations tab', builder: (c) => _onGround(InspectorPanel(look: _look(c)))),
          ]),
          WidgetbookComponent(name: 'Stage chrome', useCases: [
            WidgetbookUseCase(name: 'Tools and bars', builder: (c) => _onGround(StageChrome(zoom: c.knobs.int.slider(label: 'Zoom %', initialValue: 100, min: 10, max: 400)), width: 560, height: 360)),
          ]),
          WidgetbookComponent(name: 'Relations', useCases: [
            WidgetbookUseCase(name: 'Tiles', builder: (c) => _onGround(Wrap(spacing: 10, runSpacing: 10, children: [for (final f in Fam.all) RelationTile(f, look: _look(c))]))),
            WidgetbookUseCase(name: 'Gadgets', builder: (c) => _onGround(Wrap(spacing: 14, runSpacing: 14, children: [
                  for (final f in Fam.all) Container(padding: const EdgeInsets.all(10), decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(10), border: Border.all(color: N.g20)), child: Gadget(f, size: 150)),
                ]))),
            WidgetbookUseCase(name: 'Card', builder: (c) => _onGround(RelationCard(Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 0, min: 0, max: 5)], look: _look(c)), width: 372)),
          ]),
          WidgetbookComponent(name: 'Controls', useCases: [
            WidgetbookUseCase(name: 'Value well', builder: (c) => _onGround(SizedBox(width: 260, child: ValueWell(label: 'Density', value: c.knobs.double.slider(label: 'Value', initialValue: .72, min: 0, max: 1), fam: Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 0, min: 0, max: 5)])))),
            WidgetbookUseCase(name: 'Switch', builder: (c) => _onGround(PillSwitch(on: c.knobs.boolean(label: 'On', initialValue: true)))),
            WidgetbookUseCase(name: 'Segmented', builder: (c) => _onGround(Segmented(items: const ['Edit', 'Play', 'Export'], index: c.knobs.int.slider(label: 'Index', initialValue: 0, min: 0, max: 2)))),
          ]),
          WidgetbookComponent(name: 'Tokens', useCases: [
            WidgetbookUseCase(name: 'Families and greys', builder: (c) => _onGround(const _Tokens())),
          ]),
        ],
      );
}

class _Tokens extends StatelessWidget {
  const _Tokens();
  Widget sw(String n, Color c) => Container(
        width: 74,
        margin: const EdgeInsets.only(right: 6, bottom: 6),
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          Container(height: 30, decoration: BoxDecoration(color: c, borderRadius: BorderRadius.circular(5), border: Border.all(color: N.g20))),
          const SizedBox(height: 4),
          Text(n, style: T.micro(N.g63)),
        ]),
      );
  @override
  Widget build(BuildContext context) => Wrap(children: [
        for (final f in Fam.all) sw(f.name, f.c),
        sw('g07', N.g07), sw('g10', N.g10), sw('g13', N.g13), sw('g20', N.g20), sw('g44', N.g44), sw('g63', N.g63), sw('g95', N.g95),
        sw('accent (mode)', C.mode), sw('danger [open]', C.danger), sw('play', C.play), sw('record', C.record), sw('bandA', N.bandA), sw('bandB', N.bandB), sw('rowLine', N.rowLine), sw('playhead', C.playhead),
      ]);
}
