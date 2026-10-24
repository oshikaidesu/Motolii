// Part of media_set.dart (cut there by size, as it was).
part of 'media_set.dart';

Widget _detailsCase(BuildContext c) {
  final kind = _kindKnob(c, Kind.video);
  final base = _byKind(kind);
  final name = c.knobs.string(label: 'Name', initialValue: base.name);
  final tags = c.knobs.boolean(label: 'Show tags', initialValue: true);
  final a = Asset(name, base.kind, base.seed, style: base.style, dur: base.dur, tags: base.tags, dims: base.dims, size: base.size);
  final extra = switch (kind) {
    Kind.image => [('Colour space', 'sRGB'), ('Alpha', 'No')],
    Kind.video => [('Frame rate', '29.97 fps'), ('Codec', 'H.264')],
    Kind.audio => [('Channels', 'Stereo'), ('Bit depth', '24-bit')],
    Kind.model => [('Format', 'glTF 2.0'), ('Materials', '3')],
    Kind.pano => [('Projection', 'Equirect'), ('Frame rate', '30 fps')],
  };
  final rows = [('Type', kind.label), (kind == Kind.audio ? 'Sample rate' : 'Dimensions', a.dims), if (a.dur > 0) ('Duration', clock(a.dur)), ...extra, ('Size', a.size), ('Modified', '12 Sep 2026')];
  return Hold<bool>(
    initial: false,
    builder: (context, fav, set) => Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
      child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        AspectRatio(
          aspectRatio: 16 / 10,
          child: Container(
            foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
            child: ClipRRect(borderRadius: BorderRadius.circular(6), child: Stack(fit: StackFit.expand, children: [
              Art(a),
              if (kind != Kind.image) Positioned(left: 6, top: 6, child: TypeBadge(kind)),
              if (a.dur > 0) Positioned(right: 6, bottom: 6, child: TimePill(clock(a.dur))),
            ])),
          ),
        ),
        const SizedBox(height: 12),
        Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Expanded(child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
          StarButton(on: fav, onChanged: set),
        ]),
        const SizedBox(height: 4),
        Text('Project / Footage / Day 1', style: T.label(N.g56)),
        const SizedBox(height: 12),
        for (var i = 0; i < rows.length; i++)
          Container(
            height: 24,
            decoration: BoxDecoration(border: Border(top: BorderSide(color: i == 0 ? N.g20 : N.rowLine))),
            child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Text(rows[i].$1, style: T.label(N.g56)),
              const Spacer(),
              Text(rows[i].$2, style: T.value(N.g91)),
            ]),
          ),
        if (tags) ...[
          const SizedBox(height: 12),
          Wrap(spacing: 4, runSpacing: 4, children: [
            for (final t in a.tags) Pill(t),
            Press(
              builder: (context, hover, focus, _) => Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
                decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: focus ? N.g56 : (hover ? N.g26 : N.g20))),
                child: Text('+ Tag', style: T.micro(hover ? N.g91 : N.g56)),
              ),
            ),
          ]),
        ],
        const SizedBox(height: 12),
        Flex(direction: Axis.horizontal, mainAxisAlignment: MainAxisAlignment.start, children: [QuietButton('Add to project', primary: true), const SizedBox(width: 8), const QuietButton('Reveal')]),
      ]),
    ),
  );
}
