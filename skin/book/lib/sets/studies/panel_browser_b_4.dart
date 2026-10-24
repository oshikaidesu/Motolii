// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

/// An effect card of the arrow-step panel: thumb, name, the family position, the two neighbours (when selected) and two parameter bars.
class _StepFace extends StatelessWidget {
  const _StepFace(this.f, this.v, {this.sel = false, this.hover = false, this.hoverUp = false, this.onStep});
  final _Fx f;
  final (double, double) v;
  final bool sel, hover, hoverUp;
  final ValueChanged<int>? onStep;
  @override
  Widget build(BuildContext context) {
    final fam = _fxs.where((x) => x.cat == f.cat).toList(), at = fam.indexWhere((x) => x.en == f.en), p = _params(f);
    Widget nb(String dir, _Fx? n, int d, bool pin) => Expanded(
          child: _Hit(
            onTap: n == null ? null : () => onStep?.call(d),
            cursor: n == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
            builder: (c, h, _) => AnimatedContainer(duration: Mo.dur, height: 22, padding: const EdgeInsets.symmetric(horizontal: 6), alignment: Alignment.centerLeft, decoration: BoxDecoration(color: n != null && (h || pin) ? N.g20 : N.g10, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)), child: Row(children: [Text(dir, style: T.value(N.g76)), const SizedBox(width: 5), Expanded(child: _t(n?.en ?? 'end of family', T.label(n == null ? N.g63 : N.g76)))])),
          ),
        );
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: sel ? N.g15 : (hover ? N.g13 : N.g10), borderRadius: BorderRadius.circular(6), border: Border.all(color: sel ? _accentLine : N.g20)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          SizedBox(width: 22, height: 16, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(f))),
          const SizedBox(width: 8),
          Expanded(child: _NameJp(f.en, f.jp, T.name(N.g95), T.label(N.g76), whole: true)),
          const SizedBox(width: 4),
          _Tag('${_catShort[f.cat]} ${at + 1}/${fam.length}'),
        ]),
        if (sel) ...[
          const SizedBox(height: 6),
          Row(children: [nb('↑', at > 0 ? fam[at - 1] : null, -1, hoverUp), const SizedBox(width: 4), nb('↓', at < fam.length - 1 ? fam[at + 1] : null, 1, false)]),
        ],
        const SizedBox(height: 6),
        _Param(p.$1, v.$1, hot: sel),
        const SizedBox(height: 4),
        _Param(p.$2, v.$2, hot: sel),
      ]),
    );
  }
}

class _CardStep extends StatefulWidget {
  const _CardStep({required this.w});
  final double w;
  @override
  State<_CardStep> createState() => _CardStepState();
}

class _CardStepState extends State<_CardStep> {
  late List<_Fx> _fx = [_fxs[0], _fxs[7], _fxs[24], _fxs[26], _fxs[35], _fxs[42]];
  final _stack = <(int, _Fx)>[];
  int _sel = 0;
  String? _msg;
  final _vals = [(.30, .55), (.25, .70), (.45, .20), (.50, .40), (.35, .60), (.55, .30)];

  List<_Fx> _family(int cat) => _fxs.where((f) => f.cat == cat).toList();

  void _step(int d, [int? card]) {
    final i = card ?? _sel;
    final fam = _family(_fx[i].cat);
    final at = fam.indexWhere((f) => f.en == _fx[i].en), to = (at + d).clamp(0, fam.length - 1);
    if (to == at) {
      setState(() => _msg = '${_catJp[_fx[i].cat]} の${d > 0 ? '最後' : '最初'}です。');
      return;
    }
    setState(() {
      _sel = i;
      _stack.add((i, _fx[i]));
      final from = _fx[i];
      _fx = [..._fx]..[i] = fam[to];
      _msg = '${from.en} → ${fam[to].en}';
    });
  }

  void _undo() {
    if (_stack.isEmpty) return;
    final (i, f) = _stack.last;
    setState(() {
      _stack.removeLast();
      _fx = [..._fx]..[i] = f;
      _msg = 'Undone: back to ${f.en}.';
    });
  }

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return false;
    if (e.logicalKey == LogicalKeyboardKey.arrowDown) {
      _step(1);
      return true;
    }
    if (e.logicalKey == LogicalKeyboardKey.arrowUp) {
      _step(-1);
      return true;
    }
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      _undo();
      return true;
    }
    return false;
  }

  Widget _card(int i) {
    final f = _fx[i], sel = i == _sel;
    return Listener(
      onPointerSignal: (s) {
        if (s is PointerScrollEvent && sel) _step(s.scrollDelta.dy > 0 ? 1 : -1);
      },
      child: _Hit(
        onTap: () => setState(() => _sel = i),
        cursor: SystemMouseCursors.basic,
        builder: (c, h, _) => _StepFace(f, _vals[i], sel: sel, hover: h, onStep: (d) => _step(d, i)),
      ),
    );
  }

  @override
  Widget build(BuildContext context) => _Frame(
        w: widget.w,
        child: _Keys(
          onKey: _key,
          child: _Col([
            _Head([
              _layerHead(),
              const SizedBox(height: 10),
              const IgnorePointer(child: Segmented(items: ['Transform', 'Effects', 'Material'], index: 1, expand: true)),
              const SizedBox(height: 10),
              _Sec('Effects', n: '${_fx.length}', trailing: _undoMark(_stack.length)),
            ], bottom: 0),
            _Grow(child: _Fade(child: ListView(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.fromLTRB(_px, 0, _px, 8), children: [for (var i = 0; i < _fx.length; i++) _card(i)]))),
            _Foot(msg: _msg, hints: const [('↑↓', 'same family'), ('wheel', 'on a card'), ('⌘Z', 'one step back')]),
          ]),
        ),
      );
}

// ================================================================ B9-d A/B: two frames, now and candidate

class _Ab extends StatefulWidget {
  const _Ab({required this.w, this.startOpen = true});
  final double w;
  final bool startOpen;
  @override
  State<_Ab> createState() => _AbState();
}

class _AbState extends State<_Ab> {
  _Fx _now = _layerFx[0];
  int _cand = -1, _undo = 0;
  String? _msg;
  final _sc = ScrollController();

  @override
  void initState() {
    super.initState();
    if (widget.startOpen) _cand = 10;
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  void _keep() {
    if (_cand < 0) return;
    setState(() {
      _now = _layerFx[_cand];
      _msg = 'Kept ${_now.en}';
      _cand = -1;
      _undo++;
    });
  }

  void _close() => setState(() {
        _cand = -1;
        _msg = 'Closed';
      });

  void _set(int i) {
    setState(() {
      _cand = i.clamp(0, _layerFx.length - 1);
      _msg = 'Candidate: ${_layerFx[_cand].en}';
    });
    if (_sc.hasClients) _sc.animateTo((_cand * 28.0 - 56).clamp(0.0, _sc.position.maxScrollExtent), duration: Mo.dur, curve: Mo.ease);
  }

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return false;
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.arrowDown) {
      _set(_cand + 1);
      return true;
    }
    if (k == LogicalKeyboardKey.arrowUp) {
      _set(_cand < 0 ? 0 : _cand - 1);
      return true;
    }
    if (_isEnter(k)) {
      _keep();
      return true;
    }
    if (k == LogicalKeyboardKey.escape && _cand >= 0) {
      _close();
      return true;
    }
    return false;
  }

  Widget _pane(String cap, _Fx fx, bool cand) => Expanded(
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          AspectRatio(aspectRatio: 16 / 10, child: DecoratedBox(position: DecorationPosition.foreground, decoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: cand ? _accent : N.g20)), child: _Shot(fx))),
          const SizedBox(height: 5),
          _t(cap, T.micro(cand ? N.g95 : N.g76).copyWith(letterSpacing: .5)),
          const SizedBox(height: 3),
          _Mid(fx.en, T.name(cand ? N.g95 : N.g76)),
        ]),
      );

  Widget _compare() => AnimatedSize(
        duration: Mo.dur,
        curve: Mo.ease,
        alignment: Alignment.topCenter,
        child: _cand < 0
            ? SizedBox(height: 52, child: Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Row(children: [Expanded(child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Shot(_now))), const SizedBox(width: 8), Expanded(child: CustomPaint(painter: DashedBox(N.g38), child: const SizedBox.expand()))])))
            : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                const SizedBox(height: 4),
                Row(crossAxisAlignment: CrossAxisAlignment.start, children: [_pane('NOW', _now, false), const SizedBox(width: 8), _pane('CANDIDATE', _layerFx[_cand], true)]),
                const SizedBox(height: 8),
                Row(children: [_Btn('Keep', primary: true, onTap: _keep), const SizedBox(width: 4), const _Kbd('↵'), const SizedBox(width: 10), _Btn('Close', onTap: _close), const SizedBox(width: 4), const _Kbd('Esc')]),
              ]),
      );

  @override
  Widget build(BuildContext context) => _Frame(
        w: widget.w,
        accent: _cand >= 0,
        child: _Keys(
          onKey: _key,
          child: _Col([
            _Head([
              const IgnorePointer(child: _Tabs(2)),
              const SizedBox(height: 8),
              _Sec('Compare · Layer 3, Effect 2', trailing: _cand >= 0 ? const _Tag('PREVIEW · not written', hi: true) : _undoMark(_undo)),
              _compare(),
              const SizedBox(height: 6),
              _Sec('Candidates', n: '${_layerFx.length}'),
            ]),
            _Grow(
              child: _Fade(child: ListView.builder(
                controller: _sc,
                shrinkWrap: _PanelH.fitOf(context),
                padding: const EdgeInsets.only(bottom: 12),
                itemExtent: 28,
                itemCount: _layerFx.length,
                itemBuilder: (c, i) {
                  final f = _layerFx[i];
                  return _Hit(onTap: () => _set(i), onDouble: () {
                    _set(i);
                    _keep();
                  }, builder: (c, h, _) => _FxRow(f, sel: i == _cand, hover: h, whole: true, tags: [if (f.en == _now.en) const _Tag('NOW')]));
                },
              )),
            ),
            _Foot(msg: _msg, hints: const [('click', 'compare'), ('↑↓', 'next')]),
          ]),
        ),
      );
}

// ================================================================ B8 replace vs add: dropping an asset on a layer (the Timeline's layer column)

class _Ly {
  _Ly(this.name, this.src, this.keys);
  String name, src;
  int keys;
}

List<_Ly> _seedLayers() => [
      _Ly('タイトル / Title', 'Text', 5),
      _Ly('Subtitle サブタイトル', 'Text', 3),
      _Ly('BG Gradient', 'Solid', 0),
      _Ly('Logo Mark', 'logo_mark.png', 4),
      _Ly('City B-roll', 'city_pass.mov', 2),
      _Ly('Lens Bloom', 'lens_bloom.mp4', 6),
      _Ly('Particles 01', 'Shape', 8),
      _Ly('Particles 02', 'Shape', 8),
      _Ly('Light Leak', 'light_leak_03.mov', 1),
      _Ly('Caption JP 字幕(日本語)', 'Text', 0),
      _Ly('Adjustment 調整レイヤー', '—', 2),
      _Ly('Interview_Tanaka-san_Cam-B_take-07_final_v2', 'interview_b_07.mov', 0),
      _Ly('Camera 1', 'Camera', 12),
      _Ly('Null: Rig Controller', 'Null', 4),
    ];

/// One layer line of the B8 panel: index, name, source, key count. Bands alternate; the selected line is g20 with the gutter tick.
class _LayerRow extends StatelessWidget {
  const _LayerRow(this.l, this.i, {this.sel = false, this.hover = false, this.showSrc = true, this.height = 23});
  final _Ly l;
  final int i;
  final bool sel, hover, showSrc;
  final double height;
  @override
  Widget build(BuildContext context) => _Line(
        height: height,
        sel: sel,
        hover: hover,
        band: i.isEven ? N.bandA : N.bandB,
        line: true,
        tick: 13,
        child: Row(children: [
          SizedBox(width: 22, child: Text('${i + 1}', style: _num())),
          Expanded(child: _t(l.name, T.name(sel ? N.g95 : N.g91))),
          if (showSrc) SizedBox(width: 104, child: Padding(padding: const EdgeInsets.only(left: 8), child: _t(l.src, _num()))),
          SizedBox(width: 44, child: Text(l.keys == 0 ? '—' : '${l.keys} keys', textAlign: TextAlign.right, maxLines: 1, style: T.label(N.g76))),
        ]),
      );
}

/// One of the two faces of B8-d (Replace | Add). Lit = the pointer is over it.
class _Face extends StatelessWidget {
  const _Face(this.en, this.jp, this.on);
  final String en, jp;
  final bool on;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        margin: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: on ? N.g26 : N.g20, borderRadius: BorderRadius.circular(5), border: Border.all(color: on ? _accent : N.g44)),
        child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [Text(en.toUpperCase(), style: T.micro(N.g95)), const SizedBox(height: 4), _t(jp, T.label(N.g76))]),
      );
}

const _dropAssets = [
  Asset('lens_bloom.mp4', Kind.video, 7, dur: 12),
  Asset('夕焼け_空撮.mov', Kind.video, 14, dur: 47),
  Asset('logo_mark.png', Kind.image, 9),
  Asset('dusk_ridge.jpg', Kind.image, 3),
];

/// The dashed drop target (B8 new-layer area, B11 save area): lit when something is carried over it.
class _DropZone extends StatelessWidget {
  const _DropZone(this.text, {this.on = false});
  final String text;
  final bool on;
  @override
  Widget build(BuildContext context) => CustomPaint(painter: DashedBox(on ? _accent : N.g38), child: Center(child: Text(text, style: T.label(on ? N.g95 : N.g76))));
}

class _DropLab extends StatefulWidget {
  const _DropLab({required this.w, required this.mode, this.dwell = 1000, this.altSim = false});
  final double w;
  final int mode; // 0 add by default (Alt replaces), 1 replace by default (Alt adds), 2 layers refuse drops, 3 two faces after a dwell
  final int dwell;
  final bool altSim;
  @override
  State<_DropLab> createState() => _DropLabState();
}
