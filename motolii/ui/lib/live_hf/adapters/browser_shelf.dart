import 'dart:convert';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/shell.dart' show GlyphBox;
import '../../hf/shell/menu.dart' show showHfMenu;
import '../../hf/bp/seat.dart';
import '../../hf/bp/common.dart' show sans;
import '../../hf/bp/shelf_grid.dart';
import '../../hf/bp/shelf_sections.dart' show PickedRing;
import '../../hf/bp/things.dart';
import '../../hf/glyphs.dart';
import '../../session/editor_session.dart';
import 'browser_session.dart';
import 'browser_user.dart';
import 'media_library.dart';
import '../../hf/neutral.dart';
import '../../hf/shell/place.dart' show H;
import '../../hf/desk/common.dart' show kPink;

/// The Media shelf as a skin over [BrowserSession]'s library: the hf shelf grammar lays it out, tiles are dressed with
/// their state, and gestures go to the session.
class LiveBrowserShelf extends StatefulWidget {
  const LiveBrowserShelf({super.key, required this.controller, required this.name, required this.user});

  final EditorSession controller;
  final String name;
  final LiveBrowserUser user;

  @override
  State<LiveBrowserShelf> createState() => _LiveBrowserShelfState();
}

/// A family's word on the shelf (the library's own family names are the menus' and facts').
const _familyLabel = {'2D': 'Images', 'Folder': 'Folders'};

class _LiveBrowserShelfState extends State<LiveBrowserShelf> {
  EditorSession get c => widget.controller;
  late final s = BrowserSession.of(c);
  late final _Seat seat = _Seat(s, widget.name);

  static const _registryShape = {
    'kinds': {
      'item': {'label': 'Item'},
    },
    'tags': ['image', 'video', 'audio', '3d', 'folder', 'imported'],
    'capabilities': {},
    'faces': {'shelf': {}},
    'panels': {
      'shelf': {
        'title': 'Shelf',
        'kinds': ['item'],
      },
    },
  };

  /// The library as the hf shelf grammar reads it: its things, their family word, the shelf face.
  List<Map<String, dynamic>> _items() => [
        for (final item in s.mediaItems)
          {...item, 'kind': 'item', 'family': _familyLabel[item['family']] ?? item['family'], 'source': 'builtin', 'capabilities': <String>[], 'face': const {'type': 'shelf'}},
      ];

  Catalog _catalog(List<Map<String, dynamic>> rows) {
    final families = <String, Map<String, dynamic>>{};
    for (final row in rows) {
      final family = '${row['family']}';
      families.putIfAbsent(family, () => {'label': family, 'kinds': ['item']});
    }
    return Catalog(Registry.fromJson({..._registryShape, 'families': families}), [for (final row in rows) (widget.name, row)]);
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([s, c.dragging]),
        builder: (context, _) {
          final items = _items();
          seat.itemsById
            ..clear()
            ..addAll({for (final item in items) '${item['id']}': item});
          final shelf = BrowserSeatScope(
            seat: seat,
            child: ShelfGridPanel(
              title: 'Media',
              noun: 'clip',
              icon: const GlyphBox(HG.image, size: 22),
              panelId: 'shelf',
              catalog: _catalog(items),
              user: widget.user.views,
              search: s.search('Media'),
              classify: s.classify('Media'),
              sections: true,
              classStrip: true,
              body: (context, sections, shown, size) => MediaLibraryBody(sections: sections, shown: shown, items: seat.itemsById, width: size.width),
            ),
          );
          // while files are carried over the window, the shelf says where they go (Classic BR-084); it takes no pointer
          return Stack(fit: StackFit.passthrough, children: [
            shelf,
            Positioned.fill(
              child: IgnorePointer(
                child: AnimatedOpacity(
                  opacity: c.dragging.value ? 1 : 0,
                  duration: const Duration(milliseconds: 120),
                  child: Container(
                    key: const ValueKey('media-drop-hint'),
                    margin: const EdgeInsets.all(6),
                    alignment: Alignment.center,
                    decoration: BoxDecoration(color: N.veil, border: Border.all(color: N.g95, width: 1.4), borderRadius: BorderRadius.circular(4.5)),
                    child: Text('Drop to import', style: sans(11, c: N.g95)),
                  ),
                ),
              ),
            ),
          ]);
        },
      );
}

/// The colours the collections 2–7 are named after (Orange, Yellow, Green, Blue, Purple, Gray).
/// a warning mark on a tile (a missing file)
const _warn = Color(0xFFFFD166);
final _collectionColors = [H.follow.b, H.face.b, H.along.b, H.stagger.b, H.attach.b, N.g63];

class _Seat extends ChangeNotifier implements BrowserSeat {
  _Seat(this.s, this.name) {
    s.addListener(notifyListeners);
  }
  final BrowserSession s;
  final String name;
  EditorSession get c => s.c;
  LiveBrowserUser get userState => s.user(name);
  final itemsById = <String, Map<String, dynamic>>{};

  /// How many tiles a row of the body holds: the arrows' up and down step (the skin's own geometry).
  int columns = 1;

  @override
  Widget tile(BuildContext context, Thing thing, Widget tile) {
    final item = itemsById[thing.id];
    if (item == null) return Opacity(opacity: .5, child: tile);
    final favorite = userState.views.favorites.contains(thing.id);
    final carried = s.mediaCarry(thing.id);
    // A media tile carried to the Timeline is placed where it lands (Classic's Media drag, the same data).
    Widget carry(Widget child) => carried == null
        ? child
        : Draggable<Map<String, dynamic>>(
            data: carried,
            dragAnchorStrategy: pointerDragAnchorStrategy,
            feedback: IgnorePointer(child: Opacity(opacity: .85, child: SizedBox(width: 54, height: 54, child: tile))),
            child: child,
          );
    return carry(
      GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: () {
          final k = HardwareKeyboard.instance;
          s.pickMedia(thing.id, range: k.isShiftPressed, toggle: k.isMetaPressed || k.isControlPressed);
        },
        onDoubleTap: () => s.placeMedia(thing.id),
        onSecondaryTapDown: (event) {
          if (!s.mediaPicked.contains(thing.id)) s.pickMedia(thing.id);
          more(context, event.globalPosition);
        },
        child: Stack(
          fit: StackFit.passthrough,
          children: [
            tile,
            if (s.mediaPicked.contains(thing.id)) const Positioned.fill(child: PickedRing()),
            // state at a glance (Classic BR-059/096): a missing file, in use, and the collection it is kept in
            if (item['missing'] == true)
              const Positioned(left: 3, top: 3, child: IgnorePointer(child: Text('!', style: TextStyle(color: kPink, fontSize: 11, fontWeight: FontWeight.w800)))),
            if (item['used'] == true)
              Positioned(right: 3, bottom: 3, child: IgnorePointer(child: Container(width: 4.5, height: 4.5, decoration: const BoxDecoration(color: H.play, shape: BoxShape.circle)))),
            if (userState.collectionOf(thing.id) case final n? when n > 1)
              Positioned(left: 3, bottom: 3, child: IgnorePointer(child: Container(width: 5, height: 5, decoration: BoxDecoration(color: _collectionColors[n - 2], shape: BoxShape.circle)))),
            if (favorite) const Positioned(right: 3, top: 3, child: IgnorePointer(child: Text('★', style: TextStyle(color: _warn, fontSize: 11)))),
          ],
        ),
      ),
    );
  }

  @override
  Widget? face(BuildContext context, Thing thing) {
    final item = itemsById[thing.id] ?? const <String, dynamic>{};
    final path = item['path'] as String?;
    final thumbnail = item['thumbnail'] as String?;
    if (thumbnail != null && thumbnail.startsWith('data:')) {
      final comma = thumbnail.indexOf(',');
      if (comma >= 0) {
        try {
          return Image.memory(base64Decode(thumbnail.substring(comma + 1)), fit: BoxFit.cover, gaplessPlayback: true);
        } catch (_) {}
      }
    }
    if (path != null && item['mime'] is String && '${item['mime']}'.startsWith('image/')) {
      return Image.file(File(path), fit: BoxFit.cover, errorBuilder: (_, __, ___) => _identity(item));
    }
    return _identity(item);
  }

  Widget _identity(Map<String, dynamic> item) {
    final family = '${item['family'] ?? item['mime'] ?? 'Media'}';
    return Center(
      child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [
        GlyphBox(family == 'Audio' ? HG.headphones : HG.image, size: 24),
        const SizedBox(height: 4.5),
        Text(family, maxLines: 1, overflow: TextOverflow.ellipsis, style: const TextStyle(color: N.g76, fontSize: 9.5)),
      ]),
    );
  }

  @override
  ({double column, double extent, double gap, double padding})? tiling(BuildContext context, double width) => null;
  @override
  double get tileScale => 1;
  @override
  Widget? tools(BuildContext context) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: c.importFiles,
        // its own place after the classes: a hairline, then a quiet tool with a small picture of what it brings in
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Container(width: 1, height: 10.5, color: N.g20),
          const SizedBox(width: 4.5),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 4.5),
            child: Row(mainAxisSize: MainAxisSize.min, children: [
              const GlyphBox(HG.plus, size: 11, color: N.g63),
              const SizedBox(width: 3),
              Text('Import', style: sans(11, c: N.g76, w: FontWeight.w500)),
            ]),
          ),
        ]),
      );
  @override
  Widget? header(BuildContext context) => null;
  @override
  Widget? editor(BuildContext context) => null;

  @override
  KeyEventResult key(FocusNode node, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    // a key typed into a field (the search) is the field's, never the shelf's (Classic frame_keys)
    final typing = FocusManager.instance.primaryFocus?.context;
    if (typing != null && (typing.widget is EditableText || typing.findAncestorWidgetOfExactType<EditableText>() != null)) return KeyEventResult.ignored;
    final picked = s.mediaPicked;
    final digit = int.tryParse(event.logicalKey.keyLabel);
    if (digit != null && digit <= 7 && picked.isNotEmpty) {
      userState.collect(picked, digit);
      return KeyEventResult.handled;
    }
    final k = event.logicalKey;
    final cmd = HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed;
    if (k == LogicalKeyboardKey.enter && picked.isNotEmpty) {
      s.placeMedia(picked.last);
    } else if ((k == LogicalKeyboardKey.delete || k == LogicalKeyboardKey.backspace) && picked.isNotEmpty) {
      // Delete removes the picked files from the library (the ones nothing uses); it never reaches the layers
      s.removePickedMedia();
    } else if (k == LogicalKeyboardKey.escape && picked.isNotEmpty) {
      s.clearMedia();
    } else if (k == LogicalKeyboardKey.keyA && cmd) {
      s.pickAllMedia();
    } else if (k == LogicalKeyboardKey.home) {
      s.pickFirstMedia();
    } else if (k == LogicalKeyboardKey.end) {
      s.pickLastMedia();
    } else if (switch (k) {
      LogicalKeyboardKey.arrowLeft => -1,
      LogicalKeyboardKey.arrowRight => 1,
      LogicalKeyboardKey.arrowUp => -columns,
      LogicalKeyboardKey.arrowDown => columns,
      _ => null,
    } case final step?) {
      s.stepMedia(step);
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  @override
  void shows(List<Thing> things, int columns) {
    s.mediaOrder = [for (final t in things) t.id];
    this.columns = columns;
  }

  @override
  void more(BuildContext context, Offset at) {
    if (s.mediaPicked.isEmpty) return;
    final id = s.mediaPicked.last;
    final item = itemsById[id];
    // Classic's Media menu: the file's name and facts (read only), Place, its own actions, then the collections
    final actions = s.mediaActionsOf(id);
    final facts = s.mediaFactsOf(id);
    final picked = s.mediaPicked.toList();
    showHfMenu<String>(
      context,
      Rect.fromLTWH(at.dx, at.dy, 240, 0),
      [
        if (item != null) ('title', '${item['name'] ?? id}'),
        for (final (i, f) in facts.indexed) ('fact:$i', f),
        ('apply', 'Place'),
        for (final a in actions) ('media:${a.value}', a.label),
        ...userState.collectionLines(id),
      ],
      info: {'title', for (var i = 0; i < facts.length; i++) 'fact:$i'},
      disabled: {for (final a in actions) if (!a.enabled) 'media:${a.value}'},
      dividers: {if (facts.isNotEmpty) 'fact:${facts.length - 1}' else if (item != null) 'title', 'apply', if (actions.isNotEmpty) 'media:${actions.last.value}'},
    ).then((action) {
      if (action == 'apply') s.placeMedia(id);
      if (action != null && action.startsWith('collect:')) userState.collect(action == 'collect:0' ? [id] : picked, int.parse(action.substring(8)));
      if (action != null && action.startsWith('media:')) s.mediaAction(id, action.substring(6));
    });
  }

  @override
  UserViews? get user => userState.views;
  @override
  Listenable get changes => this;

  @override
  void dispose() {
    s.removeListener(notifyListeners);
    super.dispose();
  }
}
