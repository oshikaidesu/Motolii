part of 'panel_inspector_b.dart';

// ---- I9: values that follow other values ---------------------------------------------------------------------------------------
// A relation of one value to another uses the Follow family's orange, nothing else (one hue = one meaning).

final Color _fol = Fam.follow.c;
/// The lasso selection (Stage dots, outline, ring) is a selection, not a link: Role.selected in a palette, the Follow orange in Grey.
Color get _selC => Role.of(_fol, Role.selected);
class _It {
  const _It(this.id, this.label, this.unit, this.v, {this.group = 'Transform', this.kind = _K.num, this.dec = 0, this.far = false, this.opts = const []});
  final String id, label, unit, group;
  final double v;
  final _K kind;
  final int dec;
  final bool far; // a value on another layer: it can be followed, it cannot be dragged from here
  final List<String> opts;
  String get layerOf => far ? label.split(' · ').first : '';
  String get propOf => far ? label.split(' · ').last : label;
}

const _items = <_It>[
  _It('pos.x', 'Position X', 'px', 960),
  _It('pos.y', 'Position Y', 'px', 540),
  _It('scale', 'Scale', '%', 100),
  _It('rot', 'Rotation', '°', 0, dec: 1),
  _It('op', 'Opacity', '%', 100),
  _It('gl.i', 'Glow Intensity', '', 1.4, group: 'Glow', dec: 2),
  _It('gl.r', 'Glow Radius', 'px', 25, group: 'Glow'),
  _It('gl.th', 'Glow Threshold', '%', 60, group: 'Glow'),
  _It('gl.c', 'Glow Color', '', 4, group: 'Glow', kind: _K.color),
  _It('gl.b', 'Blending Mode', '', 0, group: 'Glow', kind: _K.choice, opts: _bl),
  _It('o.op', 'ring_glow · Opacity', '%', 72, group: 'On other layers', far: true),
  _It('o.px', 'title_card · Position X', 'px', 960, group: 'On other layers', far: true),
  _It('o.hue', 'bg_gradient · Hue Shift', '°', -12, group: 'On other layers', far: true, dec: 1),
  _It('o.lvl', 'kick_loop.wav · Level', '%', 38, group: 'On other layers', far: true),
];

bool _compat(_It a, _It b) => a.id != b.id && a.kind == _K.num && b.kind == _K.num && a.unit == b.unit;

class _Bind {
  _Bind(this.src, this.lo, this.hi);
  String src;
  double lo, hi;
}

/// The small ring that starts a link. Hollow at rest, filled in the Follow orange when the value follows something.
class _Ring extends StatefulWidget {
  const _Ring({required this.bound, required this.active, this.onStart, this.onMove, this.onEnd, this.pose});
  final _Pose? pose;
  final bool bound, active;
  final ValueChanged<Offset>? onStart, onMove;
  final VoidCallback? onEnd;
  @override
  State<_Ring> createState() => _RingState();
}

class _RingState extends State<_Ring> {
  bool h = false;
  @override
  Widget build(BuildContext context) {
    final on = widget.onStart != null;
    final hv = h || widget.pose == _Pose.hover;
    return MouseRegion(
      cursor: on ? SystemMouseCursors.precise : SystemMouseCursors.basic,
      onEnter: (_) => setState(() => h = true),
      onExit: (_) => setState(() => h = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanStart: on ? (d) => widget.onStart!(d.globalPosition) : null,
        onPanUpdate: on ? (d) => widget.onMove?.call(d.globalPosition) : null,
        onPanEnd: on ? (_) => widget.onEnd?.call() : null,
        child: SizedBox(
          width: 22,
          height: 22,
          child: Center(
            child: AnimatedContainer(
              duration: Mo.dur,
              curve: Mo.ease,
              width: widget.active || hv && on ? 10 : 8,
              height: widget.active || hv && on ? 10 : 8,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: widget.bound || widget.active ? _fol : null,
                border: Border.all(color: widget.bound || widget.active ? _fol : (on ? (hv ? N.g95 : N.g63) : N.g20), width: 1.5),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _LinePaint extends CustomPainter {
  const _LinePaint(this.a, this.b);
  final Offset a, b;
  @override
  void paint(Canvas c, Size s) {
    final bow = math.min(a.dx, b.dx) - 36;
    final path = Path()..moveTo(a.dx, a.dy)..cubicTo(bow, a.dy, bow, b.dy, b.dx, b.dy);
    c.drawPath(path, Paint()..color = _fol..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round);
    c.drawCircle(b, 3.5, Paint()..color = _fol);
  }

  @override
  bool shouldRepaint(_LinePaint o) => o.a != a || o.b != b;
}

// ---- I9-a: grab and tie ------------------------------------------------------------------------------------------------------------

class _WhipPanel extends StatefulWidget {
  const _WhipPanel({required this.h, required this.mode});
  final double h;
  final String mode; // live | mid-grab | bound
  @override
  State<_WhipPanel> createState() => _WhipPanelState();
}

class _WhipPanelState extends State<_WhipPanel> {
  final keys = {for (final i in _items) i.id: GlobalKey()};
  final stackKey = GlobalKey();
  final vals = {for (final i in _items) i.id: i.v};
  final binds = <String, _Bind>{};
  String? grab, over;
  Offset? pointer;
  final scroll = ScrollController();

  @override
  void initState() {
    super.initState();
    _mode();
  }

  @override
  void didUpdateWidget(_WhipPanel old) {
    super.didUpdateWidget(old);
    if (old.mode != widget.mode) {
      _mode();
    }
  }

  @override
  void dispose() {
    scroll.dispose();
    super.dispose();
  }

  _It _by(String id) => _items.firstWhere((i) => i.id == id);

  void _mode() {
    binds.clear();
    grab = over = null;
    pointer = null;
    if (widget.mode == 'bound') {
      binds['scale'] = _Bind('o.op', 100, 180);
      binds['pos.x'] = _Bind('o.px', 880, 1040);
      binds['gl.i'] = _Bind('o.lvl', .4, 2.4);
    } else if (widget.mode == 'mid-grab') {
      grab = 'op';
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted || grab == null) {
          return;
        }
        final c = _center('o.op');
        if (c != null) {
          setState(() {
            pointer = c;
            over = 'o.op';
          });
        }
      });
    }
    setState(() {});
  }

  Offset? _center(String id) {
    final rb = keys[id]!.currentContext?.findRenderObject() as RenderBox?;
    return rb == null || !rb.hasSize ? null : rb.localToGlobal(rb.size.center(Offset.zero));
  }

  Offset? _ringPos(String id) {
    final rb = keys[id]!.currentContext?.findRenderObject() as RenderBox?;
    return rb == null || !rb.hasSize ? null : rb.localToGlobal(Offset(12 + 11, rb.size.height / 2));
  }

  void _move(Offset g) {
    String? hit;
    for (final i in _items) {
      final rb = keys[i.id]!.currentContext?.findRenderObject() as RenderBox?;
      if (rb == null || !rb.hasSize) {
        continue;
      }
      if ((rb.localToGlobal(Offset.zero) & rb.size).contains(g)) {
        hit = i.id;
      }
    }
    setState(() {
      pointer = g;
      over = hit != null && grab != null && _compat(_by(grab!), _by(hit)) ? hit : null;
    });
  }

  void _end() {
    if (grab != null && over != null) {
      final it = _by(grab!);
      binds[grab!] = _Bind(over!, it.unit == '%' ? 0 : it.v * .5, it.unit == '%' ? 100 : it.v * 1.5);
    }
    setState(() {
      grab = over = null;
      pointer = null;
    });
  }

  Widget _row(_It it) {
    final ga = grab == null ? null : _by(grab!);
    final ok = ga == null || _compat(ga, it) || it.id == grab;
    final isOver = over == it.id, isGrab = grab == it.id, bound = binds[it.id];
    final target = ga != null && _compat(ga, it);
    Widget ctl;
    if (bound != null) {
      final src = _by(bound.src);
      ctl = _BoundCell(text: src.label, onUnlink: () => setState(() => binds.remove(it.id)));
    } else if (it.kind == _K.num) {
      ctl = _Num(
        value: vals[it.id]!,
        dec: it.dec,
        unit: it.unit,
        enabled: !it.far,
        onDelta: (d) => setState(() => vals[it.id] = vals[it.id]! + d * (it.dec > 0 ? .1 : 1)),
        onType: (x) => setState(() => vals[it.id] = x),
      );
    } else if (it.kind == _K.color) {
      ctl = _ColorCell(index: vals[it.id]!.round(), onTap: () => setState(() => vals[it.id] = (vals[it.id]! + 1) % _swatch.length));
    } else {
      ctl = _Pick(text: it.opts[vals[it.id]!.round()], onTap: () => setState(() => vals[it.id] = (vals[it.id]! + 1) % it.opts.length));
    }
    final label = it.far ? _FarLabel(it.layerOf, it.propOf) : _Mid(it.label, T.label(bound != null || target || isGrab ? N.g95 : N.g63).copyWith(fontSize: 11, height: 1.2));
    return KeyedSubtree(
      key: keys[it.id],
      child: AnimatedOpacity(
        duration: Mo.dur,
        opacity: ok ? 1 : .3,
        child: _Gut(
          tick: bound != null ? _fol : (target ? _fol.withValues(alpha: .6) : null),
          child: AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          constraints: const BoxConstraints(minHeight: 28),
          padding: const EdgeInsets.only(left: 12, right: 12, top: 4, bottom: 4),
          decoration: BoxDecoration(color: isOver ? _fol.withValues(alpha: .16) : (isGrab ? N.g20 : null)),
          child: Row(children: [
            _Ring(
              bound: bound != null,
              active: isGrab,
              onStart: it.far ? null : (g) => setState(() {
                    grab = it.id;
                    pointer = g;
                  }),
              onMove: _move,
              onEnd: _end,
            ),
            Expanded(flex: 5, child: label),
            const SizedBox(width: 8),
            Expanded(flex: 6, child: ctl),
          ]),
        )),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final rows = <Widget>[];
    String? g;
    for (final it in _items) {
      if (it.group != g) {
        g = it.group;
        rows.add(_Sect(g));
      }
      rows.add(_row(it));
    }
    Offset? from, to;
    if (grab != null && pointer != null) {
      final rb = stackKey.currentContext?.findRenderObject() as RenderBox?;
      final rp = _ringPos(grab!);
      if (rb != null && rp != null) {
        from = rb.globalToLocal(rp);
        to = rb.globalToLocal(pointer!);
      }
    }
    final nBound = binds.length;
    return _Panel(
      title: 'star_burst',
      sub: 'Shape layer · $nBound follow${nBound == 1 ? '' : 's'}',
      h: widget.h,
      child: Column(mainAxisSize: _FitScope.of(context) ? MainAxisSize.min : MainAxisSize.max, children: [
        _grow(
          context,
          Stack(key: stackKey, children: [
            _Lv(controller: scroll, children: rows),
            if (from != null && to != null) Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _LinePaint(from, to)))),
          ]),
        ),
      ]),
    );
  }
}

/// A value on another layer: the layer on one line (g63, 10 px), the property under it (g91), each cut in its own middle, so neither name hides the other.
class _FarLabel extends StatelessWidget {
  const _FarLabel(this.layer, this.prop);
  final String layer, prop;
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
        _Mid(layer, T.label(N.g63).copyWith(fontSize: 10, height: 1.2)),
        _Mid(prop, T.label(N.g91).copyWith(fontSize: 11, height: 1.2)),
      ]);
}

/// A bound value reads "← source" in place of its number; Unlink is always written (g76, g95 and underlined on hover), so hovering moves nothing.
class _BoundCell extends StatelessWidget {
  const _BoundCell({required this.text, required this.onUnlink, this.pose});
  final _Pose? pose;
  final String text;
  final VoidCallback onUnlink;
  @override
  Widget build(BuildContext context) => _Hv(
        pose: pose,
        onTap: onUnlink,
        builder: (_, h, _) => Container(
          height: 20,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: h ? N.g38 : N.g20)),
          child: Row(children: [
            Expanded(child: Text('← $text', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g95).copyWith(fontSize: 11))),
            const SizedBox(width: 8),
            Text('Unlink', maxLines: 1, style: T.label(h ? N.g95 : N.g76).copyWith(decoration: h ? TextDecoration.underline : TextDecoration.none, decorationColor: N.g95)),
          ]),
        ),
      );
}

// ---- I9-b: lasso, one to many ---------------------------------------------------------------------------------------------------

class _LassoPanel extends StatefulWidget {
  const _LassoPanel({required this.h, required this.count, required this.locks, required this.drawn, required this.followed, this.w = 282});
  final double h, w;
  final int count;
  final bool locks, drawn, followed;
  @override
  State<_LassoPanel> createState() => _LassoPanelState();
}
