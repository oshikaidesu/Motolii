// The window's own drawing of the controls a form may replace. A form that does not override these keeps this picture.
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import '../workspace/ws.dart' show WsT;
import 'forms.dart';

class _Hot extends StatefulWidget {
  const _Hot({required this.builder, this.onTap});
  final Widget Function(bool hot) builder;
  final VoidCallback? onTap;
  @override
  State<_Hot> createState() => _HotState();
}

class _HotState extends State<_Hot> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: widget.onTap == null ? MouseCursor.defer : SystemMouseCursors.click,
    onEnter: (_) => setState(() => _h = true),
    onExit: (_) => setState(() => _h = false),
    child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(_h)),
  );
}

abstract final class Usual {
  /// A well of round items, the chosen one filled.
  static Widget segment(BuildContext context, ChoiceSpec c) => Container(
    height: Shape.z(20),
    padding: EdgeInsets.all(Shape.s(2)),
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var i = 0; i < c.labels.length; i++)
          _Hot(
            onTap: () => c.onSelect(i),
            builder: (h) => Container(
              height: Shape.z(16),
              padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
              alignment: Alignment.center,
              decoration: BoxDecoration(color: i == c.selected ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
              child: Text(c.labels[i], style: T.label(i == c.selected ? WsT.onAccent : (h ? Grey.g95 : Grey.g76)).copyWith(fontWeight: FontWeight.w600)),
            ),
          ),
      ],
    ),
  );

  /// An on/off chip: raised, with the mark in the accent, when on.
  static Widget flag(BuildContext context, FlagSpec f) => _Hot(
    onTap: f.onTap,
    builder: (h) {
      final ink = f.on ? WsT.accent : (h ? Grey.g91 : Grey.g56);
      return Container(
        height: Shape.z(20),
        padding: EdgeInsets.only(left: Shape.s(6), right: Shape.s(8)),
        decoration: BoxDecoration(color: f.on ? Grey.g26 : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (f.mark != null) f.mark!(ink),
            if (f.mark != null) SizedBox(width: Shape.s(5)),
            Text(f.label, style: T.label(f.on || h ? Grey.g95 : Grey.g63).copyWith(fontWeight: FontWeight.w600)),
          ],
        ),
      );
    },
  );

  /// Round filter chips in a sideways scroll, moved as any sideways list on a Mac is: a trackpad's sideways swipe, or a
  /// mouse's wheel with Shift held (Flutter's own ScrollBehavior.pointerAxisModifiers); the wheel alone moves what is
  /// up and down, not this (golf_noise.md (93)). The active one is the accent.
  static Widget chips(BuildContext context, ChipsSpec c) => SizedBox(
    height: Shape.z(28),
    child: SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(2), WsT.inset, Shape.s(6)),
      child: Row(
        children: [
          for (final (name, n) in c.chips) ...[
            _Hot(
              onTap: () => c.onPick(name),
              builder: (h) {
                final on = name == c.active;
                return AnimatedContainer(
                  duration: Mo.dur,
                  curve: Mo.ease,
                  height: Shape.z(20),
                  padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
                  decoration: BoxDecoration(color: on ? WsT.accent : (h ? WsT.raised : WsT.well), borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(name, style: T.micro(on ? WsT.onAccent : Grey.g91).copyWith(fontWeight: on ? FontWeight.w700 : FontWeight.w500)),
                      if (n != null) ...[SizedBox(width: Shape.s(4)), Text('$n', style: T.value(on ? WsT.onAccent : Grey.g56).copyWith(fontSize: T.px(10)))],
                    ],
                  ),
                );
              },
            ),
            SizedBox(width: WsT.gap),
          ],
        ],
      ),
    ),
  );

  /// One timeline name: a band, an accent edge when picked, the kind and the three toggles.
  static Widget track(BuildContext context, TrackSpec t) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: t.onSelect,
    child: Container(
      color: t.selected ? Grey.g20 : (t.hot ? Grey.g15 : (t.index.isEven ? Grey.bandA : Grey.bandB)),
      padding: EdgeInsets.only(bottom: Shape.s(1)),
      child: Stack(
        children: [
          if (t.selected) Positioned(left: 0, top: 0, bottom: 0, width: Shape.z(3), child: ColoredBox(color: WsT.accent)),
          Padding(
            padding: EdgeInsets.only(left: Shape.s(8), right: Shape.s(6)),
            child: Row(
              children: [
                SizedBox(width: 14, child: Text('${t.index + 1}', softWrap: false, overflow: TextOverflow.visible, style: T.value(t.selected ? Grey.g91 : Grey.g56).copyWith(fontSize: T.px(10)))),
                SizedBox(width: Shape.s(6)),
                t.kind,
                SizedBox(width: Shape.s(7)),
                Expanded(
                  child: Text(
                    t.name,
                    style: T.name(t.dim ? Grey.g56 : (t.selected ? Grey.g100 : Grey.g91)).copyWith(fontWeight: t.selected ? FontWeight.w700 : FontWeight.w500),
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
        ],
      ),
    ),
  );
}
