// Light-table forms: film looked at on a lit opal table. Black film with sprocket holes at the edges (the Render leader,
// each card's left edge), graphite words, a colour sticker dot for a card's kind, and a grease-pencil orange loop around
// what is picked or under the hand. What a hand does is naked_ui's.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:naked_ui/naked_ui.dart';

import '../theme.dart';
import '../tokens.dart';
import 'forms.dart';

const _film = Color(0xFF1B1A18), _opal = Color(0xFFF6F7F4), _grease = Color(0xFFF06A1E), _graphite = Color(0xFF26292B), _loupe = Color(0xFFA9AEAB);

TextStyle _word(Color c, double size, {FontWeight w = FontWeight.w600}) => T.name(c).copyWith(fontSize: T.px(size), fontWeight: w, height: 1.1);

TextStyle _mono(Color c, double size) =>
    T.name(c).copyWith(fontFamily: ColorTheme.word('FontMono', 'Menlo'), fontSize: T.px(size), height: 1.1, fontFeatures: const [FontFeature.tabularFigures()]);

/// A grease-pencil loop around its box: never closed clean, it overshoots its start, wobbles and leans a little; the same
/// hand for the same seed.
class _Loop extends CustomPainter {
  const _Loop(this.seed, {this.ink = _grease});
  final int seed;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final rnd = math.Random(seed), a0 = rnd.nextDouble() * math.pi * 2, lean = (rnd.nextDouble() - .5) * .08;
    final rx = (s.width / 2 - 2) / 1.08, ry = (s.height / 2 - 2) / 1.08, k = 2 + rnd.nextInt(2), ph = rnd.nextDouble() * 6;
    final p = Path();
    for (var i = 0; i <= 72; i++) {
      final t = a0 + i / 64 * math.pi * 2; // (72 of 64 steps: the pencil runs past where it started)
      final w = 1 + .045 * math.sin(k * t + ph) - .05 * i / 72;
      final x = rx * w * math.cos(t), y = ry * w * math.sin(t);
      final pt = Offset(s.width / 2 + x * math.cos(lean) - y * math.sin(lean), s.height / 2 + x * math.sin(lean) + y * math.cos(lean));
      i == 0 ? p.moveTo(pt.dx, pt.dy) : p.lineTo(pt.dx, pt.dy);
    }
    c.drawPath(
      p,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.2
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round,
    );
  }

  @override
  bool shouldRepaint(_Loop o) => o.seed != seed || o.ink != ink;
}

/// Black film with a row of sprocket holes along each long edge, the table's light showing through them. Across: the
/// holes run left to right on top and bottom; down: one row, in a strip [strip] wide at the left.
class _Sprockets extends CustomPainter {
  const _Sprockets({this.down = false, this.strip = 0});
  final bool down;
  final double strip;
  @override
  void paint(Canvas c, Size s) {
    final film = Paint()..color = _film, light = Paint()..color = _opal;
    if (down) {
      c.drawRect(Rect.fromLTWH(0, 0, strip, s.height), film);
      final hw = strip * .42, hh = strip * .32, pitch = strip * .78;
      for (var y = strip * .3; y + hh < s.height; y += pitch) {
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH((strip - hw) / 2, y, hw, hh), Radius.circular(hh * .25)), light);
      }
      return;
    }
    c.drawRect(Offset.zero & s, film);
    final hh = s.height * .09, hw = s.height * .13, pitch = s.height * .26, m = s.height * .06;
    for (var x = pitch * .3; x + hw < s.width; x += pitch) {
      for (final y in [m, s.height - m - hh]) {
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x, y, hw, hh), Radius.circular(hh * .3)), light);
      }
    }
  }

  @override
  bool shouldRepaint(_Sprockets o) => o.down != down || o.strip != strip;
}

/// Edge-print arrow: ▸ shut, ▾ open.
class _Arrow extends CustomPainter {
  const _Arrow(this.open, this.ink);
  final bool open;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final r = Rect.fromCenter(center: s.center(Offset.zero), width: s.shortestSide * .5, height: s.shortestSide * .5);
    c.drawPath(
      Path()..addPolygon(open ? [r.topLeft, r.topRight, r.bottomCenter] : [r.topLeft, r.centerRight, r.bottomLeft], true),
      Paint()..color = ink,
    );
  }

  @override
  bool shouldRepaint(_Arrow o) => o.open != open || o.ink != ink;
}

Widget _looped(bool on, int seed, Widget child, {bool hand = false}) => CustomPaint(
  foregroundPainter: on ? _Loop(seed) : (hand ? _Loop(seed, ink: _grease.withValues(alpha: .45)) : null),
  child: child,
);

class LightTableForms extends Forms {
  const LightTableForms();

  /// A black film leader with sprocket rows, the word lit through it; under the hand the pencil rings the word.
  @override
  Widget action(BuildContext context, ActionSpec a) => NakedButton(
    onPressed: a.onPressed,
    builder: (context, st, _) => Transform.translate(
      offset: Offset(0, st.isPressed ? 1 : 0),
      child: CustomPaint(
        painter: const _Sprockets(),
        child: Container(
          height: Shape.z(36),
          padding: EdgeInsets.symmetric(horizontal: Shape.s(16)),
          alignment: Alignment.center,
          child: _looped(
            st.isHovered || st.isPressed,
            7,
            Padding(
              padding: EdgeInsets.symmetric(horizontal: Shape.s(8), vertical: Shape.s(2)),
              child: Text(a.label, maxLines: 1, style: _word(_opal, 15, w: FontWeight.w700)),
            ),
          ),
        ),
      ),
    ),
  );

  /// Slide mounts side by side, each a window with its glyph and word; the picked one ringed in grease pencil.
  @override
  Widget choice(BuildContext context, ChoiceSpec c) => NakedToggleGroup<int>(
    selectedValue: c.selected,
    enabled: c.enabled,
    onChanged: (v) => v == null ? null : c.onSelect(v),
    child: SizedBox(
      height: Shape.z(34),
      child: Row(
        children: [
          for (var i = 0; i < c.labels.length; i++) ...[
            if (i > 0) SizedBox(width: Shape.s(4)),
            Expanded(
              child: NakedToggleOption<int>(
                value: i,
                semanticLabel: c.labels[i],
                builder: (context, st, _) => _looped(
                  st.isSelected,
                  31 + i,
                  Padding(
                    padding: EdgeInsets.symmetric(horizontal: Shape.s(9), vertical: Shape.s(6)),
                    child: Container(
                      decoration: BoxDecoration(
                        color: st.isHovered ? _opal : null,
                        border: Border.all(color: st.isSelected ? _graphite : _loupe),
                        borderRadius: BorderRadius.all(Shape.r(2)),
                      ),
                      padding: EdgeInsets.symmetric(horizontal: Shape.s(4)),
                      child: Row(
                        children: [
                          if (i < c.glyphs.length)
                            SizedBox(
                              width: Shape.z(16),
                              height: Shape.z(14),
                              child: CustomPaint(painter: _Glyph(c.glyphs[i], st.isSelected ? _graphite : _loupe)),
                            ),
                          SizedBox(width: Shape.s(3)),
                          Flexible(
                            child: FittedBox(
                              fit: BoxFit.scaleDown,
                              alignment: Alignment.centerLeft,
                              child: Text(c.labels[i], maxLines: 1, style: _mono(st.isSelected ? _graphite : Grey.g56, 12)),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ],
        ],
      ),
    ),
  );

  /// Words on the table, the place you are ringed in grease pencil.
  @override
  Widget tabs(BuildContext context, ChoiceSpec c) => NakedToggleGroup<int>(
    selectedValue: c.selected,
    enabled: c.enabled,
    onChanged: (v) => v == null ? null : c.onSelect(v),
    child: SizedBox(
      height: Shape.z(30),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var i = 0; i < c.labels.length; i++)
            NakedToggleOption<int>(
              value: i,
              semanticLabel: c.labels[i],
              builder: (context, st, _) => _looped(
                st.isSelected,
                11 + i,
                hand: st.isHovered,
                Container(
                  constraints: BoxConstraints(minWidth: Shape.z(72)),
                  padding: EdgeInsets.symmetric(horizontal: Shape.s(12)),
                  alignment: Alignment.center,
                  child: Text(
                    c.labels[i],
                    maxLines: 1,
                    style: _word(st.isSelected ? _graphite : Grey.g56, 16, w: st.isSelected ? FontWeight.w700 : FontWeight.w500),
                  ),
                ),
              ),
            ),
        ],
      ),
    ),
  );

  /// The word, ringed when on.
  @override
  Widget onOff(BuildContext context, SwitchSpec s) => NakedToggle(
    value: s.on,
    asSwitch: true,
    enabled: s.enabled,
    onChanged: s.onChanged,
    builder: (context, st, _) => Opacity(
      opacity: s.enabled ? 1 : .4,
      child: _looped(
        s.on,
        5,
        hand: st.isHovered,
        Container(
          width: Shape.z(40),
          height: Shape.z(22),
          alignment: Alignment.center,
          child: Text(s.on ? 'On' : 'Off', maxLines: 1, style: _mono(s.on ? _graphite : Grey.g56, 12)),
        ),
      ),
    ),
  );

  /// A strip of film down the card's left edge; the head a colour sticker dot for its kind, the title, an edge-print arrow.
  @override
  Widget card(BuildContext context, CardSpec c) {
    final strip = Shape.z(12);
    final head = c.title == null
        ? c.head
        : NakedButton(
            onPressed: c.onToggle,
            builder: (context, st, _) => SizedBox(
              height: Shape.z(30),
              child: Row(
                children: [
                  Container(
                    width: Shape.z(9),
                    height: Shape.z(9),
                    decoration: BoxDecoration(color: c.tone ?? _loupe, shape: BoxShape.circle),
                  ),
                  SizedBox(width: Shape.s(7)),
                  Expanded(
                    child: FittedBox(
                      fit: BoxFit.scaleDown,
                      alignment: Alignment.centerLeft,
                      child: Text(c.title!, maxLines: 1, style: _word(_graphite, 17, w: FontWeight.w600)),
                    ),
                  ),
                  ?c.trailing,
                  SizedBox.square(
                    dimension: Shape.z(22),
                    child: CustomPaint(painter: _Arrow(c.open, st.isHovered ? _grease : _loupe)),
                  ),
                ],
              ),
            ),
          );
    return CustomPaint(
      painter: _Sprockets(down: true, strip: strip),
      child: Container(
        color: _opal.withValues(alpha: .6),
        margin: EdgeInsets.only(left: strip),
        padding: EdgeInsets.fromLTRB(Shape.s(10), Shape.s(2), Shape.s(6), Shape.s(6)),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [?head, if (c.open) ...c.body],
        ),
      ),
    );
  }
}

class _Glyph extends CustomPainter {
  const _Glyph(this.draw, this.ink);
  final void Function(Canvas c, Paint stroke) draw;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    c.save();
    c.scale(s.width / 16, s.height / 14);
    draw(
      c,
      Paint()
        ..color = ink
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.2,
    );
    c.restore();
  }

  @override
  bool shouldRepaint(_Glyph o) => o.ink != ink || o.draw != draw;
}
