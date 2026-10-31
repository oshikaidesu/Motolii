// ws_menu.dart: the window's right-click menu: what can be done to what is under the hand, there, so the mouse alone
// finishes it (Ableton Live's context menus, everywhere; Figma's right-click on a thing). Flutter's own RawMenuAnchor
// opens it where the hand is and shuts it on a click outside; its rows are the Inspector's right-click rows
// (inspector_rows_prop_row.dart: a word, and why when it cannot be done), floating as the design's context menu does
// (sets/foundations/navigation_2.dart: the keys it is pressed with too, dim on the right).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_small.dart' show FloatShut, Hov;
import '../tokens.dart';
import 'keys.dart' show CommandIntent;
import 'ws.dart';

/// A thing's own menu by a right click, on its bar, its row or the stage (Ableton Live's clip context menu: Rename,
/// Duplicate, Split, Deactivate, Delete; Figma's right click on a layer or a thing: mouse_map.md, 7), each the window's
/// own command as the menu bar and the keys say it; the thing is picked first, so it is the one they work on.
List<WsMenuItem> wsThingMenu(BuildContext context, Ws ws, WsLayer l) {
  ws.select(l.id);
  void say(String c) => Actions.maybeInvoke(context, CommandIntent(c, false));
  return [
    WsMenuItem('Rename…', () => say('thing.rename'), keys: '⌘R'),
    WsMenuItem('Duplicate', () => say('thing.duplicate'), keys: '⌘D'),
    WsMenuItem('Split', ws.frame > l.inF && ws.frame < l.outF ? () => say('thing.split') : null, keys: '⌘E', why: 'The playhead is not inside it'),
    const WsMenuItem.rule(),
    WsMenuItem(l.hidden ? 'Activate' : 'Deactivate', () => say('thing.deactivate'), keys: '0'),
    WsMenuItem('Delete', () => say('thing.remove'), keys: '⌫'),
  ];
}

/// One row: [go] null, it cannot be done now and [why] says why; [keys] what it is pressed with too (never the only way).
/// A rule between groups: [WsMenuItem.rule].
class WsMenuItem {
  const WsMenuItem(this.label, this.go, {this.keys = '', this.why = ''}) : colours = const [], picked = -1, pick = null;
  const WsMenuItem.rule() : label = '', go = null, keys = '', why = '', colours = const [], picked = -1, pick = null;

  /// Colours to pick one of, as Ableton Live's context menus end in the clip's or the track's colours: [picked] the one
  /// it has (-1: none), [pick] told the number of the one pressed.
  const WsMenuItem.colours(this.colours, this.picked, this.pick) : label = '', go = null, keys = '', why = '';
  final String label, keys, why;
  final VoidCallback? go;
  final List<Color> colours;
  final int picked;
  final void Function(int)? pick;
}

class WsMenu extends StatefulWidget {
  const WsMenu({super.key, required this.child});
  final Widget child;
  @override
  State<WsMenu> createState() => WsMenuState();
}

class WsMenuState extends State<WsMenu> {
  final _m = MenuController();
  List<WsMenuItem> _items = const [];

  /// Opens at [at], a place of the child, with [items].
  void show(Offset at, List<WsMenuItem> items) {
    setState(() => _items = items);
    _m.open(position: at);
  }

  double get _row => Shape.z(22);
  double get _swatch => Shape.z(20);
  double _high(WsMenuItem it) => it.colours.isNotEmpty ? (it.colours.length / 8).ceil() * (_swatch + Shape.s(4)) + Shape.s(4) : it.label.isEmpty ? Shape.s(8) + 1 : _row;

  Widget _one(WsMenuItem it) => it.colours.isNotEmpty
      ? Padding(   // (eight to a row)
          padding: EdgeInsets.all(Shape.s(4)),
          child: Wrap(
            spacing: Shape.s(4),
            runSpacing: Shape.s(4),
            children: [
              for (var k = 0; k < it.colours.length; k++)
                Hov(
                  key: ValueKey('ws-menu-colour-$k'),
                  cursor: SystemMouseCursors.click,
                  onTap: () {
                    _m.close();
                    it.pick?.call(k);
                  },
                  builder: (_, h) => Container(
                    width: _swatch,
                    height: _swatch,
                    decoration: BoxDecoration(color: it.colours[k], borderRadius: BorderRadius.all(Shape.r(3)), border: Border.all(color: k == it.picked || h ? Grey.g95 : const Color(0x00000000), width: 2)),
                  ),
                ),
            ],
          ),
        )
      : it.label.isEmpty
      ? Padding(padding: EdgeInsets.symmetric(vertical: Shape.s(4)), child: SizedBox(height: 1, child: ColoredBox(color: Grey.g20)))
      : Hov(
          key: ValueKey('ws-menu-${it.label}'),
          cursor: it.go != null ? SystemMouseCursors.click : SystemMouseCursors.basic,
          onTap: it.go == null
              ? null
              : () {
                  _m.close();
                  it.go!();
                },
          builder: (_, h) => Container(
            height: _row,
            padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
            decoration: BoxDecoration(color: h && it.go != null ? Grey.g20 : null, borderRadius: BorderRadius.all(Shape.r(4))),
            child: Row(
              children: [
                Expanded(child: Text(it.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(it.go != null ? Grey.g91 : Grey.g56))),
                if (it.go == null && it.why.isNotEmpty) Text(it.why, maxLines: 1, style: T.label(Grey.g56)) else if (it.keys.isNotEmpty) Text(it.keys, maxLines: 1, style: T.label(Grey.g56)),
              ],
            ),
          ),
        );

  @override
  Widget build(BuildContext context) => RawMenuAnchor(
    controller: _m,
    overlayBuilder: (context, info) {
      final w = Shape.z(220), pad = Shape.s(4);
      final h = math.min(info.overlaySize.height * .5, [for (final it in _items) _high(it)].fold(0.0, (a, b) => a + b)) + pad * 2;
      final at = info.anchorRect.topLeft + (info.position ?? Offset.zero);
      return Positioned(
        left: math.max(0.0, math.min(at.dx, info.overlaySize.width - w)),
        top: at.dy + h <= info.overlaySize.height ? at.dy : math.max(0.0, at.dy - h),   // (down from the hand, or up where there is no room)
        child: FloatShut(   // (the anchor shuts it on a left press outside and no more: every press, and Esc, as all that floats)
          group: info.tapRegionGroupId,
          shut: _m.close,
          child: Container(
            key: const ValueKey('ws-menu'),
            width: w,
            padding: EdgeInsets.all(pad),
            decoration: BoxDecoration(
              color: Grey.g13,
              borderRadius: BorderRadius.all(Shape.r(6)),
              border: Border.all(color: Grey.g26),
              boxShadow: [BoxShadow(color: const Color(0x80000000), blurRadius: 24, offset: const Offset(0, 8))],
            ),
            child: ConstrainedBox(   // (a long one, as many palettes as are kept, scrolls within half the window)
              constraints: BoxConstraints(maxHeight: info.overlaySize.height * .5),
              child: SingleChildScrollView(child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [for (final it in _items) _one(it)])),
            ),
          ),
        ),
      );
    },
    child: widget.child,
  );
}
