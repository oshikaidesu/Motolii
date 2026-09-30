import 'dart:io';

import 'package:flutter/widgets.dart';

import '../parts.dart' show sans;
import '../../theme/metrics.dart';
import '../../theme/neutral.dart';
import 'catalog_session.dart';
import '../item.dart';
import '../user_state.dart';
import 'explore/graph.dart';
import 'media_browser.dart';
import 'fluid.dart';
import 'preview.dart';
import '../../session/editor_session.dart' show EditorSession;
import '../../session/media_actions.dart' show mediaAct, mediaActions, revealLabel;
import 'project_source.dart';

/// The catalog's controls over the Thumbnail view: SOURCES (which folders), TYPES (what), a search (which). Where, What
/// and Which are separate; the view (How) is the shelf's own masonry, unchanged. A skin over [CatalogSession].
class CatalogMedia extends StatefulWidget {
  const CatalogMedia({super.key, required this.session, this.explore, this.initial = BrowserView.thumbnail, this.startOn, this.startOpen = false, this.exploreLayout, this.exploreRepaint, this.exploreNote, this.exploreBar, this.exploreChoice, this.startColumn = 60, this.startProject = false});
  final CatalogSession session;
  final ExploreLayout? exploreLayout;
  final Listenable? exploreRepaint;
  final String? exploreNote;
  final Widget? exploreBar;

  /// Explore's own choices (global or local, the overlay, a first zoom) when the caller holds them (a story).
  final ExploreChoice? exploreChoice;
  final double startColumn;

  /// Starts on the work's own assets (a story, a restored choice).
  final bool startProject;
  final String? startOn;
  final bool startOpen;
  final ExploreBuilder? explore;
  final BrowserView initial;

  @override
  State<CatalogMedia> createState() => _CatalogMediaState();
}

class _CatalogMediaState extends State<CatalogMedia> {
  CatalogSession get session => widget.session;
  late bool project = widget.startProject;
  bool bundled = false;
  late final BundledSource _bundled = BundledSource(session.c, kinds: () => session.kinds, text: () => session.text);
  final _browser = GlobalKey<MediaBrowserState>();
  List<String> _seenImport = const [];

  /// What the person keeps and came back to, in the same user library the other Browsers use (saved with the settings, so it is
  /// still there when the app is opened again). An asset is named by its catalog id: it stays the same asset when its file is
  /// moved or renamed. Only catalog assets are kept: the work's own assets and the bundled ones belong to other owners.
  late final LiveBrowserUser _user = LiveBrowserUser(session.c, 'catalog');
  late final _Marked _marked = _Marked(session, _user, _hashes);

  /// Which kept set the result shows: '' (none), 'favorites' or 'recent'.
  String _keep = '';

  Set<String> _hashes() => {
        for (final a in EditorSession.maps(_c.state['assets']))
          if (a['contentHash'] != null) '${a['contentHash']}',
      };

  List<String> _keepIds() => _keep == 'recent' ? List.of(_user.views.recent) : _user.views.favorites.toList();

  Future<void> _showKept(String which) {
    setState(() {
      project = false;
      bundled = false;
      _keep = which;
    });
    return session.choose(sources: () => null, keep: which.isEmpty ? () => null : _keepIds, keepOrdered: which == 'recent');
  }

  void _userChanged() {
    if (_keep.isNotEmpty) session.choose(keep: _keepIds);
  }

  bool _catalog(BrowserItem i) => !i.id.startsWith(ProjectSource.prefix) && !i.id.startsWith(BundledSource.prefix);

  /// `F` or the menu: keep the picked catalog assets, or let them go when they are all kept already.
  void _favorite(List<BrowserItem> picked) {
    final ids = [for (final i in picked) if (_catalog(i)) i.id];
    if (ids.isEmpty) return;
    _user.collect(ids, ids.every(_user.views.favorites.contains) ? 0 : 1);
  }

  @override
  void initState() {
    super.initState();
    session.c.importedAssets.addListener(_imported);
    _user.addListener(_userChanged);
  }

  /// Something was just imported (a button, a drop): the work's own assets are shown and the new ones picked, as the old
  /// shelf did.
  void _imported() {
    final ids = session.c.importedAssets.value;
    if (ids.isEmpty || identical(ids, _seenImport)) return;
    _seenImport = ids;
    setState(() => project = true);
    WidgetsBinding.instance.addPostFrameCallback((_) => _browser.currentState?.pick([for (final id in ids) '${ProjectSource.prefix}$id']));
  }

  EditorSession get _c => session.c;

  late final ProjectSource _project = ProjectSource(session.c, kinds: () => session.kinds, text: () => session.text);

  /// Explore is the sparse nearest-neighbour map over what the Browser shows; what the work holds (by content hash) is one of the
  /// cheap things that make two assets near. A caller (a story) may bring its own.
  late final _explore = widget.exploreChoice ?? ExploreChoice();
  late final ExploreLayout _layout = exploreLayout(_explore, () => {
        for (final a in EditorSession.maps(_c.state['assets']))
          if (a['contentHash'] != null) '${a['contentHash']}',
      });

  @override
  void dispose() {
    session.c.importedAssets.removeListener(_imported);
    _user.removeListener(_userChanged);
    _user.dispose();
    _marked.dispose();
    _project.dispose();
    _bundled.dispose();
    if (widget.exploreChoice == null) _explore.dispose();
    super.dispose();
  }

  static const _types = [('All', null), ('▣', 'image'), ('▶', 'video'), ('♪', 'audio'), ('3D', 'model'), ('360°', 'environment')];

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([session, _user, session.c.slice('mediaAssets', const ['assets', 'backgrounds'])]),
        builder: (context, _) {
          final chosen = session.chosenSources;
          return Stack(fit: StackFit.passthrough, children: [
            MediaBrowser(
            key: _browser,
            source: project ? _project : (bundled ? _bundled : _marked),
            faces: _Faces(session),
            explore: widget.explore,
            exploreLayout: widget.exploreLayout ?? _layout,
            exploreRepaint: widget.exploreRepaint ?? _explore,
            exploreNote: widget.exploreNote,
            exploreBar: widget.exploreBar ?? (widget.exploreLayout == null ? ExploreBar(choice: _explore) : null),
            startColumn: widget.startColumn,
            onReveal: (item) {
              if (item.path.isNotEmpty) session.c.native('reveal', {'path': item.path});
            },
            menuOf: _menuOf,
            onMenu: _onMenu,
            onRemove: _onRemove,
            onFavorite: _favorite,
            onSeen: (item) {
              if (_catalog(item)) _user.used(item.id);
            },
            carry: _carry,
            onPlace: _place,
            initial: widget.initial,
            startOn: widget.startOn,
            startOpen: widget.startOpen,
            sort: session.sort,
            descending: session.descending,
            onSort: (key) => session.choose(sort: key, descending: session.sort == key ? !session.descending : false),
            controls: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _Row(label: 'Sources', children: [
                _Chip('This project', project, () => setState(() {
                  project = true;
                  bundled = false;
                })),
                _Chip('Bundled', bundled, () => setState(() {
                  bundled = true;
                  project = false;
                })),
                _Chip('All', !project && !bundled && _keep.isEmpty && chosen == null, () {
                  setState(() {
                    project = false;
                    bundled = false;
                    _keep = '';
                  });
                  session.choose(sources: () => null, keep: () => null, keepOrdered: false);
                }),
                _Chip('★ Favorites', !project && !bundled && _keep == 'favorites', () => _showKept('favorites'), dim: _user.views.favorites.isEmpty),
                _Chip('Recent', !project && !bundled && _keep == 'recent', () => _showKept('recent'), dim: _user.views.recent.isEmpty),
                for (final s in session.sources)
                  _Chip(s.name + (s.available ? '' : ' ·off'), !project && !bundled && _keep.isEmpty && (chosen?.contains(s.id) ?? false), () {
                    setState(() {
                      project = false;
                      bundled = false;
                      _keep = '';
                    });
                    session.choose(sources: () => {s.id}, keep: () => null, keepOrdered: false);
                  }, dim: !s.enabled || !s.available),
                _Chip('＋ Folder…', false, _addFolder),
                if (session.c.supports('import')) _Chip('＋ Import…', false, () => session.c.importFiles()),
              ]),
              _Row(label: 'Types', children: [
                for (final (label, kind) in _types) _Chip(label, kind == null ? session.kinds.isEmpty : session.kinds.contains(kind), () => session.choose(kinds: kind == null ? {} : {kind})),
              ]),
              if (session.folderSource != null) _Folders(session),
              Container(
                height: Surface.control + 4,
                margin: const EdgeInsets.fromLTRB(6, 3, 6, 3),
                padding: const EdgeInsets.symmetric(horizontal: 6),
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(Surface.controlRadius)),
                alignment: Alignment.centerLeft,
                child: _Search(session),
              ),
              if (session.failure != null) Padding(padding: const EdgeInsets.all(9), child: Text('${session.failure}', style: Dn.label(N.g69))),
            ]),
            ),
            // while files are carried over the window the Browser says where they go; it takes no pointer
            Positioned.fill(
              child: IgnorePointer(
                child: ListenableBuilder(
                  listenable: session.c.dragging,
                  builder: (context, _) => AnimatedOpacity(
                    opacity: session.c.dragging.value ? 1 : 0,
                    duration: const Duration(milliseconds: 120),
                    child: Container(
                      key: const ValueKey('media-drop-hint'),
                      margin: const EdgeInsets.all(6),
                      alignment: Alignment.center,
                      decoration: BoxDecoration(color: N.veil, border: Border.all(color: N.g95, width: 1.4), borderRadius: BorderRadius.circular(Surface.faceRadius)),
                      child: Text('Drop to import', style: sans(Dn.nameSize, c: N.g95)),
                    ),
                  ),
                ),
              ),
            ),
          ]);
        },
      );

  // ---- what the person can do to what they picked: the old Media shelf's own operations, over both kinds of item --------

  /// A folder becomes a Source (the catalog indexes it and watches it; nothing in it is ever written).
  Future<void> _addFolder() async {
    final picked = await _c.native('pickImport', {});
    if (picked is! List) return;
    for (final path in picked.whereType<String>()) {
      if (Directory(path).existsSync()) await session.addSource(path);
    }
  }

  bool _mine(BrowserItem i) => i.id.startsWith(ProjectSource.prefix);

  void _place(BrowserItem item) {
    if (item.id.startsWith(BundledSource.prefix)) {
      _c.command('create', {'kind': 'background:${item.id.substring(BundledSource.prefix.length)}'});
    } else if (_mine(item)) {
      _c.command('placeAsset', {'id': item.id.substring(ProjectSource.prefix.length)});
    } else if (_c.supports('placeCatalogAsset')) {
      _c.command('placeCatalogAsset', {'id': item.id});
      _user.used(item.id); // the way back to what was used
    }
  }

  /// A work's asset carries the document's own menu; a catalog asset only what does nothing to any file.
  List<({String value, String label, bool enabled})> _menuOf(BrowserItem item, List<BrowserItem> picked) {
    if (item.id.startsWith(BundledSource.prefix)) return [(value: 'place', label: 'Place', enabled: _c.supports('create'))];
    final raw = _project.raw(item.id);
    final place = (value: 'place', label: 'Place', enabled: !item.missing && _c.supports(raw != null ? 'placeAsset' : 'placeCatalogAsset'));
    if (raw != null) return [place, ...mediaActions(_c, raw)];
    final has = item.path.isNotEmpty && !item.missing;
    return [
      place,
      (value: 'favorite', label: _user.views.favorites.contains(item.id) ? 'Remove from Favorites' : 'Add to Favorites', enabled: true),
      if (has) (value: 'reveal', label: revealLabel, enabled: true),
      if (has) (value: 'open', label: 'Open with default app', enabled: true),
      if (has && item.mime.startsWith('image/')) (value: 'palette', label: 'Extract palette', enabled: true),
      if (item.path.isNotEmpty) (value: 'copyPath', label: 'Copy path', enabled: true),
    ];
  }

  void _onMenu(String action, BrowserItem item, List<BrowserItem> picked) {
    if (action == 'place') return _place(item);
    if (action == 'favorite') return _favorite(picked.isEmpty ? [item] : picked);
    final raw = _project.raw(item.id);
    mediaAct(_c, action, raw ?? {'path': item.path, 'mime': item.mime, 'name': item.name}, id: raw?['id']);
  }

  /// Delete over the pick removes the work's own unused assets from its library (the host keeps the used ones). A catalog
  /// asset has nothing to remove here, and no source file is ever deleted by the Browser.
  void _onRemove(List<BrowserItem> picked) {
    final ids = [for (final i in picked) if (_project.raw(i.id) case final raw?) raw['id']];
    if (ids.isNotEmpty && _c.supports('removeAsset')) _c.command('removeAsset', {'ids': ids});
  }

  Map<String, dynamic>? _carry(BrowserItem item) {
    if (item.missing || item.id.startsWith(BundledSource.prefix)) return null;
    final raw = _project.raw(item.id);
    if (raw != null) return _c.supports('placeAsset') ? {'asset': '${raw['id']}', 'name': item.name} : null;
    return _c.supports('placeCatalogAsset') ? {'asset': item.id, 'name': item.name, 'catalog': true} : null;
  }
}

/// The preview's questions, answered by the owner through the session.
class _Faces implements FaceService {
  _Faces(this.session);
  final CatalogSession session;
  @override
  Future<String?> frameAt(BrowserItem item, double seconds, {int edge = 480}) async => item.id.startsWith(ProjectSource.prefix) ? null : session.frameAt(item.id, seconds, edge: edge);
  @override
  Future<String?> pictureOf(BrowserItem item) async => item.id.startsWith(ProjectSource.prefix) ? null : session.pictureOf(item.id);
}

class _Row extends StatelessWidget {
  const _Row({required this.label, required this.children});
  final String label;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(6, 4, 6, 0),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          SizedBox(width: 54, height: 20, child: Align(alignment: Alignment.centerLeft, child: Text(label.toUpperCase(), style: Dn.micro(N.g51)))),
          Expanded(child: Wrap(spacing: 0, runSpacing: 3, children: [for (final c in children) SizedBox(height: 20, child: c)])),
        ]),
      );
}

class _Chip extends StatelessWidget {
  const _Chip(this.label, this.on, this.tap, {this.dim = false});
  final String label;
  final bool on, dim;
  final VoidCallback tap;
  @override
  Widget build(BuildContext context) => GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: tap,
        child: Container(
          margin: const EdgeInsets.only(right: 3),
          padding: const EdgeInsets.symmetric(horizontal: 6),
          decoration: BoxDecoration(color: on ? N.g20 : null, borderRadius: BorderRadius.circular(Surface.controlRadius)),
          child: Center(widthFactor: 1, child: Text(label, softWrap: false, style: sans(10.5, c: on ? N.g95 : (dim ? N.g44 : N.g63), w: on ? FontWeight.w600 : FontWeight.w500))),
        ),
      );
}

class _Search extends StatefulWidget {
  const _Search(this.session);
  final CatalogSession session;
  @override
  State<_Search> createState() => _SearchState();
}

class _SearchState extends State<_Search> {
  final controller = TextEditingController();
  final focus = FocusNode();
  @override
  void dispose() {
    controller.dispose();
    focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Stack(alignment: Alignment.centerLeft, children: [
        ListenableBuilder(listenable: controller, builder: (_, __) => controller.text.isEmpty ? Text('Search assets', style: Dn.label(N.g44)) : const SizedBox.shrink()),
        _field(),
      ]);

  Widget _field() => EditableText(
        controller: controller,
        focusNode: focus,
        style: Dn.name(N.g95),
        cursorColor: N.g82,
        backgroundCursorColor: N.g00,
        onChanged: (v) => widget.session.choose(text: v),
      );
}

/// Folder browsing: where in the source the result set is taken from. A crumb goes up; a folder goes in. It is one more
/// way to make the result set (Where), not another browser: the views below read the same set.
class _Folders extends StatelessWidget {
  const _Folders(this.session);
  final CatalogSession session;
  @override
  Widget build(BuildContext context) {
    final source = session.folderSource!;
    final path = (session.folder?.prefix ?? '').split('/').where((p) => p.isNotEmpty).toList();
    void go(List<String> parts) => session.choose(folder: () => (source: source, prefix: parts.join('/')));
    return _Row(label: 'Folder', children: [
      _Chip('/', path.isEmpty, () => go(const [])),
      for (final (i, part) in path.indexed) _Chip('/ $part', i == path.length - 1, () => go(path.sublist(0, i + 1))),
      for (final f in session.subfolders) _Chip('${f.name} ${f.assets}', false, () => go([...path, f.name]), dim: true),
    ]);
  }
}

/// The catalog's result set with what the work and the person's library say about each asset: the work holds it (by content
/// hash) and the person keeps it (Favorites). The asset is the same; only its marks differ.
class _Marked extends ChangeNotifier implements ResultSource {
  _Marked(this.session, this.user, this.hashes) {
    session.addListener(notifyListeners);
    user.addListener(notifyListeners);
  }
  final CatalogSession session;
  final LiveBrowserUser user;
  final Set<String> Function() hashes;

  @override
  List<BrowserItem> get items {
    final held = hashes(), kept = user.views.favorites;
    return [for (final i in session.items) i.marked(used: inWork(i, held), favorite: kept.contains(i.id))];
  }

  @override
  void dispose() {
    session.removeListener(notifyListeners);
    user.removeListener(notifyListeners);
    super.dispose();
  }
}
