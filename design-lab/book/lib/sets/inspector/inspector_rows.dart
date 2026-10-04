part of 'inspector_parts.dart';

// ---- 5. sections and rows -------------------------------------------------------------------------------------------
class Rule extends StatelessWidget {
  const Rule({super.key});
  @override
  Widget build(BuildContext context) => Container(height: 1, color: Ed(Ctx.of(context).cfg.ed).rule);
}

/// A group: a heading and its rows. How groups are told apart is the Editorial knob: A a rule above, B a grey step, C a rule under a large heading.
class Sect extends StatelessWidget {
  const Sect({
    super.key,
    this.title,
    this.children = const [],
    this.first = false,
    this.trailing,
    this.open = true,
    this.onToggle,
    this.hint,
    this.dim = false,
    this.sub = false,
    this.tone,
    this.mark,
    this.brief = const [],
    this.keyIds = const [],
    this.headWrap,
  });
  final String? title, hint;
  final List<Widget> children;
  final bool first, open, dim, sub;
  final Widget? trailing;
  final VoidCallback? onToggle;

  /// Pop only: the card's colour stripe, and the rows whose keys light the card's key dot while it is folded.
  final Color? tone;
  final CardMark? mark;
  final List<String> keyIds;

  /// Pop only: the values a folded card shows, one chip each (falls back to [hint]).
  final List<String> brief;

  /// Pop only: wraps the card's head, e.g. to make it a drag handle.
  final Widget Function(Widget head)? headWrap;

  @override
  Widget build(BuildContext context) {
    final ed = Ed(Ctx.of(context).cfg.ed);
    final head = (title == null && trailing == null)
        ? null
        : GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: onToggle,
            child: SizedBox(
              height: 24,
              child: Row(
                children: [
                  if (title != null)
                    Expanded(
                      child: Text(sub ? title! : ed.head(title!), maxLines: 1, overflow: TextOverflow.ellipsis, style: sub ? ed.subStyle() : ed.headStyle()),
                    )
                  else
                    const Spacer(),
                  if (!open && hint != null)
                    Padding(
                      padding: const EdgeInsets.only(right: 8),
                      child: Text(hint!, style: T.label(Grey.g56)),
                    ),
                  ?trailing,
                ],
              ),
            ),
          );
    final body = [if (open) ...children];
    Widget col(List<Widget> kids) => Opacity(
      opacity: dim ? .45 : 1,
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: kids),
    );
    if (Pop.on(context)) return _PopCard(this);
    switch (ed.e) {
      case Editorial.a:
        return col([
          if (!first)
            Padding(
              padding: EdgeInsets.only(top: ed.gap),
              child: const Rule(),
            ),
          SizedBox(height: first ? 0 : 4),
          ?head,
          ...body,
        ]);
      case Editorial.b:
        return Padding(
          padding: EdgeInsets.only(bottom: ed.gap),
          child: Container(
            padding: const EdgeInsets.fromLTRB(8, 4, 8, 6),
            decoration: BoxDecoration(color: Grey.g13, borderRadius: BorderRadius.circular(6)),
            child: col([?head, ...body]),
          ),
        );
      case Editorial.c:
        return Padding(
          padding: EdgeInsets.only(top: first ? 0 : ed.gap),
          child: col([
            if (head != null) ...[head, const SizedBox(height: 2), const Rule(), const SizedBox(height: 6)],
            ...body,
          ]),
        );
    }
  }
}

/// A card's fold mark: points right when folded, down when open, so a card that can fold says so before it is touched.
class FoldMark extends StatelessWidget {
  const FoldMark({super.key, required this.open, this.hot = false});
  final bool open, hot;
  @override
  Widget build(BuildContext context) => AnimatedRotation(
    turns: open ? .25 : 0,
    duration: const Duration(milliseconds: 120),
    child: CustomPaint(size: const Size(8, 8), painter: _FoldPaint(hot ? Grey.g95 : Grey.g56)),
  );
}

class _FoldPaint extends CustomPainter {
  const _FoldPaint(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) => c.drawPath(
    Path()
      ..moveTo(s.width * .3, s.height * .12)
      ..lineTo(s.width * .7, s.height * .5)
      ..lineTo(s.width * .3, s.height * .88),
    Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.3
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round,
  );
  @override
  bool shouldRepaint(_FoldPaint o) => o.ink != ink;
}

/// The Pop card behind [Sect]: every titled card folds (its own toggle, or a per-title flag), untitled ones never do.
/// Folded, a card is one line that still says what it holds: its colour, its name, a key dot if anything in it is animated, and its values in brief.
/// Folded summaries all start here, so a closed stack reads as a two-column list: what, then how much.
const _briefX = 136.0;

class _PopCard extends StatelessWidget {
  const _PopCard(this.s);
  final Sect s;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, title = s.title, tone = s.tone;
    final open = s.onToggle != null ? s.open : (title == null || !(doc.b['fold.$title'] ?? false));
    final toggle = s.onToggle ?? (title == null ? null : () => doc.flag('fold.$title', open));
    final keys = s.keyIds.map((i) => doc.keys[i] ?? KeyS.off).fold(KeyS.off, (a, b) => b.index > a.index ? b : a);
    final chips = s.brief.isNotEmpty ? s.brief : [if (s.hint != null && s.hint!.isNotEmpty) s.hint!];
    final brief = !open && chips.isNotEmpty;
    List<Widget> lead(bool h) => [
      // the mark sits centred in its box, and the gap after it equals the card's inset, so it has the same air on both sides
      if (toggle != null) ...[FoldMark(open: open, hot: h), const SizedBox(width: Pop.inset)],
      if (s.mark != null && tone != null) ...[ToneBadge(s.mark!, tone, dim: s.dim), const SizedBox(width: 6)],
      if (title != null)
        Flexible(
          child: Text(
            title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: T
                .title(s.dim ? Grey.g56 : (h && toggle != null ? Grey.g100 : Grey.g95))
                .copyWith(fontSize: 12, fontWeight: FontWeight.w700, decoration: s.dim ? TextDecoration.lineThrough : null),
          ),
        ),
      if (!open && keys != KeyS.off)
        Container(
          width: 5,
          height: 5,
          margin: const EdgeInsets.only(left: 6),
          decoration: BoxDecoration(color: keys == KeyS.at ? Pop.keyDot : Pop.keyDot.withValues(alpha: .5), shape: BoxShape.circle),
        ),
    ];
    final head = title == null && s.trailing == null
        ? null
        : Hov(
            onTap: toggle,
            builder: (_, h) => SizedBox(
              height: Pop.titleH,
              child: Row(
                children: [
                  if (brief)
                    SizedBox(
                      width: _briefX,
                      child: Row(children: lead(h)),
                    )
                  else
                    ...lead(h),
                  Expanded(
                    child: brief
                        ? Padding(
                            padding: const EdgeInsets.only(right: 8),
                            child: LayoutBuilder(
                              builder: (context, box) {
                                var used = 0.0;
                                final fit = <String>[];
                                for (final t in chips) {
                                  final w = BriefChip.width(t) + (fit.isEmpty ? 0 : 4);
                                  if (used + w > box.maxWidth) break;
                                  used += w;
                                  fit.add(t);
                                }
                                if (fit.isEmpty) {
                                  return Row(
                                    children: [Flexible(child: BriefChip(chips.first, dim: s.dim))],
                                  );
                                }
                                return Row(
                                  children: [
                                    for (final (i, t) in fit.indexed) ...[if (i > 0) const SizedBox(width: 4), BriefChip(t, dim: s.dim)],
                                  ],
                                );
                              },
                            ),
                          )
                        : const SizedBox(),
                  ),
                  ?s.trailing,
                ],
              ),
            ),
          );
    final body = open ? s.children : const <Widget>[];
    final content = Padding(
      padding: const EdgeInsets.symmetric(horizontal: Pop.inset, vertical: Pop.insetY),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: [
          if (head != null) s.headWrap?.call(head) ?? head,
          if (head != null && body.isNotEmpty) const SizedBox(height: Pop.titleGap),
          ...body,
        ],
      ),
    );
    return Padding(
      padding: const EdgeInsets.only(bottom: Pop.gap),
      child: ColoredBox(color: Pop.card, child: content),
    );
  }
}

bool _fits(String s, TextStyle style, double w) =>
    (TextPainter(
      text: TextSpan(text: s, style: style),
      maxLines: 1,
      textDirection: TextDirection.ltr,
    )..layout()).width <=
    w;

class _RowInfo extends InheritedWidget {
  const _RowInfo({required this.unit, required this.keys, required super.child});
  final String? unit;
  final KeyS keys;
  static _RowInfo? of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_RowInfo>();
  @override
  bool updateShouldNotify(_RowInfo old) => old.unit != unit || old.keys != keys;
}

/// Under it, coordinate fields drop their axis letters: one header names the columns.
class AxisColumns extends InheritedWidget {
  const AxisColumns({super.key, required super.child});
  static bool on(BuildContext c) => c.getInheritedWidgetOfExactType<AxisColumns>() != null;
  @override
  bool updateShouldNotify(AxisColumns old) => false;
}

/// The X / Y / Z column header, laid on the same grid as PropRow's cells.
class AxisHeader extends StatelessWidget {
  const AxisHeader({super.key});
  @override
  Widget build(BuildContext context) {
    final cfg = Ctx.of(context).cfg;
    return LayoutBuilder(
      builder: (context, box) => SizedBox(
        height: 16,
        child: Row(
          children: [
            const SizedBox(width: _labelWPop),
            Expanded(
              child: Row(
                children: [
                  for (final (i, a) in const [(0, 'X'), (1, 'Y'), (2, 'Z')]) ...[
                    if (i > 0) const SizedBox(width: 4),
                    Expanded(
                      child: Text(
                        a,
                        textAlign: TextAlign.center,
                        style: T.micro(Color.lerp(_axes[i], Grey.g100, .3)!).copyWith(fontWeight: FontWeight.w700),
                      ),
                    ),
                  ],
                ],
              ),
            ),
            if (cfg.resetArrow) const SizedBox(width: _resetW),
          ],
        ),
      ),
    );
  }
}

/// A small picture of a row's value, drawn before its label, so the value reads as a shape before it reads as a number.
class RowMark extends InheritedWidget {
  const RowMark({super.key, required this.marks, required super.child});
  final Map<String, Widget> marks;
  static Widget? of(BuildContext context, String id) => context.dependOnInheritedWidgetOfExactType<RowMark>()?.marks[id];
  static bool has(BuildContext context, String id) => context.getInheritedWidgetOfExactType<RowMark>()?.marks.containsKey(id) ?? false;
  @override
  bool updateShouldNotify(RowMark old) => !mapEquals(old.marks, marks);
}

/// One property: label, its value cells, the key mark, the reset mark. The columns are the same on every row (B1) so X/Y/Z line up.
/// contract: right-click a row for Key this frame / Remove key / Remove animation; unavailable ones say why (D3).
class PropRow extends StatefulWidget {
  const PropRow({
    super.key,
    required this.id,
    required this.label,
    required this.cells,
    this.ids,
    this.keyable = true,
    this.enabled = true,
    this.under,
    this.lead,
    this.trail,
    this.note,
    this.rel,
    this.relName,
  });
  final String id, label;
  final List<Widget> cells;
  final List<String>? ids;
  final bool keyable, enabled;
  final Widget? under, lead, trail;
  final String? note, relName;
  final Fam? rel;
  @override
  State<PropRow> createState() => _PropRowState();
}

class _PropRowState extends State<PropRow> {
  bool _menu = false, _was = false;

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), cfg = x.cfg, doc = x.doc, ed = Ed(cfg.ed);
    final ids = widget.ids ?? [widget.id];
    final changed = ids.any((i) => doc.changed(i) && (Role.grey || doc.get(i) != null)),
        mixedAny = ids.any((i) => doc.get(i) == null),
        ks = doc.keys[widget.id] ?? KeyS.off;
    final active = doc.revealed == widget.id || doc.directHover == widget.id;
    final mark = widget.lead ?? RowMark.of(context, widget.id);
    final pop = Pop.on(context);
    final units = widget.cells.whereType<NumField>().map((f) => f.unit).toSet();
    final shared = pop && units.length == 1 ? units.single : null;
    final unit = shared != null && _fits('${ed.lab(widget.label)} $shared', ed.labStyle(Grey.g63), _labelWPop - 34 - 16) ? shared : null;
    final revealed = doc.revealed == widget.id;
    if (revealed && !_was) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) Scrollable.ensureVisible(context, alignment: .2, duration: const Duration(milliseconds: 160), curve: Curves.easeOut);
      });
    }
    _was = revealed;
    final locked = cfg.locked;
    final showRel = cfg.maybe && widget.rel != null;
    final cellsOn = widget.enabled && !showRel && !locked;
    return LayoutBuilder(
      builder: (context, box) {
        final lw = pop ? _labelWPop : (box.maxWidth < 300 ? _labelWNarrow : _labelW);
        final slots = (pop ? 0 : _keyW) + (cfg.resetArrow ? _resetW : 0);
        final cellsW = box.maxWidth - lw - slots - 8;
        final stack = widget.cells.length == 3 && (cellsW - 8) / 3 < (pop ? 56 : _stack3);
        Widget gap(double w) => SizedBox(width: w);
        Widget cellsRow(List<Widget> cs) => Row(
          children: [
            for (var i = 0; i < cs.length; i++) ...[if (i > 0) gap(4), Expanded(child: cs[i])],
          ],
        );
        final cells = stack
            ? Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  cellsRow(widget.cells.take(2).toList()),
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      Expanded(child: widget.cells[2]),
                      gap(4),
                      const Expanded(child: SizedBox()),
                    ],
                  ),
                ],
              )
            : cellsRow(widget.cells);
        final label = Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                if (mark != null)
                  Padding(
                    padding: const EdgeInsets.only(right: 6),
                    child: Opacity(
                      opacity: cellsOn ? 1 : .4,
                      child: pop
                          ? Container(
                              width: 28,
                              height: Pop.cell,
                              padding: const EdgeInsets.all(2),
                              decoration: BoxDecoration(color: active ? Grey.g20 : null, borderRadius: BorderRadius.circular(7)),
                              child: FittedBox(child: mark),
                            )
                          : mark,
                    ),
                  )
                else if (pop)
                  const SizedBox(width: 34),
                (unit != null ? Expanded.new : Flexible.new)(
                  child: Text(
                    ed.lab(widget.label),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: ed.labStyle(
                      !cellsOn ? Grey.g56 : (changed ? Role.of(Grey.g95, Role.changed) : (mixedAny ? Role.of(Grey.g63, Role.info) : Grey.g63)),
                    ),
                  ),
                ),
                ?widget.trail,
                if (unit != null) ...[
                  Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: Text(unit, style: T.label(Grey.g44)),
                  ),
                ],
                if (showRel)
                  Padding(
                    padding: const EdgeInsets.only(left: 4),
                    child: Container(
                      width: 5,
                      height: 5,
                      decoration: BoxDecoration(color: widget.rel!.c, shape: BoxShape.circle),
                    ),
                  ),
              ],
            ),
            if (widget.under != null) Padding(padding: const EdgeInsets.only(top: 4, bottom: 2), child: widget.under),
          ],
        );
        Widget item(String s, bool ok, String why, VoidCallback go) => Hov(
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
            child: Row(
              children: [
                Text(s, style: T.name(ok ? Grey.g91 : Grey.g56)),
                const Spacer(),
                if (!ok) Text(why, style: T.label(Grey.g56)),
              ],
            ),
          ),
        );
        return Listener(
          behavior: HitTestBehavior.translucent,
          onPointerDown: (e) {
            if (e.buttons == kSecondaryButton && widget.keyable) setState(() => _menu = !_menu);
          },
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Container(
                constraints: BoxConstraints(minHeight: pop ? Pop.row : ed.row),
                padding: const EdgeInsets.symmetric(vertical: 1),
                decoration: pop
                    ? BoxDecoration(color: active ? Grey.g15 : null, borderRadius: BorderRadius.circular(6))
                    : BoxDecoration(
                        color: active ? Grey.g20 : null,
                        border: active ? Border(left: BorderSide(color: Grey.g95, width: 2)) : null,
                      ),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    SizedBox(width: lw, child: label),
                    Expanded(
                      child: RowOff(
                        off: !cellsOn,
                        narrow: box.maxWidth < 300,
                        child: IgnorePointer(
                          ignoring: !cellsOn,
                          child: pop ? _RowInfo(unit: unit, keys: widget.keyable ? ks : KeyS.off, child: cells) : cells,
                        ),
                      ),
                    ),
                    // the key column is reserved on every row so every field ends at one edge
                    if (pop)
                      const SizedBox.shrink()
                    else if (widget.keyable)
                      KeyMark(
                        state: ks,
                        enabled: !locked,
                        // contract: click = key at the playhead; on a key here it removes that key (the track stays animated).
                        onTap: () => doc.setKey(widget.id, ks == KeyS.at ? KeyS.anim : KeyS.at),
                      )
                    else
                      const SizedBox(width: _keyW),
                    if (cfg.resetArrow) ResetMark(show: (changed || mixedAny) && !locked, onTap: () => doc.resetIds(ids)),
                  ],
                ),
              ),
              if (widget.note != null)
                Padding(
                  padding: EdgeInsets.only(left: lw, bottom: 2),
                  child: Text(widget.note!, style: T.label(Grey.g56)),
                ),
              if (showRel)
                Padding(
                  padding: EdgeInsets.only(left: lw, bottom: 2),
                  child: Text(
                    'Driven by ${widget.rel!.name}${widget.relName != null ? ' · ${widget.relName}' : ''}. Edit it in Relations.',
                    style: T.label(Role.linkedFor(widget.rel)),
                  ),
                ),
              if (_menu)
                Container(
                  margin: const EdgeInsets.only(left: 0, top: 2, bottom: 4),
                  decoration: BoxDecoration(
                    color: Grey.g13,
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: ed.rule),
                  ),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      item('Key this frame', ks != KeyS.at, 'Already keyed', () => doc.setKey(widget.id, KeyS.at)),
                      item('Remove key', ks == KeyS.at, 'No key here', () => doc.setKey(widget.id, KeyS.anim)),
                      item('Remove animation', ks != KeyS.off, 'Not animated', () => doc.setKey(widget.id, KeyS.off)),
                      if (cfg.maybe) item('Relate…', true, '', () {}),
                    ],
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}
