// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

/// The fixed preview surface: a 1 px g20 edge, the picture, whose frame it is + Hide on top, Playing or Paused + the item's name below.
class _SurfaceFrame extends StatelessWidget {
  const _SurfaceFrame({required this.w, required this.h, required this.picture, required this.sample, required this.playing, required this.name, this.onHide, this.hoverHide});
  final double w, h;
  final Widget picture;
  final bool sample, playing;
  final String name;
  final VoidCallback? onHide;
  final bool? hoverHide;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        height: h,
        child: DecoratedBox(
          decoration: BoxDecoration(border: Border.all(color: N.g20)),
          child: ClipRect(
            child: Stack(fit: StackFit.expand, children: [
              picture,
              _edge(_pill(sample ? 'Sample frame' : 'Your frame · Layer 3'), _H(onTap: onHide, force: hoverHide, builder: (context, h) => _pill('Hide', fill: h ? N.g26 : _scrim))),
              _edge(_pill(playing ? 'Playing' : 'Paused'), _pill(name, fill: N.g20), top: false),
            ]),
          ),
        ),
      );
}

/// What is left of the preview surface when it is hidden: one 26 px line and a Show word.
class _ClosedBar extends StatelessWidget {
  const _ClosedBar({this.onShow, this.hoverShow});
  final VoidCallback? onShow;
  final bool? hoverShow;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)),
        child: Row(children: [
          Expanded(child: Text('Preview hidden', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
          const SizedBox(width: 8),
          _Word('Show', onTap: onShow, hover: hoverShow),
        ]),
      );
}

class _BigPreview extends StatefulWidget {
  const _BigPreview({super.key, required this.w, this.open = true, this.sample = false});
  final double w;
  final bool open, sample;
  @override
  State<_BigPreview> createState() => _BigPreviewState();
}

class _BigPreviewState extends State<_BigPreview> {
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController(initialScrollOffset: 14 * 34.0 - 68);
  late bool _open = widget.open;
  bool _play = true;
  int _sel = 14;
  List<It> get _pool => kItems.where((i) => i.place != 'Project').toList();

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, _pool.length - 1));
      _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 34);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.space) {
      setState(() => _play = !_play);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final fw = widget.w - 20, fh = fw * 9 / 16, it = _pool[_sel];
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          AnimatedSize(
            duration: Mo.dur,
            curve: Mo.ease,
            alignment: Alignment.topCenter,
            child: _open
                ? Padding(
                    padding: const EdgeInsets.fromLTRB(10, 10, 10, 0),
                    child: _SurfaceFrame(
                      w: fw,
                      h: fh,
                      sample: widget.sample,
                      playing: _play,
                      name: it.name,
                      onHide: () => setState(() => _open = false),
                      picture: _Loop(running: _play, rest: .6, builder: (context, t) => _Fx(it.fx, t, base: _ownBase(sample: widget.sample))),
                    ),
                  )
                : SizedBox(width: widget.w, child: Padding(padding: const EdgeInsets.fromLTRB(10, 10, 10, 0), child: _ClosedBar(onShow: () => setState(() => _open = true)))),
          ),
          _pad(
            Text('${_sel + 1} of ${_pool.length}', maxLines: 1, overflow: TextOverflow.ellipsis, style: _num(N.g76)),
            t: 8,
            b: 8,
          ),
          Container(height: 1, color: N.rowLine),
          Expanded(
            child: _Fit(natural: _pool.length * 34.0, child: _QuietKind(_majority(_pool), child: _FadeBottom(child: ListView.builder(
              controller: _sc,
              padding: EdgeInsets.zero,
              itemExtent: 34,
              itemCount: _pool.length,
              itemBuilder: (context, i) => _ItemRow(_pool[i], selected: i == _sel, onTap: () {
                setState(() => _sel = i);
                _root.requestFocus();
              }),
            )))),
          ),
        ]),
      ),
    );
  }
}

// ================================================================================ B10  favourites, recents, shelves

/// Rows with arrow navigation over the flat list; the item behind each row is kept for the keys.
mixin _RowKeys<T extends StatefulWidget> on State<T> {
  final sc = ScrollController();
  int sel = 0;
  List<_Ent> entries = const [];
  List<It?> rowItems = const [];

  void moveSel(int d) {
    final cnt = _rowCount(entries);
    if (cnt == 0) return;
    setState(() => sel = (sel + d).clamp(0, cnt - 1));
    final (t, b) = _entSpan(entries, sel);
    _revealAt(sc, t, b);
  }

  It? get selItem => sel >= 0 && sel < rowItems.length ? rowItems[sel] : null;
}

/// The right end of a starred-list row: a quiet word (when it was used, how often) and the star.
class _StarTail extends StatelessWidget {
  const _StarTail(this.tail, {required this.on, this.onToggle});
  final String? tail;
  final bool on;
  final VoidCallback? onToggle;
  @override
  Widget build(BuildContext context) => Row(mainAxisSize: MainAxisSize.min, children: [
        if (tail != null) Padding(padding: const EdgeInsets.only(right: 4), child: Text(tail!, style: T.label(N.g76))),
        StarButton(on: on, onChanged: (_) => onToggle?.call()),
      ]);
}

class _StarRecent extends StatefulWidget {
  const _StarRecent({super.key, this.empty = false});
  final bool empty;
  @override
  State<_StarRecent> createState() => _StarRecentState();
}

class _StarRecentState extends State<_StarRecent> with _RowKeys<_StarRecent> {
  late final _root = FocusNode(onKeyEvent: _key);
  late final Map<String, int> _uses = {for (final i in kItems) i.name: widget.empty ? 0 : i.uses};
  late final List<String> _recent = widget.empty ? [] : (kItems.where((i) => i.recent >= 0 && i.place != 'Project').toList()..sort((a, b) => a.recent.compareTo(b.recent))).map((i) => i.name).toList();
  late final Set<String> _stars = widget.empty ? {} : {'Glow', 'Gaussian Blur', 'Pop In Overshoot', 'Ease Out Back (Soft)'};
  String _say = '';

  List<It> get _pool => kItems.where((i) => i.place != 'Project').toList();
  It _by(String n) => kItems.firstWhere((i) => i.name == n);

  void _build() {
    final out = <_Ent>[], items = <It?>[];
    void row(It it, {String? tail}) {
      final i = items.length;
      items.add(it);
      out.add(_Ent.row((s) => _ItemRow(it, selected: s, onTap: () => setState(() => sel = i), onUse: () => _use(it), right: _StarTail(tail, on: _stars.contains(it.name), onToggle: () => _toggle(it)))));
    }

    void hint(String s) => out.add(_Ent.row((_) => _HintRow(s), h: 36));
    final stars = _pool.where((i) => _stars.contains(i.name)).toList();
    out.add(_Ent.head('Starred', right: '${stars.length}'));
    stars.isEmpty ? hint('No stars yet') : stars.forEach(row);
    final rec = _recent.take(6).map(_by).toList();
    out.add(_Ent.head('Recent', right: '${rec.length}'));
    rec.isEmpty ? hint('None yet') : [for (final it in rec) row(it, tail: it.recent >= 0 && _recent.first != it.name ? _ago(it.recent) : 'just now')];
    final freq = _pool.where((i) => (_uses[i.name] ?? 0) >= 5).toList()..sort((a, b) => _uses[b.name]!.compareTo(_uses[a.name]!));
    out.add(_Ent.head('Frequent', right: '${math.min(5, freq.length)}'));
    freq.isEmpty ? hint('None yet') : [for (final it in freq.take(5)) row(it, tail: '${_uses[it.name]}×')];
    out.add(_Ent.head('All', right: '${_pool.length}'));
    for (final it in _pool) {
      row(it);
    }
    entries = out;
    rowItems = items;
  }

  void _toggle(It it) => setState(() {
        if (!_stars.remove(it.name)) _stars.add(it.name);
        _say = _stars.contains(it.name) ? 'Starred “${it.name}”.' : 'Unstarred “${it.name}”.';
      });

  void _use(It it) => setState(() {
        _uses[it.name] = (_uses[it.name] ?? 0) + 1;
        _recent.remove(it.name);
        _recent.insert(0, it.name);
        _say = 'Used “${it.name}” (${_uses[it.name]}×)';
      });

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      moveSel(e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyF && selItem != null) {
      _toggle(selItem!);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter && selItem != null) {
      _use(selItem!);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    _build();
    return Focus(
      focusNode: _root,
      autofocus: true,
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onTap: _root.requestFocus,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _pad(_Tabs(_places, 0, (_) {}, toggle: false), t: 10, b: 4),
          Expanded(child: _Fit(natural: _entH(entries), child: _QuietKind(_kindOf(_places[0]), child: _FlatList(entries: entries, sel: sel, sc: sc, fade: true)))),
          _Status(_say, keys: const ['F', 'Enter']),
        ]),
      ),
    );
  }
}

/// One line of the used shelf, 30 px: thumb, name, ×count when folded. [flash] 1..0 is the g26 glow of a row that was just used.
class _UsedRow extends StatelessWidget {
  const _UsedRow(this.it, this.count, {this.flash = 0});
  final It it;
  final int count;
  final double flash;
  @override
  Widget build(BuildContext context) => Container(
        height: 30,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        color: Color.lerp(_clear, N.g26, flash),
        child: Row(children: [
          _Thumb(it, w: 26, h: 18),
          const SizedBox(width: 8),
          Expanded(child: Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name())),
          if (count > 1) ...[const SizedBox(width: 8), Text('×$count', style: _num(N.g76))],
        ]),
      );
}

class _UsedStack extends StatefulWidget {
  const _UsedStack({super.key, required this.shelf});
  final int shelf;
  @override
  State<_UsedStack> createState() => _UsedStackState();
}

class _UsedStackState extends State<_UsedStack> {
  static const _cap = 12;
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  late final List<(String, int)> _stack = [for (final e in const ['Glow', 'Gaussian Blur', 'Pop In Overshoot', 'Hue / Saturation', 'Glitch RGB Split', 'Title In: Soft Rise', 'Film Grain', 'Curves', 'Echo', 'Typewriter', 'Bloom', 'Wave Warp'].indexed.take(widget.shelf)) (e.$2, const [4, 1, 3, 1, 2, 1, 1, 2, 1, 1, 1, 1][e.$1])];
  int _sel = 0, _stamp = 0;
  String _say = '';
  List<It> get _pool => kItems.where((i) => i.place != 'Project').toList();
  It _by(String n) => kItems.firstWhere((i) => i.name == n);

  void _use(It it) => setState(() {
        final at = _stack.indexWhere((e) => e.$1 == it.name);
        var cnt = 1;
        if (at >= 0) {
          cnt = _stack[at].$2 + 1;
          _stack.removeAt(at);
        }
        _stack.insert(0, (it.name, cnt));
        String? dropped;
        if (_stack.length > _cap) dropped = _stack.removeLast().$1;
        _stamp++;
        _say = at >= 0 ? '“${it.name}” ×$cnt' : (dropped != null ? 'Full: dropped “$dropped”' : 'Added “${it.name}”');
      });

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, _pool.length - 1));
      _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 34);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      _use(_pool[_sel]);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        focusNode: _root,
        autofocus: true,
        child: GestureDetector(
          behavior: HitTestBehavior.translucent,
          onTap: _root.requestFocus,
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _Sec('Used', right: Text('${_stack.length} of $_cap', style: T.label(N.g76))),
            SizedBox(
              height: math.min(_stack.length, 6) * 30.0 + (_stack.isEmpty ? 40 : 0),
              child: _stack.isEmpty
                  ? const Align(alignment: Alignment.topCenter, child: _HintRow('Nothing used yet'))
                  : ListView.builder(
                      padding: EdgeInsets.zero,
                      itemExtent: 30,
                      itemCount: _stack.length,
                      itemBuilder: (context, i) {
                        final it = _by(_stack[i].$1);
                        return TweenAnimationBuilder<double>(
                          key: ValueKey(i == 0 ? 'top$_stamp' : 'row${_stack[i].$1}'),
                          tween: Tween(begin: i == 0 && _stamp > 0 ? 1.0 : 0.0, end: 0.0),
                          duration: const Duration(milliseconds: 600),
                          curve: Curves.easeOut,
                          builder: (context, v, _) => _UsedRow(it, _stack[i].$2, flash: v),
                        );
                      },
                    ),
            ),
            Container(height: 1, margin: const EdgeInsets.only(top: 4), color: N.rowLine),
            _Sec('Effects and presets', right: Text('${_pool.length}', style: T.label(N.g76))),
            Expanded(
              child: _Fit(natural: _pool.length * 34.0, child: _QuietKind(_majority(_pool), child: _FadeBottom(child: ListView.builder(
                controller: _sc,
                padding: EdgeInsets.zero,
                itemExtent: 34,
                itemCount: _pool.length,
                itemBuilder: (context, i) => _ItemRow(_pool[i], selected: i == _sel, onTap: () {
                  setState(() => _sel = i);
                  _root.requestFocus();
                }, onUse: () => _use(_pool[i])),
              )))),
            ),
            _Status(_say, keys: const ['Enter']),
          ]),
        ),
      );
}

/// One collection slot, 24 px: its number key mark, its name and how many it holds. An unnamed slot is drawn as an empty slot: a dashed g26 outline and the word "empty" in g63.
class _SlotRow extends StatelessWidget {
  const _SlotRow(this.n, this.name, this.count, {this.on = false, this.hover = false});
  final int n, count;
  final String name;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => CustomPaint(
        foregroundPainter: name.isEmpty ? const DashedBox(N.g26, radius: 3) : null,
        child: Container(
        height: 24,
        color: on ? N.g20 : (hover ? N.g15 : null),
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(10, 0, 8, 0),
            child: Row(children: [
              _Kbd('$n', on: on),
              const SizedBox(width: 8),
              Expanded(child: Text(name.isEmpty ? 'empty' : name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(name.isEmpty ? N.g76 : (on ? N.g95 : N.g91)))),
              if (name.isNotEmpty) Text('$count', style: _num(N.g76)),
            ]),
          ),
          _GutterTick(on, height: 12),
        ]),
        ),
      );
}

/// The collections a row belongs to, as the same grey number plates the slot buttons carry (the slot being shown is not repeated); more than 3 ends in a +.
class _SlotBadges extends StatelessWidget {
  const _SlotBadges(this.slots, {this.skip = 0});
  final List<int> slots;
  final int skip;
  @override
  Widget build(BuildContext context) {
    final m = [for (final n in slots) if (n != skip) n];
    return Row(mainAxisSize: MainAxisSize.min, children: [
      for (final n in m.take(3)) Padding(padding: const EdgeInsets.only(left: 2), child: _Kbd('$n')),
      if (m.length > 3) Padding(padding: const EdgeInsets.only(left: 2), child: Text('+', style: _num(N.g76))),
    ]);
  }
}

class _Collections extends StatefulWidget {
  const _Collections({super.key, required this.w});
  final double w;
  @override
  State<_Collections> createState() => _CollectionsState();
}
