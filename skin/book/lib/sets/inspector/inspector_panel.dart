import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../../tokens.dart';
import 'inspector_direct_transform.dart';
import 'inspector_copies.dart';
import 'inspector_light.dart';
import 'inspector_parts.dart';
import 'inspector_look_preview.dart';
import 'inspector_motion_preview.dart';

// ---- 13. the whole panel ----------------------------------------------------------------------------------------------
/// The cards the window knows and draws by hand, by the inside's card name (Transform, Camera, Text, Fill, Blending):
/// their values are the document's own names ('v_x', 'v_opacity'). A card of one's own may be given here too; a card
/// with no hand of its own is drawn from what the inside says of its values ([ValuesCard]).
final inspectorCards = <String, Widget Function(Fx card)>{
  'Transform': (_) => const DirectTransform(),
  'Camera': (_) => const CameraGroup(),
  'Text': (_) => const TextGroup(),
  'Fill': (_) => const FillStroke(),
  'Blending': (_) => const BlendRows(),
  'Light': (c) => ValuesCard(c, top: const LightDrawing()), // (its values as tiles, under a drawing of the light you pull)
  'Copies': (c) => ValuesCard(c, top: const CopiesDrawing()), // (and the copies as they lie)
  'Wave': (c) => ValuesCard(c, side: const MotionPreview('Wave')), // (and a square moving as the thing will, while touched: Good Boy Ninja's Wiggle)
  'Physics': (c) => ValuesCard(c, side: const MotionPreview('Physics')), // (falling and bouncing)
  'Scatter': (c) => ValuesCard(c, side: const MotionPreview('Scatter')), // (drifting)
  'Field': (c) => ValuesCard(c, side: const MotionPreview('Field')), // (its copies, grown and turned as near the field as they lie)
  'Connect': (c) => ValuesCard(c, side: const MotionPreview('Connect')), // (its copies, where they lie)
  // (the cards of how it looks: the thing alone, as the stage draws it)
  'Gradient': (c) => ValuesCard(c, side: const LookPreview()),
  'Type': (c) => ValuesCard(c, side: const LookPreview()),
  'Line': (c) => ValuesCard(c, side: const LookPreview()),
  'Path': (c) => ValuesCard(c, side: const LookPreview()),
  'Deform': (c) => ValuesCard(c, side: const LookPreview()),
  'Surface': (c) => ValuesCard(c, side: const LookPreview()),
  'Glass': (c) => ValuesCard(c, side: const LookPreview()),
  'Shine': (c) => ValuesCard(c, side: const LookPreview()),
  'Trim Paths': (c) => ValuesCard(c, side: const LookPreview()),
  'Gaussian Blur': (c) => ValuesCard(c, side: const LookPreview()),
  'Drop Shadow': (c) => ValuesCard(c, side: const LookPreview()),
  'Glow': (c) => ValuesCard(c, side: const LookPreview()),
  'Duotone': (c) => ValuesCard(c, side: const LookPreview()),
  'Gradient Map': (c) => ValuesCard(c, side: const LookPreview()),
};

/// The Inspector: one scrolling stack of the host's cards. Needs an InspHost above it.
class InspectorPanel extends StatefulWidget {
  const InspectorPanel({super.key, this.width = 372, this.height = 720});
  final double width, height;
  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> {
  final _node = FocusNode(debugLabel: 'inspector');
  final _scroll = ScrollController();

  @override
  void dispose() {
    _node.dispose();
    _scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, k = doc.kind;
    final body = <Widget>[
      if (k == Kind.none)
        const EmptyState()
      else ...[
        const InspHeader(),
        // contract: one stack, After Effects style: the host's cards, Transform first and the effects on it last; every titled card folds on its own.
        const SizedBox(height: Pop.gap),
        for (final c in doc.cards) inspectorCards[c.name]?.call(c) ?? ValuesCard(c), // (a card the window knows, drawn by hand; one it does not (an effect, a script's knobs), drawn from what the inside says)
      ],
    ];
    final panel = Focus(
      focusNode: _node,
      child: Listener(
        onPointerDown: (_) => WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && !_node.hasFocus) _node.requestFocus();
        }),
        child: Container(
          width: widget.width,
          height: widget.height,
          color: Grey.g07,
          // (only the cards in sight are built: a stage with seventy effects on it builds as one with seven)
          child: ListView(controller: _scroll, padding: const EdgeInsets.all(Pop.gap), children: body),
        ),
      ),
    );
    final floated = Overlay.maybeOf(context) == null ? Overlay.wrap(child: panel) : panel;
    return Pop(child: floated);
  }
}

/// A quiet heading over a pile of cards, aligned with the cards' own titles.
class PopGroupLabel extends StatelessWidget {
  const PopGroupLabel(this.text, {super.key, this.count});
  final String text;
  final int? count;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(Pop.inset, Pop.insetY, Pop.inset, Pop.gap),
    child: Row(
      children: [
        Text(text.toUpperCase(), style: T.micro(Grey.g56).copyWith(fontWeight: FontWeight.w700, letterSpacing: .6)),
        if (count != null) ...[SizedBox(width: Shape.s(6)), Text('$count', style: T.micro(Grey.g44).copyWith(fontWeight: FontWeight.w700))],
      ],
    ),
  );
}
