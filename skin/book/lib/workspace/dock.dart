// A browser shelf as a pane of its own (Motolii's Browser: Media, Effects, Fonts, Colors, Create): header, search, body, footer.
// Each shelf is one tab of the window's panes; Media is the one file browser (project and disk, dock_files.dart).
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_effects.dart';
import 'dock_files.dart';
import 'dock_filters.dart';
import 'dock_fonts.dart';
import 'dock_glyphs.dart';
import 'dock_make.dart';
import 'dock_parts.dart';
import 'ws.dart';

/// The shelves: name (also the pane's id and [Ws.shelf]), the mark on its tab, the search hint.
const dockShelves = <(String, DockG, String)>[
  ('Media', DockG.media, 'Search files and media'),
  ('Effects', DockG.effects, 'Search effects and features'),
  ('Fonts', DockG.fonts, 'Search 15 families'),
  ('Colors', DockG.colors, 'Search palettes'),
  ('Create', DockG.create, 'Search layers and shapes'),
];

const _facts = {'Effects': '33 in library', 'Fonts': '15 families', 'Colors': '6 palettes', 'Create': '24 kinds'};

class WsShelf extends StatefulWidget {
  const WsShelf(this.name, {super.key});
  final String name;
  @override
  State<WsShelf> createState() => _WsShelfState();
}

class _WsShelfState extends State<WsShelf> {
  final _mem = DockMem();
  late final _search = TextEditingController()..addListener(() => setState(() {}));

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), name = widget.name;
    final (_, _, hint) = dockShelves.firstWhere((s) => s.$1 == name);
    final c = DockCtx(name, _mem, _search.text.trim(), ws, setState);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        WsHeader(
          name,
          trailing: [
            if (name == 'Media') ...[
              DockIconButton(DockG.grid, on: !_mem.mediaList, onTap: () => setState(() => _mem.mediaList = false)),
              DockIconButton(DockG.list, on: _mem.mediaList, onTap: () => setState(() => _mem.mediaList = true)),
            ] else
              ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 88),
                child: Text(_facts[name] ?? '', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g56)),
              ),
          ],
        ),
        DockSearch(
          controller: _search,
          hint: hint,
          trailing: name == 'Effects'
              ? DockFilterToggle(open: _mem.filtersOpen, active: fxActiveFilters(_mem), onTap: () => setState(() => _mem.filtersOpen = !_mem.filtersOpen))
              : null,
        ),
        Expanded(
          child: switch (name) {
            'Effects' => EffectsShelf(c),
            'Fonts' => FontsShelf(c),
            'Colors' => ColorsShelf(c),
            'Create' => CreateShelf(c),
            _ => MediaShelf(c),
          },
        ),
      ],
    );
  }
}
