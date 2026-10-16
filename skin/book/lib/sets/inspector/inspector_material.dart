import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';

import 'inspector_parts.dart';

// ---- 9. material -------------------------------------------------------------------------------------------------------

class FillStroke extends StatelessWidget {
  const FillStroke({super.key, this.first = true});
  final bool first;
  @override
  // contract: the field shows the value and hands over to the Colours tool aimed at THIS slot (I1). Stroke width counts only while a stroke exists.
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, none = (doc.s2['stroke'] ?? 'None') == 'None';
    return Sect(
      title: 'Fill and stroke',
      first: false,
      tone: Pop.toneFill,
      keyIds: const ['stroke.w'],
      mark: CardMark.fill,
      brief: [doc.s2['fill'] ?? '#D9D2C3', none ? 'No stroke' : '${doc.s2['stroke']} ${brief(doc.get('stroke.w'))} px'],
      children: [
        PropRow(
          id: 'fill',
          label: 'Fill',
          keyable: false,
          cells: [PanelLink(to: LinkTo.colors, value: doc.s2['fill'] ?? '#D9D2C3', lead: LinkSwatch(doc.s2['fill'] ?? '#D9D2C3'))],
        ),
        PropRow(
          id: 'stroke',
          label: 'Stroke',
          keyable: false,
          cells: [
            Chooser(id: 'stroke', options: doc.choices['stroke'] ?? const ['None', '#F2F2F2', '#C1C1C1', '#8E8E8E']),
          ],
        ),
        PropRow(
          id: 'stroke.w',
          label: 'Stroke width',
          enabled: !none,
          cells: [NumField(id: 'stroke.w', unit: 'px', decimals: 1, min: 0, max: 64, perPx: .1)],
        ),
      ],
    );
  }
}
