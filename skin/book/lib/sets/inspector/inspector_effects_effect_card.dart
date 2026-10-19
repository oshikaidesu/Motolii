// What follows the pointer while an effect is dragged: the folded head, lifted.
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import 'package:lab_book/tokens.dart';

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
              borderRadius: BorderRadius.all(Shape.r(4)),
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
        Text(fx.name, style: T.title(Grey.g95).copyWith(fontSize: T.px(12), fontWeight: FontWeight.w700)),
      ],
    ),
  );
}
