part of 'inspector_parts.dart';

/// Ideas for host features where the panel does the work: small stages inside cards, edited in place, so nothing fights the Stage's gizmos.
/// One column per topic; each idea is labelled with its letter and its pitch.
class IdeaGallery extends StatelessWidget {
  const IdeaGallery({super.key});

  static const columns = <(String, Widget)>[
    ('Text', TextIdeas()),
    ('Path operations', PathIdeas()),
    ('Layout (CSS)', LayoutIdeas()),
    ('Placement and anchor', PlacementIdeas()),
    ('Masks, time, links', LayerIdeas()),
  ];

  @override
  Widget build(BuildContext context) {
    final board = Pop(
      child: Container(
        color: Grey.g07,
        padding: const EdgeInsets.all(Pop.gap),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final (title, ideas) in columns)
              Padding(
                padding: const EdgeInsets.only(right: 24),
                child: SizedBox(
                  width: 372,
                  child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [PopGroupLabel(title), ideas]),
                ),
              ),
          ],
        ),
      ),
    );
    return Overlay.maybeOf(context) == null ? Overlay.wrap(child: board) : board;
  }
}

/// The label over one idea: its letter, a short name, and one line on what it solves.
class IdeaLabel extends StatelessWidget {
  const IdeaLabel(this.letter, this.name, this.pitch, {super.key});
  final String letter, name, pitch;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(Pop.inset, 14, Pop.inset, 6),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Container(
              width: 18,
              height: 18,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: Pop.accent, borderRadius: BorderRadius.circular(4)),
              child: Text(letter, style: T.label(Pop.onAccent).copyWith(fontWeight: FontWeight.w800, height: 1)),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(name, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
            ),
          ],
        ),
        const SizedBox(height: 4),
        Text(pitch, style: T.label(Grey.g63).copyWith(height: 1.35)),
      ],
    ),
  );
}
