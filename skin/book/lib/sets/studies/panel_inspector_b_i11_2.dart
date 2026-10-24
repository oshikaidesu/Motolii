// Part of panel_inspector_b_i11.dart (cut there by size, as it was).
part of 'panel_inspector_b.dart';

class _ShelfPanelState extends State<_ShelfPanel> {
  int tab = 0;
  late List<List<_ShelfItem>> items = _make();
  String status = '';
  int? hot;

  static const _names = [
    ['Snappy out', 'Soft in-out', 'Overshoot small', 'Bounce ×3', 'ゆっくり戻る', 'Anticipate', 'Slow start, fast end', 'Elastic tail', 'Back out strong', 'Linear (hold)'],
    ['Sunset trio', 'Neon edge', '落ち着いた青', 'Ink on paper', 'Warm glow', 'Mint pop'],
    ['Soft bloom stack', 'Film grain + vignette', 'Glitch light', 'ぼかし+色収差', 'Clean sharpen'],
    ['Intro offset', 'Title tracking', 'Shadow soft', 'Bounce scale', 'Fade 12f'],
  ];

  List<List<_ShelfItem>> _make() => [
        for (var t = 0; t < 4; t++)
          [
            for (var i = 0; i < widget.count; i++) _ShelfItem(i < _names[t].length ? _names[t][i] : '${_names[t][i % _names[t].length]} ${i ~/ _names[t].length + 1}', i % 3 == 0 ? 'ring_glow · Glow Intensity' : (i % 3 == 1 ? 'title_card' : '見出し_タイトル'), t * 17 + i),
          ],
      ];

  @override
  void didUpdateWidget(_ShelfPanel old) {
    super.didUpdateWidget(old);
    if (old.count != widget.count) {
      items = _make();
      status = '';
    }
  }

  static const _kinds = ['ease', 'colour', 'effect stack', 'value'];

  @override
  Widget build(BuildContext context) {
    final list = items[tab];
    return _Panel(
      title: 'Mine',
      sub: 'Saved aspects',
      w: 320,
      h: widget.h,
      child: Column(mainAxisSize: _FitScope.of(context) ? MainAxisSize.min : MainAxisSize.max, children: [
        Padding(padding: const EdgeInsets.fromLTRB(12, 8, 12, 8), child: Segmented(items: const ['Ease', 'Color', 'Effects', 'Values'], index: tab, expand: true, onChanged: (i) => setState(() => tab = i))),
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 0, 12, 8),
          child: Row(children: [
            Expanded(child: Text('${list.length}', maxLines: 1, style: _cnt(N.g76))), // the tab already names the kind; only the number is new
            QuietButton('Save from selected layer', onTap: () => setState(() {
                  items[tab] = [_ShelfItem('${_kinds[tab][0].toUpperCase()}${_kinds[tab].substring(1)} from ring_glow', 'just now', 90 + tab), ...list];
                  status = 'Saved';
                })),
          ]),
        ),
        const _Hair(),
        _grow(
          context,
          list.isEmpty
              ? Padding(padding: const EdgeInsets.fromLTRB(12, 12, 12, 12), child: Text('Nothing saved yet', style: T.label(N.g63).copyWith(fontSize: 11)))
              : _ScrollFade(child: ListView.builder(
                  shrinkWrap: _FitScope.of(context),
                  padding: const EdgeInsets.only(bottom: 8),
                  itemCount: list.length,
                  itemExtent: 40,
                  itemBuilder: (_, i) => Hov(
                    onTap: () => setState(() {
                      hot = i;
                      status = tab == 0 ? 'Applied to 2 of 3 · 1 skipped' : 'Applied to 3';
                    }),
                    builder: (_, h) => _Gut(
                      tick: hot == i ? N.g95 : null,
                      child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 12),
                      color: hot == i ? N.g20 : (h ? N.g15 : null),
                      child: Row(children: [
                        SizedBox(width: 44, height: 24, child: CustomPaint(painter: _ShelfThumb(tab, list[i].seed))),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                            Text(list[i].name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(h || hot == i ? N.g95 : N.g91)),
                            const SizedBox(height: 4),
                            Text(list[i].meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
                          ]),
                        ),
                        SizedBox(width: 56, child: Align(alignment: Alignment.centerRight, child: h ? Text('Apply to 3', maxLines: 1, style: T.label(N.g95)) : null)),
                      ]),
                    )),
                  ),
                )),
        ),
        if (status.isNotEmpty) ...[
          const _Hair(),
          _Hint(status),
        ],
      ]),
    );
  }
}

class _ShelfThumb extends CustomPainter {
  const _ShelfThumb(this.tab, this.seed);
  final int tab, seed;
  @override
  void paint(Canvas c, Size s) {
    final r = math.Random(seed);
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(3)), Paint()..color = N.g07);
    switch (tab) {
      case 0:
        final a = r.nextDouble() * .6, b = .4 + r.nextDouble() * .9;
        final path = Path()..moveTo(4, s.height - 4)..cubicTo(s.width * a, s.height - 4, s.width * (1 - a * .5), s.height * (1 - b) + 2, s.width - 4, 4);
        c.drawPath(path, Paint()..color = N.g95..style = PaintingStyle.stroke..strokeWidth = 1.2);
      case 1:
        for (var i = 0; i < 4; i++) {
          c.drawRect(Rect.fromLTWH(3.0 + i * 9.5, 3, 8, s.height - 6), Paint()..color = _swatch[1 + (seed + i * 2) % 6].withValues(alpha: .85));
        }
      case 2:
        for (var i = 0; i < 3; i++) {
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(4, 3.0 + i * 6.5, s.width - 8 - (i * 6 + seed % 3 * 3), 4.5), const Radius.circular(1.5)), Paint()..color = N.g63);
        }
      default:
        final tp = TextPainter(text: TextSpan(text: '${(seed * 7) % 90 + 10}', style: T.value(N.g91)), textDirection: TextDirection.ltr)..layout();
        tp.paint(c, Offset((s.width - tp.width) / 2, (s.height - tp.height) / 2));
    }
  }

  @override
  bool shouldRepaint(_ShelfThumb o) => o.tab != tab || o.seed != seed;
}

List<WidgetbookUseCase> _i11Cases() => [
      _story('I11-a Clipboard of one aspect', 'I11-a', 'Copy Value, Ease, Color, Keys or Effects; the clipboard says which one it holds', _Habit.habit,
          'Contract: one aspect at a time, named in a word; paste goes to every selected layer in one step; layers that cannot take it say why. Knob starts the state: empty, holding, pasted, or pasted with skips.',
          (c, h) => _ClipPanel(h: h, preset: c.knobs.object.dropdown<String>(label: 'State', options: const ['empty', 'holding', 'pasted', 'partial'], initialOption: 'holding', labelBuilder: (s) => switch (s) { 'empty' => 'Empty', 'holding' => 'Holding Ease', 'pasted' => 'Pasted', _ => 'Pasted, 2 skipped' }))),
      _story('I11-a Clipboard partial paste', 'I11-a', 'Ease pasted to 4 layers; two cannot take it', _Habit.habit,
          'Contract: a failure is a sentence with the reason and a way forward (X6): the skipped rows carry a red tick and say "no keys" or "no Glow", the summary says what to do next.',
          (c, h) => _ClipPanel(h: h, preset: 'partial')),
      _story('I11-b Copy with link', 'I11-b', 'A pasted value stays tied to its source: change one, all move', _Habit.habit,
          'Contract: a linked row has a purple tick (Attach family) and the word "Shared ×3"; Unlink keeps the number; deleting the source keeps the value on the pasted layers. Knob: linked / unlinked / source deleted.',
          (c, h) => _LinkPanel(h: h, preset: c.knobs.object.dropdown<String>(label: 'State', options: const ['linked', 'unlinked', 'deleted'], initialOption: 'linked', labelBuilder: (s) => s))),
      _story('I11-c Eyedropper', 'I11-c', 'Pick a value from another layer: hover to read, click to keep, Esc to leave', _Habit.addition,
          'Contract: hovering shows the other layer\'s value in the top row without writing it (peek); a layer without that property is grey and says so; release commits as one step. Knob starts a pick on Intensity or Color.',
          (c, h) => _DropPanel(h: h, start: c.knobs.object.dropdown<String>(label: 'Start', options: const ['none', 'gi', 'gc'], initialOption: 'gi', labelBuilder: (s) => switch (s) { 'none' => 'Not picking', 'gi' => 'Intensity', _ => 'Color' }))),
      _story('I11-d Shelf of aspects', 'I11-d', 'Save an ease, colour or effect stack once; apply it to selected layers later', _Habit.addition,
          'Contract: one shelf per aspect; Save needs no name; applying writes only same-named properties and the footer says what was skipped. Browser-side region at 320 px. Knob: items per shelf (0 = empty, 30 = scale).',
          (c, h) => _ShelfPanel(h: h, count: c.knobs.int.slider(label: 'Items per shelf', initialValue: 6, min: 0, max: 40))),
    ];
