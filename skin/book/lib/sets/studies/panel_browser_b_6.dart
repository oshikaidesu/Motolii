// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

// ================================================================ B10 favourites and recent

const _collNames = ['Titles', 'Glitch', 'Soft light', 'Wipes 切替', 'Camera', 'Looks ルック', 'Client A', 'Night 夜', 'Scratch'];
final List<_Fx> _plainFx = _fxs.where((f) => !f.isKey).toList();

/// B10-a: nine named collections reached by the number keys 1-9; 0 clears. Names, not colours (D-8).
class _Collections extends StatefulWidget {
  const _Collections({required this.w, this.empty = false});
  final double w;
  final bool empty;
  @override
  State<_Collections> createState() => _CollectionsState();
}

class _CollectionsState extends State<_Collections> {
  final _mem = <String, Set<int>>{};
  int _sel = -1, _filter = 0; // 0 = all, 1..9 = a collection
  String? _msg;
  final _recent = [_fxs[24], _fxs[48], _fxs[0], _fxs[26], _fxs[8], _fxs[32], _fxs[16], _fxs[3], _fxs[27], _fxs[25], _fxs[7], _fxs[2]];

  @override
  void initState() {
    super.initState();
    if (!widget.empty) {
      void put(String en, List<int> c) => _mem[en] = {...c};
      put('Typewriter', [1]);
      put('Per-character Fade', [1]);
      put('Tracking Reveal', [1, 6]);
      put('Glitch RGB Split', [2]);
      put('Chromatic Aberration with Lens Distortion', [2, 8]);
      put('Scanlines', [2]);
      put('Glow', [3, 6]);
      put('Vignette', [3, 8]);
      put('Film Grain', [3, 8]);
      put('Linear Wipe', [4]);
      put('Radial Wipe', [4]);
      put('Block Dissolve', [4]);
      put('Lens Blur', [5]);
      put('Hue/Saturation', [6]);
      put('Levels', [6]);
      put('Gradient Map', [6, 8]);
    }
  }

  int _count(int c) => _mem.values.where((s) => s.contains(c)).length;
  List<_Fx> get _vis => _filter == 0 ? _plainFx : _plainFx.where((f) => _mem[f.en]?.contains(_filter) ?? false).toList();

  void _toggle(int c) {
    if (_sel < 0) {
      setState(() => _msg = 'Select an effect first');
      return;
    }
    final f = _plainFx[_sel], s = _mem.putIfAbsent(f.en, () => {});
    setState(() {
      if (!s.add(c)) s.remove(c);
      _msg = s.contains(c) ? '${f.en} → $c ${_collNames[c - 1]}' : '${f.en} removed from $c ${_collNames[c - 1]}';
    });
  }

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent) return false;
    final ch = e.character;
    if (ch != null && ch.length == 1 && '123456789'.contains(ch)) {
      _toggle(int.parse(ch));
      return true;
    }
    if (ch == '0' && _sel >= 0) {
      setState(() {
        _mem.remove(_plainFx[_sel].en);
        _msg = '${_plainFx[_sel].en}  removed from every collection';
      });
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final vis = _vis;
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 8),
            _Sec('Collections', trailing: _filter == 0 ? null : _Chip('Show all', onTap: () => setState(() => _filter = 0))),
            LayoutBuilder(builder: (c, box) {
              final cols = box.maxWidth >= 280 ? 3 : 2, cw = ((box.maxWidth - 4 * (cols - 1)) / cols).floorToDouble();
              return Wrap(spacing: 4, runSpacing: 4, children: [
                for (var n = 1; n <= 9; n++)
                  SizedBox(
                    width: cw,
                    child: _Hit(
                      onTap: () => setState(() => _filter = _filter == n ? 0 : n),
                      builder: (c, h, _) => _CollChip(n, _collNames[n - 1], _count(n), on: _filter == n, hover: h),
                    ),
                  ),
              ]);
            }),
            const SizedBox(height: 8),
            const _Sec('Recent'),
            SizedBox(
              height: 58,
              child: _fadeSide(ListView.separated(
                scrollDirection: Axis.horizontal,
                itemCount: _recent.length,
                separatorBuilder: (c, i) => const SizedBox(width: 6),
                itemBuilder: (c, i) => SizedBox(width: 66, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  SizedBox(height: 38, child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Thumb(_recent[i]))),
                  const SizedBox(height: 4),
                  _Mid(_recent[i].en, T.label(N.g76)),
                ])),
              )),
            ),
            const SizedBox(height: 4),
            _Sec(_filter == 0 ? 'All' : 'Matches', n: _filter == 0 ? '${vis.length}' : '${vis.length} of ${_plainFx.length}'),
          ]),
          _Grow(
            child: vis.isEmpty
                ? SingleChildScrollView(child: _empty('$_filter ${_collNames[_filter - 1]} はまだ空です', ''))
                : _Fade(child: ListView.builder(
                    shrinkWrap: _PanelH.fitOf(context),
                    padding: const EdgeInsets.only(bottom: 12),
                    itemExtent: 28,
                    itemCount: vis.length,
                    itemBuilder: (c, i) {
                      final f = vis[i], at = _plainFx.indexOf(f), m = [for (final n in (_mem[f.en] ?? <int>{}).toList()..sort()) if (n != _filter) n];
                      return _Hit(onTap: () => setState(() => _sel = at), builder: (c, h, _) => _FxRow(f, sel: _sel == at, hover: h, whole: true, tags: [for (final n in m.take(3)) Tooltip(message: '$n ${_collNames[n - 1]}', waitDuration: const Duration(milliseconds: 400), child: _Tag('$n')), if (m.length > 3) const _Tag('…')]));
                    },
                  )),
          ),
          _Foot(msg: _msg, hints: const [('1-9', 'add / remove'), ('0', 'clear'), ('click', 'filter')]),
        ]),
      ),
    );
  }
}

/// A numbered collection button (1-9): key number, name, how many it holds.
class _CollChip extends StatelessWidget {
  const _CollChip(this.n, this.name, this.count, {this.on = false, this.hover = false});
  final int n, count;
  final String name;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 24,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(color: on ? N.g26 : (hover ? N.g20 : N.g13), borderRadius: BorderRadius.circular(4), border: Border.all(color: on ? _accent : N.g20)),
        child: Row(children: [
          Text('$n', style: _num(N.g95)),
          const SizedBox(width: 4),
          Expanded(child: _t(name, T.label(on ? N.g95 : N.g76))),
          const SizedBox(width: 4),
          Text(count == 0 ? '–' : '$count', style: _num(count == 0 ? N.g63 : N.g76)),
        ]),
      );
}

/// B10-b: one star (F), recents and frequents fill themselves.
class _StarAuto extends StatefulWidget {
  const _StarAuto({required this.w, this.start = 0, this.fresh = false});
  final double w;
  final int start;
  final bool fresh;
  @override
  State<_StarAuto> createState() => _StarAutoState();
}

class _StarAutoState extends State<_StarAuto> {
  late int _view = widget.start;
  final _stars = <String>{};
  final _use = <String, int>{};
  final _recent = <String>[];
  int _sel = -1;
  String? _msg;

  @override
  void initState() {
    super.initState();
    if (!widget.fresh) {
      _stars.addAll(['Glow', 'Gaussian Blur', 'Vignette', 'Typewriter']);
      const u = {'Gaussian Blur': 41, 'Glow': 33, 'Levels': 19, 'Drop Shadow': 17, 'Hue/Saturation': 12, 'Linear Wipe': 9, 'Fractal Noise': 6, 'Echo': 3};
      _use.addAll(u);
      _recent.addAll(['Levels', 'Glow', 'Linear Wipe', 'Gaussian Blur', 'Echo', 'Drop Shadow', 'Fractal Noise', 'Hue/Saturation']);
    }
  }

  List<_Fx> get _vis {
    final byName = {for (final f in _plainFx) f.en: f};
    switch (_view) {
      case 1:
        return [for (final f in _plainFx) if (_stars.contains(f.en)) f];
      case 2:
        return [for (final n in _recent) byName[n]!];
      case 3:
        final k = _use.keys.toList()..sort((a, b) => _use[b]!.compareTo(_use[a]!));
        return [for (final n in k) byName[n]!];
      default:
        return _plainFx;
    }
  }

  void _star(_Fx f) => setState(() {
        _stars.contains(f.en) ? _stars.remove(f.en) : _stars.add(f.en);
        _msg = _stars.contains(f.en) ? 'Starred ${f.en}' : 'Unstarred ${f.en}';
      });

  void _useIt(_Fx f) => setState(() {
        _use[f.en] = (_use[f.en] ?? 0) + 1;
        _recent.remove(f.en);
        _recent.insert(0, f.en);
        if (_recent.length > 12) _recent.removeLast();
        _msg = 'Used ${f.en} (${_use[f.en]}×)';
      });

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent) return false;
    if (e.logicalKey == LogicalKeyboardKey.keyF && _sel >= 0 && _sel < _vis.length) {
      _star(_vis[_sel]);
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final vis = _vis;
    final emptyMsg = switch (_view) {
      1 => ('星はまだありません', ''),
      2 => ('まだ何も使っていません', ''),
      3 => ('まだ回数がありません', ''),
      _ => ('', ''),
    };
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 8),
            LayoutBuilder(builder: (c, box) => Segmented(items: box.maxWidth < 250 ? const ['All', 'Stars', 'Recent', 'Often'] : const ['All', 'Starred', 'Recent', 'Frequent'], index: _view, expand: true, onChanged: (i) => setState(() {
                  _view = i;
                  _sel = -1;
                }))),
            const SizedBox(height: 6),
            _Sec(const ['By family', 'By family', 'Latest first', 'By use count'][_view], n: '${vis.length}'),
          ]),
          _Grow(
            child: vis.isEmpty
                ? SingleChildScrollView(child: _empty(emptyMsg.$1, emptyMsg.$2))
                : _Fade(child: ListView.builder(
                    shrinkWrap: _PanelH.fitOf(context),
                    padding: const EdgeInsets.only(bottom: 12),
                    itemExtent: 28,
                    itemCount: vis.length,
                    itemBuilder: (c, i) {
                      final f = vis[i], on = _stars.contains(f.en);
                      return _Hit(
                        onTap: () => setState(() => _sel = i),
                        onDouble: () => _useIt(f),
                        builder: (c, h, _) => _FxRow(f, sel: _sel == i, hover: h, whole: true, tags: [
                          if (_view == 3) Text('${_use[f.en]}×', style: _num()),
                          StarButton(on: on, onChanged: (_) => _star(f), size: 14),
                        ]),
                      );
                    },
                  )),
          ),
          _Foot(msg: _msg, hints: const [('F', 'star'), ('dbl', 'use'), ('click', 'select')]),
        ]),
      ),
    );
  }
}

/// B10-c: a pile of what was used; the same thing folds to the front; the oldest falls off.
class _Pile extends StatefulWidget {
  const _Pile({super.key, required this.w, this.cap = 12});
  final double w;
  final int cap;
  @override
  State<_Pile> createState() => _PileState();
}

class _PileState extends State<_Pile> {
  final List<(String, int)> _pile = [('Glow', 33), ('Levels', 19), ('Gaussian Blur', 41), ('Drop Shadow', 17), ('Hue/Saturation', 12)];
  String? _flash, _msg;
  Timer? _tick2;

  @override
  void dispose() {
    _tick2?.cancel();
    super.dispose();
  }

  void _use(_Fx f) {
    final at = _pile.indexWhere((e) => e.$1 == f.en);
    final n = at < 0 ? 1 : _pile[at].$2 + 1;
    setState(() {
      if (at >= 0) _pile.removeAt(at);
      _pile.insert(0, (f.en, n));
      String? dropped;
      while (_pile.length > widget.cap) {
        dropped = _pile.removeLast().$1;
      }
      _flash = f.en;
      _msg = at >= 0 ? '${f.en} moved to the front' : '${f.en} on top${dropped == null ? '' : ' · $dropped dropped'}';
    });
    _tick2?.cancel();
    _tick2 = Timer(const Duration(milliseconds: 900), () {
      if (mounted) setState(() => _flash = null);
    });
  }

  @override
  Widget build(BuildContext context) {
    final byName = {for (final f in _plainFx) f.en: f};
    final rows = math.min(_pile.length, 7);
    return _Frame(
      w: widget.w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(2)),
          const SizedBox(height: 8),
          _Sec('Used', n: '${_pile.length} / ${widget.cap}'),
        ]),
        Container(
          height: _pile.isEmpty ? 56 : rows * 28.0 + 8,
          padding: const EdgeInsets.symmetric(vertical: 4),
          decoration: const BoxDecoration(color: N.g07, border: Border.symmetric(horizontal: BorderSide(color: N.g20))),
          child: _pile.isEmpty
              ? Padding(padding: const EdgeInsets.symmetric(horizontal: _px), child: CustomPaint(painter: DashedBox(N.g38), child: Center(child: Text('まだ空です', style: T.label(N.g76)))))
              : _Fade(child: ListView.builder(
                  itemExtent: 28,
                  itemCount: _pile.length,
                  itemBuilder: (c, i) {
                    final (n, k) = _pile[i];
                    return _Hit(onDouble: () => _use(byName[n]!), builder: (c, h, _) => _FxRow(byName[n]!, hover: h, sel: _flash == n, whole: true, tags: [Text('$k×', style: _num())]));
                  },
                )),
        ),
        Padding(padding: const EdgeInsets.fromLTRB(_px, 8, _px, 0), child: _Sec('All', n: '${_plainFx.length}')),
        _Grow(
          child: _Fade(child: ListView.builder(
            shrinkWrap: _PanelH.fitOf(context),
            padding: const EdgeInsets.only(bottom: 12),
            itemExtent: 28,
            itemCount: _plainFx.length,
            itemBuilder: (c, i) {
              final f = _plainFx[i];
              return _Hit(onDouble: () => _use(f), builder: (c, h, _) => _FxRow(f, hover: h, whole: true));
            },
          )),
        ),
        _Foot(msg: _msg, hints: const [('dbl', 'use')]),
      ]),
    );
  }
}

// ================================================================ B11 own presets on the shelf

class _NameEdit extends StatefulWidget {
  const _NameEdit({required this.initial, required this.onDone, this.autofocus = true});
  final String initial;
  final ValueChanged<String> onDone;
  final bool autofocus;
  @override
  State<_NameEdit> createState() => _NameEditState();
}

class _NameEditState extends State<_NameEdit> {
  late final _c = TextEditingController(text: widget.initial);
  final _n = FocusNode();
  bool _done = false;
  @override
  void initState() {
    super.initState();
    if (!widget.autofocus) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      _n.requestFocus();
      _c.selection = TextSelection(baseOffset: 0, extentOffset: _c.text.length);
    });
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  void _finish(String s) {
    if (_done) return;
    _done = true;
    widget.onDone(s.trim().isEmpty ? widget.initial : s.trim());
  }

  @override
  Widget build(BuildContext context) => Focus(
        onKeyEvent: (node, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
            _finish(widget.initial);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Container(
          height: 24,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          alignment: Alignment.centerLeft,
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: _accent)),
          child: EditableText(controller: _c, focusNode: _n, style: T.name(N.g95), cursorColor: _accent, backgroundCursorColor: N.g26, selectionColor: _accent.withValues(alpha: .35), maxLines: 1, onSubmitted: _finish),
        ),
      );
}

class _Mine {
  _Mine(this.name, this.kind, this.scope, {this.naming = false});
  String name;
  final String kind;
  String scope;
  bool naming;
}

List<_Mine> _seedMine() => [
      _Mine('Soft Glow + Levels (client A)', 'Effects', 'This project'),
      _Mine('Overshoot 少し強め', 'Curve', 'Everywhere'),
      _Mine('夜の青 Night Blue', 'Colour', 'Everywhere'),
      _Mine('Hero Title Stack', 'Layer', 'This project'),
      _Mine('Handheld Shake 弱', 'Effects', 'Everywhere'),
      _Mine('Bounce 0.8 / 重め', 'Curve', 'Everywhere'),
    ];

/// One Mine row, 28 px: name, the kind tag only when it differs from the tab (the Effects tab never says "Effects"), and the scope word only when it is not "Everywhere".
Widget _mineRow(_Mine m, {bool hover = false, bool sel = false, bool kind = true, Widget? trailing}) => _Line(
      height: 28,
      hover: hover,
      sel: sel,
      child: Row(children: [
        Expanded(child: _Mid(m.name, T.name(sel ? N.g95 : N.g91))),
        if (kind && m.kind != 'Effects') Padding(padding: const EdgeInsets.only(left: 6), child: _Tag(m.kind)),
        if (trailing != null) trailing else if (m.scope == 'This project') Padding(padding: const EdgeInsets.only(left: 6), child: Text('Project', style: T.label(N.g76))),
      ]),
    );

/// A recessed well for a short list: g07, a hairline above and below, rows full width so their text starts at the one inset.
Widget _well({required Widget child, Color edge = N.g20}) => DecoratedBox(decoration: BoxDecoration(color: N.g07, border: Border.symmetric(horizontal: BorderSide(color: edge))), child: child);

/// B11-a: carry a thing from the Inspector onto "Mine"; it is saved, the original stays, and a name box opens in place.
class _SaveToShelf extends StatefulWidget {
  const _SaveToShelf({required this.w, this.empty = false});
  final double w;
  final bool empty;
  @override
  State<_SaveToShelf> createState() => _SaveToShelfState();
}
