// Part of world_c_kit.dart (cut there by size, as it was).
part of 'world_c_kit.dart';

class _NumRow extends StatelessWidget {
  const _NumRow(this.label, this.value);
  final String label, value;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g20, width: 1))),
        child: Row(children: [
          Expanded(child: Text(label, style: T.label(N.g63))),
          Text(value, style: T.value(N.g91)),
        ]),
      );
}

/// The large version (320 x 200) with its caption outside the frame.
WidgetbookUseCase worldLarge(WorldDef d) => WidgetbookUseCase(
      name: '${d.no} ${d.name} · large',
      builder: (c) {
        final dr = wcDrivenKnob(c);
        return ColoredBox(
          color: N.g07,
          child: Center(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              WorldBox(key: ValueKey('L${d.no}'), def: d, size: const Size(320, 200), driven: dr),
              _caption(d.caption),
            ]),
          ),
        );
      },
    );

/// The same world in a 282 px Inspector row between two plain number rows.
WidgetbookUseCase worldRow(WorldDef d, {double height = 112}) => WidgetbookUseCase(
      name: '${d.no} ${d.name} · inspector row',
      builder: (c) {
        final dr = wcDrivenKnob(c);
        final h = c.knobs.double.slider(label: 'Row height', initialValue: height, min: 72, max: 132, divisions: 60);
        return ColoredBox(
          color: N.g07,
          child: Center(
            child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Container(
                width: 282,
                color: N.g10,
                child: Column(mainAxisSize: MainAxisSize.min, children: [
                  const _NumRow('Opacity', '100 %'),
                  WorldBox(key: ValueKey('R${d.no}'), def: d, size: Size(282, h), driven: dr),
                  const _NumRow('Blend', 'Normal'),
                ]),
              ),
              _caption(d.caption),
            ]),
          ),
        );
      },
    );
