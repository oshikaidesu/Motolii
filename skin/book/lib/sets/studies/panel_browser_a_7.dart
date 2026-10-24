// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

/// A key the empty field offers: an outlined chip (g13, 1 px g20), so it reads as "tap to try", not as a filter already on.
class _HintChip extends StatelessWidget {
  const _HintChip(this.text, {this.hover = false});
  final String text;
  final bool hover;
  @override
  Widget build(BuildContext context) => Container(height: 22, padding: const EdgeInsets.symmetric(horizontal: 8), decoration: BoxDecoration(color: hover ? N.g20 : N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)), child: Center(widthFactor: 1, child: Text(text, style: T.label(N.g76))));
}

/// One line of the completion list under the chip field: 24 px, mono; the first is the one Enter takes.
class _SugRow extends StatelessWidget {
  const _SugRow(this.text, {this.first = false, this.hover = false});
  final String text;
  final bool first, hover;
  @override
  Widget build(BuildContext context) => Container(
        height: 24,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        alignment: Alignment.centerLeft,
        color: first ? N.g20 : (hover ? N.g15 : null),
        child: Row(children: [Expanded(child: Text(text, style: T.value(first ? N.g95 : N.g76))), if (first) const _Kbd('Enter')]),
      );
}

/// The completion list: a g13 box with a g26 edge around [_SugRow]s.
class _SugBox extends StatelessWidget {
  const _SugBox(this.children);
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Container(
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g26)),
        child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: children),
      );
}

/// The field that holds chips and the text being typed: g07, 1 px g20 (g63 when focused), 4 px inside.
class _FieldShell extends StatelessWidget {
  const _FieldShell({required this.focused, required this.children});
  final bool focused;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        constraints: const BoxConstraints(minHeight: 30),
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: focused ? N.g63 : N.g20)),
        child: Wrap(spacing: 4, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: children),
      );
}

// ================================================================================ B2-c  The result heading says the filter as a sentence

/// The result heading as a sentence: each word is a filter and drops itself when clicked; the tail says how many remain.
class _SentenceHead extends StatelessWidget {
  const _SentenceHead({required this.words, required this.count, required this.total, this.taps, this.hoverIndex, this.note});
  final List<String> words;
  final String? note;
  final List<VoidCallback>? taps;
  final int count, total;
  final int? hoverIndex;
  @override
  Widget build(BuildContext context) => Container(
        width: double.infinity,
        padding: const EdgeInsets.all(12),
        decoration: const BoxDecoration(color: N.g13, border: Border.symmetric(horizontal: BorderSide(color: N.rowLine))),
        child: words.isEmpty
            ? Text('All $total items', style: T.title())
            : Wrap(spacing: 8, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: [
                for (var i = 0; i < words.length; i++) ...[
                  if (i > 0) Text('·', style: T.title(N.g63)),
                  _Word(words[i], onTap: taps?[i], style: T.title(N.g95), rule: true, hover: hoverIndex == null ? null : hoverIndex == i),
                ],
                Text(count == 0 ? '— nothing' : '— $count of $total${note == null ? '' : ' · $note'}', style: T.name(N.g76)),
              ]),
      );
}

class _Sentence extends StatefulWidget {
  const _Sentence({super.key, this.kind = 0, this.chr = 0, this.fav = false, this.text = ''});
  final int kind, chr;
  final bool fav;
  final String text;
  @override
  State<_Sentence> createState() => _SentenceState();
}

class _SentenceState extends State<_Sentence> {
  static const _kinds = ['Any', 'Effect', 'Preset', 'Media'], _chars = ['Any', 'Soft', 'Glitch', 'Retro', 'Bright', 'Grade'];
  late int _k = widget.kind, _ch = widget.chr;
  late bool _fav = widget.fav;
  late final _c = TextEditingController(text: widget.text);
  late final _n = FocusNode(onKeyEvent: (n, e) => _esc(e, _c, n, (_) {}));

  List<It> _by({bool k = true, bool ch = true, bool fav = true, bool tx = true}) => _match(kItems.where((i) => (!k || _k == 0 || i.kind == _kinds[_k]) && (!ch || _ch == 0 || i.tags.contains(_chars[_ch].toLowerCase())) && (!fav || !_fav || i.fav)), tx ? _c.text : '');

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: _c,
        builder: (context, _) {
          final r = _by(), q = _c.text.trim();
          final words = <(String, VoidCallback)>[
            if (_ch > 0) (_chars[_ch], () => setState(() => _ch = 0)),
            if (_k > 0) (_kinds[_k] == 'Media' ? 'Media' : '${_kinds[_k]}s', () => setState(() => _k = 0)),
            if (_fav) ('Favorites', () => setState(() => _fav = false)),
            if (q.isNotEmpty) ('“$q”', _c.clear),
          ];
          final fixes = <(String, int, VoidCallback)>[];
          if (r.isEmpty) {
            if (_ch > 0) fixes.add((_chars[_ch], _by(ch: false).length, () => setState(() => _ch = 0)));
            if (_k > 0) fixes.add((_kinds[_k] == 'Media' ? 'Media' : '${_kinds[_k]}s', _by(k: false).length, () => setState(() => _k = 0)));
            if (_fav) fixes.add(('Favorites', _by(fav: false).length, () => setState(() => _fav = false)));
            if (q.isNotEmpty) fixes.add(('“$q”', _by(tx: false).length, _c.clear));
            fixes.sort((a, b) => b.$2.compareTo(a.$2));
          }
          return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(_Search(controller: _c, node: _n, onChanged: (_) => setState(() {})), t: 10, b: 8),
            _pad(_Tabs(_kinds, _k, (i) => setState(() => _k = i < 0 ? 0 : i), toggle: false, chevron: true), b: 6),
            _pad(_Tabs(_chars, _ch, (i) => setState(() => _ch = i < 0 ? 0 : i), toggle: false, chevron: true), b: 8),
            _pad(Row(children: [PillSwitch(on: _fav, onChanged: (v) => setState(() => _fav = v)), const SizedBox(width: 8), Text('Favorites only', style: T.label(N.g76))]), b: 8),
            _SentenceHead(words: [for (final w in words) w.$1], taps: [for (final w in words) w.$2], count: r.length, total: kItems.length, note: _k == 0 ? _mostly(r) : null),
            Expanded(
              child: r.isEmpty
                  ? _Empty('Nothing fits', const [], actions: [for (final f in fixes.take(3)) _Btn('Drop ${f.$1} → ${f.$2}', onTap: f.$3)])
                  : _filtered(r, fade: true, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, tokens: _tokens(_c.text), showKind: _k == 0)),
            ),
          ]);
        },
      );
}

// ================================================================================ B2-d  A saved filter becomes a place

/// A place or a saved filter, 26 px: name and live count; on hover a saved filter swaps the count for its Delete word.
class _PlaceRow extends StatelessWidget {
  const _PlaceRow(this.label, this.count, {this.on = false, this.hover = false, this.right});
  final String label;
  final int count;
  final bool on, hover;
  final Widget? right;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        color: on ? N.g20 : (hover ? N.g15 : null),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Row(children: [
              Expanded(child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g91))),
              const SizedBox(width: 8),
              if (right != null && hover) right! else Text('$count', style: _num(N.g76)),
            ]),
          ),
          _GutterTick(on, height: 12),
        ]),
      );
}

class _SavedF {
  _SavedF(this.name, this.spec);
  final String name;
  final Set<String> spec; // 'cat:Blur', 'tag:soft', 'kind:Effect'
}

bool _specPass(It i, Set<String> s) => s.every((x) {
      final p = x.split(':');
      return switch (p[0]) { 'cat' => i.cat == p[1], 'tag' => i.tags.contains(p[1]), _ => i.kind == p[1] };
    });

class _Saved extends StatefulWidget {
  const _Saved({super.key, this.extra = 0});
  final int extra;
  @override
  State<_Saved> createState() => _SavedState();
}

class _SavedState extends State<_Saved> {
  final List<_SavedF> _saved = [
    _SavedF('Soft blurs', {'cat:Blur', 'tag:soft'}),
    _SavedF('Retro looks', {'tag:retro'}),
    _SavedF('Title presets', {'cat:Title'}),
    _SavedF('Bright & loop', {'tag:bright', 'tag:loop'}),
  ];
  final List<It> _extra = [];
  Set<String> _spec = {};
  _SavedF? _active;
  int _place = -1;
  bool _naming = false, _more = false;
  final _name = TextEditingController();
  late final _nn = FocusNode(onKeyEvent: (n, e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      setState(() => _naming = false);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      _commit();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  });
  String _say = '“Soft blurs” = cat:Blur + tag:soft';

  @override
  void initState() {
    super.initState();
    _active = _saved.first;
    _spec = {..._saved.first.spec};
    for (var i = 0; i < widget.extra; i++) {
      _saved.add(_SavedF(const ['Grade only', 'Glow family', 'Moving type', 'Glitch + retro', 'Ease curves', 'Everything soft', 'Used 10+ times', 'Client A look', 'Loops', 'Bokeh set', 'Time effects', 'Distortions'][i % 12], {i.isEven ? 'tag:soft' : 'cat:Glow'}));
    }
  }

  List<It> get _pool => [...kItems, ..._extra];
  List<It> get _res => _pool.where((i) => (_place < 0 || i.place == _places[_place]) && _specPass(i, _spec)).toList();

  void _commit() {
    final n = _name.text.trim();
    if (n.isEmpty || _spec.isEmpty) return;
    final s = _SavedF(n, {..._spec});
    setState(() {
      _saved.add(s);
      _active = s;
      _naming = false;
      _say = 'Saved “$n”';
    });
  }

  @override
  void dispose() {
    _name.dispose();
    _nn.dispose();
    super.dispose();
  }

  Widget _chipRow(String label, String prefix, List<String> vals) {
    String key(String v) => '$prefix:${prefix == 'tag' ? v.toLowerCase() : v}';
    return _TagGroup(
      label: label,
      values: vals,
      on: {for (final v in vals) if (_spec.contains(key(v))) v},
      onTap: (v) => setState(() {
        final k = key(v);
        _spec.contains(k) ? _spec.remove(k) : _spec.add(k);
        _active = null;
      }),
    );
  }

  @override
  Widget build(BuildContext context) {
    final r = _res;
    final shownSaved = !_more && _saved.length > 4 ? _saved.take(3).toList() : _saved;
    Widget place(String label, int count, bool on, VoidCallback tap, {Widget? right}) => _H(onTap: tap, builder: (context, h) => _PlaceRow(label, count, on: on, hover: h, right: right));
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      const _Sec('Places'),
      for (var i = 0; i < _places.length; i++) place(_places[i], _pool.where((x) => x.place == _places[i]).length, _place == i && _active == null && _spec.isEmpty, () => setState(() {
            _place = _place == i ? -1 : i;
            _spec = {};
            _active = null;
          })),
      _Sec('Saved filters', right: Text('${_saved.length}', style: T.label(N.g76))),
      SizedBox(
        height: math.min(shownSaved.length * 26.0, 130),
        child: ListView(padding: EdgeInsets.zero, children: [
      for (final s in shownSaved)
        place(s.name, _pool.where((x) => _specPass(x, s.spec)).length, identical(_active, s), () => setState(() {
              _active = s;
              _spec = {...s.spec};
              _place = -1;
              _say = '“${s.name}” = ${s.spec.join(' + ')}';
            }), right: _Word('Delete', onTap: () => setState(() {
              _saved.remove(s);
              if (identical(_active, s)) _active = null;
              _say = 'Deleted “${s.name}”';
            }))),
        ]),
      ),
      if (!_more && _saved.length > 4) Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 0), child: Row(children: [Flexible(child: _Word('Show ${_saved.length - 3} more saved filters', onTap: () => setState(() => _more = true)))])),
      if (_more && _saved.length > 4) Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 0), child: Row(children: [Flexible(child: _Word('Fold to 3', onTap: () => setState(() => _more = false)))])),
      const _Sec('Filter'),
      _chipRow('Category', 'cat', const ['Blur', 'Glow', 'Color', 'Title']),
      _chipRow('Character', 'tag', const ['Soft', 'Retro', 'Bright', 'Loop']),
      Padding(
        padding: const EdgeInsets.fromLTRB(10, 2, 10, 6),
        child: _naming
            ? SizedBox(height: 28, child: _Search(controller: _name, node: _nn, autofocus: true, slash: false, hint: 'Filter name', trailing: const _Kbd('Esc')))
            : Row(children: [
                Expanded(
                  child: Wrap(spacing: 6, runSpacing: 4, children: [
                    _Btn('Save as place…', onTap: _spec.isEmpty ? null : () => setState(() {
                          _naming = true;
                          _name.clear();
                        })),
                    _Btn('Simulate import', onTap: () => setState(() {
                          _extra.add(It('Soft Rain Blur ${_extra.length + 1}', 'やわらか雨ぼかし', 'Effects', 'Blur', Fx.blur, const ['soft']));
                          _say = 'Added “Soft Rain Blur ${_extra.length}”';
                        })),
                  ]),
                ),
              ]),
      ),
      Container(height: 1, color: N.rowLine),
      Expanded(
        child: r.isEmpty
            ? _Empty('Nothing in this filter yet', const [], actions: [_Btn('Show everything', onTap: () => setState(() {
                    _spec = {};
                    _active = null;
                    _place = -1;
                  }))])
            : _filtered(r, quiet: false, fade: true, hidden: _pool.length - r.length, row: (it, i) => _ItemRow(it, showKind: !_spec.any((x) => x.startsWith('kind:')), showCat: _spec.where((x) => x.startsWith('cat:')).length != 1)),
      ),
      _Status(_say),
    ]);
  }
}

// ================================================================================ B5/B6  preview before applying

Widget _ownBase({bool sample = false}) => sample
    ? Stack(fit: StackFit.expand, children: [const Art(Asset('', Kind.image, 9, style: ArtStyle.gradient)), Center(child: Text('SAMPLE', style: T.micro(N.g76).copyWith(letterSpacing: 8)))])
    : Stack(fit: StackFit.expand, children: [
        const Art(Asset('', Kind.image, 3)),
        Center(child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
          Text('光の速さで', style: T.title().copyWith(fontSize: 22, height: 1.1)),
          const SizedBox(height: 6),
          Text('LIGHTSPEED', style: T.micro(N.g95).copyWith(letterSpacing: 4)),
        ])),
      ]);

const _scrim = Color(0x99000000);

/// A label laid over a picture: 16 px tall, 6 px sides, one line, ellipsis when the picture is narrow.
Widget _pill(String s, {Color fill = _scrim, Color ink = N.g95}) => Container(
      height: 16,
      padding: const EdgeInsets.symmetric(horizontal: 6),
      decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(3)),
      child: Center(widthFactor: 1, child: Text(s, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(ink).copyWith(letterSpacing: .4))),
    );

/// Two labels on one edge of a picture, 6 px in; the right one gives way (ellipsis) before it touches the left one.
Widget _edge(Widget left, Widget? right, {bool top = true}) => Positioned(left: 6, right: 6, top: top ? 6 : null, bottom: top ? null : 6, child: Row(children: [Flexible(flex: 3, child: Align(alignment: Alignment.centerLeft, child: left)), const SizedBox(width: 6), Expanded(flex: 2, child: Align(alignment: Alignment.centerRight, child: right ?? const SizedBox()))]));

// ---- B5-a  every tile is a small screening

class _MovingTiles extends StatefulWidget {
  const _MovingTiles({super.key, required this.count, required this.reduce, required this.w});
  final int count;
  final bool reduce;
  final double w;
  @override
  State<_MovingTiles> createState() => _MovingTilesState();
}

class _MovingTilesState extends State<_MovingTiles> {
  final _sc = ScrollController();
  static const _gap = 8.0;
  List<It> get _pool => kItems.where((i) => i.asset == null).toList();

  It _at(int i) {
    final p = _pool, b = p[(i * 5) % p.length];
    return i < p.length ? b : It('${b.name} v${i ~/ p.length + 1}', b.jp, b.place, b.cat, b.fx, b.tags);
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final tw = (widget.w - 20 - _gap) / 2, th = tw * .62 + 42, rowH = th + _gap;
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      AnimatedBuilder(
        animation: _sc,
        builder: (context, _) {
          final vp = _sc.hasClients ? _sc.position.viewportDimension : 500.0, off = _sc.hasClients ? _sc.offset : 0.0;
          final first = (off / rowH).floor(), last = ((off + vp) / rowH).floor();
          final vis = math.min(widget.count, (last - first + 1) * 2);
          // Right of the title, in words: how many tiles move now, or "held" (reduce motion). No mark on the tiles.
          return _pad(
            Row(children: [
              Expanded(child: Text('Effects and presets', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
              Text(widget.reduce ? 'held' : '$vis playing', style: widget.reduce ? T.label(N.g63) : T.label(N.g76)),
            ]),
            t: 12,
            b: 8,
          );
        },
      ),
      Expanded(
        child: _Loop(
          running: !widget.reduce,
          rest: .55,
          builder: (context, t) => _FadeBottom(child: GridView.builder(
            controller: _sc,
            padding: const EdgeInsets.fromLTRB(10, 0, 10, 10),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: 2, mainAxisSpacing: _gap, crossAxisSpacing: _gap, mainAxisExtent: th),
            itemCount: widget.count,
            itemBuilder: (context, i) {
              final it = _at(i);
              return _H(builder: (context, h) => _TileCard(it, t: t, w: tw, hover: h));
            },
          )),
        ),
      ),
    ]);
  }
}

/// One screening tile: the moving picture (g20 edge, g56 on hover), the name, then category and kind.
class _TileCard extends StatelessWidget {
  const _TileCard(this.it, {required this.t, required this.w, this.hover = false});
  final It it;
  final double t, w;
  final bool hover;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
          Container(
            height: w * .62,
            decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: hover ? N.g56 : N.g20)),
            child: ClipRRect(borderRadius: BorderRadius.circular(5), child: Stack(fit: StackFit.expand, children: [_Fx(it.fx, t, seed: it.seed)])),
          ),
          const SizedBox(height: 6),
          Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name()),
          const SizedBox(height: 4),
          Text('${it.cat} · ${it.kind}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
        ]),
      );
}

// ---- B5-b / B6  hover to play, x to scrub

/// The one picture area of a tile: still at rest, a loop while hovered; timed items scrub by pointer x.
class _HoverThumb extends StatefulWidget {
  const _HoverThumb(this.it, {required this.w, required this.h, this.onInfo, this.selected = false, this.holdHover = false, this.holdX});
  final It it;
  final double w, h;
  final bool selected;

  /// Parts sheets: start with the pointer already on the tile (and at x = holdX), loop held at its middle.
  final bool holdHover;
  final double? holdX;
  final ValueChanged<String?>? onInfo;
  @override
  State<_HoverThumb> createState() => _HoverThumbState();
}
