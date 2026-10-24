// A browser shelf as a pane of its own (Motolii's Browser: Media, Effects, Fonts, Colors, Create): header, search, body, footer.
// Each shelf is one tab of the window's panes; Media is the one file browser (project and disk, dock_files.dart).
import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_effects.dart';
import 'dock_files.dart';
import 'dock_fonts.dart';
import 'dock_glyphs.dart';
import 'dock_make.dart';
import 'dock_parts.dart';
import 'dock_tags.dart';
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
  // (what the shelf keeps of itself, its picks, folds and search: the window's view, so it is the same when opened again)
  DockMem get _mem => WsScope.read(context).view.putIfAbsent('dock.${widget.name}', DockMem.new) as DockMem;
  late final TextEditingController _search = TextEditingController(text: WsScope.read(context).viewOf('dock.${widget.name}.search', ''))
    ..addListener(() => setState(() => WsScope.read(context).view['dock.${widget.name}.search'] = _search.text));

  // (the shelf follows the document a picture after the hand: a change made, the inspector and the stage are built in
  //  that picture and the shelf in the next, once for all the changes since. It was half of each picture's work)
  Ws? _ws;
  bool _due = false;
  Object? _seen;
  // (a new reading of the document is looked at for what this shelf shows of it: a number a hand let go of changes
  //  none of the effects on, the motion or the things laid, so the shelf is not built again for it)
  int? _rev;
  Object? _shows;
  Object _showing(Ws ws) => switch (widget.name) {
    'Effects' => (ws.screenFx.join(' '), ws.motion, ws.fxOf(ws.selected).join(' ')),
    'Create' => ws.layers.map((l) => '${l.id} ${l.name}').join(' '),
    _ => ws.rev,
  };
  void _later() {
    final ws = _ws!;
    if (ws.rev != _rev) {
      _rev = ws.rev;
      final shows = _showing(ws);
      if (shows != _shows) {
        _shows = shows;
        _seen = null;
      }
    }
    final now = (_shows, ws.selected); // (what a shelf shows of the window: its part of the document, what is picked)
    if (_due || now == _seen) return;
    _seen = now;
    _due = true;
    SchedulerBinding.instance
      ..addPostFrameCallback((_) {
        _due = false;
        if (mounted) setState(() {}); // (built in the next picture, on its own)
      })
      ..ensureVisualUpdate();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final ws = WsScope.read(context);
    if (ws != _ws) {
      _ws?.removeListener(_later);
      _ws = ws..addListener(_later);
    }
  }

  @override
  void dispose() {
    _ws?.removeListener(_later);
    _search.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.read(context), name = widget.name;
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
              DockIconButton(DockG.folder, on: _mem.mediaDeep, onTap: () => setState(() => _mem.mediaDeep = !_mem.mediaDeep)),   // (the folders below listed with it)
              DockIconButton(DockG.files, on: _mem.treeSide != 0, onTap: () => setState(() => _mem.treeSide = (_mem.treeSide + 1) % 3)),   // (the tree: by the room, left, above)
            ] else
              ConstrainedBox(
                constraints: BoxConstraints(maxWidth: Shape.z(88)),
                child: ValueListenableBuilder<int>(
                  valueListenable: dockRead,
                  builder: (context, _, _) => Text(
                    switch (name) { // (what the inside has: the fonts it sets, the effects it does, the colours the document uses, the tiles that make)
                      'Fonts' => '${fontsOffered ?? '…'} fonts',
                      'Effects' => '${(ws.inside?.thingEffects.length ?? 0) + (ws.inside?.effects.length ?? 0)} ready',
                      'Colors' => '${usedColours(ws).values.toSet().length} used here',
                      'Create' => createFact(ws),
                      _ => _facts[name] ?? '',
                    },
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: T.label(Grey.g56),
                  ),
                ),
              ),
          ],
        ),
        DockSearch(
          controller: _search,
          hint: name == 'Fonts' ? 'Search fonts' : hint,
        ),
        Builder(
          builder: (context) {
            // (the word being typed, finished by the tags the shelf has: a press takes one)
            final done = DockTags.completing(name, _search.text);
            if (done.isEmpty) return const SizedBox.shrink();
            return Padding(
              padding: EdgeInsets.symmetric(horizontal: WsT.inset, vertical: Shape.s(3)),
              child: Wrap(
                spacing: Shape.s(4),
                runSpacing: Shape.s(3),
                children: [
                  for (final t in done)
                    DockHover(
                      onTap: () {
                        final text = DockTags.complete(_search.text, t);
                        _search.value = TextEditingValue(text: text, selection: TextSelection.collapsed(offset: text.length));
                      },
                      builder: (context, h) => Container(
                        padding: EdgeInsets.symmetric(horizontal: Shape.s(7), vertical: Shape.s(2)),
                        decoration: BoxDecoration(color: h ? WsT.raised : WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
                        child: Text(t, style: T.micro(Grey.g91)),
                      ),
                    ),
                ],
              ),
            );
          },
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
