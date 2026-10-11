// The dose study, part 1: the shared selection frame, the choice strip (keyboard contract), widgets 1-5.
// Every seasoning number comes from `Dose` in tokens.dart. research/pop-magazine.md sections 4, 5, 7.
import 'dart:math' as math;

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../parts/glyphs.dart';
import '../../parts/relations.dart';
import '../../tokens.dart';

/// How many selection animations are running right now (the too-far meter reads it).
abstract final class DoseMotion {
  static final running = ValueNotifier<int>(0);
  static void add(int n) => Future.microtask(() => running.value = math.max(0, running.value + n));
}

/// A chamfered (45 degree) rectangle: top-right and bottom-left corners cut. cut 0 gives a plain rounded rectangle.
Path cutPath(Rect r, double cut, {double radius = 0}) {
  if (cut <= 0) return Path()..addRRect(RRect.fromRectAndRadius(r, Radius.circular(radius)));
  return Path()
    ..moveTo(r.left, r.top)
    ..lineTo(r.right - cut, r.top)
    ..lineTo(r.right, r.top + cut)
    ..lineTo(r.right, r.bottom)
    ..lineTo(r.left + cut, r.bottom)
    ..lineTo(r.left, r.bottom - cut)
    ..close();
}

/// The one selection frame of the study. dose 0: a g20 pill with a 1px accent underline (controls) or outline (tiles); dose 1+: a 1px hue hairline at the dose's alpha,
/// chamfer from dose 2, inner highlight at dose 3. The frame animates over the dose's duration (0 = instant). Hover only tints the frame; it never changes the box.
class SelBox extends StatefulWidget {
  const SelBox({super.key, required this.d, required this.selected, required this.hue, required this.w, required this.h, required this.child, this.hovered = false, this.instant = false, this.tile = false});
  final Dose d;
  final bool selected, hovered, instant, tile;
  final Color hue;
  final double w, h;
  final Widget child;
  @override
  State<SelBox> createState() => _SelBoxState();
}

class _SelBoxState extends State<SelBox> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: widget.d.selDur, value: widget.selected ? 1 : 0);
  bool _counted = false;

  @override
  void initState() {
    super.initState();
    _c.addStatusListener((s) {
      final run = s == AnimationStatus.forward || s == AnimationStatus.reverse;
      if (run != _counted) {
        _counted = run;
        DoseMotion.add(run ? 1 : -1);
      }
    });
  }

  @override
  void didUpdateWidget(SelBox old) {
    super.didUpdateWidget(old);
    _c.duration = widget.d.selDur;
    if (old.selected != widget.selected) {
      if (widget.d.selMs == 0 || widget.instant) {
        _c.value = widget.selected ? 1 : 0;
      } else {
        widget.selected ? _c.forward() : _c.reverse();
      }
    }
  }

  @override
  void dispose() {
    if (_counted) DoseMotion.add(-1);
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => SizedBox(
        width: widget.w,
        height: widget.h,
        child: AnimatedBuilder(
          animation: _c,
          builder: (context, child) => CustomPaint(
            painter: _SelPainter(Curves.easeOut.transform(_c.value), widget.hue, widget.d, widget.tile, widget.hovered),
            child: child,
          ),
          child: widget.child,
        ),
      );
}

class _SelPainter extends CustomPainter {
  const _SelPainter(this.t, this.hue, this.d, this.tile, this.hovered);
  final double t;
  final Color hue;
  final Dose d;
  final bool tile, hovered;

  @override
  void paint(Canvas c, Size s) {
    final r = (Offset.zero & s).deflate(Dose.hair / 2);
    final path = cutPath(r, d.cut * t, radius: Dose.radius);
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    if (t > 0 && !tile) c.drawPath(path, Paint()..color = N.g20.withValues(alpha: t));
    if (t < 1) c.drawPath(path, line..color = N.g100.withValues(alpha: (hovered ? Dose.crossAlpha : Dose.hairIdle) * (1 - t)));
    if (t <= 0) return;
    if (!d.selHue) {
      if (tile) {
        c.drawPath(path, line..color = C.mode.withValues(alpha: t));
      } else {
        c.drawLine(Offset(Dose.radius, s.height - Dose.hair / 2), Offset(s.width - Dose.radius, s.height - Dose.hair / 2), line..color = C.mode.withValues(alpha: t));
      }
      return;
    }
    c.drawPath(path, line..color = hue.withValues(alpha: d.selAlpha * t));
    if (d.innerAlpha > 0) c.drawPath(cutPath(r.deflate(Dose.hair), math.max(0, d.cut * t - Dose.hair), radius: Dose.radius), line..color = N.g100.withValues(alpha: d.innerAlpha * t));
  }

  @override
  bool shouldRepaint(_SelPainter o) => o.t != t || o.hue != hue || o.d != d || o.hovered != hovered || o.tile != tile;
}

/// What an item of a choice strip is told.
class ItemState {
  const ItemState({required this.selected, required this.hovered, required this.committed, required this.instant});
  final bool selected, hovered, committed, instant;
}

/// A set of placed choices with the keyboard contract: arrows move the preview, digit keys jump, Enter commits, Esc reverts to the committed one.
/// Click commits. Hover previews (frame tint, caption) and moves nothing. Under key repeat the animation is skipped.
class ChoiceStrip extends StatefulWidget {
  const ChoiceStrip({super.key, required this.names, required this.itemW, required this.itemH, required this.itemBuilder, required this.hint, this.columns, this.jump = const {}, this.kicker, this.kickerH = 0, this.onCommit, this.initial = 0});
  final List<String> names;
  final double itemW, itemH, kickerH;
  final int? columns;
  final Map<LogicalKeyboardKey, int> jump;
  final Widget Function(BuildContext, int, ItemState) itemBuilder;

  /// A label over the selected item, in a slot that is always reserved (so selecting never reflows).
  final Widget Function(int cursor)? kicker;
  final String hint;
  final ValueChanged<int>? onCommit;
  final int initial;
  @override
  State<ChoiceStrip> createState() => _ChoiceStripState();
}

class _ChoiceStripState extends State<ChoiceStrip> {
  final _focus = FocusNode();
  late int _cursor = widget.initial, _committed = widget.initial;
  int? _hover;
  bool _repeat = false;

  int get _cols => widget.columns ?? widget.names.length;

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  void _move(int to) => setState(() => _cursor = to.clamp(0, widget.names.length - 1));

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is KeyUpEvent) return KeyEventResult.ignored;
    final k = e.logicalKey;
    _repeat = e is KeyRepeatEvent;
    if (k == LogicalKeyboardKey.arrowRight) {
      _move(_cursor + 1);
    } else if (k == LogicalKeyboardKey.arrowLeft) {
      _move(_cursor - 1);
    } else if (k == LogicalKeyboardKey.arrowDown && _cols < widget.names.length) {
      _move(_cursor + _cols);
    } else if (k == LogicalKeyboardKey.arrowUp && _cols < widget.names.length) {
      _move(_cursor - _cols);
    } else if (widget.jump.containsKey(k)) {
      _move(widget.jump[k]!);
    } else if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
      setState(() => _committed = _cursor);
      widget.onCommit?.call(_cursor);
    } else if (k == LogicalKeyboardKey.escape) {
      setState(() => _cursor = _committed);
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final n = widget.names.length, rows = (n / _cols).ceil();
    final total = _cols * widget.itemW + (_cols - 1) * Dose.gap;
    final x = (_cursor % _cols) * (widget.itemW + Dose.gap);
    final focused = _focus.hasFocus;
    Widget item(int i) => MouseRegion(
          onEnter: (_) => setState(() => _hover = i),
          onExit: (_) => setState(() => _hover = null),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () {
              _focus.requestFocus();
              _repeat = false;
              setState(() => _cursor = _committed = i);
              widget.onCommit?.call(i);
            },
            child: widget.itemBuilder(context, i, ItemState(selected: i == _cursor, hovered: i == _hover, committed: i == _committed, instant: _repeat)),
          ),
        );
    final shown = _hover ?? _cursor;
    return Focus(
      focusNode: _focus,
      onKeyEvent: _key,
      onFocusChange: (_) => setState(() {}),
      child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        if (widget.kicker != null && widget.kickerH > 0)
          SizedBox(
            width: total,
            height: widget.kickerH,
            child: Stack(clipBehavior: Clip.none, children: [
              if (_cursor % _cols > _cols / 2) Positioned(right: total - x - widget.itemW, bottom: 0, child: widget.kicker!(_cursor)) else Positioned(left: x, bottom: 0, child: widget.kicker!(_cursor)),
            ]),
          ),
        for (var r = 0; r < rows; r++) ...[
          if (r > 0) const SizedBox(height: Dose.gap),
          Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
            for (var i = r * _cols; i < math.min(n, (r + 1) * _cols); i++) ...[if (i % _cols > 0) const SizedBox(width: Dose.gap), item(i)],
          ]),
        ],
        const SizedBox(height: Dose.gapL),
        Text('${_hover != null ? 'preview' : 'selected'} ${widget.names[shown]}${_cursor != _committed ? '  (committed ${widget.names[_committed]})' : ''}', style: T.label(N.g76)),
        const SizedBox(height: Dose.gap),
        Text(widget.hint, style: T.label(focused ? N.g76 : N.g56)),
      ]),
    );
  }
}

/// Caps label, +0.2em tracking: informational text keeps the lab's 10 px floor.
Text kickerText(String s, Color c) => Text(s.toUpperCase(), maxLines: 1, softWrap: false, style: T.micro(c).copyWith(fontSize: Dose.kickerPx, letterSpacing: Dose.kickerPx * Dose.kickerTrack / 10));

// ---------------------------------------------------------------- 1 relation-family chips

class RelationChips extends StatelessWidget {
  const RelationChips({super.key, required this.d});
  final Dose d;

  static final _keys = {
    LogicalKeyboardKey.digit1: 0, LogicalKeyboardKey.digit2: 1, LogicalKeyboardKey.digit3: 2, LogicalKeyboardKey.digit4: 3, LogicalKeyboardKey.digit5: 4, LogicalKeyboardKey.digit6: 5,
  };

  @override
  Widget build(BuildContext context) => ChoiceStrip(
        key: ValueKey('chips${d.level}'),
        names: [for (final f in Fam.all) f.name],
        itemW: Dose.chipW,
        itemH: Dose.chipH,
        jump: _keys,
        hint: '← →  1-6  Enter  Esc',
        kickerH: d.kickerHeader ? Dose.kickerSlot : 0,
        kicker: (i) => kickerText('${d.kickerItems ? '${(i + 1).toString().padLeft(2, '0')} ' : ''}${Fam.all[i].name}', Fam.all[i].c),
        itemBuilder: (ctx, i, s) => SelBox(
          d: d, selected: s.selected, hovered: s.hovered, instant: s.instant, hue: Fam.all[i].c, w: Dose.chipW, h: Dose.chipH,
          child: Center(child: FamIcon(Fam.all[i], size: Dose.pictoPx)),
        ),
      );
}

// ---------------------------------------------------------------- 2 mode plates

class _PlatePainter extends CustomPainter {
  const _PlatePainter(this.mode, this.c);
  final int mode;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final p = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = Dose.hair..strokeJoin = StrokeJoin.round;
    final u = s.height * .62, o = Offset((s.width - u) / 2, (s.height - u) / 2);
    Offset q(double x, double y) => o + Offset(x * u, y * u);
    switch (mode) {
      case 0:
        cv.drawRect(Rect.fromPoints(q(0, 0), q(1, 1)), p);
      case 1:
        cv.drawPath(Path()..moveTo(q(.22, .08).dx, q(.22, .08).dy)..lineTo(q(1.1, .08).dx, q(1.1, .08).dy)..lineTo(q(.78, .94).dx, q(.78, .94).dy)..lineTo(q(-.1, .94).dx, q(-.1, .94).dy)..close(), p);
      default:
        final f = Rect.fromPoints(q(0, .22), q(.78, 1)), b = f.shift(Offset(u * .22, -u * .22));
        cv.drawRect(f, p);
        cv.drawRect(b, p);
        for (final pair in [(f.topLeft, b.topLeft), (f.topRight, b.topRight), (f.bottomRight, b.bottomRight), (f.bottomLeft, b.bottomLeft)]) {
          cv.drawLine(pair.$1, pair.$2, p);
        }
    }
  }

  @override
  bool shouldRepaint(_PlatePainter o) => o.mode != mode || o.c != c;
}

class ModePlates extends StatelessWidget {
  const ModePlates({super.key, required this.d});
  final Dose d;
  @override
  Widget build(BuildContext context) => ChoiceStrip(
        key: ValueKey('plates${d.level}'),
        names: const ['2D', '2.5D', '3D'],
        itemW: Dose.plateW,
        itemH: Dose.plateH,
        initial: 1,
        jump: {LogicalKeyboardKey.digit2: 0, LogicalKeyboardKey.digit5: 1, LogicalKeyboardKey.digit3: 2},
        hint: '← →  2 5 3  Enter  Esc',
        itemBuilder: (ctx, i, s) => SelBox(
          d: d, selected: s.selected, hovered: s.hovered, instant: s.instant, hue: C.mode, w: Dose.plateW, h: Dose.plateH,
          child: CustomPaint(painter: _PlatePainter(i, s.selected ? (d.selHue ? C.mode : N.g95) : N.g63)),
        ),
      );
}

// ---------------------------------------------------------------- 3 easing thumbnails

class EaseDef {
  const EaseDef(this.name, this.curve);
  final String name;
  final Curve? curve; // null = hold (a step)
  double at(double t) => curve == null ? (t < .5 ? 0 : 1) : curve!.transform(t);
}

const easings = [
  EaseDef('Linear', Curves.linear),
  EaseDef('Ease in', Cubic(.42, 0, 1, 1)),
  EaseDef('Ease out', Cubic(0, 0, .58, 1)),
  EaseDef('Ease in-out', Cubic(.42, 0, .58, 1)),
  EaseDef('Back in', Cubic(.36, 0, .66, -.56)),
  EaseDef('Back out', Cubic(.34, 1.56, .64, 1)),
  EaseDef('Quint in', Cubic(.64, 0, .78, 0)),
  EaseDef('Quint out', Cubic(.22, 1, .36, 1)),
  EaseDef('Circ in-out', Cubic(.85, 0, .15, 1)),
  EaseDef('Hold', null),
];

class _EasePainter extends CustomPainter {
  const _EasePainter(this.e, this.color, this.px, this.dot);
  final EaseDef e;
  final Color color;
  final double px;
  final double? dot; // 0..1 along the curve, or null
  @override
  void paint(Canvas c, Size s) {
    final box = (Offset.zero & s).deflate(Dose.gap);
    Offset at(double t) => Offset(box.left + t * box.width, box.bottom - ((e.at(t) + .2) / 1.4).clamp(0.0, 1.0) * box.height);
    final path = Path()..moveTo(at(0).dx, at(0).dy);
    if (e.curve == null) {
      path..lineTo(at(.5).dx, at(0).dy)..lineTo(at(.5).dx, at(1).dy)..lineTo(at(1).dx, at(1).dy);
    } else {
      for (var i = 1; i <= 24; i++) {
        final p = at(i / 24);
        path.lineTo(p.dx, p.dy);
      }
    }
    c.drawPath(path, Paint()..color = color..style = PaintingStyle.stroke..strokeWidth = px..strokeJoin = StrokeJoin.round);
    if (dot != null) c.drawCircle(at(dot!), Dose.dotPx, Paint()..color = N.g95);
  }

  @override
  bool shouldRepaint(_EasePainter o) => o.e != e || o.color != color || o.px != px || o.dot != dot;
}

/// One thumbnail: the curve at 1px (1.5px when selected); on hover (dose 1+) a dot runs along the curve inside the thumbnail only, and stops with reduced motion.
class EaseThumb extends StatefulWidget {
  const EaseThumb({super.key, required this.d, required this.def, required this.s});
  final Dose d;
  final EaseDef def;
  final ItemState s;
  @override
  State<EaseThumb> createState() => _EaseThumbState();
}

class _EaseThumbState extends State<EaseThumb> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: Dose.dotMs));
  bool _counted = false;

  bool get _want => widget.s.hovered && widget.d.hoverDot && !(MediaQuery.maybeOf(context)?.disableAnimations ?? false);

  void _sync() {
    if (_want && !_c.isAnimating) {
      _c.repeat();
      if (!_counted) {
        _counted = true;
        DoseMotion.add(1);
      }
    } else if (!_want && _c.isAnimating) {
      _c.stop();
      _c.value = 0;
      if (_counted) {
        _counted = false;
        DoseMotion.add(-1);
      }
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _sync();
  }

  @override
  void didUpdateWidget(EaseThumb o) {
    super.didUpdateWidget(o);
    _sync();
  }

  @override
  void dispose() {
    if (_counted) DoseMotion.add(-1);
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final sel = widget.s.selected;
    return SelBox(
      d: widget.d, selected: sel, hovered: widget.s.hovered, instant: widget.s.instant, hue: C.mode, w: Dose.thumbW, h: Dose.thumbH, tile: true,
      child: AnimatedBuilder(
        animation: _c,
        builder: (context, _) => CustomPaint(
          painter: _EasePainter(widget.def, sel ? N.g95 : N.g63, sel && widget.d.level >= 1 ? Dose.curveSelPx : Dose.curvePx, _c.isAnimating ? _c.value : null),
        ),
      ),
    );
  }
}

class EasingThumbs extends StatelessWidget {
  const EasingThumbs({super.key, required this.d});
  final Dose d;
  @override
  Widget build(BuildContext context) => ChoiceStrip(
        key: ValueKey('ease${d.level}'),
        names: [for (final e in easings) e.name],
        itemW: Dose.thumbW,
        itemH: Dose.thumbH,
        columns: 5,
        hint: '← → ↑ ↓  Enter  Esc',
        itemBuilder: (ctx, i, s) => EaseThumb(d: d, def: easings[i], s: s),
      );
}

// ---------------------------------------------------------------- 4 browser tile

/// A tiny state tag. Below dose 2 it is a plain grey word; from dose 2 a 1px framed, chamfered, skewed tag; at dose 3 a selected tile's badge gets a hue-shifted 1px offset shadow.
class DoseBadge extends StatelessWidget {
  const DoseBadge(this.label, {super.key, required this.d, required this.hue, this.shadow = false});
  final String label;
  final Dose d;
  final Color hue;
  final bool shadow;
  @override
  Widget build(BuildContext context) {
    final text = Text(label.toUpperCase(), maxLines: 1, softWrap: false, style: T.micro(d.badgeLoud ? N.g91 : N.g63).copyWith(letterSpacing: Dose.kickerPx * .1));
    if (!d.badgeLoud) return SizedBox(height: Dose.badgeH, child: Padding(padding: const EdgeInsets.symmetric(horizontal: Dose.badgePadX), child: Center(child: text)));
    return Transform(
      alignment: Alignment.center,
      transform: Matrix4.skewX(d.skewDeg * math.pi / 180),
      child: CustomPaint(
        painter: _BadgePainter(d, hue, shadow && d.offsetShadow),
        child: SizedBox(height: Dose.badgeH, child: Padding(padding: const EdgeInsets.symmetric(horizontal: Dose.badgePadX), child: Center(child: text))),
      ),
    );
  }
}

class _BadgePainter extends CustomPainter {
  const _BadgePainter(this.d, this.hue, this.shadow);
  final Dose d;
  final Color hue;
  final bool shadow;
  @override
  void paint(Canvas c, Size s) {
    final r = (Offset.zero & s).deflate(Dose.hair / 2), p = cutPath(r, d.cut), line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    if (shadow) c.drawPath(p.shift(const Offset(Dose.hair, Dose.hair)), line..color = Dose.shifted(hue, d.shadowHueShift));
    c.drawPath(p, Paint()..color = N.g13);
    c.drawPath(p, line..color = hue.withValues(alpha: d.selAlpha));
  }

  @override
  bool shouldRepaint(_BadgePainter o) => o.d != d || o.hue != hue || o.shadow != shadow;
}

class TileDef {
  const TileDef(this.name, this.category);
  final String name, category;
}

const tileDefs = [TileDef('Gaussian Blur', 'Blur'), TileDef('Glow', 'Light'), TileDef('Ripple', 'Warp')];
const tileBadgeLabels = ['New', 'GPU', '3D'];
List<Color> get tileBadgeHues => [C.mode, Fam.along.c, Fam.follow.c];

class BrowserTile extends StatelessWidget {
  const BrowserTile({super.key, required this.d, required this.def, required this.selected, required this.badges, this.hovered = false, this.onTap});
  final Dose d;
  final TileDef def;
  final bool selected, hovered;
  final int badges;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) {
    final n = math.min(badges, d.badgesMax);
    return GestureDetector(
      onTap: onTap,
      child: SelBox(
        d: d, selected: selected, hovered: hovered, hue: C.mode, w: Dose.tileW, h: Dose.tileH, tile: true,
        child: Padding(
          padding: const EdgeInsets.all(Dose.tilePad),
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
            SizedBox(height: Dose.kickerSlot, child: d.kickerItems ? kickerText(def.category, N.g63) : null),
            Container(
              height: Dose.tileThumbH,
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(Dose.radius)),
              child: const Center(child: Glyph(G.effect, size: 22, color: N.g63)),
            ),
            const SizedBox(height: Dose.gap),
            SizedBox(
              height: Dose.badgeH,
              width: Dose.tileW - 2 * Dose.tilePad,
              child: FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerLeft, child: Flex(direction: Axis.horizontal, children: [
                for (var i = 0; i < n; i++) ...[if (i > 0) const SizedBox(width: Dose.gap), DoseBadge(tileBadgeLabels[i], d: d, hue: tileBadgeHues[i], shadow: selected)],
              ])),
            ),
            const SizedBox(height: Dose.gap),
            SizedBox(
              height: Dose.tileNameCell,
              child: Align(alignment: Alignment.centerLeft, child: Text(def.name, maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: T.name(selected ? N.g95 : N.g76).copyWith(fontSize: d.selLabelPx(selected)))),
            ),
          ]),
        ),
      ),
    );
  }
}

/// Three tiles, one selected, with hover tracked per tile.
class TileRow extends StatefulWidget {
  const TileRow({super.key, required this.d, required this.badges});
  final Dose d;
  final int badges;
  @override
  State<TileRow> createState() => _TileRowState();
}

class _TileRowState extends State<TileRow> {
  int _sel = 1;
  int? _hover;
  @override
  Widget build(BuildContext context) => Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < tileDefs.length; i++) ...[
          if (i > 0) const SizedBox(width: Dose.gapL),
          MouseRegion(
            onEnter: (_) => setState(() => _hover = i),
            onExit: (_) => setState(() => _hover = null),
            child: BrowserTile(d: widget.d, def: tileDefs[i], selected: i == _sel, hovered: i == _hover, badges: widget.badges, onTap: () => setState(() => _sel = i)),
          ),
        ],
      ]);
}

// ---------------------------------------------------------------- 5 panel header strip

class _HeaderPainter extends CustomPainter {
  const _HeaderPainter(this.d);
  final Dose d;
  @override
  void paint(Canvas c, Size s) {
    final cut = d.headerCut ? d.cut : 0.0;
    final body = Path()..moveTo(0, 0)..lineTo(s.width - cut, 0)..lineTo(s.width, cut)..lineTo(s.width, s.height)..lineTo(0, s.height)..close();
    c.drawPath(body, Paint()..color = N.g13);
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    if (cut > 0) c.drawLine(Offset(s.width - cut, Dose.hair / 2), Offset(s.width, cut + Dose.hair / 2), line..color = N.g100.withValues(alpha: Dose.leadAlpha));
    final y = s.height - Dose.hair / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), line..color = d.leadLine ? N.g100.withValues(alpha: Dose.leadAlpha) : N.rowLine);
  }

  @override
  bool shouldRepaint(_HeaderPainter o) => o.d != d;
}

/// Halftone on a 45 degree lattice: pitch and alpha from the dose; a strip of its own, never under text.
class _HalftonePainter extends CustomPainter {
  const _HalftonePainter(this.pitch, this.alpha);
  final double pitch, alpha;
  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    final p = Paint()..color = N.g100.withValues(alpha: alpha);
    var row = 0;
    for (var y = pitch / 2; y < s.height + pitch; y += pitch / 2, row++) {
      for (var x = row.isOdd ? pitch / 2 : 0.0; x < s.width + pitch; x += pitch) {
        c.drawCircle(Offset(x, y), pitch * .22, p);
      }
    }
  }

  @override
  bool shouldRepaint(_HalftonePainter o) => o.pitch != pitch || o.alpha != alpha;
}

class PanelHeader extends StatelessWidget {
  const PanelHeader({super.key, required this.d, this.kicker = 'Inspector', this.title = 'Relations', this.number = 1});
  final Dose d;
  final String kicker, title;
  final int number;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: Dose.headerW,
        child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
          CustomPaint(
            painter: _HeaderPainter(d),
            child: SizedBox(
              height: Dose.headerH,
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: Dose.gapL + Dose.gap),
                child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  SizedBox(
                    height: Dose.kickerSlot,
                    child: d.kickerHeader ? Flex(direction: Axis.horizontal, children: [
                      if (d.headerNumber) ...[Text(number.toString().padLeft(2, '0'), style: T.value(N.g56).copyWith(fontSize: Dose.kickerPx)), const SizedBox(width: Dose.gapL)],
                      kickerText(kicker, N.g76),
                    ]) : null,
                  ),
                  Text(title, style: T.title()),
                ]),
              ),
            ),
          ),
          if (d.halftone) SizedBox(width: Dose.headerW, height: Dose.stripH, child: CustomPaint(painter: _HalftonePainter(d.halftonePitch, d.halftoneAlpha))) else const SizedBox(height: Dose.stripH),
        ]),
      );
}
