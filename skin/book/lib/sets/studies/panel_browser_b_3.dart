// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

/// The foot fade only where the list really is cut: it reads the scroll metrics, so a list that fits shows no fade and a cut last row always has one.
class _Fade extends StatefulWidget {
  const _Fade({required this.child});
  final Widget child;
  @override
  State<_Fade> createState() => _FadeState();
}

class _FadeState extends State<_Fade> {
  bool _more = false;
  bool _note(Notification n) {
    final m = n is ScrollNotification ? n.metrics : (n is ScrollMetricsNotification ? n.metrics : null);
    if (m != null && m.axis == Axis.vertical) {
      final v = m.maxScrollExtent - m.pixels > 1;
      if (v != _more) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && v != _more) setState(() => _more = v);
        });
      }
    }
    return false;
  }

  @override
  Widget build(BuildContext context) => NotificationListener<Notification>(
        onNotification: _note,
        child: Stack(children: [
          widget.child,
          if (_more) Positioned(left: 0, right: 0, bottom: 0, height: 20, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, stops: const [0, .75, 1], colors: [N.g10.withValues(alpha: 0), N.g10.withValues(alpha: .85), N.g10])))))
        ]),
      );
}

/// A strip that is cut at its right edge: a 20 px fade into the panel ground (a half item is a scroll cue, not a cut word).
Widget _fadeSide(Widget child) => Stack(children: [
      child,
      Positioned(top: 0, bottom: 0, right: 0, width: 32, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.centerLeft, end: Alignment.centerRight, stops: const [0, .7, 1], colors: [N.g10.withValues(alpha: 0), N.g10.withValues(alpha: .9), N.g10])))))
    ]);

/// An empty slot: a dashed outline with the one-word state in it (the outline says "a place that fills"), a reason line only when there is a fact to give, and the way out as a button.
Widget _empty(String title, String hint, {Widget? action, Color? tone}) => Padding(
      padding: const EdgeInsets.fromLTRB(_px, 12, _px, 12),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        if (title.isNotEmpty) SizedBox(height: 56, child: CustomPaint(painter: DashedBox(tone != null && !Role.grey ? tone : N.g38), child: Center(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: Row(mainAxisSize: MainAxisSize.min, children: [if (tone != null) ErrMark(gap: 6, color: tone), Flexible(child: Text(title, textAlign: TextAlign.center, maxLines: 2, overflow: TextOverflow.ellipsis, style: T.name(N.g76)))]))))),
        if (hint.isNotEmpty) ...[const SizedBox(height: 8), Text(hint, textAlign: TextAlign.center, style: T.label(N.g63).copyWith(height: 1.35))],
        if (action != null) ...[const SizedBox(height: 10), Center(child: action)],
      ]),
    );

// ================================================================ B9-a Hot-swap: link mode (Q)

class _HotSwap extends StatefulWidget {
  const _HotSwap({required this.w, this.mode = 0, this.startLinked = false});
  final double w;
  final int mode; // 0 normal, 1 no effect on the layer yet, 2 layer locked
  final bool startLinked;
  @override
  State<_HotSwap> createState() => _HotSwapState();
}

class _HotSwapState extends State<_HotSwap> {
  bool _fam = false, _linked = false;
  int _cur = -1, _undo = 0;
  _Fx? _before, _now;
  String? _msg;
  final _sc = ScrollController();

  List<_Fx> get _vis => _fam ? _layerFx.where((f) => f.cat == 0).toList() : _layerFx;
  bool get _locked => widget.mode == 2;
  bool get _can => !_locked && _now != null;

  @override
  void initState() {
    super.initState();
    if (widget.mode == 0) _before = _now = _layerFx[0];
    if (widget.startLinked && _can) {
      _linked = true;
      _cur = 2;
      _now = _vis[2];
      _msg = 'Linked · Effect 2';
    }
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  void _scroll(int i) {
    if (_sc.hasClients) _sc.animateTo((i * 28.0 - 84).clamp(0.0, _sc.position.maxScrollExtent), duration: Mo.dur, curve: Mo.ease);
  }

  void _link() {
    if (_locked) {
      setState(() => _msg = 'Layer 3 is locked.');
      return;
    }
    if (_now == null) {
      setState(() => _msg = 'No effect on Layer 3.');
      return;
    }
    setState(() {
      _linked = true;
      _cur = math.max(0, _vis.indexOf(_before!));
      _msg = 'Linked · Effect 2';
    });
  }

  void _commit() => setState(() {
        _linked = false;
        _before = _now;
        _undo++;
        _msg = 'Kept ${_now!.en}';
      });

  void _cancel() => setState(() {
        _linked = false;
        _now = _before;
        _msg = 'Reverted to ${_before!.en}';
      });

  void _pick(int i) {
    setState(() {
      _cur = i.clamp(0, _vis.length - 1);
      if (_linked) {
        _now = _vis[_cur];
        _msg = 'Trying ${_now!.en}';
      }
    });
    _scroll(_cur);
  }

  void _add(int i) => setState(() {
        _before = _now = _vis[i];
        _undo++;
        _msg = 'Added ${_now!.en} to Layer 3';
      });

  bool _key(KeyEvent e) {
    if (e is KeyUpEvent) return false;
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.keyQ) {
      _linked ? _cancel() : _link();
      return true;
    }
    if (k == LogicalKeyboardKey.arrowDown) {
      _pick(_cur + 1);
      return true;
    }
    if (k == LogicalKeyboardKey.arrowUp) {
      _pick(_cur <= 0 ? 0 : _cur - 1);
      return true;
    }
    if (_isEnter(k) && _linked) {
      _commit();
      return true;
    }
    if (k == LogicalKeyboardKey.escape && _linked) {
      _cancel();
      return true;
    }
    return false;
  }

  Widget _target(double iw) {
    final shotW = (iw - 22 - 8) / 2;
    final Widget body;
    if (_locked) {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [Expanded(child: _t('Layer 3', T.title())), const _Tag('LOCKED · ロック中', hi: true)]),
        const SizedBox(height: 10),
        const _Btn('Link'),
      ]);
    } else if (_now == null) {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [Expanded(child: _t('Layer 3', T.title())), const _Tag('NO EFFECT · なし')]),
        const SizedBox(height: 10),
        const _Btn('Link'),
      ]);
    } else if (!_linked) {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text.rich(TextSpan(children: [TextSpan(text: _now!.en, style: T.title()), TextSpan(text: '  ${_now!.jp}', style: T.label(N.g76))]), maxLines: 1, overflow: TextOverflow.ellipsis),
        const SizedBox(height: 4),
        Text('2 / 3', style: _num()),
        const SizedBox(height: 10),
        Row(children: [_Btn('Link', onTap: _link), const SizedBox(width: 6), const _Kbd('Q')]),
      ]);
    } else {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        _t('LINKED · not final', T.micro(N.g95)),
        const SizedBox(height: 6),
        SizedBox(height: 40, child: Row(children: [
          SizedBox(width: shotW, child: _Shot(_before)),
          const SizedBox(width: 8),
          SizedBox(width: shotW, child: DecoratedBox(position: DecorationPosition.foreground, decoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: _accent)), child: _Shot(_now))),
        ])),
        const SizedBox(height: 4),
        Row(children: [
          SizedBox(width: shotW, child: _t('BEFORE · ${_before!.en}', T.label(N.g76))),
          const SizedBox(width: 8),
          SizedBox(width: shotW, child: _t('NOW · ${_now!.en}', T.label(N.g95))),
        ]),
        const SizedBox(height: 10),
        Row(children: [_Btn('Keep', primary: true, onTap: _commit), const SizedBox(width: 4), const _Kbd('↵'), const SizedBox(width: 10), _Btn('Revert', onTap: _cancel), const SizedBox(width: 4), const _Kbd('Esc')]),
      ]);
    }
    return AnimatedSize(
      duration: Mo.dur,
      curve: Mo.ease,
      alignment: Alignment.topCenter,
      child: Container(padding: const EdgeInsets.all(_px), decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: _linked ? _accentLine : N.g20)), child: body),
    );
  }

  @override
  Widget build(BuildContext context) {
    final vis = _vis;
    return _Frame(
      w: widget.w,
      accent: _linked,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 10),
            LayoutBuilder(builder: (c, box) => _target(box.maxWidth)),
            const SizedBox(height: 10),
            Row(children: [
              _Chip('All', n: _layerFx.length, on: !_fam, onTap: () => setState(() => _fam = false)),
              const SizedBox(width: 6),
              _Chip('Blur family', n: 7, on: _fam, onTap: () => setState(() => _fam = true)),
              Expanded(child: Align(alignment: Alignment.centerRight, child: _undoMark(_undo))),
            ]),
            const SizedBox(height: 6),
          ]),
          _Grow(
            child: ListView.builder(
              controller: _sc,
              shrinkWrap: _PanelH.fitOf(context),
              padding: const EdgeInsets.only(bottom: 4),
              itemExtent: 28,
              itemCount: vis.length,
              itemBuilder: (c, i) {
                final f = vis[i];
                final isBefore = _before != null && f.en == _before!.en;
                return _Hit(
                  onTap: () => _pick(i),
                  onDouble: () => _linked ? _commit() : (_now == null && !_locked ? _add(i) : null),
                  builder: (c, h, _) => _FxRow(f, sel: i == _cur, hover: h, tags: [if (isBefore && !_linked) const _Tag('ON LAYER')]),
                );
              },
            ),
          ),
          _Foot(msg: _msg, hints: const [('↑↓', 'next / prev')]),
        ]),
      ),
    );
  }
}

/// An effect card of the Inspector's Effects region: thumb, name, two parameter bars. `replace` = a carried tile is over it.
class _CardFace extends StatelessWidget {
  const _CardFace(this.c, {this.replace = false, this.number = 1, this.kept = 0});
  final _Card c;
  final bool replace;
  final int number, kept;
  @override
  Widget build(BuildContext context) {
    final p = _params(c.fx);
    return AnimatedContainer(
      duration: Mo.dur,
      curve: Mo.ease,
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: replace ? N.g15 : N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: replace ? _accent : N.g20)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          SizedBox(width: 22, height: 16, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(c.fx))),
          const SizedBox(width: 8),
          Expanded(child: _NameJp(c.fx.en, c.fx.jp, T.name(N.g95), T.label(N.g76), whole: true)),
          const SizedBox(width: 4),
          if (replace) _Tag('REPLACE $kept/2', hi: true) else Text('$number', style: _num()),
        ]),
        const SizedBox(height: 6),
        _Param(p.$1, c.a, hot: replace),
        const SizedBox(height: 4),
        _Param(p.$2, c.b, hot: replace),
      ]),
    );
  }
}

/// A stand-in Browser tile in a tray: thumb and name. `on` = picked up.
class _TrayChip extends StatelessWidget {
  const _TrayChip(this.fx, {this.on = false, this.hover = false});
  final _Fx fx;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 24,
        padding: const EdgeInsets.fromLTRB(4, 0, 8, 0),
        decoration: BoxDecoration(color: on ? N.g26 : (hover ? N.g20 : N.g13), borderRadius: BorderRadius.circular(4), border: Border.all(color: on ? _accent : N.g20)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [SizedBox(width: 22, height: 16, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(fx))), const SizedBox(width: 8), Text(fx.en, style: T.label(N.g91))]),
      );
}

/// The 2 px insertion line under a card: Alt held while a tile is over it means "add a new card here".
class _AddLine extends StatelessWidget {
  const _AddLine(this.on);
  final bool on;
  @override
  Widget build(BuildContext context) => SizedBox(height: 8, child: Center(child: AnimatedContainer(duration: Mo.dur, height: 2, margin: const EdgeInsets.symmetric(horizontal: 4), decoration: BoxDecoration(color: on ? _accent : _clear, borderRadius: BorderRadius.circular(1)))));
}

// ================================================================ B9-b Drop replaces (Alt adds): the Inspector's Effects region

class _Card {
  _Card(this.fx, this.a, this.b);
  _Fx fx;
  double a, b;
}

List<_Card> _seedCards() => [
      _Card(_fxs[0], .30, .55),
      _Card(_fxs[24], .45, .20),
      _Card(_fxs[8], .60, .35),
      _Card(_fxs[7], .25, .70),
      _Card(_fxs[25], .50, .40),
    ];

class _DropCards extends StatefulWidget {
  const _DropCards({required this.w, this.altSim = false});
  final double w;
  final bool altSim;
  @override
  State<_DropCards> createState() => _DropCardsState();
}

class _DropCardsState extends State<_DropCards> {
  List<_Card> _cards = _seedCards();
  final _history = <List<_Card>>[];
  _Fx? _carry;
  int _hov = -1;
  bool _alt = false;
  String? _msg;
  static final _tray = [_fxs[2], _fxs[26], _fxs[33], _fxs[17], _fxs[38]];

  bool get _isAlt => _alt || widget.altSim;

  List<_Card> _copy() => [for (final c in _cards) _Card(c.fx, c.a, c.b)];

  void _drop(int i) {
    final fx = _carry;
    if (fx == null) return;
    _history.add(_copy());
    setState(() {
      if (_isAlt) {
        _cards.insert(i + 1, _Card(fx, .4, .4));
        _msg = 'Added ${fx.en} below card ${i + 1}';
      } else {
        final old = _cards[i], p0 = _params(old.fx), p1 = _params(fx);
        final keep = (p0.$1 == p1.$1 ? 1 : 0) + (p0.$2 == p1.$2 ? 1 : 0);
        _cards[i] = _Card(fx, p0.$1 == p1.$1 ? old.a : .4, p0.$2 == p1.$2 ? old.b : .4);
        _msg = 'Replaced ${old.fx.en} with ${fx.en} · $keep/2 values kept';
      }
      _carry = null;
    });
  }

  bool _key(KeyEvent e) {
    setState(() => _alt = HardwareKeyboard.instance.isAltPressed);
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _carry != null) {
      setState(() {
        _carry = null;
        _msg = 'Cancelled';
      });
      return true;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed) && _history.isNotEmpty) {
      setState(() {
        _cards = _history.removeLast();
        _msg = 'Undone.';
      });
      return true;
    }
    return false;
  }

  Widget _cardView(int i) {
    final c = _cards[i];
    final target = _carry != null && _hov == i;
    final replace = target && !_isAlt, add = target && _isAlt;
    final kept = replace ? () {
      final p0 = _params(c.fx), p1 = _params(_carry!);
      return (p0.$1 == p1.$1 ? 1 : 0) + (p0.$2 == p1.$2 ? 1 : 0);
    }() : 0;
    return _Hit(
      cursor: _carry != null ? SystemMouseCursors.copy : SystemMouseCursors.basic,
      onHover: (v) => setState(() => _hov = v ? i : (_hov == i ? -1 : _hov)),
      onTap: () => _drop(i),
      builder: (cx, h, _) => Column(children: [
        _CardFace(c, replace: replace, number: i + 1, kept: kept),
        _AddLine(add),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) => _Keys(
        onKey: _key,
        child: _Follow(
          ghost: _carry == null ? null : _Ghost(_isAlt ? '${_carry!.en}  + Add' : '${_carry!.en}  = Replace', thumb: _Thumb(_carry!)),
          child: _withSide(
            _Frame(
              w: widget.w,
              child: _Col([
                _Head([_layerHead(), const SizedBox(height: 10), const IgnorePointer(child: Segmented(items: ['Transform', 'Effects', 'Material'], index: 1, expand: true))], bottom: 8),
                Padding(padding: const EdgeInsets.fromLTRB(_px, 0, _px, 0), child: _Sec('Effects', n: '${_cards.length}', trailing: _undoMark(_history.length))),
                _Grow(child: _Fade(child: ListView.builder(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.fromLTRB(_px, 0, _px, 12), itemCount: _cards.length, itemBuilder: (c, i) => _cardView(i)))),
                _Foot(msg: _msg, hints: const [('drop', 'replace'), ('Alt', 'add instead'), ('Esc', 'cancel'), ('⌘Z', 'undo')]),
              ]),
            ),
            _Side('Browser (outside)', [
              for (final f in _tray) _Hit(onTap: () => setState(() => _carry = _carry == f ? null : f), builder: (c, h, _) => _TrayChip(f, on: _carry == f, hover: h)),
            ]),
          ),
        ),
      );
}

// ================================================================ B9-c ↑↓ on the effect card (same family, in place)
