// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

/// The tail of a tag group: "+N hidden tags" (grammatical: "+1 hidden tag") or "Hide N", as a quiet text button on the chip line: g76 word, g20 fill on hover, pressed g20 + the accent line at 50%, keyboard focus a 1 px g63 edge, a 10 px chevron (down = shows them, up = hides). hover / focus / pressed pin a state (Parts).
class _HiddenToggle extends StatelessWidget {
  const _HiddenToggle(this.n, {this.open = false, this.onTap, this.hover, this.focus, this.pressed});
  final int n;
  final bool open;
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  Widget build(BuildContext context) => _Ctl(
        onTap: onTap ?? () {},
        hover: hover,
        focus: focus,
        pressed: pressed,
        builder: (context, h, f, p) => _ctlBody(
          height: 22,
          pad: const EdgeInsets.only(left: 6, right: 4),
          fill: h || p ? N.g20 : _clear,
          edge: f ? N.g63 : _clear,
          pressed: p,
          child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Text(open ? 'Hide $n' : '+$n hidden ${n == 1 ? 'tag' : 'tags'}', maxLines: 1, softWrap: false, style: T.micro(N.g76)),
            const SizedBox(width: 3),
            CustomPaint(size: const Size(10, 10), painter: _ChevronPaint(open, N.g76)),
          ]),
        ),
      );
}

class _ChevronPaint extends CustomPainter {
  const _ChevronPaint(this.up, this.color);
  final bool up;
  final Color color;
  @override
  void paint(Canvas c, Size s) {
    final y0 = up ? 6.5 : 3.5, y1 = up ? 3.5 : 6.5;
    c.drawPath(Path()..moveTo(2, y0)..lineTo(5, y1)..lineTo(8, y0), Paint()..color = color..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  }

  @override
  bool shouldRepaint(_ChevronPaint o) => o.up != up || o.color != color;
}

/// One filter group: its name on the left (64 px, g63) and its chips; every chip starts at x 74 (10 + 64), the label at x 10.
/// Chips with count 0 are not drawn at rest: one trailing text button "+N hidden tags" (a chevron, g76, g20 on hover) follows the last chip on the same line when it fits, and shows them (dashed, g56); it then reads "Hide N".
/// Folded = one line ("any" or the choice) and a Show word. [hoverIndex] pins one chip's hover look; [zeroOpen] pins whether the hidden ones are shown (null = the group's own state).
class _TagGroup extends StatefulWidget {
  const _TagGroup({required this.label, required this.values, required this.on, this.counts, this.open = true, this.canHide = false, this.onTap, this.onShow, this.onHide, this.hoverIndex, this.zeroOpen, this.hoverToggle});
  final String label;
  final List<String> values;
  final Set<String> on;
  final Map<String, int>? counts;
  final bool open, canHide;
  final ValueChanged<String>? onTap;
  final VoidCallback? onShow, onHide;
  final int? hoverIndex;
  final bool? zeroOpen, hoverToggle;
  @override
  State<_TagGroup> createState() => _TagGroupState();
}

class _TagGroupState extends State<_TagGroup> {
  bool _zero = false;
  @override
  Widget build(BuildContext context) {
    final w = widget, showZero = w.zeroOpen ?? _zero;
    bool isZero(String v) => w.counts != null && w.counts![v] == 0 && !w.on.contains(v);
    final live = [for (final v in w.values) if (!isZero(v)) v], dead = [for (final v in w.values) if (isZero(v)) v];
    Widget chip(String v, int i) => _ChipBtn(v, count: w.counts?[v], on: w.on.contains(v), hover: w.hoverIndex == null ? null : w.hoverIndex == i, onTap: () => w.onTap?.call(v));
    return Padding(
      padding: const EdgeInsets.fromLTRB(10, 0, 10, 6),
      child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
        SizedBox(width: 64, height: 22, child: Align(alignment: Alignment.centerLeft, child: Text(w.label, style: T.label(N.g76)))),
        Expanded(
          child: w.open
              ? Wrap(spacing: 4, runSpacing: 4, children: () {
                  // Chips pack as many to a line as the width allows; the toggle is the next item after the last chip, so it sits right after it and wraps only when the width is out.
                  final chips = [for (final v in live) chip(v, w.values.indexOf(v)), if (showZero) for (final v in dead) chip(v, w.values.indexOf(v))];
                  if (dead.isEmpty) return chips;
                  return [...chips, _HiddenToggle(dead.length, open: showZero, hover: w.hoverToggle, onTap: () => setState(() => _zero = !_zero))];
                }())
              : SizedBox(height: 22, child: Row(children: [
                  Expanded(child: Text(w.on.isEmpty ? 'any' : w.on.join(', '), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(w.on.isEmpty ? N.g76 : N.g95))),
                  const SizedBox(width: 8),
                  _Word('Show ${w.values.length}', onTap: w.onShow),
                ])),
        ),
        if (w.open && w.canHide && w.on.isEmpty) Padding(padding: const EdgeInsets.only(left: 8), child: SizedBox(height: 22, child: Center(child: _Word('Hide', onTap: w.onHide)))),
      ]),
    );
  }
}

class _TagBands extends StatefulWidget {
  const _TagBands({super.key, this.fold = false, this.add = false, this.start = const {}});
  final bool fold, add;
  final Map<String, List<String>> start;
  @override
  State<_TagBands> createState() => _TagBandsState();
}

class _TagBandsState extends State<_TagBands> {
  final _c = TextEditingController();
  late final _n = FocusNode(onKeyEvent: (n, e) => _esc(e, _c, n, (_) {}));
  late final _Sel _s = {for (final g in _groups.keys) g: <String>{...(widget.start[g] ?? const [])}};
  late final Set<String> _open = {if (!widget.fold) ..._groups.keys else 'Kind', for (final e in widget.start.entries) if (e.value.isNotEmpty) e.key};
  int _pick = 0;

  bool get _addMode => widget.add || HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed || HardwareKeyboard.instance.isShiftPressed;

  List<It> get _res => _match(kItems.where((i) => _pass(i, _s)), _c.text);

  int _count(String g, String v) => _match(kItems.where((i) => _pass(i, _s, skip: g) && _in(i, g, v)), _c.text).length;

  void _tap(String g, String v) => setState(() {
        final cur = _s[g]!;
        if (cur.contains(v)) {
          cur.remove(v);
        } else {
          if (!_addMode) cur.clear();
          cur.add(v);
        }
        _open.add(g);
      });

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
          final r = _res, n = _nFilters(_s) + (_c.text.trim().isEmpty ? 0 : 1);
                    Widget group(String g) => _TagGroup(
                label: g,
                values: _groups[g]!,
                on: _s[g]!,
                counts: {for (final v in _groups[g]!) v: _count(g, v)},
                open: _open.contains(g),
                canHide: widget.fold,
                onTap: (v) => _tap(g, v),
                onShow: () => setState(() => _open.add(g)),
                onHide: () => setState(() => _open.remove(g)),
              );

          final worst = <String>[];
          if (r.isEmpty && n > 0) {
            for (final e in _s.entries) {
              for (final v in e.value) {
                final t = {for (final k in _s.keys) k: {..._s[k]!}}..[e.key]!.remove(v);
                final k = _match(kItems.where((i) => _pass(i, t)), _c.text).length;
                worst.add('$k\t${e.key}: $v');
              }
            }
            worst.sort((a, b) => int.parse(b.split('\t').first).compareTo(int.parse(a.split('\t').first)));
            worst.removeWhere((w) => w.startsWith('0\t'));
          }
          return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(_Search(controller: _c, node: _n, onChanged: (_) => setState(() {})), t: 10, b: 8),
            for (final g in _groups.keys) group(g),
            _Band(filters: n, shown: r.length, total: kItems.length, extra: _kindFixed(_s['Kind']!) ? null : _mostly(r), onClear: () => setState(() {
                  for (final s in _s.values) {
                    s.clear();
                  }
                  _c.clear();
                })),
            Expanded(
              child: r.isEmpty
                  ? _Empty('No item has all $n filters', const [], actions: [
                      for (final w in worst.take(2)) _Btn('Remove ${w.split('\t').last} → ${w.split('\t').first}', onTap: () => setState(() {
                            final p = w.split('\t').last.split(': ');
                            _s[p[0]]!.remove(p[1]);
                          })),
                      _Btn('Clear $n filters', onTap: () => setState(() {
                            for (final s in _s.values) {
                              s.clear();
                            }
                            _c.clear();
                          })),
                    ])
                  : _filtered(r, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, selected: i == _pick, tokens: _tokens(_c.text), showKind: !_kindFixed(_s['Kind']!), showCat: !_kindFixed(_s['Category']!), onTap: () => setState(() => _pick = i))),
            ),
            _Status(_addMode ? 'Add' : 'Only', keys: const ['Cmd', 'Shift']),
          ]);
        },
      );
}

// ---- B4 alone: the band and what it tells

class _BandDemo extends StatefulWidget {
  const _BandDemo({super.key, required this.n, required this.text, this.hoverClear, this.hoverX});
  final int n;
  final String text;
  final bool? hoverClear, hoverX;
  @override
  State<_BandDemo> createState() => _BandDemoState();
}

class _BandDemoState extends State<_BandDemo> {
  static const _all = [('Kind', 'Effect'), ('Category', 'Glow'), ('Character', 'Soft'), ('Source', 'Built-in')];
  late List<(String, String)> _on = _all.take(widget.n).toList();
  late String _text = widget.text;

  List<It> get _res {
    final s = <String, Set<String>>{for (final g in _groups.keys) g: {}};
    for (final f in _on) {
      s[f.$1]!.add(f.$2);
    }
    return _match(kItems.where((i) => _pass(i, s)), _text);
  }

  @override
  Widget build(BuildContext context) {
    final r = _res, n = _on.length + (_text.isEmpty ? 0 : 1);
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _pad(
        Container(
          height: 28,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
          child: Row(children: [
            Expanded(child: Text(_text.isEmpty ? 'Search effects, presets, media' : _text, style: T.name(_text.isEmpty ? N.g76 : N.g95))),
            if (_text.isNotEmpty) _Cross(hover: widget.hoverX, onTap: () => setState(() => _text = '')),
          ]),
        ),
        t: 10,
        b: 8,
      ),
      _Band(filters: n, shown: r.length, total: kItems.length, extra: _on.any((f) => f.$1 == 'Kind') ? null : _mostly(r), hoverClear: widget.hoverClear, onClear: () => setState(() {
            _on = [];
            _text = '';
          })),
      if (n > 0)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 8, 10, 4),
          child: Wrap(spacing: 4, runSpacing: 4, children: [
            for (final f in _on) _ActiveChip(f.$2, onTap: () => setState(() => _on = [..._on]..remove(f))),
            if (_text.isNotEmpty) _ActiveChip('“$_text”', onTap: () => setState(() => _text = '')),
          ]),
        ),
      Expanded(
        child: r.isEmpty
            ? _Empty('Nothing matches $n filters', const [], actions: [_Btn('Clear all', onTap: () => setState(() {
                    _on = [];
                    _text = '';
                  }))])
            : _filtered(r, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, tokens: _tokens(_text), showKind: !_on.any((f) => f.$1 == 'Kind'), showCat: !_on.any((f) => f.$1 == 'Category'))),
      ),
    ]);
  }
}

// ================================================================================ B2-b  Chips inside the field

const _qkeys = <String, List<String>>{
  'kind': ['effect', 'preset', 'media'],
  'cat': ['blur', 'color', 'glow', 'distort', 'stylize', 'time', 'generate', 'title', 'motion', 'ease'],
  'tag': ['soft', 'glitch', 'retro', 'bright', 'grade', 'moving', 'loop'],
  'in': ['effects', 'presets', 'project', 'mine'],
  'fav': ['yes'],
};

class _Q {
  const _Q(this.key, this.val);
  final String key, val;
  String get raw => '$key:$val';
  bool test(It i) => switch (key) {
        'kind' => i.kind.toLowerCase() == val,
        'cat' => i.cat.toLowerCase() == val,
        'tag' => i.tags.contains(val),
        'in' => i.place.toLowerCase() == val,
        _ => i.fav,
      };
}

_Q? _parseQ(String t) {
  final p = t.toLowerCase().split(':');
  if (p.length != 2 || !_qkeys.containsKey(p[0]) || !(_qkeys[p[0]]!.contains(p[1]))) return null;
  return _Q(p[0], p[1]);
}

class _ChipField extends StatefulWidget {
  const _ChipField({super.key, this.chips = 0, this.text = ''});
  final int chips;
  final String text;
  @override
  State<_ChipField> createState() => _ChipFieldState();
}

class _ChipFieldState extends State<_ChipField> {
  static const _seed = ['kind:effect', 'tag:soft', 'cat:glow', 'in:effects', 'tag:bright', 'fav:yes', 'tag:moving', 'tag:retro'];
  late final _c = TextEditingController(text: widget.text);
  late final _n = FocusNode(onKeyEvent: _key);
  late final List<_Q> _chips = [for (final s in _seed.take(widget.chips)) _parseQ(s)!];
  String _say = '';

  @override
  void initState() {
    super.initState();
    _n.addListener(() => setState(() {}));
    _c.addListener(_watch);
  }

  void _watch() {
    final t = _c.text;
    if (t.endsWith(' ') && t.trim().isNotEmpty) {
      final parts = t.trim().split(RegExp(r'\s+'));
      final q = _parseQ(parts.last);
      if (q != null) {
        _chips.add(q);
        final rest = parts.sublist(0, parts.length - 1).join(' ');
        _c.value = TextEditingValue(text: rest.isEmpty ? '' : '$rest ', selection: TextSelection.collapsed(offset: rest.isEmpty ? 0 : rest.length + 1));
        _say = 'Added ${q.raw}';
      }
    }
    setState(() {});
  }

  String get _tok => _c.text.split(' ').last;

  /// (label, insertion) pairs for the token being typed.
  List<(String, String)> _suggest2() {
    final t = _tok.toLowerCase();
    if (t.isEmpty) return const [];
    if (!t.contains(':')) return [for (final k in _qkeys.keys) if (k.startsWith(t) && k != t) ('$k:', '$k:')];
    final p = t.split(':');
    final vals = _qkeys[p[0]];
    if (vals == null || p.length > 2) return const [];
    return [for (final v in vals) if (v.startsWith(p[1])) ('${p[0]}:$v', '${p[0]}:$v ')];
  }

  void _insert(String s) {
    final words = _c.text.split(' ')..removeLast();
    final t = [...words, s].join(' ');
    _c.value = TextEditingValue(text: t, selection: TextSelection.collapsed(offset: t.length));
    _n.requestFocus();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.backspace && _c.text.isEmpty && _chips.isNotEmpty) {
      final q = _chips.removeLast();
      _c.value = TextEditingValue(text: q.raw, selection: TextSelection.collapsed(offset: q.raw.length));
      setState(() => _say = 'Editing ${q.raw}');
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      final s = _suggest2();
      if (s.isNotEmpty) _insert(s.first.$2);
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s));
  }

  List<It> get _res {
    final free = _c.text.split(' ').where((w) => w.isNotEmpty && _parseQ(w) == null).join(' ');
    return _match(kItems.where((i) => _chips.every((q) => q.test(i))), free);
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  Widget _chip(_Q q) => _FilterChip(q.key, q.val, onRemove: () => setState(() => _chips.remove(q)));

  @override
  Widget build(BuildContext context) {
    final f = _n.hasFocus, collapse = !f && _chips.length > 3;
    final shownChips = collapse ? _chips.take(2).toList() : _chips;
    final r = _res, sg = _suggest2();
    final bad = _tok.contains(':') && _qkeys.containsKey(_tok.split(':').first.toLowerCase()) && sg.isEmpty && _tok.split(':').length == 2 && _tok.split(':')[1].isNotEmpty && !_tok.endsWith(' ');
    final tp = TextPainter(text: TextSpan(text: _c.text, style: T.name(N.g95)), textDirection: TextDirection.ltr)..layout();
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      LayoutBuilder(builder: (context, box) {
        final tw = _c.text.isEmpty && _chips.isEmpty ? box.maxWidth - 28 : (tp.width + 16).clamp(90.0, box.maxWidth - 36);
        return _pad(
          GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: _n.requestFocus,
            child: _FieldShell(
              focused: f,
              children: [
                for (final q in shownChips) _chip(q),
                if (collapse) _MoreChip(_chips.length - 2),
                SizedBox(
                  width: tw,
                  height: 20,
                  child: Stack(alignment: Alignment.centerLeft, children: [
                    if (_c.text.isEmpty && _chips.isEmpty) IgnorePointer(child: Text('Search, or kind: tag: cat:', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g76))),
                    EditableText(controller: _c, focusNode: _n, style: T.name(N.g95), cursorColor: N.g95, backgroundCursorColor: N.g44, selectionColor: Role.selected.withValues(alpha: .35), cursorWidth: 1, maxLines: 1),
                  ]),
                ),
              ],
            ),
          ),
          t: 10,
        );
      }),
      if (_c.text.isEmpty && _chips.isEmpty)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 8, 10, 0),
          child: Wrap(spacing: 4, runSpacing: 4, crossAxisAlignment: WrapCrossAlignment.center, children: [
            for (final k in const ['kind:', 'cat:', 'tag:', 'in:', 'fav:yes']) _H(onTap: () => _insert(k), builder: (context, h) => _HintChip(k, hover: h)),
          ]),
        ),
      if (sg.isNotEmpty)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 4, 10, 0),
          child: _SugBox([
            for (var i = 0; i < math.min(sg.length, 5); i++) _H(onTap: () => _insert(sg[i].$2), builder: (context, h) => _SugRow(sg[i].$1, first: i == 0, hover: h)),
          ]),
        ),
      if (bad)
        Padding(
          padding: const EdgeInsets.fromLTRB(10, 6, 10, 0),
          child: _Notice('“${_tok.split(':')[1]}” is not a ${_tok.split(':')[0]}  ·  ${_qkeys[_tok.split(':')[0].toLowerCase()]!.take(3).join(', ')}', fill: N.g13),
        ),
      const SizedBox(height: 8),
      _Band(filters: _chips.length, shown: r.length, total: kItems.length, showFilters: false, extra: _chips.any((q) => q.key == 'kind') ? null : _mostly(r), onClear: () => setState(() {
            _chips.clear();
            _c.clear();
          })),
      Expanded(
        child: r.isEmpty
            ? _Empty('No match for these chips', const [], actions: [if (_chips.isNotEmpty) _Btn('Remove ${_chips.last.raw}', onTap: () => setState(() => _chips.removeLast())), _Btn('Clear', onTap: () => setState(() {
                      _chips.clear();
                      _c.clear();
                    }))])
            : _filtered(r, fade: true, hidden: kItems.length - r.length, row: (it, i) => _ItemRow(it, tokens: _tokens(_c.text), showKind: _chips.where((q) => q.key == 'kind').length != 1, showCat: _chips.where((q) => q.key == 'cat').length != 1)),
      ),
      _Status(_say, keys: const ['Space', 'Bksp']),
    ]);
  }
}

/// The chip in a field or a filter row: key g63, value g95 bold, a 16 px cross. 20 px tall.
class _FilterChip extends StatelessWidget {
  const _FilterChip(this.k, this.v, {this.onRemove, this.hoverCross});
  final String k, v;
  final VoidCallback? onRemove;
  final bool? hoverCross;
  @override
  Widget build(BuildContext context) => Container(
        height: 20,
        padding: const EdgeInsets.only(left: 6),
        decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(4)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          Text('$k:', style: T.label(N.g76)),
          const SizedBox(width: 2),
          Flexible(child: Text(v, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g95).copyWith(fontWeight: FontWeight.w600))),
          _Cross(onTap: onRemove, hover: hoverCross),
        ]),
      );
}

/// What a crowded field shows instead of the chips it folded.
class _MoreChip extends StatelessWidget {
  const _MoreChip(this.n);
  final int n;
  @override
  Widget build(BuildContext context) => Container(height: 20, padding: const EdgeInsets.symmetric(horizontal: 6), decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(4)), child: Center(widthFactor: 1, child: Text('+$n more', style: T.label(N.g76))));
}
