// Part of panel_browser_a.dart (cut there by size, as it was).
part of 'panel_browser_a.dart';

class _HoverThumbState extends State<_HoverThumb> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 1200));
  late bool _hover = widget.holdHover;
  late double? _x = widget.holdX;

  @override
  void initState() {
    super.initState();
    if (_hover && !_scrubbed) _c.value = .5;
  }

  double get _dur => widget.it.asset != null ? widget.it.asset!.dur : (widget.it.long ? 2.4 : 0);
  bool get _scrubbed => _dur > 0;

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final it = widget.it;
    final scrub = _scrubbed && _hover && _x != null ? (_x! / widget.w).clamp(0.0, 1.0) : null;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) {
        setState(() => _hover = true);
        if (!_scrubbed) _c.repeat();
      },
      onHover: (e) {
        setState(() => _x = e.localPosition.dx);
        if (_scrubbed) widget.onInfo?.call('${it.name} at ${_clock2((e.localPosition.dx / widget.w).clamp(0.0, 1.0) * _dur)} of ${_clock2(_dur)}');
      },
      onExit: (_) {
        setState(() {
          _hover = false;
          _x = null;
        });
        _c.stop();
        _c.value = 0;
        widget.onInfo?.call(null);
      },
      child: Container(
        width: widget.w,
        height: widget.h,
        decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: widget.selected ? Role.selected : (_hover ? N.g56 : N.g20))),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(5),
          child: Stack(fit: StackFit.expand, children: [
            if (it.asset != null)
              Art(scrub == null ? it.asset! : Asset(it.asset!.name, it.asset!.kind, it.asset!.seed + 1 + (scrub * 6).floor(), style: it.asset!.style, dur: it.asset!.dur, aspect: it.asset!.aspect))
            else
              AnimatedBuilder(animation: _c, builder: (context, _) => _Fx(it.fx, scrub ?? (_hover ? _c.value : .6), seed: it.seed)),
            if (it.asset != null && it.asset!.kind != Kind.image) Positioned(left: 4, top: 4, child: TypeBadge(it.asset!.kind)),
            if (_hover && !_scrubbed) Positioned(left: 4, top: 4, child: _pill('Playing')),
            if (_scrubbed) Positioned(right: 4, bottom: 4, child: TimePill(scrub == null ? _clock2(_dur) : _clock2(scrub * _dur))),
            // A 2 px rail along the foot of a scrubbable picture: it fills to the pointer, so the picture reads as a strip of time before anyone touches it.
            if (_scrubbed) Positioned(left: 0, right: 0, bottom: 0, height: 2, child: Stack(children: [const ColoredBox(color: Color(0x66000000)), Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: scrub ?? 0, child: const ColoredBox(color: N.g95)))])),
            if (scrub != null) Positioned(left: (scrub * widget.w).clamp(0.0, widget.w - 1), top: 0, bottom: 0, width: 1, child: const ColoredBox(color: N.g100)),
            if (widget.selected) Positioned(right: 0, top: 0, child: CustomPaint(size: const Size(8, 8), painter: _CornerPaint(Role.selected))),
          ]),
        ),
      ),
    );
  }
}

/// An 8 px triangle in the top right corner: the shape that says "selected" beside the accent edge.
class _CornerPaint extends CustomPainter {
  const _CornerPaint(this.color, {this.bottomLeft = false});
  final Color color;
  final bool bottomLeft;
  @override
  void paint(Canvas c, Size s) => c.drawPath(bottomLeft ? (Path()..moveTo(0, 0)..lineTo(0, s.height)..lineTo(s.width, s.height)..close()) : (Path()..moveTo(0, 0)..lineTo(s.width, 0)..lineTo(s.width, s.height)..close()), Paint()..color = color);
  @override
  bool shouldRepaint(_CornerPaint o) => o.color != color || o.bottomLeft != bottomLeft;
}

String _clock2(double s) => s < 60 ? '${s.toStringAsFixed(1)}s' : clock(s);

/// A tile of the hover grid: the thumbnail (hover plays, x scrubs), the name (selects), then kind or category and length.
class _HoverTile extends StatelessWidget {
  const _HoverTile(this.it, {required this.w, this.selected = false, this.onInfo, this.onSelect});
  final It it;
  final double w;
  final bool selected;
  final ValueChanged<String?>? onInfo;
  final VoidCallback? onSelect;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
          _HoverThumb(it, w: w, h: w * .62, selected: selected, onInfo: onInfo),
          const SizedBox(height: 6),
          _H(onTap: onSelect, builder: (context, h) => Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91).copyWith(decoration: selected ? TextDecoration.underline : TextDecoration.none, decorationColor: Role.selected))),
          const SizedBox(height: 4),
          Text(it.asset != null ? (it.asset!.kind == Kind.image ? 'Image · ${it.asset!.dims}' : it.asset!.dims) : '${it.cat} · ${it.kind}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
        ]),
      );
}

/// A row of the hover list, 50 px: the thumbnail is the hover target (60 or 76 wide), the name selects.
class _HoverRow extends StatelessWidget {
  const _HoverRow(this.it, {required this.tw, this.selected = false, this.onInfo, this.onSelect, this.holdHover = false, this.holdX});
  final It it;
  final double tw;
  final bool selected, holdHover;
  final double? holdX;
  final ValueChanged<String?>? onInfo;
  final VoidCallback? onSelect;
  @override
  Widget build(BuildContext context) {
    final meta = it.asset != null ? it.asset!.dims : (it.long ? '2.4s' : 'loop');
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      child: Row(children: [
        _HoverThumb(it, w: tw, h: 42, selected: selected, onInfo: onInfo, holdHover: holdHover, holdX: holdX),
        const SizedBox(width: 8),
        Expanded(
          child: _H(
            onTap: onSelect,
            builder: (context, h) => Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
              Text(it.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91).copyWith(decoration: selected ? TextDecoration.underline : TextDecoration.none, decorationColor: Role.selected)),
              const SizedBox(height: 4),
              Text(it.asset != null && it.asset!.kind != Kind.image ? meta : (it.asset != null ? 'Image · $meta' : '${it.cat} · $meta'), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
            ]),
          ),
        ),
      ]),
    );
  }
}

class _HoverTiles extends StatefulWidget {
  const _HoverTiles({super.key, required this.w, this.rows = false});
  final double w;
  final bool rows;
  @override
  State<_HoverTiles> createState() => _HoverTilesState();
}

class _HoverTilesState extends State<_HoverTiles> {
  String? _info;
  int _sel = -1;

  List<It> get _items {
    It n(String s) => kItems.firstWhere((i) => i.name == s);
    return [n('Glow'), n('Gaussian Blur'), n('lens_bloom.mp4'), n('Glitch RGB Split'), n('Typewriter'), n('city_pass.mov'), n('Pop In Overshoot'), n('sunrise.jpg'), n('sphere_tour.mp4'), n('Wave Warp'), n('wind_pad.wav'), n('Film Grain'), n('Lower Third Slide'), n('dusk_ridge.jpg')];
  }

  @override
  Widget build(BuildContext context) {
    final its = _items;
    final String idle = _sel >= 0 ? its[_sel].name : '${its.length} items';
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _pad(_Tabs(_places, 0, (_) {}, toggle: false), t: 10, b: 8),
      Expanded(
        child: widget.rows
            ? _Fit(natural: its.length * 50.0, child: _FadeBottom(child: ListView.builder(
                padding: EdgeInsets.zero,
                itemExtent: 50,
                itemCount: its.length,
                itemBuilder: (context, i) {
                  final tw = widget.w < 300 ? 60.0 : 76.0;
                  return _HoverRow(its[i], tw: tw, selected: _sel == i, onInfo: (s) => setState(() => _info = s), onSelect: () => setState(() => _sel = i));
                })))
            : LayoutBuilder(builder: (context, box) {
                final tw = (widget.w - 20 - 8) / 2;
                return _FadeBottom(child: SingleChildScrollView(
                  padding: const EdgeInsets.fromLTRB(10, 0, 10, 10),
                  child: Wrap(spacing: 8, runSpacing: 10, children: [
                    for (var i = 0; i < its.length; i++)
                      _HoverTile(its[i], w: tw, selected: _sel == i, onInfo: (s) => setState(() => _info = s), onSelect: () => setState(() => _sel = i)),
                  ]),
                ));
              }),
      ),
      _Status(_info ?? idle),
    ]);
  }

}

/// B6: long clips, one big tile each; x maps to time.
class _ScrubClips extends StatefulWidget {
  const _ScrubClips({super.key, required this.w});
  final double w;
  @override
  State<_ScrubClips> createState() => _ScrubClipsState();
}

class _ScrubClipsState extends State<_ScrubClips> {
  String? _info;
  @override
  Widget build(BuildContext context) {
    final names = ['lens_bloom.mp4', 'city_pass.mov', 'sphere_tour.mp4', 'Lower Third Slide', 'kick_loop.wav'];
    final tw = widget.w - 20;
    return Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      _pad(Text('Clips', style: T.title()), t: 12, b: 2),
      const SizedBox(height: 8),
      Expanded(
        child: ListView(padding: const EdgeInsets.fromLTRB(10, 0, 10, 10), children: [
          for (final n in names) ...[
            _HoverThumb(kItems.firstWhere((i) => i.name == n), w: tw, h: tw * .5, onInfo: (s) => setState(() => _info = s)),
            const SizedBox(height: 6),
            Row(children: [
              Expanded(child: Text(n, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name())),
              Text(() {
                final a = kItems.firstWhere((i) => i.name == n);
                return a.asset != null ? a.asset!.dims : '2.4s · preset';
              }(), style: _num(N.g76)),
            ]),
            const SizedBox(height: 12),
          ],
        ]),
      ),
      _Status(_info ?? '${names.length} clips'),
    ]);
  }
}

// ---- B5-c  preview on your own frame

/// Your own frame in the Browser: a 1 px edge (g20, the mode colour while previewing), the picture, the layer label on the left and the state label on the right.
class _OwnFrameView extends StatelessWidget {
  const _OwnFrameView({required this.w, required this.h, required this.label, required this.previewing, required this.picture, this.applied, this.noTarget = false});
  final double w, h;
  final String label;
  final bool previewing, noTarget;
  final String? applied;
  final Widget picture;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        height: h,
        child: CustomPaint(
          foregroundPainter: previewing ? DashedBox(Role.selected, radius: 0) : null,
          child: DecoratedBox(
          decoration: BoxDecoration(border: Border.all(color: previewing ? _clear : N.g20)),
          child: ClipRect(
            child: Stack(fit: StackFit.expand, children: [
              picture,
              _edge(_pill(label), previewing ? _pill('PREVIEW', fill: N.g20) : (applied != null ? _pill('APPLIED · $applied', fill: N.g26) : null)),
              if (previewing) const Positioned(right: 6, bottom: 6, child: Row(mainAxisSize: MainAxisSize.min, children: [_Kbd('Enter'), SizedBox(width: 4), _Kbd('Esc')])),
            ]),
          ),
        ),
        ),
      );
}

/// Tried and Written under the frame: the written count brightens (g95) once it is not zero; right = the Down key mark to start (the Enter and Esc marks sit on the preview frame), nothing when hover starts it.
class _Counters extends StatelessWidget {
  const _Counters(this.tried, this.written, this.hint);
  final int tried, written;
  final String? hint;
  @override
  Widget build(BuildContext context) => Row(children: [
        Text('Tried ', style: T.label(N.g76)),
        Text('$tried', style: T.value(N.g95)),
        const SizedBox(width: 12),
        Text('Written ', style: T.label(N.g76)),
        Text('$written', style: T.value(written > 0 ? N.g95 : N.g76)),
        const Spacer(),
        if (hint != null && hint!.isNotEmpty) _Kbd(hint!),
      ]);
}

class _OwnFrame extends StatefulWidget {
  const _OwnFrame({super.key, required this.w, required this.hover, required this.noTarget});
  final double w;
  final bool hover, noTarget;
  @override
  State<_OwnFrame> createState() => _OwnFrameState();
}

class _OwnFrameState extends State<_OwnFrame> {
  late final _root = FocusNode(onKeyEvent: _key);
  final _sc = ScrollController();
  List<It> get _pool => kItems.where((i) => i.place != 'Project' && i.applies != 'key').toList();
  int _sel = 0, _tries = 0, _writes = 0;
  It? _pv;
  final List<It> _applied = [];
  String _say = 'Layer 3 · Title';

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _peek(_pool.indexWhere((i) => i.name == 'Glow'));
    });
  }

  void _peek(int i) {
    final c = _pool.length;
    setState(() {
      _sel = i.clamp(0, c - 1);
      _pv = _pool[_sel];
      _tries++;
      _say = '“${_pv!.name}”';
    });
    _revealAt(_sc, _sel * 34.0, _sel * 34.0 + 38);
  }

  void _commit() {
    if (_pv == null) return;
    if (widget.noTarget) {
      setState(() => _say = 'No target selected');
      return;
    }
    setState(() {
      _applied.add(_pv!);
      _writes++;
      _say = 'Applied “${_pv!.name}”';
      _pv = null;
    });
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (_isDown(e, LogicalKeyboardKey.arrowDown)) {
      _peek(_pv == null && _tries == 0 ? 0 : _sel + 1);
      return KeyEventResult.handled;
    }
    if (_isDown(e, LogicalKeyboardKey.arrowUp)) {
      _peek(_sel - 1);
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.enter) {
      _commit();
      return KeyEventResult.handled;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      setState(() {
        _pv = null;
        _say = 'Dropped';
      });
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  void dispose() {
    _root.dispose();
    _sc.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final fw = widget.w - 20, fh = fw * 9 / 16, pool = _pool;
    final label = widget.noTarget ? 'Sample frame (no selection)' : 'Layer 3 · Title';
    return Focus(
      focusNode: _root,
      autofocus: true,
      onFocusChange: (f) {
        if (!f && !widget.hover && mounted) setState(() => _pv = null);
      },
      child: MouseRegion(
        onExit: (_) {
          if (widget.hover) setState(() => _pv = null);
        },
        child: GestureDetector(
          behavior: HitTestBehavior.translucent,
          onTap: _root.requestFocus,
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            _pad(
              _OwnFrameView(
                w: fw,
                h: fh,
                label: label,
                previewing: _pv != null,
                applied: _applied.isNotEmpty ? _applied.last.name : null,
                noTarget: widget.noTarget,
                picture: _Loop(running: _pv != null, builder: (context, t) => _Fx(_pv?.fx ?? (_applied.isEmpty ? Fx.none : _applied.last.fx), _pv != null ? t : .6, base: _ownBase(sample: widget.noTarget))),
              ),
              t: 10,
            ),
            _pad(
              _Counters(_tries, _writes, _pv != null ? null : (widget.hover ? '' : 'Down')),
              t: 8,
              b: 8,
            ),
            Container(height: 1, color: N.rowLine),
            Expanded(
              child: _Fit(natural: pool.length * 34.0 + 4, child: _QuietKind(_majority(pool), child: _FadeBottom(top: true, child: ListView.builder(
                controller: _sc,
                padding: const EdgeInsets.only(top: 4),
                itemExtent: 34,
                itemCount: pool.length,
                itemBuilder: (context, i) => MouseRegion(
                  onEnter: (_) {
                    if (widget.hover) _peek(i);
                  },
                  child: _ItemRow(pool[i], selected: i == _sel && (_pv != null || _tries > 0), onTap: () {
                    _peek(i);
                    _root.requestFocus();
                  }, onUse: () {
                    _peek(i);
                    _commit();
                  }),
                ),
              )))),
            ),
            _Status(_say),
          ]),
        ),
      ),
    );
  }
}

enum _Phase { preview, applied, dropped }

/// The band under a frame that says which state it is in: a word and a shape. PREVIEW = dashed g56 edge + the Esc key mark, APPLIED = g20 ground, the tick + the Cmd Z key mark, DROPPED = g63 word struck through.
class _PhaseBanner extends StatelessWidget {
  const _PhaseBanner(this.phase);
  final _Phase phase;
  @override
  Widget build(BuildContext context) {
    final (head, tail) = switch (phase) {
      _Phase.preview => ('PREVIEW', '0 writes'),
      _Phase.applied => ('APPLIED', '1 write'),
      _Phase.dropped => ('DROPPED', '0 writes'),
    };
    final dim = phase == _Phase.dropped;
    return Container(
      height: 28,
      decoration: BoxDecoration(color: phase == _Phase.applied ? N.g20 : N.g10),
      child: CustomPaint(
        foregroundPainter: phase == _Phase.preview ? const DashedBox(N.g56, radius: 0) : null,
        child: Stack(fit: StackFit.expand, children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(10, 0, 8, 0),
            child: Row(children: [
              Text(head, style: T.micro(dim ? N.g63 : N.g95).copyWith(letterSpacing: .6, decoration: dim ? TextDecoration.lineThrough : TextDecoration.none, decorationColor: N.g63)),
              const SizedBox(width: 8),
              Expanded(child: Text(tail, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
              if (phase == _Phase.preview) const _Kbd('Esc'),
              if (phase == _Phase.applied) const _Kbd('Cmd Z'),
            ]),
          ),
          _GutterTick(phase == _Phase.applied, height: 12),
        ]),
      ),
    );
  }
}

/// B5-c, the three states side by side: previewing, applied, dropped. Every state is a word and a grey step.
class _Phases extends StatelessWidget {
  const _Phases({required this.w, required this.fx});
  final double w;
  final It fx;
  @override
  Widget build(BuildContext context) {
    final fw = w - 20, fh = fw * 9 / 16 * (w < 300 ? .9 : .62);
    Widget frame(Widget child, Widget banner) => Padding(
          padding: const EdgeInsets.fromLTRB(10, 0, 10, 12),
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.stretch, children: [SizedBox(width: fw, height: fh, child: ClipRect(child: child)), banner]),
        );
    return ListView(padding: const EdgeInsets.only(top: 12), children: [
      frame(_Loop(builder: (context, t) => _Fx(fx.fx, t, base: _ownBase())), const _PhaseBanner(_Phase.preview)),
      frame(_Fx(fx.fx, .6, base: _ownBase()), const _PhaseBanner(_Phase.applied)),
      frame(_ownBase(), const _PhaseBanner(_Phase.dropped)),
    ]);
  }
}

// ---- B5-d  a fixed big preview surface
