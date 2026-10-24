part of 'panel_inspector_a.dart';

// Panel drafts, Inspector A (I2 numbers, I3 space, I4 key state, I5 ease, I6 diff/reset).
// One panel = one option. Each panel is a real-looking Inspector column (282 px, or 220 px for a dock) with its own local state; nothing talks to another panel.
// Shared rule under every draft (options-inspector.md D-I3): scrub right = up, one pixel = one displayed unit, Shift x10, Alt x0.1, a double-click goes to the default.

// ---- sizes of the draft (surface: none of these is a token yet; they are the book's own, named once) ----
const _panelW = 282.0, _panelH = 700.0, _dockW = 220.0; // surface: Inspector column 282, dock 220, a ~700 px tall window
const _rowH = 28.0, _cellH = 24.0, _labW = 96.0, _labWDock = 72.0, _keyW = 24.0, _footH = 24.0; // all on the 4 px grid
const _gutter = 12.0;

// Label inks: a 10 px thin label never goes under g76 (rest), a changed one is white, a locked one g63. One rule for every label of every panel.
const _labRest = N.g76, _labOff = N.g63;
/// "Several layers / mixed" is not "changed": in a coloured palette a mixed value carries neither the changed colour nor the corner mark (Grey keeps today's look).
bool _chg(_M m, String id) => m.changed(id) && (Role.grey || !m.isMixed(id));
Color get _labChanged => Role.of(N.g95, Role.changed);

/// The one label style of every row. State is never tone alone: a changed label also carries a 1 px dotted underline (the shape cue), a locked one is dimmer and has none.
TextStyle _labStyle({required bool changed, bool enabled = true, bool hot = false, Color? tone}) => T.label(hot ? N.g100 : (tone ?? (!enabled ? _labOff : (changed ? _labChanged : _labRest)))).copyWith(
      decoration: changed && enabled ? TextDecoration.underline : TextDecoration.none,
      decorationStyle: TextDecorationStyle.dotted,
      decorationColor: Role.of(N.g63, Role.changed),
      decorationThickness: 1,
    );

/// A grip: two hairlines, 3 px apart, the mark of a field or label that is dragged. Fine, never a box.
class _GripPaint extends CustomPainter {
  const _GripPaint(this.c);
  final Color c;
  @override
  void paint(Canvas z, Size s) {
    final p = Paint()..color = c..strokeWidth = 1;
    z.drawLine(Offset(.5, 0), Offset(.5, s.height), p);
    z.drawLine(Offset(3.5, 0), Offset(3.5, s.height), p);
  }

  @override
  bool shouldRepaint(_GripPaint o) => o.c != c;
}

Widget _grip(Color c, {double h = 8}) => CustomPaint(size: Size(4, h), painter: _GripPaint(c));

/// A changed value: a 4 px corner triangle on the field (a shape, so grey-scale and colour-blind readers see it).
class _CornerPaint extends CustomPainter {
  const _CornerPaint();
  @override
  void paint(Canvas z, Size s) => z.drawPath(Path()..moveTo(0, 0)..lineTo(s.width, 0)..lineTo(0, s.height)..close(), Paint()..color = Role.of(N.g76, Role.changed));

  @override
  bool shouldRepaint(_CornerPaint o) => false;
}

/// Unavailable: a hatch of 1 px diagonals at 4 px, over the field.
class _HatchPaint extends CustomPainter {
  const _HatchPaint();
  @override
  void paint(Canvas z, Size s) {
    final p = Paint()..color = N.g15..strokeWidth = 1;
    for (var x = -s.height; x < s.width; x += 4) {
      z.drawLine(Offset(x, s.height), Offset(x + s.height, 0), p);
    }
  }

  @override
  bool shouldRepaint(_HatchPaint o) => false;
}

enum _Hab { habit, addition, departs, withdrawn }

String _habWord(_Hab h) => switch (h) {
      _Hab.habit => 'HABIT',
      _Hab.addition => 'ADDITION',
      _Hab.departs => 'DEPARTS FROM HABIT',
      _Hab.withdrawn => 'DEPARTS · WITHDRAWN',
    };

Color get _rule => const Color(0x1FFFFFFF);

// ---- the use-case frame: ground, caption outside the panel, the panel, optional side material ----

class _Ground extends StatefulWidget {
  const _Ground({required this.child, this.keys = false});
  final Widget child;
  final bool keys; // a sheet: PageUp / PageDown / Home / End / arrows scroll it
  @override
  State<_Ground> createState() => _GroundState();
}

class _GroundState extends State<_Ground> {
  final _v = ScrollController(), _h = ScrollController();
  @override
  void dispose() {
    _v.dispose();
    _h.dispose();
    super.dispose();
  }

  KeyEventResult _key(FocusNode _, KeyEvent e) {
    if (!widget.keys || e is KeyUpEvent || !_v.hasClients) {
      return KeyEventResult.ignored;
    }
    final p = _v.position, page = p.viewportDimension * .9;
    final k = e.logicalKey;
    double? to;
    if (k == LogicalKeyboardKey.pageDown || k == LogicalKeyboardKey.space) {
      to = _v.offset + page;
    } else if (k == LogicalKeyboardKey.pageUp) {
      to = _v.offset - page;
    } else if (k == LogicalKeyboardKey.arrowDown) {
      to = _v.offset + 48;
    } else if (k == LogicalKeyboardKey.arrowUp) {
      to = _v.offset - 48;
    } else if (k == LogicalKeyboardKey.home) {
      to = 0;
    } else if (k == LogicalKeyboardKey.end) {
      to = p.maxScrollExtent;
    }
    if (to == null) {
      return KeyEventResult.ignored;
    }
    _v.jumpTo(to.clamp(0.0, p.maxScrollExtent));
    return KeyEventResult.handled;
  }

  /// Both axes scroll and both bars are always drawn: a sheet wider or taller than the window is never cut without a cue.
  @override
  Widget build(BuildContext context) => Focus(
        autofocus: widget.keys,
        onKeyEvent: _key,
        child: ColoredBox(
        color: N.g07,
        child: LayoutBuilder(
          builder: (context, box) => RawScrollbar(
            controller: _v,
            thumbVisibility: true,
            thickness: 6,
            radius: const Radius.circular(3),
            thumbColor: N.g38,
            child: RawScrollbar(
              controller: _h,
              thumbVisibility: true,
              thickness: 6,
              radius: const Radius.circular(3),
              thumbColor: N.g38,
              notificationPredicate: (n) => n.depth == 1,
              child: SingleChildScrollView(
                controller: _v,
                child: SingleChildScrollView(
                  controller: _h,
                  scrollDirection: Axis.horizontal,
                  child: ConstrainedBox(
                    constraints: BoxConstraints(minWidth: box.maxWidth, minHeight: box.maxHeight),
                    child: Center(child: Padding(padding: const EdgeInsets.fromLTRB(24, 24, 24, 40), child: widget.child)),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
      );
}

/// The one-line story caption sits outside the frame: option id, habit status, one sentence of contract. Nothing explanatory lives inside a panel.
Widget _story(String id, _Hab hab, String contract, Widget panel, {Widget? side, double captionW = 720}) => _Ground(
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        SizedBox(
          width: captionW,
          child: Text.rich(TextSpan(children: [
            TextSpan(text: '$id   ', style: T.name(N.g95)),
            TextSpan(text: '${_habWord(hab)}   ', style: T.micro(N.g76)),
            TextSpan(text: contract, style: T.label(N.g76).copyWith(height: 1.35)),
          ])),
        ),
        const SizedBox(height: 8),
        Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
          panel,
          if (side != null) ...[const SizedBox(width: 24), side],
        ]),
      ]),
    );

/// The Panel height knob: Fit list (default) = the frame is as tall as its content, never taller than a 700 px window; 700 tall = the stress frame, always the full window.
class _FitMode extends InheritedWidget {
  const _FitMode({required this.fit, required super.child});
  final bool fit;
  static bool of(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_FitMode>()?.fit ?? false;
  @override
  bool updateShouldNotify(_FitMode o) => o.fit != fit;
}

/// One use case. Every panel use case carries the Panel height knob (Fit list default | 700 tall); a Parts sheet is not a panel and has none.
WidgetbookUseCase _uc(String name, Widget Function(BuildContext) build) => WidgetbookUseCase(
      name: name,
      builder: (c) {
        if (name.startsWith('Parts')) {
          return build(c);
        }
        final fit = c.knobs.object.dropdown<String>(label: 'Panel height', options: const ['Fit list (default)', '700 tall'], initialOption: 'Fit list (default)', labelBuilder: (v) => v) != '700 tall';
        return _FitMode(fit: fit, child: build(c));
      },
    );

// ---- state shell: hover, keyboard focus, pressed (pinned for the Parts sheets) ----

/// One shell for every small control that is not a number cell: hover, focus and pressed come from the pointer / the keyboard, or are pinned (Parts specimens).
/// [enabled] false = a dead control: no state shows and the arrow cursor stays. Enter / Space activate it when focused.
class _Ctl extends StatefulWidget {
  const _Ctl({required this.builder, this.onTap, this.onLong, this.onSecondary, this.enabled = true, this.hover, this.focus, this.pressed, this.onHover, this.onFocus});
  final Widget Function(BuildContext, bool hover, bool focus, bool pressed) builder;
  final VoidCallback? onTap, onLong, onSecondary;
  final bool enabled;
  final bool? hover, focus, pressed;
  final ValueChanged<bool>? onHover, onFocus;
  @override
  State<_Ctl> createState() => _CtlState2();
}

class _CtlState2 extends State<_Ctl> {
  bool _h = false, _f = false, _p = false;
  @override
  Widget build(BuildContext context) {
    final on = widget.enabled;
    return FocusableActionDetector(
      enabled: on,
      onShowHoverHighlight: (v) {
        setState(() => _h = v);
        widget.onHover?.call(v);
      },
      onShowFocusHighlight: (v) {
        setState(() => _f = v);
        widget.onFocus?.call(v);
      },
      mouseCursor: on ? SystemMouseCursors.click : SystemMouseCursors.basic,
      actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) {
        widget.onTap?.call();
        return null;
      })},
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: on ? widget.onTap : null,
        onLongPress: on ? widget.onLong : null,
        onSecondaryTap: on ? widget.onSecondary : null,
        onTapDown: on ? (_) => setState(() => _p = true) : null,
        onTapUp: (_) => setState(() => _p = false),
        onTapCancel: () => setState(() => _p = false),
        child: widget.builder(context, on && (widget.hover ?? _h), on && (widget.focus ?? _f), on && (widget.pressed ?? _p)),
      ),
    );
  }
}

/// A word button. rest g20 fill + g44 edge; hover g26 + g63; focus g95 edge; pressed sinks to g13 with a 1 px accent line at 50 % on the foot; open (a menu is out) g20 + g63 edge; off g13 + g20 edge, g63 text.
class _Btn extends StatelessWidget {
  const _Btn(this.text, {this.h = 24, this.on = true, this.open = false, this.hover = false, this.focus = false, this.pressed = false});
  final String text;
  final double h;
  final bool on, open, hover, focus, pressed;
  @override
  Widget build(BuildContext context) {
    final fill = !on ? N.g13 : (pressed ? N.g13 : (hover ? N.g26 : N.g20));
    final edge = !on ? N.g20 : (focus ? N.g95 : ((hover || pressed || open) ? N.g63 : N.g44));
    return AnimatedContainer(
      duration: Mo.dur,
      curve: Mo.ease,
      height: h,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(4), border: Border.all(color: edge)),
      child: Stack(alignment: Alignment.center, children: [
        Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(!on ? N.g63 : (hover || open || focus || pressed ? N.g100 : N.g91))),
        Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: pressed && on ? Role.selected.withValues(alpha: .5) : const Color(0x00000000))),
      ]),
    );
  }
}

/// A full-bleed row highlight: the fill runs the whole panel width, the content stays on the gutter, and the 2 px selection tick sits in the gutter at x 2-4 (never moves the content).
/// Put it inside a gutter-padded body (the bleed is 12 px each side of the row).
class _Bleed extends StatelessWidget {
  const _Bleed({required this.child, this.fill, this.tick = false, this.height});
  final Widget child;
  final Color? fill;
  final bool tick;
  final double? height;
  @override
  Widget build(BuildContext context) => SizedBox(
        height: height,
        child: Stack(clipBehavior: Clip.none, children: [
          if (fill != null) Positioned(left: -_gutter, right: -_gutter, top: 0, bottom: 0, child: ColoredBox(color: fill!)),
          if (tick) Positioned(left: -(_gutter - 2), top: 4, bottom: 4, width: 2, child: const DecoratedBox(decoration: BoxDecoration(color: N.g95, borderRadius: BorderRadius.all(Radius.circular(1))))),
          height == null ? child : Positioned.fill(child: child),
        ]),
      );
}

/// A specimen of a row that lives in a gutter-padded body: the same 12 px gutters, so the bleed and the tick show.
Widget _gut(Widget w) => Padding(padding: const EdgeInsets.symmetric(horizontal: _gutter), child: w);

// ---- panel chrome ----

class _Frame extends StatelessWidget {
  const _Frame({required this.w, required this.child, this.head, this.foot});
  final double w;
  final Widget child;
  final Widget? head, foot;
  @override
  Widget build(BuildContext context) {
    final fit = _FitMode.of(context);
    return Container(
      width: w,
      height: fit ? null : _panelH,
      constraints: fit ? const BoxConstraints(maxHeight: _panelH) : null,
      clipBehavior: Clip.hardEdge,
      decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
      child: Column(mainAxisSize: fit ? MainAxisSize.min : MainAxisSize.max, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        ?head,
        fit ? Flexible(child: child) : Expanded(child: child),
        ?foot,
      ]),
    );
  }
}

class _Head extends StatelessWidget {
  const _Head({required this.name, this.kind = 'Shape group', this.trailing, this.sub});
  final String name, kind;
  final String? sub;
  final Widget? trailing;
  @override
  Widget build(BuildContext context) => Container(
        height: 48,
        padding: const EdgeInsets.symmetric(horizontal: _gutter),
        decoration: BoxDecoration(border: Border(bottom: BorderSide(color: _rule))),
        child: Row(children: [
          const SizedBox(width: 16, height: 16, child: CustomPaint(painter: _LayersPaint())),
          const SizedBox(width: 8),
          Expanded(
            child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Text(sub == null ? kind : '$kind · $sub', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
              const SizedBox(height: 4),
              Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title()),
            ]),
          ),
          if (trailing != null) ...[const SizedBox(width: 8), trailing!],
        ]),
      );
}

class _Foot extends StatelessWidget {
  const _Foot(this.m);
  final _M m;
  @override
  Widget build(BuildContext context) => _FootBar(m.log == 'nothing yet' ? '' : m.log, steps: m.steps);
}

/// The foot of a panel: a record of the last write, g76: the undo arrow with its count, then what was written. It does not exist until something was written (no "nothing yet").
class _FootBar extends StatelessWidget {
  const _FootBar(this.text, {this.steps = 0});
  final String text;
  final int steps;
  @override
  Widget build(BuildContext context) => text.isEmpty && steps == 0
      ? const SizedBox.shrink()
      : Container(
          height: _footH,
          padding: const EdgeInsets.symmetric(horizontal: _gutter),
          alignment: Alignment.centerLeft,
          decoration: BoxDecoration(color: N.g07, border: Border(top: BorderSide(color: _rule))),
          child: Row(children: [
            const SizedBox(width: 10, height: 10, child: CustomPaint(painter: _UndoPaint())),
            const SizedBox(width: 4),
            SizedBox(width: 16, child: Text('$steps', style: T.label(N.g76))),
            Expanded(child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
          ]),
        );
}

/// The undo arrow of the foot: a 270 degree arc with a head.
class _UndoPaint extends CustomPainter {
  const _UndoPaint();
  @override
  void paint(Canvas c, Size z) {
    final p = Paint()..color = N.g76..style = PaintingStyle.stroke..strokeWidth = 1;
    c.drawArc(Rect.fromLTWH(1, 2, 8, 7), -math.pi / 2, math.pi * 1.5, false, p);
    c.drawPath(Path()..moveTo(3.2, 0.5)..lineTo(5, 2)..lineTo(3.2, 3.5), p);
  }

  @override
  bool shouldRepaint(_UndoPaint o) => false;
}

/// A scrolling body with the panel's gutters.
class _Body extends StatefulWidget {
  const _Body({required this.children});
  final List<Widget> children;
  static const pad = EdgeInsets.fromLTRB(_gutter, 4, _gutter, 16);
  @override
  State<_Body> createState() => _BodyState();
}

class _BodyState extends State<_Body> {
  final _sc = ScrollController();
  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  /// The scroll cue: a 4 px thumb that is always drawn when there is more below (a cut group is never mistaken for the end).
  @override
  Widget build(BuildContext context) => RawScrollbar(
        controller: _sc,
        thumbVisibility: true,
        thickness: 4,
        radius: const Radius.circular(2),
        thumbColor: N.g38,
        crossAxisMargin: 2,
        child: SingleChildScrollView(
          controller: _sc,
          padding: _Body.pad,
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: widget.children),
        ),
      );
}

/// A group of rows: a rule above, a small caps title, an optional hint and trailing widget.
class _Sec extends StatelessWidget {
  const _Sec(this.title, this.children, {this.first = false, this.trailing, this.hint});
  final String title;
  final List<Widget> children;
  final bool first;
  final Widget? trailing;
  final String? hint;
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
        if (!first) Padding(padding: const EdgeInsets.only(top: 12), child: Container(height: 1, color: _rule)),
        SizedBox(
          height: 28,
          child: Row(children: [
            Expanded(child: Text(title.toUpperCase(), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g76).copyWith(letterSpacing: 1))),
            if (hint != null) Flexible(child: Padding(padding: const EdgeInsets.only(right: 8), child: Text(hint!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)))),
            ?trailing,
          ]),
        ),
        ...children,
      ]);
}

// ---- the mock document ----

enum _K { num, toggle, choice }

class _P {
  const _P(this.id, this.label, this.def, {this.unit = '', this.min, this.max, this.dec = 0, this.kind = _K.num, this.options = const []});
  final String id, label, unit;
  final double def;
  final double? min, max;
  final int dec;
  final _K kind;
  final List<String> options;
}

class _R {
  const _R(this.label, this.cells, {this.keyable = true});
  final String label;
  final List<_P> cells;
  final bool keyable;
  String get id => cells.first.id;
  bool get isNum => cells.first.kind == _K.num;
}
