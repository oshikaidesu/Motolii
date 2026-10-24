// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

class _CtlState extends State<_Ctl> {
  bool _h = false, _f = false, _p = false;
  @override
  Widget build(BuildContext context) {
    final off = widget.onTap == null;
    return FocusableActionDetector(
      enabled: !off,
      onShowHoverHighlight: (v) => setState(() => _h = v),
      onShowFocusHighlight: (v) => setState(() => _f = v),
      mouseCursor: off ? SystemMouseCursors.basic : SystemMouseCursors.click,
      actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) {
        widget.onTap?.call();
        return null;
      })},
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onTap,
        onTapDown: off ? null : (_) => setState(() => _p = true),
        onTapUp: (_) => setState(() => _p = false),
        onTapCancel: () => setState(() => _p = false),
        child: widget.builder(context, !off && (widget.hover ?? _h), !off && (widget.focus ?? _f), !off && (widget.pressed ?? _p)),
      ),
    );
  }
}

/// The shared button body: [fill], an optional 1 px [edge] drawn over it, a pressed accent line on the foot.
Widget _ctlBody({required double height, required EdgeInsets pad, required Color fill, required Color edge, required bool pressed, required Widget child}) => ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: height,
        padding: pad,
        color: fill,
        foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: edge)),
        child: Stack(alignment: Alignment.center, children: [
          child,
          Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: pressed ? Role.selected.withValues(alpha: .5) : _clear)),
        ]),
      ),
    );

/// A key mark: 16 px tall, 6 px sides, g20 (g26 when it is the live one).
class _Kbd extends StatelessWidget {
  const _Kbd(this.text, {this.on = false});
  final String text;
  final bool on;
  @override
  Widget build(BuildContext context) => Container(
        height: 16,
        constraints: const BoxConstraints(minWidth: 18),
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(color: on ? N.g26 : N.g20, borderRadius: BorderRadius.circular(3)),
        child: Center(widthFactor: 1, child: Text(text, style: _num(on ? N.g95 : N.g76))),
      );
}

/// A word that acts: g76 at rest, g95 and underlined on hover.
class _Word extends StatelessWidget {
  const _Word(this.text, {this.onTap, this.style, this.hover, this.rule = false});
  final String text;
  final VoidCallback? onTap;
  final TextStyle? style;
  final bool? hover;

  /// A dotted g44 underline at rest: the word is a control (it drops a filter), not a caption.
  final bool rule;
  @override
  Widget build(BuildContext context) => _H(
        onTap: onTap,
        force: hover,
        builder: (context, h) => Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: (style ?? T.label(N.g76)).copyWith(color: h ? N.g95 : (style?.color ?? N.g76), decoration: h || rule ? TextDecoration.underline : TextDecoration.none, decorationStyle: h ? TextDecorationStyle.solid : TextDecorationStyle.dotted, decorationColor: h ? N.g95 : N.g44)),
      );
}

/// The one remove mark: a 14 px multiplication sign centred in a [w] x 20 target (g63 at rest, g95 on hover). In a search field w = 18, the width of the "/" key mark, so both share one centre and one right inset.
class _Cross extends StatelessWidget {
  const _Cross({this.onTap, this.hover, this.w = 16});
  final VoidCallback? onTap;
  final bool? hover;
  final double w;
  @override
  Widget build(BuildContext context) => _H(onTap: onTap, force: hover, builder: (context, h) => Container(width: w, height: 20, alignment: Alignment.center, child: Text('×', style: T.name(h ? N.g95 : N.g63).copyWith(fontSize: 14, height: 1))));
}

/// A 2 px selection tick inset in the left gutter (x 2 to 4, never on the panel edge); everything else in a row starts at x 10.
class _GutterTick extends StatelessWidget {
  const _GutterTick(this.on, {this.height = 16});
  final bool on;
  final double height;
  @override
  Widget build(BuildContext context) => Positioned(left: 2, top: 0, bottom: 0, width: 2, child: Center(child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, width: 2, height: height, decoration: BoxDecoration(color: on ? N.g95 : _clear, borderRadius: BorderRadius.circular(1)))));
}

class _Sec extends StatelessWidget {
  const _Sec(this.title, {this.right, this.pad = const EdgeInsets.fromLTRB(10, 10, 10, 4)});
  final String title;
  final Widget? right;
  final EdgeInsets pad;
  @override
  Widget build(BuildContext context) => Padding(
        padding: pad,
        child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
          Expanded(child: Text(title.toUpperCase(), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g76).copyWith(letterSpacing: .8))),
          ?right,
        ]),
      );
}

/// The foot line: what just happened (one fact, no instruction) and, on the right, the keys that belong to this panel as key marks.
class _Status extends StatelessWidget {
  const _Status(this.text, {this.keys = const []});
  final String text;
  final List<String> keys;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(color: N.g07, border: Border(top: BorderSide(color: N.rowLine))),
        child: Row(children: [
          Expanded(child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
          for (final k in keys) Padding(padding: const EdgeInsets.only(left: 4), child: _Kbd(k)),
        ]),
      );
}

/// A note that stops something: g10 or g13 ground, a 2 px danger bar on the left, 8 px all round.
class _Notice extends StatelessWidget {
  const _Notice(this.text, {this.fill = N.g10});
  final String text;
  final Color fill;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(4), border: Border(left: BorderSide(color: Role.error, width: 2))),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [const ErrMark(size: 10, gap: 6), Expanded(child: Text(text, style: T.label(N.g91).copyWith(height: 1.35)))]),
      );
}

/// The quiet text button of these panels, 24 px: g13 at rest, g20 on hover, a g63 edge when focused by keyboard; onTap null = disabled (g10 ground, g63 word, no hover). [primary] is one grey step up. hover / focus pin a state (Parts).
class _Btn extends StatelessWidget {
  const _Btn(this.text, {this.onTap, this.primary = false, this.hover, this.focus});
  final String text;
  final VoidCallback? onTap;
  final bool primary;
  final bool? hover, focus;
  @override
  Widget build(BuildContext context) {
    final off = onTap == null;
    return Press(
      onTap: onTap,
      cursor: off ? SystemMouseCursors.basic : SystemMouseCursors.click,
      builder: (context, h, f, _) {
        final hv = !off && (hover ?? h), fc = !off && (focus ?? f);
        // Disabled = no fill, a dashed outline instead of a solid edge, a g63 word (shape as well as tone).
        return CustomPaint(
          foregroundPainter: off ? const DashedBox(N.g44, radius: 4) : null,
          child: AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: 24,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(color: off ? N.g10 : (primary ? (hv ? N.g26 : N.g20) : (hv ? N.g20 : N.g13)), borderRadius: BorderRadius.circular(4), border: Border.all(color: off ? _clear : (fc ? N.g63 : (primary ? N.g26 : N.g20)))),
            child: Center(widthFactor: 1, child: Text(text, style: T.label(off ? N.g63 : N.g91).copyWith(fontWeight: FontWeight.w500))),
          ),
        );
      },
    );
  }
}

/// Name with the matched query tokens brighter.
class _Hl extends StatelessWidget {
  const _Hl(this.text, this.tokens, {this.on = false}) : sub = false;

  /// The second line (Japanese name and category): matched letters are bold there too, so a match that lives in the category still shows why the row is listed.
  const _Hl.sub(this.text, this.tokens) : on = false, sub = true;
  final String text;
  final List<String> tokens;
  final bool on, sub;
  @override
  Widget build(BuildContext context) {
    final base = sub ? T.label(N.g76) : T.name(on ? N.g95 : N.g91), hit = sub ? T.label(N.g91).copyWith(fontWeight: FontWeight.w700) : T.name(N.g100).copyWith(fontWeight: FontWeight.w700);
    if (tokens.isEmpty) return Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: sub ? T.label(N.g76) : T.name(on ? N.g95 : N.g91));
    final low = text.toLowerCase(), mark = List<bool>.filled(text.length, false);
    for (final t in tokens) {
      var i = low.indexOf(t);
      while (i >= 0) {
        for (var k = i; k < i + t.length && k < mark.length; k++) {
          mark[k] = true;
        }
        i = low.indexOf(t, i + t.length);
      }
    }
    final spans = <TextSpan>[];
    var k = 0;
    while (k < text.length) {
      var e = k;
      while (e < text.length && mark[e] == mark[k]) {
        e++;
      }
      spans.add(TextSpan(text: text.substring(k, e), style: mark[k] ? hit : base));
      k = e;
    }
    return Text.rich(TextSpan(children: spans), maxLines: 1, overflow: TextOverflow.ellipsis);
  }
}

class _Empty extends StatelessWidget {
  const _Empty(this.title, this.lines, {this.actions = const []});
  final String title;
  final List<String> lines;
  final List<Widget> actions;
  @override
  Widget build(BuildContext context) => _Fit(
        natural: null,
        child: Center(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 20),
          child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
            const Glyph(G.effect, size: 26, color: N.g56),
            const SizedBox(height: 12),
            if (title.contains('match')) Row(mainAxisSize: MainAxisSize.min, children: [ErrMark(gap: 6, color: Role.warning), Flexible(child: Text(title, textAlign: TextAlign.center, style: T.name(N.g91).copyWith(height: 1.3)))]) else Text(title, textAlign: TextAlign.center, style: T.name(N.g91).copyWith(height: 1.3)),
            const SizedBox(height: 8),
            for (final l in lines) Padding(padding: const EdgeInsets.only(bottom: 4), child: Text(l, textAlign: TextAlign.center, style: T.label(N.g76).copyWith(height: 1.35))),
            if (actions.isNotEmpty) ...[const SizedBox(height: 8), Wrap(alignment: WrapAlignment.center, spacing: 8, runSpacing: 8, children: actions)],
          ]),
        ),
      ),
      );
}

/// A list row for an item, 34 px. One left rhythm: the 2 px tick lives in the gutter (x 2 to 4), thumbnail / name / everything else starts at x 10. Selected = g20 + tick; hover = g15.
/// A filter that already fixes a fact must not be repeated by its rows: [showKind] false drops the right-hand kind label, [showCat] false drops the category from the second line.
/// A media row never repeats its media kind ("Video") in the second line: the thumbnail badge and the right-hand label say it.
class _ItemRow extends StatelessWidget {
  const _ItemRow(this.it, {this.selected = false, this.tokens = const [], this.onTap, this.onUse, this.right, this.hover, this.showKind = true, this.showCat = true});
  final It it;
  final bool selected, showKind, showCat;
  final List<String> tokens;
  final VoidCallback? onTap, onUse;
  final Widget? right;
  final bool? hover;
  @override
  Widget build(BuildContext context) {
    final sub = (it.cat == it.place || !showCat || it.asset != null) ? it.jp : '${it.jp} · ${it.cat}';
    final tail = right ?? (showKind && _QuietKind.of(context) != it.kind ? Text(it.kind, style: T.micro(N.g76)) : null);
    return _H(
      onTap: onTap,
      onDouble: onUse,
      force: hover,
      builder: (context, hover) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 34,
        color: selected ? N.g20 : (hover ? N.g15 : _clear),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              _Thumb(it, w: 34, h: 22),
              const SizedBox(width: 8),
              Expanded(
                child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  _Hl(it.name, tokens, on: selected),
                  const SizedBox(height: 4),
                  _Hl.sub(sub, tokens),
                ]),
              ),
              if (tail != null) ...[const SizedBox(width: 8), tail],
            ]),
          ),
          _GutterTick(selected),
        ]),
      ),
    );
  }
}

/// The foot of a result list (g76, 26 px, hairline above). It says exactly one of two things: "End of results", or "N more (where)" with N = only what lies OUTSIDE the count line's scope, so it can never repeat the count line's numbers.
/// A list whose count line already says it ("8 of 72", "72 items") gets no foot at all (see [_Results.foot]).
class _EndLine extends StatelessWidget {
  const _EndLine(this.more, {this.where = 'in other places'});
  final int more;
  final String where;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        alignment: Alignment.centerLeft,
        decoration: const BoxDecoration(border: Border(top: BorderSide(color: N.rowLine))),
        child: Text(more == 0 ? 'End of results' : '$more more $where', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
      );
}

/// What a panel frame tells its body: whether the frame is only as tall as its content ("Fit list"), where the body is (so the chrome around the list can be measured), and where to report the height the content needs.
class _FitScope extends InheritedWidget {
  const _FitScope({required this.fit, required this.bodyKey, required this.report, required super.child});
  final bool fit;
  final GlobalKey bodyKey;
  final void Function(double? need) report;
  static _FitScope? of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_FitScope>();
  @override
  bool updateShouldNotify(_FitScope o) => o.fit != fit;
}

/// The one flexible region of a panel (it sits where an Expanded child sits). Outside a fitting frame it is its child. In "Fit list" mode it takes only [natural] px (top-aligned) and reports frame height = everything around it + [natural];
/// natural null = the content wants the whole frame (an empty state): the frame keeps its default height.
class _Fit extends StatelessWidget {
  const _Fit({required this.natural, required this.child});
  final double? natural;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final s = _FitScope.of(context);
    if (s == null || !s.fit) return child;
    return LayoutBuilder(builder: (context, box) {
      final e = box.maxHeight, n = natural;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        final bodyH = (s.bodyKey.currentContext?.findRenderObject() as RenderBox?)?.size.height;
        if (bodyH != null) s.report(n == null ? null : bodyH - e + n);
      });
      if (n == null) return child;
      return Align(alignment: Alignment.topCenter, child: SizedBox(height: math.min(n, e), child: child));
    });
  }
}

/// The results area. Fit list (default): the list is exactly its rows and the end line sits directly under the last row. 700 tall: the list is top-aligned and the end line is pinned at the foot; the free space stays empty (no hint block).
/// [listH] is the height the list takes (default: [rows] x 34). [foot] false = no foot (the count line above already states what is shown and what is not); the frame then ends right under the last row.
class _Results extends StatelessWidget {
  const _Results({required this.list, required this.rows, required this.more, this.where = 'in other places', this.listH, this.foot = true});
  final Widget list;
  final int rows, more;
  final String where;
  final double? listH;
  final bool foot;
  @override
  Widget build(BuildContext context) => _Fit(
        natural: (listH ?? rows * 34.0) + (foot ? 26 : 0),
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Expanded(child: Align(alignment: Alignment.topCenter, child: list)),
          if (foot) _EndLine(more, where: where),
        ]),
      );
}

/// The one kind a list is mostly made of: more than 80% of at least 5 rows share it. Those rows drop the kind word; the exceptions keep it and the count line says it once ("mostly effects"). null = mixed.
String? _dominant(List<It> r, String Function(It) of) {
  if (r.length < 5) return null;
  final n = <String, int>{};
  for (final i in r) {
    n[of(i)] = (n[of(i)] ?? 0) + 1;
  }
  final top = n.entries.reduce((a, b) => a.value >= b.value ? a : b);
  return top.value * 5 > r.length * 4 ? top.key : null;
}

/// "mostly effects" for a count line, from the kind of the rows ([_dominant] on [It.kind]); null when mixed.
String? _mostly(List<It> r) {
  final k = _dominant(r, (i) => i.kind);
  return k == null ? null : 'mostly ${k == 'Media' ? 'media' : '${k.toLowerCase()}s'}';
}

/// Tells the rows below which kind is said once in the count line, so they leave their own kind word off (only the exceptions show it).
class _QuietKind extends InheritedWidget {
  const _QuietKind(this.kind, {required super.child});
  final String? kind;
  static String? of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_QuietKind>()?.kind;
  @override
  bool updateShouldNotify(_QuietKind o) => o.kind != kind;
}

/// Where a list has nothing yet: an empty slot (28 px, dashed g38 outline) with one short word in g63, not a sentence.
class _HintRow extends StatelessWidget {
  const _HintRow(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(10, 4, 10, 4),
        child: CustomPaint(
          foregroundPainter: const DashedBox(N.g38, radius: 4),
          child: Container(height: 28, alignment: Alignment.centerLeft, padding: const EdgeInsets.symmetric(horizontal: 8), child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g63))),
        ),
      );
}

// ---- the search field (EditableText; Esc and arrows are the owner's, through the focus node). Edge: rest g20, hover g38, focus g63.

class _Search extends StatelessWidget {
  const _Search({required this.controller, required this.node, this.hint = 'Search effects, presets, media', this.onChanged, this.slash = true, this.autofocus = false, this.trailing, this.focusLook, this.hoverLook, this.clearHover});
  final TextEditingController controller;
  final FocusNode node;
  final String hint;
  final ValueChanged<String>? onChanged;
  final bool slash, autofocus;
  final Widget? trailing;

  /// Pins the focused look (Parts sheets); null = follow the focus node.
  final bool? focusLook;

  /// Pins the hover look (Parts sheets): edge g38 between rest g20 and focus g63; null = follow the pointer.
  final bool? hoverLook;

  /// Pins the hover look of the clear mark (Parts sheets).
  final bool? clearHover;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([controller, node]),
        builder: (context, _) {
          final f = focusLook ?? node.hasFocus, empty = controller.text.isEmpty;
          return _H(
            onTap: node.requestFocus,
            cursor: SystemMouseCursors.text,
            force: hoverLook,
            builder: (context, hv) => AnimatedContainer(
              duration: Mo.dur,
              curve: Mo.ease,
              height: 28,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g63 : (hv ? N.g38 : N.g20))),
              child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                Expanded(
                  child: LayoutBuilder(builder: (context, box) {
                    // Typed text: at rest it ends in an ellipsis like the hint; while editing the field scrolls, so the right edge fades instead.
                    final over = !empty && (TextPainter(text: TextSpan(text: controller.text, style: T.name(N.g95)), maxLines: 1, textDirection: TextDirection.ltr)..layout()).width > box.maxWidth;
                    final rest = over && !f;
                    final editable = EditableText(
                      controller: controller,
                      focusNode: node,
                      autofocus: autofocus,
                      style: T.name(N.g95),
                      cursorColor: N.g95,
                      backgroundCursorColor: N.g44,
                      selectionColor: Role.selected.withValues(alpha: .35),
                      cursorWidth: 1,
                      maxLines: 1,
                      onChanged: onChanged,
                    );
                    return Stack(alignment: Alignment.centerLeft, children: [
                      if (empty) IgnorePointer(child: Text(hint, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g76))),
                      if (rest) IgnorePointer(child: Text(controller.text, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: T.name(N.g95))),
                      Offstage(
                        offstage: rest,
                        child: over
                            ? ShaderMask(
                                blendMode: BlendMode.dstIn,
                                shaderCallback: (r) => ui.Gradient.linear(Offset(r.width - 16, 0), Offset(r.width, 0), const [Color(0xFFFFFFFF), Color(0x00FFFFFF)]),
                                child: editable,
                              )
                            : editable,
                      ),
                    ]);
                  }),
                ),
                if (trailing != null) ...[const SizedBox(width: 8), trailing!],
                if (!empty)
                  _Cross(w: 18, hover: clearHover, onTap: () {
                    controller.clear();
                    onChanged?.call('');
                    node.requestFocus();
                  })
                else if (slash && !f) ...[const SizedBox(width: 8), const SizedBox(width: 18, child: _Kbd('/'))],
              ]),
            ),
          );
        },
      );
}
