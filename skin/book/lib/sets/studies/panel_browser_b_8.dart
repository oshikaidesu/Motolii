// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

class _LibraryState extends State<_Library> {
  late int _view = widget.view;
  late String _q = widget.query;
  double _size = 84;
  int _sort = 1, _sel = -1;
  String? _note;
  List<Asset>? _all;
  List<Asset> _cache = const [];
  String _key = '';

  List<Asset> get _items {
    final all = _all ??= _gen(widget.n);
    final k = '$_q|$_sort|${widget.n}';
    if (k != _key) {
      _key = k;
      final q = _q.toLowerCase();
      final l = [for (final a in all) if (q.isEmpty || a.name.toLowerCase().contains(q)) a];
      if (_sort == 0) l.sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
      if (_sort == 1) l.sort((a, b) => b.seed.compareTo(a.seed));
      if (_sort == 2) l.sort((a, b) => a.kind.index != b.kind.index ? a.kind.index.compareTo(b.kind.index) : a.name.compareTo(b.name));
      _cache = l;
    }
    return _cache;
  }

  @override
  Widget build(BuildContext context) {
    final w = widget.w, fit = _PanelH.fitOf(context);
    final items = widget.state == 0 ? _items : const <Asset>[];
    const gutter = EdgeInsets.symmetric(horizontal: _px);
    Widget body;
    if (widget.state == 1) {
      body = Padding(
        padding: gutter,
        child: LayoutBuilder(builder: (c, box) {
          final cols = math.max(2, ((box.maxWidth + 8) / (_size + 8)).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols;
          return _Pulse(child: SingleChildScrollView(physics: const NeverScrollableScrollPhysics(), child: Wrap(spacing: 8, runSpacing: 10, children: [for (var i = 0; i < cols * 5; i++) _Skel(tw)])));
        }),
      );
    } else if (widget.state == 2) {
      body = SingleChildScrollView(child: _empty('このフォルダを読めません', 'Permission denied\n/Volumes/RAID_A/Footage/2026/Day3_Shoot', tone: Role.error, action: Row(mainAxisSize: MainAxisSize.min, children: [_Btn('Retry', onTap: () => setState(() => _note = 'Still unreadable')), const SizedBox(width: 6), _Btn('Locate…', onTap: () => setState(() => _note = 'Folder picker'))])));
    } else if (items.isEmpty) {
      body = SingleChildScrollView(child: _empty('"$_q" に一致する素材はありません', '', tone: Role.warning, action: _Btn('Clear search', onTap: () => setState(() => _q = ''))));
    } else if (_view == 0) {
      body = Padding(
        padding: gutter,
        child: LayoutBuilder(builder: (c, box) {
          final cols = math.max(2, ((box.maxWidth + 8) / (_size + 8)).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols, names = tw >= 70;
          return GridView.builder(
            shrinkWrap: fit,
            padding: const EdgeInsets.only(bottom: 10),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: 10, crossAxisSpacing: 8, mainAxisExtent: tw * .62 + (names ? 33 : 0)),
            itemCount: items.length,
            itemBuilder: (c, i) {
              final a = items[i];
              return _Hit(onTap: () => setState(() => _sel = a.seed), builder: (c, h, _) => _AssetCell(a, tw, sel: _sel == a.seed, hover: h, names: names));
            },
          );
        }),
      );
    } else {
      body = ListView.builder(
        shrinkWrap: fit,
        padding: const EdgeInsets.only(bottom: 4),
        itemExtent: 28,
        itemCount: items.length,
        itemBuilder: (c, i) {
          final a = items[i];
          return _Hit(onTap: () => setState(() => _sel = a.seed), builder: (c, h, _) => _AssetRow(a, sel: _sel == a.seed, hover: h));
        },
      );
    }
    final selName = _sel >= 0 && _all != null ? _all![_sel].name : null;
    final count = widget.state == 1 ? 'reading…' : (widget.state == 2 ? 'offline' : (items.length == widget.n ? _fmt(widget.n) : '${_fmt(items.length)} of ${_fmt(widget.n)}'));
    return _Frame(
      w: w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(3)),
          const SizedBox(height: 8),
          TextBox(text: _q, hint: 'Search items', look: Look.quiet, clearable: true, leading: const nav.Glyph(nav.G.search, color: N.g76), onChanged: (s) => setState(() => _q = s)),
          const SizedBox(height: 8),
          Row(children: [
            Segmented(items: const ['Grid', 'List'], index: _view, onChanged: (i) => setState(() => _view = i)),
            const Spacer(),
            if (_view == 0) ...[Text('size', style: T.label(N.g76)), const SizedBox(width: 4), MiniSlider(value: _size, min: 44, max: 140, width: w < 280 ? 64 : 92, onChanged: (v) => setState(() => _size = v))],
          ]),
          const SizedBox(height: 6),
          Row(children: [for (final (i, s) in ['Name', 'Date', 'Type'].indexed) ...[_Chip(s, on: _sort == i, onTap: () => setState(() => _sort = i)), const SizedBox(width: 4)]]),
          const SizedBox(height: 4),
          _Sec('Items', n: count),
        ]),
        _Grow(child: _fadeFoot(body, on: widget.state == 0 && items.length > 24)),
        _Foot(msg: widget.state == 2 ? _note : selName),
      ]),
    );
  }
}

/// B12-b: "find similar" turns the list into a ranking by a made-up 8-number description; the anchor stays first.
double _dist(int a, int b) {
  List<double> v(int i) {
    final k = _bases[i % _bases.length];
    final r = math.Random(i * 7919 + 13);
    return [for (var d = 0; d < 8; d++) ((k.$3.index * 3 + k.$4.index + d * 2) % 7) / 7 + (r.nextDouble() - .5) * .35];
  }

  final x = v(a), y = v(b);
  var s = 0.0;
  for (var d = 0; d < 8; d++) {
    s += (x[d] - y[d]) * (x[d] - y[d]);
  }
  return math.sqrt(s);
}

/// The removable filter chip "Similar to NAME": g20, hover g26, an accent line, and the close glyph. [w] is the width it may take.
class _SimChip extends StatelessWidget {
  const _SimChip(this.name, this.w, {this.hover = false});
  final String name;
  final double w;
  final bool hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        height: 24,
        padding: const EdgeInsets.only(left: 8, right: 6),
        decoration: BoxDecoration(color: hover ? N.g26 : N.g20, borderRadius: BorderRadius.circular(4), border: Border.all(color: _accentLine)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [Text('Similar to', style: T.label(N.g76)), const SizedBox(width: 5), ConstrainedBox(constraints: BoxConstraints(maxWidth: math.max(40.0, w - 84)), child: _Mid(name, T.name(N.g95))), const SizedBox(width: 6), const nav.Glyph(nav.G.close, color: N.g76, size: 10)]),
      );
}

/// The context menu of a tile: Find similar is live, the rest are disabled in this draft (g63 word, arrow cursor, no hover).
class _CtxMenu extends StatelessWidget {
  const _CtxMenu({required this.onFind});
  final VoidCallback onFind;
  @override
  Widget build(BuildContext context) => Container(
        width: 160,
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 20, offset: const Offset(0, 8))]),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (final (label, key, act) in <(String, String, VoidCallback?)>[('Find similar', 'S', onFind), ('Reveal in Finder', '', null), ('Rename…', '↵', null), ('Remove from project', '⌫', null)])
            _Hit(onTap: act, cursor: act == null ? SystemMouseCursors.basic : SystemMouseCursors.click, builder: (c, h, _) => Container(height: 26, padding: const EdgeInsets.symmetric(horizontal: 8), decoration: BoxDecoration(color: h && act != null ? N.g26 : _clear, borderRadius: BorderRadius.circular(4)), child: Row(children: [Expanded(child: _t(label, T.name(act == null ? N.g63 : N.g95))), if (key.isNotEmpty) Text(key, style: _num())]))),
        ]),
      );
}

class _Similar extends StatefulWidget {
  const _Similar({required this.w, this.start = -1});
  final double w;
  final int start; // asset index the panel starts "similar to"
  @override
  State<_Similar> createState() => _SimilarState();
}

class _SimilarState extends State<_Similar> {
  final _all = _gen(240);
  late int _sel = widget.start, _anchor = widget.start;
  Offset _ptr = Offset.zero;
  int _menu = -1;

  void _similar(int i) => setState(() {
        _anchor = i;
        _sel = i;
        _menu = -1;
      });

  bool _key(KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyS && _sel >= 0) {
      _similar(_sel);
      return true;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      setState(() {
        _menu = -1;
        _anchor = -1;
      });
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final items = _anchor < 0 ? _all : ([..._all]..sort((a, b) => _dist(_anchor, a.seed).compareTo(_dist(_anchor, b.seed))));
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: Listener(
          onPointerDown: (e) => _ptr = e.localPosition,
          child: Stack(children: [
            _Col([
              _Head([
                const IgnorePointer(child: _Tabs(3)),
                const SizedBox(height: 8),
                if (_anchor >= 0) ...[
                  Align(alignment: Alignment.centerLeft, child: _Hit(onTap: () => setState(() => _anchor = -1), builder: (c, h, _) => _SimChip(_all[_anchor].name, widget.w - 2 * _px, hover: h))),
                  const SizedBox(height: 6),
                ],
                _Sec(_anchor < 0 ? 'All' : 'Closest first', n: _anchor < 0 ? '${items.length}' : null),
              ]),
              _Grow(
                child: _Fade(child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: _px),
                  child: LayoutBuilder(builder: (c, box) {
                    final cols = math.max(2, ((box.maxWidth + 8) / 96).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols;
                    return GridView.builder(
                      shrinkWrap: _PanelH.fitOf(context),
                      padding: const EdgeInsets.only(bottom: 16),
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: 10, crossAxisSpacing: 8, mainAxisExtent: tw * .62 + 33),
                      itemCount: items.length,
                      itemBuilder: (c, i) {
                        final a = items[i];
                        final pct = _anchor < 0 ? null : (a.seed == _anchor ? 'source' : '${(100 - _dist(_anchor, a.seed) * 38).clamp(3, 99).round()}%');
                        return _Hit(
                          onTap: () => setState(() {
                            _sel = a.seed;
                            _menu = -1;
                          }),
                          onSecondary: () => setState(() {
                            _sel = a.seed;
                            _menu = a.seed;
                          }),
                          builder: (c, h, _) => _AssetCell(a, tw, sel: _sel == a.seed, hover: h, badge: pct),
                        );
                      },
                    );
                  }),
                )),
              ),
              const _Foot(hints: [('right-click', 'menu'), ('S', 'find similar'), ('Esc', 'clear')]),
            ]),
            if (_menu >= 0)
              Positioned(
                left: _ptr.dx.clamp(8.0, widget.w - 168),
                top: _ptr.dy.clamp(8.0, 560),
                child: _CtxMenu(onFind: () => _similar(_menu)),
              ),
          ]),
        ),
      ),
    );
  }
}

/// A demo histogram for the Parts sheet.
const _demoHist = [2, 4, 7, 12, 9, 5, 3, 2, 1, 1, 2, 4, 8, 14, 20, 16, 9, 5, 3, 2, 1, 1, 0, 1];

/// The band of B12-c: a histogram of the library on one axis, the kept range over it, and labels where their values sit (the duration axis is logarithmic).
class _Band extends StatelessWidget {
  const _Band({required this.hist, required this.axis, required this.bw, this.r});
  final List<int> hist;
  final int axis;
  final double bw;
  final (double, double)? r;
  @override
  Widget build(BuildContext context) {
    final peak = hist.reduce(math.max), bins = hist.length, rr = r;
    return SizedBox(
      width: bw,
      height: 112,
      child: Stack(children: [
        Positioned.fill(child: DecoratedBox(decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(5), border: Border.all(color: N.g20)))),
        Positioned(left: 0, right: 0, top: 6, bottom: 22, child: Row(crossAxisAlignment: CrossAxisAlignment.end, children: [
          for (var i = 0; i < bins; i++)
            Expanded(child: Padding(padding: const EdgeInsets.symmetric(horizontal: .5), child: FractionallySizedBox(heightFactor: math.max(.03, hist[i] / peak), child: DecoratedBox(decoration: BoxDecoration(color: rr == null || ((i + .5) / bins >= rr.$1 && (i + .5) / bins <= rr.$2) ? N.g91 : N.g38, borderRadius: BorderRadius.circular(1)))))),
        ])),
        if (rr != null) Positioned(left: bw * rr.$1, width: math.max(2, bw * (rr.$2 - rr.$1)), top: 0, bottom: 0, child: DecoratedBox(decoration: BoxDecoration(color: _accent.withValues(alpha: .12), border: Border.symmetric(vertical: BorderSide(color: _accent))))),
        if (rr != null) for (final x in [rr.$1, rr.$2]) Positioned(left: (bw * x - 3).clamp(0.0, bw - 5), top: 52, width: 5, height: 8, child: const CustomPaint(painter: _GripP(N.g95))),
        Positioned(left: 0, right: 0, bottom: 5, height: 12, child: _labels()),
      ]),
    );
  }

  Widget _labels() {
    if (axis == 0) {
      return Padding(padding: const EdgeInsets.symmetric(horizontal: 4), child: DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(2), gradient: LinearGradient(colors: [for (var i = 0; i <= 12; i++) HSVColor.fromAHSV(1, i * 30.0, .5, .72).toColor()]))));
    }
    final labels = axis == 1 ? const [('still', .04), ('10 s', .40), ('1 min', .686), ('5 min', .953)] : const [('Image', .1), ('Video', .3), ('Audio', .5), ('3D', .7), ('360°', .9)];
    return Stack(children: [for (final (l, f) in labels) Align(alignment: Alignment(f * 2 - 1, 0), child: Text(l, style: T.label(N.g76)))]);
  }
}

/// B12-c: one axis, one band. Drag across the band to keep a range.
class _AxisStrip extends StatefulWidget {
  const _AxisStrip({required this.w, this.axis = 0});
  final double w;
  final int axis;
  @override
  State<_AxisStrip> createState() => _AxisStripState();
}

class _AxisStripState extends State<_AxisStrip> {
  static const _bins = 24;
  final _all = _gen(400);
  late int _axis = widget.axis;
  double? _a, _b;

  /// Position of an asset on the axis, 0..1.
  double _pos(Asset a) {
    switch (_axis) {
      case 0:
        final k = a.kind.index * .19, r = math.Random(a.seed * 31 + 5).nextDouble();
        return (k + (r - .5) * .5 + (a.style.index * .07)) % 1;
      case 1:
        return a.dur == 0 ? 0 : (math.log(a.dur + 1) / math.log(400)).clamp(.04, 1.0);
      default:
        return (a.kind.index + .5) / 5;
    }
  }

  List<int> _hist() {
    final h = List.filled(_bins, 0);
    for (final a in _all) {
      h[(_pos(a) * _bins).floor().clamp(0, _bins - 1)]++;
    }
    return h;
  }

  (double, double)? get _range => _a == null || _b == null ? null : (math.min(_a!, _b!), math.max(_a!, _b!));

  @override
  Widget build(BuildContext context) {
    final hist = _hist();
    final r = _range;
    final items = r == null ? _all : [for (final a in _all) if (_pos(a) >= r.$1 && _pos(a) <= r.$2) a];
    return _Frame(
      w: widget.w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(3)),
          const SizedBox(height: 8),
          Segmented(items: const ['Colour', 'Duration', 'Type'], index: _axis, expand: true, onChanged: (i) => setState(() {
            _axis = i;
            _a = _b = null;
          })),
          const SizedBox(height: 8),
          _Sec('Range', trailing: r == null ? null : _Chip('Clear range', onTap: () => setState(() => _a = _b = null))),
          LayoutBuilder(builder: (c, box) {
            final bw = box.maxWidth;
            return MouseRegion(cursor: SystemMouseCursors.resizeLeftRight, child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanStart: (d) => setState(() => _a = _b = (d.localPosition.dx / bw).clamp(0.0, 1.0)),
              onPanUpdate: (d) => setState(() => _b = (d.localPosition.dx / bw).clamp(0.0, 1.0)),
              onTapDown: (d) => setState(() => _a = _b = null),
              child: _Band(hist: hist, axis: _axis, bw: bw, r: r),
            ));
          }),
          const SizedBox(height: 8),
          _Sec(r == null ? 'All' : 'In range', n: r == null ? '${items.length}' : '${items.length} of ${_all.length}'),
        ]),
        _Grow(
          child: _Fade(child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: _px),
            child: items.isEmpty
                ? SingleChildScrollView(child: _empty('この範囲に素材はありません', ''))
                : LayoutBuilder(builder: (c, box) {
                    final cols = math.max(2, ((box.maxWidth + 8) / 92).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols;
                    return GridView.builder(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.only(bottom: 16), gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: 10, crossAxisSpacing: 8, mainAxisExtent: tw * .62 + 33), itemCount: items.length, itemBuilder: (c, i) => _AssetCell(items[i], tw));
                  }),
          )),
        ),
        const _Foot(hints: [('drag', 'keep a range'), ('click', 'clear')]),
      ]),
    );
  }
}

/// B12-d: a folder on disk is a live source. A file that disappears stays as a missing row.
class _DiskFile {
  _DiskFile(this.name, {this.added});
  final String name;
  bool onDisk = true;
  DateTime? added;
}

/// A file row of the live folder. `gone` (missing on disk, or the whole drive offline) is shown by a flat thumb and g63 name, never by opacity.
class _LiveRow extends StatelessWidget {
  const _LiveRow(this.name, this.asset, {this.sel = false, this.hover = false, this.fresh = false, this.onDisk = true, this.offline = false, this.onLocate});
  final String name;
  final Asset asset;
  final bool sel, hover, fresh, onDisk, offline;
  final VoidCallback? onLocate;
  @override
  Widget build(BuildContext context) {
    final gone = !onDisk || offline;
    return _Line(
      height: 28,
      sel: sel,
      hover: hover || fresh,
      child: Row(children: [
        SizedBox(width: 24, height: 18, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: gone ? const ColoredBox(color: N.g20) : _Art(asset))),
        const SizedBox(width: 8),
        Expanded(child: _Mid(name, T.name(gone ? N.g63 : N.g91))),
        if (fresh) const _Tag('NEW', hi: true),
        if (!onDisk && !offline) ...[const _Tag('Missing', warn: true), const SizedBox(width: 4), _Btn('Locate…', onTap: onLocate ?? _noop)],
      ]),
    );
  }
}

class _LiveFolder extends StatefulWidget {
  const _LiveFolder({required this.w});
  final double w;
  @override
  State<_LiveFolder> createState() => _LiveFolderState();
}
