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

class _Grp {
  const _Grp(this.title, this.rows);
  final String title;
  final List<_R> rows;
}

const _gTransform = _Grp('Transform  トランスフォーム', [
  _R('Anchor Point', [_P('anchor.x', 'X', 960, unit: 'px', dec: 1), _P('anchor.y', 'Y', 540, unit: 'px', dec: 1)]),
  _R('Position', [_P('pos.x', 'X', 960, unit: 'px', dec: 1), _P('pos.y', 'Y', 540, unit: 'px', dec: 1)]),
  _R('Scale', [_P('scale.x', 'X', 100, unit: '%', dec: 1), _P('scale.y', 'Y', 100, unit: '%', dec: 1)]),
  _R('Rotation', [_P('rot', '', 0, unit: '°', dec: 1)]),
  _R('Opacity', [_P('op', '', 100, unit: '%', min: 0, max: 100)]),
]);

const _gBlur = _Grp('Gaussian Blur  ガウスぼかし', [
  _R('Blurriness', [_P('blur.a', '', 0, unit: 'px', dec: 1, min: 0, max: 250)]),
  _R('Blur Dimensions', [_P('blur.d', '', 0, kind: _K.choice, options: ['Horizontal and Vertical', 'Horizontal', 'Vertical'])], keyable: false),
  _R('Repeat Edge Pixels', [_P('blur.e', '', 0, kind: _K.toggle)], keyable: false),
]);

const _gGlow = _Grp('Glow  グロー', [
  _R('Glow Threshold', [_P('glow.t', '', 60, unit: '%', min: 0, max: 100)]),
  _R('Glow Radius', [_P('glow.r', '', 10, unit: 'px', dec: 1, min: 0)]),
  _R('Glow Intensity', [_P('glow.i', '', 1, dec: 2, min: 0)]),
  _R('Glow Colors', [_P('glow.c', '', 0, kind: _K.choice, options: ['Original Colors', 'A & B Colors', 'Arbitrary Map'])], keyable: false),
]);

const _gWave = _Grp('Wave Warp  波形ワープ', [
  _R('Wave Type', [_P('wave.k', '', 0, kind: _K.choice, options: ['Sine', 'Triangle', 'Square', 'Noise'])], keyable: false),
  _R('Wave Height', [_P('wave.h', '', 20, unit: 'px', dec: 1)]),
  _R('Wave Width', [_P('wave.w', '', 20, unit: 'px', dec: 1)]),
  _R('Direction', [_P('wave.d', '', 90, unit: '°', dec: 1)]),
  _R('Wave Speed', [_P('wave.s', '', 1, dec: 2)]),
  _R('Phase', [_P('wave.p', '', 0, unit: '°', dec: 1)]),
]);

const _gTurb = _Grp('Turbulent Displace  タービュレント', [
  _R('Amount', [_P('turb.a', '', 50, dec: 1)]),
  _R('Size', [_P('turb.s', '', 100, dec: 1)]),
  _R('Offset (Turbulence)', [_P('turb.x', 'X', 960, unit: 'px', dec: 1), _P('turb.y', 'Y', 540, unit: 'px', dec: 1)]),
  _R('Complexity', [_P('turb.c', '', 1, dec: 1, min: 1, max: 10)]),
  _R('Evolution', [_P('turb.e', '', 0, unit: '°', dec: 1)]),
  _R('Evolution Options: Cycle Evolution Revolutions', [_P('turb.r', '', 1, dec: 1)]),
]);

const _gShadow = _Grp('Drop Shadow  ドロップシャドウ', [
  _R('Shadow Color', [_P('sh.c', '', 0, kind: _K.choice, options: ['Black', 'Brand 04', 'Custom'])], keyable: false),
  _R('Opacity', [_P('sh.o', '', 50, unit: '%', min: 0, max: 100)]),
  _R('Direction', [_P('sh.d', '', 135, unit: '°', dec: 1)]),
  _R('Distance', [_P('sh.x', '', 5, unit: 'px', dec: 1)]),
  _R('Softness', [_P('sh.f', '', 0, unit: 'px', dec: 1)]),
]);

const _gText = _Grp('Text  テキスト', [
  _R('Font Size', [_P('tx.s', '', 48, unit: 'px', dec: 1, min: 1)]),
  _R('Tracking', [_P('tx.t', '', 0, dec: 0)]),
  _R('Leading', [_P('tx.l', '', 120, unit: '%', dec: 0)]),
]);

const _allGroups = [_gTransform, _gBlur, _gGlow, _gWave, _gTurb, _gShadow, _gText];

/// What the layer looks like when the panel opens: some values already changed, some animated (so the diff and the key states have something to say).
const _startValues = <String, double>{
  'pos.x': 1180.5, 'pos.y': 540, 'scale.x': 112, 'scale.y': 112, 'rot': 15, 'op': 80, 'blur.a': 24, 'blur.e': 1, 'glow.r': 38, 'glow.i': 1.35,
  'wave.h': 34, 'turb.a': 62, 'sh.o': 65, 'sh.f': 12, 'tx.t': 40, 'blur.d': 1,
};

const _startKeys = <String, Map<int, double>>{
  'pos.x': {0: 960, 48: 1180.5, 96: 1320},
  'pos.y': {0: 540, 48: 540, 96: 400},
  'rot': {12: 0, 84: 15},
  'op': {0: 0, 24: 100, 90: 80},
  'blur.a': {30: 0, 60: 24},
  'glow.r': {0: 10, 72: 38},
};

String _two(int n) => n.toString().padLeft(2, '0');
String _fmtT(int f) => '00:${_two(f ~/ 30)}:${_two(f % 30)}';
const _fps = 30, _end = 120;

double _textW(String t, TextStyle st) => (TextPainter(text: TextSpan(text: t, style: st), textDirection: TextDirection.ltr, maxLines: 1)..layout()).width;

String _fmtNum(double v, int dec) {
  var d = dec;
  var s = v.toStringAsFixed(d);
  while (v != 0 && double.parse(s) == 0 && d < 4) {
    d++;
    s = v.toStringAsFixed(d);
  }
  return s == '-0' ? '0' : s;
}

double _round(double v) => (v * 10000).round() / 10000;

/// The scrub multiplier of the one rule (D-I3): Shift x10, Alt x0.1.
double _mult() {
  final k = HardwareKeyboard.instance;
  if (k.isShiftPressed) {
    return 10;
  }
  if (k.isAltPressed) {
    return .1;
  }
  return 1;
}

class _M extends ChangeNotifier {
  _M({List<_Grp> groups = _allGroups, Map<String, double> start = _startValues, Map<String, Map<int, double>> keys = _startKeys, this.animated = true}) {
    for (final g in groups) {
      for (final r in g.rows) {
        rows[r.id] = r;
        for (final p in r.cells) {
          byId[p.id] = p;
          def[p.id] = p.def;
          v[p.id] = start[p.id] ?? p.def;
        }
      }
    }
    if (animated) {
      keys.forEach((id, k) {
        if (byId.containsKey(id)) {
          tracks[id] = SplayTreeMap<int, double>.of(k);
        }
      });
    }
  }
  final bool animated;
  final v = <String, double>{}, def = <String, double>{};
  final byId = <String, _P>{};
  final rows = <String, _R>{};
  final tracks = <String, SplayTreeMap<int, double>>{};
  final sev = <String, List<double>>{}; // several layers selected: one value per layer
  int frame = 48, steps = 0;
  bool animate = false;
  String log = 'nothing yet';
  Map<String, double>? _sv;
  Map<String, Map<int, double>>? _st;
  Map<String, List<double>>? _ss;

  bool isMixed(String id) {
    final s = sev[id];
    return s != null && s.any((x) => x != s.first);
  }

  double at(String id) {
    final t = tracks[id];
    if (t == null || t.isEmpty) {
      return v[id]!;
    }
    final f = frame;
    if (f <= t.firstKey()!) {
      return t[t.firstKey()]!;
    }
    if (f >= t.lastKey()!) {
      return t[t.lastKey()]!;
    }
    final a = t.lastKeyBefore(f + 1)!, b = t.firstKeyAfter(f)!;
    final u = (f - a) / (b - a);
    return t[a]! + (t[b]! - t[a]!) * u;
  }

  bool changed(String id) {
    final s = sev[id];
    if (s != null) {
      return s.any((x) => x != def[id]);
    }
    return _round(at(id)) != _round(def[id]!);
  }

  int changedCount(Iterable<String> ids) => ids.where(changed).length;

  KeyS stateOf(String id) {
    final t = tracks[id];
    if (t == null || t.isEmpty) {
      return KeyS.off;
    }
    return t.containsKey(frame) ? KeyS.at : KeyS.anim;
  }

  void begin() {
    _sv ??= Map.of(v);
    _st ??= {for (final e in tracks.entries) e.key: Map.of(e.value)};
    _ss ??= {for (final e in sev.entries) e.key: List.of(e.value)};
  }

  void end(String what) {
    final a = _sv, b = _st, c = _ss;
    _sv = null;
    _st = null;
    _ss = null;
    if (a == null) {
      return;
    }
    final same = a.entries.every((e) => v[e.key] == e.value) && _sameTracks(b!) && _sameSev(c!);
    if (!same) {
      steps++;
      log = what;
    }
    notifyListeners();
  }

  bool _sameTracks(Map<String, Map<int, double>> b) {
    if (b.length != tracks.length) {
      return false;
    }
    for (final e in tracks.entries) {
      final o = b[e.key];
      if (o == null || o.length != e.value.length || e.value.entries.any((x) => o[x.key] != x.value)) {
        return false;
      }
    }
    return true;
  }

  bool _sameSev(Map<String, List<double>> c) {
    for (final e in sev.entries) {
      final o = c[e.key];
      if (o == null) {
        return false;
      }
      for (var i = 0; i < o.length; i++) {
        if (o[i] != e.value[i]) {
          return false;
        }
      }
    }
    return c.length == sev.length;
  }

  /// Esc during a gesture: everything is where the gesture found it.
  void cancel() {
    if (_sv == null) {
      return;
    }
    v
      ..clear()
      ..addAll(_sv!);
    tracks
      ..clear()
      ..addEntries(_st!.entries.map((e) => MapEntry(e.key, SplayTreeMap<int, double>.of(e.value))));
    sev
      ..clear()
      ..addEntries(_ss!.entries.map((e) => MapEntry(e.key, List.of(e.value))));
    _sv = null;
    _st = null;
    _ss = null;
    log = 'Esc: put back';
    notifyListeners();
  }

  double clamp(String id, double x) {
    final p = byId[id]!;
    var y = x;
    if (p.min != null && y < p.min!) {
      y = p.min!;
    }
    if (p.max != null && y > p.max!) {
      y = p.max!;
    }
    return _round(y);
  }

  /// Write one value. A value with a track writes a key at the playhead; a value without one only creates a track when Animate is on.
  void set(String id, double x, {bool notify = true}) {
    final y = clamp(id, x);
    final t = tracks[id];
    if (t != null) {
      t[frame] = y;
    } else if (animate && byId[id]!.kind == _K.num && (rows[_rowOf(id)]?.keyable ?? false)) {
      tracks[id] = SplayTreeMap<int, double>()..[frame] = y;
    } else {
      v[id] = y;
    }
    if (notify) {
      notifyListeners();
    }
  }

  String _rowOf(String id) {
    for (final r in rows.values) {
      if (r.cells.any((c) => c.id == id)) {
        return r.id;
      }
    }
    return id;
  }

  void add(String id, double dx) {
    final s = sev[id];
    if (s != null) {
      for (var i = 0; i < s.length; i++) {
        s[i] = clamp(id, s[i] + dx);
      }
      notifyListeners();
      return;
    }
    set(id, at(id) + dx);
  }

  void typed(String id, double x) {
    begin();
    sev.remove(id);
    set(id, x, notify: false);
    end('typed ${byId[id]!.label.isEmpty ? rows[_rowOf(id)]!.label : byId[id]!.label} = ${_fmtNum(clamp(id, x), byId[id]!.dec)}');
  }

  void reset(String id) {
    begin();
    sev.remove(id);
    set(id, def[id]!, notify: false);
    end('reset ${rows[_rowOf(id)]?.label ?? id}');
  }

  void resetMany(Iterable<String> ids, String what) {
    begin();
    for (final id in ids) {
      sev.remove(id);
      if (byId[id]!.kind == _K.num) {
        set(id, def[id]!, notify: false);
      } else {
        v[id] = def[id]!;
      }
    }
    end(what);
  }

  /// Group reset (I6-c): values go to default; keys, animated rows' other keys and driven values are left alone. Returns what was kept.
  String resetKeep(Iterable<String> ids, Set<String> driven, String name) {
    begin();
    var n = 0, anim = 0, drv = 0;
    for (final id in ids) {
      if (driven.contains(id)) {
        drv++;
        continue;
      }
      if (tracks.containsKey(id)) {
        if (tracks[id]!.containsKey(frame)) {
          tracks[id]![frame] = def[id]!;
          n++;
        } else {
          anim++;
        }
        continue;
      }
      if (byId[id]!.kind == _K.num) {
        v[id] = def[id]!;
      } else {
        v[id] = def[id]!;
      }
      n++;
    }
    final kept = [if (anim > 0) '$anim animated', if (drv > 0) '$drv driven'].join(' + ');
    end('reset $name: $n values${kept.isEmpty ? '' : ', kept $kept'}');
    return kept;
  }

  void toggleKey(_R r) {
    begin();
    final here = stateOf(r.id) == KeyS.at;
    for (final p in r.cells) {
      final t = tracks[p.id];
      if (here) {
        t?.remove(frame);
        if (t != null && t.isEmpty) {
          v[p.id] = v[p.id]!;
          tracks.remove(p.id);
        }
      } else if (t == null) {
        tracks[p.id] = SplayTreeMap<int, double>()..[frame] = v[p.id]!;
      } else {
        t[frame] = at(p.id);
      }
    }
    end(here ? 'removed key ${r.label} @ ${_fmtT(frame)}' : 'key ${r.label} @ ${_fmtT(frame)}');
  }

  int? neighbour(_R r, {required bool back}) {
    final t = tracks[r.id];
    if (t == null || t.isEmpty) {
      return null;
    }
    return back ? t.lastKeyBefore(frame) : t.firstKeyAfter(frame);
  }

  void seek(int f, {String? why}) {
    frame = f.clamp(0, _end);
    if (why != null) {
      log = why;
    }
    notifyListeners();
  }

  void poke() => notifyListeners();

  void note(String s) {
    log = s;
    notifyListeners();
  }

  void toggleAnimate() {
    animate = !animate;
    log = animate ? 'Animate on' : 'Animate off';
    notifyListeners();
  }

  /// Mix the value of three layers so that the field has something to say about "several".
  void mix(Map<String, List<double>> m) {
    sev
      ..clear()
      ..addAll(m);
    notifyListeners();
  }
}

/// A model that lives as long as its panel and rebuilds it when something changes.
class _Host extends StatefulWidget {
  const _Host({super.key, required this.make, required this.build});
  final _M Function() make;
  final Widget Function(BuildContext, _M) build;
  @override
  State<_Host> createState() => _HostState();
}

class _HostState extends State<_Host> {
  late final _M m = widget.make();
  @override
  void dispose() {
    m.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(listenable: m, builder: (context, _) => widget.build(context, m));
}

// ---- the number cell: one control, five grammars ----

enum _Gr { face, label, ladder, stepper }

const _bands = [10.0, 1.0, .1, .01];
const _bandWords = ['x10', 'x1', 'x0.1', 'x0.01'];

/// The one ladder of a field's edge: rest g20, hover g38, focused / pressed g63, editing accent, rejected danger. Pressed also lifts the fill.
enum _Pose { rest, hover, focus, press, edit, bad }

Color _faceLine(_Pose p) => switch (p) { _Pose.rest => N.g20, _Pose.hover => N.g38, _Pose.focus => N.g76, _Pose.press => N.g63, _Pose.edit => Role.selected, _Pose.bad => Role.error };

class _Cell extends StatefulWidget {
  const _Cell(this.m, this.p, {this.g = _Gr.face, this.tag, this.enabled = true, this.narrow = false, this.ghost, this.pose, this.startBad = false});
  final _M m;
  final _P p;
  final _Gr g;
  final _Pose? pose; // a specimen: the cell is drawn in this pose with the same code (the Parts sheets); null = live
  final String? tag, ghost; // ghost: the value this cell would have after a pending reset (I6-d), shown instead of the real one
  final bool enabled, narrow, startBad; // startBad: the rejected look from the first frame (a field that already refused a value)
  @override
  State<_Cell> createState() => _CellState();
}

class _CellState extends State<_Cell> {
  bool _edit = false, _hover = false, _bad = false, _focus = false;
  int _band = 1;
  bool _live = false;
  double _dy = 0;
  final _link = LayerLink();
  final _portal = OverlayPortalController();
  final _ctl = TextEditingController();
  final _node = FocusNode(debugLabel: 'cell');
  final _efn = FocusNode(debugLabel: 'cell-edit');
  Timer? _hold;

  @override
  void initState() {
    super.initState();
    _bad = widget.startBad;
    _efn.addListener(() {
      if (!_efn.hasFocus && _edit) {
        _commit(_ctl.text);
      }
    });
  }

  @override
  void dispose() {
    _hold?.cancel();
    _ctl.dispose();
    _node.dispose();
    _efn.dispose();
    super.dispose();
  }

  _P get p => widget.p;
  _M get m => widget.m;
  String get _label => p.label.isEmpty ? (m.rows[p.id]?.label ?? p.id) : '${m.rows[p.id]?.label ?? ''} ${p.label}'.trim();

  void _open() {
    if (!widget.enabled) {
      return;
    }
    _ctl.text = m.isMixed(p.id) ? '' : _fmtNum(m.at(p.id), p.dec);
    _ctl.selection = TextSelection(baseOffset: 0, extentOffset: _ctl.text.length);
    setState(() {
      _edit = true;
      _bad = false;
    });
    _efn.requestFocus();
  }

  void _commit(String s) {
    final x = double.tryParse(s.trim().replaceAll(RegExp(r'[^0-9eE+\-.]'), ''));
    setState(() => _edit = false);
    if (x == null) {
      setState(() => _bad = s.trim().isNotEmpty);
      if (s.trim().isNotEmpty) {
        m.note('"${s.trim()}" is not a number: kept ${_fmtNum(m.at(p.id), p.dec)}');
      }
      return;
    }
    m.typed(p.id, x);
  }

  void _cancelEdit() {
    setState(() => _edit = false);
    _node.requestFocus();
  }

  void _start() {
    _live = true;
    _dy = 0;
    _band = 1;
    m.begin();
    _node.requestFocus();
    if (widget.g == _Gr.ladder) {
      _portal.show();
    }
  }

  void _update(DragUpdateDetails d) {
    if (!_live) {
      return;
    }
    if (widget.g == _Gr.ladder) {
      final k = HardwareKeyboard.instance;
      _dy += d.delta.dy;
      var b = (1 + (_dy / 20).round()).clamp(0, 3);
      if (k.isShiftPressed) {
        b = (b - 1).clamp(0, 3);
      }
      if (k.isAltPressed) {
        b = (b + 1).clamp(0, 3);
      }
      if (b != _band) {
        setState(() => _band = b);
      }
      m.add(p.id, d.delta.dx * _bands[b]);
    } else {
      m.add(p.id, d.delta.dx * _mult());
    }
  }

  void _stop() {
    if (!_live) {
      return;
    }
    _live = false;
    _portal.hide();
    final extra = widget.g == _Gr.ladder ? ' (${_bandWords[_band]})' : '';
    m.end('scrub $_label$extra');
    setState(() => _band = 1);
  }

  void _stepHold(int dir) {
    void once() {
      m.begin();
      m.add(p.id, dir * _mult());
      m.end('step $_label ${dir > 0 ? '+' : '-'}${_fmtNum(_mult(), _mult() < 1 ? 1 : 0)}');
    }

    once();
    var n = 0;
    _hold?.cancel();
    _hold = Timer(const Duration(milliseconds: 380), () {
      _hold = Timer.periodic(const Duration(milliseconds: 70), (_) {
        n++;
        m.begin();
        m.add(p.id, dir * _mult() * (n > 12 ? 5 : 1));
        m.end('hold $_label ${dir > 0 ? '+' : '-'}');
      });
    });
  }

  void _stepRelease() {
    _hold?.cancel();
    _hold = null;
  }

  @override
  Widget build(BuildContext context) {
    if (widget.ghost != null && p.kind != _K.num) {
      return Container(
        height: _cellH,
        alignment: Alignment.centerRight,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: Text('→ ${widget.ghost}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Role.of(N.g76, Role.changed))),
      );
    }
    if (p.kind == _K.toggle) {
      final on = m.v[p.id]! > .5;
      return FittedBox(
        fit: BoxFit.scaleDown,
        alignment: Alignment.centerRight,
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Opacity(
            opacity: widget.enabled ? 1 : .5,
            child: PillSwitch(
              on: on,
              onChanged: !widget.enabled
                  ? null
                  : (b) {
                      m.begin();
                      m.v[p.id] = b ? 1 : 0;
                      m.end('${m.rows[p.id]?.label}: ${b ? 'On' : 'Off'}');
                    },
            ),
          ),
          const SizedBox(width: 8),
          SizedBox(width: 18, child: Text(on ? 'On' : 'Off', style: T.label(!widget.enabled ? N.g63 : (on ? N.g100 : N.g76)))),
        ]),
      );
    }
    if (p.kind == _K.choice) {
      final i = m.v[p.id]!.round().clamp(0, p.options.length - 1);
      final off = !widget.enabled;
      bool? pin(_Pose x) => widget.pose == null ? null : widget.pose == x;
      return _Ctl(
        enabled: !off,
        hover: pin(_Pose.hover),
        focus: pin(_Pose.focus),
        pressed: pin(_Pose.press),
        onTap: () {
          m.begin();
          m.v[p.id] = ((i + 1) % p.options.length).toDouble();
          m.end('${m.rows[p.id]?.label}: ${p.options[(i + 1) % p.options.length]}');
        },
        builder: (_, h, f, pr) => Container(
          height: _cellH,
          clipBehavior: Clip.antiAlias,
          decoration: BoxDecoration(color: off ? N.g10 : (pr ? N.g15 : N.g07), borderRadius: BorderRadius.circular(4), border: Border.all(color: off ? const Color(0x00000000) : _faceLine(f ? _Pose.focus : (pr ? _Pose.press : (h ? _Pose.hover : _Pose.rest))))),
          child: CustomPaint(
            painter: off ? const _HatchPaint() : null,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(children: [
                Expanded(child: Text(p.options[i], maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(off ? N.g63 : N.g91))),
                const SizedBox(width: 4),
                Text('›', style: T.name(h || f || pr ? N.g95 : N.g63)),
              ]),
            ),
          ),
        ),
      );
    }
    final mixed = m.isMixed(p.id);
    final ladderDec = widget.g == _Gr.ladder && _live ? math.max(p.dec, _band >= 2 ? _band - 1 : 0) : p.dec;
    final dim = !widget.enabled;
    final forced = widget.pose;
    final pose = forced ?? (_bad ? _Pose.bad : (_edit ? _Pose.edit : (_live ? _Pose.press : (_focus ? _Pose.focus : (_hover && !dim ? _Pose.hover : _Pose.rest)))));
    final tagW = widget.tag == null || widget.tag!.isEmpty ? null : Text(widget.tag!, style: T.label(pose == _Pose.hover && widget.g == _Gr.label ? N.g100 : N.g76));
    final isChanged = m.changed(p.id) && (Role.grey || !mixed) && !dim && widget.ghost == null && pose != _Pose.edit;
    bool showTag0(double w) => tagW != null && widget.g != _Gr.stepper && w >= 66;
    final tagRoom = widget.tag == null ? 0.0 : _textW(widget.tag!, T.label(N.g76)) + 4;
    Widget value(double w) {
      // one narrowing rule, in this order: the value is always whole. (1) the unit goes first, (2) the value may shrink to 10 px (11 -> 10, FittedBox scaleDown), (3) only then decimals drop one by one; a value that is not zero never reads 0
      final tagOn = showTag0(w);
      final base = w - 2 - (tagOn ? 8 : 16) - (tagOn ? tagRoom : 0) - 1;
      final unitW = p.unit.isEmpty ? 0.0 : 4 + _textW(p.unit, T.label(N.g76));
      var text = mixed ? '—' : _fmtNum(m.at(p.id), ladderDec);
      var showUnit = p.unit.isNotEmpty && w >= 96 && !mixed;
      if (!mixed) {
        if (showUnit && _textW(text, T.value(N.g91)) * (10 / 11) + unitW > base) {
          showUnit = false;
        }
        for (var d = ladderDec; d > 0 && _textW(text, T.value(N.g91)) * (10 / 11) > base; d--) {
          text = _fmtNum(m.at(p.id), d - 1);
        }
      }
      if (forced == _Pose.edit) {
        return Align(alignment: Alignment.centerRight, child: ColoredBox(color: C.mode.withValues(alpha: .45), child: Text(text, maxLines: 1, style: T.value(N.g100))));
      }
      if (_edit) {
        return Focus(
          onKeyEvent: (_, e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
              _cancelEdit();
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: EditableText(
            controller: _ctl,
            focusNode: _efn,
            autofocus: true,
            textAlign: TextAlign.right,
            style: T.value(N.g100),
            cursorColor: N.g95,
            backgroundCursorColor: N.g44,
            selectionColor: Role.selected.withValues(alpha: .45),
            keyboardType: TextInputType.number,
            onSubmitted: _commit,
            maxLines: 1,
          ),
        );
      }
      if (widget.ghost != null) {
        final x = double.tryParse(widget.ghost!), narrow = w < 80;
        return Align(alignment: Alignment.centerRight, child: Text(narrow ? (x == null ? widget.ghost! : _fmtNum(x, 0)) : '→ ${widget.ghost}', maxLines: 1, overflow: TextOverflow.clip, style: T.value(Role.of(N.g76, Role.changed))));
      }
      return Row(mainAxisAlignment: MainAxisAlignment.end, children: [
        if (pose == _Pose.bad) const ErrMark(size: 10, gap: 4),
        Flexible(child: FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerRight, child: Text(text, maxLines: 1, softWrap: false, textAlign: TextAlign.right, style: T.value(dim ? N.g63 : (mixed && !Role.grey ? Role.info : (m.changed(p.id) ? Role.of(N.g95, Role.changed) : N.g91)))))),
        if (showUnit) Padding(padding: const EdgeInsets.only(left: 4), child: Text(p.unit, style: T.label(N.g76))),
      ]);
    }

    Widget face(double w) {
      final showTag = tagW != null && widget.g != _Gr.stepper && w >= 66;
      final grabbable = !dim && widget.g != _Gr.label && widget.g != _Gr.stepper && !_edit;
      final hot = pose == _Pose.hover || pose == _Pose.press || pose == _Pose.focus;
      return MouseRegion(
          cursor: dim ? SystemMouseCursors.forbidden : (_edit ? SystemMouseCursors.text : SystemMouseCursors.resizeLeftRight),
          onEnter: (_) => setState(() => _hover = true),
          onExit: (_) => setState(() => _hover = false),
          child: CompositedTransformTarget(
            link: _link,
            child: OverlayPortal(
              controller: _portal,
              overlayChildBuilder: (_) => Stack(children: [
                CompositedTransformFollower(
                  link: _link,
                  showWhenUnlinked: false,
                  targetAnchor: Alignment.centerLeft,
                  followerAnchor: Alignment.centerRight,
                  offset: const Offset(-8, 0),
                  child: Align(alignment: Alignment.centerRight, child: _Ladder(band: _band)),
                ),
              ]),
              child: Container(
                height: _cellH,
                clipBehavior: Clip.antiAlias,
                padding: EdgeInsets.symmetric(horizontal: showTag ? 4 : 8),
                decoration: BoxDecoration(
                  color: pose == _Pose.press ? N.g15 : (dim ? N.g10 : N.g07),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: dim ? const Color(0x00000000) : _faceLine(pose)),
                ),
                child: CustomPaint(
                  painter: dim ? const _HatchPaint() : null,
                  child: Stack(fit: StackFit.expand, clipBehavior: Clip.none, children: [
                    Row(children: [
                      if (widget.g == _Gr.stepper) _StepBtn('−', visible: (pose == _Pose.hover || pose == _Pose.press) && !dim, onDown: () => _stepHold(-1), onUp: _stepRelease, rest: dim ? null : _grip(N.g38)),
                      if (showTag)
                        if (widget.g == _Gr.label)
                          _LabelHandle(m: m, p: p, child: Padding(padding: const EdgeInsets.only(right: 4), child: tagW))
                        else
                          Padding(padding: const EdgeInsets.only(right: 4), child: tagW),
                      Expanded(child: value(w)),
                      if (widget.g == _Gr.stepper) _StepBtn('+', visible: (pose == _Pose.hover || pose == _Pose.press) && !dim, onDown: () => _stepHold(1), onUp: _stepRelease),
                    ]),
                    if (grabbable) Positioned(left: showTag ? -4 : -5, top: 0, bottom: 0, child: Center(child: _grip(hot ? N.g63 : N.g38))),
                    if (isChanged) Positioned(left: showTag ? -5 : -7, top: 0, child: const CustomPaint(size: Size(4, 4), painter: _CornerPaint())),
                  ]),
                ),
              ),
            ),
          ),
        );
    }

    Widget wrapGestures(Widget child) {
      if (dim) {
        return child;
      }
      final labelMode = widget.g == _Gr.label;
      return Focus(
        focusNode: _node,
        onFocusChange: (f) => setState(() => _focus = f),
        onKeyEvent: (_, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _live) {
            _live = false;
            _portal.hide();
            m.cancel();
            return KeyEventResult.handled;
          }
          if (e is KeyDownEvent && (e.logicalKey == LogicalKeyboardKey.arrowUp || e.logicalKey == LogicalKeyboardKey.arrowDown)) {
            m.begin();
            m.add(p.id, (e.logicalKey == LogicalKeyboardKey.arrowUp ? 1 : -1) * _mult());
            m.end('arrow $_label');
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Listener(
          onPointerSignal: (e) {
            if (e is PointerScrollEvent && _hover) {
              m.begin();
              m.add(p.id, (e.scrollDelta.dy > 0 ? -1 : 1) * _mult());
              m.end('wheel $_label');
            }
          },
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            // the label grammar has no double-click on the value, so a click opens the field at once
            onTap: _open,
            onDoubleTap: labelMode ? null : () => m.reset(p.id),
            onPanStart: widget.g == _Gr.label ? null : (_) => _start(),
            onPanUpdate: widget.g == _Gr.label ? null : _update,
            onPanEnd: widget.g == _Gr.label ? null : (_) => _stop(),
            onPanCancel: widget.g == _Gr.label ? null : _stop,
            child: child,
          ),
        ),
      );
    }

    return LayoutBuilder(builder: (context, box) => wrapGestures(face(box.maxWidth)));
  }
}

class _StepBtn extends StatefulWidget {
  const _StepBtn(this.glyph, {required this.visible, required this.onDown, required this.onUp, this.hot = false, this.down = false, this.rest});
  final String glyph;
  final Widget? rest; // what the slot shows while the button is hidden (a grip, so the field reads as draggable)
  final bool visible, hot, down; // hot / down: a specimen's hover / pressed
  final VoidCallback onDown, onUp;
  @override
  State<_StepBtn> createState() => _StepBtnState();
}

class _StepBtnState extends State<_StepBtn> {
  bool _d = false;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: 20,
        child: widget.visible
            ? GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () {},
                child: Listener(
                  onPointerDown: (_) {
                    setState(() => _d = true);
                    widget.onDown();
                  },
                  onPointerUp: (_) {
                    setState(() => _d = false);
                    widget.onUp();
                  },
                  onPointerCancel: (_) {
                    setState(() => _d = false);
                    widget.onUp();
                  },
                  // rest: bare glyph g63. hover: g20 fill, g95. pressed: g26 fill, g100.
                  child: Hov(builder: (_, h) {
                    final dn = _d || widget.down, hv = h || widget.hot;
                    return Container(color: dn ? N.g26 : (hv ? N.g20 : null), alignment: Alignment.center, child: Text(widget.glyph, style: T.name(dn ? N.g100 : (hv ? N.g95 : N.g63)).copyWith(fontSize: 13)));
                  }),
                ),
              )
            : (widget.rest == null ? null : Center(child: widget.rest)),
      );
}

/// A label that is a handle: drag scrubs, a double-click goes to the default.
class _LabelHandle extends StatefulWidget {
  const _LabelHandle({required this.m, required this.p, required this.child});
  final _M m;
  final _P p;
  final Widget child;
  @override
  State<_LabelHandle> createState() => _LabelHandleState();
}

class _LabelHandleState extends State<_LabelHandle> {
  bool _live = false;
  @override
  Widget build(BuildContext context) {
    final m = widget.m, p = widget.p;
    final lab = m.rows[p.id]?.label ?? p.id;
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onDoubleTap: () => m.reset(p.id),
        onPanStart: (_) {
          _live = true;
          m.begin();
        },
        onPanUpdate: (d) {
          if (_live) {
            m.add(p.id, d.delta.dx * _mult());
          }
        },
        onPanEnd: (_) {
          _live = false;
          m.end('scrub $lab (label)');
        },
        onPanCancel: () {
          _live = false;
          m.end('scrub $lab (label)');
        },
        child: widget.child,
      ),
    );
  }
}

/// I2-c: the ladder that is visible only while the value is held.
class _Ladder extends StatelessWidget {
  const _Ladder({required this.band});
  final int band;
  @override
  Widget build(BuildContext context) => Container(
        width: 60,
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g26), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 14, offset: const Offset(0, 5))]),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (var i = 0; i < 4; i++)
            Container(
              height: 20,
              alignment: Alignment.centerLeft,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: i == band ? N.g20 : null),
              foregroundDecoration: i == band ? const BoxDecoration(border: Border(left: BorderSide(color: N.g95, width: 2))) : null,
              child: Text(_bandWords[i], style: T.value(i == band ? N.g95 : N.g63)),
            ),
        ]),
      );
}

// ---- rows ----

/// One row: the label column, the cells, an optional trailing column (key mark, reset). Every row has the same columns so the fields end at one edge.
Widget _shell({required Widget label, required Widget cells, Widget? trail, double labW = _labW, double trailW = _keyW, Widget? under, Color? band, bool selected = false}) => Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _Bleed(
          fill: selected ? N.g20 : band,
          tick: selected,
          child: Container(
            constraints: const BoxConstraints(minHeight: _rowH),
            child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
              SizedBox(width: labW, child: label),
              Expanded(child: cells),
              SizedBox(width: trailW, child: trail),
            ]),
          ),
        ),
        ?under,
      ],
    );

Widget _cellsOf(_R r, _M m, {_Gr g = _Gr.face, bool dock = false, bool enabled = true, Map<String, String>? ghost}) {
  if (r.cells.length == 1) {
    return _Cell(m, r.cells.first, g: g, narrow: dock, enabled: enabled, ghost: ghost?[r.cells.first.id]);
  }
  return Row(children: [
    for (final (i, p) in r.cells.indexed) ...[
      if (i > 0) const SizedBox(width: 4),
      Expanded(child: _Cell(m, p, g: g, tag: p.label.isEmpty ? null : p.label, narrow: dock, enabled: enabled, ghost: ghost?[p.id])),
    ],
  ]);
}

/// The standard row. [tone] picks the label's grey, [trail] fills the key column, [labelHandle] makes the label a scrub handle.
Widget _prow(_R r, _M m, {_Gr g = _Gr.face, bool dock = false, Widget? trail, Color? tone, VoidCallback? labelDbl, Widget? under, bool enabled = true, double? labW, Widget? lead, Color? band, bool selected = false, double? trailW, Map<String, String>? ghost, Widget Function(Widget)? wrapLabel, bool labelHot = false}) {
  final changed = r.cells.any((c) => _chg(m, c.id));
  final single = r.cells.length == 1;
  final handle = g == _Gr.label && single && r.isNum && enabled;
  Widget labelRow([bool hot = false]) => Row(children: [
        if (lead != null) ...[lead, const SizedBox(width: 4)],
        Expanded(child: Text(r.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: _labStyle(changed: changed, enabled: enabled, hot: hot, tone: tone))),
        if (handle) ...[const SizedBox(width: 4), _grip(hot ? N.g63 : N.g38)],
      ]);
  Widget text = labelRow();
  if (g == _Gr.label && single && r.isNum) {
    text = Hov(cursor: SystemMouseCursors.resizeLeftRight, builder: (_, h) => _LabelHandle(m: m, p: r.cells.first, child: Padding(padding: const EdgeInsets.only(right: 8), child: labelRow(h || labelHot))));
  } else if (labelDbl != null) {
    text = GestureDetector(behavior: HitTestBehavior.opaque, onDoubleTap: labelDbl, child: Padding(padding: const EdgeInsets.only(right: 8), child: text));
  } else {
    text = Padding(padding: const EdgeInsets.only(right: 8), child: text);
  }
  if (wrapLabel != null) {
    text = wrapLabel(text);
  }
  return _shell(
    label: text,
    cells: _cellsOf(r, m, g: g, dock: dock, enabled: enabled, ghost: ghost),
    trail: trail,
    labW: labW ?? (r.cells.length == 1 && r.cells.first.kind == _K.toggle ? (dock ? 104 : 136) : (dock ? _labWDock : _labW)),
    under: under,
    band: band,
    selected: selected,
    trailW: trailW ?? _keyW,
  );
}

// ---- key mark (own painter: the shapes of the lab's KeyMark, plus the "will key" state of I4-b) ----

class _Key extends StatelessWidget {
  const _Key(this.s, {this.will = false, this.onTap, this.onLong, this.onSecondary, this.enabled = true, this.hover, this.focus, this.pressed});
  final KeyS s;
  final bool will, enabled;
  final bool? hover, focus, pressed; // pinned states (Parts)
  final VoidCallback? onTap, onLong, onSecondary;

  /// rest: the mark alone. hover g15 plate; focus 1 px g76 edge; pressed g26 plate. Disabled: no plate, no state.
  @override
  Widget build(BuildContext context) => _Ctl(
        enabled: enabled,
        onTap: onTap,
        onLong: onLong,
        onSecondary: onSecondary,
        hover: hover,
        focus: focus,
        pressed: pressed,
        builder: (_, h, f, p) => Container(
          width: _keyW,
          height: _cellH,
          decoration: BoxDecoration(color: p ? N.g26 : (h ? N.g15 : null), borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g76 : const Color(0x00000000))),
          child: CustomPaint(painter: _KeyPaint(s, h || p, will, enabled)),
        ),
      );
}

/// A dashed outline (2 on, 2 off): the shape of "present but not available".
void _dashed(Canvas c, Path p, Paint paint) {
  for (final m in p.computeMetrics()) {
    for (var d = 0.0; d < m.length; d += 4) {
      c.drawPath(m.extractPath(d, math.min(d + 2, m.length)), paint);
    }
  }
}

class _KeyPaint extends CustomPainter {
  const _KeyPaint(this.s, this.hover, this.will, this.enabled);
  final KeyS s;
  final bool hover, will, enabled;
  @override
  void paint(Canvas c, Size z) {
    final o = Offset(z.width / 2, z.height / 2), r = s == KeyS.at ? (hover ? 6 : 5.2) : 4.6;
    final dia = Path()..moveTo(o.dx, o.dy - r)..lineTo(o.dx + r, o.dy)..lineTo(o.dx, o.dy + r)..lineTo(o.dx - r, o.dy)..close();
    Paint line(Color col, double w) => Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = w;
    switch (s) {
      case KeyS.off:
        if (!enabled) {
          _dashed(c, dia, line(N.g38, 1));
        } else {
          c.drawPath(dia, line(will ? Role.of(C.mode, Role.still) : (hover ? Role.of(N.g76, Role.still) : Role.of(N.g44, Role.still)), will ? 1.5 : 1));
        }
      case KeyS.anim:
        final col = !enabled ? Role.disabled : (hover ? Role.of(N.g95, Role.key) : Role.of(N.g91, Role.key));
        c.drawPath(dia, line(col, 1));
        c.drawLine(Offset(o.dx - r - 5, o.dy), Offset(o.dx - r, o.dy), line(col, 1));
        c.drawLine(Offset(o.dx + r, o.dy), Offset(o.dx + r + 5, o.dy), line(col, 1));
      case KeyS.at:
        c.drawPath(dia, Paint()..color = enabled ? C.playhead : N.g44);
        c.drawPath(dia, line(enabled ? N.g100 : N.g56, 1.5));
    }
  }

  @override
  bool shouldRepaint(_KeyPaint o) => o.s != s || o.hover != hover || o.will != will || o.enabled != enabled;
}

// ---- small shared bits ----

/// A small word-button in the lab's choice-control style (g20 pill, g95 text, accent underline when on).
Widget _gap(double h) => SizedBox(height: h);

/// Count word, never colour alone: "変更 7".
Widget _count(String s, {Color tone = N.g63}) => Text(s, style: T.label(tone));

/// The header mark: two stacked diamonds (layers). Not a box, so it never reads as a checkbox.
class _LayersPaint extends CustomPainter {
  const _LayersPaint();
  @override
  void paint(Canvas c, Size s) {
    final line = Paint()..color = N.g63..style = PaintingStyle.stroke..strokeWidth = 1.2..strokeJoin = StrokeJoin.round;
    Path d(double cy) => Path()..moveTo(8, cy - 4)..lineTo(15, cy)..lineTo(8, cy + 4)..lineTo(1, cy)..close();
    c.drawPath(d(11), line);
    c.drawPath(d(7), line..color = N.g76);
  }

  @override
  bool shouldRepaint(_LayersPaint o) => false;
}
