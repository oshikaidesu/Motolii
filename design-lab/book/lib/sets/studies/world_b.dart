// World B: proposals 7-12 of research/op1-translation.md as small WORLDS (one box per relation/effect, up to four colour-matched grab zones).
// One use case per world: the 320x200 box, the same world as a 282 px Inspector row between two plain number rows, and a Driven knob
// (Off | Wiggle | Pulse) that moves the parameters from outside so the graphic is seen moving, the way OP-1 shows modulation.
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../tokens.dart';
import 'world_b_beads.dart';
import 'world_b_ear.dart';
import 'world_b_fan.dart';
import 'world_b_kit.dart';
import 'world_b_lattice.dart';
import 'world_b_thread.dart';
import 'world_b_trace.dart';

class WbEntry {
  const WbEntry(this.name, this.make, this.above, this.below);
  final String name;
  final WbWorld Function() make;
  final (String, String) above, below; // the two plain number rows of the Inspector
}

final _worlds = <WbEntry>[
  WbEntry('Rotation distribution', WbFan.new, ('Scale', '100 %'), ('Opacity', '100 %')),
  WbEntry('Colour gradient', WbBeads.new, ('Position', '960, 540'), ('Opacity', '100 %')),
  WbEntry('Repeat', WbLattice.new, ('Position', '960, 540'), ('Scale', '100 %')),
  WbEntry('Noise', WbTrace.new, ('Target', 'Rotation'), ('Weight', '1.00')),
  WbEntry('Graph / Link', WbThread.new, ('Source', 'Layer 2'), ('Target', 'Scale')),
  WbEntry('Audio react', WbEar.new, ('Source', 'Track 1'), ('Target', 'Scale')),
];

WidgetbookComponent worldBSet() => WidgetbookComponent(name: 'World B', useCases: [
      for (final e in _worlds)
        WidgetbookUseCase(
          name: e.name,
          builder: (c) => onGround(
            WbPair(entry: e, mode: c.knobs.object.dropdown<WbDriven>(label: 'Driven', options: WbDriven.values, initialOption: WbDriven.off, labelBuilder: (m) => m.label)),
            padding: const EdgeInsets.all(20),
          ),
        ),
    ]);

class WbPair extends StatefulWidget {
  const WbPair({super.key, required this.entry, required this.mode});
  final WbEntry entry;
  final WbDriven mode;
  @override
  State<WbPair> createState() => _WbPairState();
}

class _WbPairState extends State<WbPair> {
  late final WbWorld _probe = widget.entry.make();
  late final WbDoc _doc = WbDoc(_probe.specs);

  Widget _row((String, String) r) => Container(
        width: 282,
        height: 22,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: const BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: N.rowLine))),
        child: Row(children: [
          Expanded(child: Text(r.$1, maxLines: 1, style: T.label(N.g63))),
          Text(r.$2, maxLines: 1, style: T.value(N.g91)),
        ]),
      );

  @override
  Widget build(BuildContext context) => Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
            WbBox(make: widget.entry.make, doc: _doc, mode: widget.mode, width: 320, height: 200),
            const SizedBox(width: 28),
            Column(mainAxisSize: MainAxisSize.min, children: [
              _row(widget.entry.above),
              WbBox(make: widget.entry.make, doc: _doc, mode: widget.mode, width: 282, height: 108),
              _row(widget.entry.below),
            ]),
          ]),
          const SizedBox(height: 10),
          SingleChildScrollView(scrollDirection: Axis.horizontal, child: Text(_probe.caption, maxLines: 1, softWrap: false, style: T.label(N.g63))),
        ],
      );
}
