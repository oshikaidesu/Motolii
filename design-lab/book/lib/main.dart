// The lab's Widgetbook. Parts are in lib/parts and read lib/tokens.dart, which reads ../DESIGN.md.
// An open design question is a knob ("Look": concept / quiet / glow): the owner picks in the real window.
import 'package:flutter/material.dart' show ThemeMode, ThemeData, Brightness;
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'a11y_addon.dart';
import 'archive.dart';
import 'colour_addon.dart';
import 'parts/controls.dart';
import 'sets/workflows/dose_set.dart';
import 'sets/foundations/feedback.dart';
import 'sets/inspector_gui_set.dart';
import 'sets/inspector/inspector_set.dart';
import 'sets/foundations/inputs.dart';
import 'sets/panels/media_set.dart';
import 'sets/foundations/navigation.dart';
import 'sets/panels/relation_set.dart';
import 'sets/panels/stage_set.dart';
import 'sets/panels/timeline_parts.dart';
import 'sets/panels/transport.dart';
import 'sets/workflows/workflow_set.dart';
import 'tokens.dart';
import 'words_addon.dart';
import 'workspace/workspace.dart';

void main() => runApp(const LabBook());

Widget _onGround(Widget child, {double? width, double? height}) => ColoredBox(
  color: N.g07,
  child: Center(
    child: SizedBox(width: width, height: height, child: child),
  ),
);

class LabBook extends StatelessWidget {
  const LabBook({super.key});
  @override
  Widget build(BuildContext context) => Widgetbook.material(
    initialRoute: '/?path=workspace/workspace/window',
    addons: [ColourAddon(), WordsAddon(), A11yAddon()],
    themeMode: ThemeMode.dark,
    darkTheme: ThemeData(brightness: Brightness.dark, fontFamily: 'Inter'),
    directories: [
      WidgetbookCategory(
        name: 'Workspace',
        children: [
          WidgetbookComponent(
            name: 'Workspace',
            useCases: [WidgetbookUseCase(name: 'Window', builder: (c) => const Workspace())],
          ),
        ],
      ),
      WidgetbookCategory(
        name: 'Parts',
        children: [
          WidgetbookFolder(
            name: 'Inspector',
            children: [
              inspectorSetSet().copyWith(name: 'Inspector Pop'),
              inspectorGuiSet().copyWith(name: 'Inspector GUI instruments'),
            ],
          ),
          timelinePartsSet().copyWith(name: 'Timeline parts'),
          stageSetSet().copyWith(name: 'Stage parts'),
          mediaSetSet().copyWith(name: 'Media parts'),
          transportSet().copyWith(name: 'Transport and bars'),
          relationSetSet().copyWith(name: 'Relation parts'),
        ],
      ),
      WidgetbookCategory(
        name: 'Foundations',
        children: [
          WidgetbookComponent(
            name: 'Tokens',
            useCases: [WidgetbookUseCase(name: 'Families and greys', builder: (c) => _onGround(const _Tokens()))],
          ),
          WidgetbookComponent(
            name: 'Controls',
            useCases: [
              WidgetbookUseCase(
                name: 'Value well',
                builder: (c) => _onGround(
                  SizedBox(
                    width: 260,
                    child: ValueWell(
                      label: 'Density',
                      value: c.knobs.double.slider(label: 'Value', initialValue: .72, min: 0, max: 1),
                      fam: Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 0, min: 0, max: 5)],
                    ),
                  ),
                ),
              ),
              WidgetbookUseCase(
                name: 'Switch',
                builder: (c) => _onGround(PillSwitch(on: c.knobs.boolean(label: 'On', initialValue: true))),
              ),
              WidgetbookUseCase(
                name: 'Segmented',
                builder: (c) => _onGround(
                  Segmented(
                    items: const ['Edit', 'Play', 'Export'],
                    index: c.knobs.int.slider(label: 'Index', initialValue: 0, min: 0, max: 2),
                  ),
                ),
              ),
            ],
          ),
          inputsSet().copyWith(name: 'Inputs'),
          navigationSet().copyWith(name: 'Navigation'),
          feedbackSet().copyWith(name: 'Feedback'),
        ],
      ),
      WidgetbookCategory(name: 'Workflows', children: [workflowSet(), doseSet()]),
      archiveCategory(),
    ],
  );
}

class _Tokens extends StatelessWidget {
  const _Tokens();
  Widget sw(String n, Color c) => Container(
    width: 74,
    margin: const EdgeInsets.only(right: 6, bottom: 6),
    child: Flex(
      direction: Axis.vertical,
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          height: 30,
          decoration: BoxDecoration(
            color: c,
            borderRadius: BorderRadius.circular(5),
            border: Border.all(color: N.g20),
          ),
        ),
        const SizedBox(height: 4),
        Text(n, style: T.micro(N.g63)),
      ],
    ),
  );
  @override
  Widget build(BuildContext context) => Wrap(
    children: [
      for (final f in Fam.all) sw(f.name, f.c),
      sw('g07', N.g07),
      sw('g10', N.g10),
      sw('g13', N.g13),
      sw('g20', N.g20),
      sw('g44', N.g44),
      sw('g63', N.g63),
      sw('g95', N.g95),
      sw('accent (mode)', C.mode),
      sw('danger [open]', C.danger),
      sw('play', C.play),
      sw('record', C.record),
      sw('bandA', N.bandA),
      sw('bandB', N.bandB),
      sw('rowLine', N.rowLine),
      sw('playhead', C.playhead),
    ],
  );
}
