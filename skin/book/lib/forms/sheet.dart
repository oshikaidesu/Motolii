// Sheet forms: one sheet of paper. A control is a word on that sheet; the sheet stays empty around it.
// The one chromatic mark is the place you are, or the control under the hand. A card's kind is a dot, because a filled head would make every card a figure.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:naked_ui/naked_ui.dart';

import '../tokens.dart';
import 'forms.dart';
import 'number_paint.dart';

enum _Mark { dot, ring, quarter }

class _MarkPaint extends CustomPainter {
  const _MarkPaint(this.mark, this.ink);
  final _Mark mark;
  final Color ink;

  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = ink;
    final r = s.shortestSide;
    switch (mark) {
      case _Mark.dot:
        c.drawCircle(s.center(Offset.zero), r * .42, p);
      case _Mark.ring:
        c.drawCircle(
          s.center(Offset.zero),
          r * .34,
          p
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.25,
        );
      case _Mark.quarter:
        c.drawArc(Rect.fromCircle(center: Offset(0, s.height), radius: r), -math.pi / 2, math.pi / 2, true, p);
    }
  }

  @override
  bool shouldRepaint(_MarkPaint o) => o.mark != mark || o.ink != ink;
}

class _GlyphPaint extends CustomPainter {
  const _GlyphPaint(this.draw, this.ink);
  final void Function(Canvas c, Paint stroke) draw;
  final Color ink;

  @override
  void paint(Canvas c, Size s) {
    c.save();
    c.scale(s.width / 16, s.height / 14);
    draw(c, Paint()..color = ink..style = PaintingStyle.stroke..strokeWidth = 1.2..strokeCap = StrokeCap.round);
    c.restore();
  }

  @override
  bool shouldRepaint(_GlyphPaint o) => o.ink != ink || o.draw != draw;
}

TextStyle _word(Color c, {FontWeight w = FontWeight.w500}) => T.name(c).copyWith(fontSize: T.px(13), fontWeight: w, height: 1.15);

class SheetForms extends Forms {
  const SheetForms();

  /// The word on the sheet, and a ring. The ring becomes the mark's quarter only while the hand is on it.
  @override
  Widget action(BuildContext context, ActionSpec a) => NakedButton(
    onPressed: a.onPressed,
    builder: (context, st, _) {
      final hot = st.isHovered || st.isPressed;
      return SizedBox(
        height: Shape.z(28),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(a.label, style: _word(Grey.g95)),
            SizedBox(width: Shape.s(8)),
            SizedBox.square(
              dimension: Shape.z(14),
              child: CustomPaint(painter: _MarkPaint(hot ? _Mark.quarter : _Mark.ring, hot ? Accent.key : Grey.g38)),
            ),
          ],
        ),
      );
    },
  );

  /// Words in a row. The picked one keeps the mark's dot; the others stay on the sheet.
  @override
  Widget tabs(BuildContext context, ChoiceSpec c) => NakedToggleGroup<int>(
    selectedValue: c.selected,
    enabled: c.enabled,
    onChanged: (v) => v == null ? null : c.onSelect(v),
    child: SizedBox(
      height: Shape.z(28),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var i = 0; i < c.labels.length; i++)
            NakedToggleOption<int>(
              value: i,
              semanticLabel: c.labels[i],
              builder: (context, st, _) => Padding(
                padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (st.isSelected) ...[
                      SizedBox.square(dimension: Shape.z(6), child: CustomPaint(painter: _MarkPaint(_Mark.dot, Accent.key))),
                      SizedBox(width: Shape.s(6)),
                    ],
                    Text(c.labels[i], style: _word(st.isSelected ? Grey.g100 : Grey.g56, w: st.isSelected ? FontWeight.w600 : FontWeight.w400)),
                  ],
                ),
              ),
            ),
        ],
      ),
    ),
  );

  /// The picked option's glyph takes the mark while the hand is on it. Nothing behind the words is filled.
  @override
  Widget choice(BuildContext context, ChoiceSpec c) => NakedToggleGroup<int>(
    selectedValue: c.selected,
    enabled: c.enabled,
    onChanged: (v) => v == null ? null : c.onSelect(v),
    child: SizedBox(
      height: Shape.z(28),
      child: Row(
        children: [
          for (var i = 0; i < c.labels.length; i++)
            Expanded(
              child: NakedToggleOption<int>(
                value: i,
                semanticLabel: c.labels[i],
                builder: (context, st, _) {
                  final hot = st.isHovered || st.isPressed;
                  final ink = st.isSelected ? Grey.g100 : Grey.g56;
                  return Row(
                    children: [
                      SizedBox.square(
                        dimension: Shape.z(18),
                        child: CustomPaint(painter: _MarkPaint(st.isSelected ? _Mark.quarter : _Mark.ring, st.isSelected ? Accent.key : Grey.g38)),
                      ),
                      SizedBox(width: Shape.s(4)),
                      if (i < c.glyphs.length)
                        SizedBox(
                          width: Shape.z(16),
                          height: Shape.z(14),
                          child: CustomPaint(painter: _GlyphPaint(c.glyphs[i], hot && st.isSelected ? Accent.key : ink)),
                        ),
                      SizedBox(width: Shape.s(4)),
                      Flexible(
                        child: FittedBox(
                          fit: BoxFit.scaleDown,
                          alignment: Alignment.centerLeft,
                          child: Text(c.labels[i], maxLines: 1, style: _word(ink, w: st.isSelected ? FontWeight.w600 : FontWeight.w400)),
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
        ],
      ),
    ),
  );

  /// A word for the state, and a circle. Colour arrives only under the hand.
  @override
  Widget onOff(BuildContext context, SwitchSpec s) => NakedToggle(
    value: s.on,
    asSwitch: true,
    enabled: s.enabled,
    onChanged: s.onChanged,
    builder: (context, st, _) => Opacity(
      opacity: s.enabled ? 1 : .4,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(s.on ? 'On' : 'Off', style: _word(s.on ? Grey.g95 : Grey.g56, w: FontWeight.w500).copyWith(fontSize: T.px(11))),
          SizedBox(width: Shape.s(6)),
          SizedBox.square(
            dimension: Shape.z(12),
            child: CustomPaint(painter: _MarkPaint(s.on ? _Mark.dot : _Mark.ring, st.isHovered ? Accent.key : Grey.g95)),
          ),
        ],
      ),
    ),
  );

  /// Title, a dot of its kind, a hairline. The rows sit on the same sheet, with no frame of their own.
  @override
  Widget card(BuildContext context, CardSpec c) {
    final head = c.title == null
        ? c.head
        : NakedButton(
            onPressed: c.onToggle,
            builder: (context, st, _) => SizedBox(
              height: Shape.z(28),
              child: Row(
                children: [
                  if (c.tone != null) ...[
                    SizedBox.square(dimension: Shape.z(6), child: CustomPaint(painter: _MarkPaint(_Mark.dot, c.tone!))),
                    SizedBox(width: Shape.s(8)),
                  ],
                  Expanded(
                    child: FittedBox(
                      fit: BoxFit.scaleDown,
                      alignment: Alignment.centerLeft,
                      child: Text(c.title!, maxLines: 1, style: _word(Grey.g95)),
                    ),
                  ),
                  ?c.trailing,
                  SizedBox.square(
                    dimension: Shape.z(12),
                    child: CustomPaint(painter: _MarkPaint(c.open ? _Mark.quarter : _Mark.ring, st.isHovered ? Accent.key : Grey.g38)),
                  ),
                ],
              ),
            ),
          );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        ?head,
        if (c.title != null) ColoredBox(color: Grey.g20, child: SizedBox(height: 1)),
        if (c.open && c.body.isNotEmpty)
          Padding(
            padding: EdgeInsets.only(top: Shape.s(6)),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: c.body),
          ),
      ],
    );
  }

  /// Each word stands next to a shape the size of the control. The picked one is a quarter in the mark's colour.
  @override
  Widget segment(BuildContext context, ChoiceSpec c) => NakedToggleGroup<int>(
    selectedValue: c.selected,
    enabled: c.enabled,
    onChanged: (v) => v == null ? null : c.onSelect(v),
    child: SizedBox(
      height: Shape.z(22),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (var i = 0; i < c.labels.length; i++)
            NakedToggleOption<int>(
              value: i,
              semanticLabel: c.labels[i],
              builder: (context, st, _) => Padding(
                padding: EdgeInsets.only(right: Shape.s(8)),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    SizedBox.square(
                      dimension: Shape.z(18),
                      child: CustomPaint(painter: _MarkPaint(st.isSelected ? _Mark.quarter : _Mark.ring, st.isSelected ? Accent.key : Grey.g38)),
                    ),
                    SizedBox(width: Shape.s(4)),
                    Text(c.labels[i], style: T.label(st.isSelected ? Grey.g100 : Grey.g56).copyWith(fontWeight: st.isSelected ? FontWeight.w600 : FontWeight.w400)),
                  ],
                ),
              ),
            ),
        ],
      ),
    ),
  );

  /// The state is an 18px circle. On, it is filled with the mark. The window's picture sits beside the word.
  @override
  Widget flag(BuildContext context, FlagSpec f) => NakedButton(
    onPressed: f.onTap,
    builder: (context, st, _) {
      final ink = st.isHovered || f.on ? Accent.key : Grey.g38;
      return SizedBox(
        height: Shape.z(22),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            SizedBox.square(dimension: Shape.z(18), child: CustomPaint(painter: _MarkPaint(f.on ? _Mark.dot : _Mark.ring, ink))),
            SizedBox(width: Shape.s(5)),
            if (f.mark != null) ...[f.mark!(f.on ? Grey.g95 : Grey.g56), SizedBox(width: Shape.s(4))],
            Text(f.label, style: T.label(f.on ? Grey.g95 : Grey.g56).copyWith(fontWeight: f.on ? FontWeight.w600 : FontWeight.w400)),
            SizedBox(width: Shape.s(8)),
          ],
        ),
      );
    },
  );

  /// Filter words in a line. The one on has the mark, and its count.
  @override
  Widget chips(BuildContext context, ChipsSpec c) => SizedBox(
    height: Shape.z(28),
    child: SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
      child: Row(
        children: [
          for (final (name, n) in c.chips)
            NakedButton(
              onPressed: () => c.onPick(name),
              builder: (context, st, _) {
                final on = name == c.active;
                return Padding(
                  padding: EdgeInsets.symmetric(horizontal: Shape.s(6)),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (on) ...[
                        SizedBox.square(dimension: Shape.z(5), child: CustomPaint(painter: _MarkPaint(_Mark.dot, Accent.key))),
                        SizedBox(width: Shape.s(4)),
                      ],
                      Text(name, style: T.label(on ? Grey.g100 : Grey.g56).copyWith(fontWeight: on ? FontWeight.w600 : FontWeight.w400)),
                      if (n != null) ...[SizedBox(width: Shape.s(4)), Text('$n', style: T.value(Grey.g56).copyWith(fontSize: T.px(10)))],
                    ],
                  ),
                );
              },
            ),
        ],
      ),
    ),
  );

  /// A name on the sheet. Picked, a quarter stands at its left. The band behind it is the sheet.
  @override
  Widget track(BuildContext context, TrackSpec t) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: t.onSelect,
    child: Padding(
      padding: EdgeInsets.only(left: Shape.s(6), right: Shape.s(6)),
      child: Row(
        children: [
          SizedBox.square(dimension: Shape.z(16), child: CustomPaint(painter: _MarkPaint(t.selected ? _Mark.quarter : _Mark.ring, t.selected ? Accent.key : Grey.g38))),
          SizedBox(width: Shape.s(6)),
          SizedBox(width: 14, child: Text('${t.index + 1}', style: T.value(Grey.g56).copyWith(fontSize: T.px(10)))),
          SizedBox(width: Shape.s(6)),
          t.kind,
          SizedBox(width: Shape.s(7)),
          Expanded(
            child: Text(
              t.name,
              style: T.name(t.dim ? Grey.g56 : Grey.g95).copyWith(fontWeight: t.selected ? FontWeight.w600 : FontWeight.w400),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
          t.eye,
          t.solo,
          t.lock,
        ],
      ),
    ),
  );

  /// A square of the axis, the number, and a rule. A disc travels the rule when the value has a range.
  @override
  Widget number(BuildContext context, NumberSpec n) {
    final shown = n.editing && n.editor != null ? n.editor! : n.value;
    final shape = n.axis ?? (n.live || n.hot ? Accent.key : Grey.g95);
    return SizedBox(
      height: n.height,
      child: Stack(
        children: [
          Padding(
            padding: EdgeInsets.only(right: n.glyph != null ? n.glyphBox : 0, bottom: n.fill == null ? 0 : 2),
            child: Row(
              children: [
                SizedBox.square(dimension: Shape.z(14), child: CustomPaint(painter: _MarkPaint(n.fill == null ? _Mark.quarter : _Mark.dot, shape))),
                SizedBox(width: Shape.s(6)),
                if (n.caption != null) ...[
                  Text(n.caption!, style: T.micro(n.on ? Grey.g63 : Grey.g44)),
                  SizedBox(width: Shape.s(6)),
                ],
                if (n.lead != null) Flexible(child: n.lead!),
                Expanded(child: Align(alignment: Alignment.centerRight, child: shown)),
              ],
            ),
          ),
          if (n.glyph != null) Positioned(right: 0, top: 0, bottom: 0, width: n.glyphBox, child: Center(child: n.glyph)),
          if (n.keyDot != null) Positioned(top: 2, right: 2, child: Container(width: Shape.z(4), height: Shape.z(4), decoration: BoxDecoration(color: n.keyDot, shape: BoxShape.circle))),
          if (n.fill != null) Positioned(left: Shape.z(20), right: 0, bottom: 0, height: Shape.z(10), child: CustomPaint(painter: _Range(n.fill, n.live || n.hot))),
        ],
      ),
    );
  }

  /// The picked layer is a red block. The others are a thin rule. No rounded bar.
  @override
  double get trackRadius => 0;

  @override
  double trackBar({required bool selected}) => Shape.z(selected ? 12 : 3);

  @override
  Color trackFill(Color kind, {required bool dim, required bool selected}) => dim ? Grey.g26 : (selected ? Accent.key : Grey.g38);

  @override
  Color trackBand(int index, bool selected) => Grey.g07;

  /// A quarter in front of the pane's own mark when that pane is the one open.
  @override
  Widget paneMark(BuildContext context, {required bool selected, required Widget mark}) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      SizedBox.square(dimension: Shape.z(14), child: CustomPaint(painter: _MarkPaint(selected ? _Mark.quarter : _Mark.ring, selected ? Accent.key : Grey.g38))),
      SizedBox(width: Shape.s(4)),
      mark,
    ],
  );

  @override
  bool get plainTabs => true;
}

class _Range extends CustomPainter {
  const _Range(this.fill, this.hot);
  final double? fill;
  final bool hot;

  @override
  void paint(Canvas c, Size s) {
    final y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = Grey.g26..strokeWidth = 1);
    if (fill == null) return;
    c.drawCircle(Offset(4 + (s.width - 8) * fill!.clamp(0, 1), y), hot ? 4 : 3, Paint()..color = hot ? Accent.key : Grey.g95);
  }

  @override
  bool shouldRepaint(_Range o) => o.fill != fill || o.hot != hot;
}
