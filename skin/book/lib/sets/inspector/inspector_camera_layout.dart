import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../parts/controls.dart';
import 'inspector_parts.dart';

// ---- 11. camera and layout ------------------------------------------------------------------------------------------
/// D4 (maybe): a group's key mark keys every member at once.
class GroupKey extends StatelessWidget {
  const GroupKey(this.group, this.members, {super.key});
  final String group;
  final List<String> members;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    if (!x.cfg.maybe) return const SizedBox.shrink();
    final all = members.every((m) => doc.keys[m] == KeyS.at),
        any = members.any((m) => (doc.keys[m] ?? KeyS.off) != KeyS.off);
    return KeyMark(
      state: all ? KeyS.at : (any ? KeyS.anim : KeyS.off),
      enabled: !x.cfg.locked,
      onTap: () {
        for (final m in members) {
          doc.keys[m] = all ? KeyS.anim : KeyS.at;
        }
        doc.poke();
      },
    );
  }
}

class CameraGroup extends StatelessWidget {
  const CameraGroup({super.key, this.first = true});
  final bool first;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc,
        driven = (doc.s2['v_parent'] ?? 'None') != 'None';
    // contract: Target / Orbit / Framing / Roll. When a target layer drives the target its fields are disabled and say why (A4).
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Sect(
          title: 'Target',
          first: first,
          trailing: const GroupKey('Target', ['v_position']),
          children: [
            const PropRow(
              id: 'v_parent',
              label: 'Layer',
              keyable: false,
              cells: [
                Chooser(id: 'v_parent', options: layers, searchable: true),
              ],
            ),
            PropRow(
              id: 'v_position',
              label: 'Point',
              enabled: !driven,
              ids: const ['v_x', 'v_y', 'v_z'],
              cells: [
                NumField(
                  id: 'v_x',
                  label: 'X',
                  axis: axX,
                  unit: 'px',
                  enabled: !driven,
                ),
                NumField(
                  id: 'v_y',
                  label: 'Y',
                  axis: axY,
                  unit: 'px',
                  enabled: !driven,
                ),
                NumField(
                  id: 'v_z',
                  label: 'Z',
                  axis: axZ,
                  unit: 'px',
                  enabled: !driven,
                ),
              ],
            ),
          ],
        ),
        Sect(
          title: 'Orbit',
          trailing: const GroupKey('Orbit', ['v_rotate_x', 'v_rotate_y']),
          children: const [
            PropRow(
              id: 'v_rotate_x',
              label: 'Pitch',
              cells: [
                NumField(
                  id: 'v_rotate_x',
                  unit: '°',
                  decimals: 1,
                  min: -90,
                  max: 90,
                  perPx: .5,
                ),
              ],
            ),
            PropRow(
              id: 'v_rotate_y',
              label: 'Yaw',
              cells: [
                NumField(id: 'v_rotate_y', unit: '°', decimals: 1, perPx: .5),
              ],
            ),
          ],
        ),
        const Sect(
          title: 'Framing',
          children: [
            PropRow(
              id: 'v_distance',
              label: 'Distance',
              cells: [
                NumField(
                  id: 'v_distance',
                  unit: 'px',
                  min: 10,
                  max: 10000,
                  perPx: 5,
                  step: 10,
                ),
              ],
            ),
            PropRow(
              id: 'v_zoom',
              label: 'Zoom',
              cells: [
                NumField(
                  id: 'v_zoom',
                  unit: '%',
                  min: 10,
                  max: 1000,
                  perPx: .5,
                ),
              ],
            ),
          ],
        ),
        const Sect(
          title: 'Roll',
          children: [
            PropRow(
              id: 'v_rotate',
              label: 'Roll',
              cells: [
                NumField(
                  id: 'v_rotate',
                  unit: '°',
                  decimals: 1,
                  perPx: .5,
                  turns: true,
                ),
              ],
            ),
          ],
        ),
      ],
    );
  }
}

class LayoutGroup extends StatelessWidget {
  const LayoutGroup({super.key, this.first = false});
  final bool first;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context),
        doc = x.doc,
        on = doc.b['lay.on'] ?? true,
        fixed = (doc.s2['size'] ?? 'Hug') == 'Fixed';
    const sizes = ['Hug', 'Fill', 'Fixed'];
    // contract (A5, maybe): turning Layout off dims everything but its switch; the size number counts only when the size mode is Fixed; a child can opt out.
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Sect(
          title: 'Layout',
          first: first,
          trailing: OnOff(on: on, onChanged: (v) => doc.flag('lay.on', v)),
          children: [
            Opacity(
              opacity: on ? 1 : .4,
              child: IgnorePointer(
                ignoring: !on,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const PropRow(
                      id: 'lay.cols',
                      label: 'Columns',
                      cells: [
                        NumField(id: 'lay.cols', min: 1, max: 12, unit: ''),
                      ],
                    ),
                    const PropRow(
                      id: 'lay.rows',
                      label: 'Rows',
                      cells: [
                        NumField(
                          id: 'lay.rows',
                          min: 1,
                          max: 12,
                          zeroWord: null,
                        ),
                      ],
                    ),
                    const PropRow(
                      id: 'lay.gap',
                      label: 'Gap',
                      cells: [
                        NumField(id: 'lay.gap', unit: 'px', min: 0, max: 200),
                      ],
                    ),
                    const PropRow(
                      id: 'lay.pad',
                      label: 'Padding',
                      cells: [
                        NumField(id: 'lay.pad', unit: 'px', min: 0, max: 200),
                      ],
                    ),
                    PropRow(
                      id: 'size',
                      label: 'Size',
                      keyable: false,
                      cells: [
                        Dis(
                          child: Segmented(
                            items: sizes,
                            index: sizes.indexOf(doc.s2['size'] ?? 'Hug'),
                            expand: true,
                            onChanged: (i) => doc.str('size', sizes[i]),
                          ),
                        ),
                      ],
                    ),
                    PropRow(
                      id: 'lay.w',
                      label: 'Width',
                      enabled: fixed,
                      cells: [
                        NumField(
                          id: 'lay.w',
                          unit: 'px',
                          min: 0,
                          max: 8000,
                          enabled: fixed,
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
        Sect(
          title: 'As a child',
          children: [
            PropRow(
              id: 'lay.ignore',
              label: 'Ignore layout',
              keyable: false,
              cells: [
                Align(
                  alignment: Alignment.centerLeft,
                  child: OnOff(
                    on: doc.b['lay.ignore'] ?? false,
                    onChanged: (v) => doc.flag('lay.ignore', v),
                  ),
                ),
              ],
            ),
          ],
        ),
      ],
    );
  }
}
