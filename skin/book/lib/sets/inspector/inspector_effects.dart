part of 'inspector_parts.dart';

// ---- 10. effects ------------------------------------------------------------------------------------------------------
/// An effect as a plugin ships it: a name and a WGSL uniform struct. Its rows are whatever [readParams] makes of the struct.
class Fx {
  Fx(this.id, this.name, this.wgsl, {required this.tone, required this.mark, this.vendor});
  final String id, name, wgsl;
  final String? vendor;
  final Color tone;
  final CardMark mark;
  late final List<Decl> decls = parseWgsl(wgsl);
  late final List<Param> params = [for (final d in decls) infer(d)];
}

final kEffects = [
  Fx('blur', 'Blur', tone: const Color(0xFF5BC0EB), mark: CardMark.blur, '''
struct Params {
  /// @range(0, 200) @default(12)
  radius: f32,
  /// @range(-180, 180)
  angle: f32,
  /// @range(0, 1) @default(1)
  mix: f32,
  /// @enum(Low, Med, High) @default(1)
  quality: u32,
  // @advanced
  /// @toggle @default(1) @label(Repeat edges)
  edge: u32,
  /// @range(0.1, 4) @default(1)
  gamma: f32,
  /// @range(-100, 100) @unit(px)
  offset: vec2f,
}
'''),
  // a third-party shader nobody wrote rows for: camelCase names, radians, a uv point, a size, a long enum
  Fx('kaleido', 'Kaleido', vendor: 'Lumen FX', tone: const Color(0xFFB8A1FF), mark: CardMark.shader, '''
struct KaleidoUniforms {
  segments: i32,      // @range(2, 12) @default(6)
  twist: f32,         // @range(-3.14159, 3.14159)
  center: vec2f,      // @default(0.5, 0.5)
  tileSize: vec2f,    // @range(8, 512) @default(64, 48)
  driftY: f32,        // @range(-1, 1)
  strength: f32,      // @range(0, 1) @default(0.8)
  // @advanced
  zoom: f32,          // @range(0.25, 4) @default(1)
  blendMode: u32,     // @enum(Normal, Add, Screen, Multiply, Overlay, Difference)
  edgeColor: vec4f,   // @default(0.1, 0.1, 0.12, 1)
  mirror: u32,
  seed: u32,
  time: f32,
}
'''),
  Fx('glow', 'Glow', tone: const Color(0xFFFFB86B), mark: CardMark.glow, '''
struct Params {
  /// @range(0, 1) @default(0.6)
  thr: f32,
  /// @range(0, 300) @default(40)
  size: f32,
  /// @range(0, 400) @unit(%) @default(100)
  gain: f32,
  /// @range(-50, 50) @unit(px)
  lift: f32,
  // @advanced
  /// @range(0, 1) @default(0.2)
  knee: f32,
  /// @default(1, 0.85, 0.6, 1)
  tint: vec4f,
  /// @range(0.1, 4) @default(1)
  aspect: f32,
  /// @range(0, 1) @default(0.5)
  falloff: f32,
  /// @default(1)
  additive: u32,
}
'''),
  Fx('color', 'Colour', tone: const Color(0xFFE974AB), mark: CardMark.colour, '''
struct Params {
  /// @range(-4, 4)
  exp: f32,
  /// @range(-100, 100)
  con: f32,
  /// @range(0, 200) @unit(%) @default(100)
  sat: f32,
  hue: f32,
  // @advanced
  /// @range(-100, 100)
  temp: f32,
  /// @range(-100, 100)
  tintc: f32,
  /// @range(0.1, 4) @default(1)
  gam: f32,
  clampc: u32,
}
'''),
  Fx('warp', 'Distort', tone: const Color(0xFF7DD5B1), mark: CardMark.distort, '''
struct Params {
  /// @range(-100, 100) @unit(%) @default(20)
  amt: f32,
  /// @range(0, 40) @default(6)
  freq: f32,
  /// @range(-10, 10) @default(1)
  speed: f32,
  // @advanced
  /// @range(0, 999)
  seed: u32,
  /// @range(1, 8) @default(2)
  oct: i32,
  /// @range(1, 4) @default(2)
  lac: f32,
  /// @range(0, 1) @default(0.5)
  rough: f32,
  pin: u32,
}
'''),
];

class EffectCard extends StatefulWidget {
  const EffectCard(this.fx, {super.key, this.first = false});
  final Fx fx;
  final bool first;
  @override
  State<EffectCard> createState() => _EffectCardState();
}

class _EffectCardState extends State<EffectCard> {
  bool _menu = false;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), fx = widget.fx, k = fx.id;
    final enabled = doc.b['$k.on'] ?? true, open = doc.b['$k.open'] ?? true, adv = doc.b['$k.adv'] ?? false;
    final q = doc.query.toLowerCase();
    final shown = [
      for (final p in fx.params)
        if (q.isEmpty || p.label.toLowerCase().contains(q)) p,
    ];
    if (q.isNotEmpty && shown.isEmpty) return const SizedBox.shrink();
    final heroes = [
          for (final p in shown)
            if (!p.advanced || q.isNotEmpty) p,
        ],
        rest = [
          for (final p in shown)
            if (p.advanced && q.isEmpty) p,
        ];
    final i = doc.fxOrder.indexOf(k), last = doc.fxOrder.length - 1;
    final locked = x.cfg.locked;
    List<Widget> pairs(List<Param> ps) => [ParamGrid(k, ps, enabled: enabled)];

    Widget mi(String s, bool ok, VoidCallback go, {bool danger = false}) => Hov(
      cursor: ok ? SystemMouseCursors.click : SystemMouseCursors.basic,
      onTap: ok && !locked
          ? () {
              go();
              setState(() => _menu = false);
            }
          : null,
      builder: (_, h) => Container(
        height: 22,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        color: h && ok ? Grey.g15 : null,
        alignment: Alignment.centerLeft,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (danger && ok) ...[const ErrMark(size: 10, gap: 5)],
            Text(s, style: T.name(!ok ? Grey.g56 : (danger ? Role.error : Grey.g91))),
          ],
        ),
      ),
    );
    final pop = Pop.on(context);
    final card = Sect(
      title: fx.name,
      sub: true,
      first: widget.first,
      open: open,
      tone: fx.tone,
      mark: fx.mark,
      // contract: hold the head and drag to reorder; the line shows where it lands. More keeps Move earlier / later for the keyboard.
      headWrap: locked || !pop
          ? null
          : (head) => LongPressDraggable<String>(
              data: k,
              delay: const Duration(milliseconds: 180),
              axis: Axis.vertical,
              feedback: _DragGhost(fx),
              childWhenDragging: Opacity(opacity: .35, child: head),
              child: head,
            ),
      dim: !enabled && Pop.on(context),
      keyIds: [for (final p in fx.params) '${k}_${p.id}'],
      brief: [
        for (final p in fx.params.where((p) => !p.advanced && p.type == WType.f32 && p.look != ParamLook.point && p.look != ParamLook.size).take(2))
          '${p.label} ${brief(doc.get('${k}_${p.id}'), p.dec)}${p.unit}',
      ],
      // contract: clicking the name folds; the switch bypasses (the body dims); More holds order, reset, randomise, remove (F1, F5).
      onToggle: () => doc.flag('$k.open', !open),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (open)
            Hov(
              onTap: () => setState(() => _menu = !_menu),
              builder: (_, h) => Padding(
                padding: const EdgeInsets.only(right: 12),
                child: Text('More', style: T.label(_menu || h ? Grey.g95 : Grey.g56)),
              ),
            ),
          OnOff(on: enabled, onChanged: (v) => doc.flag('$k.on', v)),
        ],
      ),
      children: [
        if (_menu)
          Container(
            margin: const EdgeInsets.only(bottom: 4),
            decoration: BoxDecoration(
              color: Grey.g13,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: ed.rule),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                mi('Move earlier', i > 0, () => doc.moveFx(k, -1)),
                mi('Move later', i < last, () => doc.moveFx(k, 1)),
                mi('Reset to defaults', true, () => doc.resetIds([for (final p in fx.params) ...paramDefaults(k, p).keys])),
                mi('Randomise values', true, () => doc.randomFx(fx)),
                mi('Remove', true, () => doc.removeFx(k), danger: true),
              ],
            ),
          ),
        Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: pairs(heroes)),
        if (rest.isNotEmpty) ...[
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Hov(
              onTap: () => doc.flag('$k.adv', !adv),
              builder: (_, h) => Container(
                height: 24,
                decoration: BoxDecoration(
                  border: Border(top: BorderSide(color: ed.rule)),
                ),
                child: Row(
                  children: [
                    Text('Advanced', style: ed.labStyle(h || adv ? Grey.g95 : Grey.g63)),
                    const Spacer(),
                    Text(adv ? 'Fold' : '${rest.length} more', style: T.label(Grey.g56)),
                  ],
                ),
              ),
            ),
          ),
          if (adv || q.isNotEmpty) Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: pairs(rest)),
        ],
      ],
    );
    if (!pop || locked) return card;
    return DragTarget<String>(
      onWillAcceptWithDetails: (d) => d.data != k,
      onAcceptWithDetails: (d) => doc.placeFx(d.data, doc.fxOrder.indexOf(k)),
      builder: (context, cand, _) {
        final from = cand.isEmpty ? -1 : doc.fxOrder.indexOf(cand.first!);
        final below = from >= 0 && from < i;
        return Stack(
          clipBehavior: Clip.none,
          children: [
            card,
            if (from >= 0)
              Positioned(
                left: 0,
                right: 0,
                top: below ? null : -Pop.gap,
                bottom: below ? 0 : null,
                height: 2,
                child: ColoredBox(color: fx.tone),
              ),
          ],
        );
      },
    );
  }
}

/// What follows the pointer while an effect is dragged: the folded head, lifted.
class _DragGhost extends StatelessWidget {
  const _DragGhost(this.fx);
  final Fx fx;
  @override
  Widget build(BuildContext context) => Container(
    width: 220,
    height: Pop.titleH + Pop.insetY * 2,
    padding: const EdgeInsets.symmetric(horizontal: Pop.inset),
    decoration: BoxDecoration(
      color: Grey.g20,
      boxShadow: [BoxShadow(color: Color(0x99000000), blurRadius: 14, offset: Offset(0, 6))],
    ),
    child: Row(
      children: [
        ToneBadge(fx.mark, fx.tone),
        const SizedBox(width: 6),
        Text(fx.name, style: T.title(Grey.g95).copyWith(fontSize: 12, fontWeight: FontWeight.w700)),
      ],
    ),
  );
}

class EffectsStack extends StatelessWidget {
  const EffectsStack({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final cards = [
      for (var i = 0; i < doc.fxOrder.length; i++) EffectCard(kEffects.firstWhere((f) => f.id == doc.fxOrder[i]), key: ValueKey(doc.fxOrder[i]), first: i == 0),
    ];
    final none =
        doc.query.isNotEmpty &&
        !doc.fxOrder.any((id) => kEffects.firstWhere((f) => f.id == id).params.any((p) => p.label.toLowerCase().contains(doc.query.toLowerCase())));
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (x.cfg.maybe) const ParamSearch(),
        if (doc.fxOrder.isEmpty)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Text('No effects. Add one from the Browser.', style: T.label(Grey.g56)),
          ),
        if (none)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Text('No parameter matches', style: T.label(Grey.g56)),
          )
        else
          ...cards,
      ],
    );
  }
}

/// H3 (maybe): search flattens the groups, unfolds Advanced, and says so when nothing matches.
class ParamSearch extends StatefulWidget {
  const ParamSearch({super.key});
  @override
  State<ParamSearch> createState() => _ParamSearchState();
}

class _ParamSearchState extends State<ParamSearch> {
  final _c = TextEditingController();
  final _f = FocusNode(debugLabel: 'param-search');
  @override
  void dispose() {
    _c.dispose();
    _f.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed);
    return Container(
      height: _cellH,
      margin: const EdgeInsets.only(bottom: 4),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      alignment: Alignment.centerLeft,
      decoration: BoxDecoration(
        color: Grey.g13,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: ed.rule),
      ),
      child: Stack(
        alignment: Alignment.centerLeft,
        children: [
          if (_c.text.isEmpty) Text('Search parameters', style: T.label(Grey.g56)),
          EditableText(
            controller: _c,
            focusNode: _f,
            style: T.name(Grey.g95),
            cursorColor: Grey.g95,
            backgroundCursorColor: Grey.g20,
            maxLines: 1,
            onChanged: (s) {
              x.doc.query = s;
              x.doc.poke();
              setState(() {});
            },
          ),
        ],
      ),
    );
  }
}
