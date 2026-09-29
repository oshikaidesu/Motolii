import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show sans;
import '../../hf/bp/things.dart';
import '../../hf/metrics.dart';
import '../../hf/neutral.dart';
import 'catalog_session.dart';
import 'media_library.dart';

/// The catalog's controls over the Thumbnail view: SOURCES (which folders), TYPES (what), a search (which). Where, What
/// and Which are separate; the view (How) is the shelf's own masonry, unchanged. A skin over [CatalogSession].
class CatalogMedia extends StatelessWidget {
  const CatalogMedia({super.key, required this.session});
  final CatalogSession session;

  static const _types = [('All', null), ('▣', 'image'), ('▶', 'video'), ('♪', 'audio'), ('3D', 'model'), ('360°', 'environment')];

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: session,
        builder: (context, _) {
          final chosen = session.chosenSources;
          final items = {for (final e in session.entries) e.id: session.itemOf(e)};
          final things = [
            for (final e in session.entries)
              Thing.fromJson({'id': e.id, 'name': e.name, 'kind': 'item', 'family': 'catalog', 'source': 'catalog', 'face': const {'type': 'shelf'}}),
          ];
          return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _Row(label: 'Sources', children: [
              _Chip('All', chosen == null, () => session.choose(sources: () => null)),
              for (final s in session.sources)
                _Chip(s.name + (s.available ? '' : ' ·off'), chosen?.contains(s.id) ?? false, () => session.choose(sources: () => {s.id}), dim: !s.enabled || !s.available),
            ]),
            _Row(label: 'Types', children: [
              for (final (label, kind) in _types)
                _Chip(label, kind == null ? session.kinds.isEmpty : session.kinds.contains(kind), () => session.choose(kinds: kind == null ? {} : {kind})),
            ]),
            Container(
              height: UiMetrics.control + 4,
              margin: const EdgeInsets.fromLTRB(6, 3, 6, 3),
              padding: const EdgeInsets.symmetric(horizontal: 6),
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(3)),
              alignment: Alignment.centerLeft,
              child: _Search(session),
            ),
            Expanded(
              child: session.failure != null
                  ? Padding(padding: const EdgeInsets.all(9), child: Text('${session.failure}', style: Dn.label(N.g69)))
                  : session.entries.isEmpty
                      ? Padding(padding: const EdgeInsets.all(9), child: Text(session.sources.isEmpty ? 'Add a folder to see its media.' : 'Nothing matches.', style: Dn.label(N.g56)))
                      : LayoutBuilder(builder: (context, box) => MediaLibraryBody(sections: {'': things}, shown: things, items: items, width: box.maxWidth)),
            ),
          ]);
        },
      );
}

class _Row extends StatelessWidget {
  const _Row({required this.label, required this.children});
  final String label;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(6, 4, 6, 0),
        child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
          SizedBox(width: 44, child: Text(label.toUpperCase(), style: Dn.micro(N.g51))),
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
  Widget build(BuildContext context) => EditableText(
        controller: controller,
        focusNode: focus,
        style: Dn.name(N.g95),
        cursorColor: N.g82,
        backgroundCursorColor: N.g00,
        onChanged: (v) => widget.session.choose(text: v),
      );
}
