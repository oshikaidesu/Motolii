// The window's own forms: each control as the window has always drawn it.
import 'package:flutter/widgets.dart';

import '../parts/controls.dart' show PillSwitch;
import '../parts/skin_parts.dart';
import '../sets/inspector/inspector_parts.dart';
import '../tokens.dart';
import '../workspace/ws.dart' show WsT;
import 'forms.dart';

class OwnForms extends Forms {
  const OwnForms();

  /// A flat lime pill with dark ink (or the skin's frame for it).
  @override
  Widget action(BuildContext context, ActionSpec a) => Hov(
    onTap: a.onPressed,
    builder: (_, h) => SkinFrame(
      'render',
      hover: h,
      usual: BoxDecoration(color: h ? WsT.accentHot : WsT.accent, borderRadius: BorderRadius.all(Shape.r(13))),
      child: Container(
        height: Shape.z(26),
        padding: EdgeInsets.symmetric(horizontal: Shape.s(14)),
        alignment: Alignment.center,
        child: Text(a.label, style: T.name(WsT.onAccent).copyWith(fontWeight: FontWeight.w800)),
      ),
    ),
  );

  /// Plates side by side, each its glyph and word, the one picked lit.
  @override
  Widget choice(BuildContext context, ChoiceSpec c) => Row(
    children: [
      for (var i = 0; i < c.labels.length; i++) ...[
        if (i > 0) SizedBox(width: Shape.s(4)),
        Expanded(
          child: GlyphPlate(c.labels[i], i < c.glyphs.length ? c.glyphs[i] : (_, _) {}, on: i == c.selected, onTap: () => c.onSelect(i)),
        ),
      ],
    ],
  );

  /// Words in a well, the one picked lit in the accent.
  @override
  Widget tabs(BuildContext context, ChoiceSpec c) => Container(
    height: Shape.z(26),
    padding: EdgeInsets.all(Shape.s(2)),
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final (i, s) in c.labels.indexed)
          Hov(
            onTap: () => c.onSelect(i),
            builder: (_, h) => Container(
              width: Shape.z(76),
              alignment: Alignment.center,
              decoration: BoxDecoration(color: i == c.selected ? WsT.accent : (h ? WsT.raised : null), borderRadius: BorderRadius.all(Shape.r(WsT.radius - 2))),
              child: Text(
                s,
                style: T.name(i == c.selected ? WsT.onAccent : (h ? Grey.g95 : Grey.g76)).copyWith(fontWeight: i == c.selected ? FontWeight.w700 : FontWeight.w500),
              ),
            ),
          ),
      ],
    ),
  );

  /// The word and a switch: the state is a word, never colour alone.
  @override
  Widget onOff(BuildContext context, SwitchSpec s) => IgnorePointer(
    ignoring: !s.enabled,
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: Shape.z(20),
          child: Text(s.on ? 'On' : 'Off', textAlign: TextAlign.right, style: T.label(s.enabled && s.on ? Grey.g95 : Grey.g56)),
        ),
        SizedBox(width: Shape.s(8)),
        Opacity(
          opacity: s.enabled ? 1 : .5,
          child: Pop.on(context) ? PopSwitch(on: s.on, onChanged: s.onChanged) : PillSwitch(on: s.on, onChanged: s.onChanged),
        ),
      ],
    ),
  );

  /// A flat card (or the skin's frame for it).
  @override
  Widget card(BuildContext context, CardSpec c) => SkinFrame('card', usual: BoxDecoration(color: Pop.card), child: c.own);
}
