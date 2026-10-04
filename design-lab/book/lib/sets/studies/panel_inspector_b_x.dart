part of 'panel_inspector_b.dart';

// ---- X: cross-cutting problems that live in an Inspector ---------------------------------------------------------------------------------------

// ---- X2: try, peek, keep or leave: one grammar ---------------------------------------------------------------------------------------------------

class _PeekPanel extends StatefulWidget {
  const _PeekPanel({required this.h});
  final double h;
  @override
  State<_PeekPanel> createState() => _PeekPanelState();
}

class _PeekPanelState extends State<_PeekPanel> {
  static const sections = [
    ('Blending Mode', ['Normal', 'Add', 'Multiply', 'Screen', 'Overlay', 'Soft Light']),
    ('Ease', ['Linear', 'Ease Out', 'Ease In-Out', 'Back Out', 'Bounce']),
    ('Glow Color', ['#F2F2F2', '#E974AB', '#7DD5B1', '#4781E5', '#EFCB4E', '#F69260']),
  ];
  final kept = [0, 1, 4];
  int? ps, pi; // peeking section / index
  int steps = 0;
  final focus = FocusNode();

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  void _enter(int s, int i) => setState(() {
        ps = s;
        pi = i;
      });

  void _leave(int s, int i) => setState(() {
        if (ps == s && pi == i) {
          ps = pi = null;
        }
      });

  void _keep(int s, int i) => setState(() {
        if (kept[s] != i) {
          kept[s] = i;
          steps++;
        }
      });

  Widget _peekChip(int s, int i) => MouseRegion(
        onEnter: (_) => _enter(s, i),
        onExit: (_) => _leave(s, i),
        child: Hov(
          onTap: () => _keep(s, i),
          builder: (_, h) {
            final on = kept[s] == i, pk = ps == s && pi == i && !on;
            final isCol = s == 2;
            return Container(
              height: 24,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: on ? N.g20 : (pk ? N.g26 : (h ? N.g15 : N.g13)), borderRadius: BorderRadius.circular(5), border: Border(bottom: BorderSide(color: on || pk ? Role.selected : const Color(0x00000000)))),
              child: Row(mainAxisSize: MainAxisSize.min, children: [
                if (isCol) ...[Container(width: on || pk ? 14 : 20, height: 10, decoration: BoxDecoration(color: Color(int.parse('FF${sections[s].$2[i].substring(1)}', radix: 16)), borderRadius: BorderRadius.circular(2))), if (on || pk) const SizedBox(width: 6)], // the hex is written only on the kept or peeked swatch
                if (!isCol || on || pk) Text(sections[s].$2[i], style: isCol ? T.value(on ? N.g95 : N.g76) : T.name(on ? N.g95 : N.g76)),
              ]),
            );
          },
        ),
      );

  Widget _peekRow(int s, int i) => MouseRegion(
        onEnter: (_) => _enter(s, i),
        onExit: (_) => _leave(s, i),
        child: Hov(
          onTap: () => _keep(s, i),
          builder: (_, h) {
            final on = kept[s] == i, pk = ps == s && pi == i && !on;
            return _Gut(
              tick: on ? N.g95 : null,
              child: Container(
                height: 28,
                padding: const EdgeInsets.symmetric(horizontal: 12),
                decoration: BoxDecoration(color: on ? N.g20 : (pk ? N.g26 : (h ? N.g15 : null)), border: Border(bottom: BorderSide(color: pk ? Role.selected : _clear))),
                child: Row(children: [
                  Expanded(child: Text(sections[s].$2[i], maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g76))),
                  SizedBox(width: 36, height: 16, child: CustomPaint(painter: _EaseMini(i, on || pk ? N.g95 : N.g63))),
                ]),
              ),
            );
          },
        ),
      );

  @override
  Widget build(BuildContext context) {
    final peeking = ps != null && pi != null && pi != kept[ps!];
    return _Panel(
      title: 'title_card',
      sub: 'Text layer',
      h: widget.h,
      child: Focus(
        focusNode: focus,
        autofocus: true,
        onKeyEvent: (n, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
            setState(() => ps = pi = null);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: _Lv(children: [
          for (var s = 0; s < sections.length; s++) ...[
            _Sect(sections[s].$1, sub: peeking && ps == s ? '${sections[ps!].$2[pi!]}  peek' : sections[s].$2[kept[s]]),
            if (s == 1)
              for (var i = 0; i < sections[s].$2.length; i++) _peekRow(s, i)
            else
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                child: Wrap(spacing: 4, runSpacing: 4, children: [for (var i = 0; i < sections[s].$2.length; i++) _peekChip(s, i)]),
              ),
          ],
          _gap(8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Row(children: [
              _Word('Undo', onTap: steps == 0 ? null : () => setState(() {
                    steps--;
                    kept..[0] = 0..[1] = 1..[2] = 4;
                  })),
              if (steps > 0) ...[const SizedBox(width: 8), Text('$steps', style: _cnt(N.g76))],
            ]),
          ),
        ]),
      ),
    );
  }
}

class _EaseMini extends CustomPainter {
  const _EaseMini(this.kind, this.c);
  final int kind;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    const curves = [Curves.linear, Curves.easeOut, Curves.easeInOut, Curves.easeOutBack, Curves.bounceOut];
    final path = Path()..moveTo(0, s.height - 1);
    for (var i = 1; i <= 24; i++) {
      final t = i / 24;
      path.lineTo(s.width * t, (s.height - 2) * (1 - curves[kind].transform(t).clamp(-.3, 1.3)) + 1);
    }
    cv.drawPath(path, Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.2);
  }

  @override
  bool shouldRepaint(_EaseMini o) => o.kind != kind || o.c != c;
}

// ---- X3: the keyboard alone ----------------------------------------------------------------------------------------------------------------

class _KeysPanel extends StatefulWidget {
  const _KeysPanel({required this.h});
  final double h;
  @override
  State<_KeysPanel> createState() => _KeysPanelState();
}

class _KeysPanelState extends State<_KeysPanel> {
  static const rows = [
    ('Position X', 'px', 0, 960.0, 'P'),
    ('Position Y', 'px', 0, 540.0, ''),
    ('Scale', '%', 0, 100.0, 'S'),
    ('Rotation', '°', 1, 0.0, 'R'),
    ('Opacity', '%', 0, 100.0, 'T'),
    ('Anchor Point X', 'px', 0, 0.0, 'A'),
  ];
  final focus = FocusNode();
  int sel = 0;
  late final List<double> v = rows.map((r) => r.$4).toList();

  @override
  void initState() {
    super.initState();
    focus.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    final k = e.logicalKey;
    final double m = (HardwareKeyboard.instance.isShiftPressed ? 10 : 1) * (HardwareKeyboard.instance.isAltPressed ? .1 : 1);
    final r = rows[sel];
    final step = (r.$3 > 0 ? .1 : 1) * m;
    setState(() {
      if (k == LogicalKeyboardKey.arrowDown) {
        sel = (sel + 1) % rows.length;
      } else if (k == LogicalKeyboardKey.arrowUp) {
        sel = (sel + rows.length - 1) % rows.length;
      } else if (k == LogicalKeyboardKey.arrowRight) {
        v[sel] += step;
      } else if (k == LogicalKeyboardKey.arrowLeft) {
        v[sel] -= step;
      } else if (k == LogicalKeyboardKey.backspace) {
        v[sel] = r.$4;
      } else {
        final i = rows.indexWhere((x) => x.$5.isNotEmpty && x.$5.toLowerCase() == (e.character ?? '').toLowerCase());
        if (i >= 0) {
          sel = i;
        }
      }
    });
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) => _Panel(
        title: 'star_burst',
        sub: 'Layer Transform',
        h: widget.h,
        child: Focus(
          focusNode: focus,
          autofocus: true,
          onKeyEvent: _key,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: focus.requestFocus,
            child: _Lv(children: [
              const _Sect('Transform'),
              for (var i = 0; i < rows.length; i++)
                _Gut(
                  tick: sel == i ? (focus.hasFocus ? N.g95 : N.g44) : null, // no keyboard focus: the tick is dimmer and the row has no fill
                  child: Container(
                    height: 28,
                    padding: const EdgeInsets.symmetric(horizontal: 12),
                    color: sel == i && focus.hasFocus ? N.g20 : (sel == i ? N.g15 : null),
                    child: Row(children: [
                      Expanded(child: Text(rows[i].$1, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(sel == i ? N.g95 : N.g63).copyWith(fontSize: 11))),
                      if (rows[i].$5.isNotEmpty) Padding(padding: const EdgeInsets.only(right: 8), child: Container(width: 16, height: 16, alignment: Alignment.center, decoration: BoxDecoration(borderRadius: BorderRadius.circular(3), border: Border.all(color: sel == i ? N.g63 : N.g26)), child: Text(rows[i].$5, style: T.micro(sel == i ? N.g95 : N.g76)))), // the jump key as a small keycap
                      SizedBox(width: 8, child: sel == i ? Text('‹', maxLines: 1, style: T.name(N.g63)) : null),
                      SizedBox(
                        width: 78,
                        child: Text.rich(TextSpan(children: [
                          TextSpan(text: _f(v[i], rows[i].$3), style: T.value(sel == i ? N.g95 : N.g91)),
                          TextSpan(text: ' ${_unit(rows[i].$2)}', style: T.label(N.g76).copyWith(fontWeight: FontWeight.w500)),
                        ]), maxLines: 1, textAlign: TextAlign.right),
                      ),
                      SizedBox(width: 8, child: sel == i ? Align(alignment: Alignment.centerRight, child: Text('›', maxLines: 1, style: T.name(N.g63))) : null),
                    ]),
                  ),
                ),
            ]),
          ),
        ),
      );
}

// ---- X4: one grammar for drag, Shift, Alt, wheel, arrows -------------------------------------------------------------------------------------------

class _GrammarPanel extends StatefulWidget {
  const _GrammarPanel({required this.h});
  final double h;
  @override
  State<_GrammarPanel> createState() => _GrammarPanelState();
}

class _GrammarPanelState extends State<_GrammarPanel> {
  final d = GDoc();
  final vals = _Vals();
  bool shift = false, alt = false;
  static const ps = [
    _P('g.rot', 'Rotation', '回転', _K.num, 0, min: -360, max: 360, unit: '°', dec: 1),
    _P('g.op', 'Opacity', '不透明度', _K.num, 100, unit: '%'),
    _P('g.scale', 'Scale', 'スケール', _K.num, 100, min: 0, max: 1000, unit: '%'),
    _P('g.blur', 'Blur', 'ブラー', _K.num, 12, max: 500, unit: 'px', dec: 1),
  ];

  bool _kb(KeyEvent e) {
    final s = HardwareKeyboard.instance.isShiftPressed, a = HardwareKeyboard.instance.isAltPressed;
    if (s != shift || a != alt) {
      setState(() {
        shift = s;
        alt = a;
      });
    }
    return false;
  }

  @override
  void initState() {
    super.initState();
    HardwareKeyboard.instance.addHandler(_kb);
    d.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_kb);
    d.dispose();
    vals.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final double mult = (shift ? 10 : 1) * (alt ? .1 : 1);
    Widget cap(String t, bool hot) => AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          height: 20,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          alignment: Alignment.center,
          decoration: BoxDecoration(color: hot ? N.g26 : null, borderRadius: BorderRadius.circular(4), border: Border.all(color: hot ? Role.selected : N.g26)),
          child: Text(t, style: T.label(hot ? N.g95 : N.g76)),
        ); // a keycap: filled and outlined in the accent while its key is held
    return _Panel(
      title: 'star_burst',
      sub: 'Layer Transform',
      h: widget.h,
      child: _Lv(children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
          child: Row(children: [
            cap('Shift', shift),
            const SizedBox(width: 4),
            cap('Alt', alt),
            const Spacer(),
            Text('×${mult == 1 ? '1' : _f(mult, mult < 1 ? 1 : 0)}', style: T.title()),
          ]),
        ),
        const _Sect('Position'),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: XyzStack(d, const [Spec('pos.x', 'X', axis: 0, unit: 'px', dec: 0), Spec('pos.y', 'Y', axis: 1, unit: 'px', dec: 0), Spec('pos.z', 'Z', axis: 2, unit: 'px', dec: 0)]),
        ),
        _gap(8),
        const _Sect('Single values'),
        for (final p in ps) _PRow(p, vals, key: ValueKey(p.id), wheel: true),
      ]),
    );
  }
}

// ---- X5: who an edit will change -----------------------------------------------------------------------------------------------------------

class _ScopePanel extends StatefulWidget {
  const _ScopePanel({required this.h, required this.start});
  final double h;
  final int start;
  @override
  State<_ScopePanel> createState() => _ScopePanelState();
}

class _ScopePanelState extends State<_ScopePanel> {
  late int scope = widget.start;
  double op = 80;

  @override
  void didUpdateWidget(_ScopePanel old) {
    super.didUpdateWidget(old);
    if (old.start != widget.start) {
      scope = widget.start;
    }
  }

  static const names = ['title_card', 'star_burst', 'bg_gradient'];

  /// One target of the edit: a tick in the gutter = it will change, none (and a ghost cell) = it is left alone; the cell shows the value it will take.
  Widget _target(String name, {String? value, bool locked = false, bool key = false}) => _Gut(
        tick: locked ? null : N.g95,
        child: Container(
          constraints: const BoxConstraints(minHeight: 28),
          padding: const EdgeInsets.fromLTRB(12, 4, 12, 4),
          child: Row(children: [
            if (key) Padding(padding: const EdgeInsets.only(right: 8), child: SizedBox(width: 8, height: 8, child: CustomPaint(painter: _KeyDiamond(locked ? Role.disabled : Role.of(N.g95, Role.key))))),
            Expanded(child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(locked ? N.g63 : N.g76).copyWith(fontSize: 11))),
            const SizedBox(width: 8),
            SizedBox(
              width: 96,
              child: locked
                  ? _Num(value: 0, enabled: false, word: 'Locked', onDelta: (_) {})
                  : Container(
                      height: 20,
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g26)),
                      child: Stack(clipBehavior: Clip.none, children: [
                        Positioned.fill(child: Align(alignment: Alignment.centerRight, child: Text.rich(TextSpan(children: [TextSpan(text: '$value', style: T.value(N.g95)), TextSpan(text: ' %', style: T.label(N.g76).copyWith(fontWeight: FontWeight.w500))]), maxLines: 1))),
                        const Positioned(left: -5, top: 0, bottom: 0, child: Center(child: CustomPaint(size: Size(5, 8), painter: _GripTicks(N.g38)))), // the same grip as the Opacity cell below: a ghost cell is a number cell that is not yet drawn
                      ]),
                    ),
            ),
          ]),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final live = scope == 0 ? 1 : (scope == 1 ? 2 : 5);
    return _Panel(
      title: scope == 1 ? '3 layers' : 'title_card',
      sub: 'Transform',
      h: widget.h,
      child: _Lv(children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
          child: Segmented(items: const ['This layer', 'Selection', 'All keys'], index: scope, expand: true, onChanged: (i) => setState(() => scope = i)),
        ),
        _Sect(scope == 2 ? 'Keys' : 'Layers', count: scope == 0 ? null : live),
        if (scope == 0) _target('title_card', value: _f(op, 0)),
        if (scope == 1)
          for (var i = 0; i < 3; i++) _target(names[i], value: _f((op + i * 7).clamp(0, 100), 0), locked: i == 2),
        if (scope == 2)
          for (var i = 0; i < 5; i++) _target('0:${(i * 6).toString().padLeft(2, '0')}', value: _f((op + i * 5).clamp(0, 100), 0), key: true),
        _gap(4),
        const _Hair(),
        _VRow('Opacity', _Num(value: op, unit: '%', onDelta: (d) => setState(() => op = (op + d).clamp(0, 100).toDouble()))),
      ]),
    );
  }
}

/// A key as a small diamond (a gray shape, not a new symbol: it is the Timeline's own key mark at 8 px).
class _KeyDiamond extends CustomPainter {
  const _KeyDiamond(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final m = s.center(Offset.zero), r = s.width / 2;
    final d = Path()..moveTo(m.dx, m.dy - r)..lineTo(m.dx + r, m.dy)..lineTo(m.dx, m.dy + r)..lineTo(m.dx - r, m.dy)..close();
    cv.drawPath(d, Paint()..color = c);
    if (Role.keyEdge != null && c != Role.disabled) cv.drawPath(d, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = Role.keyEdge!);
  }

  @override
  bool shouldRepaint(_KeyDiamond o) => o.c != c;
}

// ---- X6: nothing, no match, failed, forbidden -----------------------------------------------------------------------------------------------------------

class _StateBlock extends StatelessWidget {
  const _StateBlock(this.what, this.next, {this.why, this.bad = false});
  final String what;
  final String? why;
  final List<String> next;
  final bool bad;
  @override
  Widget build(BuildContext context) => _Band(
        tick: bad ? Role.error : N.g63,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          if (bad) Row(children: [const ErrMark(gap: 6), Flexible(child: Text(what, style: T.name(N.g95).copyWith(height: 1.3)))]) else Text(what, style: T.name(N.g95).copyWith(height: 1.3)),
          if (why != null) ...[_gap(4), Text(why!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g63).copyWith(fontSize: 11))],
          _gap(8),
          _words([for (final n in next) _Word(n, onTap: _noop)]),
        ]),
      );
}

class _StatesPanel extends StatelessWidget {
  const _StatesPanel({required this.h, required this.state});
  final double h;
  final String state;
  @override
  Widget build(BuildContext context) {
    final all = <String, Widget>{
      'No layer selected': const _StateBlock('Nothing is selected', ['Select the top layer', 'Open Browser']),
      'No match': const _StateBlock('No property named "blur radi"', ['Show all 31', 'Search every layer']),
      'Effect failed': const _StateBlock('Glow could not run', ['Retry', 'Open source', 'Bypass'], why: 'Shader error, line 14', bad: true),
      'Locked layer': Column(children: [
        const _StateBlock('title_card is locked', ['Unlock layer']),
        _VRow('Position X', _Num(value: 960, unit: 'px', enabled: false, word: 'Locked', onDelta: (_) {})),
        _VRow('Opacity', _Num(value: 100, unit: '%', enabled: false, word: 'Locked', onDelta: (_) {})),
      ]),
      'Missing media': const _StateBlock('city_pass.mov is missing', ['Relink', 'Replace with solid'], bad: true),
    };
    return _Panel(
      title: state == 'All five' ? 'Five states' : (state == 'No layer selected' ? 'Nothing selected' : 'title_card'),
      sub: 'Inspector',
      h: h,
      child: _Lv(children: [
        if (state == 'All five')
          for (final e in all.entries) ...[_Sect(e.key), e.value]
        else
          all[state]!,
      ]),
    );
  }
}

// ---- X8: placed from the Browser, now adjust --------------------------------------------------------------------------------------------------------

/// A look is a grey shape, a different one per look (soft = a filled disc, neon = a ring, film = lines, dream = two overlapping discs).
class _LookThumb extends CustomPainter {
  const _LookThumb(this.i);
  final int i;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(2)), Paint()..color = N.g07);
    final m = s.center(Offset.zero), r = s.height * .36;
    switch (i) {
      case 0:
        c.drawCircle(m, r, Paint()..color = N.g63..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2));
      case 1:
        c.drawCircle(m, r, Paint()..color = N.g91..style = PaintingStyle.stroke..strokeWidth = 1.5);
      case 2:
        for (var k = 0; k < 4; k++) {
          c.drawLine(Offset(m.dx - r * 1.4, m.dy - r + k * r * .66), Offset(m.dx + r * 1.4, m.dy - r + k * r * .66), Paint()..color = N.g63..strokeWidth = 1);
        }
      default:
        c.drawCircle(m.translate(-r * .5, 0), r, Paint()..color = N.g44.withValues(alpha: .8));
        c.drawCircle(m.translate(r * .5, 0), r, Paint()..color = N.g76.withValues(alpha: .6));
    }
  }

  @override
  bool shouldRepaint(_LookThumb o) => o.i != i;
}

class _TweakPanel extends StatefulWidget {
  const _TweakPanel({required this.h, required this.fx, this.lang = 0});
  final double h;
  final _Fx fx;
  final int lang;
  @override
  State<_TweakPanel> createState() => _TweakPanelState();
}

class _TweakPanelState extends State<_TweakPanel> {
  final vals = _Vals();
  int look = 0;
  bool more = false;
  static const looks = ['Soft', 'Neon', 'Film', 'Dream'];

  void _look(int i) {
    final fx = widget.fx;
    setState(() {
      look = i;
      vals.clear();
      for (final p in fx.params.where((p) => p.major && p.k == _K.num)) {
        vals.set(p, p.def * (.6 + i * .35), touch: false);
      }
    });
  }

  @override
  void didUpdateWidget(_TweakPanel old) {
    super.didUpdateWidget(old);
    if (old.fx.id != widget.fx.id) {
      vals.clear();
      look = 0;
    }
  }

  @override
  void dispose() {
    vals.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final fx = widget.fx;
    final main = fx.params.where((p) => p.major).toList(), rest = fx.params.where((p) => !p.major).toList();
    return _Panel(
      title: _fxName(fx, widget.lang),
      sub: 'Placed on title_card',
      h: widget.h,
      child: _Lv(children: [
        const _Sect('Looks', trailing: _Word('Undo', onTap: _noop)),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: Row(children: [
            for (var i = 0; i < looks.length; i++) ...[
              if (i > 0) const SizedBox(width: 4),
              Expanded(
                child: Hov(
                  onTap: () => _look(i),
                  builder: (_, h) => AnimatedContainer(
                    duration: Mo.dur,
                    curve: Mo.ease,
                    height: 44,
                    padding: const EdgeInsets.all(4),
                    decoration: BoxDecoration(color: look == i ? N.g20 : (h ? N.g26 : N.g13), borderRadius: BorderRadius.circular(5), border: Border.all(color: look == i ? Role.selected : _clear)),
                    child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                      Expanded(child: CustomPaint(painter: _LookThumb(i))),
                      const SizedBox(height: 4),
                      Center(child: Text(looks[i], style: T.label(look == i ? N.g95 : N.g76), maxLines: 1)),
                    ]),
                  ),
                ),
              ),
            ],
          ]),
        ),
        _gap(8),
        _Sect('Main', count: main.length),
        for (final p in main) _PRow(p, vals, key: ValueKey(p.id), lang: widget.lang, jp: fx.jpFx),
        if (rest.isNotEmpty) ...[
          _FoldRow(count: rest.length, open: more, onTap: () => setState(() => more = !more)),
          if (more) for (final p in rest) _PRow(p, vals, key: ValueKey(p.id), lang: widget.lang, jp: fx.jpFx),
        ],
      ]),
    );
  }
}

List<WidgetbookUseCase> _xCases() => [
      _story('X2 Try keep or leave', 'X2', 'Hover = peek, click = keep, Esc = leave: the same in every list of choices', _Habit.addition,
          'Contract: a peek writes nothing (the peeked item takes the accent underline, the heading shows it as "peek"); one click is one undo step; Esc or moving away restores. Shown on three kinds of choice (blend, ease, colour) to prove it is one grammar.',
          (c, h) => _PeekPanel(h: h)),
      _story('X3 Keyboard only', 'X3', 'Choose a row, change it, jump to it, reset it, without the mouse', _Habit.habit,
          'Contract: arrows choose and change (Shift x10, Alt x0.1), P S R T A jump (After Effects habit), Backspace resets; the selected row shows its keys with arrows either side, the tick dims when the panel loses the keyboard. Click the panel once to give it the keyboard.',
          (c, h) => _KeysPanel(h: h)),
      _story('X4 One grammar for numbers', 'X4', 'Drag, Shift, Alt, arrows and wheel mean the same thing on every number', _Habit.habit,
          'Contract (D-I3): right = up, one step per pixel, Shift x10, Alt x0.1, arrows and wheel the same, double-click resets, click types. The keycaps light and the step shows while you hold a modifier; the grip ticks on every cell say it drags; stacked and single fields obey the same rule.',
          (c, h) => _GrammarPanel(h: h)),
      _story('X5 Who an edit will change', 'X5', 'A band above the values names the target before you drag', _Habit.addition,
          'Contract: the targets are listed before you drag (this layer / selection / all keys): a tick = it changes, no tick and a ghost cell = left alone; the count sits in the heading. Knob picks the scope.',
          (c, h) => _ScopePanel(h: h, start: c.knobs.int.slider(label: 'Scope (0 layer, 1 selection, 2 keys)', initialValue: 1, min: 0, max: 2))),
      _story('X6 Dead ends say why and what next', 'X6', 'Nothing selected, no match, failed, locked, missing media', _Habit.habit,
          'Contract: every empty or blocked state has what, why, and the next step as words; red only for real failure. Knob switches the state, or shows all five.',
          (c, h) => _StatesPanel(h: h, state: c.knobs.object.dropdown<String>(label: 'State', options: const ['All five', 'No layer selected', 'No match', 'Effect failed', 'Locked layer', 'Missing media'], initialOption: 'All five', labelBuilder: (s) => s))),
      _story('X7 Docked widths one voice', 'X7', 'The same panel at 282, 224 and 176 px', _Habit.habit,
          'Contract: the same type, lines and emphasis at every width; below 220 px the label stacks over its value; nothing shrinks below 10 px or is cut off silently.',
          (c, h) => Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
                for (final w in const [282.0, 224.0, 176.0]) ...[
                  _ParamPanel(fx: _hue, h: h, w: w, lang: _lang(c), layer: w == 282 ? 'grain_overlay.mov' : 'grain_overlay', find: true, fold: true, presetOpen: true),
                  if (w != 176) const SizedBox(width: 12),
                ],
              ])),
      _story('X8 Placed, now adjust', 'X8', 'Right after placing from the Browser: looks, the main knobs, and the rest folded', _Habit.addition,
          'Contract: what people change first is on top; other starting points are one click; the long tail is folded with its count; Undo is right there. Knob picks the effect.',
          (c, h) {
            final k = c.knobs.object.dropdown<String>(label: 'Effect', options: const ['Glow', 'Fractal Noise', '色相/彩度'], initialOption: 'Glow', labelBuilder: (s) => s);
            return _TweakPanel(h: h, lang: _lang(c), fx: switch (k) { 'Glow' => _glow, 'Fractal Noise' => _fractal, _ => _hue });
          }),
    ];
