// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

/// One tab word, a choice control (DESIGN.md selected state): 22 px tall, 8 px sides; rest g63 text, hover g15, selected = g20 pill + g95 text + a 1 px accent underline. Used by the tab trough and the menu's family bar.
class _TabCell extends StatelessWidget {
  const _TabCell(this.text, {this.on = false, this.hover = false});
  final String text;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => ClipRRect(
        borderRadius: BorderRadius.circular(4),
        child: AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          height: 22,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          color: on ? N.g20 : (hover ? N.g15 : _clear),
          child: Stack(alignment: Alignment.center, children: [
            Text(text, maxLines: 1, softWrap: false, style: T.micro(on ? N.g95 : N.g76)),
            Positioned(left: 0, right: 0, bottom: 0, height: 1, child: ColoredBox(color: on ? Role.selected : _clear)),
          ]),
        ),
      );
}

/// Esc: first press clears the text, second leaves the field. Returns null when the key is not Esc.
KeyEventResult _esc(KeyEvent e, TextEditingController c, FocusNode n, void Function(String) say, {VoidCallback? cleared}) {
  if (e is! KeyDownEvent || e.logicalKey != LogicalKeyboardKey.escape) return KeyEventResult.ignored;
  if (c.text.isNotEmpty) {
    c.clear();
    cleared?.call();
    say('Cleared');
  } else {
    n.unfocus();
    say('Left the field');
  }
  return KeyEventResult.handled;
}

bool _isDown(KeyEvent e, LogicalKeyboardKey k) => (e is KeyDownEvent || e is KeyRepeatEvent) && e.logicalKey == k;

// ================================================================================ stage (caption outside, panel inside)

enum Hab { habit, addition, departs }

class _Caption extends StatelessWidget {
  const _Caption(this.id, this.title, this.hab, this.contract);
  final String id, title, contract;
  final Hab hab;
  @override
  Widget build(BuildContext context) {
    final (word, jp) = switch (hab) { Hab.habit => ('HABIT', '手癖'), Hab.addition => ('ADDITION', '追加'), Hab.departs => ('DEPARTS', '手癖から外れる') };
    return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
        Text(id, style: T.value(N.g95).copyWith(fontWeight: FontWeight.w700)),
        const SizedBox(width: 8),
        Flexible(child: Text(title, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
        const SizedBox(width: 8),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
          decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(4), border: Border(bottom: BorderSide(color: hab == Hab.departs ? Role.selected : N.g26))),
          child: Text('$word $jp', style: T.micro(hab == Hab.departs ? N.g95 : N.g76)),
        ),
      ]),
      const SizedBox(height: 6),
      Text(contract, style: T.label(N.g76).copyWith(height: 1.35)),
    ]);
  }
}

class _Stage extends StatefulWidget {
  const _Stage({super.key, required this.id, required this.title, required this.hab, required this.contract, required this.width, required this.body, this.fit = true});
  final String id, title, contract;
  final Hab hab;
  final double width;
  final Widget body;

  /// "Fit list": the frame is as tall as its content (no floor, max 700). false = the 700 tall stress frame.
  final bool fit;
  @override
  State<_Stage> createState() => _StageState();
}

class _StageState extends State<_Stage> {
  final _bodyKey = GlobalKey();
  double? _need;

  void _report(double? n) {
    if (!mounted) return;
    final same = n == null ? _need == null : (_need != null && (n - _need!).abs() < .5);
    if (!same) setState(() => _need = n);
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g07,
        child: LayoutBuilder(builder: (context, box) {
          final avail = box.maxHeight.isFinite ? box.maxHeight : 840.0;
          final legacy = (avail - 24 - 78 - 10).clamp(440.0, 700.0);
          // Fit: the body reports what its list needs (+2 for the frame edge). A body with nothing to fit (an empty state, tiles) keeps the default height.
          final h = !widget.fit ? 700.0 : (_need == null ? legacy : (_need! + 2).clamp(0.0, 700.0));
          return Center(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(12),
              child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                SizedBox(width: math.max(widget.width, 560), child: _Caption(widget.id, widget.title, widget.hab, widget.contract)),
                const SizedBox(height: 10),
                SizedBox(
                  width: widget.width,
                  height: h,
                  child: DecoratedBox(
                    decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
                    child: ClipRect(child: _FitScope(fit: widget.fit, bodyKey: _bodyKey, report: _report, child: KeyedSubtree(key: _bodyKey, child: widget.body))),
                  ),
                ),
              ]),
            ),
          );
        }),
      );
}

/// One use case = one panel draft. Two knobs are on every one: Panel width (Browser 320 / narrow dock 232) and Panel height (Fit list = the frame is as tall as its content, no floor / 700 tall = the stress frame).
WidgetbookUseCase _uc(String id, String title, Hab hab, String contract, Widget Function(BuildContext c, double w) build, {bool narrow = false}) => WidgetbookUseCase(
      name: '$id $title',
      builder: (c) {
        final w = c.knobs.object.dropdown<double>(label: 'Panel width', options: const [320.0, 232.0], initialOption: narrow ? 232.0 : 320.0, labelBuilder: (v) => v > 300 ? 'Browser 320' : 'Narrow dock 232');
        final fit = c.knobs.object.dropdown<String>(label: 'Panel height', options: const ['Fit list (default)', '700 tall'], initialOption: 'Fit list (default)', labelBuilder: (v) => v) != '700 tall';
        return _Stage(key: ValueKey('$id $title'), id: id, title: title, hab: hab, contract: contract, width: w, fit: fit, body: build(c, w));
      },
    );


// ================================================================================ shared list + tabs

void _revealAt(ScrollController sc, double top, double bot) {
  WidgetsBinding.instance.addPostFrameCallback((_) {
    if (!sc.hasClients) return;
    final vp = sc.position.viewportDimension;
    if (top < sc.offset) {
      sc.jumpTo(top);
    } else if (bot > sc.offset + vp) {
      sc.jumpTo(bot - vp);
    }
  });
}

/// A flat list of section headings and selectable rows (variable height). Selection is counted over rows only.
class _Ent {
  const _Ent.head(this.title, {this.right}) : row = null, h = 26;
  const _Ent.row(this.row, {this.h = 34}) : title = null, right = null;
  final String? title, right;
  final Widget Function(bool sel)? row;
  final double h;
  bool get isHead => row == null;
}

double _entH(List<_Ent> es) => es.fold<double>(0, (a, e) => a + e.h);

int _rowCount(List<_Ent> e) => e.where((x) => !x.isHead).length;

(double, double) _entSpan(List<_Ent> es, int rowIdx) {
  var y = 0.0, n = 0;
  for (final e in es) {
    if (!e.isHead) {
      if (n == rowIdx) return (y, y + e.h);
      n++;
    }
    y += e.h;
  }
  return (y, y);
}

class _FlatList extends StatelessWidget {
  const _FlatList({required this.entries, required this.sel, required this.sc, this.fade = false});
  final bool fade;
  final List<_Ent> entries;
  final int sel;
  final ScrollController sc;
  @override
  Widget build(BuildContext context) {
    var n = 0;
    final idx = [for (final e in entries) e.isHead ? -1 : n++];
    final list = ListView.builder(
      controller: sc,
      padding: EdgeInsets.zero,
      itemCount: entries.length + 1,
      itemBuilder: (context, i) {
        if (i == entries.length) return const SizedBox(height: 12); // air, so the last row is never cut by the footer
        final e = entries[i];
        return SizedBox(height: e.h, child: e.isHead ? _Sec(e.title!, right: e.right == null ? null : Text(e.right!, style: T.label(N.g76))) : e.row!(idx[i] == sel));
      },
    );
    return fade ? _FadeBottom(child: list) : list;
  }
}

/// Tabs for a place: words in a g07 trough, scrolls instead of shrinking the text. Tapping the selected one again clears it (-1 = all places). [hoverIndex] pins one tab's hover look (Parts).
class _Tabs extends StatelessWidget {
  const _Tabs(this.items, this.index, this.onChanged, {this.toggle = true, this.hoverIndex, this.chevron = false});
  final bool chevron;
  final List<String> items;
  final int index;
  final ValueChanged<int> onChanged;
  final bool toggle;
  final int? hoverIndex;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        width: double.infinity,
        padding: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6)),
        child: _FadeScroll(
          index: index,
          chevron: chevron,
          children: [
            for (var i = 0; i < items.length; i++)
              _H(
                onTap: () => onChanged(toggle && i == index ? -1 : i),
                force: hoverIndex == null ? null : hoverIndex == i,
                builder: (context, h) => _TabCell(items[i], on: i == index, hover: h),
              ),
          ],
        ),
      );
}

/// A row of cells that scrolls sideways, never cuts a word without saying so: a 16 px fade on the side that has more, and the chosen cell is kept in view.
class _FadeScroll extends StatefulWidget {
  const _FadeScroll({required this.children, this.index = -1, this.pad = EdgeInsets.zero, this.chevron = false});
  final bool chevron;
  final List<Widget> children;
  final int index;
  final EdgeInsets pad;
  @override
  State<_FadeScroll> createState() => _FadeScrollState();
}

class _FadeScrollState extends State<_FadeScroll> {
  final _sc = ScrollController();
  late final List<GlobalKey> _keys = List.generate(widget.children.length, (_) => GlobalKey());

  @override
  void initState() {
    super.initState();
    _sc.addListener(() => setState(() {}));
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        setState(() {});
        _show(false);
      }
    });
  }

  @override
  void didUpdateWidget(_FadeScroll o) {
    super.didUpdateWidget(o);
    if (o.index != widget.index) WidgetsBinding.instance.addPostFrameCallback((_) => _show(true));
  }

  void _show(bool animate) {
    final i = widget.index;
    if (!mounted || i < 0 || i >= _keys.length) return;
    final c = _keys[i].currentContext;
    if (c != null) Scrollable.ensureVisible(c, alignment: .5, duration: animate ? Mo.dur : Duration.zero, curve: Mo.ease);
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final has = _sc.hasClients && _sc.position.hasContentDimensions;
    final more = has && _sc.position.maxScrollExtent > 1, left = more && _sc.offset > 1, right = more && _sc.offset < _sc.position.maxScrollExtent - 1;
    final row = SingleChildScrollView(
      controller: _sc,
      scrollDirection: Axis.horizontal,
      physics: const ClampingScrollPhysics(),
      padding: widget.pad,
      child: Row(mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < widget.children.length; i++) ...[
          if (i > 0) const SizedBox(width: 2),
          KeyedSubtree(key: _keys[i], child: widget.children[i]),
        ],
      ]),
    );
    if (!left && !right) return row;
    final masked = ShaderMask(
      blendMode: BlendMode.dstIn,
      shaderCallback: (r) {
        final f = math.min(.5, (widget.chevron ? 28 : 16) / r.width);
        return ui.Gradient.linear(Offset.zero, Offset(r.width, 0), [Color(left ? 0x00FFFFFF : 0xFFFFFFFF), const Color(0xFFFFFFFF), const Color(0xFFFFFFFF), Color(right ? 0x00FFFFFF : 0xFFFFFFFF)], [0, f, 1 - f, 1]);
      },
      child: row,
    );
    if (!widget.chevron) return masked;
    // A › (or ‹) on the side that has more, so the cut is a cue and not a clipped word.
    return Stack(children: [
      Positioned.fill(child: masked),
      if (right) Positioned(right: 4, top: 0, bottom: 0, child: IgnorePointer(child: Center(child: Text('›', style: T.name(N.g76).copyWith(fontSize: 14, height: 1))))),
      if (left) Positioned(left: 4, top: 0, bottom: 0, child: IgnorePointer(child: Center(child: Text('‹', style: T.name(N.g76).copyWith(fontSize: 14, height: 1))))),
    ]);
  }
}

/// A scrolling list that says there is more below: a 20 px fade at the foot while the list can still scroll down, so a row is never cut without a cue.
class _FadeBottom extends StatefulWidget {
  const _FadeBottom({required this.child, this.top = false});
  final Widget child;
  /// Also fade the top edge while the list is scrolled away from its start (never at rest).
  final bool top;
  @override
  State<_FadeBottom> createState() => _FadeBottomState();
}

class _FadeBottomState extends State<_FadeBottom> {
  bool _more = false, _before = false;
  bool _on(ScrollMetrics m) {
    if (m.axis != Axis.vertical) return false;
    final more = m.extentAfter > 1, before = widget.top && m.extentBefore > 1;
    if ((more != _more || before != _before) && mounted) setState(() { _more = more; _before = before; });
    return false;
  }

  @override
  Widget build(BuildContext context) => NotificationListener<ScrollMetricsNotification>(
        onNotification: (n) => _on(n.metrics),
        child: NotificationListener<ScrollNotification>(
          onNotification: (n) => _on(n.metrics),
          child: ShaderMask(
            blendMode: BlendMode.dstIn,
            shaderCallback: (r) {
              final f = math.min(.5, 20 / r.height), t = math.min(.25, 12 / r.height);
              return ui.Gradient.linear(Offset.zero, Offset(0, r.height), [Color(_before ? 0x00FFFFFF : 0xFFFFFFFF), const Color(0xFFFFFFFF), const Color(0xFFFFFFFF), Color(_more ? 0x00FFFFFF : 0xFFFFFFFF)], [0, _before ? t : 0, 1 - f, 1]);
            },
            child: widget.child,
          ),
        ),
      );
}

/// The kind most of these rows share (always one, unlike [_dominant]): the rows of that kind leave the kind word off and only the exceptions say it.
String? _majority(Iterable<It> r) {
  final n = <String, int>{};
  for (final i in r) {
    n[i.kind] = (n[i.kind] ?? 0) + 1;
  }
  if (n.isEmpty) return null;
  return n.entries.reduce((a, b) => a.value >= b.value ? a : b).key;
}

Widget _pad(Widget c, {double l = 10, double t = 0, double r = 10, double b = 0}) => Padding(padding: EdgeInsets.fromLTRB(l, t, r, b), child: SizedBox(width: double.infinity, child: c));

// ================================================================================ B3-a  Find: unified query (always-on field)

class _FindA extends StatefulWidget {
  const _FindA({super.key, this.query = '', this.grouped = false, this.tab = -1, this.clearHover});
  final String query;
  final bool grouped;
  final int tab;

  /// Pins the hover look of the field's clear mark (a use case that shows it).
  final bool? clearHover;
  @override
  State<_FindA> createState() => _FindAState();
}
