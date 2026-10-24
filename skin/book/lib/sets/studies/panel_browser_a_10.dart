// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

class _CollectionsState extends State<_Collections> {
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  final List<String> _names = ['Titles', 'Glows', 'Client A', 'Retro', 'Soft blurs', '', '', '', ''];
  final Map<String, Set<int>> _mem = {
    'Title In: Soft Rise': {1},
    'Pop In Overshoot': {1},
    'Glow': {2},
    'Bloom': {2, 5},
    'Neon Sign Flicker': {2, 3},
    '夜のネオン (client A)': {3},
    'Film Grain': {4},
    'Halation (Film Stock 500T)': {2, 4},
    'Gaussian Blur': {5},
    'Lens Blur': {5},
  };
  int _slot = 1, _sel = 0;
  String _say = '';

  List<It> get _pool => kItems.where((i) => i.place != 'Project' && (_slot < 0 || (_mem[i.name] ?? {}).contains(_slot + 1))).toList();
  int _n(int s) => kItems.where((i) => (_mem[i.name] ?? {}).contains(s)).length;

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final p = _pool;
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = p.isEmpty ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, p.length - 1));
      _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 34);
      return KeyEventResult.handled;
    }
    if (e is! KeyDownEvent || p.isEmpty) return KeyEventResult.ignored;
    final it = p[_sel.clamp(0, p.length - 1)];
    final k = e.logicalKey;
    final digits = [LogicalKeyboardKey.digit0, LogicalKeyboardKey.digit1, LogicalKeyboardKey.digit2, LogicalKeyboardKey.digit3, LogicalKeyboardKey.digit4, LogicalKeyboardKey.digit5, LogicalKeyboardKey.digit6, LogicalKeyboardKey.digit7, LogicalKeyboardKey.digit8, LogicalKeyboardKey.digit9];
    final d = digits.indexOf(k);
    if (d < 0) return KeyEventResult.ignored;
    setState(() {
      if (d == 0) {
        _mem.remove(it.name);
        _say = 'Took “${it.name}” out of all';
      } else {
        if (_names[d - 1].isEmpty) _names[d - 1] = 'Collection $d';
        final s = _mem.putIfAbsent(it.name, () => {});
        s.contains(d) ? s.remove(d) : s.add(d);
        _say = s.contains(d) ? '“${it.name}” → $d ${_names[d - 1]}' : '“${it.name}” left $d ${_names[d - 1]}';
      }
    });
    return KeyEventResult.handled;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = _pool, two = widget.w > 300, cw = two ? (widget.w - 4) / 2 : widget.w;
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          const _Sec('Collections'),
          Padding(
            padding: EdgeInsets.zero,
            child: Wrap(spacing: 4, children: [
              for (var i = 0; i < 9; i++)
                SizedBox(
                  width: cw,
                  child: _H(
                    onTap: () => setState(() {
                      if (_names[i].isEmpty) {
                        _names[i] = 'Collection ${i + 1}';
                        _say = 'Named slot ${i + 1}';
                      } else {
                        _slot = _slot == i ? -1 : i;
                        _sel = 0;
                      }
                    }),
                    builder: (context, h) => _SlotRow(i + 1, _names[i], _n(i + 1), on: _slot == i, hover: h),
                  ),
                ),
            ]),
          ),
          const SizedBox(height: 8),
          _Band(filters: _slot < 0 ? 0 : 1, shown: p.length, total: kItems.where((i) => i.place != 'Project').length, showFilters: false, onClear: () => setState(() => _slot = -1)),
          Expanded(
            child: p.isEmpty
                ? _Empty('${_names[_slot]} is empty', const [], actions: [_Btn('Show everything', onTap: () => setState(() => _slot = -1))])
                : _Results(
                    rows: p.length,
                    more: kItems.where((i) => i.place != 'Project').length - p.length,
                    where: 'outside this collection',
                    foot: false,
                    list: _FadeBottom(child: ListView.builder(
                      controller: _sc,
                      padding: EdgeInsets.zero,
                      itemExtent: 34,
                      itemCount: p.length,
                      itemBuilder: (context, i) {
                        final m = ((_mem[p[i].name] ?? {}).toList()..sort());
                        return _ItemRow(p[i], selected: i == _sel, onTap: () {
                          setState(() => _sel = i);
                          _root.requestFocus();
                        }, right: _SlotBadges(m, skip: _slot + 1));
                      },
                    )),
                  ),
          ),
          _Status(_say, keys: const ['1-9', '0']),
        ]),
      ),
    );
  }
}

// ================================================================================ B12-b  Similar to this

final List<Asset> _lib = [
  ...sampleAssets,
  for (var i = 0; i < 24; i++) Asset('frame_${(i + 1).toString().padLeft(2, '0')}.png', Kind.image, 30 + i * 7, style: ArtStyle.values[i % 3], aspect: .7),
];

List<double> _vec(Asset a) {
  final r = math.Random(a.name.codeUnits.fold<int>(7, (s, c) => s * 31 + c) & 0xFFFF);
  return [for (var i = 0; i < 8; i++) r.nextDouble()];
}

/// A tile of the similar view: picture (g20 edge, g56 on hover, mode when selected), name, and the closeness bar with its number.
class _SimTile extends StatelessWidget {
  const _SimTile(this.a, {required this.w, this.score, this.selected = false, this.hover = false, this.source = false, this.onSimilar, this.simHover});
  final Asset a;
  final double w;
  final double? score;
  final bool selected, hover, source;
  final VoidCallback? onSimilar;
  final bool? simHover;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
          Container(
            height: w * .7,
            decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: selected ? Role.selected : (hover ? N.g56 : N.g20))),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(5),
              child: Stack(fit: StackFit.expand, children: [
                Art(a),
                if (source) Positioned(left: 6, top: 6, child: _pill('Source', fill: N.g26)),
                if (selected) Positioned(left: 0, bottom: 0, child: CustomPaint(size: const Size(8, 8), painter: _CornerPaint(Role.selected, bottomLeft: true))),
                if (hover && !source) Positioned(right: 6, top: 6, child: _H(onTap: onSimilar, force: simHover, builder: (context, hh) => _pill('Similar', fill: hh ? N.g26 : N.g20))),
              ]),
            ),
          ),
          const SizedBox(height: 6),
          Text(a.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91).copyWith(decoration: selected ? TextDecoration.underline : TextDecoration.none, decorationColor: Role.selected)),
          if (score != null) ...[
            const SizedBox(height: 4),
            Row(children: [
              Expanded(child: Container(height: 2, alignment: Alignment.centerLeft, color: N.g20, child: FractionallySizedBox(widthFactor: score!.clamp(0.0, 1.0), child: Container(color: N.g76)))),
              const SizedBox(width: 8),
              Text(score!.toStringAsFixed(2), style: _num(N.g76)),
            ]),
          ],
        ]),
      );
}

class _Similar extends StatefulWidget {
  const _Similar({super.key, required this.w});
  final double w;
  @override
  State<_Similar> createState() => _SimilarState();
}

class _SimilarState extends State<_Similar> {
  late final _root = FocusNode(onKeyEvent: (n, e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyY && _sel != null) {
      _find(_sel!);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _from != null) {
      setState(() => _from = null);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  });
  Asset? _from = sampleAssets[0], _sel = sampleAssets[0];
  String _say = '';

  void _find(Asset a) => setState(() {
        _from = a;
        _sel = a;
        _say = '';
      });

  List<(Asset, double)> get _list {
    if (_from == null) return [for (final a in _lib) (a, 0.0)];
    final v = _vec(_from!);
    double d(Asset a) {
      final u = _vec(a);
      var s = 0.0;
      for (var i = 0; i < 8; i++) {
        s += (u[i] - v[i]) * (u[i] - v[i]);
      }
      return math.sqrt(s);
    }

    final all = [for (final a in _lib) (a, d(a))];
    final mx = all.map((e) => e.$2).reduce(math.max);
    final out = [for (final e in all) (e.$1, 1 - e.$2 / mx)];
    out.sort((a, b) => b.$2.compareTo(a.$2));
    return out;
  }

  @override
  void dispose() {
    _root.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final cols = widget.w > 300 ? 3 : 2, tw = (widget.w - 20 - (cols - 1) * 6) / cols, l = _list;
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _pad(_Tabs(const ['All', 'Image', 'Video', 'Audio', '3D'], 0, (_) {}, toggle: false), t: 10, b: 8),
          if (_from != null)
            _pad(
              Align(
                alignment: Alignment.centerLeft,
                child: _FilterChip('similar', _from!.name, onRemove: () => setState(() => _from = null)),
              ),
              b: 8,
            ),
          _Band(filters: _from == null ? 0 : 1, shown: l.length, total: _lib.length, showFilters: false, showCount: _from == null, extra: _from == null ? null : 'by similarity', onClear: () => setState(() => _from = null)),
          Expanded(
            child: _FadeBottom(child: SingleChildScrollView(
              padding: const EdgeInsets.fromLTRB(10, 10, 10, 10),
              child: Wrap(spacing: 6, runSpacing: 10, children: [
                for (final e in l)
                  _H(
                    onTap: () => setState(() => _sel = e.$1),
                    onDouble: () => _find(e.$1),
                    onSecondary: () => _find(e.$1),
                    builder: (context, h) => _SimTile(e.$1, w: tw, score: _from == null ? null : e.$2, selected: identical(_sel, e.$1), hover: h, source: _from != null && identical(e.$1, _from), onSimilar: () => _find(e.$1)),
                  ),
              ]),
            )),
          ),
          _Status(_say, keys: const ['Y', 'Esc']),
        ]),
      ),
    );
  }
}

// ================================================================================ the set

// ================================================================================ Parts @200%
// Each Parts sheet draws the small parts of one option group at twice their size, every state side by side, a 10 px caption under each.
// Vector text and lines are scaled by the canvas (FittedBox), not rasterised: nothing is blurred, so a 1 px line is a 2 px line and a padding is exactly doubled.
// Cell ground is g10 (the panel's own); the sheet is g00 so the cell's edge shows. Hover / focus are pinned through each part's own parameter, never faked.

class _PC {
  const _PC(this.caption, this.w, this.child, {this.h, this.ground = N.g10, this.pad = 0});
  final String caption;
  final double w;
  final double? h;
  final Widget child;
  final Color ground;
  final double pad;
}

class _Zoom extends StatelessWidget {
  const _Zoom(this.c);
  final _PC c;
  static const k = 2.0;
  @override
  Widget build(BuildContext context) {
    final w = c.w + c.pad * 2;
    return SizedBox(
      width: w * k,
      child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text(c.caption, softWrap: true, style: T.label(N.g76).copyWith(height: 1.3)),
        const SizedBox(height: 4),
        ColoredBox(
          color: c.ground,
          child: SizedBox(
            width: w * k,
            child: FittedBox(fit: BoxFit.fitWidth, alignment: Alignment.topLeft, child: SizedBox(width: w, height: c.h == null ? null : c.h! + c.pad * 2, child: Padding(padding: EdgeInsets.all(c.pad), child: c.child))),
          ),
        ),
      ]),
    );
  }
}

class _Parts extends StatefulWidget {
  const _Parts(this.id, this.title, this.note, this.sections);
  final String id, title, note;
  final List<(String, List<_PC>)> sections;
  @override
  State<_Parts> createState() => _PartsState();
}

class _PartsState extends State<_Parts> {
  final _v = ScrollController(), _h = ScrollController();
  late final List<GlobalKey> _keys = [for (final _ in widget.sections) GlobalKey()];
  @override
  void dispose() {
    _v.dispose();
    _h.dispose();
    super.dispose();
  }

  void _go(int i) {
    final c = _keys[i].currentContext;
    if (c != null) Scrollable.ensureVisible(c, duration: Mo.dur, curve: Mo.ease, alignment: 0, alignmentPolicy: ScrollPositionAlignmentPolicy.explicit);
  }

  /// Every lower row must be reachable without a pointer wheel: PageUp / PageDown / Home / End / arrows scroll the sheet, and the jump bar at the top goes to a section.
  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (!(e is KeyDownEvent || e is KeyRepeatEvent) || !_v.hasClients) return KeyEventResult.ignored;
    final vp = _v.position.viewportDimension, k = e.logicalKey;
    double? to;
    if (k == LogicalKeyboardKey.pageDown || k == LogicalKeyboardKey.space) to = _v.offset + vp * .9;
    if (k == LogicalKeyboardKey.pageUp) to = _v.offset - vp * .9;
    if (k == LogicalKeyboardKey.arrowDown) to = _v.offset + 80;
    if (k == LogicalKeyboardKey.arrowUp) to = _v.offset - 80;
    if (k == LogicalKeyboardKey.home) to = 0;
    if (k == LogicalKeyboardKey.end) to = _v.position.maxScrollExtent;
    if (to == null) return KeyEventResult.ignored;
    _v.jumpTo(to.clamp(0.0, _v.position.maxScrollExtent));
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g00,
        child: Focus(
          autofocus: true,
          onKeyEvent: _key,
          child: LayoutBuilder(builder: (context, box) {
          // The sheet is as wide as the viewport, or as the widest cell if that is wider: then it scrolls sideways instead of being cut.
          final widest = [for (final s in widget.sections) for (final c in s.$2) (c.w + c.pad * 2) * _Zoom.k].fold<double>(0, math.max);
          final inner = math.max(box.maxWidth - 32 - 12, widest);
          return RawScrollbar(
            controller: _v,
            thumbVisibility: true,
            thumbColor: N.g38,
            radius: const Radius.circular(3),
            thickness: 6,
            child: RawScrollbar(
              controller: _h,
              thumbVisibility: true,
              thumbColor: N.g38,
              radius: const Radius.circular(3),
              thickness: 6,
              notificationPredicate: (n) => n.depth == 1,
              child: SingleChildScrollView(
                controller: _v,
                child: SingleChildScrollView(
                  controller: _h,
                  scrollDirection: Axis.horizontal,
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(16, 16, 28, 28),
                    child: SizedBox(
                      width: inner,
                      child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                          Text(widget.id, style: T.value(N.g95).copyWith(fontWeight: FontWeight.w700)),
                          const SizedBox(width: 8),
                          Text(widget.title, style: T.title()),
                          const SizedBox(width: 8),
                          Text('@200%', style: T.micro(N.g76)),
                        ]),
                        const SizedBox(height: 8),
                        Text(widget.note, style: T.label(N.g76).copyWith(height: 1.35)),
                        const SizedBox(height: 10),
                        Wrap(spacing: 12, runSpacing: 6, crossAxisAlignment: WrapCrossAlignment.center, children: [
                          Text('JUMP TO', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                          for (var i = 0; i < widget.sections.length; i++) _Word(widget.sections[i].$1, style: T.label(N.g76), onTap: () => _go(i)),
                          Text('(PageDown, End and the arrows scroll)', style: T.label(N.g76)),
                        ]),
                        for (var i = 0; i < widget.sections.length; i++) ...[
                          const SizedBox(height: 24),
                          KeyedSubtree(key: _keys[i], child: Text(widget.sections[i].$1.toUpperCase(), style: T.micro(N.g76).copyWith(letterSpacing: .8))),
                          const SizedBox(height: 8),
                          Wrap(spacing: 16, runSpacing: 16, children: [for (final c in widget.sections[i].$2) _Zoom(c)]),
                        ],
                        const SizedBox(height: 24),
                        Text('END OF SHEET', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                      ]),
                    ),
                  ),
                ),
              ),
            ),
          );
        }),
        ),
      );
}

WidgetbookUseCase _parts(String name, Widget Function() build) => WidgetbookUseCase(name: 'Parts @200% $name', builder: (_) => build());

It _by(String n) => kItems.firstWhere((i) => i.name == n);
Widget _left(Widget w) => Align(alignment: Alignment.centerLeft, child: w);

/// A search field with its own text and focus node; [focus] pins the focused look.
class _SearchPart extends StatefulWidget {
  const _SearchPart({this.text = '', this.focus = false, this.hover, this.clearHover, this.hint = 'Search effects, presets, media', this.slash = true, this.trailing});
  final String text, hint;
  final bool focus, slash;
  final bool? hover, clearHover;
  final Widget? trailing;
  @override
  State<_SearchPart> createState() => _SearchPartState();
}

class _SearchPartState extends State<_SearchPart> {
  late final _c = TextEditingController(text: widget.text);
  final _n = FocusNode();
  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => _Search(controller: _c, node: _n, hint: widget.hint, slash: widget.slash, trailing: widget.trailing, focusLook: widget.focus, hoverLook: widget.hover, clearHover: widget.clearHover);
}

const _statusLong = 'Applied “Directional Blur (Motion Trail, 8-sample, GPU)” to Layer 3';
