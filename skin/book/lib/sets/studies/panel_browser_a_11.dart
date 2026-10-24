// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

// ---- B3 Find
Widget _pFindChrome() {
  // One plate width per row of the sheet: 300 fields, 212 narrow, 72 marks, 64 cells, 320 lines.
  _PC field(String cap, Widget w, {double width = 300}) => _PC(cap, width, w, h: 28, pad: 8);
  _PC mark(String cap, Widget w, {double h = 16}) => _PC(cap, 72, _left(w), h: h, pad: 8);
  _PC cell(String cap, Widget w) => _PC(cap, 64, _left(w), h: 22, pad: 8, ground: N.g07);
  _PC line(String cap, Widget w, {double? h}) => _PC(cap, 320, w, h: h);
  Widget count(String t) => Padding(padding: const EdgeInsets.fromLTRB(10, 8, 10, 6), child: Text(t, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)));
  const longText = 'Directional Blur motion trail 8-sample GPU soft';
  return _Parts('B3', 'Find: field, tabs, key marks, status', 'The chrome every find option shares. Field 300 x 28 in a 320 panel; tabs are a 26 px trough with 22 px cells; key marks 16 px. Field edge: rest g20, hover g38, focus g63. Not designed: a disabled field (find is always on).', [
    ('Search field: states', [
      field('rest, empty: hint + "/" key', const _SearchPart()),
      field('hover, empty: g38 edge', const _SearchPart(hover: true)),
      field('focused, empty: g63 edge, no "/" key', const _SearchPart(focus: true)),
      field('typed, resting: clear mark', const _SearchPart(text: 'bl')),
      field('typed, hover: g38 edge', const _SearchPart(text: 'bl', hover: true)),
      field('typed, clear mark hovered: x g95', const _SearchPart(text: 'bl', clearHover: true)),
      field('typed, focused', const _SearchPart(text: 'bl', focus: true)),
      field('Tab menu variant: Tab key mark', const _SearchPart(focus: true, slash: false, hint: 'Add effect to Layer 3', trailing: _Kbd('Tab'))),
    ]),
    ('Search field: long text', [
      field('long text, resting: ellipsis', const _SearchPart(text: longText)),
      field('long text, focused: fades at the edge', const _SearchPart(text: longText, focus: true)),
    ]),
    ('Search field: narrow dock 232 (field 212)', [
      field('narrow dock 232: field 212, hint', const _SearchPart(), width: 212),
      field('narrow dock 232: typed, resting', const _SearchPart(text: 'bl'), width: 212),
      field('narrow dock 232: long text, resting', const _SearchPart(text: longText), width: 212),
      field('narrow dock 232: long text, focused', const _SearchPart(text: longText, focus: true), width: 212),
    ]),
    ('Key marks and the clear mark (one centre, one right inset)', [
      _PC('"/" and the clear mark in the same 18 px slot: same centre, same 8 px right inset', 300, Column(crossAxisAlignment: CrossAxisAlignment.end, mainAxisSize: MainAxisSize.min, children: [
        Container(height: 20, padding: const EdgeInsets.only(right: 8), alignment: Alignment.centerRight, color: N.g07, child: const SizedBox(width: 18, child: _Kbd('/'))),
        Container(height: 20, padding: const EdgeInsets.only(right: 8), alignment: Alignment.centerRight, color: N.g07, child: const _Cross(w: 18)),
      ]), pad: 8),
      mark('"/" rest', const _Kbd('/')),
      mark('Tab', const _Kbd('Tab')),
      mark('Enter', const _Kbd('Enter')),
      mark('Esc', const _Kbd('Esc')),
      mark('live mark (g26)', const _Kbd('#', on: true)),
      mark('clear mark rest (g63)', const _Cross(w: 18), h: 20),
      mark('clear mark hover (g95)', const _Cross(w: 18, hover: true), h: 20),
    ]),
    ('Tabs', [
      _PC('nothing selected = all places: four cells at rest (no tab is added for All)', 300, _Tabs(_places, -1, (_) {}, toggle: false), h: 26, pad: 8),
      _PC('first selected: g20 pill + g95 + 1 px accent line', 300, _Tabs(_places, 0, (_) {}, toggle: false), h: 26, pad: 8),
      _PC('selected 1st, hover on 2nd (g15)', 300, _Tabs(_places, 0, (_) {}, toggle: false, hoverIndex: 1), h: 26, pad: 8),
      _PC('narrow 212: scrolls, text never shrinks', 212, _Tabs(_places, 0, (_) {}, toggle: false), h: 26, pad: 8),
      cell('one cell: rest', const _TabCell('Effects')),
      cell('one cell: hover', const _TabCell('Effects', hover: true)),
      cell('one cell: selected (g20 pill, g95, 1 px accent line)', const _TabCell('Effects', on: true)),
      cell('one cell: selected + hover', const _TabCell('Effects', on: true, hover: true)),
    ]),
    ('Count line, heading, status', [
      line('count line, no tab on', count('12 of 82  ·  no tab on: all places')),
      line('count line, one tab on', count('9 of 31  ·  Presets')),
      line('foot, results outside the scope: "N more in other places" (only what the count line does not show)', const _EndLine(22), h: 26),
      line('foot, nothing outside and the count line does not say so: "End of results" (otherwise no foot)', const _EndLine(0), h: 26),
      line('count line, all places, one place is most rows: said once', count('72 of 72  ·  no tab on: all places  ·  mostly effects')),
      line('count line, no result', count('No results')),
      line('section heading + count', _Sec('Effects', right: Text('12', style: T.label(N.g76)))),
      line('status, short', const _Status('Layer 3 · Title', keys: ['Esc']), h: 26),
      line('status, long: one line, ellipsis', const _Status(_statusLong), h: 26),
      _PC('status, narrow 232', 232, const _Status('Layer 3 · Title', keys: ['Esc']), h: 26),
    ]),
    ('Empty result', [
      line('no match: what was searched + ways out', _Empty('Nothing matches “glwo”', const ['Searched only Presets.'], actions: [_Btn('Clear search', onTap: () {}), _Btn('Search all places', onTap: () {}), _Btn('Did you mean “glow”?', onTap: () {})]), h: 220),
    ]),
  ]);
}

Widget _pRows() {
  final g = _by('Gaussian Blur'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)'), jp = _by('夜のネオン (client A)'), pre = _by('Typewriter'), media = _by('lens_bloom.mp4'), lut = _by('Apply LUT (Cinematic Teal & Orange 3D)');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 34);
  return _Parts('B3', 'Find: item rows', 'The row every list shares, 34 px. Two left edges on purpose: the 2 px selected tick sits in the gutter (x 2 to 4) and the content edge stays at 10 px, so the tick never pushes text. Thumbnail and name start at x 10. Name 11 px, second line 10 px g63, kind or place on the right. A filter that already fixes a fact is not repeated by its rows.', [
    ('States', [
      row('rest', _ItemRow(g)),
      row('hover: g15', _ItemRow(g, hover: true)),
      row('selected: g20 + 2 px tick', _ItemRow(g, selected: true)),
      row('query "bl" matched, rest', _ItemRow(g, tokens: const ['bl'])),
      row('query "bl" matched, selected', _ItemRow(g, tokens: const ['bl'], selected: true)),
    ]),
    ('Content', [
      row('long English name: ellipsis', _ItemRow(long)),
      row('long name, selected', _ItemRow(long, selected: true)),
      row('Japanese-led name', _ItemRow(jp)),
      row('long name with units', _ItemRow(lut)),
      row('preset', _ItemRow(pre)),
      row('media (asset art)', _ItemRow(media)),
      row('mine: place on the right, not repeated below', _ItemRow(_by('My Soft Bloom 01'), right: Text('Mine', style: T.micro(N.g76)))),
      row('grouped under a heading: no label on the right', _ItemRow(g, right: const SizedBox.shrink())),
      row('place on the right (across places)', _ItemRow(g, right: Text(g.place, style: T.micro(N.g76)))),
    ]),
    ('A filter already says it: the row does not repeat it', [
      row('no filter: kind on the right, category in line 2', _ItemRow(g)),
      row('Kind = Effect on: no kind label on the right', _ItemRow(g, showKind: false)),
      row('Category = Blur on: line 2 has no category', _ItemRow(g, showCat: false)),
      row('Kind and Category on: nothing repeated', _ItemRow(g, showKind: false, showCat: false)),
      row('mixed list, media: place on the right, line 2 has no "Video"', _ItemRow(media, right: Text(media.place, style: T.micro(N.g76)))),
      row('mixed list, effect: place on the right, category kept', _ItemRow(g, right: Text(g.place, style: T.micro(N.g76)))),
    ]),
    ('Thumbnails: one shape per family', [
      row('blur: concentric soft rings', _ItemRow(_by('Radial Blur'), showKind: false)),
      row('glow: a halo', _ItemRow(_by('Light Rays'), showKind: false)),
      row('warp: a wave', _ItemRow(_by('Twirl'), showKind: false)),
      row('stylize: blocks', _ItemRow(_by('Halftone'), showKind: false)),
      row('colour: a gradient chip', _ItemRow(_by('Levels'), showKind: false)),
      row('glitch / echo / grain / wipe / pop', _ItemRow(_by('VHS Wobble'), showKind: false)),
    ]),
    ('Narrow dock 232', [
      row('rest', _ItemRow(long), width: 232),
      row('selected', _ItemRow(long, selected: true), width: 232),
      row('Japanese-led, hover', _ItemRow(jp, hover: true), width: 232),
    ]),
  ]);
}

// ---- B3 Tab menu
Widget _pMenu() {
  final g = _by('Glow'), long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC mrow(String cap, Widget w) => _PC(cap, 296, w, h: 28, ground: N.g13);
  return _Parts('B3-b', 'Tab menu: pieces', 'The small menu that opens at the cursor: g13, 1 px g26 edge, 6 px radius. Rows 28 px, family bar 26 px. One left rhythm: the tick in the gutter, field, tabs, names and headings all start at x 10.', [
    ('Family bar', [
      _PC('rest: g63 word', 72, const Center(child: _TabCell('Distort')), h: 26, ground: N.g13),
      _PC('hover: g15', 72, const Center(child: _TabCell('Distort', hover: true)), h: 26, ground: N.g13),
      _PC('selected: g20 pill + g95 + 1 px accent line', 72, const Center(child: _TabCell('Distort', on: true)), h: 26, ground: N.g13),
    ]),
    ('Menu rows', [
      mrow('rest: family on the right', _MenuRow(g.name, tail: g.cat)),
      mrow('hover: g15', _MenuRow(g.name, tail: g.cat, hover: true)),
      mrow('selected: g20 + Enter key mark', _MenuRow(g.name, selected: true)),
      mrow('empty input: last used, time on the right', _MenuRow(g.name, tail: _ago(3))),
      mrow('query "gl" matched, rest', _MenuRow(g.name, tokens: const ['gl'], tail: g.cat)),
      mrow('query "gl" matched, selected', _MenuRow(g.name, tokens: const ['gl'], selected: true)),
      mrow('long name: ellipsis, tail kept', _MenuRow(long.name, tail: _ago(240))),
      mrow('selected without a target: tail, no Enter', _MenuRow(g.name, selected: true, tail: g.cat)),
    ]),
    ('Heading, notice, empty line', [
      _PC('heading: last used', 296, const _Sec('Last used first', pad: EdgeInsets.fromLTRB(10, 6, 10, 2)), ground: N.g13),
      _PC('heading: matches', 296, const _Sec('Matches · 4', pad: EdgeInsets.fromLTRB(10, 6, 10, 2)), ground: N.g13),
      _PC('no target: danger bar, g10 on g13', 276, const _Notice(_noTargetText), ground: N.g13, pad: 10),
      _PC('nothing in this family', 296, Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 10), child: Text('Nothing in Glow matches “zz”', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))), ground: N.g13),
    ]),
    ('Input and the closed card', [
      _PC('input: Tab key mark', 276, const _SearchPart(text: 'gl', focus: true, slash: false, trailing: _Kbd('Tab')), h: 28, pad: 10, ground: N.g13),
      _PC('input, no target', 276, const _SearchPart(slash: false, hint: 'Add effect (no target)', trailing: _Kbd('Tab')), h: 28, pad: 10, ground: N.g13),
      _PC('closed card: after Enter', 296, const _ClosedCard('Applied “Glow” to Layer 3 · Title')),
      _PC('closed card: after Esc', 296, const _ClosedCard('Closed, nothing written')),
    ]),
  ]);
}

// ---- B3 Prefix
Widget _pPrefix() {
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 34);
  return _Parts('B3-c', 'Prefix: rows, area line, marks', 'Rows of tags, actions and layers share the item row geometry: tick, name 11 px, second line 10 px, count or key on the right.', [
    ('Rows', [
      row('tag row', const _PlainRow(title: '#soft', sub: 'tag', right: '12', count: true, selected: false)),
      row('tag row, hover', const _PlainRow(title: '#soft', sub: 'tag', right: '12', count: true, selected: false, hover: true)),
      row('tag row, selected', const _PlainRow(title: '#soft', sub: 'tag', right: '12', count: true, selected: true)),
      row('action: mono key on the right', const _PlainRow(title: 'Add keyframe at playhead', sub: '現在位置にキーを打つ', right: 'K', selected: false, kbd: true)),
      row('action: long name and long key', const _PlainRow(title: 'Reset all effects on Layer 3', sub: 'Layer 3 のエフェクトをリセット', right: 'Cmd Shift R', selected: false, kbd: true)),
      row('action: query "re" matched', const _PlainRow(title: 'Reset all effects on Layer 3', sub: 'Layer 3 のエフェクトをリセット', right: 'Cmd Shift R', selected: false, kbd: true, toks: ['re'])),
      row('layer row', const _PlainRow(title: 'キャッチコピー', sub: 'Layer', right: 'Text', selected: false)),
      row('mark row: the example and the chevron', const _PlainRow(title: '#  tag', sub: '#soft', right: '›', selected: false)),
      row('narrow 232, long action', const _PlainRow(title: 'Align to composition center', sub: 'コンポジションの中央に整列', right: 'Cmd Alt C', selected: false, kbd: true), width: 232),
    ]),
    ('Area line and heading', [
      _PC('no mark: all areas', 300, const _AreaLine('all areas', ''), pad: 8),
      _PC('# lit', 300, const _AreaLine('tags', '#'), pad: 8),
      _PC('> lit', 300, const _AreaLine('actions', '>'), pad: 8),
      _PC('@ lit', 300, const _AreaLine('layers', '@'), pad: 8),
      _PC('section heading + count', 320, _Sec('Tags', right: Text('12', style: T.label(N.g76)))),
      _PC('capped section: what is hidden + the key', 320, const _MoreLine('+10 more'), h: 24),
    ]),
    ('Field and empty', [
      _PC('field hint, 300', 300, const _SearchPart(hint: 'Search, or use # > @'), h: 28, pad: 8),
      _PC('field hint, narrow 212', 212, const _SearchPart(hint: 'Search, or use # > @'), h: 28, pad: 8),
      _PC('typed mark', 300, const _SearchPart(text: '#so', focus: true), h: 28, pad: 8),
      _PC('nothing in this area', 320, _Empty('Nothing in tags for “zz”', const [], actions: [_Btn('Search all areas', onTap: () {})]), h: 180),
    ]),
  ]);
}

// ---- B3 Empty query
Widget _pFreq() {
  final g = _by('Gaussian Blur'), t = _by('Title In: Soft Rise');
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 34);
  return _Parts('B3-d', 'Empty query: headings, hints, reasons', 'Four sections answer an empty field. The reason a row is there is a g63 word on its right, at most 92 px.', [
    ('Headings', [
      _PC('Recent + where it comes from', 320, _Sec('Recent', right: Text('by you, all projects', style: T.label(N.g76))), h: 26),
      _PC('Fits + the selection', 320, const _Sec('Fits Layer 3 · Title (Text)'), h: 26),
      _PC('All others + count', 320, _Sec('All others', right: Text('64', style: T.label(N.g76))), h: 26),
    ]),
    ('Hints instead of empty boxes', [
      _PC('first run: Recent', 320, const _HintRow('None yet'), h: 36),
      _PC('no selection: Fits', 320, const _HintRow('No selection'), h: 36),
      _PC('narrow 232', 232, const _HintRow('No selection'), h: 36),
    ]),
    ('Reasons on the right of a row', [
      row('recent: time', _ItemRow(g, right: const _Why('3 min ago'))),
      row('fits: why', _ItemRow(t, right: const _Why('made for text'))),
      row('fits: why, long: ellipsis at 92', _ItemRow(t, right: const _Why('used on 12 text layers today'))),
      row('others: use count', _ItemRow(g, right: const _Why('41×'))),
      row('selected, with a reason', _ItemRow(t, selected: true, right: const _Why('made for text'))),
      row('hover, with a reason', _ItemRow(t, hover: true, right: const _Why('made for text'))),
      row('key selected: curve for keys', _ItemRow(_by('Ease Out Back (Soft)'), right: const _Why('curve for keys'))),
    ]),
  ]);
}

// ---- B2-a Tag bands
Widget _pTags() {
  _PC chip(String cap, Widget c, {double w = 104}) => _PC(cap, w, _left(c), h: 22, pad: 8);
  _PC grey(String cap, Color c) => _PC(cap, 72, Container(height: 22, decoration: BoxDecoration(color: c, border: Border.all(color: N.g20), borderRadius: BorderRadius.circular(4))), h: 22, pad: 8);
  const kind = ['Effect', 'Preset', 'Media'], kc = {'Effect': 52, 'Preset': 15, 'Media': 15};
  const cat = ['Blur', 'Color', 'Glow', 'Distort', 'Stylize', 'Time', 'Generate', 'Title', 'Motion', 'Ease'];
  const cc = {'Blur': 6, 'Color': 7, 'Glow': 6, 'Distort': 7, 'Stylize': 6, 'Time': 3, 'Generate': 5, 'Title': 5, 'Motion': 5, 'Ease': 0};
  Widget grp(String label, List<String> v, Set<String> on, {Map<String, int>? counts, bool open = true, bool hide = false, int? hover, bool? zero, bool? hoverToggle}) => _TagGroup(label: label, values: v, on: on, counts: counts, open: open, canHide: hide, hoverIndex: hover, zeroOpen: zero, hoverToggle: hoverToggle, onTap: (_) {});
  return _Parts('B2-a', 'Tag bands: chips and groups', 'A chip is 22 px, on the panel plate g10. Rest g15 (visibly above the plate), hover g20, on g26 + a 1 px accent line, pressed g20 + the accent line at 50%, keyboard focus a 1 px g63 edge. Count 0 is not drawn at rest: the group folds all of them into one "+N hidden tags" text button on the chip line (g76, g20 on hover, a 10 px chevron; "+1 hidden tag" for one); shown, they are g56 with a dashed g56 hairline and no fill, so they never read as live chips. Plates fit their content (the cell width is the plate width). A group row is 64 px of name (g63) and the chips; label at x 10, chips from x 74. List rows keep two left edges on purpose: the selected tick sits in the gutter (x 2 to 4), the content edge stays at 10 px.', [
    ('The greys (cell ground is the plate)', [
      grey('plate g10 (panel body)', N.g10),
      grey('rest g15', N.g15),
      grey('hover g20', N.g20),
      grey('on g26 (+ accent line)', N.g26),
      grey('label / count text g63', N.g13),
    ]),
    ('Chip states', [
      chip('rest: g15 on the g10 plate', const _ChipBtn('Blur', count: 6)),
      chip('hover: g20', const _ChipBtn('Blur', count: 6, hover: true)),
      chip('pressed: g20 + accent line 50%', const _ChipBtn('Blur', count: 6, pressed: true)),
      chip('keyboard focus: 1 px g63 edge', const _ChipBtn('Blur', count: 6, focus: true)),
      chip('on: g26 + 1 px accent line', const _ChipBtn('Blur', count: 6, on: true)),
      chip('on + hover: g26 + line + g63 edge', const _ChipBtn('Blur', count: 6, on: true, hover: true)),
      chip('no count (saved filter)', const _ChipBtn('Soft')),
      chip('no count, on', const _ChipBtn('Soft', on: true)),
      chip('two-digit count', const _ChipBtn('Stylize', count: 18), w: 120),
      chip('two-digit count, on', const _ChipBtn('Stylize', count: 18, on: true), w: 120),
      chip('long word, hover', const _ChipBtn('Directional', count: 112, hover: true), w: 136),
      chip('zero, when shown: g63 word, dashed g56 hairline, no fill', const _ChipBtn('Blur', count: 0), w: 120),
      chip('zero + hover: no change', const _ChipBtn('Blur', count: 0, hover: true), w: 120),
    ]),
    ('A long chip in a narrow group (ends in an ellipsis, never widens the row)', [
      _PC('232 dock: group 142 wide, chip takes what it has', 232, grp('Source', const ['Directional Blur Motion Trail 8-sample GPU', 'Mine'], {}, counts: const {'Directional Blur Motion Trail 8-sample GPU': 4, 'Mine': 3})),
      _PC('232 dock: the same long chip, on', 232, grp('Source', const ['Directional Blur Motion Trail 8-sample GPU', 'Mine'], {'Directional Blur Motion Trail 8-sample GPU'}, counts: const {'Directional Blur Motion Trail 8-sample GPU': 4, 'Mine': 3})),
    ]),
    ('Group rows, 320 wide', [
      _PC('open, nothing chosen', 320, grp('Kind', kind, {}, counts: kc)),
      _PC('open, one on, one hover', 320, grp('Kind', kind, {'Effect'}, counts: kc, hover: 1)),
      _PC('open, chips pack per line (Distort stays on line 1 when it fits); the zero chip folded: "+1 hidden tag", right after the last chip', 320, grp('Category', cat, {'Blur'}, counts: cc)),
      _PC('narrow 232: the row is full, chips pack to the width, the button follows the last chip and wraps only when the width is out', 232, grp('Category', cat, {}, counts: cc)),
      _PC('toggle hovered: g20 fill under "+1 hidden tag"', 320, grp('Category', cat, {'Blur'}, counts: cc, hoverToggle: true)),
      _PC('same group, hidden ones shown: dashed, g56, then "Hide 1"', 320, grp('Category', cat, {'Blur'}, counts: cc, zero: true)),
      _PC('many zeros (Media filtered): 8 dead chips collapse into one word', 320, grp('Category', cat, {}, counts: const {'Blur': 0, 'Color': 0, 'Glow': 6, 'Distort': 0, 'Stylize': 0, 'Time': 0, 'Generate': 0, 'Title': 0, 'Motion': 0, 'Ease': 0})),
      _PC('same, expanded', 320, grp('Category', cat, {}, counts: const {'Blur': 0, 'Color': 0, 'Glow': 6, 'Distort': 0, 'Stylize': 0, 'Time': 0, 'Generate': 0, 'Title': 0, 'Motion': 0, 'Ease': 0}, zero: true)),
      _PC('folded, nothing chosen', 232, grp('Category', const ['Blur', 'Color', 'Glow'], {}, open: false)),
      _PC('folded, a choice stays visible', 232, grp('Category', const ['Blur', 'Color', 'Glow'], {'Blur', 'Glow'}, open: false)),
      _PC('open in a dock: Hide word', 232, grp('Kind', kind, {}, hide: true)),
    ]),
    ('Words that act', [
      _PC('Show N: rest', 96, _left(const _Word('Show 10')), h: 14, pad: 8),
      _PC('Show N: hover, underlined', 96, _left(const _Word('Show 10', hover: true)), h: 14, pad: 8),
      _PC('+N hidden tags: rest (g76, chevron)', 120, _left(const _HiddenToggle(8)), h: 22, pad: 8),
      _PC('+N hidden tags: hover (g20 fill)', 120, _left(const _HiddenToggle(8, hover: true)), h: 22, pad: 8),
      _PC('+N hidden tags: pressed (g20 + accent line 50%)', 120, _left(const _HiddenToggle(8, pressed: true)), h: 22, pad: 8),
      _PC('+N hidden tags: keyboard focus (1 px g63 edge)', 120, _left(const _HiddenToggle(8, focus: true)), h: 22, pad: 8),
      _PC('+1 hidden tag: singular', 120, _left(const _HiddenToggle(1)), h: 22, pad: 8),
      _PC('open: Hide N, chevron up', 96, _left(const _HiddenToggle(8, open: true)), h: 22, pad: 8),
      _PC('status line, rest', 320, const _Status('Only', keys: ['Cmd', 'Shift']), h: 26),
      _PC('status line, add mode', 320, const _Status('Add', keys: ['Cmd', 'Shift']), h: 26),
    ]),
    ('End of a short list (always shown, directly under the last row)', [
      _PC('more results exist: only the cue, the count lives in the band', 320, const _EndLine(64, where: 'hidden by the filters'), h: 26),
      _PC('nothing more and the band does not say it: End of results', 320, const _EndLine(0, where: 'hidden by the filters'), h: 26),
    ]),
  ]);
}

// ---- B4 band
Widget _pBand() {
  _PC b(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 28);
  return _Parts('B4', 'Results band: states', 'One 28 px band under the filters: g13, 1 px row line above and below. The filter count is the brightest word; Clear is one word.', [
    ('Band', [
      b('no filter: quiet, Clear dimmed (g63, g20 hairline, no hover)', _Band(filters: 0, shown: 82, total: 82, onClear: () {})),
      b('no filter, pointer over Clear: no change (dimmed, arrow)', _Band(filters: 0, shown: 82, total: 82, onClear: () {}, hoverClear: true)),
      b('one filter: Clear at rest (g76, g26 hairline)', _Band(filters: 1, shown: 52, total: 82, onClear: () {})),
      b('three filters', _Band(filters: 3, shown: 6, total: 82, onClear: () {})),
      b('three filters, Clear hover (g20 fill)', _Band(filters: 3, shown: 6, total: 82, onClear: () {}, hoverClear: true)),
      b('three filters, Clear pressed (g20 + accent line 50%)', _Band(filters: 3, shown: 6, total: 82, onClear: () {}, pressClear: true)),
      b('three filters, Clear keyboard focus (1 px g63 edge)', _Band(filters: 3, shown: 6, total: 82, onClear: () {}, focusClear: true)),
      b('search text on: the text alone counts as one filter', _Band(filters: 1, shown: 9, total: 82, extra: 'text “glow”', onClear: () {})),
      b('narrow 232: the count text gives way, Clear stays', _Band(filters: 3, shown: 12, total: 82, extra: 'in 1 Soft blurs', onClear: () {}), width: 232),
      b('zero results: says so', _Band(filters: 4, shown: 0, total: 82, onClear: () {})),
      b('with a note: in 2 Glows', _Band(filters: 1, shown: 9, total: 64, extra: 'in 2 Glows', onClear: () {})),
      b('by similarity', _Band(filters: 1, shown: 40, total: 40, extra: 'by similarity', onClear: () {})),
    ]),
    ('Active filters: the same filled chip as B2, and the search box that carries x', [
      _PC('active chip: rest (g15, x inside)', 96, _left(const _ActiveChip('Effect')), h: 22, pad: 8),
      _PC('active chip: hover (g20, x g95)', 96, _left(const _ActiveChip('Effect', hover: true)), h: 22, pad: 8),
      _PC('active chip: pressed (g20 + accent line 50%)', 96, _left(const _ActiveChip('Effect', pressed: true)), h: 22, pad: 8),
      _PC('active chip: keyboard focus (1 px g63 edge)', 96, _left(const _ActiveChip('Effect', focus: true)), h: 22, pad: 8),
      _PC('active chip for the search text', 110, _left(const _ActiveChip('“glow”')), h: 22, pad: 8),
      _PC('search box with text: x', 300, const _SearchPart(text: 'glow'), h: 28, pad: 8),
      _PC('search box, clear mark hovered: x g95', 300, const _SearchPart(text: 'glow', clearHover: true), h: 28, pad: 8),
      _PC('end line, more results exist', 320, const _EndLine(70, where: 'hidden by the filters'), h: 26),
      _PC('end line, nothing more: End of results', 320, const _EndLine(0, where: 'hidden by the filters'), h: 26),
    ]),
  ]);
}

// ---- B2-b Chips
Widget _pChips() {
  _PC chip(String cap, Widget c, {double w = 144}) => _PC(cap, w, _left(c), h: 20, pad: 8);
  Widget field(List<Widget> kids, {bool focused = false}) => _FieldShell(focused: focused, children: kids);
  Widget typed(String t, {double w = 90}) => SizedBox(width: w, height: 20, child: Align(alignment: Alignment.centerLeft, child: Text(t, style: T.name(N.g95))));
  Widget hint(String t, {double w = 250}) => SizedBox(width: w, height: 20, child: Align(alignment: Alignment.centerLeft, child: Text(t, style: T.name(N.g76))));
  return _Parts('B2-b', 'Chips: parts', 'A chip is 20 px: g20, key g63, value g95 semibold, a 16 x 20 cross. The field is g07 with a 1 px g20 edge (g63 focused), 4 px inside.', [
    ('Chip', [
      chip('rest', const _FilterChip('kind', 'effect')),
      chip('cross hover: g95', const _FilterChip('kind', 'effect', hoverCross: true)),
      chip('longer value', const _FilterChip('cat', 'generate')),
      chip('similar chip: long file name, clipped', const _FilterChip('similar', 'dusk_ridge_golden_hour_v2.jpg'), w: 160),
      chip('fold chip: +N more (g15)', const _MoreChip(4), w: 80),
      _PC('hint chip: rest', 72, _left(const _HintChip('kind:')), h: 22, pad: 8),
      _PC('hint chip: hover', 72, _left(const _HintChip('kind:', hover: true)), h: 22, pad: 8),
      _PC('hint chip: longest', 88, _left(const _HintChip('fav:yes')), h: 22, pad: 8),
    ]),
    ('Field, 300 wide', [
      _PC('empty, resting: hint', 300, field([hint('Search, or kind: tag: cat:')]), pad: 8),
      _PC('empty, focused', 300, field([hint('Search, or kind: tag: cat:')], focused: true), pad: 8),
      _PC('two chips, resting', 300, field(const [_FilterChip('kind', 'effect'), _FilterChip('tag', 'soft')]), pad: 8),
      _PC('two chips, focused', 300, field([const _FilterChip('kind', 'effect'), const _FilterChip('tag', 'soft'), typed('bl')], focused: true), pad: 8),
      _PC('six chips wrap (narrow 212, focused)', 212, field(const [_FilterChip('kind', 'effect'), _FilterChip('tag', 'soft'), _FilterChip('cat', 'glow'), _FilterChip('in', 'effects'), _FilterChip('tag', 'bright'), _FilterChip('fav', 'yes')], focused: true), pad: 8),
      _PC('six chips folded when not in use', 212, field(const [_FilterChip('kind', 'effect'), _FilterChip('tag', 'soft'), _MoreChip(4)]), pad: 8),
    ]),
    ('Completion list and the typo note', [
      _PC('keys: first row is the one Enter takes', 300, const _SugBox([_SugRow('kind:', first: true), _SugRow('cat:'), _SugRow('tag:')]), pad: 8),
      _PC('values, one row hover (g15)', 300, const _SugBox([_SugRow('tag:soft', first: true), _SugRow('tag:glitch', hover: true), _SugRow('tag:retro')]), pad: 8),
      _PC('unknown value: why and the valid words', 300, const _Notice('“vidoe” is not a kind. Try effect, preset, media.', fill: N.g13), pad: 8),
      _PC('"Try" label and keys', 300, Row(children: [Padding(padding: const EdgeInsets.only(right: 4), child: Text('Try', style: T.label(N.g76))), const _HintChip('kind:'), const SizedBox(width: 4), const _HintChip('cat:'), const SizedBox(width: 4), const _HintChip('tag:')]), pad: 8),
    ]),
  ]);
}

// ---- B2-c Sentence
Widget _pSentence() => _Parts('B2-c', 'Sentence heading: states', 'The heading is 13 px semibold on g13 with a row line above and below, 12 px inside. Each word is a filter; hover = g95 + underline; the tail says how many remain.', [
      ('Heading, 320', [
        _PC('no filter', 320, const _SentenceHead(words: [], count: 82, total: 82)),
        _PC('one word', 320, const _SentenceHead(words: ['Soft'], count: 22, total: 82)),
        _PC('three words, rest', 320, const _SentenceHead(words: ['Soft', 'Effects', 'Favorites'], count: 3, total: 82)),
        _PC('three words, 2nd hovered', 320, const _SentenceHead(words: ['Soft', 'Effects', 'Favorites'], count: 3, total: 82, hoverIndex: 1)),
        _PC('search text as a word', 320, const _SentenceHead(words: ['Soft', 'Presets', '“glow”'], count: 0, total: 82)),
        _PC('nothing fits', 320, const _SentenceHead(words: ['Media', 'Soft'], count: 0, total: 82)),
        _PC('narrow 232: wraps, never cut', 232, const _SentenceHead(words: ['Soft', 'Effects', 'Favorites', '“glow”'], count: 2, total: 82)),
      ]),
      ('Around it', [
        _PC('Favorites only: switch off, word to its right', 300, Row(children: [PillSwitch(on: false, onChanged: (_) {}), const SizedBox(width: 8), Text('Favorites only', style: T.label(N.g76))]), pad: 8),
        _PC('Favorites only: switch on', 300, Row(children: [PillSwitch(on: true, onChanged: (_) {}), const SizedBox(width: 8), Text('Favorites only', style: T.label(N.g76))]), pad: 8),
        _PC('kind tabs: Effect selected', 300, _Tabs(const ['Any', 'Effect', 'Preset', 'Media'], 1, (_) {}, toggle: false), h: 26, pad: 8),
        _PC('character tabs: Soft, Retro hover', 300, _Tabs(const ['Any', 'Soft', 'Glitch', 'Retro', 'Bright', 'Grade'], 1, (_) {}, toggle: false, hoverIndex: 3), h: 26, pad: 8),
      ]),
    ]);

// ---- B2-d Saved filters
Widget _pSaved() {
  _PC row(String cap, Widget w, {double width = 320}) => _PC(cap, width, w, h: 26);
  return _Parts('B2-d', 'Saved filters: places and buttons', 'A place or a saved filter is a 26 px row: tick, name, live count. On hover a saved filter swaps its count for Delete.', [
    ('Place rows', [
      row('place, rest', const _PlaceRow('Effects', 52)),
      row('place, hover', const _PlaceRow('Effects', 52, hover: true)),
      row('place, selected', const _PlaceRow('Effects', 52, on: true)),
      row('saved filter, rest: count', const _PlaceRow('Soft blurs', 9)),
      row('saved filter, hover: Delete replaces the count', _PlaceRow('Soft blurs', 9, hover: true, right: const _Word('Delete'))),
      row('saved filter, selected', const _PlaceRow('Soft blurs', 9, on: true)),
      row('long name: ellipsis, count kept', const _PlaceRow('Everything soft and bright, client A look', 14)),
      row('count 0', const _PlaceRow('Bokeh set', 0)),
      row('narrow 232, hover', _PlaceRow('Everything soft and bright', 14, hover: true, right: const _Word('Delete')), width: 232),
    ]),
    ('Headings and folding', [
      _PC('Places', 320, const _Sec('Places'), h: 24),
      _PC('Saved filters + count', 320, _Sec('Saved filters', right: Text('12', style: T.label(N.g76))), h: 24),
      _PC('fold word: rest', 240, const Padding(padding: EdgeInsets.fromLTRB(10, 4, 10, 0), child: _Word('Show 9 more saved filters')), h: 20),
      _PC('fold word: hover', 240, const Padding(padding: EdgeInsets.fromLTRB(10, 4, 10, 0), child: _Word('Show 9 more saved filters', hover: true)), h: 20),
    ]),
    ('Filter chips and buttons', [
      _PC('Category group, one on', 320, _TagGroup(label: 'Category', values: const ['Blur', 'Glow', 'Color', 'Title'], on: const {'Glow'}, onTap: (_) {})),
      _PC('Character group, hover on 3rd', 320, _TagGroup(label: 'Character', values: const ['Soft', 'Retro', 'Bright', 'Loop'], on: const {}, hoverIndex: 2, onTap: (_) {})),
      _PC('button: rest', 120, _left(_Btn('Save as place…', onTap: () {})), h: 24, pad: 8),
      _PC('button: hover', 120, _left(_Btn('Save as place…', onTap: () {}, hover: true)), h: 24, pad: 8),
      _PC('button: keyboard focus (g63 edge)', 120, _left(_Btn('Save as place…', onTap: () {}, focus: true)), h: 24, pad: 8),
      _PC('button: disabled (nothing to save)', 120, _left(const _Btn('Save as place…')), h: 24, pad: 8),
      _PC('button: primary', 120, _left(_Btn('Show everything', onTap: () {}, primary: true)), h: 24, pad: 8),
      _PC('name field: Esc key mark', 300, const _SearchPart(focus: true, slash: false, hint: 'Name this filter, Enter to save', trailing: _Kbd('Esc')), h: 28, pad: 8),
    ]),
  ]);
}

// ---- B5-a Moving tiles
Widget _pTiles() {
  final long = _by('Directional Blur (Motion Trail, 8-sample, GPU)');
  _PC tile(String cap, It it, {double w = 146, bool hover = false, double t = .55}) => _PC(cap, w, _TileCard(it, t: t, w: w, hover: hover), pad: 8);
  return _Parts('B5-a', 'Moving tiles: card and stand-in pictures', 'A tile is a 1 px g20 picture edge (g56 on hover), 6 px, the name 11 px, 4 px, then category and kind 10 px g63. Pictures are held at t = .55 here.', [
    ('Tile, two across in a 320 panel (146 wide)', [
      tile('rest', _by('Glow')),
      tile('hover: edge g56', _by('Glow'), hover: true),
      tile('long English name: ellipsis', long),
      tile('Japanese-led name', _by('夜のネオン (client A)')),
      tile('preset', _by('Typewriter')),
      tile('media-less stand-in: grain', _by('Film Grain')),
    ]),
    ('Tile, two across in a 232 dock (102 wide)', [
      tile('rest', _by('Glow'), w: 102),
      tile('hover', _by('Glow'), w: 102, hover: true),
      tile('long name', long, w: 102),
    ]),
    ('The eleven stand-in pictures at t = .55 (what each effect family looks like)', [
      for (final f in Fx.values) _PC(f.name, 102, SizedBox(width: 102, height: 63, child: ClipRRect(borderRadius: BorderRadius.circular(5), child: _Fx(f, .55))), h: 63, pad: 8),
    ]),
  ]);
}

// ---- B5-b / B6 hover and scrub
Widget _pHover() {
  final vid = _by('lens_bloom.mp4'), img = _by('dusk_ridge.jpg'), aud = _by('kick_loop.wav'), glow = _by('Glow'), tw = _by('Typewriter'), pano = _by('sphere_tour.mp4');
  _PC thumb(String cap, It it, {bool sel = false, bool hov = false, double? x, double w = 146}) => _PC(cap, w, _HoverThumb(it, w: w, h: w * .62, selected: sel, holdHover: hov, holdX: x), h: w * .62, pad: 8);
  return _Parts('B5-b', 'Hover and scrub: thumbnails', 'A thumbnail is a 1 px edge (g20 rest, g56 hover, mode selected), 6 px radius. Time pill and type badge sit 4 px in. A scrubbing line is 1 px white.', [
    ('Thumbnail states (146 x 91)', [
      thumb('still image, rest', img),
      thumb('video, rest: type badge + length', vid),
      thumb('video, hover at 0 %', vid, hov: true, x: 0),
      thumb('video, hover at 50 %: line + time', vid, hov: true, x: 73),
      thumb('video, hover at 100 %', vid, hov: true, x: 146),
      thumb('video, selected: mode edge', vid, sel: true),
      thumb('audio, rest', aud),
      thumb('360 clip, hover at 30 %', pano, hov: true, x: 44),
      thumb('preset effect, rest', glow),
      thumb('preset effect, hover: playing', glow, hov: true),
      thumb('timed preset, hover at 40 %', tw, hov: true, x: 58),
      thumb('selected still', img, sel: true),
    ]),
    ('Thumbnail in a 232 dock (102 wide)', [
      thumb('video, hover at 50 %', vid, hov: true, x: 51, w: 102),
      thumb('preset, hover', glow, hov: true, w: 102),
    ]),
    ('Badges over a picture (the 16 px rule)', [
      _PC('Playing label', 72, _left(_pill('Playing')), h: 16, pad: 8, ground: N.g26),
      _PC('Source label', 72, _left(_pill('Source', fill: N.g26)), h: 16, pad: 8),
      _PC('Similar label', 72, _left(_pill('Similar', fill: N.g20)), h: 16, pad: 8),
      _PC('type badge (media_parts)', 56, _left(const TypeBadge(Kind.video)), h: 16, pad: 8, ground: N.g26),
      _PC('time pill (media_parts)', 56, _left(const TimePill('12.0s')), h: 16, pad: 8, ground: N.g26),
    ]),
  ]);
}
