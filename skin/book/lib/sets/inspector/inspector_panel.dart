part of 'inspector_parts.dart';

// ---- 13. the whole panel ----------------------------------------------------------------------------------------------
/// The Inspector. Needs an InspHost above it. Tabs or one scrolling column is the Layout knob; P / S / R / T / A reveal their row (B7, I2).
class InspectorPanel extends StatefulWidget {
  const InspectorPanel({super.key, this.width = 372, this.height = 720, this.directTransform = false});
  final double width, height;
  final bool directTransform;
  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> {
  int _tab = 0;
  final _node = FocusNode(debugLabel: 'inspector');
  final _scroll = ScrollController();

  @override
  void dispose() {
    _node.dispose();
    _scroll.dispose();
    super.dispose();
  }

  List<String> _tabs(Kind k) => k == Kind.camera
      ? const ['Camera', 'Relations', 'Effects']
      : (k == Kind.group ? const ['Transform', 'Relations', 'Effects'] : const ['Transform', 'Relations', 'Effects', 'Material']);

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
    setState(() => _tab = 0);
    doc.reveal(id);
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), cfg = x.cfg, doc = x.doc, ed = Ed(cfg.ed), k = doc.kind;
    final tabs = _tabs(k), tab = tabs[_tab.clamp(0, tabs.length - 1)];
    final titled = !cfg.tabs;
    List<Widget> section(String t, {required bool first}) {
      switch (t) {
        case 'Transform':
          return [
            if (widget.directTransform) DirectTransform(titled: titled, first: first) else TransformGroup(titled: titled, first: first),
            if (k == Kind.group && cfg.maybe) const LayoutGroup(),
          ];
        case 'Camera':
          return [CameraGroup(first: first)];
        case 'Relations':
          return [RelationsBlock(titled: titled, first: first)];
        case 'Effects':
          return [if (titled) Sect(title: 'Effects', first: first, children: const []), const EffectsStack()];
        default:
          return [
            if (k == Kind.text) TextGroup(first: first),
            if (k == Kind.shape || k == Kind.text) FillStroke(first: first),
            BlendRows(titled: true, first: first && k != Kind.text && k != Kind.shape),
          ];
      }
    }

    final pop = widget.directTransform;
    final body = <Widget>[
      if (k == Kind.none)
        const EmptyState()
      else ...[
        const InspHeader(),
        if (cfg.locked)
          Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Text('Locked. Values are readable; unlock the layer to edit.', style: T.label(Grey.g56)),
          ),
        if (pop) ...[
          // contract: one stack, After Effects style: transform, material, then effects piled below; every titled card folds on its own.
          const SizedBox(height: Pop.gap),
          const DirectTransform(),
          if (k == Kind.text) const TextGroup(),
          if (k == Kind.shape || k == Kind.text) const FillStroke(),
          const BlendRows(),
          PopGroupLabel('Effects', count: Ctx.of(context).doc.fxOrder.length),
          const EffectsStack(),
        ] else if (cfg.tabs) ...[
          const SizedBox(height: 4),
          Segmented(items: tabs, index: _tab.clamp(0, tabs.length - 1), expand: true, onChanged: (i) => setState(() => _tab = i)),
          SizedBox(height: ed.gap - 4),
          ...section(tab, first: true),
        ] else ...[
          const SizedBox(height: 4),
          for (var i = 0; i < tabs.length; i++) ...section(tabs[i], first: i == 0),
        ],
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
          color: pop ? Grey.g07 : Grey.g10,
          child: SingleChildScrollView(
            controller: _scroll,
            padding: EdgeInsets.all(pop ? Pop.gap : 12),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: body),
          ),
        ),
      ),
    );
    final floated = Overlay.maybeOf(context) == null ? Overlay.wrap(child: panel) : panel;
    return pop ? Pop(child: floated) : floated;
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
