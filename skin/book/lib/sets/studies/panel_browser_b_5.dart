// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

class _DropLabState extends State<_DropLab> {
  static const _rh = 23.0, _voidH = 46.0;
  final List<_Ly> _ly = _seedLayers();
  final _hist = <List<_Ly>>[];
  Asset? _carry;
  int _hov = -2, _sel = 3, _face = -1, _half = -1;
  bool _alt = false;
  String? _msg;
  Timer? _timer;
  double _aw = 320;

  bool get _isAlt => _alt || widget.altSim;
  int get _n => _ly.length;

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  List<_Ly> _copy() => [for (final l in _ly) _Ly(l.name, l.src, l.keys)];
  int _rowAt(double y) => y < 0 ? -1 : (y < _n * _rh ? (y / _rh).floor() : _n);
  Rect _facesRect(int i) => Rect.fromLTWH(8, (i + 1) * _rh + 2, _aw - 16, 44);
  String _base(Asset a) => a.name.replaceAll(RegExp(r'\.[a-z0-9]+$'), '');

  void _do(int i, bool replace) {
    final a = _carry!;
    _hist.add(_copy());
    setState(() {
      if (replace) {
        final l = _ly[i];
        l.src = a.name;
        _msg = 'Replaced "${l.name}" · bar and ${l.keys} keys kept';
      } else {
        _ly.insert(i, _Ly(_base(a), a.name, 0));
        _sel = i;
        _msg = 'Added "${_base(a)}" above layer ${i + 1}';
      }
      _carry = null;
      _face = -1;
      _hov = -2;
    });
  }

  void _newTop() {
    final a = _carry!;
    _hist.add(_copy());
    setState(() {
      _ly.insert(0, _Ly(_base(a), a.name, 0));
      _sel = 0;
      _msg = 'New layer "${_base(a)}" on top';
      _carry = null;
      _hov = -2;
    });
  }

  void _hover(Offset p) {
    if (_carry == null) {
      final i = _rowAt(p.dy);
      if (i != _hov) setState(() => _hov = i);
      return;
    }
    if (widget.mode == 3 && _face >= 0 && _facesRect(_face).contains(p)) {
      setState(() => _half = p.dx < _aw / 2 ? 0 : 1);
      return;
    }
    final i = _rowAt(p.dy);
    if (i != _hov || _face >= 0) {
      _timer?.cancel();
      setState(() {
        _hov = i;
        _face = -1;
        _half = -1;
      });
      if (widget.mode == 3 && i >= 0 && i < _n) {
        _timer = Timer(Duration(milliseconds: widget.dwell), () {
          if (mounted && _hov == i) setState(() => _face = i);
        });
      }
    }
  }

  void _tap(Offset p) {
    final i = _rowAt(p.dy);
    if (_carry == null) {
      if (i >= 0 && i < _n) setState(() => _sel = i);
      return;
    }
    if (widget.mode == 3) {
      if (_face >= 0 && _facesRect(_face).contains(p)) {
        _do(_face, p.dx < _aw / 2);
      } else {
        setState(() {
          _carry = null;
          _face = -1;
          _msg = 'Cancelled';
        });
      }
      return;
    }
    if (i == _n) {
      _newTop();
    } else if (i >= 0) {
      switch (widget.mode) {
        case 0:
          _do(i, _isAlt);
        case 1:
          _do(i, !_isAlt);
        default:
          setState(() => _msg = 'ここには落とせません');
      }
    }
  }

  bool _key(KeyEvent e) {
    setState(() => _alt = HardwareKeyboard.instance.isAltPressed);
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _carry != null) {
      setState(() {
        _carry = null;
        _face = -1;
        _msg = 'Cancelled';
      });
      return true;
    }
    return false;
  }

  Widget _row(int i, bool showSrc) => _LayerRow(_ly[i], i, sel: i == _sel, hover: _carry == null && i == _hov, showSrc: showSrc, height: _rh);

  Widget _faceBox(String en, String jp, bool on) => Expanded(child: _Face(en, jp, on));

  Widget _overlay() {
    final h = _hov;
    final out = <Widget>[];
    if (_carry == null) return const SizedBox.shrink();
    if (h == _n) {
      out.add(Positioned(left: _px, right: _px, top: _n * _rh + 4, height: _voidH - 8, child: IgnorePointer(child: CustomPaint(painter: DashedBox(_accent), child: Center(child: Text('new layer on top', style: T.label(N.g95)))))));
    } else if (h >= 0 && h < _n) {
      final String? kind = switch (widget.mode) {
        0 => _isAlt ? 'replace' : 'add',
        1 => _isAlt ? 'add' : 'replace',
        2 => 'refuse',
        _ => _face < 0 ? 'dwell' : null,
      };
      if (kind == 'add') {
        out.add(Positioned(left: 4, right: 4, top: h * _rh - 1, height: 2, child: IgnorePointer(child: ColoredBox(color: _accent))));
        out.add(Positioned(right: _px, top: h * _rh + 3, child: const IgnorePointer(child: _Tag('ADD ABOVE', hi: true))));
      } else if (kind == 'replace') {
        out.add(Positioned(left: 1, right: 1, top: h * _rh, height: _rh, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(color: _accent.withValues(alpha: .12), border: Border.all(color: _accent))))));
        out.add(Positioned(right: _px, top: h * _rh + 3, child: const IgnorePointer(child: _Tag('REPLACE', hi: true))));
      } else if (kind == 'refuse') {
        out.add(Positioned(left: 0, right: 0, top: h * _rh, height: _rh, child: IgnorePointer(child: ColoredBox(color: N.g00.withValues(alpha: .55), child: Center(child: Text('ここには落とせません', style: T.label(N.g91)))))));
      } else if (kind == 'dwell') {
        out.add(Positioned(left: 1, right: 1, top: h * _rh, height: _rh, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: N.g56))))));
      }
      if (widget.mode == 3 && _face >= 0) {
        final r = _facesRect(_face);
        out.add(Positioned(left: r.left, top: r.top, width: r.width, height: r.height, child: IgnorePointer(child: Container(decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(7), border: Border.all(color: N.g26), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))]), child: Row(children: [_faceBox('Replace', '絵だけ差し替え', _half == 0), _faceBox('Add', '新規 layer', _half == 1)])))));
      }
    }
    return Stack(children: out);
  }

  @override
  Widget build(BuildContext context) {
    final m = widget.mode;
    final hints = switch (m) {
      0 => const [('drop', 'add'), ('Alt', 'replace'), ('Esc', 'cancel')],
      1 => const [('drop', 'replace'), ('Alt', 'add'), ('Esc', 'cancel')],
      2 => const [('drop', 'empty space or Source'), ('Esc', 'cancel')],
      _ => [('hold', '${(widget.dwell / 1000).toStringAsFixed(1)} s: two faces'), ('drop', 'on a face'), ('Esc', 'cancel')],
    };
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Follow(
          ghost: _carry == null ? null : _Ghost(_carry!.name, thumb: _Art(_carry!)),
          child: _Col([
            _Head([
              Row(children: [Expanded(child: _t('Layers · Intro v3', T.title())), _undoMark(_hist.length)]),
              const SizedBox(height: 8),
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Row(children: [Expanded(child: _standIn('Browser')), if (_carry != null) const _Tag('CARRYING', hi: true)]),
                  const SizedBox(height: 6),
                  Row(children: [
                    for (final a in _dropAssets)
                      Expanded(
                        child: _Hit(
                          onTap: () => setState(() => _carry = _carry == a ? null : a),
                          builder: (c, h, _) => Padding(padding: const EdgeInsets.only(right: 4), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                            AnimatedContainer(duration: Mo.dur, height: 34, foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: _carry == a ? _accent : (h ? N.g56 : N.g20))), child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Art(a))),
                            const SizedBox(height: 3),
                            _t(a.name, T.label(N.g76)),
                          ])),
                        ),
                      ),
                  ]),
                ]),
              ),
            ], bottom: 8),
            _Grow(
              child: LayoutBuilder(builder: (c, box) {
                _aw = box.maxWidth;
                final showSrc = box.maxWidth >= 300;
                return SingleChildScrollView(
                  physics: _carry == null ? null : const NeverScrollableScrollPhysics(),
                  child: MouseRegion(
                    onHover: (e) => _hover(e.localPosition),
                    onExit: (_) {
                      _timer?.cancel();
                      if (_hov != -2 || _face >= 0) {
                        setState(() {
                          _hov = -2;
                          _face = -1;
                        });
                      }
                    },
                    cursor: _carry == null ? MouseCursor.defer : (m == 2 && _hov >= 0 && _hov < _n ? SystemMouseCursors.forbidden : SystemMouseCursors.copy),
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTapUp: (d) => _tap(d.localPosition),
                      child: Stack(children: [
                        Column(children: [
                          for (var i = 0; i < _n; i++) _row(i, showSrc),
                          SizedBox(height: _voidH, child: Padding(padding: const EdgeInsets.symmetric(horizontal: _px, vertical: 4), child: _DropZone('New layer on top'))),
                        ]),
                        Positioned.fill(child: _overlay()),
                      ]),
                    ),
                  ),
                );
              }),
            ),
            if (m == 2)
              _Hit(
                onTap: () {
                  if (_carry != null) _do(_sel, true);
                },
                cursor: _carry == null ? SystemMouseCursors.basic : SystemMouseCursors.copy,
                builder: (c, h, _) => Container(
                  margin: const EdgeInsets.fromLTRB(_px, 8, _px, 8),
                  padding: const EdgeInsets.all(_px),
                  decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: _carry != null ? _accent : N.g20)),
                  child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Row(children: [Expanded(child: _t('INSPECTOR · LAYER ${_sel + 1}', T.micro(N.g76).copyWith(letterSpacing: .5))), if (_carry != null) const _Tag('DROP HERE', hi: true)]),
                    const SizedBox(height: 6),
                    Row(children: [Text('Source', style: T.label(N.g76)), const SizedBox(width: 10), Expanded(child: _t(_ly[_sel].src, T.value(N.g95)))]),
                  ]),
                ),
              ),
            _Foot(msg: _msg, hints: hints),
          ]),
        ),
      ),
    );
  }
}

// ================================================================ B7 click grammar: what click / double-click / Enter / drag mean on a shelf

class _MediaTile extends StatelessWidget {
  const _MediaTile(this.a, this.width, {this.sel = false, this.hover = false});
  final Asset a;
  final double width;
  final bool sel, hover;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: width,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          Stack(children: [
            AnimatedContainer(
              duration: Mo.dur,
              curve: Mo.ease,
              width: width,
              height: width * .62,
              foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: sel ? _accent : (hover ? N.g56 : N.g20))),
              child: ClipRRect(borderRadius: BorderRadius.circular(6), child: _Art(a)),
            ),
            if (a.kind != Kind.image) Positioned(left: 4, top: 4, child: TypeBadge(a.kind)),
            if (a.dur > 0) Positioned(right: 4, bottom: 4, child: TimePill(clock(a.dur))),
          ]),
          const SizedBox(height: 5),
          _Mid(a.name, T.name(sel ? N.g95 : N.g91)),
          const SizedBox(height: 3),
          _t(a.dims, T.label(N.g76)),
        ]),
      );
}

const _shelfIdx = [0, 2, 24, 25, 8, 7, 16, 17, 26, 31, 32, 34];
final List<_Fx> _shelf = [for (final i in _shelfIdx) _fxs[i]];
final List<Asset> _mediaShelf = [
  ...sampleAssets.take(7),
  const Asset('夕焼け_空撮_001.mov', Kind.video, 14, dur: 47, dims: '3840×2160'),
  const Asset('Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', Kind.video, 6, dur: 312),
  const Asset('背景_和紙テクスチャ.png', Kind.image, 21, dims: '4096×4096'),
];

/// The target line of B7-d: what a click will act on, in one line.
class _TargetLine extends StatelessWidget {
  const _TargetLine(this.sel);
  final _Sel sel;
  @override
  Widget build(BuildContext context) {
    final none = sel == _Sel.none;
    return Container(
      height: 30,
      padding: const EdgeInsets.symmetric(horizontal: _px),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: none ? N.g20 : N.g38)),
      child: Row(children: [
        Text('TARGET', style: T.micro(N.g76).copyWith(letterSpacing: .6)),
        const SizedBox(width: 8),
        Expanded(child: _t(_selName(sel), T.name(none ? N.g63 : N.g95))),
      ]),
    );
  }
}

class _GrammarLab extends StatefulWidget {
  const _GrammarLab({required this.w, required this.mode, this.target = _Sel.text, this.startTab = 0});
  final double w;
  final int mode; // 0 select + dbl/Enter use, 1 two shelves with two meanings (counter-example), 2 single click uses, 3 browser shows its target
  final _Sel target;
  final int startTab;
  @override
  State<_GrammarLab> createState() => _GrammarLabState();
}

class _GrammarLabState extends State<_GrammarLab> {
  late int _tab = widget.startTab;
  final _sel = [-1, -1];
  final _stack = <String>['Gaussian Blur', 'Glow'];
  int _layers = 14, _cols = 3;
  final _undo = <(int, String)>[]; // (0 = effect added, 1 = layer added, name)
  String? _msg;

  _Sel get _target => widget.mode >= 2 ? widget.target : _Sel.text;
  int get _count => _tab == 0 ? _shelf.length : _mediaShelf.length;
  String _name(int tab, int i) => tab == 0 ? _shelf[i].en : _mediaShelf[i].name;

  /// Whether a gesture on this shelf means "use". This is the whole point of the case: one rule everywhere, or two.
  bool _singleUses(int tab) => switch (widget.mode) {
        0 => false,
        1 => tab == 0,
        _ => true,
      };

  void _use(int tab, int i) {
    final n = _name(tab, i);
    if (widget.mode >= 2 && _target == _Sel.none) {
      setState(() => _msg = widget.mode == 2 ? 'Select a layer' : 'Select a layer first');
      return;
    }
    if (tab == 0 && widget.mode >= 2 && !_fits(_shelf[i], _target)) {
      setState(() => _msg = '$n needs a Layer');
      return;
    }
    setState(() {
      if (tab == 0) {
        if (_target == _Sel.many) {
          _msg = 'Applied $n to 5 layers';
          _undo.add((2, n));
        } else {
          _stack.add(n);
          _msg = 'Added $n to Layer 3';
          _undo.add((0, n));
        }
      } else {
        _layers++;
        _msg = 'Added "$n" as layer $_layers';
        _undo.add((1, n));
      }
    });
  }

  void _doUndo() {
    if (_undo.isEmpty) return;
    final (k, n) = _undo.removeLast();
    setState(() {
      if (k == 0) _stack.removeLast();
      if (k == 1) _layers--;
      _msg = 'Undone: $n';
    });
  }

  void _select(int tab, int i) => setState(() {
        _sel[tab] = i;
        _msg = widget.mode == 0 ? 'Peek: ${_name(tab, i)}' : 'Selected ${_name(tab, i)}';
      });

  void _click(int tab, int i) => _singleUses(tab) ? _use(tab, i) : _select(tab, i);

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return false;
    final k = e.logicalKey;
    final cur = _sel[_tab];
    int? to;
    if (k == LogicalKeyboardKey.arrowRight) to = cur + 1;
    if (k == LogicalKeyboardKey.arrowLeft) to = cur - 1;
    if (k == LogicalKeyboardKey.arrowDown) to = cur + _cols;
    if (k == LogicalKeyboardKey.arrowUp) to = cur - _cols;
    if (to != null) {
      _select(_tab, to.clamp(0, _count - 1));
      return true;
    }
    if (_isEnter(k) && cur >= 0) {
      _use(_tab, cur);
      return true;
    }
    if (k == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      _doUndo();
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final m = widget.mode;
    final hints = switch (m) {
      0 => const [('click', 'select · peek'), ('dbl / ↵', 'use'), ('drag', 'place'), ('⌘Z', 'undo')],
      1 => const [('click', 'effects: USE · media: select'), ('dbl', 'media: use')],
      2 => const [('click', 'use')],
      _ => const [('click / ↵', 'apply to the target only')],
    };
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            _Tabs(_tab + 2, onChanged: (i) {
              if (i >= 2) setState(() => _tab = i - 2);
            }),
            const SizedBox(height: 8),
            if (m == 3) ...[_TargetLine(_target), const SizedBox(height: 8)],
          ]),
          _Grow(
            child: _Fade(child: SingleChildScrollView(
              padding: const EdgeInsets.fromLTRB(_px, 0, _px, 12),
              child: LayoutBuilder(builder: (c, box) {
                _cols = math.max(2, ((box.maxWidth + 8) / 92).floor());
                return _Grid(
                  n: _count,
                  item: (i, w) => _Hit(
                    onTap: () => _click(_tab, i),
                    onDouble: _singleUses(_tab) ? null : () => _use(_tab, i),
                    builder: (c, h, _) => _tab == 0 ? _FxTile(_shelf[i], w, sel: _sel[0] == i, hover: h) : _MediaTile(_mediaShelf[i], w, sel: _sel[1] == i, hover: h),
                  ),
                );
              }),
            )),
          ),
          Container(
            padding: const EdgeInsets.fromLTRB(_px, 6, _px, 6),
            decoration: const BoxDecoration(color: N.g13, border: Border(top: BorderSide(color: N.g20))),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
              _Sec(_target == _Sel.many ? '5 layers' : 'Layer 3 · タイトル', n: '${_stack.length} effects', trailing: Text('comp $_layers layers', style: _num())),
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 66),
                child: ListView(shrinkWrap: true, padding: EdgeInsets.zero, children: [for (var i = _stack.length - 1; i >= 0; i--) SizedBox(height: 22, child: Row(children: [SizedBox(width: 20, child: Text('${i + 1}', style: _num())), Expanded(child: _t(_stack[i], T.name(i == _stack.length - 1 ? N.g95 : N.g76)))]))]),
              ),
            ]),
          ),
          _Foot(msg: _msg, hints: hints, trailing: m == 2 && _undo.isNotEmpty ? _Btn('Undo', onTap: _doUndo) : null),
        ]),
      ),
    );
  }
}
