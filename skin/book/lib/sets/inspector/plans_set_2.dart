// Part of plans_set.dart (cut there by size, as it was).
part of 'plans_set.dart';

class _PlanC extends StatefulWidget {
  const _PlanC(this.s);
  final _S s;
  @override
  State<_PlanC> createState() => _PlanCState();
}

class _PlanCState extends State<_PlanC> {
  int? open;
  @override
  Widget build(BuildContext context) {
    final s = widget.s;
    return ListenableBuilder(
      listenable: s,
      builder: (context, _) {
        final shown = [for (final k in _ranges.keys) if (s.touched.contains(k)) k];
        return Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Expanded(child: _Stage(s, 'C  触った値だけ　行 ${shown.length}')),
          _inspector([
            _head(),
            for (final k in shown)
              Padding(
                padding: const EdgeInsets.only(bottom: 5),
                child: GestureDetector(
                  onDoubleTap: () => s.forget(k),
                  child: k == 'layout'
                      ? Row(children: [
                          SizedBox(width: 60, child: Text(_jp[k]!, style: T.label(N.g76))),
                          Expanded(child: Segmented(items: _layouts, index: s['layout'].round(), expand: true, onChanged: (i) => s.set('layout', i.toDouble()))),
                        ])
                      : ValueWell(
                          label: _jp[k]!,
                          value: s[k],
                          min: _ranges[k]!.$1,
                          max: _ranges[k]!.$2,
                          decimals: _ranges[k]!.$2 <= 4 ? 2 : 0,
                          fam: Fam.all[_verbKeys.indexWhere((g) => g.contains(k))],
                          onChanged: (x) => s.set(k, x),
                        ),
                ),
              ),
            const SizedBox(height: 10),
            Row(children: [
              Text('＋', style: T.title(N.g76)),
              const SizedBox(width: 8),
              for (final (i, w) in _verbs.indexed)
                Padding(
                  padding: const EdgeInsets.only(right: 5),
                  child: GestureDetector(
                    onTap: () => setState(() => open = open == i ? null : i),
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 5),
                      decoration: BoxDecoration(color: open == i ? N.g26 : N.g15, borderRadius: BorderRadius.circular(4), border: Border(bottom: BorderSide(color: Fam.all[i].c))),
                      child: Text(w, style: T.name(open == i ? N.g95 : N.g76)),
                    ),
                  ),
                ),
            ]),
            if (open != null) ...[
              const SizedBox(height: 8),
              Wrap(spacing: 5, runSpacing: 5, children: [
                for (final k in _verbKeys[open!])
                  if (!s.touched.contains(k))
                    GestureDetector(
                      onTap: () => s.set(k, s[k]),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g26)),
                        child: Text(_jp[k]!, style: T.label(N.g76)),
                      ),
                    ),
              ]),
            ],
            const Spacer(),
            Text('全部 $_rowsA', style: T.label(N.g56)),
          ]),
        ]);
      },
    );
  }
}

// ---------------------------------------------------------------- D: macros

class _PlanD extends StatefulWidget {
  const _PlanD(this.s);
  final _S s;
  @override
  State<_PlanD> createState() => _PlanDState();
}

class _PlanDState extends State<_PlanD> {
  int? open;
  @override
  Widget build(BuildContext context) {
    final s = widget.s;
    return ListenableBuilder(
      listenable: s,
      builder: (context, _) => Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Expanded(child: _Stage(s, 'D  マクロ4つ　行 4')),
        _inspector([
          _head(),
          GridView.count(
            crossAxisCount: 2,
            shrinkWrap: true,
            mainAxisSpacing: 10,
            childAspectRatio: 1.05,
            physics: const NeverScrollableScrollPhysics(),
            children: [
              for (final (i, (name, _)) in _macros.indexed)
                GestureDetector(
                  onVerticalDragUpdate: (d) => s.setMacro(i, s.macro[i] - d.delta.dy / 160),
                  onTap: () => setState(() => open = open == i ? null : i),
                  child: Column(mainAxisSize: MainAxisSize.min, children: [
                    SizedBox.square(
                      dimension: 84,
                      child: CustomPaint(
                        painter: _DialPaint(s.macro[i], color: Fam.all[i].c),
                        child: Center(child: Text('${(s.macro[i] * 100).round()}', style: T.value(N.g95).copyWith(fontSize: 14))),
                      ),
                    ),
                    const SizedBox(height: 6),
                    Row(mainAxisAlignment: MainAxisAlignment.center, children: [
                      _Badge(i),
                      const SizedBox(width: 5),
                      Text(name, style: T.name(open == i ? N.g95 : N.g76)),
                    ]),
                  ]),
                ),
            ],
          ),
          if (open != null) ...[
            const SizedBox(height: 10),
            Container(height: 1, color: N.g20, margin: const EdgeInsets.only(bottom: 10)),
            for (final (k, lo, hi) in _macros[open!].$2)
              Padding(
                padding: const EdgeInsets.only(bottom: 9),
                child: Row(children: [
                  _Badge(open!),
                  const SizedBox(width: 6),
                  SizedBox(width: 52, child: Text(_jp[k]!, style: T.label(N.g76))),
                  Expanded(child: _Span(lo, hi, s[k], _ranges[k]!, Fam.all[open!].c)),
                  SizedBox(width: 46, child: Text(_say(k, s[k]), textAlign: TextAlign.right, style: T.value(N.g91).copyWith(fontSize: 10))),
                ]),
              ),
          ],
        ]),
      ]),
    );
  }
}

class _Badge extends StatelessWidget {
  const _Badge(this.i);
  final int i;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
    decoration: BoxDecoration(border: Border.all(color: Fam.all[i].c), borderRadius: BorderRadius.circular(3)),
    child: Text('M${i + 1}', style: T.micro(Fam.all[i].c).copyWith(fontSize: 9)),
  );
}

/// A macro's reach in one value: the low-to-high span it moves over, and where the value is now.
class _Span extends StatelessWidget {
  const _Span(this.lo, this.hi, this.now, this.range, this.c);
  final double lo, hi, now;
  final (double, double) range;
  final Color c;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
    double x(double v) => box.maxWidth * ((v - range.$1) / (range.$2 - range.$1)).clamp(0.0, 1.0);
    final a = math.min(x(lo), x(hi)), b = math.max(x(lo), x(hi));
    return SizedBox(
      height: 12,
      child: Stack(children: [
        Positioned(left: 0, right: 0, top: 5, height: 2, child: ColoredBox(color: N.g20)),
        Positioned(left: a, width: math.max(2, b - a), top: 4, height: 4, child: ColoredBox(color: c.withValues(alpha: .5))),
        Positioned(left: x(now) - 1, width: 2, top: 0, bottom: 0, child: ColoredBox(color: N.g95)),
      ]),
    );
  });
}

// ---------------------------------------------------------------- the set

/// One ring shared by every plan on the page, so a knob turned in one moves them all.
class _Plans extends StatefulWidget {
  const _Plans(this.which);
  final int which; // 0 A .. 3 D, -1 all four side by side
  @override
  State<_Plans> createState() => _PlansState();
}

class _PlansState extends State<_Plans> {
  final s = _S();
  @override
  void dispose() {
    s.dispose();
    super.dispose();
  }

  Widget plan(int i) => switch (i) {
    0 => _PlanA(s),
    1 => _PlanB(s),
    2 => _PlanC(s),
    _ => _PlanD(s),
  };

  Widget framed(int i) => Padding(
    padding: const EdgeInsets.all(6),
    child: FittedBox(
      child: Container(
        width: 980,
        height: 640,
        decoration: BoxDecoration(border: Border.all(color: N.g20)),
        child: plan(i),
      ),
    ),
  );

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: N.g00,
    child: widget.which >= 0
        ? plan(widget.which)
        : Column(children: [
            Expanded(child: Row(children: [Expanded(child: framed(0)), Expanded(child: framed(1))])),
            Expanded(child: Row(children: [Expanded(child: framed(2)), Expanded(child: framed(3))])),
          ]),
  );
}

WidgetbookComponent inspectorPlansSet() => WidgetbookComponent(
  name: '星の輪',
  useCases: [
    WidgetbookUseCase(name: '並べて', builder: (c) => const _Plans(-1)),
    WidgetbookUseCase(name: 'A 今のまま', builder: (c) => const _Plans(0)),
    WidgetbookUseCase(name: 'B 動詞', builder: (c) => const _Plans(1)),
    WidgetbookUseCase(name: 'C 触った値', builder: (c) => const _Plans(2)),
    WidgetbookUseCase(name: 'D マクロ', builder: (c) => const _Plans(3)),
  ],
);
