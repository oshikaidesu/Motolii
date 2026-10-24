// Part of panel_inspector_b_x.dart (cut there by size, as it was).
part of 'panel_inspector_b.dart';

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
