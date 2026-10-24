part of 'panel_inspector_b.dart';

// ---- the frame of a story: a caption outside, a panel inside --------------------------------------------------------------

enum _Habit { habit, addition, departs, retired }

String _habitWord(_Habit h) => switch (h) { _Habit.habit => 'HABIT', _Habit.addition => 'ADDITION', _Habit.departs => 'DEPARTS FROM HABIT', _Habit.retired => 'WITHDRAWN (kept to compare)' };

class _Caption extends StatelessWidget {
  const _Caption(this.id, this.title, this.habit, this.contract);
  final String id, title, contract;
  final _Habit habit;
  @override
  Widget build(BuildContext context) {
    final loud = habit == _Habit.departs || habit == _Habit.retired;
    return SizedBox(
      width: 560,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Row(children: [
          Text(id, style: T.value(N.g95).copyWith(fontWeight: FontWeight.w700)),
          const SizedBox(width: 8),
          Flexible(child: Text(title, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
          const SizedBox(width: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(4), border: Border(bottom: BorderSide(color: loud ? Role.selected : N.g26))),
            child: Text(_habitWord(habit), maxLines: 1, style: T.micro(loud ? N.g95 : N.g76)),
          ),
        ]),
        const SizedBox(height: 8),
        Text(contract, style: T.label(N.g76).copyWith(height: 1.4)),
      ]),
    );
  }
}

/// What the story frame tells its panel: "Fit list" (default) = the panel is only as tall as its content; "700 tall" = the stress frame.
class _FitScope extends InheritedWidget {
  const _FitScope({required this.fit, required super.child});
  final bool fit;
  static bool of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_FitScope>()?.fit ?? false;
  @override
  bool updateShouldNotify(_FitScope o) => o.fit != fit;
}

/// Where an Expanded child sits: in Fit mode it takes only what it needs (and still scrolls past the 700 px cap).
Widget _grow(BuildContext c, Widget child) => _FitScope.of(c) ? Flexible(child: child) : Expanded(child: child);

/// The scrolling list of a panel: exactly its rows in Fit mode, the whole frame in 700 tall.
class _Lv extends StatelessWidget {
  const _Lv({required this.children, this.controller});
  final List<Widget> children;
  final ScrollController? controller;
  @override
  Widget build(BuildContext context) => _ScrollFade(child: ListView(controller: controller, shrinkWrap: _FitScope.of(context), padding: const EdgeInsets.only(bottom: 8), children: children));
}

/// The cue for a list that goes on below the edge: a 16 px fade into the panel ground g10 over the last row, shown only while there is more to scroll to. Wraps any scrollable.
class _ScrollFade extends StatefulWidget {
  const _ScrollFade({required this.child});
  final Widget child;
  @override
  State<_ScrollFade> createState() => _ScrollFadeState();
}

class _ScrollFadeState extends State<_ScrollFade> {
  bool more = false;

  bool _metrics(ScrollMetrics m) {
    final next = m.extentAfter > 1;
    if (next != more) {
      WidgetsBinding.instance.addPostFrameCallback((_) => mounted ? setState(() => more = next) : null);
    }
    return false;
  }

  @override
  Widget build(BuildContext context) => NotificationListener<ScrollMetricsNotification>(
        onNotification: (n) => _metrics(n.metrics),
        child: NotificationListener<ScrollNotification>(
          onNotification: (n) => _metrics(n.metrics),
          child: Stack(fit: StackFit.passthrough, children: [
            widget.child,
            Positioned(
              left: 0,
              right: 0,
              bottom: 0,
              height: 16,
              child: IgnorePointer(
                child: AnimatedOpacity(
                  duration: Mo.dur,
                  opacity: more ? 1 : 0,
                  child: const DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [Color(0x00191919), N.g10]))),
                ),
              ),
            ),
          ]),
        ),
      );
}

/// One use case: caption outside, then the panel frame. [panels] gets the height a frame may take (700). Knob "Panel height" is on every story.
WidgetbookUseCase _story(String name, String id, String title, _Habit habit, String contract, Widget Function(BuildContext c, double h) panels) => WidgetbookUseCase(
      name: name,
      builder: (c) {
        final fit = c.knobs.object.dropdown<String>(label: 'Panel height', options: const ['Fit list (default)', '700 tall'], initialOption: 'Fit list (default)', labelBuilder: (v) => v) != '700 tall';
        return ColoredBox(
          color: N.g07,
          child: LayoutBuilder(builder: (ctx, box) => SingleChildScrollView(
                child: ConstrainedBox(
                  constraints: BoxConstraints(minHeight: box.maxHeight),
                  child: Center(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(vertical: 20, horizontal: 16),
                      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                        _Caption(id, title, habit, contract),
                        const SizedBox(height: 12),
                        _FitScope(fit: fit, child: Builder(builder: (ctx) => panels(c, 700))),
                      ]),
                    ),
                  ),
                ),
              )),
        );
      },
    );

/// A panel frame: ground g10, a hairline, a header with the layer's name. Fit mode: as tall as the content (cap 700, then the list scrolls).
class _Panel extends StatelessWidget {
  const _Panel({required this.title, required this.sub, required this.child, required this.h, this.w = 282});
  final String title, sub;
  final Widget child;
  final double w, h;
  @override
  Widget build(BuildContext context) {
    final fit = _FitScope.of(context);
    return Container(
      width: w,
      height: fit ? null : h,
      constraints: fit ? const BoxConstraints(maxHeight: 700) : null,
      decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
      child: Column(mainAxisSize: fit ? MainAxisSize.min : MainAxisSize.max, children: [
        Container(
          height: 44,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: const BoxDecoration(color: N.g10, border: Border(bottom: BorderSide(color: N.g07))),
          child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
            SizedBox(width: double.infinity, child: Text(title, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
            const SizedBox(height: 4),
            SizedBox(width: double.infinity, child: Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
          ]),
        ),
        _grow(context, ClipRect(child: child)),
      ]),
    );
  }
}

/// Owns a [_St] for the life of a use case.
class _Live extends StatefulWidget {
  const _Live({required this.builder});
  final Widget Function(BuildContext context, _St st) builder;
  @override
  State<_Live> createState() => _LiveState();
}

class _LiveState extends State<_Live> {
  final st = _St();
  @override
  void initState() {
    super.initState();
    st.addListener(_up);
  }

  void _up() => setState(() {});

  @override
  void dispose() {
    st.removeListener(_up);
    st.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.builder(context, st);
}

// ---- a forced pose, so a sheet can show every state at once -------------------------------------------------------------

/// A state a part can be held in for the Parts sheets. null = live (follows the pointer).
enum _Pose { hover, pressed, focused }

/// Hover and press in one place; with a [pose] it holds that state instead of listening to the pointer.
class _Hv extends StatefulWidget {
  const _Hv({this.pose, this.onTap, this.cursor = SystemMouseCursors.click, required this.builder});
  final _Pose? pose;
  final VoidCallback? onTap;
  final MouseCursor cursor;
  final Widget Function(BuildContext context, bool hover, bool down) builder;
  @override
  State<_Hv> createState() => _HvState();
}

class _HvState extends State<_Hv> {
  bool h = false, d = false;
  @override
  Widget build(BuildContext context) {
    final p = widget.pose;
    if (p != null) {
      return widget.builder(context, p == _Pose.hover || p == _Pose.pressed, p == _Pose.pressed);
    }
    return MouseRegion(
      cursor: widget.cursor,
      onEnter: (_) => setState(() => h = true),
      onExit: (_) => setState(() => h = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onTap,
        onTapDown: (_) => setState(() => d = true),
        onTapUp: (_) => setState(() => d = false),
        onTapCancel: () => setState(() => d = false),
        child: widget.builder(context, h, d),
      ),
    );
  }
}

/// Keyboard focus for a part that has an [onTap]: Enter or Space acts; a held [pose] of focused (or the real focus) draws a 1px g63 ring over the part.
class _Fcs extends StatelessWidget {
  const _Fcs({required this.onTap, required this.pose, required this.child, this.radius = 0});
  final VoidCallback? onTap;
  final _Pose? pose;
  final Widget child;
  final double radius;
  @override
  Widget build(BuildContext context) => Focus(
        canRequestFocus: onTap != null,
        onKeyEvent: (n, e) {
          if (e is KeyDownEvent && (e.logicalKey == LogicalKeyboardKey.enter || e.logicalKey == LogicalKeyboardKey.space) && onTap != null) {
            onTap!();
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Builder(builder: (ctx) {
          final on = pose == _Pose.focused || (pose == null && Focus.of(ctx).hasFocus);
          return DecoratedBox(
            position: DecorationPosition.foreground,
            decoration: BoxDecoration(borderRadius: radius > 0 ? BorderRadius.circular(radius) : null, border: Border.all(color: on ? N.g63 : const Color(0x00000000))),
            child: child,
          );
        }),
      );
}

// ---- small shared chrome ---------------------------------------------------------------------------------------------------

/// One chevron for closed and open: both fit the same 12 px box and share the left edge (x 3); the open one is the closed one turned down, drawn, not rotated, so nothing sticks out.
class _Chev extends StatelessWidget {
  const _Chev(this.open, {this.color = N.g63});
  final bool open;
  final Color color;
  @override
  Widget build(BuildContext context) => TweenAnimationBuilder<double>(
        tween: Tween(end: open ? 1 : 0),
        duration: Mo.dur,
        curve: Mo.ease,
        builder: (_, t, _) => SizedBox(width: 12, height: 12, child: CustomPaint(painter: _ChevPaint(color, t))),
      );
}

class _ChevPaint extends CustomPainter {
  const _ChevPaint(this.c, this.t);
  final Color c;
  final double t;
  @override
  void paint(Canvas cv, Size s) {
    Offset l(Offset a, Offset b) => Offset.lerp(a, b, t)!;
    final p = Path()
      ..moveTo(l(const Offset(3, 2.5), const Offset(3, 4.5)).dx, l(const Offset(3, 2.5), const Offset(3, 4.5)).dy)
      ..lineTo(l(const Offset(6.5, 6), const Offset(6.5, 8)).dx, l(const Offset(6.5, 6), const Offset(6.5, 8)).dy)
      ..lineTo(l(const Offset(3, 9.5), const Offset(10, 4.5)).dx, l(const Offset(3, 9.5), const Offset(10, 4.5)).dy);
    cv.drawPath(p, Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  }

  @override
  bool shouldRepaint(_ChevPaint o) => o.c != c || o.t != t;
}

const _clear = Color(0x00000000);

/// The one selection / state tick of the book: 2 px wide, inset in the left gutter (x 2 to 4, never on the panel edge), 16 px tall (or the row, if shorter). Content starts at x 12 in every row, ticked or not.
class _Gut extends StatelessWidget {
  const _Gut({required this.tick, required this.child, this.full = false});
  final Color? tick;
  final Widget child;

  /// A tall block (a band) carries a tick as long as the block minus 4 px at each end, not the 16 px of a row.
  final bool full;
  @override
  Widget build(BuildContext context) => Stack(fit: StackFit.passthrough, children: [
        child,
        if (tick != null && !full && tick == Role.error)
          const Positioned(left: 1, top: 0, bottom: 0, width: 10, child: Center(child: ErrMark(size: 9)))
        else if (tick != null)
          Positioned(
            left: 2,
            top: full ? 4 : 0,
            bottom: full ? 4 : 0,
            width: 2,
            child: full ? DecoratedBox(decoration: BoxDecoration(color: tick, borderRadius: BorderRadius.circular(1))) : Center(child: Container(width: 2, height: 16, decoration: BoxDecoration(color: tick, borderRadius: BorderRadius.circular(1)))),
          ),
      ]);
}

/// A number in the label voice: the same sans baseline as the words around it, fixed-width figures.
TextStyle _cnt([Color c = N.g76]) => T.label(c).copyWith(fontFeatures: const [FontFeature.tabularFigures()]);

/// A group heading: small caps, count on the same baseline, a chevron when it folds. Rest g10 (the plate), hover g15, selected g20 + the gutter tick, pressed g26.
class _Sect extends StatelessWidget {
  const _Sect(this.title, {this.count, this.open, this.onTap, this.trailing, this.sub, this.upper = true, this.pose, this.selected = false});
  final _Pose? pose;
  final bool selected;
  final String title;
  final bool upper;
  final int? count;
  final bool? open;
  final String? sub;
  final VoidCallback? onTap;
  final Widget? trailing;
  @override
  Widget build(BuildContext context) {
    // title and count share one baseline
    final titleRow = Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.baseline, textBaseline: TextBaseline.alphabetic, children: [
      Flexible(child: Text(upper ? title.toUpperCase() : title, style: T.micro(N.g76).copyWith(letterSpacing: upper ? .6 : 0), maxLines: 1, overflow: TextOverflow.ellipsis)),
      if (count != null) ...[const SizedBox(width: 8), Text('$count', style: _cnt(N.g76).copyWith(fontWeight: FontWeight.w500, letterSpacing: .15))],
    ]);
    return _Hv(
        pose: pose,
        onTap: onTap,
        cursor: onTap == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
        builder: (_, h, d) => _Fcs(
          onTap: onTap,
          pose: pose,
          child: _Gut(
            tick: selected ? N.g95 : null,
            child: Container(
              height: 28,
              color: d ? N.g26 : (selected ? N.g20 : (h && (onTap != null || pose != null) ? N.g15 : null)),
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Row(children: [
                if (open != null) ...[_Chev(open!), const SizedBox(width: 4)],
                if (sub != null) ...[
                  Flexible(child: titleRow),
                  const SizedBox(width: 8),
                  Expanded(child: Text(sub!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
                ] else
                  Expanded(child: Align(alignment: Alignment.centerLeft, child: titleRow)),
                if (trailing != null) ...[const SizedBox(width: 8), trailing!],
              ]),
            ),
          ),
        ),
    );
  }
}

/// A text button that reads as a word: g76 at rest, g95 + underline on hover, g95 on g26 pressed, a 1 px g63 ring on focus; [on] = selected (g20 fill + accent line). The text sits on the content edge (no padding): the fill grows 4 px past it.
class _Word extends StatelessWidget {
  const _Word(this.text, {this.onTap, this.on = false, this.pose});
  final _Pose? pose;
  final String text;
  final bool on;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => _Hv(
        pose: pose,
        onTap: onTap,
        cursor: onTap == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
        builder: (_, h, d) => _Fcs(
          onTap: onTap,
          pose: pose,
          radius: 4,
          child: Opacity(
            opacity: onTap == null && pose == null && !on ? .5 : 1,
            child: SizedBox(
            height: 20,
            child: Stack(clipBehavior: Clip.none, alignment: Alignment.center, children: [
              Positioned(
                left: -4,
                right: -4,
                top: 0,
                bottom: 0,
                child: DecoratedBox(decoration: BoxDecoration(color: d ? N.g26 : (on ? N.g20 : null), borderRadius: BorderRadius.circular(4), border: Border(bottom: BorderSide(color: on ? Role.selected : _clear)))),
              ),
              Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(on || h || d ? N.g95 : N.g76).copyWith(decoration: h && !on ? TextDecoration.underline : TextDecoration.none, decorationColor: N.g95)),
            ]),
          ),
        )),
      );
}

/// A band: a g13 card on the 12 px content edges, with an optional state tick in its gutter; its text starts 12 px inside. One look for every note, result and dead end.
class _Band extends StatelessWidget {
  const _Band({required this.child, this.tick, this.margin = const EdgeInsets.fromLTRB(12, 8, 12, 8)});
  final Widget child;
  final Color? tick;
  final EdgeInsets margin;
  @override
  Widget build(BuildContext context) => Container(
        margin: margin,
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)),
        child: _Gut(tick: tick ?? N.g26, full: true, child: Padding(padding: const EdgeInsets.fromLTRB(12, 8, 8, 8), child: child)),
      );
}

/// Words that act, side by side: 12 px apart, wrapping.
Widget _words(List<Widget> ws) => Wrap(spacing: 12, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: ws);

/// A quiet button that looks disabled when it cannot act (g63 text, 50 %), and says nothing in a tooltip-less way: the label itself states what is missing.
class _Btn extends StatelessWidget {
  const _Btn(this.text, {this.onTap, this.primary = false});
  final String text;
  final bool primary;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => onTap == null ? Opacity(opacity: .5, child: IgnorePointer(child: QuietButton(text))) : QuietButton(text, primary: primary, onTap: onTap);
}

/// The reset mark of a changed value: a word-sized target, g63 at rest, g95 on hover.
class _Reset extends StatelessWidget {
  const _Reset({required this.show, this.onTap});
  final bool show;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: 20,
        child: show ? Hov(onTap: onTap, builder: (_, h) => Center(child: Text('↺', style: T.label(h ? N.g95 : N.g63).copyWith(fontSize: 12)))) : null,
      );
}

/// A one-line status or hint at the foot of a panel: a state in a sentence, never an instruction that lingers.
class _Hint extends StatelessWidget {
  const _Hint(this.text, {this.color = N.g76});
  final String text;
  final Color color;
  @override
  Widget build(BuildContext context) => Align(alignment: Alignment.centerLeft, child: Padding(padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4), child: color == Role.error ? Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [const ErrMark(gap: 5), Flexible(child: Text(text, style: T.label(color).copyWith(height: 1.4)))]) : Text(text, style: T.label(color).copyWith(height: 1.4))));
}

/// The text field of the lab (single line, EditableText).
class _Field extends StatefulWidget {
  const _Field({required this.ctrl, required this.hint, this.count, this.pose});
  final _Pose? pose;
  final TextEditingController ctrl;
  final String hint;
  final String? count;
  @override
  State<_Field> createState() => _FieldState();
}
