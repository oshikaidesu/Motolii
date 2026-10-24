// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

Widget _pHoverRows() {
  final vid = _by('lens_bloom.mp4'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 50);
  return _Parts('B5-b / B6', 'Hover and scrub: rows, tiles, big clips', 'A row is 50 px: the thumbnail (76 wide, 60 in a dock) is the hover target; the name is what selects.', [
    ('Rows', [
      row('rest', _HoverRow(vid, tw: 76)),
      row('thumb hover at 40 %', _HoverRow(vid, tw: 76, holdHover: true, holdX: 30)),
      row('selected', _HoverRow(vid, tw: 76, selected: true)),
      row('long name: ellipsis', _HoverRow(long, tw: 76)),
      row('preset', _HoverRow(_by('Glow'), tw: 76)),
      row('narrow 232, hover', _HoverRow(vid, tw: 60, holdHover: true, holdX: 30), width: 232),
    ]),
    ('Tile captions (146)', [
      _PC('tile: rest', 146, _HoverTile(vid, w: 146), pad: 8),
      _PC('tile: selected: name g95', 146, _HoverTile(vid, w: 146, selected: true), pad: 8),
      _PC('tile: preset', 146, _HoverTile(_by('Glow'), w: 146), pad: 8),
      _PC('tile: long name', 146, _HoverTile(long, w: 146), pad: 8),
    ]),
    ('B6 big clip (300 x 150) and its caption line', [
      _PC('poster frame at rest', 300, _HoverThumb(vid, w: 300, h: 150), h: 150, pad: 8),
      _PC('scrub at 40 %', 300, _HoverThumb(vid, w: 300, h: 150, holdHover: true, holdX: 120), h: 150, pad: 8),
      _PC('caption: name and size (the length is on the picture)', 300, Row(children: [Expanded(child: Text(vid.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name())), Text('1920×1080', style: _num(N.g76))]), pad: 8),
    ]),
  ]);
}

// ---- B5-c Own frame
Widget _pOwn() {
  Widget frame(String label, {bool pv = false, String? applied, bool noTarget = false, double w = 300}) => _OwnFrameView(w: w, h: w * 9 / 16, label: label, previewing: pv, applied: applied, noTarget: noTarget, picture: _Fx(pv ? Fx.glow : (applied != null ? Fx.glow : Fx.none), .5, base: _ownBase(sample: noTarget)));
  return _Parts('B5-c', 'Own frame: overlays, banner, counters', 'The frame has a 1 px edge: g20, the mode colour only while previewing. Labels are 16 px, 6 px in; the right one gives way before it touches the left.', [
    ('Frame, 300 x 169', [
      _PC('nothing tried yet', 300, frame('Layer 3 · Title'), pad: 8),
      _PC('previewing: mode edge + PREVIEW label', 300, frame('Layer 3 · Title', pv: true), pad: 8),
      _PC('applied: APPLIED label + name', 300, frame('Layer 3 · Title', applied: 'Glow'), pad: 8),
      _PC('no selection: sample frame', 300, frame('Sample frame (no selection)', noTarget: true), pad: 8),
      _PC('narrow 212: labels ellipsize', 212, frame('Layer 3 · Title', pv: true, w: 212), pad: 8),
      _PC('applied, long name, 212', 212, frame('Layer 3 · Title', applied: 'Directional Blur (Motion Trail, 8-sample, GPU)', w: 212), pad: 8),
    ]),
    ('Phase banner, 300 x 28', [
      _PC('PREVIEW: dashed g56 edge', 300, const _PhaseBanner(_Phase.preview), h: 28, pad: 8),
      _PC('APPLIED: g20 ground + tick', 300, const _PhaseBanner(_Phase.applied), h: 28, pad: 8),
      _PC('DROPPED: g63 words', 300, const _PhaseBanner(_Phase.dropped), h: 28, pad: 8),
      _PC('PREVIEW, narrow 212', 212, const _PhaseBanner(_Phase.preview), h: 28, pad: 8),
    ]),
    ('Counters', [
      _PC('nothing yet', 300, const _Counters(0, 0, 'Down'), pad: 8),
      _PC('tried 3, hover trigger', 300, const _Counters(3, 0, ''), pad: 8),
      _PC('previewing: Enter and Esc key marks', 300, const _Counters(3, 1, null), pad: 8),
      _PC('written > 0: g95', 300, const _Counters(5, 2, 'Down'), pad: 8),
    ]),
  ]);
}

// ---- B5-d Preview surface
Widget _pSurface() {
  Widget surf(String name, {bool sample = false, bool playing = true, bool hide = false, double w = 300}) => _SurfaceFrame(w: w, h: w * 9 / 16, sample: sample, playing: playing, name: name, hoverHide: hide ? true : null, picture: _Fx(Fx.glow, .5, base: _ownBase(sample: sample)));
  return _Parts('B5-d', 'Preview surface: overlays and closed bar', 'The surface is 16:9 with a 1 px g20 edge. Top: whose frame, and Hide. Bottom: Playing or Paused, and the item name.', [
    ('Surface, 300 x 169', [
      _PC('your frame, playing', 300, surf('Glow'), pad: 8),
      _PC('your frame, paused', 300, surf('Glow', playing: false), pad: 8),
      _PC('sample frame', 300, surf('Glow', sample: true), pad: 8),
      _PC('Hide hovered: g26', 300, surf('Glow', hide: true), pad: 8),
      _PC('long item name', 300, surf('Directional Blur (Motion Trail, 8-sample, GPU)'), pad: 8),
      _PC('narrow 212, long name', 212, surf('Directional Blur (Motion Trail, 8-sample, GPU)', w: 212), pad: 8),
    ]),
    ('Closed bar, position line', [
      _PC('closed bar: rest', 300, const _ClosedBar(), h: 26, pad: 8),
      _PC('closed bar: Show hover', 300, const _ClosedBar(hoverShow: true), h: 26, pad: 8),
      _PC('closed bar, narrow 212: line clips, Show stays', 212, const _ClosedBar(), h: 26, pad: 8),
      _PC('position line', 300, Text('15 of 64', style: _num(N.g76)), pad: 8),
    ]),
  ]);
}

// ---- B10 star, used, collections
Widget _pStar() {
  final g = _by('Glow'), t = _by('Title In: Soft Rise');
  _PC row(String cap, Widget w, {double width = 320, double h = 34}) => _PC(cap, width, w, h: h);
  return _Parts('B10-b', 'Star and recent: rows and headings', 'The star is a 24 px target at the right end; the quiet word before it (when, how often) is 10 px g63. Star hover is the live use case only (its own part owns it).', [
    ('Rows', [
      row('starred', _ItemRow(g, right: _StarTail(null, on: true))),
      row('not starred', _ItemRow(g, right: _StarTail(null, on: false))),
      row('recent: time + star', _ItemRow(t, right: _StarTail(_ago(40), on: false))),
      row('recent: just now', _ItemRow(g, right: _StarTail('just now', on: true))),
      row('frequent: count', _ItemRow(g, right: _StarTail('41×', on: true))),
      row('selected', _ItemRow(t, selected: true, right: _StarTail(_ago(40), on: true))),
      row('hover', _ItemRow(t, hover: true, right: _StarTail(_ago(40), on: false))),
      row('long name, narrow 232', _ItemRow(_by('Directional Blur (Motion Trail, 8-sample, GPU)'), right: _StarTail('6×', on: false)), width: 232),
    ]),
    ('Headings and first-run hints', [
      row('Starred + count', _Sec('Starred', right: Text('4', style: T.label(N.g76))), h: 26),
      row('Recent + automatic', _Sec('Recent', right: Text('automatic', style: T.label(N.g76))), h: 26),
      row('first run: Starred', const _HintRow('No stars yet'), h: 36),
      row('first run: Recent', const _HintRow('None yet'), h: 36),
      row('first run: Frequent', const _HintRow('None yet'), h: 36),
    ]),
  ]);
}

Widget _pUsed() {
  final g = _by('Glow'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 30);
  return _Parts('B10-c', 'Used stack: rows and heading', 'A shelf row is 30 px: thumbnail 26 x 18, name, and x count when the same thing was folded in. A row just used glows g26 and fades in 600 ms.', [
    ('Rows', [
      row('single use', _UsedRow(g, 1)),
      row('folded: x3', _UsedRow(g, 3)),
      row('just used: g26 at full', _UsedRow(g, 2, flash: 1)),
      row('fading: halfway', _UsedRow(g, 2, flash: .5)),
      row('long name, count kept', _UsedRow(long, 12)),
      row('narrow 232', _UsedRow(long, 2), width: 232),
    ]),
    ('Heading and empty', [
      _PC('Used: 6 of 12', 320, _Sec('Used', right: Text('6 of 12', style: T.label(N.g76))), h: 24),
      _PC('Used: 12 of 12 (at the cap)', 320, _Sec('Used', right: Text('12 of 12', style: T.label(N.g76))), h: 24),
      _PC('empty shelf', 320, const _HintRow('Nothing used yet'), h: 40),
      _PC('status: folded', 320, const _Status('“Glow” ×4'), h: 26),
      _PC('status: full', 320, const _Status('Full: dropped “Typewriter”'), h: 26),
    ]),
  ]);
}

Widget _pCollections() {
  final g = _by('Glow');
  _PC slot(String cap, Widget w, {double width = 158}) => _PC(cap, width, w, h: 24);
  return _Parts('B10-a', 'Collections: slots and membership', 'A slot is 24 px: the tick in the gutter of its cell, then (x 10) its number as a key mark, its name, how many it holds. Two across in a 320 panel (158 each), one in a 232 dock.', [
    ('Slots, 158 wide', [
      slot('rest', const _SlotRow(1, 'Titles', 3)),
      slot('hover: g15', const _SlotRow(1, 'Titles', 3, hover: true)),
      slot('selected: g20, tick, lit key', const _SlotRow(1, 'Titles', 3, on: true)),
      slot('empty slot: dashed outline', const _SlotRow(6, '', 0)),
      slot('long name: ellipsis', const _SlotRow(5, 'Soft blurs for client A', 12)),
      slot('count 0', const _SlotRow(4, 'Retro', 0)),
      slot('count 41', const _SlotRow(2, 'Glows', 41)),
    ]),
    ('Slots, 232 wide (dock)', [
      slot('rest', const _SlotRow(3, 'Client A', 2), width: 232),
      slot('selected', const _SlotRow(3, 'Client A', 2, on: true), width: 232),
      slot('empty', const _SlotRow(7, '', 0), width: 232),
    ]),
    ('Heading, band, membership column', [
      _PC('Collections', 320, const _Sec('Collections'), h: 24),
      _PC('band: count only (the slot above is lit)', 320, _Band(filters: 1, shown: 4, total: 64, showFilters: false, onClear: () {}), h: 28),
      _PC('row: also in 1 more (number plate as on the slot)', 320, _ItemRow(g, right: const _SlotBadges([2, 3])), h: 34),
      _PC('row: also in 3', 320, _ItemRow(g, right: const _SlotBadges([2, 3, 5])), h: 34),
      _PC('row: more than 3', 320, _ItemRow(g, right: const _SlotBadges([1, 2, 3, 5])), h: 34),
      _PC('empty collection', 320, _Empty('Retro is empty', const [], actions: [_Btn('Show everything', onTap: () {})]), h: 170),
    ]),
  ]);
}

// ---- B12-b Similar
Widget _pSimilar() {
  final a = sampleAssets[0], v = sampleAssets[1];
  _PC tile(String cap, Widget t, {double w = 96}) => _PC(cap, w, t, pad: 8);
  return _Parts('B12-b', 'Similar: tile, chip, band', 'A tile is a 1 px picture edge (g20, g56 hover, mode selected), the name, and a 2 px closeness bar with its number in mono 10 px.', [
    ('Tile, three across in a 320 panel (96 wide)', [
      tile('all view: no bar', _SimTile(a, w: 96)),
      tile('rest with closeness', _SimTile(v, w: 96, score: .82)),
      tile('hover: Similar label', _SimTile(v, w: 96, score: .82, hover: true)),
      tile('hover, label hovered (g26)', _SimTile(v, w: 96, score: .82, hover: true, simHover: true)),
      tile('selected: mode edge', _SimTile(v, w: 96, score: .82, selected: true)),
      tile('source: first, 1.00', _SimTile(a, w: 96, score: 1, selected: true, source: true)),
      tile('low closeness', _SimTile(v, w: 96, score: .12)),
    ]),
    ('Tile, two across in a 232 dock (103 wide)', [
      tile('rest', _SimTile(v, w: 103, score: .64), w: 103),
      tile('hover', _SimTile(v, w: 103, score: .64, hover: true), w: 103),
    ]),
    ('Chip, band, tabs', [
      _PC('filter chip', 200, _left(const _FilterChip('similar', 'dusk_ridge.jpg')), h: 20, pad: 8),
      _PC('filter chip: cross hover', 200, _left(const _FilterChip('similar', 'dusk_ridge.jpg', hoverCross: true)), h: 20, pad: 8),
      _PC('filter chip: long name, 212', 212, _left(const _FilterChip('similar', 'dusk_ridge_golden_hour_v2_final.jpg')), h: 20, pad: 8),
      _PC('band: by similarity (the chip above says what from)', 320, _Band(filters: 1, shown: 38, total: 38, showFilters: false, showCount: false, extra: 'by similarity', onClear: () {}), h: 28),
      _PC('type tabs', 300, _Tabs(const ['All', 'Image', 'Video', 'Audio', '3D'], 0, (_) {}, toggle: false), h: 26, pad: 8),
      _PC('status', 320, const _Status('', keys: ['Y', 'Esc']), h: 26),
    ]),
  ]);
}

// ================================================================================ the sets

String _s(BuildContext c, String label, List<String> opts, String init) => c.knobs.object.dropdown<String>(label: label, options: opts, initialOption: init, labelBuilder: (s) => s.isEmpty ? '(empty)' : s);

/// One component per problem id, in the order to release them (simplest and most useful first). Every component opens with its Parts @200% sheet(s), then the whole-panel drafts.
List<WidgetbookComponent> panelBrowserASets() => [
      WidgetbookComponent(name: 'Browser B3 Find', useCases: [
        _parts('Find chrome', _pFindChrome),
        _parts('Find rows', _pRows),
        _uc('B3-a Find', 'unified query', Hab.habit, 'Field is always visible / each letter updates the list / several words = AND / Esc clears the text, a second Esc leaves the field.', (c, w) {
                final q = _s(c, 'Query', const ['', 'bl', 'glow soft', 'グロー', 'dir', 'zzz'], 'bl');
                return _FindA(key: ValueKey('a1$q$w'), query: q);
              }),
        _uc('B3-a Find', 'tab on', Hab.habit, 'One place chosen: its tab is the g20 pill with g95 text and the accent line / the count line names the place / results that exist elsewhere are cued in the footer / the tab again clears it.', (c, w) {
                final tab = _s(c, 'Place', const [..._places], 'Effects');
                return _FindA(key: ValueKey('a5$tab$w'), query: 'bl', tab: _places.indexOf(tab));
              }),
        _uc('B3-a Find', 'clear mark hovered', Hab.habit, 'With text in the field the x is the clear mark: g63 at rest, g95 under the pointer (pinned here); one click empties the field and keeps focus.', (c, w) => _FindA(key: ValueKey('a6$w'), query: 'bl', clearHover: true)),
        _uc('B3-a Find', 'more results exist', Hab.habit, 'With a place chosen, results in the other places are not hidden silently: the footer says how many more there are.', (c, w) => _FindA(key: ValueKey('a7$w'), query: 'soft', tab: 1)),
        _uc('B3-a Find', 'across places (D-4)', Hab.habit, 'One search spans all places and each result carries its place / choosing a place narrows it / the typed text stays when the place changes.', (c, w) {
                final tab = _s(c, 'Place', const ['All', ..._places], 'All');
                return _FindA(key: ValueKey('a2$tab$w'), query: 'soft', grouped: true, tab: tab == 'All' ? -1 : _places.indexOf(tab));
              }),
        _uc('B3-a Find', 'narrow dock, long names', Hab.habit, 'Same contract in a 232 px dock: names ellipsize, nothing is clipped mid-word, the field never shrinks.', (c, w) => _FindA(key: ValueKey('a3$w'), query: 'blur'), narrow: true),
        _uc('B3-a Find', 'no match, next step', Hab.habit, 'An empty result says what was searched and offers a way out: clear, widen to all places, or a spelling suggestion.', (c, w) {
                final q = _s(c, 'Query', const ['glwo', 'zzzz', 'blu'], 'glwo');
                final tab = _s(c, 'Place', const ['All', ..._places], 'Presets');
                return _FindA(key: ValueKey('a4$q$tab$w'), query: q, tab: tab == 'All' ? -1 : _places.indexOf(tab));
              }),
      ]),
      WidgetbookComponent(name: 'Browser B2 Tag bands', useCases: [
        _parts('Chips and groups', _pTags),
        _uc('B2-a Tag bands', 'Ableton type', Hab.habit, 'Groups cross as AND / the band counts the filters on / Clear removes all. The click rule is said once, in the status line under the panel.', (c, w) {
                final add = c.knobs.boolean(label: 'Add mode (as if Cmd is held)', initialValue: false);
                return _TagBands(key: ValueKey('e1$add$w'), add: add, start: const {'Kind': ['Effect'], 'Character': ['Soft']});
              }),
        _uc('B2-a Tag bands', 'groups fold (narrow dock)', Hab.habit, 'A group with nothing chosen folds to one line and a group with a choice shows it even when folded, so a filter is never hidden.', (c, w) => _TagBands(key: ValueKey('e2$w'), fold: true, start: const {'Category': ['Blur'], 'Character': ['Soft']}), narrow: true),
        _uc('B2-a Tag bands', 'zero results', Hab.habit, 'Narrowing never widens, so zero is reachable: the panel names what to remove and how many that returns, one click each.', (c, w) => _TagBands(key: ValueKey('e3$w'), start: const {'Kind': ['Media'], 'Category': ['Blur'], 'Character': ['Soft']})),
      ]),
      WidgetbookComponent(name: 'Browser B4 Results band', useCases: [
        _parts('Band states', _pBand),
        _uc('B4 Results band', 'n filters and Clear', Hab.addition, 'The band counts every filter that is on, search text included / zero says 0 and the way out / Clear is one click and dim when nothing is on.', (c, w) {
                final n = c.knobs.int.slider(label: 'Filters on', initialValue: 3, min: 0, max: 4);
                final t = _s(c, 'Search text', const ['', 'glow', 'blur', 'zzz'], '');
                return _BandDemo(key: ValueKey('f1$n$t$w'), n: n, text: t);
              }),
        _uc('B4 Results band', 'Clear and x hovered', Hab.addition, 'Clear is a button: g76 word, g26 hairline, g20 fill under the pointer (pinned here) / the search box x brightens to g95 / the footer cues the results the filters hide.', (c, w) => _BandDemo(key: ValueKey('f2$w'), n: 3, text: 'glow', hoverClear: true, hoverX: true)),
        _uc('B4 Results band', 'no filter, Clear dimmed', Hab.addition, 'With nothing on, Clear is dimmed (g63, no hover, arrow cursor) and the footer says end of results.', (c, w) => _BandDemo(key: ValueKey('f3$w'), n: 0, text: '', hoverClear: true)),
      ]),
      WidgetbookComponent(name: 'Browser B3 Tab menu', useCases: [
        _parts('Menu pieces', _pMenu),
        _uc('B3-b Tab menu', 'type-ahead at the cursor', Hab.addition, 'Tab opens a small search where the pointer is / Enter applies the first (or selected) to the selection / Esc writes nothing / Tab again cycles the family.', (c, w) {
                final t = _s(c, 'Target', const ['Layer', 'None', 'Key'], 'Layer');
                return _TabMenu(key: ValueKey('b1$t$w'), w: w, target: const ['Layer', 'None', 'Key'].indexOf(t), text: 'gl');
              }),
        _uc('B3-b Tab menu', 'empty input = last used first', Hab.addition, 'With nothing typed, the most recently used effect is on top, so Tab then Enter repeats it / Tab again moves to the next family.', (c, w) => _TabMenu(key: ValueKey('b2$w'), w: w)),
        _uc('B3-b Tab menu', 'no target selected', Hab.addition, 'Without a layer or key selected, Enter writes nothing and the menu says why in a line, not by going dim.', (c, w) => _TabMenu(key: ValueKey('b3$w'), w: w, target: 1, text: 'blur')),
        _uc('B3-b Tab menu', 'key selected: only curves', Hab.addition, 'The candidates follow the selection: with a key selected the list holds only what a key can take (ease presets, curves).', (c, w) => _TabMenu(key: ValueKey('b4$w'), w: w, target: 2)),
      ]),
      WidgetbookComponent(name: 'Browser B3 Prefix', useCases: [
        _parts('Rows, area line, marks', _pPrefix),
        _uc('B3-c Prefix', 'marks pick the area', Hab.addition, 'The first mark changes the area and its heading / no mark = every area under its own heading / the marks are listed under the field.', (c, w) {
                final q = _s(c, 'Query', const ['', 'lo', '#so', '>re', '@ti', '#', '@zz'], 'lo');
                return _Prefix(key: ValueKey('c1$q$w'), query: q);
              }),
        _uc('B3-c Prefix', 'discovery: empty field', Hab.addition, 'An empty field teaches the three marks (tap one to try it); the plain list stays below, so a newcomer who never types a mark loses nothing.', (c, w) => _Prefix(key: ValueKey('c2$w'))),
      ]),
      WidgetbookComponent(name: 'Browser B3 Empty query', useCases: [
        _parts('Headings, hints, reasons', _pFreq),
        _uc('B3-d Empty query', 'recent, then what fits', Hab.addition, 'Empty field = Recent, then what fits the selection, then the rest / each row says why it is there / one letter turns it into normal search.', (c, w) => _Freq(key: ValueKey('d1$w'), sel: 1, history: true)),
        _uc('B3-d Empty query', 'nothing selected', Hab.addition, 'With no selection the fit section does not guess: it says to select something, and Recent still works.', (c, w) => _Freq(key: ValueKey('d2$w'), sel: 0, history: true)),
        _uc('B3-d Empty query', 'a key selected', Hab.addition, 'The fit section follows the selection kind: for a key it lists curves and ease presets, not layer effects.', (c, w) => _Freq(key: ValueKey('d3$w'), sel: 2, history: true)),
        _uc('B3-d Empty query', 'first run, no history', Hab.addition, 'No history yet: Recent explains itself in a line instead of an empty box; Fit and the rest still answer.', (c, w) => _Freq(key: ValueKey('d4$w'), sel: 1, history: false)),
      ]),
      WidgetbookComponent(name: 'Browser B2 Chips', useCases: [
        _parts('Chips and field', _pChips),
        _uc('B2-b Chips', 'sentences become chips', Hab.addition, 'A filter typed as kind:effect becomes a chip you can see and remove / chip count = filter count / removing all chips returns the plain list.', (c, w) => _ChipField(key: ValueKey('g1$w'), chips: 2)),
        _uc('B2-b Chips', 'first use: the keys are offered', Hab.addition, 'An empty field offers the keys as tappable hints, and typing a key completes its values: the syntax is learnable without a manual.', (c, w) => _ChipField(key: ValueKey('g2$w')), narrow: false),
        _uc('B2-b Chips', 'many chips, narrow dock', Hab.addition, 'Chips wrap instead of scrolling away; when the field is not in use, extra chips fold to +N and the count in the band stays true.', (c, w) => _ChipField(key: ValueKey('g3$w'), chips: 6), narrow: true),
        _uc('B2-b Chips', 'a typo gets a reason', Hab.addition, 'An unknown value does not silently match nothing: the field says what it is not and lists the valid words.', (c, w) => _ChipField(key: ValueKey('g4$w'), chips: 1, text: 'kind:vidoe')),
      ]),
      WidgetbookComponent(name: 'Browser B2 Sentence', useCases: [
        _parts('Heading states', _pSentence),
        _uc('B2-c Sentence heading', 'the filter, said in words', Hab.departs, 'The heading names every filter in one sentence / clicking a word drops only that filter / with no filter it says All N.', (c, w) => _Sentence(key: ValueKey('h1$w'), kind: 1, chr: 1, fav: true)),
        _uc('B2-c Sentence heading', 'nothing fits', Hab.departs, 'When the sentence matches nothing it says so, and each word offers its own count if dropped.', (c, w) => _Sentence(key: ValueKey('h2$w'), kind: 3, chr: 1)),
        _uc('B2-c Sentence heading', 'wraps in a narrow dock', Hab.departs, 'The sentence wraps onto lines instead of truncating, because a cut-off filter is a hidden filter.', (c, w) => _Sentence(key: ValueKey('h3$w'), kind: 1, chr: 3, fav: true, text: 'glow'), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B2 Saved filters', useCases: [
        _parts('Places and buttons', _pSaved),
        _uc('B2-d Saved filters', 'a filter becomes a place', Hab.addition, 'A saved filter is listed as a place with its live count / new matching items join by themselves / deleting it never deletes the items.', (c, w) => _Saved(key: ValueKey('i1$w'))),
        _uc('B2-d Saved filters', 'many saved, folded', Hab.addition, 'Past four saved filters the list folds to three plus Show N more, so places stay a short list however many you keep.', (c, w) => _Saved(key: ValueKey('i2$w'), extra: 8), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B10 Collections', useCases: [
        _parts('Slots and membership', _pCollections),
        _uc('B10-a Collections', 'number keys 1 to 9', Hab.habit, '1 to 9 add the selected row to that collection, 0 removes it from all / a collection is one click and shows as a filter in the band / colours are not used.', (c, w) => _Collections(key: ValueKey('q1$w'), w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B10 Star and recent', useCases: [
        _parts('Star rows and headings', _pStar),
        _uc('B10-b Star and auto recent', 'F stars, the rest fills itself', Hab.habit, 'F stars the selected row / Recent and Frequent come from use with nothing to maintain / a double-click or Enter counts as a use.', (c, w) => _StarRecent(key: ValueKey('o1$w'))),
        _uc('B10-b Star and auto recent', 'first run', Hab.habit, 'On first run each automatic section explains in a line when it fills, so empty never looks broken.', (c, w) => _StarRecent(key: ValueKey('o2$w'), empty: true)),
      ]),
      WidgetbookComponent(name: 'Browser B10 Used stack', useCases: [
        _parts('Used rows', _pUsed),
        _uc('B10-c Used stack', 'newest on top, no tidying', Hab.addition, 'Using something puts it first / the same thing folds into one row with a count / past 12 the oldest drops off.', (c, w) {
                final n = c.knobs.int.slider(label: 'Shelf size', initialValue: 6, min: 0, max: 12);
                return _UsedStack(key: ValueKey('p1$n$w'), shelf: n);
              }),
        _uc('B10-c Used stack', 'shelf at its cap', Hab.addition, 'At 12 a new use pushes the oldest off and says which one, so nothing disappears silently.', (c, w) => _UsedStack(key: ValueKey('p2$w'), shelf: 12), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B12 Similar', useCases: [
        _parts('Tile, chip, band', _pSimilar),
        _uc('B12-b Similar', 'more like this', Hab.addition, 'Y or the Similar button re-sorts the same list by closeness and shows it as one removable filter / the source stays first / Clear returns all.', (c, w) => _Similar(key: ValueKey('r1$w'), w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B5 Moving tiles', useCases: [
        _parts('Tile and stand-ins', _pTiles),
        _uc('B5-a Moving tiles', 'every tile screens itself', Hab.departs, 'A tile moves only while visible / every tile has the same period / reduce-motion holds one still frame.', (c, w) {
                final n = c.knobs.int.slider(label: 'Tiles', initialValue: 24, min: 4, max: 200);
                final r = c.knobs.boolean(label: 'Reduce motion', initialValue: false);
                return _MovingTiles(key: ValueKey('j1$n$r$w'), count: n, reduce: r, w: w);
              }),
        _uc('B5-a Moving tiles', '200 tiles, reduce motion', Hab.departs, 'With 200 tiles and reduce-motion on, nothing moves and the panel still shows what each does; the cost is one frame, not 200 clocks.', (c, w) => _MovingTiles(key: ValueKey('j2$w'), count: 200, reduce: true, w: w), narrow: true),
      ]),
      WidgetbookComponent(name: 'Browser B5 Hover and scrub', useCases: [
        _parts('Thumbnail states', _pHover),
        _parts('Rows, tiles, big clips', _pHoverRows),
        _uc('B5-b Hover plays', 'tiles', Hab.habit, 'Hover plays that tile only / leaving returns to its first picture / looking never writes, so skimming the shelf leaves no history.', (c, w) => _HoverTiles(key: ValueKey('k1$w'), w: w)),
        _uc('B5-b Hover plays', 'rows in a narrow dock', Hab.habit, 'The same contract in rows: the thumbnail is the hover target, x on it scrubs timed items, the name still selects.', (c, w) => _HoverTiles(key: ValueKey('k2$w'), w: w, rows: true), narrow: true),
        _uc('B6 Scrub', 'long clips and presets', Hab.habit, 'x position over a tile is the time / a time pill and a line show where you are / leaving returns to the poster frame.', (c, w) => _ScrubClips(key: ValueKey('l1$w'), w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B5 Own frame', useCases: [
        _parts('Frame, banner, counters', _pOwn),
        _uc('B5-c Own frame', 'peek with the arrows', Hab.addition, 'Down or Up shows the next candidate on your own frame / nothing is written until Enter / Esc, leaving the panel or Cmd-Z puts it back.', (c, w) {
                final hov = _s(c, 'Trigger', const ['Arrows (D-1)', 'Hover'], 'Arrows (D-1)') == 'Hover';
                return _OwnFrame(key: ValueKey('m1$hov$w'), w: w, hover: hov, noTarget: false);
              }),
        _uc('B5-c Own frame', 'no selection: sample frame', Hab.addition, 'With nothing selected the frame is a sample and says so (D-9) / Enter writes nothing because there is no target.', (c, w) => _OwnFrame(key: ValueKey('m2$w'), w: w, hover: false, noTarget: true)),
        _uc('B5-c Own frame', 'previewing, applied, dropped', Hab.addition, 'The three states differ by word and grey step: dashed PREVIEW = not applied, solid APPLIED = one write, quiet DROPPED = back to the original.', (c, w) {
                final n = _s(c, 'Effect', const ['Glow', 'Gaussian Blur', 'Glitch RGB Split', 'Film Grain', 'Wave Warp'], 'Glow');
                return _Phases(w: w, fx: kItems.firstWhere((i) => i.name == n));
              }),
      ]),
      WidgetbookComponent(name: 'Browser B5 Preview surface', useCases: [
        _parts('Overlays and closed bar', _pSurface),
        _uc('B5-d Preview surface', 'a fixed big surface', Hab.habit, 'The surface follows the selection with no press / Space plays and pauses / Hide gives its height back and leaves nothing behind.', (c, w) {
                final s = c.knobs.boolean(label: 'Sample frame instead of yours', initialValue: false);
                return _BigPreview(key: ValueKey('n1$s$w'), w: w, sample: s);
              }),
        _uc('B5-d Preview surface', 'hidden', Hab.habit, 'With the surface hidden the list takes the whole height and a one-line bar offers it back.', (c, w) => _BigPreview(key: ValueKey('n2$w'), w: w, open: false)),
        _uc('B5-d Preview surface', 'narrow dock', Hab.habit, 'In a 232 px dock the surface scales with the width (16:9) and the list keeps at least ten rows.', (c, w) => _BigPreview(key: ValueKey('n3$w'), w: w), narrow: true),
      ]),
    ];
