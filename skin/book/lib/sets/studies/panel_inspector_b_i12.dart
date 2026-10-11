part of 'panel_inspector_b.dart';

// ---- I12: many layers at once ------------------------------------------------------------------------------------------------------------

const _layerNames = ['title_card', 'bg_gradient', 'star_burst', '見出し_タイトル', 'particles_main', 'logo_end_card_final_v2', 'flare_01', 'kick_visual'];
String _ln(int i) => i < _layerNames.length ? _layerNames[i] : 'card_${(i + 1).toString().padLeft(2, '0')}';

class _MProp {
  const _MProp(this.id, this.label, this.unit, this.dec, this.lo, this.hi);
  final String id, label, unit;
  final int dec;
  final double lo, hi;
}

const _mprops = [
  _MProp('px', 'Position X', 'px', 0, 0, 1920),
  _MProp('py', 'Position Y', 'px', 0, 0, 1080),
  _MProp('sc', 'Scale', '%', 0, 20, 220),
  _MProp('ro', 'Rotation', '°', 1, -45, 45),
  _MProp('op', 'Opacity', '%', 0, 100, 100),
];

String _thousands(double v, int dec) {
  final s = _f(v, dec), neg = s.startsWith('-'), body = neg ? s.substring(1) : s, parts = body.split('.');
  final buf = StringBuffer();
  for (var i = 0; i < parts[0].length; i++) {
    if (i > 0 && (parts[0].length - i) % 3 == 0) {
      buf.write(',');
    }
    buf.write(parts[0][i]);
  }
  return '${neg ? '-' : ''}$buf${parts.length > 1 ? '.${parts[1]}' : ''}';
}

// ---- I12-a: drag is relative, typing is absolute --------------------------------------------------------------------------------------------

class _MultiPanel extends StatefulWidget {
  const _MultiPanel({required this.h, required this.n, required this.locked, this.w = 282});
  final double h, w;
  final int n, locked;
  @override
  State<_MultiPanel> createState() => _MultiPanelState();
}

class _MultiPanelState extends State<_MultiPanel> {
  late List<Map<String, double>> v = _make();
  int blend = -1; // -1 mixed

  List<Map<String, double>> _make() {
    final r = math.Random(3);
    return [
      for (var i = 0; i < widget.n; i++)
        {for (final p in _mprops) p.id: p.id == 'op' ? (i % 9 == 4 ? 60 : 100) : (p.lo + (p.hi - p.lo) * (.1 + .8 * r.nextDouble())).roundToDouble()},
    ];
  }

  @override
  void didUpdateWidget(_MultiPanel old) {
    super.didUpdateWidget(old);
    if (old.n != widget.n || old.locked != widget.locked) {
      v = _make();
    }
  }

  bool _lk(int i) => i < widget.locked;
  Iterable<int> get _live => [for (var i = 0; i < widget.n; i++) if (!_lk(i)) i];

  ({bool mixed, double lo, double hi}) _stat(_MProp p) {
    final xs = [for (final i in _live) v[i][p.id]!];
    if (xs.isEmpty) {
      return (mixed: false, lo: 0, hi: 0);
    }
    final lo = xs.reduce(math.min), hi = xs.reduce(math.max);
    return (mixed: (hi - lo).abs() > 1e-6, lo: lo, hi: hi);
  }

  @override
  Widget build(BuildContext context) {
    final n = widget.n, live = n - widget.locked;
    return _Panel(
      title: n == 1 ? _ln(0) : '$n layers',
      sub: n == 1 ? 'Shape layer' : (widget.locked > 0 ? '$live of $n editable' : 'Selected together'),
      w: widget.w,
      h: widget.h,
      child: _Lv(children: [
        if (n > 1)
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              Wrap(spacing: 4, runSpacing: 4, children: [
                for (var i = 0; i < math.min(n, 4); i++) Pill(_lk(i) ? '${_ln(i)} · locked' : _ln(i), fill: N.g13),
                if (n > 4) Pill('+${n - 4} more', fill: N.g13),
              ]),
            ]),
          ),
        const _Sect('Transform'),
        for (final p in _mprops) _row(p),
        _VRow('Blending Mode', _Pick(text: blend < 0 ? 'Mixed' : _bl[blend], onTap: () => setState(() => blend = blend < 0 ? 0 : (blend + 1) % _bl.length))),
      ]),
    );
  }

  Widget _row(_MProp p) {
    final s = _stat(p), n = widget.n;
    final sub = s.mixed ? '${_thousands(s.lo, p.dec)} to ${_thousands(s.hi, p.dec)}' : null;
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
      _VRow(
        p.label,
        _Num(
          value: s.lo,
          dec: p.dec,
          unit: p.unit,
          mixed: s.mixed,
          enabled: _live.isNotEmpty,
          onDelta: (d) => setState(() {
            final dd = d * (p.dec > 0 ? .1 : 1);
            for (final i in _live) {
              v[i][p.id] = v[i][p.id]! + dd;
            }
          }),
          onType: (x) => setState(() {
            for (final i in _live) {
              v[i][p.id] = x;
            }
          }),
        ),
        tick: s.mixed && n > 1 ? N.g44 : null,
      ),
      if (sub != null) Padding(padding: const EdgeInsets.fromLTRB(12, 0, 12, 4), child: Text(sub, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(fontFamily: T.mono))),
    ]);
  }
}

// ---- I12-b: stagger beside the value ---------------------------------------------------------------------------------------------------------

class _StaggerPanel extends StatefulWidget {
  const _StaggerPanel({required this.h, required this.n, required this.basis});
  final double h;
  final int n, basis;
  @override
  State<_StaggerPanel> createState() => _StaggerPanelState();
}

class _StaggerPanelState extends State<_StaggerPanel> {
  double base = 120, step = 140, tBase = 0, tStep = 2;
  late int basis = widget.basis;

  @override
  void didUpdateWidget(_StaggerPanel old) {
    super.didUpdateWidget(old);
    if (old.basis != widget.basis) {
      basis = widget.basis;
    }
  }

  List<String> _order() {
    final names = [for (var i = 0; i < widget.n; i++) _ln(i)];
    return switch (basis) {
      0 => ([...names]..sort((a, b) => (a.hashCode % 97).compareTo(b.hashCode % 97))),
      1 => names,
      _ => names.reversed.toList(),
    };
  }

  @override
  Widget build(BuildContext context) {
    final n = widget.n;
    if (n < 2) {
      return _Panel(
        title: n == 1 ? _ln(0) : 'Nothing selected',
        sub: 'Stagger',
        h: widget.h,
        child: const _NoMatch(what: 'Stagger needs two or more layers', next: 'Select all layers', onNext: _noop),
      );
    }
    final order = _order();
    final xs = [for (var i = 0; i < n; i++) base + step * i];
    final off = xs.where((x) => x > 1920 || x < 0).length;
    return _Panel(
      title: '$n layers',
      sub: 'Stagger',
      h: widget.h,
      child: _Lv(children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Text('Order', style: T.label(N.g76)),
            _gap(4),
            Segmented(items: const ['Stage', 'Timeline', 'Picked'], index: basis, expand: true, onChanged: (i) => setState(() => basis = i)),
          ]),
        ),
        const _Sect('Position X'),
        _VRow('First layer', _Num(value: base, unit: 'px', onDelta: (d) => setState(() => base += d), onType: (x) => setState(() => base = x))),
        _VRow('Step per layer', _Num(value: step, unit: 'px', onDelta: (d) => setState(() => step = (step + d).clamp(-200, 200).roundToDouble()), onType: (x) => setState(() => step = x.clamp(-200, 200).roundToDouble()))),
        Padding(padding: const EdgeInsets.fromLTRB(12, 4, 12, 0), child: SizedBox(height: 56, child: CustomPaint(painter: _BarsPaint(xs, 0, 1920), size: Size.infinite))),
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 4, 12, 0),
          child: Row(children: [
            Expanded(child: Text('${order.first}  ${_thousands(xs.first, 0)}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(fontFamily: T.mono))),
            Text(' … ', style: T.label(N.g76)),
            Expanded(child: Text('${_thousands(xs.last, 0)}  ${order.last}', maxLines: 1, textAlign: TextAlign.right, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(fontFamily: T.mono))),
          ]),
        ),
        if (off > 0) Padding(padding: const EdgeInsets.fromLTRB(12, 6, 12, 0), child: Text('$off off Stage', style: _cnt(N.g91))),
        _gap(8),
        const _Sect('Start time'),
        _VRow('First layer', _Num(value: tBase, unit: 'f', onDelta: (d) => setState(() => tBase += d), onType: (x) => setState(() => tBase = x))),
        _VRow('Step per layer', _Num(value: tStep, unit: 'f', onDelta: (d) => setState(() => tStep = (tStep + d).clamp(-12, 12).roundToDouble()), onType: (x) => setState(() => tStep = x.clamp(-12, 12).roundToDouble()))),
        _Hint('${order.last}  f${_f(tBase + tStep * (n - 1), 0)}'),
      ]),
    );
  }
}

void _noop() {}

class _BarsPaint extends CustomPainter {
  const _BarsPaint(this.v, this.lo, this.hi);
  final List<double> v;
  final double lo, hi;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(3)), Paint()..color = N.g07);
    final n = v.length, bw = (s.width - 8) / n;
    for (var i = 0; i < n; i++) {
      final t = ((v[i] - lo) / (hi - lo)).clamp(0.0, 1.0), h = 2 + (s.height - 10) * t;
      final out = v[i] > hi || v[i] < lo;
      c.drawRect(Rect.fromLTWH(4 + i * bw + math.min(1, bw * .15), s.height - 4 - h, math.max(1, bw - math.min(2, bw * .3)), h), Paint()..color = out ? N.g63 : Fam.stagger.c.withValues(alpha: .85));
    }
  }

  @override
  bool shouldRepaint(_BarsPaint o) => true;
}

// ---- I12-c: draw the distribution (Falloff) -------------------------------------------------------------------------------------------------------

class _FalloffPanel extends StatefulWidget {
  const _FalloffPanel({required this.h, required this.n});
  final double h;
  final int n;
  @override
  State<_FalloffPanel> createState() => _FalloffPanelState();
}

class _FalloffPanelState extends State<_FalloffPanel> {
  late List<double> y = _seed();
  int shape = 0, last = -1;
  Offset? a, b;
  int? drag;
  static const ch = 200.0;

  List<double> _seed() {
    final r = math.Random(5);
    return [for (var i = 0; i < widget.n; i++) .25 + .5 * r.nextDouble()];
  }

  @override
  void didUpdateWidget(_FalloffPanel old) {
    super.didUpdateWidget(old);
    if (old.n != widget.n) {
      y = _seed();
      last = -1;
    }
  }

  double _x(int i, double w) => widget.n == 1 ? w / 2 : 12 + (w - 24) * i / (widget.n - 1);
  double _vy(double py) => (1 - (py - 8) / (ch - 16)).clamp(0.0, 1.0);

  double _shape(double t) => switch (shape) { 0 => t, 1 => t * t * (3 - 2 * t), _ => t >= .5 ? 1.0 : 0.0 };

  void _apply(double w) {
    final p = a!, q = b!;
    for (var i = 0; i < widget.n; i++) {
      final dx = q.dx - p.dx, t = dx.abs() < 1 ? 1.0 : ((_x(i, w) - p.dx) / dx).clamp(0.0, 1.0);
      y[i] = _vy(p.dy) + (_vy(q.dy) - _vy(p.dy)) * _shape(t);
    }
  }

  @override
  Widget build(BuildContext context) {
    final w = 282.0 - 24, n = widget.n;
    if (n < 2) {
      return _Panel(title: 'Falloff', sub: 'Distribution of a value', h: widget.h, child: const _NoMatch(what: 'Select two or more layers to shape a distribution', next: 'Select all layers', onNext: _noop));
    }
    return _Panel(
      title: '$n layers',
      sub: 'Falloff · Position Y',
      h: widget.h,
      child: _Lv(children: [
        const _Sect('Shape'),
        Padding(padding: const EdgeInsets.symmetric(horizontal: 12), child: Segmented(items: const ['Linear', 'Ease', 'Step'], index: shape, expand: true, onChanged: (i) => setState(() => shape = i))),
        _gap(8),
        const _Sect('Distribution'),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: GestureDetector(
            onPanStart: (d) {
              final p = d.localPosition;
              int? hit;
              for (var i = 0; i < n; i++) {
                if ((Offset(_x(i, w), 8 + (ch - 16) * (1 - y[i])) - p).distance < 11) {
                  hit = i;
                }
              }
              setState(() {
                drag = hit;
                if (hit == null) {
                  a = b = p;
                } else {
                  last = hit;
                }
              });
            },
            onPanUpdate: (d) => setState(() {
              if (drag != null) {
                y[drag!] = _vy(d.localPosition.dy);
              } else {
                b = d.localPosition;
                _apply(w);
              }
            }),
            onPanEnd: (_) => setState(() {
              drag = null;
              a = b = null;
            }),
            child: Container(
              width: w,
              height: ch,
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
              child: CustomPaint(painter: _FallPaint(y, a, b, last, [for (var i = 0; i < n; i++) _x(i, w)])),
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 4, 12, 0),
          child: Row(children: [
            Flexible(child: Text(_ln(0), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
            const Spacer(),
            Flexible(child: Text(_ln(n - 1), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
          ]),
        ),
        _gap(8),
        _VRow('Last touched', Text(last < 0 ? 'none yet' : '${_ln(last)}  ${_thousands(1080 * y[last], 0)} px', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.value(N.g91).copyWith(fontSize: 11))),
        Padding(padding: const EdgeInsets.fromLTRB(12, 4, 12, 0), child: Align(alignment: Alignment.centerLeft, child: QuietButton('Flatten to the average', onTap: () => setState(() {
              final m = y.reduce((x, z) => x + z) / n;
              y = List.filled(n, m);
            })))),
      ]),
    );
  }
}

class _FallPaint extends CustomPainter {
  const _FallPaint(this.y, this.a, this.b, this.last, this.xs);
  final List<double> y, xs;
  final Offset? a, b;
  final int last;
  @override
  void paint(Canvas c, Size s) {
    final hair = Paint()..color = N.g15..strokeWidth = 1;
    for (var i = 1; i < 4; i++) {
      c.drawLine(Offset(0, s.height * i / 4), Offset(s.width, s.height * i / 4), hair);
    }
    final path = Path();
    for (var i = 0; i < y.length; i++) {
      final p = Offset(xs[i], 8 + (s.height - 16) * (1 - y[i]));
      i == 0 ? path.moveTo(p.dx, p.dy) : path.lineTo(p.dx, p.dy);
    }
    c.drawPath(path, Paint()..color = Fam.stagger.c.withValues(alpha: .35)..style = PaintingStyle.stroke..strokeWidth = 1);
    if (a != null && b != null) {
      c.drawLine(a!, b!, Paint()..color = Role.selected..strokeWidth = 1.5..strokeCap = StrokeCap.round);
    }
    for (var i = 0; i < y.length; i++) {
      final p = Offset(xs[i], 8 + (s.height - 16) * (1 - y[i]));
      c.drawCircle(p, i == last ? 5 : 3.5, Paint()..color = i == last ? N.g100 : Fam.stagger.c);
    }
  }

  @override
  bool shouldRepaint(_FallPaint o) => true;
}

// ---- I12-d: gather the same property into one row ------------------------------------------------------------------------------------------------

class _GrabPanel extends StatefulWidget {
  const _GrabPanel({required this.h, required this.n});
  final double h;
  final int n;
  @override
  State<_GrabPanel> createState() => _GrabPanelState();
}

class _GrabPanelState extends State<_GrabPanel> {
  int prop = 0;
  bool open = false;
  late List<List<double>> v = _make();

  List<List<double>> _make() {
    final r = math.Random(11);
    return [for (final p in _mprops.take(3)) [for (var i = 0; i < widget.n; i++) (p.lo + (p.hi - p.lo) * r.nextDouble()).roundToDouble()]];
  }

  @override
  void didUpdateWidget(_GrabPanel old) {
    super.didUpdateWidget(old);
    if (old.n != widget.n) {
      v = _make();
    }
  }

  bool _lk(int i) => widget.n > 3 && i == widget.n - 1;

  @override
  Widget build(BuildContext context) {
    final n = widget.n, p = _mprops[prop], xs = v[prop];
    final live = [for (var i = 0; i < n; i++) if (!_lk(i)) i];
    final lo = live.map((i) => xs[i]).reduce(math.min), hi = live.map((i) => xs[i]).reduce(math.max);
    final mixed = hi - lo > 1e-6;
    return _Panel(
      title: '$n layers',
      sub: 'Same property on all',
      h: widget.h,
      child: _Lv(children: [
        Padding(padding: const EdgeInsets.fromLTRB(12, 8, 12, 8), child: Segmented(items: [for (final q in _mprops.take(3)) q.label], index: prop, expand: true, onChanged: (i) => setState(() => prop = i))),
        _VRow(
          p.label,
          _Num(
            value: lo,
            dec: p.dec,
            unit: p.unit,
            mixed: mixed,
            onDelta: (d) => setState(() {
              for (final i in live) {
                xs[i] += d;
              }
            }),
            onType: (x) => setState(() {
              for (final i in live) {
                xs[i] = x;
              }
            }),
          ),
          tick: mixed ? N.g44 : null,
        ),
        if (mixed) Padding(padding: const EdgeInsets.fromLTRB(12, 0, 12, 4), child: Text('${_thousands(lo, 0)} to ${_thousands(hi, 0)}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(fontFamily: T.mono))),
        _Sect('Each layer', open: open, onTap: () => setState(() => open = !open), trailing: open ? _Word('Set all to first', onTap: () => setState(() {
              for (final i in live) {
                xs[i] = xs.first;
              }
            })) : null),
        if (open)
          for (var i = 0; i < n; i++)
            _VRow(
              _ln(i),
              _Num(value: xs[i], unit: p.unit, enabled: !_lk(i), word: _lk(i) ? 'Locked' : null, onDelta: (d) => setState(() => xs[i] += d), onType: (x) => setState(() => xs[i] = x)),
              tick: _lk(i) ? null : N.g44,
              labelColor: N.g63,
            ),
      ]),
    );
  }
}

List<WidgetbookUseCase> _i12Cases() => [
      _story('I12-a Mixed values', 'I12-a', 'Select several: differing values read Mixed; drag moves each by the same amount', _Habit.habit,
          'Contract (D-I1): Mixed is a word with a dash and the range under it; scrub is relative, typing sets all; locked layers are not touched and the header says how many are editable. Knobs: how many layers, how many locked.',
          (c, h) => _MultiPanel(h: h, n: c.knobs.int.slider(label: 'Layers selected', initialValue: 4, min: 1, max: 40), locked: c.knobs.int.slider(label: 'Locked among them', initialValue: 1, min: 0, max: 3))),
      _story('I12-a Mixed values 30 layers', 'I12-a', 'The same rows with thirty layers selected', _Habit.habit,
          'Contract: the header shows four names and a count, never thirty chips; the rule is the same at any size; a drag is still one undo step.',
          (c, h) => _MultiPanel(h: h, n: 30, locked: 3)),
      _story('I12-a Mixed values in a dock', 'I12-a', 'Mixed at 200 px', _Habit.habit,
          'Contract: "Mixed" and its range stay as words; label stacks over the cell below 220 px (X7).',
          (c, h) => _MultiPanel(h: h, w: 200, n: 4, locked: 1)),
      _story('I12-b Stagger beside the value', 'I12-b', 'A step per layer, as a slider next to the first value', _Habit.addition,
          'Contract: step = the difference between neighbours; the order basis is chosen in words (Stage / Timeline / Picked); the bars preview all layers; layers pushed off the Stage are named. Knobs: layers, basis. 1 = the hidden state.',
          (c, h) => _StaggerPanel(h: h, n: c.knobs.int.slider(label: 'Layers selected', initialValue: 8, min: 1, max: 40), basis: c.knobs.object.dropdown<int>(label: 'Order follows', options: const [0, 1, 2], initialOption: 0, labelBuilder: (i) => const ['Stage', 'Timeline', 'Picked'][i]))),
      _story('I12-b Stagger 30 layers', 'I12-b', 'Thirty layers: the preview compresses, the numbers stay', _Habit.addition,
          'Contract: the preview reads at any count; the first and last layer are named with their values; a large step shows the off-Stage count.',
          (c, h) => _StaggerPanel(h: h, n: 30, basis: 1)),
      _story('I12-c Falloff', 'I12-c', 'Points of the selection on a small chart; draw a line to set them all', _Habit.addition,
          'Contract: touching a dot moves that layer; a line drawn in empty space writes every layer (linear / smooth / stepped); one undo step per release. Knob: layers (1 shows the empty state).',
          (c, h) => _FalloffPanel(h: h, n: c.knobs.int.slider(label: 'Layers selected', initialValue: 12, min: 1, max: 40))),
      _story('I12-d Grab one property from all', 'I12-d', 'One row stands for the same property on every selected layer', _Habit.addition,
          'Contract: editing the row edits all; each layer is one click away ("Each layer"); a locked layer is skipped and says Locked. Knob: layers.',
          (c, h) => _GrabPanel(h: h, n: c.knobs.int.slider(label: 'Layers selected', initialValue: 6, min: 2, max: 40))),
    ];
