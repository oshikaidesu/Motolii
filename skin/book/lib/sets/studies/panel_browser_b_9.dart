// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

class _LiveFolderState extends State<_LiveFolder> {
  final _files = <_DiskFile>[
    for (final n in ['A001_C003_0612_R2X5.mov', 'A001_C004_0612_R2X5.mov', 'A001_C005_0612_R2X5.mov', 'メイキング_001.mov', 'メイキング_002.mov', 'Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', 'drone_north_ridge.mp4', 'drone_north_ridge_b.mp4', 'BTS_stills_0412.jpg', 'BTS_stills_0413.jpg', 'ambience_wind.wav', 'slate_scene12.png']) _DiskFile(n),
  ];
  bool _offline = false;
  int _n = 0, _sel = -1;
  Timer? _tick1;

  @override
  void initState() {
    super.initState();
    _tick1 = Timer.periodic(const Duration(seconds: 1), (_) {
      if (mounted) setState(() {});
    });
  }

  @override
  void dispose() {
    _tick1?.cancel();
    super.dispose();
  }

  bool _fresh(_DiskFile f) => f.added != null && DateTime.now().difference(f.added!) < const Duration(seconds: 4);

  void _add() => setState(() {
        _n++;
        _files.insert(0, _DiskFile('new_clip_${_n.toString().padLeft(3, '0')}.mov', added: DateTime.now()));
      });

  void _remove() => setState(() {
        final f = _files.where((f) => f.onDisk && !_fresh(f)).toList();
        if (f.isNotEmpty) f[(_n * 5 + 3) % f.length].onDisk = false;
        _n++;
      });

  @override
  Widget build(BuildContext context) {
    final missing = _files.where((f) => !f.onDisk).length, shown = _files.length;
    final frame = _Frame(
      w: widget.w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(3)),
          const SizedBox(height: 8),
          _Sec('Places'),
          Container(
            padding: const EdgeInsets.all(_px),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: _offline ? N.g38 : N.g20)),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Row(children: [
                Expanded(child: _t('Footage / Day3_Shoot', T.title())),
                _Tag(_offline ? 'OFFLINE' : 'LIVE', hi: !_offline),
              ]),
              const SizedBox(height: 4),
              _t('/Volumes/RAID_A/Footage/2026/Day3_Shoot', _num()),
              if (_offline) ...[
                const SizedBox(height: 6),
                Text('見つかりません · $shown 件を保持', style: T.label(N.g91)),
                const SizedBox(height: 6),
                Row(children: [_Btn('Locate folder…', onTap: () => setState(() => _offline = false)), const SizedBox(width: 6), _Btn('Retry', onTap: () => setState(() => _offline = false))]),
              ],
            ]),
          ),
          const SizedBox(height: 8),
          _Sec('Files', n: missing > 0 ? '$shown · $missing missing' : '$shown'),
        ], bottom: 0),
        _Grow(
          child: _Fade(child: ListView.builder(
            shrinkWrap: _PanelH.fitOf(context),
            padding: const EdgeInsets.only(bottom: 12),
            itemExtent: 28,
            itemCount: _files.length,
            itemBuilder: (c, i) {
              final f = _files[i];
              final a = Asset(f.name, f.name.endsWith('.jpg') || f.name.endsWith('.png') ? Kind.image : (f.name.endsWith('.wav') ? Kind.audio : Kind.video), i + 3);
              return _Hit(
                onTap: () => setState(() => _sel = i),
                builder: (c, h, _) => _LiveRow(f.name, a, sel: _sel == i, hover: h, fresh: _fresh(f), onDisk: f.onDisk, offline: _offline, onLocate: () => setState(() => f.onDisk = true)),
              );
            },
          )),
        ),
      ]),
    );
    final side = _Side('Disk (outside the panel)', [
      _Btn('Drop a new file in', onTap: _offline ? null : _add),
      _Btn('Delete a file on disk', onTap: _offline ? null : _remove),
      _Btn(_offline ? 'Reconnect the drive' : 'Unplug the drive', onTap: () => setState(() => _offline = !_offline)),
    ]);
    return _withSide(frame, side);
  }
}

// ================================================================ B13 context filter: what to do with things that do not fit the selection

String _need(_Fx f) => f.isKey ? 'key' : (f.isText ? 'Text layer' : 'layer');

/// A layer card on the stand-in Stage (B13-c): `ok` = this layer fits what was pressed.
class _PickCard extends StatelessWidget {
  const _PickCard(this.name, this.kind, {required this.ok, this.hover = false});
  final String name, kind;
  final bool ok, hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 44,
        decoration: BoxDecoration(color: hover && ok ? N.g20 : N.g13, borderRadius: BorderRadius.circular(5), border: ok ? Border.all(color: _accent) : null),
        child: CustomPaint(
          painter: ok ? null : DashedBox(N.g38),
          child: Padding(padding: const EdgeInsets.all(8), child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.center, children: [_t(name, T.name(ok ? N.g95 : N.g63)), const SizedBox(height: 4), _t(kind, T.label(N.g76))])),
        ),
      );
}

class _Context extends StatefulWidget {
  const _Context({required this.w, required this.mode, this.sel = _Sel.text});
  final double w;
  final int mode; // 0 misfits are not shown, 1 misfits stay, dim, with a reason word, 2 press a misfit then pick a layer
  final _Sel sel;
  @override
  State<_Context> createState() => _ContextState();
}

class _ContextState extends State<_Context> {
  late _Sel _s = widget.sel;
  bool _all = false;
  _Fx? _pending;
  String? _msg;

  @override
  void didUpdateWidget(_Context o) {
    super.didUpdateWidget(o);
    if (o.sel != widget.sel) {
      _s = widget.sel;
      _pending = null;
    }
  }

  void _apply(_Fx f, String where) => setState(() {
        _pending = null;
        _msg = 'Applied ${f.en} to $where';
      });

  void _press(_Fx f) {
    final ok = _fits(f, _s);
    if (ok && _s != _Sel.none) {
      _apply(f, _selName(_s));
      return;
    }
    if (_s == _Sel.none) {
      setState(() => _msg = 'Select a target');
      return;
    }
    switch (widget.mode) {
      case 1:
        setState(() => _msg = '${f.en}: ${_why(f, _s)}');
      case 2:
        setState(() {
          _pending = f;
          _msg = 'Pick a ${_need(f)}';
        });
      default:
        break;
    }
  }

  bool _key(KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _pending != null) {
      setState(() {
        _pending = null;
        _msg = 'Cancelled';
      });
      return true;
    }
    return false;
  }

  Widget _stage() {
    final f = _pending!;
    const layers = [('タイトル', 'Text', _Sel.text), ('Subtitle', 'Text', _Sel.text), ('Logo Mark', 'Shape', _Sel.shape), ('BG Gradient', 'Solid', _Sel.shape)];
    return Container(
      margin: const EdgeInsets.only(top: 8),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6), border: Border.all(color: _accentLine)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [Expanded(child: _t('Pick a ${_need(f)} for ${f.en}', T.name(N.g95))), const _Kbd('Esc')]),
        const SizedBox(height: 6),
        Row(children: [
          for (final l in layers)
            Expanded(
              child: Padding(
                padding: const EdgeInsets.only(right: 4),
                child: Builder(builder: (c) {
                  final ok = !f.isKey && (f.isText ? l.$3 == _Sel.text : true);
                  final card = _Hit(
                    onTap: () {
                      if (ok) {
                        _apply(f, '${l.$1} (${l.$2})');
                      } else {
                        setState(() => _msg = '${l.$1}: ${l.$2}');
                      }
                    },
                    builder: (c, h, _) => _PickCard(l.$1, l.$2, ok: ok, hover: h),
                  );
                  return ok ? _Pulse(child: card) : card;
                }),
              ),
            ),
        ]),
        const SizedBox(height: 4),
        _standIn('Stage stand-in'),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) {
    final m = widget.mode;
    final fitting = _fxs.where((f) => _fits(f, _s)).toList();
    final hidden = _fxs.length - fitting.length;
    final showFits = m == 0 && !_all && _s != _Sel.none;
    final list = showFits ? fitting : _fxs;
    final cats = [for (final c in const [7, 8, 0, 1, 2, 3, 4, 5, 6]) if (list.any((f) => f.cat == c)) c];
    final none = _s == _Sel.none;
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 8),
            _TargetLine(_s),
            if (!none && m == 0 && hidden > 0) ...[
              const SizedBox(height: 8),
              SizedBox(
                height: 22,
                child: Row(children: [Text('$hidden hidden', style: T.label(N.g76)), const Spacer(), _Chip(_all ? 'Hide misfits' : 'Show all', on: _all, onTap: () => setState(() => _all = !_all))]),
              ),
            ],
            if (_pending != null) _stage(),
            const SizedBox(height: 6),
          ]),
          _Grow(
            child: _fadeFoot(ListView(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.fromLTRB(_px, 0, _px, 16), children: [
              for (final c in cats) ...[
                _Sec(_cats[c], n: '${list.where((f) => f.cat == c).length}'),
                _Grid(
                  n: list.where((f) => f.cat == c).length,
                  item: (i, w) {
                    final f = list.where((f) => f.cat == c).toList()[i];
                    final miss = !_fits(f, _s) && !none;
                    return _Hit(
                      cursor: miss && m == 1 ? SystemMouseCursors.basic : SystemMouseCursors.click,
                      onTap: () => _press(f),
                      builder: (cx, h, _) => _FxTile(f, w, hover: h, dim: miss && m == 1, reason: miss && m == 1 ? _why(f, _s) : null),
                    );
                  },
                ),
                const SizedBox(height: 10),
              ],
            ]), on: list.length > 12)),
          _Foot(msg: _msg, hints: switch (m) {
            0 => const [('click', 'apply')],
            1 => const [('click', 'apply to a tile that fits')],
            _ => const [('Esc', 'cancel')],
          }),
        ]),
      ),
    );
  }
}

// ================================================================ Parts @200%: the small parts of each option group, large, every state side by side

/// The parts are drawn by the very widgets the panels use (never a copy), laid out at their real size and then scaled 2x as vectors
/// (Transform.scale without a filter layer), so a 1 px line is a crisp 2 px line and a 4 px padding is a measurable 8 px.
const _zk = 2.0;

/// One labelled cell: a 10 px caption, then the part on its panel ground inside a 4 px margin, at 200 %. Every cell of a group is drawn on a plate of the same size ([sized]).
class _Cell extends StatelessWidget {
  const _Cell(this.cap, this.w, this.h, this.child, {this.ground = N.g10, this.keepH = false});
  final String cap;
  final double w, h;
  final Widget child;
  final Color ground;

  /// The plate keeps the height of its own content (cards: no empty band under the last bar); only the width is shared.
  final bool keepH;
  _Cell sized(double nw, double nh) => _Cell(cap, nw, keepH ? h : nh, child, ground: ground, keepH: keepH);
  @override
  Widget build(BuildContext context) {
    final lw = math.max(w, 56.0) + 8, lh = h + 8;
    return SizedBox(
      width: lw * _zk,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        ConstrainedBox(constraints: const BoxConstraints(minHeight: 26), child: Padding(padding: const EdgeInsets.only(bottom: 2), child: Text(cap, softWrap: true, style: T.label(N.g76).copyWith(height: 1.25)))),
        DecoratedBox(
          position: DecorationPosition.foreground,
          decoration: BoxDecoration(border: Border.all(color: N.g20)),
          child: SizedBox(
            width: lw * _zk,
            height: lh * _zk,
            child: ClipRect(
              child: OverflowBox(
                alignment: Alignment.topLeft,
                minWidth: lw,
                maxWidth: lw,
                minHeight: lh,
                maxHeight: lh,
                child: Transform.scale(scale: _zk, alignment: Alignment.topLeft, child: ColoredBox(color: ground, child: Padding(padding: const EdgeInsets.all(4), child: Align(alignment: Alignment.topLeft, child: child)))),
              ),
            ),
          ),
        ),
      ]),
    );
  }
}

Widget _fixed(double w, Widget child) => SizedBox(width: w, child: child);

/// Every plate of a group the same size: the widest and the tallest cell set it, so a row of plates lines up.
List<Widget> _plates(List<Widget> cells) {
  final cs = cells.whereType<_Cell>().toList();
  if (cs.isEmpty) return cells;
  final w = cs.map((c) => c.w).reduce(math.max), h = cs.map((c) => c.h).reduce(math.max);
  return [for (final c in cells) c is _Cell ? c.sized(w, h) : c];
}

/// A Parts sheet: groups of cells, each group a titled wrap of equal plates. [absent] says which states do not exist for these parts (never silently left out).
WidgetbookUseCase _parts(String name, String line, List<(String, List<Widget>)> groups, {required String absent}) => WidgetbookUseCase(
      name: 'Parts @200% · $name',
      builder: (c) => ColoredBox(color: N.g07, child: _PartsSheet(key: ValueKey(name), name: name, line: line, absent: absent, groups: [for (final (t, cs) in groups) (t, _plates(cs))])),
    );

/// The sheet scrolls both ways with visible bars and by keyboard (PageUp / PageDown / Home / End / arrows); the JUMP TO bar goes to a group. A cell wider than the window is reached by the horizontal bar, never lost.
class _PartsSheet extends StatefulWidget {
  const _PartsSheet({super.key, required this.name, required this.line, required this.absent, required this.groups});
  final String name, line, absent;
  final List<(String, List<Widget>)> groups;
  @override
  State<_PartsSheet> createState() => _PartsSheetState();
}

class _PartsSheetState extends State<_PartsSheet> {
  final _v = ScrollController(), _h = ScrollController();
  late final List<GlobalKey> _keys = [for (final _ in widget.groups) GlobalKey()];
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
  Widget build(BuildContext context) => Focus(
        autofocus: true,
        onKeyEvent: _key,
        child: LayoutBuilder(builder: (c, box) {
          // The sheet is as wide as the window, or as the widest plate if that is wider: then it scrolls sideways instead of being cut.
          final widest = [for (final g in widget.groups) for (final cell in g.$2.whereType<_Cell>()) (math.max(cell.w, 56.0) + 8) * _zk].fold<double>(0, math.max);
          final inner = math.max(box.maxWidth - 32 - 12, math.max(widest, 680.0));
          return RawScrollbar(
            controller: _v,
            thumbVisibility: true,
            thickness: 6,
            radius: const Radius.circular(3),
            thumbColor: N.g44,
            child: RawScrollbar(
              controller: _h,
              thumbVisibility: true,
              thickness: 6,
              radius: const Radius.circular(3),
              thumbColor: N.g44,
              notificationPredicate: (n) => n.depth == 1,
              scrollbarOrientation: ScrollbarOrientation.bottom,
              child: SingleChildScrollView(
                controller: _v,
                child: SingleChildScrollView(
                  controller: _h,
                  scrollDirection: Axis.horizontal,
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(16, 16, 28, 28),
                    child: SizedBox(
                      width: inner,
                      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Row(children: [Text('PARTS', style: T.name(N.g95)), const SizedBox(width: 8), Text('@200%', style: T.micro(N.g76)), const SizedBox(width: 8), Flexible(child: Text(widget.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title()))]),
                        const SizedBox(height: 8),
                        ConstrainedBox(constraints: const BoxConstraints(maxWidth: 720), child: Text(widget.line, style: T.label(N.g76).copyWith(height: 1.35))),
                        const SizedBox(height: 4),
                        ConstrainedBox(constraints: const BoxConstraints(maxWidth: 720), child: Text('Not drawn, because they do not exist: ${widget.absent}', style: T.label(N.g76).copyWith(height: 1.35))),
                        const SizedBox(height: 10),
                        Wrap(spacing: 12, runSpacing: 6, crossAxisAlignment: WrapCrossAlignment.center, children: [
                          Text('JUMP TO', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                          for (var i = 0; i < widget.groups.length; i++) _Hit(onTap: () => _go(i), builder: (c, h, _) => Text(widget.groups[i].$1, maxLines: 1, style: T.label(h ? N.g95 : N.g76).copyWith(decoration: h ? TextDecoration.underline : TextDecoration.none, decorationColor: N.g95))),
                          Text('(PageDown, End and the arrows scroll)', style: T.label(N.g76)),
                        ]),
                        for (var i = 0; i < widget.groups.length; i++) ...[
                          const SizedBox(height: 24),
                          KeyedSubtree(key: _keys[i], child: Text(widget.groups[i].$1.toUpperCase(), style: T.micro(N.g76).copyWith(letterSpacing: .8))),
                          const SizedBox(height: 8),
                          Wrap(spacing: 12, runSpacing: 12, children: widget.groups[i].$2),
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
      );
}

_Fx _fx(String en) => _fxs.firstWhere((f) => f.en == en);
Asset _asset(Kind k) => [..._mediaShelf, ...sampleAssets].firstWhere((a) => a.kind == k, orElse: () => _mediaShelf.first);

const _noTag = 'tag and key mark are labels, so they have no hover, pressed, focus or disabled look';
