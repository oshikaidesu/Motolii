import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';

// ---- 13. the whole panel ----------------------------------------------------------------------------------------------
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

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final doc = Ctx.read(context).doc;
    if (doc.kind == Kind.camera || doc.kind == Kind.none) return KeyEventResult.ignored;
    final k = HardwareKeyboard.instance;
    if (k.isShiftPressed || k.isControlPressed || k.isAltPressed || k.isMetaPressed) return KeyEventResult.ignored;
    final f = FocusManager.instance.primaryFocus;
    if (f?.context?.findAncestorWidgetOfExactType<EditableText>() != null) return KeyEventResult.ignored;
    final id = {
      LogicalKeyboardKey.keyP: 'pos',
      LogicalKeyboardKey.keyS: 'scale',
      LogicalKeyboardKey.keyR: 'rot',
      LogicalKeyboardKey.keyT: 'op',
      LogicalKeyboardKey.keyA: 'anchor',
    }[e.logicalKey];
    if (id == null) return KeyEventResult.ignored;
    doc.reveal(id);
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, k = doc.kind;
    final body = <Widget>[
      if (k == Kind.none)
        const EmptyState()
      else ...[
        const InspHeader(),
        // contract: one stack, After Effects style: the host's cards (Transform first), the material, then effects piled below; every titled card folds on its own.
        const SizedBox(height: Pop.gap),
        for (final c in doc.cards) ValuesCard(c), // (the host's own cards)
        PopGroupLabel('Effects', count: Ctx.of(context).doc.fxOrder.length),
        const EffectsStack(),
      ],
    ];
    final panel = Focus(
      focusNode: _node,
      onKeyEvent: _key,
      child: Listener(
        onPointerDown: (_) => WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && !_node.hasFocus) _node.requestFocus();
        }),
        child: Container(
          width: widget.width,
          height: widget.height,
          color: Grey.g07,
          child: SingleChildScrollView(
            controller: _scroll,
            padding: const EdgeInsets.all(Pop.gap),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: body),
          ),
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
        if (count != null) ...[const SizedBox(width: 6), Text('$count', style: T.micro(Grey.g44).copyWith(fontWeight: FontWeight.w700))],
      ],
    ),
  );
}
