// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

class _FindAState extends State<_FindA> {
  late final _c = TextEditingController(text: widget.query);
  late final _n = FocusNode(onKeyEvent: _key);
  late final _root = FocusNode(onKeyEvent: _rootKey);
  final _sc = ScrollController();
  late int _tab = widget.tab;
  int _sel = 0;
  String _say = 'Layer 3 · Title';

  Iterable<It> get _pool => _tab < 0 ? kItems : kItems.where((i) => i.place == _places[_tab]);
  List<It> get _res => _match(_pool, _c.text);

  /// One place that is more than 80% of an all-places result is said once in the count line ("mostly effects"); only the exceptions keep their place word.
  String? _dom(List<It> r) => widget.grouped || _tab >= 0 ? null : _dominant(r, (i) => i.place);

  List<_Ent> _entries() {
    final r = _res, toks = _tokens(_c.text), dom = _dom(r);
    final out = <_Ent>[];
    var k = 0;
    _Ent row(It it) {
      final i = k++;
      // Grouped under a place heading the place is already said, so no label on the right. Across places the place stays on the right and a media row's second line never repeats its kind.
      return _Ent.row((sel) => _ItemRow(it, selected: sel, tokens: toks, right: widget.grouped || it.place == dom ? const SizedBox.shrink() : Text(it.place, style: T.micro(N.g76)), onTap: () => setState(() => _sel = i), onUse: () => _use(it)));
    }

    if (widget.grouped) {
      for (final p in _places) {
        final g = r.where((i) => i.place == p).toList();
        if (g.isEmpty) continue;
        out.add(_Ent.head(p, right: '${g.length}'));
        for (final it in g) {
          out.add(row(it));
        }
      }
    } else {
      for (final it in r) {
        out.add(row(it));
      }
    }
    return out;
  }

  void _use(It it) => setState(() => _say = 'Applied “${it.name}” to Layer 3');

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final es = _entries(), cnt = _rowCount(es);
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cnt == 0 ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, cnt - 1));
      final (t, b) = _entSpan(es, _sel);
      _revealAt(_sc, t, b);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter && cnt > 0) {
      final it = _res[_sel.clamp(0, _res.length - 1)];
      _use(it);
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s), cleared: () => _sel = 0);
  }

  KeyEventResult _rootKey(FocusNode n, KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.slash) {
      _n.requestFocus();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        focusNode: _root,
        autofocus: true,
        child: ListenableBuilder(
          listenable: _c,
          builder: (context, _) {
            final r = _res, es = _entries(), empty = r.isEmpty;
            final sg = empty ? _suggest(_c.text) : const <String>[];
            final dom = _dom(r), outside = _tab >= 0 ? _match(kItems, _c.text).length - r.length : 0;
            return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _pad(_Tabs(_places, _tab, (i) => setState(() {
                    _tab = i;
                    _sel = 0;
                  })), t: 10),
              const SizedBox(height: 8),
              _pad(_Search(controller: _c, node: _n, clearHover: widget.clearHover, onChanged: (_) => setState(() => _sel = 0))),
              _pad(
                Row(children: [
                  Expanded(child: Text(empty ? 'No results' : '${r.length} of ${_pool.length}  ·  ${_tab < 0 ? 'no tab on: all places' : _places[_tab]}${dom == null ? '' : '  ·  mostly ${dom.toLowerCase()}'}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
                ]),
                t: 8,
                b: 6,
              ),
              Expanded(
                child: empty
                    ? _Empty('Nothing matches “${_c.text}”', [
                        _tab < 0 ? 'Searched all 4 places, ${kItems.length} items.' : 'Searched only ${_places[_tab]}.',
                      ], actions: [
                        _Btn('Clear search', onTap: () => setState(() => _c.clear())),
                        if (_tab >= 0) _Btn('Search all places', onTap: () => setState(() => _tab = -1)),
                        for (final s in sg) _Btn('Did you mean “$s”?', onTap: () => setState(() => _c.text = s)),
                      ])
                    : _Results(
                        list: _FlatList(entries: es, sel: _sel, sc: _sc),
                        rows: r.length,
                        listH: _entH(es),
                        more: outside,
                        where: 'in other places',
                        foot: outside > 0,
                      ),
              ),
              _Status(_say),
            ]);
          },
        ),
      );
}

// ================================================================================ B3-b  Tab menu (type-ahead at the cursor)

const _fams = ['All', 'Blur', 'Color', 'Glow', 'Distort', 'Stylize', 'Time', 'Title', 'Ease'];

class _TabMenu extends StatefulWidget {
  const _TabMenu({super.key, required this.w, this.target = 0, this.text = ''});
  final double w;
  final int target; // 0 layer, 1 none, 2 key
  final String text;
  @override
  State<_TabMenu> createState() => _TabMenuState();
}

class _TabMenuState extends State<_TabMenu> {
  late final _c = TextEditingController(text: widget.text);
  late final _n = FocusNode(onKeyEvent: _key);
  late final _root = FocusNode(onKeyEvent: _rootKey);
  bool _open = true;
  int _fam = 0, _sel = 0;
  String? _done;

  String get _tgt => const ['Layer 3 · Title', '', 'Key · Position X'][widget.target];

  List<It> get _cands {
    var p = kItems.where((i) => i.place != 'Project' && (widget.target == 1 || (widget.target == 2 ? i.applies != 'layer' : i.applies != 'key')));
    if (_fam > 0) p = p.where((i) => i.cat == _fams[_fam]);
    final r = _match(p, _c.text);
    if (_c.text.trim().isEmpty) {
      r.sort((a, b) {
        final ra = a.recent < 0 ? 1 << 20 : a.recent, rb = b.recent < 0 ? 1 << 20 : b.recent;
        return ra != rb ? ra.compareTo(rb) : b.uses.compareTo(a.uses);
      });
    }
    return r;
  }

  void _close(String msg) {
    setState(() {
      _open = false;
      _done = msg;
    });
    _root.requestFocus();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final cs = _cands;
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.tab) {
      setState(() {
        _fam = (_fam + 1) % _fams.length;
        _sel = 0;
      });
      return KeyEventResult.handled;
    }
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cs.isEmpty ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, math.min(cs.length, 8) - 1));
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      if (widget.target == 1) {
        setState(() => _done = null);
        return KeyEventResult.handled; // stays open: the empty-target message is already visible
      }
      if (cs.isNotEmpty) _close('Applied “${cs[_sel.clamp(0, cs.length - 1)].name}” to $_tgt');
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      _close('Closed, nothing written');
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  KeyEventResult _rootKey(FocusNode n, KeyEvent e) {
    if (!_open && e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.tab) {
      setState(() {
        _open = true;
        _c.clear();
        _fam = 0;
        _sel = 0;
      });
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _root.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        focusNode: _root,
        autofocus: false,
        child: Stack(fit: StackFit.expand, children: [
          const Art(Asset('', Kind.image, 3)),
          const ColoredBox(color: Color(0x73131313)),
          // the stage fragment behind the menu: the selected layer with its name
          Positioned(left: 28, right: 28, top: 28, height: 104, child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: widget.target == 1 ? N.g44 : Role.selected), borderRadius: BorderRadius.circular(2)), child: Stack(children: [
            Align(alignment: Alignment.topLeft, child: Container(padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4), color: widget.target == 1 ? N.g26 : Role.selected, child: Text(widget.target == 1 ? 'nothing selected' : (widget.target == 2 ? 'Position X · key at 00:12' : 'Layer 3 · Title'), style: T.micro(N.g100)))),
            if (widget.target != 2) Center(child: Text('光の速さで', style: T.title().copyWith(fontSize: 22))),
          ]))),
          // The menu scrolls inside the frame (never a cut row); the closed card keeps the Tab key mark where the menu was.
          if (_open) Positioned(left: 12, right: 12, top: 144, bottom: 34, child: LayoutBuilder(builder: (context, box) => Align(alignment: Alignment.topCenter, child: SingleChildScrollView(child: _menu(box.maxHeight))))) else Positioned(left: 12, right: 12, top: 152, child: _closedCard()),
          Positioned(left: 0, right: 0, bottom: 0, child: _Status('', keys: _open ? const ['Esc'] : const [])),
        ]),
      );

  Widget _closedCard() => _ClosedCard(_done ?? 'Closed');

  /// [avail] is the height the frame leaves for the card: the rows shown are only as many as fit whole (28 px each), and the "+N more" line says the rest, so no row is ever cut.
  Widget _menu(double avail) {
    final noTarget = widget.target == 1;
    final cs = _cands, empty = _c.text.trim().isEmpty;
    final fit = ((avail - 146 - (noTarget ? 44 : 0)) / 28).floor().clamp(3, 8);
    final shown = cs.take(fit).toList();
    return Container(
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26), boxShadow: const [BoxShadow(color: Color(0x80000000), blurRadius: 18, offset: Offset(0, 6))]),
      child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Padding(padding: const EdgeInsets.fromLTRB(10, 10, 10, 6), child: _Search(controller: _c, node: _n, autofocus: true, slash: false, hint: noTarget ? 'Add effect (no target)' : 'Add effect to ${widget.target == 2 ? 'Position X' : 'Layer 3'}', onChanged: (_) => setState(() => _sel = 0), trailing: const _Kbd('Tab'))),
        SizedBox(
          height: 26,
          child: _FadeScroll(
            index: _fam,
            chevron: true,
            pad: const EdgeInsets.symmetric(horizontal: 10),
            children: [
              for (var i = 0; i < _fams.length; i++)
                _H(
                  onTap: () => setState(() {
                    _fam = i;
                    _sel = 0;
                    _n.requestFocus();
                  }),
                  builder: (context, h) => Center(child: _TabCell(_fams[i], on: i == _fam, hover: h)),
                ),
            ],
          ),
        ),
        if (noTarget)
          const Padding(padding: EdgeInsets.fromLTRB(10, 2, 10, 4), child: _Notice(_noTargetText)),
        _Sec(empty ? 'Recent' : (cs.isEmpty ? 'No match' : 'Matches · ${cs.length}'), pad: const EdgeInsets.fromLTRB(10, 6, 10, 2)),
        if (shown.isEmpty) Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 10), child: Text('Nothing in ${_fams[_fam]} matches “${_c.text}”', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(height: 1.35))),
        for (var i = 0; i < shown.length; i++)
          _H(
            onTap: () => setState(() => _sel = i),
            onDouble: () => _close('Applied “${shown[i].name}” to $_tgt'),
            builder: (context, h) => _MenuRow(shown[i].name, tokens: _tokens(_c.text), selected: i == _sel, hover: h, tail: i == _sel && !noTarget ? null : (empty && shown[i].recent >= 0 ? _ago(shown[i].recent) : (_fam > 0 ? '' : shown[i].cat))),
          ),
        if (cs.length > shown.length) _MoreLine('+${cs.length - shown.length} more'),
        const SizedBox(height: 6),
      ]),
    );
  }
}

const _noTargetText = 'No target selected';

/// A menu row, 28 px: tail = a quiet word on the right, '' = nothing (the family tab already says it), null = the Enter key mark (the row Enter would use).
class _MenuRow extends StatelessWidget {
  const _MenuRow(this.name, {this.tokens = const [], this.selected = false, this.hover = false, this.tail});
  final String name;
  final List<String> tokens;
  final bool selected, hover;
  final String? tail;
  @override
  Widget build(BuildContext context) => Container(
        height: 28,
        color: selected ? N.g20 : (hover ? N.g15 : null),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Row(children: [
              Expanded(child: _Hl(name, tokens, on: selected)),
              const SizedBox(width: 8),
              if (tail == null) const _Kbd('Enter') else if (tail!.isNotEmpty) Text(tail!, style: T.label(N.g76)),
            ]),
          ),
          _GutterTick(selected, height: 12),
        ]),
      );
}

/// What stays where the menu was after it closes: the Tab key mark and one line of what happened.
class _ClosedCard extends StatelessWidget {
  const _ClosedCard(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26)),
        child: Row(children: [const _Kbd('Tab'), const SizedBox(width: 8), Expanded(child: Text(text, style: T.name(N.g91).copyWith(height: 1.3)))]),
      );
}

// ================================================================================ B3-c  Prefix: one field, the first character picks the area

const _actions = [
  ('Apply to selection', '選択に適用', 'Enter'),
  ('Reset all effects on Layer 3', 'Layer 3 のエフェクトをリセット', 'Cmd Shift R'),
  ('Add keyframe at playhead', '現在位置にキーを打つ', 'K'),
  ('Align to composition center', 'コンポジションの中央に整列', 'Cmd Alt C'),
  ('Open Export…', '書き出し…', 'Cmd E'),
  ('Toggle Inspector', 'Inspector の表示切替', 'Cmd 2'),
  ('Duplicate layer', 'レイヤーを複製', 'Cmd D'),
  ('Solo layer', 'ソロ', 'S'),
  ('Pre-compose…', 'プリコンポーズ…', 'Cmd Shift C'),
  ('Undo', '取り消し', 'Cmd Z'),
];

const _layers = [
  ('Title', 'Text'), ('キャッチコピー', 'Text'), ('BG Sky', 'Image'), ('Logo mark', 'Shape'), ('Subtitle (EN)', 'Text'), ('字幕 02', 'Text'), ('Particles A', 'Shape'),
  ('Particles B', 'Shape'), ('Light leak 03', 'Video'), ('Camera 1', 'Camera'), ('Null: Rig', 'Null'), ('city_pass.mov', 'Video'), ('Shape 14', 'Shape'), ('Lower third bar', 'Shape'),
  ('Lower third name', 'Text'), ('Vignette', 'Shape'), ('Adjustment: Grade', 'Adjust'), ('Wind pad', 'Audio'), ('Kick loop', 'Audio'), ('Sphere tour', 'Video'),
];

Map<String, int> _tagCounts() {
  final m = <String, int>{};
  for (final i in kItems) {
    for (final t in i.tags) {
      m[t] = (m[t] ?? 0) + 1;
    }
  }
  return m;
}

/// The line that closes a capped section: how many are hidden, as "+N more" and nothing else (24 px, text at the same x 10 as the rows' names).
class _MoreLine extends StatelessWidget {
  const _MoreLine(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Padding(padding: const EdgeInsets.only(left: 10, top: 4), child: Text(text, style: T.label(N.g76)));
}

/// The line under the prefix field: which area the first mark chose, and the three marks (the live one lit).
class _AreaLine extends StatelessWidget {
  const _AreaLine(this.area, this.mark);
  final String area, mark;
  @override
  Widget build(BuildContext context) => Row(children: [
        Text('Area', style: T.label(N.g76)),
        const SizedBox(width: 8),
        Text(area, style: T.name(N.g95)),
        const Spacer(),
        for (final m in const ['#', '>', '@']) Padding(padding: const EdgeInsets.only(left: 4), child: _Kbd(m, on: mark == m)),
      ]);
}

class _Prefix extends StatefulWidget {
  const _Prefix({super.key, this.query = ''});
  final String query;
  @override
  State<_Prefix> createState() => _PrefixState();
}
