part of 'inspector_parts.dart';

// ---- 12. relations ----------------------------------------------------------------------------------------------------
/// The relation card and rows come from the lab's relations part (RelationCard); only the section around them is here.
class RelationsBlock extends StatefulWidget {
  const RelationsBlock({super.key, this.titled = true, this.first = true});
  final bool titled, first;
  @override
  State<RelationsBlock> createState() => _RelationsBlockState();
}

class _RelationsBlockState extends State<RelationsBlock> {
  final on = <String, bool>{'Scatter': true, 'Stagger': true, 'Along Path': true, 'Face': false};
  String open = 'Scatter';
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed), fams = [Fam.scatter, Fam.stagger, Fam.along, Fam.face], locked = x.cfg.locked;
    // contract: one relation open (its three values and a small plot), the others one hairline row each; the switch bypasses; clicking a row opens it (I4).
    // Same field language as everything else: flat fills, hairlines, no card, no gradient. Hue only on the relation's own mark and plot.
    final rows = <Widget>[
      for (final f in fams) ...[
        Hov(
          onTap: () => setState(() => open = f.name),
          builder: (_, h) => Container(
            height: 30,
            decoration: BoxDecoration(
              border: Border(bottom: BorderSide(color: ed.rule)),
              color: h && !locked ? Grey.g13 : null,
            ),
            child: Row(
              children: [
                Container(
                  width: 6,
                  height: 6,
                  decoration: BoxDecoration(color: on[f.name]! ? f.c : Grey.g38, shape: BoxShape.circle),
                ),
                const SizedBox(width: 10),
                Expanded(child: Text(f.name, style: T.name(on[f.name]! && !locked ? Grey.g95 : Grey.g56))),
                if (f.name == open)
                  Padding(
                    padding: const EdgeInsets.only(right: 12),
                    child: Text('Open', style: T.label(Grey.g56)),
                  ),
                OnOff(on: on[f.name]!, onChanged: (v) => setState(() => on[f.name] = v)),
              ],
            ),
          ),
        ),
        if (f.name == open)
          Padding(
            padding: const EdgeInsets.only(top: 4, bottom: 6),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const PropRow(
                  id: 'rel.d',
                  label: 'Density',
                  keyable: false,
                  cells: [NumField(id: 'rel.d', unit: '%', min: 0, max: 100, perPx: .5)],
                ),
                const PropRow(
                  id: 'rel.s',
                  label: 'Spread',
                  keyable: false,
                  cells: [NumField(id: 'rel.s', unit: '%', min: 0, max: 100, perPx: .5)],
                ),
                const PropRow(
                  id: 'rel.f',
                  label: 'Falloff',
                  keyable: false,
                  cells: [NumField(id: 'rel.f', unit: '%', min: 0, max: 100, perPx: .5)],
                ),
                Padding(
                  padding: const EdgeInsets.only(top: 6),
                  child: Opacity(
                    opacity: on[f.name]! && !locked ? 1 : .5,
                    child: Align(alignment: Alignment.centerLeft, child: Gadget(f, size: 72)),
                  ),
                ),
              ],
            ),
          ),
      ],
      Hov(
        builder: (_, h) => Container(
          height: 28,
          alignment: Alignment.centerLeft,
          child: Text('Add relation', style: T.name(h && !locked ? Grey.g95 : Grey.g63)),
        ),
      ),
    ];
    return Sect(
      title: widget.titled ? 'Relations' : null,
      first: widget.first,
      children: [
        IgnorePointer(
          ignoring: locked,
          child: Opacity(
            opacity: locked ? .6 : 1,
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: rows),
          ),
        ),
      ],
    );
  }
}
