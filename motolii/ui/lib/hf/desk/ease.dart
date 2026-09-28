import 'dart:math' as math;
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import 'common.dart';
import '../../foundation/ease_meaning.dart';

typedef Shape = double Function(double t);

Shape bezierShape(double x1, double y1, double x2, double y2) => (t) {
      var lo = 0.0, hi = 1.0;
      for (var i = 0; i < 24; i++) {
        final u = (lo + hi) / 2;
        final x = 3 * (1 - u) * (1 - u) * u * x1 + 3 * (1 - u) * u * u * x2 + u * u * u;
        if (x < t) { lo = u; } else { hi = u; }
      }
      final u = (lo + hi) / 2;
      return 3 * (1 - u) * (1 - u) * u * y1 + 3 * (1 - u) * u * u * y2 + u * u * u;
    };

double _bounce(double x) {
  if (x < 1 / 2.75) return 7.5625 * x * x;
  if (x < 2 / 2.75) { final y = x - 1.5 / 2.75; return 7.5625 * y * y + .75; }
  if (x < 2.5 / 2.75) { final y = x - 2.25 / 2.75; return 7.5625 * y * y + .9375; }
  final y = x - 2.625 / 2.75;
  return 7.5625 * y * y + .984375;
}

class Preset {
  const Preset(this.name, this.bez, this.fn);
  final String name;
  final List<double>? bez;
  final Shape? fn;
  Shape get shape => bez != null ? bezierShape(bez![0], bez![1], bez![2], bez![3]) : fn!;
}

final easePresets = <Preset>[
  const Preset('Linear', [0, 0, 1, 1], null),
  const Preset('Ease', [.42, 0, .58, 1], null),
  const Preset('Bezier', [.25, .1, .78, .92], null),
  Preset('Spring', null, (t) => 1 - math.exp(-6 * t) * math.cos(t * math.pi * 5)),
  Preset('Bounce', null, _bounce),
];

class Seg {
  Seg(this.p, this.frames, [List<Preset>? kinds]) : kinds = kinds ?? easePresets {
    final b = this.kinds[p].bez;
    if (b != null) { x1 = b[0]; y1 = b[1]; x2 = b[2]; y2 = b[3]; }
  }
  int p;
  final int frames;
  final List<Preset> kinds;
  double x1 = .25, y1 = .1, x2 = .78, y2 = .92;
  bool get bez => kinds[p].bez != null;
  Shape get shape => bez ? bezierShape(x1, y1, x2, y2) : kinds[p].fn!;
  List<double> get values => [x1, y1, x2, y2];
  void choose(int i) {
    p = i;
    final b = kinds[i].bez;
    if (b != null) { x1 = b[0]; y1 = b[1]; x2 = b[2]; y2 = b[3]; }
  }
  void setValues(List<double> v) { p = kinds.indexWhere((k) => k.bez != null && k.name == 'Bezier').clamp(0, kinds.length - 1); x1 = v[0]; y1 = v[1]; x2 = v[2]; y2 = v[3]; }
}

/// A host that owns the curves: its kinds (the preset row), its intervals, and where a changed curve goes.
/// [apply] is given the interval the curve belongs to, or null for every interval shown.
abstract class EaseHost implements Listenable {
  List<Preset> get kinds;
  List<Seg> get intervals;
  void apply(int? interval, Seg curve);

  /// The curve while a handle is held (the host may show it before [apply]); [cancel] drops it.
  void preview(int? interval, Seg curve);
  void cancel();

  /// The kept curves: the copied one first, then the saved presets.
  List<Seg> get saved;
  void copyCurve(Seg curve);
  void savePreset(Seg curve);
  void clearSaved();

  /// New keys take this curve (and Animate, when on, keys with it from now).
  void useForNewKeys(Seg curve);

  /// What the desk edits, in words (a layer's property and frames, a sequence, or the workspace curve).
  String get target;

  /// The graph's corner label when it is not an interval's frames (a sequence's layers, "Workspace").
  String? get caption;

  /// The interval the playhead is in or last passed, and where in [interval] (0–1) it stands; null when outside.
  int? get current;
  double? playhead(int? interval);

  /// How many layers the curve spreads delays over (the Ease desk's ghost mode: several layers picked, no keys);
  /// 0 when it shapes key intervals.
  int get sequence;
}

Rect easePlotBox(Size s) => Rect.fromLTRB(14, 34, s.width - 34, s.height - 22);
Offset easeAt(Size s, double x, double y) {
  final r = easePlotBox(s);
  return Offset(r.left + r.width * x, r.bottom - r.height * (y + .15) / 1.3);
}

const _segColors = [kViolet, kMint, kPink];
const _presetColors = [kBlue, kViolet, kPink, kMint, kYellow];

class EaseDesk extends StatefulWidget {
  const EaseDesk({super.key, this.host});
  final EaseHost? host;
  @override
  State<EaseDesk> createState() => _EaseDeskState();
}

class _EaseDeskState extends State<EaseDesk> with SingleTickerProviderStateMixin {
  final _segs = [Seg(2, 12), Seg(1, 8), Seg(3, 4)];
  List<Seg> _held = const [];
  List<Seg> get segs => widget.host == null ? _segs : _held;
  List<Preset> get presets => widget.host?.kinds ?? easePresets;

  @override
  void initState() {
    super.initState();
    widget.host?.addListener(_absorb);
    _absorb();
  }

  void _absorb() {
    final h = widget.host;
    if (h == null || drag != null) return;
    setState(() {
      _held = h.intervals;
      if (sel >= _held.length) sel = _held.isEmpty ? 0 : _held.length - 1;
      // the interval shown is the one the playhead is in (or last passed), as Classic's desk follows it
      final now = h.current;
      if (now != null && sel >= 0 && now < _held.length) sel = now;
    });
  }

  void _write() => widget.host?.apply(mixed ? null : sel, cur);
  final saved = <List<double>>[[.7, 0, .3, 1], [.2, .9, .6, 1.2]];
  int sel = 0; // -1 = all intervals (mixed)
  bool ghost = false;
  int? drag;
  bool _moved = false;
  int? peek; // preset under the pointer or the keyboard: shown on the plot until it leaves
  int keyFocus = 2;
  final _presetFocus = FocusNode();
  late final AnimationController _play = AnimationController(vsync: this, duration: const Duration(milliseconds: 1400))..addListener(() => setState(() {}));

  @override
  void dispose() {
    widget.host?.removeListener(_absorb);
    _play.dispose();
    _presetFocus.dispose();
    _plotKeys.dispose();
    super.dispose();
  }

  Seg get cur => segs.isEmpty ? Seg(0, 1, presets) : segs[sel < 0 ? 0 : sel];
  bool get mixed => sel < 0;
  String _n(double v) => mixed ? '—' : v.toStringAsFixed(2);

  @override
  Widget build(BuildContext context) => DeskShell(
        kind: DeskKind.ease,
        title: 'Ease',
        subtitle: widget.host?.target ?? 'KEYFRAMES · MOTION',
        full: (c, s) => _body(s.height, s.width),
        strip: (c, s) => _body(s.height, s.width),
        tall: (c, s) => _body(s.height, s.width),
      );

  // Primary loop (interval -> preset -> curve -> play) always shares one viewport.
  // Precision, saved curves and options are secondary and are the only thing that scrolls.
  Widget _body(double h, double w) {
    final pad = w < 230 ? 8.0 : 12.0;
    if (h < 200) {
      // a short strip: the face first, everything else one scroll away
      return SingleChildScrollView(
        key: const ValueKey('ease-scroll'),
        physics: drag != null ? const NeverScrollableScrollPhysics() : null,
        padding: EdgeInsets.fromLTRB(pad, 12, pad, 16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(height: math.max(56, h - 20), child: _plot(labels: false)),
          const SizedBox(height: 8),
          _presetRow(),
          const SizedBox(height: 8),
          _navigator(w),
          _secondary(w),
        ]),
      );
    }
    final plotH = (h * .46).clamp(110.0, 300.0);
    return Padding(
      padding: EdgeInsets.fromLTRB(pad, 12, pad, 0),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        SizedBox(height: plotH, child: _plot(labels: w >= 230 && plotH >= 150)),
        const SizedBox(height: 10),
        _presetRow(),
        const SizedBox(height: 8),
        _navigator(w),
        Expanded(child: SingleChildScrollView(key: const ValueKey('ease-scroll'), padding: const EdgeInsets.only(bottom: 16), child: _secondary(w))),
      ]),
    );
  }

  Widget _secondary(double w) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _section('VALUES', _values(w)),
        _section('SAVED', _savedRow()),
        _section('OPTIONS', _options()),
      ]);

  Widget _section(String t, Widget child) => Padding(
        padding: const EdgeInsets.only(top: 16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Text(t, style: sans(9.5, c: kMuted, w: FontWeight.w500, ls: 1.4)),
          const SizedBox(height: 8),
          child,
        ]),
      );

  void _step(int d) => setState(() {
        final n = segs.length;
        sel = sel < 0 ? (d > 0 ? 0 : n - 1) : (sel + d + n) % n;
      });

  // Play, previous / next interval, and the intervals themselves, in one strip under the presets.
  Widget _navigator(double w) {
    final narrow = w < 230;
    Widget arrow(String key, String t, int d) => GestureDetector(
          key: ValueKey(key),
          behavior: HitTestBehavior.opaque,
          onTap: () => _step(d),
          child: SizedBox(width: 22, height: 36, child: Center(child: Text(t, style: sans(18, c: kMuted)))),
        );
    return SizedBox(
      height: 36,
      child: Row(children: [
        GestureDetector(
          key: const ValueKey('ease-play'),
          onTap: () => _play.isAnimating ? _play.stop() : _play.forward(from: 0),
          child: Container(width: 36, height: 36, decoration: const BoxDecoration(color: kMint, shape: BoxShape.circle), child: CustomPaint(painter: _PlayP(_play.isAnimating))),
        ),
        const SizedBox(width: 6),
        if (!narrow) arrow('ease-prev', '‹', -1),
        Expanded(child: _intervals(narrow)),
        if (!narrow) arrow('ease-next', '›', 1),
      ]),
    );
  }

  Widget _values(double w) {
    Widget cell(String l, double v) => Expanded(
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(l, style: sans(9.5, c: mixed ? kMuted : const Color(0xB31B1B1D), w: FontWeight.w700, ls: .6)),
            const SizedBox(height: 3),
            Text(_n(v), softWrap: false, style: sans(22, c: mixed ? kMuted : const Color(0xFF1B1B1D), w: FontWeight.w600, ls: -.4)),
          ]),
        );
    Widget block(String a, double va, String b, double vb) => Container(
          padding: const EdgeInsets.fromLTRB(12, 9, 8, 10),
          decoration: BoxDecoration(color: mixed ? kRaised : kYellow, borderRadius: BorderRadius.circular(6)),
          child: Row(children: [cell(a, va), cell(b, vb)]),
        );
    final b1 = block('X1', cur.x1, 'Y1', cur.y1), b2 = block('X2', cur.x2, 'Y2', cur.y2);
    return w >= 230 ? Row(children: [Expanded(child: b1), const SizedBox(width: 6), Expanded(child: b2)]) : Column(children: [b1, const SizedBox(height: 6), b2]);
  }

  // Presets: shape only. Hover or arrow keys peek the shape on the plot; Enter or click applies.
  Widget _presetRow() => Focus(
        focusNode: _presetFocus,
        onFocusChange: (f) { if (!f) setState(() => peek = null); },
        onKeyEvent: (_, e) {
          if (e is! KeyDownEvent) return KeyEventResult.ignored;
          final k = e.logicalKey;
          final n = presets.length;
          if (k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowDown) { setState(() { keyFocus = (keyFocus + 1) % n; peek = keyFocus; }); }
          else if (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowUp) { setState(() { keyFocus = (keyFocus + n - 1) % n; peek = keyFocus; }); }
          else if (k == LogicalKeyboardKey.home) { setState(() { keyFocus = 0; peek = 0; }); }
          else if (k == LogicalKeyboardKey.end) { setState(() { keyFocus = n - 1; peek = n - 1; }); }
          else if (k == LogicalKeyboardKey.enter) { setState(() { _apply(keyFocus); peek = null; }); }
          else if (k == LogicalKeyboardKey.escape) { setState(() => peek = null); }
          else { return KeyEventResult.ignored; }
          return KeyEventResult.handled;
        },
        child: Row(children: [
          for (final (i, p) in presets.indexed) ...[
            Expanded(
              child: MouseRegion(
                onEnter: (_) => setState(() => peek = i),
                onExit: (_) => setState(() { if (peek == i) peek = null; }),
                child: GestureDetector(
                  key: ValueKey('ease-preset-$i'),
                  onTap: () { _presetFocus.requestFocus(); setState(() { keyFocus = i; _apply(i); }); },
                  child: Container(
                    height: 42,
                    margin: EdgeInsets.only(right: i == presets.length - 1 ? 0 : 5),
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(color: _presetColors[i % _presetColors.length], borderRadius: BorderRadius.circular(5), border: !mixed && cur.p == i ? Border.all(color: kInk, width: 2.4) : null),
                    child: CustomPaint(size: Size.infinite, painter: _Icon(p.shape, const Color(0xFF1B1B1D), 2.4)),
                  ),
                ),
              ),
            ),
          ],
        ]),
      );

  void _apply(int i) {
    if (mixed) { for (final s in segs) { s.choose(i); } } else { cur.choose(i); }
    _write();
  }

  Widget _savedRow() => widget.host != null ? _keptRow(widget.host!) : Wrap(spacing: 5, runSpacing: 5, children: [
        for (final (i, v) in saved.indexed)
          GestureDetector(
            key: ValueKey('ease-saved-$i'),
            onTap: () {
              setState(() { if (mixed) { for (final s in segs) { s.setValues(v); } } else { cur.setValues(v); } });
              _write();
            },
            child: Container(width: 46, height: 34, padding: const EdgeInsets.all(8), decoration: BoxDecoration(color: kRaised, borderRadius: BorderRadius.circular(4)), child: CustomPaint(size: Size.infinite, painter: _Icon(bezierShape(v[0], v[1], v[2], v[3]), kMint, 2))),
          ),
        _chip('ease-copy', 'Copy curve', () => setState(() => saved.add(List.of(cur.values)))),
        if (saved.isNotEmpty) _chip('ease-clear', 'Clear', () => setState(saved.clear)),
      ]);

  /// Hosted: the host's kept curves (not the demo ones) and what can be done with the current one.
  Widget _keptRow(EaseHost h) => Wrap(spacing: 5, runSpacing: 5, children: [
        for (final (i, s) in h.saved.indexed)
          GestureDetector(
            key: ValueKey('ease-saved-$i'),
            onTap: () {
              void take(Seg to) { to.p = s.p; if (s.bez) to.setValues(s.values); }
              setState(() { if (mixed) { for (final t in segs) { take(t); } } else { take(cur); } });
              _write();
            },
            child: Container(width: 46, height: 34, padding: const EdgeInsets.all(8), decoration: BoxDecoration(color: kRaised, borderRadius: BorderRadius.circular(4)), child: CustomPaint(size: Size.infinite, painter: _Icon(s.shape, kMint, 2))),
          ),
        _chip('ease-copy', 'Copy curve', () => h.copyCurve(cur)),
        _chip('ease-save', 'Save preset', () => h.savePreset(cur)),
        _chip('ease-new-keys', 'Use for new keys', () => h.useForNewKeys(cur)),
        if (h.saved.isNotEmpty) _chip('ease-clear', 'Clear', h.clearSaved),
      ]);

  Widget _chip(String key, String t, VoidCallback f) => GestureDetector(
        key: ValueKey(key),
        onTap: f,
        child: Container(height: 34, padding: const EdgeInsets.symmetric(horizontal: 12), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: const Color(0xFF3A3B40)), borderRadius: BorderRadius.circular(4)), child: Text(t, style: sans(11, c: const Color(0xFFC4C6CB)))),
      );

  /// Hosted, the switch says whether the desk is in its ghost mode (set by what is picked); unhosted it is a toy.
  bool get _ghost => widget.host == null ? ghost : widget.host!.sequence > 1;

  Widget _options() => Row(children: [
        Expanded(child: GestureDetector(
          key: const ValueKey('ease-seq'),
          behavior: HitTestBehavior.opaque,
          onTap: widget.host == null ? () => setState(() => ghost = !ghost) : null,
          child: Row(children: [
            Container(width: 34, height: 20, padding: const EdgeInsets.all(2), alignment: _ghost ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: _ghost ? kBlue : const Color(0xFF34353A), borderRadius: BorderRadius.circular(10)), child: Container(width: 16, height: 16, decoration: const BoxDecoration(color: kInk, shape: BoxShape.circle))),
            const SizedBox(width: 10),
            Flexible(child: Text('Sequence ghosts', softWrap: false, overflow: TextOverflow.ellipsis, style: sans(11.5, c: const Color(0xFFC4C6CB)))),
          ]),
        )),
      ]);

  // Intervals between the selected keyframes: width = duration, each its own colour.
  Widget _intervals(bool compact) => Row(children: [
        GestureDetector(
          key: const ValueKey('ease-all'),
          onTap: () => setState(() => sel = -1),
          child: Container(
            width: compact ? 24 : 34,
            height: 36,
            margin: const EdgeInsets.only(right: 4),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: mixed ? kYellow : kRaised, borderRadius: BorderRadius.circular(5)),
            child: Text(compact ? '≠' : 'All', style: sans(10.5, c: mixed ? const Color(0xFF1B1B1D) : kMuted, w: FontWeight.w600)),
          ),
        ),
        for (final (i, sg) in segs.indexed)
          Expanded(
            flex: sg.frames,
            child: GestureDetector(
              key: ValueKey('ease-seg-$i'),
              onTap: () => setState(() => sel = i),
              child: Container(
                height: 36,
                margin: EdgeInsets.only(right: i == segs.length - 1 ? 0 : 4),
                padding: EdgeInsets.symmetric(horizontal: compact ? 2 : 5, vertical: 6),
                decoration: BoxDecoration(color: _segColors[i % _segColors.length], borderRadius: BorderRadius.circular(5), border: sel == i ? Border.all(color: kInk, width: 2.4) : null),
                child: Row(children: [
                  if (!compact && sg.frames >= 8) Text('${sg.frames}f', style: sans(10, c: const Color(0xFF1B1B1D), w: FontWeight.w700)),
                  if (!compact && sg.frames >= 8) const SizedBox(width: 4),
                  Expanded(child: CustomPaint(size: Size.infinite, painter: _Icon(sg.shape, const Color(0xFF1B1B1D), 1.8))),
                ]),
              ),
            ),
          ),
      ]);

  Widget _plot({required bool labels}) => LayoutBuilder(builder: (context, box) {
        final size = Size(box.maxWidth, box.maxHeight);
        final startFrame = mixed ? 0 : segs.take(sel).fold<int>(0, (a, b) => a + b.frames);
        final shown = peek == null ? cur.shape : presets[peek!].shape;
        final painter = _PlotP(
          shown, mixed && peek == null ? segs.map((e) => e.shape).toList() : const [], cur.x1, cur.y1, cur.x2, cur.y2,
          peek == null && !mixed && cur.bez, labels, peek == null ? presets[cur.p].name : '${presets[peek!].name}  ·  peek', mixed && peek == null, startFrame,
          mixed ? segs.fold<int>(0, (a, b) => a + b.frames) : cur.frames, widget.host == null && ghost,
          // the preview's motion while it plays; otherwise, hosted, where the document playhead stands in the interval
          _play.isAnimating || _play.value > 0 ? _play.value : (widget.host?.playhead(mixed ? null : sel) ?? .5),
          layers: widget.host?.sequence ?? 0,
          caption: widget.host?.caption,
          meaning: widget.host == null || peek != null ? null : curveMeaning(presets[cur.p].name),
        );
        void pick(Offset p) {
          if (mixed || !cur.bez || peek != null) return;
          final h = [easeAt(size, cur.x1, cur.y1), easeAt(size, cur.x2, cur.y2)];
          var best = -1;
          var bd = 28.0;
          for (var i = 0; i < 2; i++) {
            final dd = (h[i] - p).distance;
            if (dd < bd) { bd = dd; best = i; }
          }
          if (best >= 0) {
            _before = (cur.p, List.of(cur.values));
            _plotKeys.requestFocus();
            setState(() => drag = best);
          }
        }
        void move(Offset p) {
          if (drag == null) return;
          final r = easePlotBox(size);
          final x = clampD((p.dx - r.left) / r.width, 0, 1);
          final y = clampD((r.bottom - p.dy) / r.height * 1.3 - .15, -.3, 1.3);
          setState(() {
            _moved = true;
            cur.p = presets.indexWhere((k) => k.name == 'Bezier').clamp(0, presets.length - 1);
            if (drag == 0) { cur.x1 = x; cur.y1 = y; } else { cur.x2 = x; cur.y2 = y; }
          });
          widget.host?.preview(mixed ? null : sel, cur);
        }
        // Raw pointers: a handle drag must not lose to the panel's own vertical scroll. Esc puts the curve back.
        return Focus(
          focusNode: _plotKeys,
          onKeyEvent: (_, e) {
            if (e is! KeyDownEvent || e.logicalKey != LogicalKeyboardKey.escape || drag == null) return KeyEventResult.ignored;
            final b = _before;
            setState(() {
              if (b != null) { cur.setValues(b.$2); cur.p = b.$1; }
              drag = null;
              _moved = false;
            });
            widget.host?.cancel();
            return KeyEventResult.handled;
          },
          child: Listener(
          key: const ValueKey('ease-plot'),
          onPointerDown: (e) => pick(e.localPosition),
          onPointerMove: (e) => move(e.localPosition),
          onPointerUp: (_) {
            final wrote = drag != null && _moved;
            _moved = false;
            setState(() => drag = null);
            if (wrote) _write();
          },
          onPointerCancel: (_) {
            if (drag != null && _moved) widget.host?.cancel();
            _moved = false;
            setState(() => drag = null);
          },
          child: CustomPaint(size: size, painter: painter),
        ));
      });

  final _plotKeys = FocusNode(debugLabel: 'ease plot');
  (int, List<double>)? _before;
}

class _PlayP extends CustomPainter {
  _PlayP(this.playing);
  final bool playing;
  @override
  void paint(Canvas c, Size s) {
    final m = s.center(Offset.zero);
    final p = Paint()..color = const Color(0xFF1B1B1D);
    if (playing) {
      c.drawRect(Rect.fromCenter(center: m + const Offset(-3.5, 0), width: 3.4, height: 11), p);
      c.drawRect(Rect.fromCenter(center: m + const Offset(3.5, 0), width: 3.4, height: 11), p);
    } else {
      c.drawPath(Path()..moveTo(m.dx - 4, m.dy - 6)..lineTo(m.dx + 6, m.dy)..lineTo(m.dx - 4, m.dy + 6)..close(), p);
    }
  }
  @override
  bool shouldRepaint(_PlayP o) => o.playing != playing;
}

class _Icon extends CustomPainter {
  _Icon(this.f, this.col, this.w);
  final Shape f;
  final Color col;
  final double w;
  @override
  void paint(Canvas c, Size s) {
    final path = Path();
    for (var k = 0; k <= 40; k++) {
      final t = k / 40;
      final v = f(t).clamp(-.3, 1.3);
      final o = Offset(s.width * t, s.height - s.height * (v + .15) / 1.3);
      k == 0 ? path.moveTo(o.dx, o.dy) : path.lineTo(o.dx, o.dy);
    }
    c.drawPath(path, Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = w..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  }
  @override
  bool shouldRepaint(_Icon o) => true;
}

class _PlotP extends CustomPainter {
  _PlotP(this.f, this.all, this.x1, this.y1, this.x2, this.y2, this.editable, this.labels, this.name, this.mixed, this.f0, this.frames, this.ghost, this.head, {this.layers = 0, this.caption, this.meaning});

  /// The corner label when it is not the frames (a sequence's layers, the workspace); the curve's one-line meaning.
  final String? caption, meaning;

  /// Layers the curve spreads delays over: a mark where each one's delay falls (i / (n - 1) through the curve).
  final int layers;
  final Shape f;
  final List<Shape> all;
  final double x1, y1, x2, y2, head;
  final bool editable, labels, mixed, ghost;
  final String name;
  final int f0, frames;

  Rect box(Size s) => labels ? easePlotBox(s) : Rect.fromLTRB(6, 6, s.width - 26, s.height - 6);
  Offset at(Size s, double x, double y) {
    final r = box(s);
    return Offset(r.left + r.width * x, r.bottom - r.height * (y + .15) / 1.3);
  }

  void _text(Canvas c, String t, Offset o, TextStyle st, {bool right = false, bool centre = false}) {
    final tp = TextPainter(text: TextSpan(text: t, style: st), textDirection: TextDirection.ltr)..layout();
    tp.paint(c, Offset(right ? o.dx - tp.width : (centre ? o.dx - tp.width / 2 : o.dx), o.dy));
  }

  Path _curve(Size s, Shape fn, {double dx = 0, double squeeze = 1}) {
    final path = Path();
    for (var k = 0; k <= 90; k++) {
      final t = k / 90;
      final o = at(s, t * squeeze + dx, fn(t));
      k == 0 ? path.moveTo(o.dx, o.dy) : path.lineTo(o.dx, o.dy);
    }
    return path;
  }

  @override
  void paint(Canvas c, Size s) {
    final r = box(s);
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(6)), Paint()..color = const Color(0xFF131316));
    final base = at(s, 0, 0).dy, top = at(s, 0, 1).dy;
    // only the two rails the curve travels between; no grid
    c.drawLine(Offset(r.left, base), Offset(r.right, base), Paint()..color = const Color(0xFF2C2D32)..strokeWidth = 1.2);
    c.drawLine(Offset(r.left, top), Offset(r.right, top), Paint()..color = const Color(0xFF2C2D32)..strokeWidth = 1.2);
    // the silhouette of the curve is the object: a flat colour field under it
    if (!mixed) {
      final area = _curve(s, f)..lineTo(at(s, 1, 0).dx, base)..lineTo(at(s, 0, 0).dx, base)..close();
      c.drawPath(area, Paint()..shader = LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: const [kPink, kViolet, kBlue], stops: const [0, .5, 1]).createShader(Rect.fromLTRB(r.left, top, r.right, base)));
    }
    if (layers > 1 && !mixed) {
      for (var i = 0; i < layers; i++) {
        final x = i / (layers - 1);
        c.drawCircle(at(s, x, f(x)), 4, Paint()..color = kInk);
      }
    }
    if (ghost && !mixed) {
      for (var k = 3; k >= 1; k--) {
        c.drawPath(_curve(s, f, dx: .06 * k, squeeze: 1 - .06 * k), Paint()..color = kInk.withValues(alpha: .40 - .09 * k)..style = PaintingStyle.stroke..strokeWidth = 2);
      }
    }
    if (mixed) {
      final cols = [kViolet, kMint, kPink];
      for (final (i, fn) in all.indexed) {
        final p = _curve(s, fn);
        c.drawPath(p, Paint()..color = cols[i]..style = PaintingStyle.stroke..strokeWidth = 3.4..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
      }
    }
    // playhead: white line, flag on top
    final ph = r.left + r.width * head;
    c.drawLine(Offset(ph, r.top - 2), Offset(ph, r.bottom), Paint()..color = kInk..strokeWidth = 1.6);
    c.drawPath(Path()..moveTo(ph - 6, r.top - 12)..lineTo(ph + 6, r.top - 12)..lineTo(ph, r.top - 2)..close(), Paint()..color = kInk);
    // handles: yellow, the touchable thing
    if (editable) {
      final h = Paint()..color = kYellow..strokeWidth = 2;
      c.drawLine(at(s, 0, 0), at(s, x1, y1), h);
      c.drawLine(at(s, 1, 1), at(s, x2, y2), h);
      for (final o in [at(s, x1, y1), at(s, x2, y2)]) {
        c.drawCircle(o, 12, Paint()..color = kYellow.withValues(alpha: .22));
        c.drawCircle(o, 7, Paint()..color = kYellow);
        c.drawCircle(o, 7, Paint()..color = const Color(0xFF131316)..style = PaintingStyle.stroke..strokeWidth = 2);
      }
    }
    if (!mixed) c.drawPath(_curve(s, f), Paint()..color = kInk..style = PaintingStyle.stroke..strokeWidth = 3.4..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
    for (final o in [at(s, 0, 0), at(s, 1, 1)]) {
      c.drawCircle(o, 7, Paint()..color = kInk);
    }
    final v = f(head);
    final pt = at(s, head, v);
    // motion strip: where the value lands at even steps, read off the right edge
    final cx = r.right + 16;
    for (var i = 0; i <= 8; i++) { c.drawCircle(Offset(cx, at(s, 0, f(i / 8)).dy), 2.8, Paint()..color = kInk.withValues(alpha: .55)); }
    c.drawLine(pt, Offset(cx, pt.dy), Paint()..color = kMint..strokeWidth = 1.6);
    c.drawCircle(Offset(cx, pt.dy), 5.4, Paint()..color = kMint);
    if (!mixed) { c.drawCircle(pt, 6, Paint()..color = kMint); c.drawCircle(pt, 6, Paint()..color = const Color(0xFF131316)..style = PaintingStyle.stroke..strokeWidth = 2); }
    if (labels) {
      _text(c, mixed ? 'Mixed' : name, Offset(r.left, 9), sans(15, c: kInk, w: FontWeight.w600));
      _text(c, caption ?? (layers > 1 ? '$layers layers' : (mixed ? '3 intervals · $frames f' : '$frames f')), Offset(s.width - 14, 12), sans(11.5, c: const Color(0xFFB4B6BB)), right: true);
      if (meaning != null) _text(c, meaning!, Offset(r.left, 28), sans(10.5, c: const Color(0xFFB4B6BB)));
      _text(c, 'f$f0', Offset(r.left, s.height - 17), mono(9.5));
      _text(c, 'f${f0 + frames}', Offset(r.right, s.height - 17), mono(9.5), right: true);
    }
  }
  @override
  bool shouldRepaint(_PlotP o) => true;
}
