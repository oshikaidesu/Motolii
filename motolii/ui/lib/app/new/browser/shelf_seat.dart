import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../foundation/theme.dart';

import '../../../browser/search.dart';
import '../../../browser/seat.dart';
import '../../../browser/things.dart';
import '../../../panels/browser/filter_library.dart';
import '../../../panels/browser/shelf.dart' show FilterGroup, FilterKind;
import '../../../panels/browser.dart' show BrowserSize;
import '../../../panels/browser/files_shelf.dart' show FilesShelf;
import '../../../panels/browser/parts.dart' show shelfAction;
import 'shelf_host.dart';
import 'tag_prompt.dart';
import 'shelf_user.dart';
import 'shelf_things.dart';
import '../../../hf/metrics.dart' show Surface;

/// The finished Browser's seat over one production shelf: a tile is picked, applied, menued and dragged the way the
/// shelf says, a picture is the shelf's own when the design has none, and the keys act on what the shelf lists.
class ShelfSeat extends ChangeNotifier implements BrowserSeat {
  ShelfSeat(this.host, this.search, {required this.mine}) {
    host.addListener(notifyListeners);
    mine.addListener(notifyListeners);
    host.extraMenu = _viewMenu;
    host.extraAct = _viewAct;
  }

  final ShelfHost host;
  final SearchCapability search;
  final ShelfUser mine;
  @override
  UserViews get user => mine.views;
  ShelfCatalog? catalog;
  var _ordered = const <Thing>[];
  var _columns = 3;
  Offset _menuAt = Offset.zero;

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
  ({double column, double extent, double gap, double padding})? tiling(BuildContext context, double width) {
    final l = host.shelf.layout(host, width, BrowserSize.base * host.tileScale);
    return l == null ? null : (column: l.column, extent: l.extent, gap: l.gap, padding: l.padding);
  }

  @override
  Widget? tools(BuildContext context) {
    final tools = [
      ...host.shelf.tools(host),
      // The places a folder listing starts from are a walk, so they sit beside the path, not among the classes.
      if (host.shelf is FilesShelf)
        Builder(builder: (context) => shelfAction('Places', () {
              final box = context.findRenderObject() as RenderBox;
              final at = box.localToGlobal(Offset(0, box.size.height));
              final rails = host.shelf.rails(host);
              showEditorMenu<String>(context, at, [for (final r in rails) EditorMenuItem<String>(value: r, child: Text(r))])
                  .then((r) => r == null ? null : host.shelf.rail(host, r));
            })),
    ];
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
                decoration: BoxDecoration(border: Border.all(color: Surface.ink.withValues(alpha: .85), width: 1.4), borderRadius: BorderRadius.circular(4)),
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
          onSecondaryTapDown: (e) {
            _menuAt = e.globalPosition;
            host.menu(item, e.globalPosition);
          },
          child: shelf.draggable(host, item, framed),
        ),
      ),
    );
  }

  Future<void> _apply(Thing thing, Map<String, dynamic> item) async {
    mine.used(thing.id);
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

  Iterable<String> _targets(Map<String, dynamic> item) {
    final id = host.id(item);
    final picked = host.selectedIds;
    return host.shelf.multiSelect && picked.length > 1 && picked.contains(id) ? picked : [id];
  }

  List<Widget> _viewMenu(Map<String, dynamic> item) {
    final now = mine.collectionOf(host.id(item));
    return [
      const EditorMenuDivider(),
      EditorMenuItem<String>(value: 'view:1', child: Text(now == 1 ? 'Remove from Favorites' : 'Add to Favorites')),
      for (var i = 2; i <= BrowserLibrary.collectionCount; i++)
        if (i != now) EditorMenuItem<String>(value: 'view:$i', child: Text('Add to ${mine.library.collectionName(i)}')),
      if (now != null && now != 1) const EditorMenuItem<String>(value: 'view:0', child: Text('Remove from collection')),
      const EditorMenuDivider(),
      const EditorMenuItem<String>(value: 'view:tag', child: Text('Tag…')),
      for (final t in mine.library.tagsOf(host.shelf.name, host.id(item))) EditorMenuItem<String>(value: 'view:untag:$t', child: Text('Remove tag "$t"')),
    ];
  }

  Future<void> _viewAct(String action, Map<String, dynamic> item) async {
    if (action == 'tag') {
      final tag = await promptText(host.context, _menuAt, 'Tag ${_targets(item).length > 1 ? '${_targets(item).length} items' : '${item['name'] ?? host.id(item)}'}');
      if (tag != null) await mine.library.tag(host.shelf.name, _targets(item), tag);
      return;
    }
    if (action.startsWith('untag:')) {
      await mine.library.untag(host.shelf.name, _targets(item), action.substring(6));
      return;
    }
    final which = int.parse(action);
    final now = mine.collectionOf(host.id(item));
    await mine.collect(_targets(item), which == now ? 0 : which);
  }

  /// The ranges a range group offers: the user's own cuts, or the shelf's seeds until they cut their own.
  List<String> _ranges(FilterGroup g) => mine.library.rangesOn(host.shelf.name, g.name) ?? g.tags;
  static String _key(FilterGroup g) => g.name.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]+'), '_');

  /// Put a range in the search text or take it out: a range filter is a word of the structured search.
  void _toggleRange(FilterGroup g, String range) {
    final word = '${_key(g)}:$range';
    final words = search.query.split(RegExp(r'\s+')).where((w) => w.isNotEmpty).toList();
    words.contains(word) ? words.remove(word) : words.add(word);
    search.controller.text = words.join(' ');
  }

  /// The header's overflow: keep this search, drop a kept one, cut a range on a continuous fact, forget what was used.
  @override
  void more(BuildContext context, Offset at) {
    final query = search.query.trim();
    final ranges = [for (final g in host.shelf.groups(host)) if (g.kind == FilterKind.range) g];
    showEditorMenu<String>(context, at, [
      EditorMenuItem<String>(value: 'save', enabled: query.isNotEmpty, child: Text(query.isEmpty ? 'Save search (type one first)' : 'Save search "$query"')),
      for (final name in user.saved.keys) EditorMenuItem<String>(value: 'drop:$name', child: Text('Remove saved "$name"')),
      if (ranges.isNotEmpty) const EditorMenuDivider(),
      for (final g in ranges) ...[
        for (final r in _ranges(g)) EditorMenuItem<String>(value: 'range:${g.name}:$r', child: Text('${g.name} ${rangeLabel(r, g.unit)}')),
        EditorMenuItem<String>(value: 'cut:${g.name}', child: Text('Add ${g.name.toLowerCase()} range…')),
        if (mine.library.rangesOn(host.shelf.name, g.name) != null) EditorMenuItem<String>(value: 'uncut:${g.name}', child: Text('Reset ${g.name.toLowerCase()} ranges')),
      ],
      const EditorMenuDivider(),
      EditorMenuItem<String>(value: 'recent', enabled: user.recent.isNotEmpty, child: const Text('Clear recent')),
    ]).then((a) async {
      if (a == null) return;
      if (a == 'save') mine.saveSearch(query, query);
      if (a == 'recent') mine.clearRecent();
      if (a.startsWith('drop:')) mine.dropSearch(a.substring(5));
      final group = a.contains(':') ? ranges.where((g) => a.split(':')[1] == g.name).firstOrNull : null;
      if (group == null) return;
      if (a.startsWith('range:')) _toggleRange(group, a.substring('range:'.length + group.name.length + 1));
      if (a.startsWith('uncut:')) await mine.library.setRanges(host.shelf.name, group.name, group.tags);
      if (a.startsWith('cut:') && context.mounted) {
        final entered = await promptText(context, at, 'Range in ${group.unit.isEmpty ? 'units' : group.unit}: 5-30, -5 or 30-');
        if (entered != null && RegExp(r'^\d*\.?\d*-\d*\.?\d*$').hasMatch(entered) && entered != '-') {
          await mine.library.setRanges(host.shelf.name, group.name, [..._ranges(group).where((r) => r != entered), entered]);
        }
      }
    });
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
    // A digit files the picked rows in a collection (1 is Favorites), 0 takes them out.
    final digit = k.keyLabel.length == 1 ? int.tryParse(k.keyLabel) : null;
    if (digit != null && digit <= BrowserLibrary.collectionCount && host.selectedIds.isNotEmpty) {
      mine.collect(host.selectedIds, digit);
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
    mine.removeListener(notifyListeners);
    super.dispose();
  }
}
