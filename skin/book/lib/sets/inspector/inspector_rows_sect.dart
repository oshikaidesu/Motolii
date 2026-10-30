// A group: a heading and its rows. How groups are told apart is the Editorial knob: A a rule above, B a grey step, C a rule under a large heading.
import 'package:flutter/services.dart' show HardwareKeyboard;
import 'package:flutter/widgets.dart';
import 'package:lab_book/forms/forms.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

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
              height: Shape.z(24),
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
                      padding: EdgeInsets.only(right: Shape.s(8)),
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
            padding: EdgeInsets.fromLTRB(Shape.s(8), Shape.s(4), Shape.s(8), Shape.s(6)),
            decoration: BoxDecoration(color: Grey.g13, borderRadius: BorderRadius.all(Shape.r(6))),
            child: col([?head, ...body]),
          ),
        );
      case Editorial.c:
        return Padding(
          padding: EdgeInsets.only(top: first ? Shape.s(0) : ed.gap),
          child: col([
            if (head != null) ...[head, SizedBox(height: Shape.s(2)), const Rule(), SizedBox(height: Shape.s(6))],
            ...body,
          ]),
        );
    }
  }
}

class _PopCard extends StatelessWidget {
  const _PopCard(this.s);
  final Sect s;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, title = s.title, tone = s.tone;
    final open = s.onToggle != null ? s.open : (title == null || !(doc.b['fold.$title'] ?? false));
    final toggle = s.onToggle ?? (title == null ? null : () {
      if (!HardwareKeyboard.instance.isAltPressed) return doc.flag('fold.$title', open);
      for (final c in doc.cards) doc.flag('fold.${c.name}', open);   // (the Mac's Option on a disclosure triangle: all of them, Finder's list; this one as they go)
      doc.flag('fold.$title', open);
    });
    final keys = s.keyIds.map((i) => doc.keys[i] ?? KeyS.off).fold(KeyS.off, (a, b) => b.index > a.index ? b : a);
    final chips = s.brief.isNotEmpty ? s.brief : [if (s.hint != null && s.hint!.isNotEmpty) s.hint!];
    final brief = !open && chips.isNotEmpty;
    List<Widget> lead(bool h) => [
      // the mark sits centred in its box, and the gap after it equals the card's inset, so it has the same air on both sides
      if (toggle != null) ...[FoldMark(open: open, hot: h), SizedBox(width: Pop.inset)],
      if (s.mark != null && tone != null) ...[ToneBadge(s.mark!, tone, dim: s.dim), SizedBox(width: Shape.s(6))],
      if (title != null)
        Flexible(
          child: Text(
            title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: T
                .title(s.dim ? Grey.g56 : (h && toggle != null ? Grey.g100 : Grey.g95))
                .copyWith(fontSize: T.px(12), fontWeight: FontWeight.w700, decoration: s.dim ? TextDecoration.lineThrough : null),
          ),
        ),
      if (!open && keys != KeyS.off)
        Container(
          width: Shape.z(5),
          height: Shape.z(5),
          margin: EdgeInsets.only(left: Shape.s(6)),
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
                      width: briefX,
                      child: Row(children: lead(h)),
                    )
                  else
                    ...lead(h),
                  Expanded(
                    child: brief
                        ? Padding(
                            padding: EdgeInsets.only(right: Shape.s(8)),
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
                                    for (final (i, t) in fit.indexed) ...[if (i > 0) SizedBox(width: Shape.s(4)), BriefChip(t, dim: s.dim)],
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
      padding: EdgeInsets.symmetric(horizontal: Pop.inset, vertical: Pop.insetY),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: [
          if (head != null) s.headWrap?.call(head) ?? head,
          if (head != null && body.isNotEmpty) SizedBox(height: Pop.titleGap),
          ...body,
        ],
      ),
    );
    return Padding(
      padding: EdgeInsets.only(bottom: Pop.gap),
      child: Forms.worn.card( // (as the skin's forms draw a card)
        context,
        CardSpec(title: title, tone: tone, open: open, onToggle: toggle, trailing: s.trailing, body: body, head: head, own: content),
      ),
    );
  }
}
