// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

class _PrefixState extends State<_Prefix> {
  late final _c = TextEditingController(text: widget.query);
  late final _n = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  int _sel = 0;
  String _say = 'Layer 3 · Title';

  String get _q => _c.text;
  String get _mark => _q.isNotEmpty && '#>@'.contains(_q[0]) ? _q[0] : '';
  String get _rest => _mark.isEmpty ? _q : _q.substring(1).trim();

  List<_Ent> _entries() {
    final toks = _tokens(_rest), out = <_Ent>[];
    var k = 0;
    _Ent tag(MapEntry<String, int> e) {
      final i = k++;
      return _Ent.row((s) => _PlainRow(title: '#${e.key}', sub: 'tag', right: '${e.value}', count: true, selected: s, onTap: () => setState(() => _sel = i), toks: toks));
    }

    _Ent act((String, String, String) a) {
      final i = k++;
      return _Ent.row((s) => _PlainRow(title: a.$1, sub: a.$2, right: a.$3, kbd: true, selected: s, onTap: () => setState(() => _sel = i), toks: toks));
    }

    _Ent lay((String, String) l) {
      final i = k++;
      return _Ent.row((s) => _PlainRow(title: l.$1, sub: 'Layer', right: l.$2, selected: s, onTap: () => setState(() => _sel = i), toks: toks));
    }

    _Ent item(It it) {
      final i = k++;
      return _Ent.row((s) => _ItemRow(it, selected: s, tokens: toks, showKind: false, onTap: () => setState(() => _sel = i)));
    }

    final tags = _tagCounts().entries.where((e) => toks.every(e.key.contains)).toList()..sort((a, b) => b.value.compareTo(a.value));
    final acts = _actions.where((a) => toks.every((t) => '${a.$1} ${a.$2}'.toLowerCase().contains(t))).toList();
    final lays = _layers.where((l) => toks.every((t) => '${l.$1} ${l.$2}'.toLowerCase().contains(t))).toList();
    final items = _match(kItems, _rest);

    void section(String name, List<_Ent> rows, int total, int cap) {
      if (rows.isEmpty) return;
      out.add(_Ent.head(name, right: '$total'));
      out.addAll(rows.take(cap));
      if (total > cap) out.add(_Ent.row((_) => _MoreLine('+${total - cap} more'), h: 24));
    }

    if (_q.isEmpty) {
      out.add(const _Ent.head('Marks'));
      for (final m in const [('#', 'tag', '#soft'), ('>', 'action', '>reset'), ('@', 'layer', '@title')]) {
        final i = k++;
        out.add(_Ent.row((s) => _PlainRow(title: '${m.$1}  ${m.$2}', sub: m.$3, right: '›', selected: s, onTap: () => setState(() {
              _sel = i;
              _c.value = TextEditingValue(text: m.$1, selection: const TextSelection.collapsed(offset: 1));
              _n.requestFocus();
            }))));
      }
    }
    switch (_mark) {
      case '#':
        section('Tags', [for (final e in tags) tag(e)], tags.length, 99);
      case '>':
        section('Actions', [for (final a in acts) act(a)], acts.length, 99);
      case '@':
        section('Layers', [for (final l in lays) lay(l)], lays.length, 99);
      default:
        final it = [for (final i in items.take(4)) item(i)];
        section('Effects, presets, media', it, items.length, 4);
        section('Tags', [for (final e in tags.take(2)) tag(e)], tags.length, 2);
        section('Actions', [for (final a in acts.take(2)) act(a)], acts.length, 2);
        section('Layers', [for (final l in lays.take(2)) lay(l)], lays.length, 2);
    }
    return out;
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final es = _entries(), cnt = _rowCount(es);
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cnt == 0 ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, cnt - 1));
      final (t, b) = _entSpan(es, _sel);
      _revealAt(_sc, t, b);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter && cnt > 0) {
      setState(() => _say = 'Row ${_sel + 1} used');
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s));
  }

  String _area() => switch (_mark) { '#' => 'tags', '>' => 'actions', '@' => 'layers', _ => 'all areas' };

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: _c,
        builder: (context, _) {
          final es = _entries();
          return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(_Search(controller: _c, node: _n, hint: 'Search, or use # > @', onChanged: (_) => setState(() => _sel = 0)), t: 10),
            _pad(
              _AreaLine(_area(), _mark),
              t: 8,
              b: 4,
            ),
            Expanded(
              child: es.isEmpty
                  ? _Empty('Nothing in ${_area()} for “$_rest”', const [], actions: [_Btn('Search all areas', onTap: () => setState(() => _c.text = _rest))])
                  : _Fit(natural: _entH(es), child: _FlatList(entries: es, sel: _sel, sc: _sc)),
            ),
            _Status(_say),
          ]);
        },
      );
}

class _PlainRow extends StatelessWidget {
  const _PlainRow({required this.title, required this.sub, required this.right, required this.selected, this.onTap, this.toks = const [], this.kbd = false, this.count = false, this.hover});
  final String title, sub, right;

  /// Three kinds of right-hand word, three looks: [kbd] a key mark (the shortcut), [count] the mono number, else a quiet kind word.
  final bool selected, kbd, count;
  final bool? hover;
  final VoidCallback? onTap;
  final List<String> toks;
  @override
  Widget build(BuildContext context) => _H(
        onTap: onTap,
        force: hover,
        builder: (context, hover) => AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          color: selected ? N.g20 : (hover ? N.g15 : _clear),
          child: Stack(fit: StackFit.expand, children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 10),
              child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                Expanded(
                  child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                    _Hl(title, toks, on: selected),
                    const SizedBox(height: 4),
                    Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
                  ]),
                ),
                const SizedBox(width: 8),
                if (kbd) _Kbd(right) else Text(right, style: count ? _num(N.g76) : T.micro(N.g63)),
              ]),
            ),
            _GutterTick(selected),
          ]),
        ),
      );
}

// ================================================================================ B3-d  Empty query: recent, then what fits the selection

/// The quiet reason a row is listed (g63, at most 92 px, ellipsis).
class _Why extends StatelessWidget {
  const _Why(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => ConstrainedBox(constraints: const BoxConstraints(maxWidth: 92), child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.right, style: T.label(N.g76)));
}

class _Freq extends StatefulWidget {
  const _Freq({super.key, required this.sel, required this.history});
  final int sel; // 0 none, 1 layer (Text), 2 key
  final bool history;
  @override
  State<_Freq> createState() => _FreqState();
}

class _FreqState extends State<_Freq> {
  final _c = TextEditingController();
  late final _n = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  int _sel = 0;
  late String _say = widget.sel == 0 ? 'No selection' : (widget.sel == 1 ? 'Layer 3 · Title' : 'Key · Position X');

  List<It> get _all => kItems.where((i) => i.place != 'Project').toList();

  (List<It>, List<It>, List<It>) _groups() {
    bool ok(It i) => widget.sel == 0 || (widget.sel == 2 ? i.applies != 'layer' : i.applies != 'key');
    final recent = widget.history ? (_all.where((i) => i.recent >= 0 && ok(i)).toList()..sort((a, b) => a.recent.compareTo(b.recent))).take(5).toList() : <It>[];
    var fit = <It>[];
    if (widget.sel == 1) {
      fit = _all.where((i) => i.applies != 'key' && !recent.contains(i)).toList()..sort((a, b) => ((b.tags.contains('title') ? 100 : 0) + b.uses).compareTo((a.tags.contains('title') ? 100 : 0) + a.uses));
      fit = fit.take(5).toList();
    } else if (widget.sel == 2) {
      fit = _all.where((i) => i.applies != 'layer' && !recent.contains(i)).toList();
    }
    final rest = _all.where((i) => !recent.contains(i) && !fit.contains(i) && ok(i)).toList()..sort((a, b) => widget.history ? b.uses.compareTo(a.uses) : 0);
    return (recent, fit, rest);
  }

  String _fitWhy(It i) => widget.sel == 2 ? 'curve for keys' : (i.tags.contains('title') ? 'made for text' : 'used on ${i.uses ~/ 4 + 1} text layers');

  List<_Ent> _entries() {
    final out = <_Ent>[];
    var k = 0;
    _Ent row(It it, String why) {
      final i = k++;
      return _Ent.row((s) => _ItemRow(it, selected: s, tokens: _tokens(_c.text), right: why.isEmpty ? null : _Why(why), onTap: () => setState(() => _sel = i), onUse: () => setState(() => _say = 'Applied “${it.name}”')));
    }

    if (_c.text.trim().isNotEmpty) {
      final r = _match(_all, _c.text);
      out.add(_Ent.head('Results', right: '${r.length}'));
      for (final it in r) {
        out.add(row(it, ''));
      }
      return out;
    }
    final (recent, fit, rest) = _groups();
    out.add(_Ent.head('Recent', right: widget.history ? 'all projects' : null));
    if (recent.isEmpty) {
      out.add(_Ent.row((s) => const _HintRow('None yet'), h: 36));
    }
    for (final it in recent) {
      out.add(row(it, _ago(it.recent)));
    }
    if (widget.sel == 0) {
      out.add(const _Ent.head('Fits your selection'));
      out.add(_Ent.row((s) => const _HintRow('No selection'), h: 36));
    } else {
      out.add(_Ent.head('Fits ${widget.sel == 1 ? 'Layer 3 · Title (Text)' : 'Key · Position X'}'));
      for (final it in fit) {
        out.add(row(it, _fitWhy(it)));
      }
    }
    out.add(_Ent.head('All others', right: '${rest.length}'));
    for (final it in rest) {
      out.add(row(it, widget.history && it.uses > 0 ? '${it.uses}×' : ''));
    }
    return out;
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    final es = _entries(), cnt = _rowCount(es);
    if (_isDown(e, LogicalKeyboardKey.arrowDown) || _isDown(e, LogicalKeyboardKey.arrowUp)) {
      setState(() => _sel = cnt == 0 ? 0 : (_sel + (e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1)).clamp(0, cnt - 1));
      final (t, b) = _entSpan(es, _sel);
      _revealAt(_sc, t, b);
      return KeyEventResult.handled;
    }
    return _esc(e, _c, n, (s) => setState(() => _say = s));
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: _c,
        builder: (context, _) => Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          _pad(_Search(controller: _c, node: _n, hint: 'Add effect…', onChanged: (_) => setState(() => _sel = 0)), t: 10, b: 4),
          Expanded(child: _Fit(natural: _entH(_entries()), child: _QuietKind(_majority(_all), child: _FlatList(entries: _entries(), sel: _sel, sc: _sc, fade: true)))),
          _Status(_say),
        ]),
      );
}

// ================================================================================ B2/B4  narrowing, and showing that a filter is on

const _groups = <String, List<String>>{
  'Kind': ['Effect', 'Preset', 'Media'],
  'Category': ['Blur', 'Color', 'Glow', 'Distort', 'Stylize', 'Time', 'Generate', 'Title', 'Motion', 'Ease'],
  'Character': ['Soft', 'Glitch', 'Retro', 'Bright', 'Grade', 'Moving', 'Loop'],
  'Source': ['Built-in', 'Mine', 'Project'],
};

bool _in(It i, String g, String v) => switch (g) {
      'Kind' => i.kind == v,
      'Category' => i.cat == v,
      'Source' => i.source == v,
      _ => i.tags.contains(v.toLowerCase()),
    };

typedef _Sel = Map<String, Set<String>>;

bool _pass(It i, _Sel s, {String? skip}) => s.entries.every((e) => e.key == skip || e.value.isEmpty || e.value.any((v) => _in(i, e.key, v)));

int _nFilters(_Sel s) => s.values.fold(0, (a, b) => a + b.length);

/// A filter that fixes a fact says it once: with exactly one Kind chosen the rows drop their kind label, with exactly one Category chosen their second line drops the category. (Two kinds chosen: the label tells them apart, so it stays.)
bool _kindFixed(Set<String> kinds) => kinds.length == 1;

Widget _maybeFade(bool on, Widget w) => on ? _FadeBottom(child: w) : w;

/// A tiny list builder for the filter panels: rows are 34 px, top-aligned. The count line above ("8 of 72") already says what is hidden, so there is no foot ([quiet]); a panel whose count line shows only the shown number passes quiet false and gets "N more hidden by the filters".
/// When one kind is more than 80% of the rows, the rows leave it off and only the exceptions say it.
Widget _filtered(List<It> r, {required int hidden, required Widget Function(It, int) row, ScrollController? sc, bool quiet = true, bool fade = false, bool useMajority = false}) => _QuietKind(
      useMajority ? _majority(r) : _dominant(r, (i) => i.kind),
      child: _Results(
        list: _maybeFade(fade, ListView.builder(controller: sc, padding: EdgeInsets.zero, itemExtent: 34, itemCount: r.length, itemBuilder: (context, i) => row(r[i], i))),
        rows: r.length,
        more: hidden,
        where: 'hidden by the filters',
        foot: !quiet && hidden > 0,
      ),
    );

/// A filter chip, 22 px, on the panel plate g10: rest g15 (visibly above the plate), hover g20, pressed g20 + the accent line at 50%, on g26 + a 1 px accent line, on + hover g26 + the line + a g63 edge,
/// keyboard focus a 1 px g63 edge. Count 0 = disabled (never shown at rest: the group folds these into one "+N hidden tags" word; when shown, no fill, g63 word, a dashed g56 hairline, arrow cursor).
/// A long word never widens the group: it takes the room it is given and ends in an ellipsis. hover / focus / pressed pin a state (Parts).
class _ChipBtn extends StatefulWidget {
  const _ChipBtn(this.text, {this.count, this.on = false, this.onTap, this.hover, this.focus, this.pressed});
  final String text;
  final int? count;
  final bool on;
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  State<_ChipBtn> createState() => _ChipBtnState();
}

class _ChipBtnState extends State<_ChipBtn> {
  bool _h = false, _f = false, _p = false;
  @override
  Widget build(BuildContext context) {
    final w = widget, zero = w.count == 0 && !w.on;
    final hv = !zero && (w.hover ?? _h), fc = !zero && (w.focus ?? _f), pr = !zero && (w.pressed ?? _p);
    final fill = zero ? _clear : (w.on ? N.g26 : (pr || hv ? N.g20 : N.g15));
    final line = w.on ? Role.selected : (pr ? Role.selected.withValues(alpha: .5) : _clear);
    final edge = fc ? N.g63 : (w.on && hv ? N.g63 : _clear);
    final body = ClipRRect(
      borderRadius: BorderRadius.circular(4),
      child: AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 22,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: edge)),
        color: fill,
        child: Stack(alignment: Alignment.center, children: [
          Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Flexible(child: Text(w.text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(w.on ? N.g95 : (zero ? N.g63 : N.g91)))),
            if (w.count != null) ...[const SizedBox(width: 4), Text('${w.count}', style: _num(w.on ? N.g95 : N.g76))],
          ]),
          Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: line)),
        ]),
      ),
    );
    return FocusableActionDetector(
      enabled: !zero,
      onShowHoverHighlight: (v) => setState(() => _h = v),
      onShowFocusHighlight: (v) => setState(() => _f = v),
      mouseCursor: zero ? SystemMouseCursors.basic : SystemMouseCursors.click,
      actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) {
        w.onTap?.call();
        return null;
      })},
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: zero ? null : w.onTap,
        onTapDown: zero ? null : (_) => setState(() => _p = true),
        onTapUp: (_) => setState(() => _p = false),
        onTapCancel: () => setState(() => _p = false),
        child: zero ? CustomPaint(foregroundPainter: const DashedBox(N.g56, radius: 4), child: body) : body,
      ),
    );
  }
}

/// Clear, as a button: g76 word and a g26 hairline, g20 fill on hover, pressed = g20 + the accent line at 50%, keyboard focus a 1 px g63 edge; with nothing on it is dimmed (g63 word, g20 hairline, no hover, arrow cursor). hover / focus / pressed pin a state (Parts).
class _ClearBtn extends StatelessWidget {
  const _ClearBtn({this.onTap, this.hover, this.focus, this.pressed});
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  Widget build(BuildContext context) {
    final off = onTap == null;
    return _Ctl(
      onTap: onTap,
      hover: off ? false : hover,
      focus: focus,
      pressed: pressed,
      builder: (context, h, f, p) => _ctlBody(
        height: 20,
        pad: const EdgeInsets.symmetric(horizontal: 8),
        fill: h || p ? N.g20 : _clear,
        edge: f ? N.g63 : (off ? N.g20 : N.g26),
        pressed: p,
        child: Center(widthFactor: 1, child: Text('Clear', style: T.label(off ? N.g63 : N.g76))),
      ),
    );
  }
}

/// The results band: how many filters are on, how many results, one Clear button.
class _Band extends StatelessWidget {
  const _Band({required this.filters, required this.shown, required this.total, required this.onClear, this.extra, this.hoverClear, this.focusClear, this.pressClear, this.showFilters = true, this.showCount = true});
  final int filters, shown, total;

  /// false = the chips / slot / tab already say how many filters are on, so the band does not count them again; [showCount] false = shown and total are not restated either (only [extra]).
  final bool showFilters, showCount;
  final VoidCallback onClear;
  final bool? hoverClear, focusClear, pressClear;
  final String? extra;
  @override
  Widget build(BuildContext context) => Container(
        height: 28,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(color: N.g13, border: Border.symmetric(horizontal: BorderSide(color: N.rowLine))),
        child: Row(children: [
          if (filters == 0)
            Expanded(child: Text('$total items', style: T.label(N.g76)))
          else
            Expanded(child: _bandText()),
          const SizedBox(width: 8),
          _ClearBtn(onTap: filters > 0 ? onClear : null, hover: hoverClear, focus: focusClear, pressed: pressClear),
        ]),
      );

  Widget _bandText() {
    if (showFilters) {
      return Text.rich(TextSpan(children: [TextSpan(text: '$filters ${filters == 1 ? 'filter' : 'filters'}', style: T.name(N.g95)), TextSpan(text: '  ·  ${shown == 0 ? 'no results' : '$shown of $total'}${extra == null ? '' : '  ·  $extra'}', style: T.label(N.g76))]), maxLines: 1, overflow: TextOverflow.ellipsis);
    }
    final parts = [if (showCount) (shown == 0 ? 'no results' : '$shown of $total'), ?extra];
    return Text(parts.join('  ·  '), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g95));
  }
}

/// An active filter, as the same filled chip object B2 draws: 22 px, rest g15, hover g20, pressed g20 + the accent line at 50%, keyboard focus a 1 px g63 edge; g91 word and an x mark inside the chip (g63, g95 when the chip is hot). The whole chip drops the filter. hover / focus / pressed pin a state (Parts).
class _ActiveChip extends StatelessWidget {
  const _ActiveChip(this.text, {this.onTap, this.hover, this.focus, this.pressed});
  final String text;
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
          pad: const EdgeInsets.only(left: 8, right: 2),
          fill: h || p ? N.g20 : N.g15,
          edge: f ? N.g63 : _clear,
          pressed: p,
          child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
            Flexible(child: Text(text, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g91))),
            const SizedBox(width: 2),
            SizedBox(width: 16, height: 20, child: Center(child: Text('×', style: T.name(h || p ? N.g95 : N.g63).copyWith(fontSize: 14, height: 1)))),
          ]),
        ),
      );
}
