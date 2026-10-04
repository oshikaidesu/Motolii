part of 'panel_inspector_b.dart';

// ---- I8: the effect stack ------------------------------------------------------------------------------------------------------------------

class _Card {
  _Card(this.id, this.fx, {this.on = true, this.open = true, this.error});
  final String id;
  _Fx fx;
  final _Vals vals = _Vals();
  bool on, open, menu = false;
  String? error;
  String name(int lang) => _fxName(fx, lang);
}

_Fx _generic(int i) {
  final n = _extraNames[i % _extraNames.length];
  return _Fx('x$i', n.$1, n.$2, [
    _P('x$i.amt', 'Amount', '量', _K.num, 50, major: true),
    _P('x$i.size', 'Size', 'サイズ', _K.num, 30, max: 500, unit: 'px', major: true),
    _P('x$i.mode', 'Mode', 'モード', _K.choice, 0, max: 2, opts: const ['Standard', 'Soft', 'Hard'], major: true),
  ]);
}

List<_Card> _makeCards(int n) => [
      for (var i = 0; i < n; i++)
        i < 6
            ? _Card('c$i', _stackLib[i],
                on: i != 1, open: i < 3, error: i == 4 ? 'Shader failed to compile: line 14, unexpected "}".' : null)
            : _Card('c$i', _generic(i), open: false),
    ];

const _swapPool = {
  'glow': ['Glow', 'Soft Bloom', 'Neon Glow', 'Lens Bloom', 'Bloom (Fast)', 'Orton Glow', 'Diffuse Light', 'Halation', 'Star Glow'],
  'blur': ['Gaussian Blur', 'Fast Box Blur', 'Directional Blur', 'Radial Blur', 'Lens Blur', 'Camera Lens Blur', 'Bilateral Blur'],
};

class _StackPanel extends StatefulWidget {
  const _StackPanel({required this.h, required this.n, required this.mode, this.w = 282, this.lang = 0, this.candidates = 9, this.solo = false, this.foldAll = false, this.peekStart = false});
  final double h, w;
  final int n, lang, candidates;
  final String mode; // a | c | d
  final bool solo, foldAll, peekStart;
  @override
  State<_StackPanel> createState() => _StackPanelState();
}

class _StackPanelState extends State<_StackPanel> {
  late List<_Card> cards = _makeCards(widget.n);
  String? selected, soloId;
  int? peek; // hot-swap candidate index being previewed
  final focus = FocusNode();

  @override
  void initState() {
    super.initState();
    _soloSync();
    if (widget.foldAll) {
      for (final c in cards) {
        c.open = false;
      }
    }
    _peekSync();
  }

  void _peekSync() {
    if (widget.mode == 'c' && widget.peekStart && cards.isNotEmpty && widget.candidates > 0) {
      selected = cards.first.id;
      peek = math.min(2, widget.candidates - 1);
    }
  }

  void _soloSync() => soloId = widget.mode == 'd' && widget.solo && cards.length > 2 ? cards[2].id : null;

  @override
  void didUpdateWidget(_StackPanel old) {
    super.didUpdateWidget(old);
    if (old.n != widget.n) {
      cards = _makeCards(widget.n);
      selected = null;
      peek = null;
      _soloSync();
    }
    if (old.peekStart != widget.peekStart || old.candidates != widget.candidates) {
      peek = null;
      _peekSync();
    }
    if (old.solo != widget.solo) {
      _soloSync();
    }
    if (old.foldAll != widget.foldAll) {
      for (final c in cards) {
        c.open = !widget.foldAll;
      }
    }
  }

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  List<String> _cand(_Card c) {
    final base = _swapPool[c.fx.id] ?? [c.fx.en, '${c.fx.en} 2', '${c.fx.en} Pro', '${c.fx.en} (Fast)', '${c.fx.en} Soft'];
    return List.generate(widget.candidates, (i) => i < base.length ? base[i] : '${base[i % base.length]} ${i ~/ base.length + 1}');
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent || widget.mode != 'c') {
      return KeyEventResult.ignored;
    }
    final c = cards.where((x) => x.id == selected).firstOrNull;
    if (c == null) {
      return KeyEventResult.ignored;
    }
    final cand = _cand(c), k = e.logicalKey;
    if (k == LogicalKeyboardKey.keyQ) {
      setState(() => peek = peek == null ? 0 : null);
    } else if (peek != null && cand.isNotEmpty && (k == LogicalKeyboardKey.arrowDown || k == LogicalKeyboardKey.arrowUp)) {
      setState(() => peek = (peek! + (k == LogicalKeyboardKey.arrowDown ? 1 : cand.length - 1)) % cand.length);
    } else if (peek != null && k == LogicalKeyboardKey.enter) {
      _commit(c, cand);
    } else if (peek != null && k == LogicalKeyboardKey.escape) {
      setState(() => peek = null);
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  void _commit(_Card c, List<String> cand) {
    final name = cand[peek!];
    setState(() {
      c.fx = _Fx(c.fx.id, name, name, c.fx.params, jpFx: false);
      peek = null;
    });
  }

  Widget _grip(int i) => ReorderableDragStartListener(
        index: i,
        child: MouseRegion(cursor: SystemMouseCursors.grab, child: SizedBox(width: 16, height: 40, child: CustomPaint(painter: _GripPaint()))),
      );

  Widget _card(int i) {
    final c = cards[i];
    final w = widget;
    final isSel = selected == c.id;
    final muted = soloId != null && soloId != c.id;
    final failed = c.error != null;
    final nm = peek != null && isSel ? _cand(c)[peek!] : c.name(w.lang);
    final rows = c.fx.params.where((p) => p.major).take(4).toList();
    final (String, Color)? status = failed ? ('Failed', Role.error) : (!c.on ? ('Bypassed', N.g76) : (muted ? ('Muted by solo', N.g76) : null));
    return Container(
      key: ValueKey(c.id),
      margin: const EdgeInsets.fromLTRB(12, 0, 12, 4),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)),
      child: _Gut(
        tick: failed ? Role.error : (isSel ? N.g95 : null),
        child: Opacity(
        opacity: muted ? .6 : 1,
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
          SizedBox(
            height: status == null ? 32 : 40,
            child: Row(children: [
              const SizedBox(width: 6),
              SizedBox(width: 20, child: Center(child: Text((i + 1).toString().padLeft(2, '0'), style: _cnt(N.g76)))),
              _grip(i),
              Expanded(
                child: Hov(
                  onTap: () => setState(() {
                    c.open = !c.open;
                    selected = c.id;
                  }),
                  builder: (_, h) => Row(children: [
                    _Chev(c.open && !failed, color: h ? N.g95 : N.g76),
                    const SizedBox(width: 4),
                    Expanded(
                      child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Text.rich(
                          TextSpan(children: [
                            TextSpan(text: nm, style: T.name(c.on ? N.g95 : N.g63).copyWith(decoration: c.on ? null : TextDecoration.lineThrough)),
                                          ]),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                        if (status != null) ...[const SizedBox(height: 4), status.$2 == Role.error ? Row(mainAxisSize: MainAxisSize.min, children: [const ErrMark(size: 10, gap: 4), Text(status.$1, style: T.label(status.$2))]) : Text(status.$1, style: T.label(status.$2))],
                      ]),
                    ),
                  ]),
                ),
              ),
              if (w.mode == 'd' && (isSel || soloId != null || soloId == c.id)) ...[
                _Word('Solo', on: soloId == c.id, onTap: () => setState(() => soloId = soloId == c.id ? null : c.id)),
                const SizedBox(width: 8),
              ] else if (w.mode == 'c' && isSel) ...[
                _Word('Swap  Q', on: peek != null, onTap: () => setState(() => peek = peek == null ? 0 : null)),
                const SizedBox(width: 8),
              ],
              Hov(
                onTap: () => setState(() => c.menu = !c.menu),
                builder: (_, h) => SizedBox(width: 22, height: 32, child: Center(child: SizedBox(width: 12, height: 12, child: CustomPaint(painter: _DotsPaint(h || c.menu ? N.g95 : N.g63))))),
              ),
              IgnorePointer(ignoring: muted, child: PillSwitch(on: c.on, onChanged: (v) => setState(() => c.on = v))),
              const SizedBox(width: 8),
            ]),
          ),
          if (c.menu)
            Padding(
              padding: const EdgeInsets.fromLTRB(36, 0, 12, 8),
              child: _words([
                _Word('Duplicate', onTap: () => setState(() {
                      cards.insert(i + 1, _Card('${c.id}d${cards.length}', c.fx)..open = false);
                      c.menu = false;
                    })),
                _Word('Reset', onTap: () => setState(() {
                      c.vals.clear();
                      c.menu = false;
                    })),
                _Word('Remove', onTap: () => setState(() => cards.removeAt(i))),
              ]),
            ),
          if (failed)
            Padding(
              padding: const EdgeInsets.fromLTRB(36, 0, 12, 8),
              child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(c.error!, style: T.label(N.g91).copyWith(height: 1.4)),
                _gap(8),
                _words([_Word('Retry', onTap: () => setState(() => c.error = null)), const _Word('Open source')]),
              ]),
            )
          else if (c.open) ...[
            const _Hair(),
            Opacity(opacity: c.on ? 1 : .5, child: Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Column(children: [for (final p in rows) _PRow(p, c.vals, key: ValueKey('${c.id}.${p.id}'), lang: w.lang, jp: c.fx.jpFx)]))),
            if (c.fx.params.length > rows.length) Padding(padding: const EdgeInsets.fromLTRB(12, 0, 12, 8), child: Text('${c.fx.params.length - rows.length} more', style: _cnt(N.g76).copyWith(fontWeight: FontWeight.w500))),
          ],
          if (isSel && w.mode == 'c' && peek != null) _swapBar(c),
        ]),
      )),
    );
  }

  Widget _swapBar(_Card c) {
    final cand = _cand(c);
    if (cand.isEmpty) {
      return const _Hint('No candidates', color: N.g63);
    }
    final lo = math.min(math.max(peek! - 2, 0), math.max(0, cand.length - 5)), shown = cand.skip(lo).take(5).toList();
    final kept = 3 - peek! % 2, lost = peek! % 3;
    return Container(
      margin: const EdgeInsets.fromLTRB(8, 0, 8, 8),
      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(8, 6, 8, 4),
          child: Row(children: [
            Text('${peek! + 1} / ${cand.length}', style: T.value(N.g76)),
            const Spacer(),
            Text('‹  ›', style: T.name(N.g63)), // the stepping marks
          ]),
        ),
        for (var j = 0; j < shown.length; j++)
          Hov(
            onTap: () => setState(() => peek = lo + j),
            builder: (_, h) => Container(
              height: 24,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: lo + j == peek ? N.g20 : (h ? N.g15 : null)),
              child: Align(alignment: Alignment.centerLeft, child: Text(shown[j], style: T.name(lo + j == peek ? N.g95 : N.g76))),
            ),
          ),
        Padding(
          padding: const EdgeInsets.fromLTRB(8, 6, 8, 6),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Row(children: [
              Text('$kept kept', style: _cnt(N.g91)),
              if (lost > 0) ...[const SizedBox(width: 8), Text('↺ $lost', style: _cnt(N.g76))],
            ]),
            _gap(4),
            _words([
              _Word('Keep', onTap: () => _commit(c, cand)),
              _Word('Leave as it was', onTap: () => setState(() => peek = null)),
            ]),
          ]),
        ),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) {
    final w = widget;
    return _Panel(
      title: 'hero_title_01',
      sub: 'Text layer',
      w: w.w,
      h: w.h,
      child: Focus(
        focusNode: focus,
        onKeyEvent: _key,
        child: Column(mainAxisSize: _FitScope.of(context) ? MainAxisSize.min : MainAxisSize.max, children: [
          _Sect('Effects', count: cards.length, sub: w.mode == 'd' && soloId != null ? 'solo on' : null, trailing: cards.isEmpty ? null : _Word(cards.every((c) => !c.open) ? 'Unfold all' : 'Fold all', onTap: () => setState(() {
                final to = cards.every((c) => !c.open);
                for (final c in cards) {
                  c.open = to;
                }
              }))),
          _grow(
            context,
            cards.isEmpty
                ? Padding(
                    padding: const EdgeInsets.fromLTRB(12, 12, 12, 12),
                    child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                      Text('No effects', style: T.name(N.g91)),
                      _gap(8),
                      QuietButton('Add Glow', onTap: () => setState(() => cards = [_Card('n', _glow)])),
                    ]),
                  )
                : ReorderableList(
                    shrinkWrap: _FitScope.of(context),
                    padding: const EdgeInsets.only(bottom: 8),
                    itemCount: cards.length,
                    proxyDecorator: (child, i, a) => Opacity(opacity: .9, child: child),
                    onReorderItem: (a, b) => setState(() {
                      final x = cards.removeAt(a);
                      cards.insert(b, x);
                    }),
                    itemBuilder: (_, i) => _card(i),
                  ),
          ),
        ]),
      ),
    );
  }
}

class _DotsPaint extends CustomPainter {
  const _DotsPaint(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    for (var i = 0; i < 3; i++) {
      cv.drawCircle(Offset(1.5 + i * 4.5, s.height / 2), 1.2, Paint()..color = c);
    }
  }

  @override
  bool shouldRepaint(_DotsPaint o) => o.c != c;
}

class _GripPaint extends CustomPainter {
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = N.g63;
    for (var i = 0; i < 2; i++) {
      for (var j = 0; j < 3; j++) {
        c.drawCircle(Offset(s.width / 2 - 2 + i * 4, s.height / 2 - 4 + j * 4), 1, p);
      }
    }
  }

  @override
  bool shouldRepaint(_GripPaint o) => false;
}

// ---- I8-b: a chain, left to right (withdrawn; kept to compare) -------------------------------------------------------------------------

class _ChainPanel extends StatefulWidget {
  const _ChainPanel({required this.h, required this.n, this.lang = 0});
  final double h;
  final int n, lang;
  @override
  State<_ChainPanel> createState() => _ChainPanelState();
}

class _ChainPanelState extends State<_ChainPanel> {
  late List<_Card> cards = _makeCards(widget.n);
  int sel = 0;

  @override
  void didUpdateWidget(_ChainPanel old) {
    super.didUpdateWidget(old);
    if (old.n != widget.n) {
      cards = _makeCards(widget.n);
      sel = 0;
    }
  }

  @override
  Widget build(BuildContext context) {
    final s = cards.isEmpty ? null : cards[sel.clamp(0, cards.length - 1)];
    return _Panel(
      title: 'hero_title_01',
      sub: 'Text layer',
      h: widget.h,
      child: _Lv(children: [
        _Sect('Chain', count: cards.length),
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 4, 12, 8),
          child: Wrap(spacing: 4, runSpacing: 6, crossAxisAlignment: WrapCrossAlignment.center, children: [
            for (var i = 0; i < cards.length; i++) ...[
              Hov(
                onTap: () => setState(() => sel = i),
                builder: (_, h) => AnimatedContainer(
                  duration: Mo.dur,
                  curve: Mo.ease,
                  height: 26,
                  constraints: const BoxConstraints(maxWidth: 150),
                  padding: const EdgeInsets.only(left: 4, right: 8),
                  decoration: BoxDecoration(color: sel == i ? N.g20 : (h ? N.g26 : N.g13), borderRadius: BorderRadius.circular(5), border: Border(bottom: BorderSide(color: sel == i ? Role.selected : const Color(0x00000000)))),
                  child: Row(mainAxisSize: MainAxisSize.min, children: [
                    Hov(
                      onTap: () => setState(() => cards[i].on = !cards[i].on),
                      builder: (_, dh) => SizedBox(
                        width: 18,
                        height: 26,
                        child: Center(child: Container(width: 8, height: 8, decoration: BoxDecoration(shape: BoxShape.circle, color: cards[i].on ? N.g95 : null, border: Border.all(color: cards[i].on ? N.g95 : (dh ? N.g76 : N.g63), width: 1.5)))),
                      ),
                    ),
                    Flexible(child: Text(cards[i].name(widget.lang), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(cards[i].on ? (sel == i ? N.g95 : N.g76) : N.g63).copyWith(decoration: cards[i].on ? null : TextDecoration.lineThrough))),
                  ]),
                ),
              ),
              if (i < cards.length - 1) Text('→', style: T.label(N.g76).copyWith(fontSize: 11)),
            ],
          ]),
        ),
        const _Hair(),
        if (s == null)
          const _Hint('Empty', color: N.g63)
        else ...[
          _Sect('${sel + 1}.  ${s.name(widget.lang)}', sub: s.on ? null : 'bypassed'),
          for (final p in s.fx.params.where((p) => p.major).take(5)) _PRow(p, s.vals, key: ValueKey('${s.id}.${p.id}'), lang: widget.lang, jp: s.fx.jpFx),
          if (s.error != null) _Hint(s.error!, color: N.g91),
        ],
      ]),
    );
  }
}

List<WidgetbookUseCase> _i8Cases() => [
      _story('I8-a Card stack', 'I8-a', 'Effects as cards: fold from the header, reorder only from the grip', _Habit.habit,
          'Contract: the header folds and only the grip drags (D-I8); the order is numbered, top applies first; a bypassed card is struck through and dimmed with a word; a failed one says why and offers Retry. Drag a grip to reorder.',
          (c, h) => _StackPanel(h: h, n: c.knobs.int.slider(label: 'Effects', initialValue: 6, min: 0, max: 40), mode: 'a', lang: _lang(c), foldAll: c.knobs.boolean(label: 'Fold all', initialValue: false))),
      _story('I8-a Card stack at scale', 'I8-a', 'The same stack when the layer carries two dozen effects', _Habit.habit,
          'Contract: all cards start folded; the count and "Unfold all" sit in the header; every header is one line, so fifteen effects fit without scrolling.',
          (c, h) => _StackPanel(h: h, n: 24, mode: 'a', lang: _lang(c), foldAll: c.knobs.boolean(label: 'Fold all', initialValue: true))),
      _story('I8-a Card stack empty', 'I8-a', 'A layer with no effects', _Habit.habit,
          'Contract: empty says what is missing and the two ways to add (Browser drop, Cmd-K), with one button for the common case (X6).',
          (c, h) => _StackPanel(h: h, n: 0, mode: 'a', lang: _lang(c))),
      _story('I8-a Card stack in a dock', 'I8-a', 'The stack at 224 px', _Habit.habit,
          'Contract: the header keeps number, grip, name, state word and switch; the name is what gives way (ellipsis); rows below stack label over value.',
          (c, h) => _StackPanel(h: h, w: 224, n: 6, mode: 'a', lang: _lang(c))),
      _story('I8-b Chain left to right', 'I8-b', 'The stack as one line of chips; only the selected effect opens', _Habit.retired,
          'Contract: the chain shows order at a glance; a dot on each chip bypasses it with one click. Withdrawn in the habit ledger (C.4: it replaces the stack people know); kept here only to compare with I8-a at 6 and 12 effects.',
          (c, h) => _ChainPanel(h: h, n: c.knobs.int.slider(label: 'Effects', initialValue: 6, min: 0, max: 14), lang: _lang(c))),
      _story('I8-c Hot-swap with Q', 'I8-c', 'Select a card, press Q, step candidates with Up / Down, Enter keeps, Esc leaves', _Habit.habit,
          'Contract: stepping previews the candidate on the card ("peek") and says how many values carry over; Enter is one undo step, Esc changes nothing. Click a card first so it has focus. Knob: candidates (0 = none to swap to).',
          (c, h) => _StackPanel(h: h, n: 4, mode: 'c', lang: _lang(c), candidates: c.knobs.int.slider(label: 'Candidates', initialValue: 9, min: 0, max: 40), peekStart: c.knobs.boolean(label: 'Swap open at start', initialValue: true))),
      _story('I8-d Solo one effect', 'I8-d', 'Solo mutes the others for a moment and remembers their own on / off', _Habit.habit,
          'Contract: while one is soloed the others are dim and say "Muted"; their switches keep their own state (the bypassed one stays bypassed after Solo ends); the Solo word shows on the selected card or while any solo is on.',
          (c, h) => _StackPanel(h: h, n: 6, mode: 'd', lang: _lang(c), solo: c.knobs.boolean(label: 'Solo on Gaussian Blur at start', initialValue: true))),
    ];
