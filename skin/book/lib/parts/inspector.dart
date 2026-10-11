// The inspector (right of the window): the layer's name, its tabs, then its relation cards. The open card is the one being tuned.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'controls.dart';
import 'relations.dart';

class InspectorPanel extends StatefulWidget {
  const InspectorPanel({super.key, this.look = Look.concept});
  final Look look;
  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> {
  final on = <String, bool>{'Scatter': true, 'Stagger': true, 'Along Path': true, 'Face': false};
  @override
  Widget build(BuildContext context) => Container(
        width: 372,
        color: N.g10,
        child: SingleChildScrollView(padding: const EdgeInsets.all(12), child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
          Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Container(width: 26, height: 26, decoration: BoxDecoration(color: Fam.scatter.c, borderRadius: BorderRadius.circular(7))),
            const SizedBox(width: 10),
            Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
              Text('Group', style: T.label(N.g56)),
              const SizedBox(height: 4),
              Text('Jewel Field', style: T.title()),
            ]),
          ]),
          const SizedBox(height: 12),
          const Segmented(items: ['Transform', 'Relations', 'Effects', 'Material'], index: 1, expand: true),
          const SizedBox(height: 12),
          RelationCard(Fam.scatter, look: widget.look, on: on['Scatter']!, onToggle: (v) => setState(() => on['Scatter'] = v)),
          const SizedBox(height: 8),
          for (final f in [Fam.stagger, Fam.along, Fam.face]) _Row(f, on[f.name]!, (v) => setState(() => on[f.name] = v)),
        ])),
      );
}

class _Row extends StatelessWidget {
  const _Row(this.fam, this.on, this.onChanged);
  final Fam fam;
  final bool on;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: 38,
        margin: const EdgeInsets.only(bottom: 4),
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(8)),
        child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Container(width: 9, height: 9, decoration: BoxDecoration(color: on ? fam.c : N.g38, shape: BoxShape.circle)),
          const SizedBox(width: 10),
          Text(fam.name, style: T.title(on ? N.g95 : N.g56)),
          const Spacer(),
          PillSwitch(on: on, fam: fam, onChanged: onChanged),
        ]),
      );
}
