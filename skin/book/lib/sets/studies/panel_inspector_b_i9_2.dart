// Part of panel_inspector_b_i9.dart (cut there by size, as it was).
part of 'panel_inspector_b.dart';

class _LassoPanelState extends State<_LassoPanel> {
  final pts = <Offset>[];
  Set<int> sel = {};
  bool bound = false;
  double now = 100, lo = 100, hi = 250;
  static const mapH = 176.0;

  List<Offset> _dots(double w) {
    final r = math.Random(7), n = widget.count, cols = (n / 3).ceil();
    return [for (var i = 0; i < n; i++) Offset(14 + (i % cols + .5) * (w - 28) / cols + (r.nextDouble() - .5) * 14, 14 + (i ~/ cols + .5) * (mapH - 28) / 3 + (r.nextDouble() - .5) * 12)];
  }

  bool _locked(int i) => widget.locks && i % 7 == 3;

  @override
  void initState() {
    super.initState();
    _seed();
  }

  /// The knobs start the panel in a state: nothing drawn, a loop drawn, or followed.
  void _seed() {
    final mapW = widget.w - 24, dots = _dots(mapW);
    pts.clear();
    sel = {};
    bound = false;
    if (widget.drawn || widget.followed) {
      pts.addAll([Offset(6, 6), Offset(mapW * .56, 6), Offset(mapW * .56, mapH - 6), Offset(6, mapH - 6)]);
      sel = {for (var i = 0; i < dots.length; i++) if (!_locked(i) && (Path()..addPolygon(pts, true)).contains(dots[i])) i};
      bound = widget.followed;
    }
  }

  @override
  void didUpdateWidget(_LassoPanel old) {
    super.didUpdateWidget(old);
    if (old.count != widget.count || old.locks != widget.locks || old.drawn != widget.drawn || old.followed != widget.followed) {
      _seed();
    }
  }

  void _finish(List<Offset> dots) {
    if (pts.length < 3) {
      pts.clear();
      return;
    }
    final path = Path()..addPolygon(pts, true);
    sel = {for (var i = 0; i < dots.length; i++) if (!_locked(i) && path.contains(dots[i])) i};
    bound = false;
  }

  @override
  Widget build(BuildContext context) {
    final mapW = widget.w - 24;
    final dots = _dots(mapW);
    final lockedAround = widget.locks ? List.generate(dots.length, (i) => i).where(_locked).length : 0;
    return _Panel(
      title: 'star_burst',
      sub: 'Relations · Opacity',
      w: widget.w,
      h: widget.h,
      child: _Lv(children: [
        const _Sect('Followers'),
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 4, 12, 0),
          child: GestureDetector(
            onPanStart: (d) => setState(() {
              pts
                ..clear()
                ..add(d.localPosition);
              sel = {};
              bound = false;
            }),
            onPanUpdate: (d) => setState(() => pts.add(d.localPosition)),
            onPanEnd: (_) => setState(() => _finish(dots)),
            child: Container(
              width: mapW,
              height: mapH,
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
              child: CustomPaint(painter: _LassoPaint(dots, sel, pts, {for (var i = 0; i < dots.length; i++) if (_locked(i)) i}, bound), child: Align(alignment: Alignment.topLeft, child: Padding(padding: const EdgeInsets.all(8), child: Text('Stage  1920 × 1080', style: T.label(N.g76))))),
            ),
          ),
        ),
        _gap(4),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
          child: Row(children: [
            Text('${sel.length}', style: _cnt(sel.isEmpty ? N.g63 : N.g95)),
            Text(' of ${widget.count}', style: _cnt(N.g76)),
            const Spacer(),
            if (lockedAround > 0) ...[
              Container(width: 8, height: 8, decoration: BoxDecoration(shape: BoxShape.circle, border: Border.all(color: N.g63))), // the hollow dot of a locked layer, as on the map
              const SizedBox(width: 4),
              Text('$lockedAround', style: _cnt(N.g76)),
            ],
          ]),
        ),
        const _Sect('Range'),
        _VRow('Opacity now', LayoutBuilder(builder: (context, b) => MiniSlider(value: now, min: 0, max: 100, width: b.maxWidth, onChanged: (v) => setState(() => now = v)))),
        _VRow('At Opacity 0 →', Row(children: [
          Expanded(child: _Num(value: lo, unit: '%', onDelta: (d) => setState(() => lo = (lo + d).clamp(0, 1000)), onType: (x) => setState(() => lo = x))),
          const SizedBox(width: 8),
          _Word('= now', onTap: () => setState(() => lo = 100 + now)),
        ])),
        _VRow('At Opacity 100 →', Row(children: [
          Expanded(child: _Num(value: hi, unit: '%', onDelta: (d) => setState(() => hi = (hi + d).clamp(0, 1000)), onType: (x) => setState(() => hi = x))),
          const SizedBox(width: 8),
          _Word('= now', onTap: () => setState(() => hi = 100 + now)),
        ])),
        _gap(8),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: bound
              ? _Band(
                  tick: _fol,
                  margin: EdgeInsets.zero,
                  child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Text('${sel.length} layers follow Opacity', style: T.name(N.g95)),
                    _gap(4),
                    Text('Scale ${_f(lo, 0)}–${_f(hi, 0)} %', style: T.label(N.g76).copyWith(height: 1.4)),
                    _gap(8),
                    _words([
                      _Word('Undo', onTap: () => setState(() => bound = false)),
                      _Word('Draw again', onTap: () => setState(() {
                            sel = {};
                            pts.clear();
                            bound = false;
                          })),
                    ]),
                  ]),
                )
              : Align(
                  alignment: Alignment.centerLeft,
                  child: _Btn('Follow', primary: true, onTap: sel.isEmpty ? null : () => setState(() => bound = true)),
                ),
        ),
      ]),
    );
  }
}

class _LassoPaint extends CustomPainter {
  const _LassoPaint(this.dots, this.sel, this.pts, this.locked, this.bound);
  final List<Offset> dots, pts;
  final Set<int> sel, locked;
  final bool bound;
  @override
  void paint(Canvas c, Size s) {
    if (pts.length > 2) {
      final p = Path()..addPolygon(pts, true);
      c.drawPath(p, Paint()..color = _selC.withValues(alpha: .08));
      c.drawPath(p, Paint()..color = _selC.withValues(alpha: .9)..style = PaintingStyle.stroke..strokeWidth = 1);
    }
    for (var i = 0; i < dots.length; i++) {
      final on = sel.contains(i);
      if (locked.contains(i)) {
        c.drawCircle(dots[i], 4, Paint()..color = N.g63..style = PaintingStyle.stroke..strokeWidth = 1);
      } else {
        c.drawCircle(dots[i], on ? 4.5 : 3.5, Paint()..color = on ? _selC : N.g63);
        if (on && bound) {
          c.drawCircle(dots[i], 7, Paint()..color = _selC.withValues(alpha: .6)..style = PaintingStyle.stroke..strokeWidth = 1);
        }
      }
    }
  }

  @override
  bool shouldRepaint(_LassoPaint o) => true;
}

// ---- I9-c: a handle that moves several values -------------------------------------------------------------------------------------------

class _Cand {
  const _Cand(this.id, this.label, this.unit, this.dec, this.lo, this.hi);
  final String id, label, unit;
  final int dec;
  final double lo, hi;
}

const _pool = <_Cand>[
  _Cand('gi', 'Glow Intensity', '', 2, .4, 2.4),
  _Cand('br', 'Blur Radius', 'px', 0, 0, 48),
  _Cand('sc', 'Scale', '%', 0, 100, 140),
  _Cand('op', 'Opacity', '%', 0, 20, 100),
  _Cand('ro', 'Rotation', '°', 1, 0, 45),
  _Cand('hs', 'Hue Shift', '°', 1, -30, 30),
  _Cand('py', 'Position Y', 'px', 0, 640, 540),
  _Cand('ed', 'Echo Decay', '', 2, .2, .8),
  _Cand('ls', 'Letter Spacing (title, large display)', 'px', 1, -2, 24),
];

class _MacroPanel extends StatefulWidget {
  const _MacroPanel({required this.h, required this.links});
  final double h;
  final int links;
  @override
  State<_MacroPanel> createState() => _MacroPanelState();
}

class _MacroPanelState extends State<_MacroPanel> {
  double t = .65;
  late final Map<String, List<double>> range = {for (final c in _pool) c.id: [c.lo, c.hi]};
  late List<String> linked = _pool.take(widget.links).map((c) => c.id).toList();

  @override
  void didUpdateWidget(_MacroPanel old) {
    super.didUpdateWidget(old);
    if (old.links != widget.links) {
      linked = _pool.take(widget.links).map((c) => c.id).toList();
    }
  }

  _Cand _by(String id) => _pool.firstWhere((c) => c.id == id);

  @override
  Widget build(BuildContext context) {
    final free = _pool.where((c) => !linked.contains(c.id)).toList();
    return _Panel(
      title: 'title_card',
      sub: 'Macro',
      h: widget.h,
      child: _Lv(children: [
        _VRow('Intro', Row(children: [
          Expanded(child: LayoutBuilder(builder: (context, b) => MiniSlider(value: t, width: b.maxWidth, onChanged: (v) => setState(() => t = v)))),
          const SizedBox(width: 8),
          SizedBox(width: 28, child: Text(_f(t, 2), maxLines: 1, textAlign: TextAlign.right, style: T.value(N.g95))),
        ])),
        _gap(8),
        _Sect('Linked', count: linked.length),
        if (linked.isEmpty) const _Hint('None', color: N.g63),
        for (final id in linked)
          _MacroRow(
            c: _by(id),
            lo: range[id]![0],
            hi: range[id]![1],
            t: t,
            onLo: (d) => setState(() => range[id]![0] += d * (_by(id).dec > 0 ? .1 : 1)),
            onHi: (d) => setState(() => range[id]![1] += d * (_by(id).dec > 0 ? .1 : 1)),
            onUnlink: () => setState(() => linked.remove(id)),
          ),
        _gap(8),
        _Sect('Not linked', count: free.length),
        for (final c in free)
          Hov(
            cursor: SystemMouseCursors.basic,
            builder: (_, h) => Container(
              constraints: const BoxConstraints(minHeight: 28),
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
              color: h ? N.g15 : null,
              child: Row(children: [
                Expanded(child: _Mid(c.label, T.label(N.g63).copyWith(fontSize: 11, height: 1.2))),
                const SizedBox(width: 8),
                _Word('Link', onTap: () => setState(() => linked.add(c.id))),
              ]),
            ),
          ),
      ]),
    );
  }
}

class _MacroRow extends StatelessWidget {
  const _MacroRow({required this.c, required this.lo, required this.hi, required this.t, required this.onLo, required this.onHi, required this.onUnlink, this.pose});
  final _Pose? pose;
  final _Cand c;
  final double lo, hi, t;
  final ValueChanged<double> onLo, onHi;
  final VoidCallback onUnlink;
  @override
  Widget build(BuildContext context) {
    final v = lo + (hi - lo) * t;
    return _Hv(
      pose: pose,
      cursor: SystemMouseCursors.basic,
      builder: (_, h, _) => _Gut(
        tick: _fol,
        child: Container(
        padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
        color: h ? N.g15 : null,
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Row(children: [
            Expanded(child: _Mid(c.label, T.label(N.g95).copyWith(fontSize: 11, height: 1.2))),
            const SizedBox(width: 8),
            Text('${_f(v, c.dec)}${c.unit.isEmpty ? '' : ' ${_unit(c.unit)}'}', maxLines: 1, style: T.value(N.g95)),
            const SizedBox(width: 12),
            _Word('Unlink', onTap: onUnlink),
          ]),
          _gap(4),
          Row(children: [
            SizedBox(width: 62, child: _Num(value: lo, dec: c.dec, onDelta: onLo, onType: (x) => onLo(x - lo))),
            const SizedBox(width: 8),
            Expanded(child: SizedBox(height: 14, child: CustomPaint(painter: _RangePaint(t)))),
            const SizedBox(width: 8),
            SizedBox(width: 62, child: _Num(value: hi, dec: c.dec, onDelta: onHi, onType: (x) => onHi(x - hi))),
          ]),
        ]),
      )),
    );
  }
}

class _RangePaint extends CustomPainter {
  const _RangePaint(this.t);
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final y = s.height / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), Paint()..color = N.g26..strokeWidth = 2..strokeCap = StrokeCap.round);
    c.drawLine(Offset(0, y), Offset(s.width * t, y), Paint()..color = _fol..strokeWidth = 2..strokeCap = StrokeCap.round);
    c.drawCircle(Offset(s.width * t, y), 4, Paint()..color = N.g95);
  }

  @override
  bool shouldRepaint(_RangePaint o) => o.t != t;
}

// ---- I9-d: the relation as one line under the row ---------------------------------------------------------------------------------------------

const _srcs = ['ring_glow · Opacity', 'kick_loop.wav · Level', 'Time (seconds)', 'Slider "Intro"', 'title_card · Position X'];
const _srcRange = {'ring_glow · Opacity': '0 – 100 %', 'kick_loop.wav · Level': '0 – 100 %', 'Time (seconds)': '0 – 10 s', 'Slider "Intro"': '0 – 1', 'title_card · Position X': '0 – 1,920 px'};

class _InlinePanel extends StatefulWidget {
  const _InlinePanel({required this.h, required this.bound, this.w = 282});
  final double h, w;
  final int bound;
  @override
  State<_InlinePanel> createState() => _InlinePanelState();
}

class _InlinePanelState extends State<_InlinePanel> {
  final vals = {for (final i in _items) i.id: i.v};
  final binds = <String, _Bind>{};
  String? open;

  @override
  void initState() {
    super.initState();
    _seed();
  }

  void _seed() {
    binds.clear();
    final mine = _items.where((i) => !i.far && i.kind == _K.num).toList();
    final ids = ['op', 'scale', 'gl.i', 'rot', 'pos.x'];
    for (var k = 0; k < widget.bound && k < ids.length; k++) {
      final it = mine.firstWhere((i) => i.id == ids[k]);
      binds[it.id] = _Bind(_srcs[k % _srcs.length], it.unit == '%' ? 20 : it.v * .5, it.unit == '%' ? 100 : (it.v == 0 ? 45 : it.v * 1.5));
    }
    open = binds.keys.isEmpty ? 'op' : binds.keys.first;
  }

  @override
  void didUpdateWidget(_InlinePanel old) {
    super.didUpdateWidget(old);
    if (old.bound != widget.bound) {
      _seed();
    }
  }

  Widget _row(_It it) {
    final b = binds[it.id], isOpen = open == it.id;
    final label = _Mid(it.label, T.label(b != null ? N.g95 : N.g63).copyWith(fontSize: 11, height: 1.2));
    Widget ctl;
    if (it.kind == _K.num) {
      ctl = _Num(
        value: vals[it.id]!,
        dec: it.dec,
        unit: it.unit,
        enabled: b == null,
        onDelta: (d) => setState(() => vals[it.id] = vals[it.id]! + d * (it.dec > 0 ? .1 : 1)),
        onType: (x) => setState(() => vals[it.id] = x),
      );
    } else if (it.kind == _K.color) {
      ctl = _ColorCell(index: vals[it.id]!.round(), onTap: () => setState(() => vals[it.id] = (vals[it.id]! + 1) % _swatch.length));
    } else {
      ctl = _Pick(text: it.opts[vals[it.id]!.round()], onTap: () => setState(() => vals[it.id] = (vals[it.id]! + 1) % it.opts.length));
    }
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
      Hov(
        onTap: it.kind == _K.num ? () => setState(() => open = isOpen ? null : it.id) : null,
        cursor: it.kind == _K.num ? SystemMouseCursors.click : SystemMouseCursors.basic,
        builder: (_, h) => LayoutBuilder(builder: (context, box) {
          final narrow = box.maxWidth < 220;
          return _Gut(
            tick: b != null ? _fol : null,
            child: Container(
            constraints: const BoxConstraints(minHeight: 28),
            padding: const EdgeInsets.fromLTRB(12, 4, 12, 4),
            color: isOpen ? N.g20 : (h ? N.g15 : null),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
              narrow
                  ? Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [label, _gap(3), ctl])
                  : Row(children: [Expanded(flex: 5, child: label), const SizedBox(width: 8), Expanded(flex: 6, child: ctl)]),
              if (b != null) Padding(padding: const EdgeInsets.only(top: 4), child: Text('← ${b.src}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
            ]),
          ));
        }),
      ),
      if (isOpen) _formula(it, b),
    ]);
  }

  Widget _formula(_It it, _Bind? b) => _Gut(
        tick: _fol,
        child: Container(
        padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
        color: N.g13,
        child: b == null
            ? Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text('Follows nothing', style: T.name(N.g91)),
                _gap(8),
                _words([
                  for (final s in _srcs) _Word(s, onTap: () => setState(() => binds[it.id] = _Bind(s, it.unit == '%' ? 0 : it.v * .5, it.unit == '%' ? 100 : (it.v == 0 ? 45 : it.v * 1.5)))),
                ]),
              ])
            : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                Row(children: [
                  Expanded(child: Text('Follows ${b.src}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g95))),
                  _Word('Unlink', onTap: () => setState(() => binds.remove(it.id))),
                ]),
                _gap(8),
                Wrap(spacing: 8, runSpacing: 8, crossAxisAlignment: WrapCrossAlignment.center, children: [
                  Text(_srcRange[b.src] ?? '0 – 1', style: T.value(N.g76)),
                  Text('→', style: T.label(N.g76).copyWith(fontSize: 11)),
                  SizedBox(width: 62, child: _Num(value: b.lo, dec: it.dec > 0 ? 1 : 0, onDelta: (d) => setState(() => b.lo += d), onType: (x) => setState(() => b.lo = x))),
                  Text('to', style: T.label(N.g76)),
                  SizedBox(width: 62, child: _Num(value: b.hi, dec: it.dec > 0 ? 1 : 0, onDelta: (d) => setState(() => b.hi += d), onType: (x) => setState(() => b.hi = x))),
                  if (it.unit.isNotEmpty) Text(_unit(it.unit), style: T.label(N.g76)),
                ]),
              ])),
      );

  @override
  Widget build(BuildContext context) {
    final rows = <Widget>[];
    String? g;
    for (final it in _items.where((i) => !i.far)) {
      if (it.group != g) {
        g = it.group;
        rows.add(_Sect(g));
      }
      rows.add(_row(it));
    }
    return _Panel(
      title: 'star_burst',
      sub: 'Shape layer · ${binds.length} follow${binds.length == 1 ? '' : 's'}',
      w: widget.w,
      h: widget.h,
      child: _Lv(children: rows),
    );
  }
}

List<WidgetbookUseCase> _i9Cases() => [
      _story('I9-a Pick-whip grab and tie', 'I9-a', 'Pull a line from a value to the value it should follow', _Habit.habit,
          'Contract: while grabbing, only values of the same kind (px to px, % to %) stay lit, the rest dim and do not move; the value you release on is the source; the row then reads "← source" and carries the word Unlink. Try it: grab the Opacity ring.',
          (c, h) => _WhipPanel(h: h, mode: c.knobs.object.dropdown<String>(label: 'State', options: const ['live', 'bound'], initialOption: 'live', labelBuilder: (s) => s == 'live' ? 'Empty, drag it yourself' : 'Three already follow'))),
      _story('I9-a Pick-whip mid-grab', 'I9-a', 'The moment of the drag, held still: the compatible values are lit', _Habit.habit,
          'Contract: the grabbed ring is orange, same-kind values carry an orange tick, the one under the pointer is tinted, the rest are at 30 %. The partner can be far away (a value on another layer).',
          (c, h) => _WhipPanel(h: h, mode: 'mid-grab')),
      _story('I9-b Lasso one to many', 'I9-b', 'Draw around layers on a small map, set the range, follow all', _Habit.addition,
          'Contract: one lasso binds every selected layer in one step; the range comes from scrub or "= now"; locked layers are left out and the panel says how many; one undo. Knobs: number of layers, locked layers.',
          (c, h) => _LassoPanel(h: h, count: c.knobs.int.slider(label: 'Layers on the map', initialValue: 24, min: 6, max: 90), locks: c.knobs.boolean(label: 'Some layers locked', initialValue: true), drawn: c.knobs.boolean(label: 'Loop already drawn', initialValue: true), followed: c.knobs.boolean(label: 'Already followed', initialValue: false))),
      _story('I9-b Lasso in a dock', 'I9-b', 'The same Relations tab at 224 px', _Habit.addition,
          'Contract: the map shrinks with the panel; the range rows stack label over value (X7); the Follow button keeps its count.',
          (c, h) => _LassoPanel(h: h, w: 224, count: 18, locks: true, drawn: true, followed: false)),
      _story('I9-c Macro handle', 'I9-c', 'Make a handle once; tie values to it, each with its own range', _Habit.addition,
          'Contract: moving the handle moves every linked value inside its own range; each row shows its range and the live value; the handle row is the owner of the values. Knob 0 shows the empty state.',
          (c, h) => _MacroPanel(h: h, links: c.knobs.int.slider(label: 'Linked at start', initialValue: 4, min: 0, max: 9))),
      _story('I9-d One-line relation under the row', 'I9-d', 'Click a row: its relation opens as one line you can scrub', _Habit.addition,
          'Contract: one row opens at a time; the range ends scrub in place; Unlink is the word on the right. Collapsed, a followed row shows a left tick and "← source". Knob: how many rows follow at start.',
          (c, h) => _InlinePanel(h: h, bound: c.knobs.int.slider(label: 'Following at start', initialValue: 3, min: 0, max: 5))),
      _story('I9-d One-line relation in a dock', 'I9-d', 'The same rows at 200 px', _Habit.addition,
          'Contract: the formula wraps instead of shrinking; the number cells keep their width; nothing is cut.',
          (c, h) => _InlinePanel(h: h, w: 200, bound: 3)),
    ];
