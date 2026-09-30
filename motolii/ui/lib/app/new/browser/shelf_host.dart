import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../../theme/editor_theme.dart';
import '../../../panels/browser.dart' show browserDocumentKeys;
import '../../../panels/browser/colors_shelf.dart';
import '../../../panels/browser/create_shelf.dart';
import '../../../panels/browser/effects_shelf.dart';
import '../../../panels/browser/files_shelf.dart';
import '../../../panels/browser/fonts_shelf.dart';
import '../../../panels/browser/media_shelf.dart';
import '../../../panels/browser/shelf.dart';
import '../../../session/editor_session.dart';

BrowserShelf shelfNamed(String name) => switch (name) {
  'Create' => CreateShelf(),
  'Media' => MediaShelf(),
  'Effects' => EffectsShelf(),
  'Fonts' => FontsShelf(),
  'Colors' => ColorsShelf(),
  'Files' => FilesShelf(),
  _ => throw ArgumentError('No shelf called $name'),
};

/// One shelf of the production Browser, held without any drawing: what it lists, what is picked, and the operations a
/// shelf asks of its panel. The shelf stays the owner of items, apply, menus and drag; the panel that draws it (the
/// finished Browser bodies) reads and calls this. A change here tells its listeners.
class ShelfHost extends ChangeNotifier implements BrowserHost {
  ShelfHost(this.controller, this.shelf, this._context, this._mounted, {this.tileScaleOf = _one}) {
    controller.deskWork.addListener(_document);
    _slice = controller.slice(
      'newBrowser:${shelf.name}',
      browserDocumentKeys,
      derived: () => shelf.derived(controller),
    )..addListener(_document);
    shelf.enter(this);
  }

  static double _one() => 1;

  /// Menu rows the finished Browser adds after the shelf's own (favorites, collections), and what they do.
  List<Widget> Function(Map<String, dynamic> item)? extraMenu;
  Future<void> Function(String action, Map<String, dynamic> item)? extraAct;

  @override
  final EditorSession controller;
  @override
  final BrowserShelf shelf;
  final BuildContext Function() _context;
  final bool Function() _mounted;
  final double Function() tileScaleOf;
  late final DocumentSlice _slice;

  @override
  BuildContext get context => _context();
  @override
  bool get mounted => _mounted();
  @override
  String get tab => shelf.name;

  /// What the body shows right now, in order; the body reports it.
  @override
  List<Map<String, dynamic>> visible = const [];
  final _picked = <String>{};
  double _tileWidth = 62;

  @override
  Set<String> get selectedIds => _picked;
  @override
  double get tileScale => tileScaleOf();
  @override
  double get captionHeight => 22;
  @override
  double get tileWidth => _tileWidth;
  set tileWidth(double v) => _tileWidth = v;
  @override
  int get viewMode => 0;

  @override
  bool has(String op) => (controller.state['capabilities'] as List? ?? const []).contains(op);
  @override
  String id(Map<String, dynamic> item) => '${item['id'] ?? item['hex']}';

  /// Everything the shelf lists, before search and class: the source of the Things.
  List<Map<String, dynamic>> items() => shelf.items(this);

  void _document() {
    if (_mounted()) notifyListeners();
  }

  @override
  void select(Map<String, dynamic> item) {
    _picked
      ..clear()
      ..add(id(item));
    notifyListeners();
  }

  /// Shift adds a range and Cmd toggles, on a shelf that lets several be picked.
  void pick(Map<String, dynamic> item, {bool range = false, bool toggle = false}) {
    final key = id(item);
    if (!shelf.multiSelect || (!range && !toggle)) {
      select(item);
      return;
    }
    if (toggle) {
      _picked.contains(key) ? _picked.remove(key) : _picked.add(key);
    } else {
      final at = visible.indexWhere((e) => id(e) == key);
      final from = visible.indexWhere((e) => _picked.contains(id(e)));
      if (at < 0 || from < 0) {
        _picked.add(key);
      } else {
        for (var i = at < from ? at : from; i <= (at < from ? from : at); i++) {
          _picked.add(id(visible[i]));
        }
      }
    }
    notifyListeners();
  }

  @override
  Future<void> apply(Map<String, dynamic> item) => shelf.apply(this, item);

  @override
  void menu(Map<String, dynamic> item, Offset point) {
    select(item);
    showEditorMenu<String>(context, point, [
      EditorMenuItem<String>(enabled: false, child: Text('${item['name'] ?? item['hex'] ?? item['id']}')),
      for (final fact in shelf.facts(this, item).where((f) => f.isNotEmpty)) EditorMenuItem<String>(enabled: false, child: Text(fact)),
      const EditorMenuDivider(),
      EditorMenuItem<String>(value: 'apply', child: Text(shelf.applyLabel(this, item))),
      ...shelf.menu(this, item),
      ...?extraMenu?.call(item),
    ]).then((action) {
      if (!_mounted() || action == null) return;
      if (action == 'apply') {
        apply(item);
      } else if (action.startsWith('view:')) {
        extraAct?.call(action.substring(5), item);
      } else {
        shelf.act(this, action, item);
      }
    });
  }

  @override
  void refresh([VoidCallback? change]) {
    change?.call();
    notifyListeners();
  }

  @override
  void relist() => notifyListeners();
  @override
  void clearSelection() {
    _picked.clear();
    notifyListeners();
  }

  @override
  void showCategory(String shelf, String category) => notifyListeners();

  @override
  void dispose() {
    controller.deskWork.removeListener(_document);
    _slice.removeListener(_document);
    shelf.dispose();
    super.dispose();
  }
}
