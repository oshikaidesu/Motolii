part of 'inspector_parts.dart';

/// Every host feature (no shader) as Inspector cards, one panel-wide column per family, side by side for review.
/// Each column is what a layer carrying those features would show under its Transform.
class HostGallery extends StatelessWidget {
  const HostGallery({super.key});

  static const columns = <(String, List<Widget>)>[
    ('Placement', [RepeaterCard(), MirrorCard(), BlobTrackCard(), MotionBlurCard()]),
    ('Path operations', [PathOpsStack()]),
    ('Text', [TextAnimatorCard(), TextMorphCard(), ParagraphCard()]),
    ('Layer', [MasksCard(), TimeCard(), HostLayoutCard(), LinksCard()]),
    ('Particles', [ParticlesCard()]),
    ('Scene', [SceneCard(), HostCameraCard(), ExtrudeCard(), BevelCard(), OverlayCard()]),
  ];

  @override
  Widget build(BuildContext context) {
    final panel = Pop(
      child: Container(
        color: Grey.g07,
        padding: const EdgeInsets.all(Pop.gap),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final (title, cards) in columns)
              Padding(
                padding: const EdgeInsets.only(right: 16),
                child: SizedBox(
                  width: 372,
                  child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [PopGroupLabel(title), ...cards]),
                ),
              ),
          ],
        ),
      ),
    );
    return Overlay.maybeOf(context) == null ? Overlay.wrap(child: panel) : panel;
  }
}
