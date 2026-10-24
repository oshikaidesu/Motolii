// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

// ---- B9 swap
WidgetbookUseCase _partsSwapChips() => _parts('chips, tags, keys, buttons', 'B9-a small parts: the chip (22 px), the tag (16 px), the key mark and the 24 px button, each in every state it has.', [
      ('CHIP · filter, sort, action', [
        _Cell('rest', 96, 22, const _Chip('All', n: 63)),
        _Cell('hover', 96, 22, const _Chip('All', n: 63, hover: true)),
        _Cell('pressed', 96, 22, const _Chip('All', n: 63, pressed: true)),
        _Cell('keyboard focus', 96, 22, const _Chip('All', n: 63, focus: true)),
        _Cell('on (selected)', 96, 22, const _Chip('All', n: 63, on: true)),
        _Cell('on + hover', 96, 22, const _Chip('All', n: 63, on: true, hover: true)),
        _Cell('disabled', 96, 22, const _Chip('All', n: 63, enabled: false)),
        _Cell('long name, rest', 96, 22, const _Chip('Blur family', n: 7)),
        _Cell('long name, on', 96, 22, const _Chip('Blur family', n: 7, on: true)),
        _Cell('no count (sort), rest', 96, 22, const _Chip('Name')),
        _Cell('no count, on', 96, 22, const _Chip('Name', on: true)),
        _Cell('action chip', 96, 22, const _Chip('Show all')),
      ]),
      ('TAG · 16 px', [
        _Cell('plain', 120, 16, const _Tag('ON LAYER')),
        _Cell('plain, short', 120, 16, const _Tag('NOW')),
        _Cell('hi (accent edge)', 120, 16, const _Tag('REPLACE', hi: true)),
        _Cell('hi, long', 120, 16, const _Tag('PREVIEW · not written', hi: true)),
        _Cell('number', 120, 16, const _Tag('3')),
        _Cell('plain, long', 120, 16, const _Tag('Missing')),
      ]),
      ('KEY MARK · 16 px', [
        _Cell('letter', 40, 16, const _Kbd('Q')),
        _Cell('arrow', 40, 16, const _Kbd('↵')),
        _Cell('word', 40, 16, const _Kbd('Esc')),
        _Cell('chord', 40, 16, const _Kbd('⌘Z')),
      ]),
      ('BUTTON · 24 px', [
        _Cell('rest', 100, 24, const _Btn('Link', onTap: _noop)),
        _Cell('hover', 100, 24, const _Btn('Link', onTap: _noop, hover: true)),
        _Cell('pressed', 100, 24, const _Btn('Link', onTap: _noop, pressed: true)),
        _Cell('keyboard focus', 100, 24, const _Btn('Link', onTap: _noop, focus: true)),
        _Cell('disabled', 100, 24, const _Btn('Link')),
        _Cell('primary, rest', 100, 24, const _Btn('Keep', onTap: _noop, primary: true)),
        _Cell('primary, hover', 100, 24, const _Btn('Keep', onTap: _noop, primary: true, hover: true)),
        _Cell('primary, pressed', 100, 24, const _Btn('Keep', onTap: _noop, primary: true, pressed: true)),
        _Cell('button + key mark', 100, 24, const Row(mainAxisSize: MainAxisSize.min, children: [_Btn('Revert', onTap: _noop), SizedBox(width: 4), _Kbd('Esc')])),
      ]),
    ], absent: '$_noTag; error and empty states do not apply to them.');

WidgetbookUseCase _partsSwapRows() => _parts('rows, foot', 'B9-a list rows (28 px: gutter tick, thumb, English name + Japanese label, tags) and the foot (what just happened + key hints).', [
      ('EFFECT ROW · 28 px', [
        _Cell('rest', 280, 28, _fixed(280, _FxRow(_fxs[0]))),
        _Cell('hover', 280, 28, _fixed(280, _FxRow(_fxs[0], hover: true))),
        _Cell('selected', 280, 28, _fixed(280, _FxRow(_fxs[0], sel: true))),
        _Cell('selected + ON LAYER tag', 280, 28, _fixed(280, _FxRow(_fxs[1], sel: true, tags: const [_Tag('ON LAYER')]))),
        _Cell('hover + ON LAYER tag', 280, 28, _fixed(280, _FxRow(_fxs[1], hover: true, tags: const [_Tag('ON LAYER')]))),
        _Cell('selected + NOW tag (hi)', 280, 28, _fixed(280, _FxRow(_fxs[2], sel: true, tags: const [_Tag('NOW', hi: true)]))),
        _Cell('long JP + EN name (ellipsis)', 280, 28, _fixed(280, _FxRow(_fxs[31]))),
        _Cell('long name, narrow dock 232', 280, 28, _fixed(230, _FxRow(_fxs[31], sel: true))),
        _Cell('curve row (key effect)', 280, 28, _fixed(280, _FxRow(_fx('Ease Out Expo')))),
        _Cell('text effect', 280, 28, _fixed(280, _FxRow(_fx('Typewriter')))),
      ]),
      ('FOOT · what happened + key hints', [
        _Cell('nothing happened yet (takes no room)', 296, 8, _fixed(296, const _Foot(hints: [('Q', 'link / revert'), ('↑↓', 'next / prev')]))),
        _Cell('message, one line (hover lists the keys)', 296, 92, _fixed(296, const _Foot(msg: 'Linked · Effect 2', hints: [('↑↓', 'next / prev')]))),
        _Cell('message, two lines (then ellipsis)', 296, 92, _fixed(296, const _Foot(msg: 'Added Gaussian Blur to Layer 3 and applied the same Glow to the five other layers in this composition', hints: [('click', 'select')]))),
        _Cell('message only', 296, 92, _fixed(296, const _Foot(msg: 'Selected: dusk_ridge_001.jpg'))),
        _Cell('message + action', 296, 92, _fixed(296, _Foot(msg: 'Applied Glow to 5 layers', trailing: const _Btn('Undo', onTap: _noop)))),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring on rows (keys act on the panel), and disabled or error rows. The foot has one look.');

// ---- B9 cards, A/B
WidgetbookUseCase _partsCards() => _parts('cards, ghost, tray', 'B9-b/c/d small parts: the effect card in the drop panel and in the arrow-step panel, the carried ghost, the tray tile, the insertion line, the preview frame and the compare region.', [
      ('EFFECT CARD · drop panel', [
        _Cell('rest', 258, 84, _fixed(258, _CardFace(_Card(_fxs[0], .30, .55))), keepH: true),
        _Cell('a tile is over it: REPLACE', 258, 84, _fixed(258, _CardFace(_Card(_fxs[0], .30, .55), replace: true, kept: 1)), keepH: true),
        _Cell('long JP + EN name', 258, 84, _fixed(258, _CardFace(_Card(_fxs[31], .45, .2), number: 2)), keepH: true),
        _Cell('Alt held: add line under the card', 258, 92, _fixed(258, Column(mainAxisSize: MainAxisSize.min, children: [_CardFace(_Card(_fxs[8], .60, .35), number: 3), const _AddLine(true)])), keepH: true),
        _Cell('no Alt: line hidden', 258, 92, _fixed(258, Column(mainAxisSize: MainAxisSize.min, children: [_CardFace(_Card(_fxs[8], .60, .35), number: 3), const _AddLine(false)])), keepH: true),
      ]),
      ('EFFECT CARD · arrow-step panel', [
        _Cell('rest', 258, 92, _fixed(258, _StepFace(_fxs[0], (.30, .55))), keepH: true),
        _Cell('hover', 258, 92, _fixed(258, _StepFace(_fxs[0], (.30, .55), hover: true)), keepH: true),
        _Cell('selected (neighbours shown)', 258, 120, _fixed(258, _StepFace(_fxs[2], (.30, .55), sel: true)), keepH: true),
        _Cell('selected, neighbour hover', 258, 120, _fixed(258, _StepFace(_fxs[2], (.30, .55), sel: true, hoverUp: true)), keepH: true),
        _Cell('selected, first of its family', 258, 120, _fixed(258, _StepFace(_fxs[0], (.30, .55), sel: true)), keepH: true),
        _Cell('selected, last of its family', 258, 120, _fixed(258, _StepFace(_fxs[6], (.30, .55), sel: true)), keepH: true),
      ]),
      ('PARAMETER BAR · 20 px', [
        _Cell('30 %', 242, 20, _fixed(242, const _Param('Blurriness', .30))),
        _Cell('hot (target)', 242, 20, _fixed(242, const _Param('Blurriness', .30, hot: true))),
        _Cell('0 %', 242, 20, _fixed(242, const _Param('Amount', 0))),
        _Cell('100 %', 242, 20, _fixed(242, const _Param('Amount', 1))),
        _Cell('long label', 242, 20, _fixed(242, const _Param('Very long parameter name パラメータ名の長いもの', .5))),
      ]),
      ('TRAY TILE · GHOST', [
        _Cell('tray rest', 196, 26, _TrayChip(_fxs[0])),
        _Cell('tray hover', 196, 26, _TrayChip(_fxs[0], hover: true)),
        _Cell('tray picked up', 196, 26, _TrayChip(_fxs[0], on: true)),
        _Cell('ghost: replace', 196, 26, _Ghost('Gaussian Blur  = Replace', thumb: _Thumb(_fxs[0]))),
        _Cell('ghost: add', 196, 26, _Ghost('Glow  + Add', thumb: _Thumb(_fxs[24]))),
      ]),
      ('SECTION HEADER · 22 px', [
        _Cell('title + count + undo mark (shown only above 0)', 258, 22, _fixed(258, _Sec('Effects', n: '5', trailing: _undoMark(3)))),
        _Cell('title + preview tag', 258, 22, _fixed(258, const _Sec('Compare · Layer 3, Effect 2', trailing: _Tag('PREVIEW · not written', hi: true)))),
        _Cell('title only', 258, 22, _fixed(258, const _Sec('Compare · Layer 3, Effect 2'))),
        _Cell('long title (ellipsis)', 258, 22, _fixed(258, const _Sec('Candidates for the second effect on this layer', n: '63'))),
      ]),
      ('PREVIEW FRAME · compare region', [
        _Cell('no effect', 124, 78, const SizedBox(width: 124, height: 78, child: _Shot(null))),
        _Cell('blur family', 124, 78, SizedBox(width: 124, height: 78, child: _Shot(_fxs[0]))),
        _Cell('colour family', 124, 78, SizedBox(width: 124, height: 78, child: _Shot(_fxs[8]))),
        _Cell('other family', 124, 78, SizedBox(width: 124, height: 78, child: _Shot(_fxs[24]))),
        _Cell('closed: now beside an empty slot', 258, 78, _fixed(258, SizedBox(height: 52, child: Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Row(children: [Expanded(child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Shot(_fxs[0]))), const SizedBox(width: 8), Expanded(child: CustomPaint(painter: DashedBox(N.g38), child: const SizedBox.expand()))]))))),
      ]),
    ], absent: 'a pressed look on cards (a click selects at once), a focus ring on cards (keys act on the panel), disabled cards and error cards. The "end of family" button is the only disabled part.');

// ---- B8 drop on layer
WidgetbookUseCase _partsDrop() => _parts('layer rows, faces, drop zones', 'B8 small parts: the 23 px layer row (two row bands), the Replace | Add faces, the dashed new-layer zone and the carried asset row.', [
      ('LAYER ROW · 23 px', [
        _Cell('band A', 300, 23, _fixed(300, _LayerRow(_seedLayers()[0], 0))),
        _Cell('band B', 300, 23, _fixed(300, _LayerRow(_seedLayers()[1], 1))),
        _Cell('hover', 300, 23, _fixed(300, _LayerRow(_seedLayers()[1], 1, hover: true))),
        _Cell('selected', 300, 23, _fixed(300, _LayerRow(_seedLayers()[3], 3, sel: true))),
        _Cell('no keys', 300, 23, _fixed(300, _LayerRow(_seedLayers()[2], 2))),
        _Cell('long name (ellipsis)', 300, 23, _fixed(300, _LayerRow(_seedLayers()[11], 11))),
        _Cell('narrow dock 232, source hidden', 300, 23, _fixed(230, _LayerRow(_seedLayers()[11], 11, sel: true, showSrc: false))),
      ]),
      ('FACES · 48 px', [
        _Cell('neither lit', 240, 48, SizedBox(width: 240, height: 48, child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: const [Expanded(child: _Face('Replace', '絵だけ差し替え', false)), Expanded(child: _Face('Add', '新規 layer', false))]))),
        _Cell('Replace lit', 240, 48, SizedBox(width: 240, height: 48, child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: const [Expanded(child: _Face('Replace', '絵だけ差し替え', true)), Expanded(child: _Face('Add', '新規 layer', false))]))),
        _Cell('Add lit', 240, 48, SizedBox(width: 240, height: 48, child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: const [Expanded(child: _Face('Replace', '絵だけ差し替え', false)), Expanded(child: _Face('Add', '新規 layer', true))]))),
      ]),
      ('DROP ZONE · 38 px', [
        _Cell('idle', 296, 38, const SizedBox(width: 296, height: 38, child: _DropZone('new layer on top'))),
        _Cell('carrying, over it', 296, 38, const SizedBox(width: 296, height: 38, child: _DropZone('New layer', on: true))),
      ]),
      ('CARRIED ASSET ROW · 28 px', [
        _Cell('rest', 280, 28, _fixed(280, _AssetRow(_dropAssets[0]))),
        _Cell('hover', 280, 28, _fixed(280, _AssetRow(_dropAssets[0], hover: true))),
        _Cell('selected', 280, 28, _fixed(280, _AssetRow(_dropAssets[0], sel: true))),
        _Cell('JP name, video clock', 280, 28, _fixed(280, _AssetRow(_dropAssets[1]))),
        _Cell('image (kind badge word)', 280, 28, _fixed(280, _AssetRow(_dropAssets[3]))),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring (keys act on the panel), and disabled rows (a layer that refuses a drop says so on the row in B8-c, it is not a row state). Error does not apply.');

// ---- B7 click grammar
WidgetbookUseCase _partsTiles() => _parts('tabs, tiles, target line', 'B7 small parts: the tab strip (full and narrow), the effect tile and the media tile at the 92 px grid minimum and at 140 px, and the target line.', [
      ('TABS · 24 px', [
        _Cell('Effects selected', 296, 28, _fixed(296, const IgnorePointer(child: _Tabs(2)))),
        _Cell('Media selected', 296, 28, _fixed(296, const IgnorePointer(child: _Tabs(3)))),
        _Cell('narrow dock 232 (short names)', 296, 28, _fixed(232, const IgnorePointer(child: _Tabs(2)))),
      ]),
      ('TARGET LINE · 30 px', [
        _Cell('one layer', 296, 30, _fixed(296, const _TargetLine(_Sel.text))),
        _Cell('several layers', 296, 30, _fixed(296, const _TargetLine(_Sel.many))),
        _Cell('keys', 296, 30, _fixed(296, const _TargetLine(_Sel.key))),
        _Cell('nothing selected', 296, 30, _fixed(296, const _TargetLine(_Sel.none))),
      ]),
      ('EFFECT TILE · 92 px', [
        _Cell('rest', 92, 88, _FxTile(_fxs[0], 92)),
        _Cell('hover', 92, 88, _FxTile(_fxs[0], 92, hover: true)),
        _Cell('selected', 92, 88, _FxTile(_fxs[0], 92, sel: true)),
        _Cell('dim + reason', 92, 88, _FxTile(_fx('Typewriter'), 92, dim: true, reason: 'Text only')),
        _Cell('dim + reason (Key only)', 92, 88, _FxTile(_fx('Bounce'), 92, dim: true, reason: 'Key only')),
        _Cell('long EN + JP name', 92, 88, _FxTile(_fxs[31], 92)),
        _Cell('key tile (curve)', 92, 88, _FxTile(_fx('Elastic'), 92)),
      ]),
      ('MEDIA TILE · 92 px', [
        _Cell('image, rest', 92, 88, _MediaTile(_asset(Kind.image), 92)),
        _Cell('image, hover', 92, 88, _MediaTile(_asset(Kind.image), 92, hover: true)),
        _Cell('image, selected', 92, 88, _MediaTile(_asset(Kind.image), 92, sel: true)),
        _Cell('video: type badge + time pill', 92, 88, _MediaTile(_mediaShelf[7], 92)),
        _Cell('video, selected', 92, 88, _MediaTile(_mediaShelf[7], 92, sel: true)),
        _Cell('long EN name + 5:12', 92, 88, _MediaTile(_mediaShelf[8], 92)),
        _Cell('long JP name', 92, 88, _MediaTile(_mediaShelf[9], 92)),
        _Cell('audio', 92, 88, _MediaTile(_asset(Kind.audio), 92)),
      ]),
      ('TILES · 140 px', [
        _Cell('effect, rest', 140, 126, _FxTile(_fxs[0], 140)),
        _Cell('effect, selected', 140, 126, _FxTile(_fxs[0], 140, sel: true)),
        _Cell('media, rest', 140, 126, _MediaTile(_mediaShelf[7], 140)),
        _Cell('media, selected', 140, 126, _MediaTile(_mediaShelf[7], 140, sel: true)),
      ]),
    ], absent: 'a pressed look on tiles (a click selects at once or uses at once, by the case), a focus ring on tiles (keys act on the panel), and disabled tiles (a tile that does not fit is the dim tile of B13). The tab strip is a prop here and has its own sheet in the Inputs set.');

// ---- B10 favourites
WidgetbookUseCase _partsFav() => _parts('collections, tagged rows, recent', 'B10 small parts: the numbered collection button, effect rows that carry collection marks, a star, the Recent thumb and the empty-collection card.', [
      ('COLLECTION BUTTON · 24 px', [
        _Cell('rest', 92, 24, _fixed(92, const _CollChip(1, 'Titles', 12))),
        _Cell('hover', 92, 24, _fixed(92, const _CollChip(1, 'Titles', 12, hover: true))),
        _Cell('on (filter)', 92, 24, _fixed(92, const _CollChip(1, 'Titles', 12, on: true))),
        _Cell('empty (count 0)', 92, 24, _fixed(92, const _CollChip(9, 'Scratch', 0))),
        _Cell('long JP + EN name', 92, 24, _fixed(92, const _CollChip(4, 'Wipes 切替 長い名前', 7))),
        _Cell('3 columns (panel 280 and up)', 92, 24, _fixed(84, const _CollChip(6, 'Looks ルック', 128))),
      ]),
      ('ROWS WITH COLLECTION MARKS · 28 px', [
        _Cell('one number', 280, 28, _fixed(280, _FxRow(_fxs[24], tags: const [_Tag('1')]))),
        _Cell('three numbers + more', 280, 28, _fixed(280, _FxRow(_fxs[24], hover: true, tags: const [_Tag('1'), _Tag('4'), _Tag('7'), _Tag('…')]))),
        _Cell('selected, use count', 280, 28, _fixed(280, _FxRow(_fxs[24], sel: true, tags: [Text('12×', style: _num())]))),
        _Cell('long name + tags', 280, 28, _fixed(280, _FxRow(_fxs[31], tags: const [_Tag('2'), _Tag('5')]))),
        _Cell('star off / on', 280, 28, _fixed(280, _FxRow(_fxs[24], tags: [StarButton(on: false, size: 14), StarButton(on: true, size: 14)]))),
      ]),
      ('RECENT THUMB', [
        _Cell('recent', 66, 62, SizedBox(width: 66, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(height: 38, child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Thumb(_fxs[0]))), const SizedBox(height: 4), _t(_fxs[0].en, T.label(N.g76))]))),
        _Cell('long name (ellipsis)', 66, 62, SizedBox(width: 66, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(height: 38, child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Thumb(_fxs[31]))), const SizedBox(height: 4), _t(_fxs[31].en, T.label(N.g76))]))),
      ]),
      ('SECTION HEADER · 22 px', [
        _Cell('section + action chip', 296, 22, _fixed(296, _Sec('Collections', trailing: const _Chip('Show all')))),
        _Cell('section + count', 296, 22, _fixed(296, const _Sec('Recent', n: '12'))),
      ]),
      ('EMPTY COLLECTION CARD', [
        _Cell('empty', 296, 124, _fixed(296, _empty('9 Scratch はまだ空です', ''))),
      ]),
    ], absent: 'a pressed look on the collection button (a press switches the filter on at once, which is the "on" look), a focus ring on rows and buttons (keys act on the panel), and disabled or error parts. An empty collection is drawn as the empty card.');

// ---- B11 own presets
WidgetbookUseCase _partsMine() => _parts('Mine rows, bars, zone, name box', 'B11 small parts: the Mine row for each kind, the history row, the name box, the parameter bar being dragged, the save zone and the empty-shelf card.', [
      ('MINE ROW · 28 px', [
        _Cell('rest (kind = the tab: no tag)', 280, 28, _fixed(280, _mineRow(_seedMine()[0]))),
        _Cell('hover', 280, 28, _fixed(280, _mineRow(_seedMine()[0], hover: true))),
        _Cell('selected (the one being edited)', 280, 28, _fixed(280, _mineRow(_seedMine()[0], sel: true))),
        _Cell('kind: Curve', 280, 28, _fixed(280, _mineRow(_seedMine()[1]))),
        _Cell('kind: Colour, JP + EN', 280, 28, _fixed(280, _mineRow(_seedMine()[2]))),
        _Cell('kind: Layer + scope word', 280, 28, _fixed(280, _mineRow(_seedMine()[3]))),
        _Cell('long name (ellipsis)', 280, 28, _fixed(280, _mineRow(_Mine('Interview_Tanaka-san_Cam-B_take-07 色補正セット final v2', 'Curve', 'This project')))),
      ]),
      ('HISTORY ROW · 36 px', [
        _Cell('rest', 296, 36, _fixed(296, const _HistRow('14:02:11', 'Glow 40 · Blur 12 · Levels'))),
        _Cell('hover', 296, 36, _fixed(296, const _HistRow('14:02:11', 'Glow 40 · Blur 12 · Levels', hover: true))),
        _Cell('empty state of a layer', 296, 36, _fixed(296, const _HistRow('13:55:09', '(empty)'))),
        _Cell('long description (ellipsis)', 296, 36, _fixed(296, const _HistRow('13:58:02', 'Glow 35 · Blur 10 · Levels · Tint #4781E5 · Echo 3'))),
      ]),
      ('NAME BOX · 24 px', [
        _Cell('focused (selected text)', 280, 24, _fixed(280, _NameEdit(initial: 'Glow 40% + Levels', onDone: (_) {}, autofocus: false))),
      ]),
      ('PARAMETER BAR (edited copy) · 20 px', [
        _Cell('value 30 %', 242, 20, _fixed(242, const _Param('Intensity', .30, hot: true))),
        _Cell('value 80 %', 242, 20, _fixed(242, const _Param('Radius', .80, hot: true))),
        _Cell('untouched (not hot)', 242, 20, _fixed(242, const _Param('Radius', .80))),
      ]),
      ('SAVE ZONE · EMPTY SHELF', [
        _Cell('zone idle', 296, 128, const SizedBox(width: 296, height: 38, child: _DropZone('carry here to save'))),
        _Cell('zone, carrying over it', 296, 128, const SizedBox(width: 296, height: 38, child: _DropZone('carry here to save', on: true))),
        _Cell('empty shelf card', 296, 128, _fixed(296, _empty('自分の棚はまだ空です', '')), ground: N.g07),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring on rows (keys act on the panel; only the name box takes focus), and disabled or error rows.');

// ---- B12 library
WidgetbookUseCase _partsCells() => _parts('thumbnails, cells, list rows', 'B12-a small parts: the grey thumbnail shape of every kind and family, the asset cell and the 28 px list row, in every state.', [
      ('THUMBNAIL SHAPE · one per kind and family', [
        for (final (cap, sh) in const [('image', _Sh.image), ('video', _Sh.video), ('audio', _Sh.audio), ('3D', _Sh.model), ('360°', _Sh.pano), ('blur', _Sh.blur), ('colour', _Sh.color), ('distort', _Sh.distort), ('stylize', _Sh.stylize), ('time', _Sh.time), ('generate', _Sh.generate), ('transition', _Sh.wipe), ('text', _Sh.text)])
          _Cell(cap, 92, 57, SizedBox(width: 92, height: 57, child: ClipRRect(borderRadius: BorderRadius.circular(6), child: CustomPaint(painter: _Ink(sh, 3), size: Size.infinite)))),
        _Cell('ease (key effect)', 92, 57, SizedBox(width: 92, height: 57, child: ClipRRect(borderRadius: BorderRadius.circular(6), child: _Thumb(_fx('Elastic'))))),
      ]),
      ('ASSET CELL · 92 px', [
        _Cell('image, rest', 92, 88, _AssetCell(_asset(Kind.image), 92)),
        _Cell('hover', 92, 88, _AssetCell(_asset(Kind.image), 92, hover: true)),
        _Cell('selected', 92, 88, _AssetCell(_asset(Kind.image), 92, sel: true)),
        _Cell('video: badge', 92, 88, _AssetCell(_mediaShelf[7], 92)),
        _Cell('audio', 92, 88, _AssetCell(_asset(Kind.audio), 92)),
        _Cell('long JP + EN name', 92, 88, _AssetCell(_mediaShelf[8], 92)),
        _Cell('long JP name', 92, 88, _AssetCell(_mediaShelf[9], 92)),
        _Cell('thumbnail only (cell under 70)', 92, 88, _AssetCell(_asset(Kind.video), 56, names: false)),
      ]),
      ('ASSET CELL · 140 px', [
        _Cell('rest', 140, 120, _AssetCell(_mediaShelf[7], 140)),
        _Cell('selected', 140, 120, _AssetCell(_mediaShelf[7], 140, sel: true)),
      ]),
      ('LIST ROW · 28 px', [
        _Cell('rest', 280, 28, _fixed(280, _AssetRow(_asset(Kind.image)))),
        _Cell('hover', 280, 28, _fixed(280, _AssetRow(_asset(Kind.image), hover: true))),
        _Cell('selected', 280, 28, _fixed(280, _AssetRow(_asset(Kind.image), sel: true))),
        _Cell('video clock', 280, 28, _fixed(280, _AssetRow(_mediaShelf[7]))),
        _Cell('long EN name (ellipsis)', 280, 28, _fixed(280, _AssetRow(_mediaShelf[8]))),
        _Cell('long JP name, selected', 280, 28, _fixed(280, _AssetRow(_mediaShelf[9], sel: true))),
        _Cell('audio', 280, 28, _fixed(280, _AssetRow(_asset(Kind.audio)))),
      ]),
    ], absent: 'a pressed look on cells and rows (a click selects at once), a focus ring (keys act on the panel), and disabled or error cells (an unreadable folder is the error card in the next sheet).');

WidgetbookUseCase _partsLibStates() => _parts('search, controls, states', 'B12-a small parts: the search field, view switch, size slider and the count header; then the loading skeleton, the no-results card and the unreadable-folder card.', [
      ('SEARCH FIELD · 28 px', [
        _Cell('empty (hint)', 296, 28, _fixed(296, const TextBox(text: '', hint: 'Search items', look: Look.quiet, clearable: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('typed, clear shown', 296, 28, _fixed(296, const TextBox(text: 'hypno', hint: 'Search items', look: Look.quiet, clearable: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('focus ring', 296, 28, _fixed(296, const TextBox(text: 'hyp', hint: 'Search items', look: Look.quiet, clearable: true, ring: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('error', 296, 28, _fixed(296, const TextBox(text: '[unclosed', hint: 'Search items', look: Look.quiet, invalid: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('disabled (offline)', 296, 28, _fixed(296, const TextBox(text: '', hint: 'Search items', look: Look.quiet, enabled: false, leading: nav.Glyph(nav.G.search, color: N.g76)))),
      ]),
      ('VIEW SWITCH · SIZE SLIDER · 24 px', [
        _Cell('view: Grid', 140, 24, IgnorePointer(child: Segmented(items: const ['Grid', 'List'], index: 0, onChanged: (_) {}))),
        _Cell('view: List', 140, 24, IgnorePointer(child: Segmented(items: const ['Grid', 'List'], index: 1, onChanged: (_) {}))),
        _Cell('size slider (grid only)', 140, 24, Row(mainAxisSize: MainAxisSize.min, children: [Text('size', style: T.label(N.g76)), const SizedBox(width: 4), MiniSlider(value: 84, min: 44, max: 140, width: 92, onChanged: (_) {})])),
      ]),
      ('COUNT HEADER · 22 px', [
        _Cell('all items', 296, 22, _fixed(296, const _Sec('Items', n: '5,000'))),
        _Cell('filtered', 296, 22, _fixed(296, const _Sec('Items', n: '37 of 5,000'))),
        _Cell('loading', 296, 22, _fixed(296, const _Sec('Items', n: 'reading…'))),
        _Cell('error', 296, 22, _fixed(296, const _Sec('Items', n: 'offline'))),
      ]),
      ('LOADING · 92 px', [
        _Cell('skeleton (static)', 92, 88, const _Skel(92)),
        _Cell('skeleton (pulsing, as shown)', 92, 88, const _Pulse(child: _Skel(92))),
        _Cell('asset cell for comparison', 92, 88, _AssetCell(_asset(Kind.image), 92)),
      ]),
      ('EMPTY · ERROR', [
        _Cell('no results', 296, 168, _fixed(296, _empty('"hypno" に一致する素材はありません', '', action: const _Btn('Clear search', onTap: _noop)))),
        _Cell('folder unreadable', 296, 168, _fixed(296, _empty('このフォルダを読めません', 'Permission denied\n/Volumes/RAID_A/Footage/2026/Day3_Shoot', action: const Row(mainAxisSize: MainAxisSize.min, children: [_Btn('Retry', onTap: _noop), SizedBox(width: 6), _Btn('Locate…', onTap: _noop)])))),
      ]),
    ], absent: 'a pressed or focus look on the view switch and the slider (they are shared Inputs parts with their own sheets), and a disabled grid / list (the two faces are always both there).');

// ---- B12 similar, axis, live folder
WidgetbookUseCase _partsLive() => _parts('similar, band, live rows', 'B12-b/c/d small parts: the closeness badge, the Similar chip and the context menu, the range band on each axis, and the live-folder row in every state.', [
      ('CLOSENESS BADGE · 92 px', [
        _Cell('source (selected)', 92, 88, _AssetCell(_asset(Kind.image), 92, sel: true, badge: 'source')),
        _Cell('92 %', 92, 88, _AssetCell(_mediaShelf[7], 92, badge: '92%')),
        _Cell('hover + 41 %', 92, 88, _AssetCell(_mediaShelf[9], 92, hover: true, badge: '41%')),
        _Cell('3 % (floor)', 92, 88, _AssetCell(_asset(Kind.audio), 92, badge: '3%')),
      ]),
      ('SIMILAR CHIP · 24 px', [
        _Cell('rest', 200, 24, const _SimChip('dusk_ridge_001.jpg', 200)),
        _Cell('hover', 200, 24, const _SimChip('dusk_ridge_001.jpg', 200, hover: true)),
        _Cell('long name (ellipsis)', 200, 24, const _SimChip('Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', 200)),
      ]),
      ('CONTEXT MENU', [
        _Cell('one live item, three disabled', 168, 120, _CtxMenu(onFind: () {})),
      ]),
      ('RANGE BAND · 112 px', [
        _Cell('colour axis', 296, 112, const _Band(hist: _demoHist, axis: 0, bw: 296)),
        _Cell('duration axis, a range kept', 296, 112, const _Band(hist: _demoHist, axis: 1, bw: 296, r: (.3, .7))),
        _Cell('type axis', 296, 112, const _Band(hist: _demoHist, axis: 2, bw: 296)),
      ]),
      ('LIVE FOLDER ROW · 28 px', [
        _Cell('rest', 296, 28, _fixed(296, _LiveRow('A001_C003_0612_R2X5.mov', _dropAssets[0]))),
        _Cell('hover', 296, 28, _fixed(296, _LiveRow('A001_C003_0612_R2X5.mov', _dropAssets[0], hover: true))),
        _Cell('selected', 296, 28, _fixed(296, _LiveRow('A001_C003_0612_R2X5.mov', _dropAssets[0], sel: true))),
        _Cell('NEW (4 s)', 296, 28, _fixed(296, _LiveRow('new_clip_001.mov', _dropAssets[0], fresh: true))),
        _Cell('missing on disk + Locate', 296, 28, _fixed(296, _LiveRow('slate_scene12.png', _dropAssets[2], onDisk: false))),
        _Cell('drive offline', 296, 28, _fixed(296, _LiveRow('メイキング_001.mov', _dropAssets[1], offline: true))),
        _Cell('long name', 296, 28, _fixed(296, _LiveRow('Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', _dropAssets[0]))),
      ]),
      ('PLACE TAG · 16 px', [
        _Cell('LIVE', 120, 16, const _Tag('LIVE', hi: true)),
        _Cell('OFFLINE', 120, 16, const _Tag('OFFLINE')),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring (keys act on the panel), and disabled cells. The disabled items of the context menu are drawn; the Locate button has the button states in the first sheet.');

// ---- B13 context filter
WidgetbookUseCase _partsContext() => _parts('tiles, pick card, lines', 'B13 small parts: the tile that fits and the tile that does not (dim + reason chip), the layer card on the stand-in Stage, the selection chips and the shown / hidden line.', [
      ('TILE: FITS vs DOES NOT FIT · 92 px', [
        _Cell('fits, rest', 92, 88, _FxTile(_fxs[0], 92)),
        _Cell('fits, hover', 92, 88, _FxTile(_fxs[0], 92, hover: true)),
        _Cell('dim: reason word', 92, 88, _FxTile(_fx('Typewriter'), 92, dim: true, reason: 'Text only')),
        _Cell('dim: Key を選ぶ', 92, 88, _FxTile(_fx('Bounce'), 92, dim: true, reason: 'Key を選ぶ')),
        _Cell('dim: Layer を選ぶ', 92, 88, _FxTile(_fxs[0], 92, dim: true, reason: 'Layer を選ぶ')),
        _Cell('dim, long name', 92, 88, _FxTile(_fxs[31], 92, dim: true, reason: 'Text only')),
      ]),
      ('LAYER CARD ON THE STAND-IN STAGE · 44 px', [
        _Cell('fits (blinks)', 70, 44, _fixed(70, const _PickCard('タイトル', 'Text', ok: true))),
        _Cell('fits, hover', 70, 44, _fixed(70, const _PickCard('タイトル', 'Text', ok: true, hover: true))),
        _Cell('does not fit', 70, 44, _fixed(70, const _PickCard('Logo Mark', 'Shape', ok: false))),
        _Cell('does not fit, hover (no change)', 70, 44, _fixed(70, const _PickCard('Logo Mark', 'Shape', ok: false, hover: true))),
        _Cell('long name', 70, 44, _fixed(70, const _PickCard('BG Gradient 長い名前', 'Solid', ok: true))),
      ]),
      ('SELECTION CHIPS · 22 px', [
        _Cell('Text on', 244, 22, const Row(mainAxisSize: MainAxisSize.min, children: [_Chip('None'), SizedBox(width: 4), _Chip('Shape'), SizedBox(width: 4), _Chip('Text', on: true), SizedBox(width: 4), _Chip('Key'), SizedBox(width: 4), _Chip('Mixed')])),
        _Cell('Key hover', 244, 22, const Row(mainAxisSize: MainAxisSize.min, children: [_Chip('None'), SizedBox(width: 4), _Chip('Shape'), SizedBox(width: 4), _Chip('Text'), SizedBox(width: 4), _Chip('Key', hover: true), SizedBox(width: 4), _Chip('Mixed')])),
        _Cell('None on', 244, 22, const Row(mainAxisSize: MainAxisSize.min, children: [_Chip('None', on: true), SizedBox(width: 4), _Chip('Shape'), SizedBox(width: 4), _Chip('Text'), SizedBox(width: 4), _Chip('Key'), SizedBox(width: 4), _Chip('Mixed')])),
      ]),
      ('COUNT LINE · SECTION · 22 px', [
        _Cell('shown / hidden + action', 296, 22, _fixed(296, Row(children: [Text('54 shown', style: T.label(N.g91)), Text(' · ', style: T.label(N.g76)), Text('9 hidden', style: T.label(N.g76)), const Spacer(), const _Chip('Show all')]))),
        _Cell('action on', 296, 22, _fixed(296, Row(children: [Text('54 shown', style: T.label(N.g91)), Text(' · ', style: T.label(N.g76)), Text('9 hidden', style: T.label(N.g76)), const Spacer(), const _Chip('Hide misfits', on: true)]))),
        _Cell('fit / do not (dim case)', 296, 22, _fixed(296, _t('54 fit · 9 do not', T.label(N.g76)))),
        _Cell('section header', 296, 22, _fixed(296, const _Sec('Blur & Sharpen', n: '7'))),
        _Cell('nothing selected: no line is drawn', 296, 22, const SizedBox.shrink()),
      ]),
    ], absent: 'a pressed look on tiles (a click applies at once), a focus ring (keys act on the panel), and disabled tiles (the dim tile is the "does not fit" state). The error case is the reason chip on the dim tile.');

// ================================================================ the set

_Sel _selKnob(BuildContext c, _Sel init) => c.knobs.object.dropdown<_Sel>(label: 'Selection', options: _Sel.values, initialOption: init, labelBuilder: (s) => _selName(s));

/// Panel drafts of group B (Browser), released as nine small components in the recommended order. Each starts with its Parts @200% sheet(s), then the panels.
/// Every panel has a one-line story outside the frame and the two knobs Panel width and Panel height.
List<WidgetbookComponent> panelBrowserBSets() => [
      WidgetbookComponent(name: 'Browser B7 Click', useCases: [
        _partsTiles(),
        _uc('B7-a Click selects, double-click uses: Effects', 'B7-a Grammar', _Hab.habit, '単クリックは選ぶだけ、ダブルクリックか Enter で選択対象に使う。全棚で同じ。', (c, w) => _GrammarLab(w: w, mode: 0)),
        _uc('B7-a Click selects, double-click uses: Media', 'B7-a Grammar', _Hab.habit, 'The same rule on the Media shelf: double-click adds the file to the comp as a layer.', (c, w) => _GrammarLab(w: w, mode: 0, startTab: 1)),
        _uc('B7-b Two shelves, two meanings', 'B7-b Split meaning', _Hab.counter, '反例: Effects は単クリックで使い、Media は単クリックで選ぶ。契約が 1 行で言えない。', (c, w) => _GrammarLab(w: w, mode: 1)),
        _uc('B7-c Single click uses, undo 1 step', 'B7-c Click uses', _Hab.departs, '単クリックで選択対象に使い、undo は 1 手。対象が無ければ何も書かない。', (c, w) => _GrammarLab(w: w, mode: 2, target: _selKnob(c, _Sel.text))),
        _uc('B7-c Single click uses: no target', 'B7-c Click uses', _Hab.departs, 'Nothing is selected, so a click asks what to apply it to and writes nothing.', (c, w) => _GrammarLab(w: w, mode: 2, target: _Sel.none)),
        _uc('B7-d Browser shows its target', 'B7-d Target line', _Hab.addition, '対象は 1 行で見え、tile の click と Enter はその対象にだけ効く。', (c, w) => _GrammarLab(w: w, mode: 3, target: _selKnob(c, _Sel.text))),
        _uc('B7-d Browser shows its target: several layers', 'B7-d Target line', _Hab.addition, 'Five layers selected: the line says so before you act, and one click is one undo for all five.', (c, w) => _GrammarLab(w: w, mode: 3, target: _Sel.many)),
      ]),
      WidgetbookComponent(name: 'Browser B9 Swap', useCases: [
        _partsSwapChips(),
        _partsSwapRows(),
        _uc('B9-a Hot-swap: link mode (Q), at rest', 'B9-a Hot-swap', _Hab.habit, 'Q ties the Browser to the selected effect; ↑ ↓ swaps it in place; Esc goes back before Enter.', (c, w) => _HotSwap(w: w)),
        _uc('B9-a Hot-swap: linked, trying candidates', 'B9-a Hot-swap', _Hab.habit, 'While linked the card shows BEFORE and NOW, and history stays untouched until Enter.', (c, w) => _HotSwap(w: w, startLinked: true)),
        _uc('B9-a Hot-swap: layer has no effect yet', 'B9-a Hot-swap', _Hab.habit, 'The layer has no effect yet: Link is disabled, the card says why, and a double-click on a row adds one.', (c, w) => _HotSwap(w: w, mode: 1)),
        _uc('B9-a Hot-swap: layer locked', 'B9-a Hot-swap', _Hab.habit, 'The layer is locked: Link is disabled and the card says why in words.', (c, w) => _HotSwap(w: w, mode: 2)),
      ]),
      WidgetbookComponent(name: 'Browser B12 Library', useCases: [
        _partsCells(),
        _partsLibStates(),
        _uc('B12-a Thumbnails, many items', 'B12-a Grid ↔ list', _Hab.habit, 'List and thumbnails are one thing with two faces: the selection and the count stay. Knob: Items.', (c, w) {
          final n = c.knobs.object.dropdown<int>(label: 'Items', options: const [5000, 200, 5], initialOption: 5000, labelBuilder: (v) => _fmt(v));
          return _Library(key: ValueKey(n), w: w, n: n);
        }),
        _uc('B12-a List, 200 items', 'B12-a Grid ↔ list', _Hab.habit, 'The list face of the same panel: long Japanese and English names truncate, never wrap.', (c, w) => _Library(w: w, n: 200, view: 1)),
        _uc('B12-a Loading', 'B12-a Grid ↔ list', _Hab.habit, 'Skeleton tiles keep the layout, so nothing jumps when the thumbnails arrive.', (c, w) => _Library(w: w, n: 5000, state: 1)),
        _uc('B12-a No results', 'B12-a Grid ↔ list', _Hab.habit, 'Nothing matches: the card says what was searched and gives a one-click way out.', (c, w) => _Library(w: w, n: 5000, query: 'hypno')),
        _uc('B12-a Folder unreadable', 'B12-a Grid ↔ list', _Hab.habit, 'The folder cannot be read: the reason, the path and two next steps.', (c, w) => _Library(w: w, n: 5000, state: 2)),
      ]),
      WidgetbookComponent(name: 'Browser B13 Context', useCases: [
        _partsContext(),
        _uc('B13-a Misfits are not shown: Text layer', 'B13-a Hidden', _Hab.addition, '選択に合わない物は見せない。選択を変えると一覧が変わり、隠した件数を 1 行で言う。', (c, w) => _Context(w: w, mode: 0, sel: _selKnob(c, _Sel.text))),
        _uc('B13-a Misfits are not shown: Key', 'B13-a Hidden', _Hab.addition, 'With keys selected only the curve presets remain; Show all brings the rest back in one click.', (c, w) => _Context(w: w, mode: 0, sel: _selKnob(c, _Sel.key))),
        _uc('B13-a Misfits are not shown: nothing selected', 'B13-a Hidden', _Hab.addition, 'Nothing selected means everything is shown, so there is no count line.', (c, w) => _Context(w: w, mode: 0, sel: _selKnob(c, _Sel.none))),
        _uc('B13-b Dim, with a reason word', 'B13-b Dim + reason', _Hab.addition, '合わない tile は淡く残し、理由を tile の上に書く。押しても何も書かない。', (c, w) => _Context(w: w, mode: 1, sel: _selKnob(c, _Sel.shape))),
        _uc('B13-c Press a misfit, then pick a layer', 'B13-c Pick later', _Hab.addition, '合わない tile を押すと layer を選ぶ手順になり、選ぶと適用、Esc で何も書かない。', (c, w) => _Context(w: w, mode: 2, sel: _selKnob(c, _Sel.shape))),
      ]),
      WidgetbookComponent(name: 'Browser B10 Favourites', useCases: [
        _partsFav(),
        _uc('B10-a Numbered collections (1-9)', 'B10-a Collections', _Hab.habit, '選んで 1-9 で付け、もう一度で外し、0 で全部外す。集合は名前で絞り、Recent は自動。', (c, w) => _Collections(w: w)),
        _uc('B10-a Numbered collections: first use (empty)', 'B10-a Collections', _Hab.habit, 'First use: nine named, empty collections; the first number key press teaches itself.', (c, w) => _Collections(w: w, empty: true)),
        _uc('B10-b One star and automatic Recent / Frequent', 'B10-b Star + auto', _Hab.habit, '星は 1 種(F キー)。Recent と Frequent は使った分から自動で、手では触らない。', (c, w) => _StarAuto(w: w)),
        _uc('B10-b One star: Frequent view', 'B10-b Star + auto', _Hab.habit, 'Frequent is ordered by use count: double-click an effect in All and watch it climb.', (c, w) => _StarAuto(w: w, start: 3)),
        _uc('B10-b One star: nothing yet (fresh install)', 'B10-b Star + auto', _Hab.habit, 'Nothing yet (fresh install): each empty view says how it fills itself.', (c, w) => _StarAuto(w: w, start: 1, fresh: true)),
        _uc('B10-c A pile of what was used', 'B10-c Pile', _Hab.addition, '使った物は先頭に積まれ、同じ物は前へ動き、上限を超えると古い物が落ちる。Knob: Cap.', (c, w) {
          final cap = c.knobs.int.slider(label: 'Cap', initialValue: 12, min: 4, max: 12);
          return _Pile(key: ValueKey(cap), w: w, cap: cap);
        }),
      ]),
      WidgetbookComponent(name: 'Browser B9 Cards A-B', useCases: [
        _partsCards(),
        _uc('B9-b Drop on an effect card replaces (Alt adds)', 'B9-b Card drop', _Hab.habit, 'カードへ drop で置換(同名の値は持ち越し)、Alt で新規、Esc で取り消し。tray が Browser の代役。', (c, w) => _DropCards(w: w, altSim: c.knobs.boolean(label: 'Alt held (simulate)', initialValue: false)), w: 282),
        _uc('B9-c Card ↑↓ walks the family', 'B9-c Card step', _Hab.addition, '選んだカードで ↑↓ かホイール = 同 family の次/前に置換。1 回 = 1 undo。', (c, w) => _CardStep(w: w), w: 282),
        _uc('B9-d A/B two frames, open', 'B9-d A/B', _Hab.addition, '候補を選ぶと今の絵と 2 枠で並び、Keep で 1 回書き、Close で何も書かず閉じる。', (c, w) => _Ab(w: w)),
        _uc('B9-d A/B two frames, closed', 'B9-d A/B', _Hab.addition, 'Closed: the compare region is one quiet line until a candidate is picked.', (c, w) => _Ab(w: w, startOpen: false)),
      ]),
      WidgetbookComponent(name: 'Browser B8 Drop', useCases: [
        _partsDrop(),
        _uc('B8-a Add by default, Alt replaces', 'B8-a Drop on layer', _Hab.habit, 'layer の上でも drop は追加、Alt で置換(bar と key は残る)。hover で結果が先に見える。', (c, w) => _DropLab(w: w, mode: 0, altSim: c.knobs.boolean(label: 'Alt held (simulate)', initialValue: false))),
        _uc('B8-b Replace by default, Alt adds', 'B8-b Drop on layer', _Hab.departs, 'layer への drop が絵の差し替え、Alt で追加、空き地は新規。D-2 は採らなかった(上書き事故)。', (c, w) => _DropLab(w: w, mode: 1, altSim: c.knobs.boolean(label: 'Alt held (simulate)', initialValue: false))),
        _uc('B8-c Layers refuse; Source field replaces', 'B8-c Refuse', _Hab.departs, 'layer は drop を受けず、Inspector の Source 欄が唯一の置換口。空き地は新規。', (c, w) => _DropLab(w: w, mode: 2)),
        _uc('B8-d Hold still: Replace | Add faces', 'B8-d Two faces', _Hab.addition, 'hover が止まると行の下に 2 面が開き、面の上で離した方を実行。面の外で離すと取り消し。', (c, w) => _DropLab(w: w, mode: 3, dwell: c.knobs.object.dropdown<int>(label: 'Dwell (ms)', options: const [1000, 500, 250], initialOption: 1000, labelBuilder: (v) => '$v'))),
      ]),
      WidgetbookComponent(name: 'Browser B11 Mine', useCases: [
        _partsMine(),
        _uc('B11-a Carry onto Mine to save', 'B11-a Save by drop', _Hab.habit, 'drop = 保存(元は変わらない)。名前欄がその場で開き、保存先は 1 語で選ぶ。', (c, w) => _SaveToShelf(w: w)),
        _uc('B11-a Carry onto Mine: empty shelf', 'B11-a Save by drop', _Hab.habit, 'Mine is empty: the card says what goes here and how to put it there.', (c, w) => _SaveToShelf(w: w, empty: true)),
        _uc('B11-b Editing a bundled preset makes a copy', 'B11-b Copy on edit', _Hab.addition, '同梱 preset は書き換わらず、値に触れた瞬間に複製が Mine に増える。', (c, w) => _CopyOnEdit(w: w)),
        _uc('B11-c History snapshots; name to keep', 'B11-c Snapshots', _Hab.addition, '履歴は上限付きで自動に取り、名前を付けた物だけが残る。layer は右の箱から動かす。', (c, w) => _Snaps(w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B12 Similar Axis Live', useCases: [
        _partsLive(),
        _uc('B12-b Find similar: ready', 'B12-b Similar', _Hab.addition, '似た物を探すと chip になり、外すと全件に戻る。Right-click a tile or press S.', (c, w) => _Similar(w: w)),
        _uc('B12-b Find similar: ranked', 'B12-b Similar', _Hab.addition, 'Ranked by a made-up 8-number description: the source stays first and each tile shows a closeness %.', (c, w) => _Similar(w: w, start: 6)),
        _uc('B12-c One-axis band: colour', 'B12-c Axis strip', _Hab.departs, '軸を 1 つ選び、帯を drag して範囲で絞る。400 件。', (c, w) => _AxisStrip(w: w)),
        _uc('B12-c One-axis band: duration', 'B12-c Axis strip', _Hab.departs, 'The same band on a log duration axis: stills pile up on the left.', (c, w) => _AxisStrip(w: w, axis: 1)),
        _uc('B12-d A folder as a live source', 'B12-d Live folder', _Hab.addition, 'ファイルを置くと一覧に出て、消すと Missing の印で残る。ディスクは右の箱から動かす。', (c, w) => _LiveFolder(w: w)),
      ]),
    ];
