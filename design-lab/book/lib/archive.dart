// The Archive: the first drafts the Workspace replaced and the studies behind it. Kept to look back at; nothing here feeds the Workspace.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import 'kit.dart';
import 'parts/browser.dart';
import 'parts/inspector.dart';
import 'parts/relations.dart';
import 'parts/shell.dart';
import 'parts/timeline.dart';
import 'sets/studies/op_01.dart';
import 'sets/studies/op_02.dart';
import 'sets/studies/op_03.dart';
import 'sets/studies/op_04.dart';
import 'sets/studies/op_05.dart';
import 'sets/studies/op_06.dart';
import 'sets/studies/op_07.dart';
import 'sets/studies/op_08.dart';
import 'sets/studies/op_09.dart';
import 'sets/studies/op_10.dart';
import 'sets/studies/pop_01.dart';
import 'sets/studies/pop_02.dart';
import 'sets/studies/pop_03.dart';
import 'sets/studies/pop_04.dart';
import 'sets/studies/pop_05.dart';
import 'sets/studies/pop_06.dart';
import 'sets/studies/pop_07.dart';
import 'sets/studies/pop_08.dart';
import 'sets/studies/pop_09.dart';
import 'sets/studies/pop_10.dart';
import 'sets/studies/world_a.dart';
import 'sets/studies/world_b.dart';
import 'sets/studies/world_c.dart';
import 'sets/studies/world_d.dart';
import 'sets/studies/world_e.dart';
import 'sets/studies/pheno_a.dart';
import 'sets/studies/pheno_b.dart';
import 'sets/studies/pheno_c.dart';
import 'sets/studies/pheno_d.dart';
import 'sets/studies/pheno_e.dart';
import 'sets/studies/panel_browser_a.dart';
import 'sets/studies/panel_browser_b.dart';
import 'sets/studies/panel_inspector_a.dart';
import 'sets/studies/panel_inspector_b.dart';
import 'tokens.dart';

Widget _draft(Widget child, {double? width, double? height}) => onGround(child, width: width, height: height, padding: EdgeInsets.zero);

WidgetbookCategory archiveCategory() => WidgetbookCategory(
  name: 'Archive',
  children: [
    WidgetbookFolder(
      name: 'First drafts',
      children: [
        WidgetbookComponent(
          name: 'Inspector first draft',
          useCases: [
            WidgetbookUseCase(
              name: 'Relations tab',
              builder: (c) => _draft(InspectorPanel(look: lookKnob(c))),
            ),
          ],
        ),
        WidgetbookComponent(
          name: 'Timeline',
          useCases: [
            WidgetbookUseCase(
              name: 'Six layers',
              builder: (c) => _draft(
                TimelinePanel(
                  selected: c.knobs.int.slider(label: 'Selected row', initialValue: 2, min: 0, max: 5),
                  head: c.knobs.int.slider(label: 'Playhead (frame)', initialValue: 60, min: 0, max: 120),
                  pixelsPerFrame: c.knobs.double.slider(label: 'Zoom (px / frame)', initialValue: 6, min: 2, max: 16),
                  tuning: TimelineTuning(
                    rowHeight: c.knobs.object.dropdown<double>(
                      label: 'Row height (23 concept, 24 Spectrum, 20 Blender)',
                      options: const [23, 24, 20],
                      initialOption: 23,
                      labelBuilder: (v) => '${v.toInt()} px',
                    ),
                    zebra: c.knobs.object.dropdown<int>(label: 'Zebra step (levels)', options: const [12, 8, 4], initialOption: 12, labelBuilder: (v) => '+$v'),
                    keyEdge: c.knobs.object.dropdown<KeyEdge>(
                      label: 'Key edge',
                      options: KeyEdge.values,
                      initialOption: KeyEdge.white,
                      labelBuilder: (v) => v.name,
                    ),
                    grid: c.knobs.object.dropdown<GridTone>(
                      label: 'Time grid',
                      options: GridTone.values,
                      initialOption: GridTone.light,
                      labelBuilder: (v) => v.name,
                    ),
                  ),
                ),
                width: 1000,
                height: 220,
              ),
            ),
          ],
        ),
        WidgetbookComponent(
          name: 'Stage chrome',
          useCases: [
            WidgetbookUseCase(
              name: 'Tools and bars',
              builder: (c) => _draft(StageChrome(zoom: c.knobs.int.slider(label: 'Zoom %', initialValue: 100, min: 10, max: 400)), width: 560, height: 360),
            ),
          ],
        ),
        WidgetbookComponent(
          name: 'Browser',
          useCases: [
            WidgetbookUseCase(
              name: 'Objects, generators, relations',
              builder: (c) => _draft(BrowserPanel(look: lookKnob(c))),
            ),
          ],
        ),
        WidgetbookComponent(
          name: 'Shell',
          useCases: [
            WidgetbookUseCase(
              name: 'Window',
              builder: (c) => Shell(look: lookKnob(c)),
            ),
            WidgetbookUseCase(name: 'Top bar', builder: (c) => _draft(const TopBar(), width: 980, height: 48)),
          ],
        ),
        WidgetbookComponent(
          name: 'Relations',
          useCases: [
            WidgetbookUseCase(
              name: 'Tiles',
              builder: (c) => _draft(Wrap(spacing: 10, runSpacing: 10, children: [for (final f in Fam.all) RelationTile(f, look: lookKnob(c))])),
            ),
            WidgetbookUseCase(
              name: 'Gadgets',
              builder: (c) => _draft(
                Wrap(
                  spacing: 14,
                  runSpacing: 14,
                  children: [
                    for (final f in Fam.all)
                      Container(
                        padding: const EdgeInsets.all(10),
                        decoration: BoxDecoration(
                          color: N.g13,
                          borderRadius: BorderRadius.circular(10),
                          border: Border.all(color: N.g20),
                        ),
                        child: Gadget(f, size: 150),
                      ),
                  ],
                ),
              ),
            ),
            WidgetbookUseCase(
              name: 'Card',
              builder: (c) =>
                  _draft(RelationCard(Fam.all[c.knobs.int.slider(label: 'Family', initialValue: 0, min: 0, max: 5)], look: lookKnob(c)), width: 372),
            ),
          ],
        ),
      ],
    ),
    WidgetbookFolder(
      name: 'Studies',
      children: [
        WidgetbookFolder(
          name: 'OP studies',
          children: [op01Set(), op02Set(), op03Set(), op04Set(), op05Set(), op06Set(), op07Set(), op08Set(), op09Set(), op10Set()],
        ),
        WidgetbookFolder(
          name: 'Pop studies',
          children: [pop01Set(), pop02Set(), pop03Set(), pop04Set(), pop05Set(), pop06Set(), pop07Set(), pop08Set(), pop09Set(), pop10Set()],
        ),
        WidgetbookFolder(name: 'Browser studies', children: [...panelBrowserASets(), ...panelBrowserBSets()]),
        WidgetbookFolder(name: 'Inspector studies', children: [...panelInspectorASets(), ...panelInspectorBSets()]),
        WidgetbookFolder(name: 'Worlds', children: [worldASet(), worldBSet(), worldCSet(), worldDSet(), worldESet()]),
        WidgetbookFolder(name: 'Phenomena', children: [phenoASet(), phenoBSet(), phenoCSet(), phenoDSet(), phenoESet()]),
      ],
    ),
  ],
);
