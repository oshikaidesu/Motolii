// Part of world_e_kit.dart (cut there by size, as it was).
part of 'world_e_kit.dart';

/// One use case: the caption (outside the frame, one line), the large world (320x200) and the same world in a 282 px Inspector row between two plain
/// number rows. Both views share one [WDoc]; one ticker (this one) moves the clock of both.
class WStory extends StatefulWidget {
  const WStory({super.key, required this.def, required this.driven});
  final WorldDef def;
  final Driven driven;
  @override
  State<WStory> createState() => _WStoryState();
}

class _WStoryState extends State<WStory> with SingleTickerProviderStateMixin {
  late final WDoc doc = WDoc(widget.def.specs)..driven = widget.driven;
  late final Ticker _tk = createTicker(_tick);
  Duration _last = Duration.zero;

  @override
  void initState() {
    super.initState();
    _tk.start();
  }

  void _tick(Duration e) {
    final dt = math.min((e - _last).inMicroseconds / 1e6, .05);
    _last = e;
    doc.advance(dt);
  }

  @override
  void didUpdateWidget(WStory old) {
    super.didUpdateWidget(old);
    doc.driven = widget.driven;
  }

  @override
  void dispose() {
    _tk.dispose();
    doc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final d = widget.def;
    return ColoredBox(
      color: N.g07,
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          SingleChildScrollView(scrollDirection: Axis.horizontal, child: Text(d.caption, maxLines: 1, softWrap: false, style: T.label(N.g63))),
          const SizedBox(height: 14),
          Wrap(spacing: 40, runSpacing: 28, crossAxisAlignment: WrapCrossAlignment.start, children: [
            WBox(doc: doc, make: d.make, size: const Size(320, 200)),
            SizedBox(
              width: 282,
              child: Column(mainAxisSize: MainAxisSize.min, children: [
                WNumRow(d.above.$1, d.above.$2),
                const SizedBox(height: 4),
                WBox(doc: doc, make: d.make, size: Size(282, d.rowH)),
                const SizedBox(height: 4),
                WNumRow(d.below.$1, d.below.$2),
              ]),
            ),
          ]),
        ]),
      ),
    );
  }
}

/// The use case of a world: one knob, 'Driven'.
WidgetbookUseCase weUseCase(WorldDef d) => WidgetbookUseCase(
      name: d.name,
      builder: (c) => WStory(def: d, driven: c.knobs.object.dropdown<Driven>(label: 'Driven', options: Driven.values, initialOption: Driven.off, labelBuilder: (m) => switch (m) { Driven.off => 'Off', Driven.wiggle => 'Wiggle', Driven.pulse => 'Pulse' })),
    );
