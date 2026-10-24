// Part of panel_inspector_a_ease.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

class _EaseAState extends State<_EaseA> {
  _E _applied = _presets[28];
  int? _peek;
  int _steps = 0;
  String _log = '';
  final _fn = FocusNode(debugLabel: 'ease-list');
  List<double>? _before;

  @override
  void dispose() {
    _fn.dispose();
    super.dispose();
  }

  void _apply(int i) => setState(() {
        _applied = _presets[i];
        _steps++;
        _log = 'wrote ${_presets[i].name} to the interval';
        _peek = null;
      });

  @override
  Widget build(BuildContext context) {
    final dock = widget.w < 250, w = widget.w - 2 * _gutter - 2;
    final peekE = _peek == null ? null : _presets[_peek!];
    final custom = _applied.name == 'custom';
    return Focus(
      focusNode: _fn,
      onKeyEvent: (_, e) {
        if (e is! KeyDownEvent) {
          return KeyEventResult.ignored;
        }
        if (e.logicalKey == LogicalKeyboardKey.arrowDown || e.logicalKey == LogicalKeyboardKey.arrowUp) {
          final d = e.logicalKey == LogicalKeyboardKey.arrowDown ? 1 : -1;
          setState(() => _peek = ((_peek ?? (d > 0 ? -1 : 0)) + d).clamp(0, _presets.length - 1));
          return KeyEventResult.handled;
        }
        if (e.logicalKey == LogicalKeyboardKey.enter && _peek != null) {
          _apply(_peek!);
          return KeyEventResult.handled;
        }
        if (e.logicalKey == LogicalKeyboardKey.escape) {
          setState(() {
            _peek = null;
            _log = 'Esc: peek removed, nothing written';
          });
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: _Frame(
        w: widget.w,
        head: const _Head(name: 'Position X', kind: 'Key interval'),
        foot: _FootBar(_log, steps: _steps),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 8),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _interval(w),
              const SizedBox(height: 8),
              Align(alignment: Alignment.centerLeft, child: _Plot(
                applied: _applied,
                peek: peekE,
                w: w,
                h: dock ? 120 : 148,
                onBez: (b) => setState(() {
                  _before ??= _applied.bez;
                  _applied = _E('custom', 'カスタム', _lin, bez: b);
                }),
                onEnd: () => setState(() {
                  _steps++;
                  _log = 'handle released: 1 undo step';
                  _before = null;
                }),
              )),
              const SizedBox(height: 8),
              Text(peekE != null ? peekE.name : (custom ? 'Custom' : _applied.name), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(peekE != null ? N.g76 : N.g95).copyWith(decoration: peekE != null ? TextDecoration.underline : TextDecoration.none, decorationStyle: TextDecorationStyle.dotted, decorationColor: N.g63)),
              const SizedBox(height: 4),
              Text(peekE != null ? peekE.nums : _applied.nums, maxLines: 1, style: T.value(N.g76)),
              const SizedBox(height: 8),
              _track(_applied, peek: peekE, w: w),
            ]),
          ),
          Container(height: 1, color: _rule),
          Padding(
            padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 4),
            child: Row(children: [
              Expanded(child: Text('PRESETS  プリセット', style: T.micro(N.g76).copyWith(letterSpacing: 1))),
              Text('${_presets.length}', style: T.label(N.g76)),
            ]),
          ),
          Expanded(
            child: MouseRegion(
              onExit: (_) => setState(() => _peek = null),
              child: ShaderMask(
                // the last visible row fades out instead of ending mid-height: the list goes on
                blendMode: BlendMode.dstIn,
                shaderCallback: (r) => LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: const [Color(0xFFFFFFFF), Color(0xFFFFFFFF), Color(0x00FFFFFF)], stops: [0, math.max(0, (r.height - 20) / r.height), 1]).createShader(r),
                child: ListView.builder(
                padding: const EdgeInsets.fromLTRB(_gutter, 0, _gutter, 16),
                itemCount: _presets.length,
                itemExtent: 28,
                itemBuilder: (_, i) {
                  final e = _presets[i], on = _applied.name == e.name, pk = _peek == i;
                  return MouseRegion(
                    onEnter: (_) => setState(() => _peek = i),
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: () {
                        _fn.requestFocus();
                        _apply(i);
                      },
                      child: _PresetRow(e, on: on, peek: pk, dock: dock, gloss: i == 0 || _presets[i - 1].jp != e.jp),
                    ),
                  );
                },
              ),
              ),
            ),
          ),
        ]),
      ),
    );
  }
}

// ---- I5-b: a direction and one strength ----

class _EaseB extends StatefulWidget {
  const _EaseB();
  @override
  State<_EaseB> createState() => _EaseBState();
}

class _EaseBState extends State<_EaseB> {
  int _dir = 1; // 0 in, 1 out, 2 both
  double _s = .55;
  int _steps = 0;
  @override
  Widget build(BuildContext context) {
    final infl = _s * .9;
    final bez = switch (_dir) { 0 => [infl, 0.0, 1.0, 1.0], 1 => [0.0, 0.0, 1 - infl, 1.0], _ => [infl, 0.0, 1 - infl, 1.0] };
    final e = _E('custom', '', _lin, bez: bez);
    final outInfl = _dir == 1 ? 0 : (infl * 100).round(), inInfl = _dir == 0 ? 0 : (infl * 100).round();
    const w = _panelW - 2 * _gutter - 2;
    return _Frame(
      w: _panelW,
      head: const _Head(name: 'Position X', kind: 'Key interval'),
      foot: _FootBar(_steps == 0 ? '' : const ['Ease in', 'Ease out', 'Both'][_dir], steps: _steps),
      child: _Body(children: [
        _interval(w),
        _gap(12),
        _Plot(applied: e, w: w, h: 148),
        _gap(12),
        Segmented(items: const ['Ease in', 'Ease out', 'Both'], index: _dir, expand: true, onChanged: (i) => setState(() {
              _dir = i;
              _steps++;
            })),
        _gap(8),
        _Slide(label: 'Strength', value: _s, onChanged: (v) => setState(() => _s = _round(v))),
        _gap(8),
        Row(children: [
          Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Text('Out', style: T.label(N.g76)), _gap(4), Text('$outInfl %', style: T.value(N.g95))])),
          Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [Text('In', style: T.label(N.g76)), _gap(4), Text('$inInfl %', style: T.value(N.g95))])),
        ]),
        _gap(12),
        _track(e, w: w),
        _Sec('Keys  キー', [_keysList(w)]),
      ]),
    );
  }
}

// ---- I5-c: a dictionary of motions, each tile moves ----

class _EaseC extends StatefulWidget {
  const _EaseC({this.w = _panelW});
  final double w;
  @override
  State<_EaseC> createState() => _EaseCState();
}

class _EaseCState extends State<_EaseC> {
  int _sel = 0;
  int _steps = 0;
  @override
  Widget build(BuildContext context) {
    final dock = widget.w < 250, w = widget.w - 2 * _gutter - 2, cols = dock ? 1 : 2, tileW = (w - (cols - 1) * 8) / cols;
    return _Frame(
      w: widget.w,
      head: const _Head(name: 'Position X', kind: 'Key interval'),
      foot: _FootBar(_steps == 0 ? '' : _words[_sel].name, steps: _steps),
      child: _Body(children: [
        _interval(w),
        _Sec('Motion words  動きの言葉', [
          Wrap(spacing: 8, runSpacing: 8, children: [
            for (final (i, e) in _words.indexed)
              _Ctl(
                onTap: () => setState(() {
                  _sel = i;
                  _steps++;
                }),
                builder: (_, h, f, p) => _Tile(e, w: tileW, sel: _sel == i, hot: h, focus: f, press: p),
              ),
          ]),
        ]),
        _Sec('Keys  キー', [_keysList(w)]),
      ]),
    );
  }
}

// ---- I5-d: measure once, paste to many ----

class _Iv {
  const _Iv(this.prop, this.from, this.to, this.e);
  final String prop, from, to;
  final _E e;
}

class _EaseD extends StatefulWidget {
  const _EaseD({this.w = _panelW});
  final double w;
  @override
  State<_EaseD> createState() => _EaseDState();
}

class _EaseDState extends State<_EaseD> {
  late List<_Iv> _iv = _intervals();

  static List<_Iv> _intervals() {
    const props = ['Position X', 'Position Y', 'Scale', 'Rotation', 'Opacity', 'Glow Radius', 'Blurriness', 'Wave Height', 'Amount'];
    const times = ['00:00', '00:12', '01:00', '02:00'];
    const pick = [28, 1, 0, 0, 3, 0, 22, 4, 0, 13, 5, 0];
    return [
      for (final (i, p) in props.indexed)
        for (var k = 0; k < 3; k++)
          if ((i + k) % 5 != 4) _Iv(p, times[k], times[k + 1], _presets[pick[(i * 3 + k) % pick.length]]),
    ];
  }
  final Set<int> _sel = {0};
  _E? _clip;
  int _steps = 0;
  String _log = '';

  void _tap(int i) {
    setState(() {
      if (HardwareKeyboard.instance.isShiftPressed || HardwareKeyboard.instance.isMetaPressed) {
        _sel.contains(i) ? _sel.remove(i) : _sel.add(i);
      } else {
        _sel
          ..clear()
          ..add(i);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final dock = widget.w < 250;
    final canCopy = _sel.length == 1, canPaste = _clip != null && _sel.isNotEmpty;
    return _Frame(
      w: widget.w,
      head: const _Head(name: 'Jewel Field', kind: 'Shape group', sub: 'Key intervals'),
      foot: _FootBar(_log, steps: _steps),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _Clip(_clip),
        Padding(
          padding: const EdgeInsets.fromLTRB(_gutter, 8, _gutter, 8),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Row(children: [
              Expanded(child: _Act('Copy ease', on: canCopy, onTap: () => setState(() {
                    _clip = _iv[_sel.first].e;
                    _log = 'copied ${_clip!.name}';
                  }))),
              const SizedBox(width: 8),
              Expanded(child: _Act(dock ? 'Paste ${_sel.length}' : 'Paste to ${_sel.length}', on: canPaste, onTap: () => setState(() {
                    _iv = [for (final (i, v) in _iv.indexed) _sel.contains(i) ? _Iv(v.prop, v.from, v.to, _clip!) : v];
                    _steps++;
                    _log = 'pasted ${_clip!.name} to ${_sel.length} intervals: 1 undo step';
                  }))),
            ]),
          ]),
        ),
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.fromLTRB(_gutter, 0, _gutter, 16),
            itemCount: _iv.length,
            itemExtent: 32,
            itemBuilder: (_, i) {
              final v = _iv[i], on = _sel.contains(i);
              return _Ctl(onTap: () => _tap(i), builder: (_, h, f, p) => _IvRow(v, on: on, hot: h, focus: f, press: p, dock: dock));
            },
          ),
        ),
      ]),
    );
  }
}

/// The clipboard of I5-d, as a chip: a dashed empty slot, or the copied ease (thumb, name, numbers).
class _Clip extends StatelessWidget {
  const _Clip(this.e);
  final _E? e;
  @override
  Widget build(BuildContext context) => Container(
        height: 32,
        padding: const EdgeInsets.symmetric(horizontal: _gutter),
        decoration: BoxDecoration(color: N.g13, border: Border(bottom: BorderSide(color: _rule))),
        child: Row(children: [
          SizedBox(width: 64, child: Text('CLIPBOARD', maxLines: 1, style: T.micro(N.g76).copyWith(letterSpacing: .3))),
          if (e == null)
            const CustomPaint(size: Size(36, 20), painter: _SlotPaint())
          else ...[
            _thumb(e!, w: 36, h: 20, on: true),
            const SizedBox(width: 8),
            Expanded(child: Text(e!.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g91))),
            Text(e!.nums, maxLines: 1, style: T.value(N.g76)),
          ],
        ]),
      );
}

/// An empty slot: a dashed 1 px outline.
class _SlotPaint extends CustomPainter {
  const _SlotPaint();
  @override
  void paint(Canvas c, Size z) => _dashed(c, Path()..addRRect(RRect.fromRectAndRadius((Offset.zero & z).deflate(.5), const Radius.circular(3))), Paint()..color = N.g44..style = PaintingStyle.stroke);

  @override
  bool shouldRepaint(_SlotPaint o) => false;
}

/// A word button (Copy ease, Paste to n, Show all rows): rest, hover, focus, pressed, off. See [_Btn].
class _Act extends StatelessWidget {
  const _Act(this.text, {required this.on, required this.onTap, this.hot = false, this.focus = false, this.pressed = false});
  final String text;
  final bool on, hot, focus, pressed; // hot / focus / pressed pin a state (Parts)
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Ctl(
        enabled: on,
        onTap: onTap,
        hover: hot ? true : null,
        focus: focus ? true : null,
        pressed: pressed ? true : null,
        builder: (_, h, f, p) => _Btn(text, on: on, hover: h, focus: f, pressed: p),
      );
}

/// One row of the preset list: a thumb, the name, the Japanese word. Selected = g20 + tick; peeked (hover or the arrow keys) = g15; pressed = g26. A list row has no focus of its own: the peek is the keyboard cursor.
class _PresetRow extends StatelessWidget {
  const _PresetRow(this.e, {this.on = false, this.peek = false, this.press = false, this.dock = false, this.gloss = true});
  final _E e;
  final bool on, peek, press, dock;

  /// The Japanese word shows once per run of rows that share it (the same word on every row is no information).
  final bool gloss;
  @override
  Widget build(BuildContext context) => _Bleed(
        height: 28,
        fill: press ? N.g26 : (on ? N.g20 : (peek ? N.g15 : null)),
        tick: on,
        child: Row(children: [
          _thumb(e, w: 36, h: 20, on: on || peek || press),
          const SizedBox(width: 8),
          Expanded(child: Text(e.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g91))),
          if (!dock && gloss) ...[const SizedBox(width: 8), ConstrainedBox(constraints: const BoxConstraints(maxWidth: 88), child: Text(e.jp, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)))],
        ]),
      );
}

/// One interval in the copy list (I5-d): property, times, thumb, ease name. Selected = g20 + tick; hover g15; focus 1 px g76 edge inside the row; pressed g26.
class _IvRow extends StatelessWidget {
  const _IvRow(this.v, {this.on = false, this.hot = false, this.focus = false, this.press = false, this.dock = false});
  final _Iv v;
  final bool on, hot, focus, press, dock;
  @override
  Widget build(BuildContext context) => _Bleed(
        height: 32,
        fill: press ? N.g26 : (on ? N.g20 : (hot ? N.g15 : null)),
        tick: on,
        child: Container(
          foregroundDecoration: focus ? BoxDecoration(border: Border.all(color: N.g76)) : null,
          child: Row(children: [
            SizedBox(width: dock ? 64 : 76, child: Text(v.prop, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g95))),
            if (!dock) SizedBox(width: 88, child: Text('${v.from} → ${v.to}', maxLines: 1, style: T.label(N.g76))),
            _thumb(v.e, w: 36, h: 20, on: on),
            const SizedBox(width: 8),
            Expanded(child: Text(v.e.name, maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.right, style: T.label(on ? N.g95 : N.g76))),
          ]),
        ),
      );
}

/// One motion word as a tile (I5-c). rest g13 + g20 edge; hover g44 edge; focus g95 edge; pressed g07 fill + g63 edge; selected g20 fill + accent edge. [t] is the moment of the loop.
class _Tile extends StatelessWidget {
  const _Tile(this.e, {required this.w, this.sel = false, this.hot = false, this.focus = false, this.press = false, this.t});
  final _E e;
  final double w;
  final bool sel, hot, focus, press;
  final double? t; // null = the running loop
  @override
  Widget build(BuildContext context) => Container(
        width: w,
        height: 80,
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: sel ? N.g20 : (press ? N.g07 : N.g13),
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: sel ? Role.selected : (focus ? N.g95 : (press ? N.g63 : (hot ? N.g44 : N.g20)))),
        ),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text(e.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title(N.g95)),
          const SizedBox(height: 4),
          Text(e.jp, maxLines: 2, overflow: TextOverflow.ellipsis, style: T.label(N.g76).copyWith(height: 1.2)),
          const Spacer(),
          t == null ? _Loop(builder: (_, t) => CustomPaint(size: Size(w - 18, 14), painter: _TrackPaint(t, e, null))) : CustomPaint(size: Size(w - 18, 14), painter: _TrackPaint(t!, e, null)),
        ]),
      );
}
