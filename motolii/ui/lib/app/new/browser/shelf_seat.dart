import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../foundation/theme.dart';
import '../../../hf/desk/common.dart' show kInk;
import '../../../hf/bp/search.dart';
import '../../../hf/bp/seat.dart';
import '../../../hf/bp/things.dart';
import 'shelf_host.dart';
import 'shelf_things.dart';

/// The finished Browser's seat over one production shelf: a tile is picked, applied, menued and dragged the way the
/// shelf says, a picture is the shelf's own when the design has none, and the keys act on what the shelf lists.
class ShelfSeat extends ChangeNotifier implements BrowserSeat {
  ShelfSeat(this.host, this.search, {required this.user}) {
    host.addListener(notifyListeners);
  }

  final ShelfHost host;
  final SearchCapability search;
  @override
  final UserViews user;
  ShelfCatalog? catalog;
  var _ordered = const <Thing>[];
  var _columns = 3;

  @override
  Listenable get changes => this;

  @override
  double get tileScale => host.tileScale;

  Map<String, dynamic>? _item(Thing thing) => catalog?.byId[thing.id];

  @override
  void shows(List<Thing> things, int columns) {
    _ordered = things;
    _columns = columns;
    host.visible = [for (final t in things) if (_item(t) != null) _item(t)!];
  }

  @override
  Widget? face(BuildContext context, Thing thing) {
    if (thing.face['type'] != 'shelf') return null;
    final item = _item(thing);
    if (item == null) return null;
    final identity = EditorTheme.of(context).kindColor(host.shelf.identity(host, item));
    return host.shelf.preview(host, item, identity);
  }

  @override
  Widget? tools(BuildContext context) {
    final tools = host.shelf.tools(host);
    return tools.isEmpty ? null : Row(mainAxisSize: MainAxisSize.min, children: tools);
  }

  @override
  Widget? header(BuildContext context) => host.shelf.header(host);

  @override
  Widget? editor(BuildContext context) => host.shelf.editor(host);

  @override
  Widget tile(BuildContext context, Thing thing, Widget tile) {
    final item = _item(thing);
    if (item == null) return tile;
    final shelf = host.shelf;
    final bare = shelf.bare;
    final twice = shelf.doubleClick(host, item);
    final supported = shelf.supported(host, item);
    final picked = host.selectedIds.contains(thing.id);
    final framed = Stack(
      fit: StackFit.passthrough,
      children: [
        tile,
        if (picked)
          Positioned.fill(
            child: IgnorePointer(
              child: DecoratedBox(
                decoration: BoxDecoration(border: Border.all(color: kInk.withValues(alpha: .85), width: 1.4), borderRadius: BorderRadius.circular(4)),
              ),
            ),
          ),
      ],
    );
    return EditorTooltip(
      message: ['${item['name'] ?? item['id']}', if (item['detail'] != null) '${item['detail']}'].join('\n'),
      child: Listener(
        onPointerDown: (e) {
          if (e.buttons != kPrimaryButton) return;
          host.pick(
            item,
            range: HardwareKeyboard.instance.isShiftPressed,
            toggle: HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed,
          );
        },
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: bare && supported && !twice ? () => _apply(thing, item) : null,
          onDoubleTap: !bare || (twice && supported) ? () => _apply(thing, item) : null,
          onSecondaryTapDown: (e) => host.menu(item, e.globalPosition),
          child: shelf.draggable(host, item, framed),
        ),
      ),
    );
  }

  Future<void> _apply(Thing thing, Map<String, dynamic> item) async {
    user.recent
      ..remove(thing.id)
      ..insert(0, thing.id);
    if (user.recent.length > 12) user.recent.removeRange(12, user.recent.length);
    // Several picked on a shelf that lets that be: the shelf applies each.
    final picked = host.selectedIds;
    if (host.shelf.multiSelect && picked.length > 1 && picked.contains(thing.id)) {
      for (final other in [for (final t in _ordered) if (picked.contains(t.id)) t]) {
        final o = _item(other);
        if (o != null) await host.apply(o);
      }
    } else {
      await host.apply(item);
    }
    notifyListeners();
  }

  @override
  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent || FocusManager.instance.primaryFocus?.context?.findAncestorWidgetOfExactType<EditableText>() != null) {
      return KeyEventResult.ignored;
    }
    final k = event.logicalKey;
    if (k == LogicalKeyboardKey.escape && !search.active) {
      host.clearSelection();
      return KeyEventResult.handled;
    }
    if (_ordered.isEmpty) return KeyEventResult.ignored;
    final at = _ordered.indexWhere((t) => host.selectedIds.contains(t.id));
    Thing at0() => _ordered[at < 0 ? 0 : at];
    if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter) {
      final item = _item(at0());
      if (item != null) _apply(at0(), item);
      return KeyEventResult.handled;
    }
    int? next;
    if (k == LogicalKeyboardKey.arrowLeft) next = at - 1;
    if (k == LogicalKeyboardKey.arrowRight) next = at + 1;
    if (k == LogicalKeyboardKey.arrowUp) next = at - _columns;
    if (k == LogicalKeyboardKey.arrowDown) next = at + _columns;
    if (k == LogicalKeyboardKey.home) next = 0;
    if (k == LogicalKeyboardKey.end) next = _ordered.length - 1;
    if (next != null) {
      final item = _item(_ordered[(at < 0 ? 0 : next).clamp(0, _ordered.length - 1)]);
      if (item != null) {
        if (HardwareKeyboard.instance.isShiftPressed) {
          host.pick(item, range: true);
        } else {
          host.select(item);
        }
      }
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) {
      final item = _item(at0());
      if (item != null) host.shelf.delete(host, item);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    host.removeListener(notifyListeners);
    super.dispose();
  }
}
