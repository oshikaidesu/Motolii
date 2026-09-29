import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show sans;
import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import 'catalog_session.dart';
import 'browser_item.dart';
import 'media_browser.dart';
import 'media_fluid.dart';
import 'media_preview.dart';
import 'project_source.dart';

/// The catalog's controls over the Thumbnail view: SOURCES (which folders), TYPES (what), a search (which). Where, What
/// and Which are separate; the view (How) is the shelf's own masonry, unchanged. A skin over [CatalogSession].
class CatalogMedia extends StatefulWidget {
  const CatalogMedia({super.key, required this.session, this.explore, this.initial = BrowserView.thumbnail, this.startOn, this.startOpen = false, this.exploreLayout, this.exploreRepaint, this.exploreNote, this.startColumn = 60, this.startProject = false});
  final CatalogSession session;
  final ExploreLayout? exploreLayout;
  final Listenable? exploreRepaint;
  final String? exploreNote;
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
  late final ProjectSource _project = ProjectSource(session.c, kinds: () => session.kinds, text: () => session.text);

  @override
  void dispose() {
    _project.dispose();
    super.dispose();
  }

  static const _types = [('All', null), ('▣', 'image'), ('▶', 'video'), ('♪', 'audio'), ('3D', 'model'), ('360°', 'environment')];

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([session, session.c.document]),
        builder: (context, _) {
          final chosen = session.chosenSources;
          return MediaBrowser(
            source: project ? _project : session,
            faces: _Faces(session),
            explore: widget.explore,
            exploreLayout: widget.exploreLayout,
            exploreRepaint: widget.exploreRepaint,
            exploreNote: widget.exploreNote,
            startColumn: widget.startColumn,
            onReveal: (item) {
              if (item.path.isNotEmpty) session.c.native('reveal', {'path': item.path});
            },
            onPlace: (item) {
              if (item.id.startsWith(ProjectSource.prefix)) {
                session.c.command('placeAsset', {'id': item.id.substring(ProjectSource.prefix.length)});
              } else if (session.c.supports('placeCatalogAsset')) {
                session.c.command('placeCatalogAsset', {'id': item.id});
              }
            },
            initial: widget.initial,
            startOn: widget.startOn,
            startOpen: widget.startOpen,
            sort: session.sort,
            descending: session.descending,
            onSort: (key) => session.choose(sort: key, descending: session.sort == key ? !session.descending : false),
            controls: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _Row(label: 'Sources', children: [
                _Chip('This project', project, () => setState(() => project = true)),
                _Chip('All', !project && chosen == null, () {
                  setState(() => project = false);
                  session.choose(sources: () => null);
                }),
                for (final s in session.sources)
                  _Chip(s.name + (s.available ? '' : ' ·off'), !project && (chosen?.contains(s.id) ?? false), () {
                    setState(() => project = false);
                    session.choose(sources: () => {s.id});
                  }, dim: !s.enabled || !s.available),
              ]),
              _Row(label: 'Types', children: [
                for (final (label, kind) in _types) _Chip(label, kind == null ? session.kinds.isEmpty : session.kinds.contains(kind), () => session.choose(kinds: kind == null ? {} : {kind})),
              ]),
              if (session.folderSource != null) _Folders(session),
              Container(
                height: UiMetrics.control + 4,
                margin: const EdgeInsets.fromLTRB(6, 3, 6, 3),
                padding: const EdgeInsets.symmetric(horizontal: 6),
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(3)),
                alignment: Alignment.centerLeft,
                child: _Search(session),
              ),
              if (session.failure != null) Padding(padding: const EdgeInsets.all(9), child: Text('${session.failure}', style: Dn.label(N.g69))),
            ]),
          );
        },
      );
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
        child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
          SizedBox(width: 54, child: Text(label.toUpperCase(), style: Dn.micro(N.g51))),
          Expanded(child: SizedBox(height: 20, child: ListView(scrollDirection: Axis.horizontal, children: children))),
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
          alignment: Alignment.center,
          decoration: BoxDecoration(color: on ? N.g20 : null, borderRadius: BorderRadius.circular(3)),
          child: Text(label, softWrap: false, style: sans(10.5, c: on ? N.g95 : (dim ? N.g44 : N.g63), w: on ? FontWeight.w600 : FontWeight.w500)),
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
