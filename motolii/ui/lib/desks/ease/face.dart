import 'dart:math' as math;
import 'package:flutter/animation.dart' show AnimationController;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../../controls/panel/toggles.dart' show EditorSwitch;
import '../../browser/parts.dart';
import '../parts.dart';
import 'meaning.dart';
import '../../theme/neutral.dart';
import '../../theme/metrics.dart' show Dn, Surface;
import '../../theme/material_icons.dart' show Glyph;

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

/// A curve read off its samples ([u, value] pairs, u rising 0–1), straight between them.
Shape sampledShape(List samples) {
  final pts = [for (final p in samples) if (p is List && p.length >= 2) ((p[0] as num).toDouble(), (p[1] as num).toDouble())];
  if (pts.length < 2) return (t) => t;
  return (t) {
    for (var i = 1; i < pts.length; i++) {
      final (u1, v1) = pts[i];
      if (t <= u1) {
        final (u0, v0) = pts[i - 1];
        return u1 == u0 ? v1 : v0 + (v1 - v0) * (t - u0) / (u1 - u0);
      }
    }
    return pts.last.$2;
  };
}

class Preset {
  const Preset(this.name, this.bez, this.fn, {this.model});
  final String name;
  final List<double>? bez;
  final Shape? fn;

  /// A hosted kind's own description (its parameters, samples and handle points), for kinds other than Bezier.
  final Map<String, dynamic>? model;
  Shape get shape => bez != null ? bezierShape(bez![0], bez![1], bez![2], bez![3]) : fn!;
}

final easePresets = <Preset>[
  const Preset('Linear', [0, 0, 1, 1], null),
  const Preset('Ease', [.42, 0, .58, 1], null),
  const Preset('Bezier', [.25, .1, .78, .92], null),
  Preset('Spring', null, (t) => 1 - math.exp(-6 * t) * math.cos(t * math.pi * 5)),
  const Preset('Bounce', null, _bounce),
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

  /// The host's description of this curve when it is not a Bezier (parameters, samples, handle points).
  late Map<String, dynamic>? model = kinds[p].model;
  bool get bez => kinds[p].bez != null;
  Shape get shape => bez ? bezierShape(x1, y1, x2, y2) : (model?['samples'] is List ? sampledShape(model!['samples'] as List) : kinds[p].fn!);
  List<double> get values => [x1, y1, x2, y2];

  /// Where the curve can be grabbed: a Bezier's two handles, else the points the host names (none for Hold, Linear).
  List<Offset> get handles => bez
      ? [Offset(x1, y1), Offset(x2, y2)]
      : [for (final h in (model?['handles'] as List? ?? const [])) if (h is List && h.length >= 2) Offset((h[0] as num).toDouble(), (h[1] as num).toDouble())];
  void choose(int i) {
    p = i;
    model = kinds[i].model;
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

  /// A non-Bezier curve with [handle] moved to [point] (or, with none, its parameters as they are now): the host's new
  /// description of it (Classic's easeModel), or null when it refuses.
  Future<Map<String, dynamic>?> model(Seg curve, int? handle, Offset? point);

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

  /// Moves when the document playhead does (the graph's line follows it without rebuilding the desk).
  Listenable get frame;

  /// Whether there are saved presets to clear (the copied curve is not one).
  bool get canClear;

  /// How many layers the curve spreads delays over (the Ease desk's ghost mode: several layers picked, no keys);
  /// 0 when it shapes key intervals.
  int get sequence;
}

Rect easePlotBox(Size s) => Rect.fromLTRB(14, 34, s.width - 34, s.height - 22);
Offset easeAt(Size s, double x, double y) {
  final r = easePlotBox(s);
  return Offset(r.left + r.width * x, r.bottom - r.height * (y + .15) / 1.3);
}


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
      // the interval shown follows the playhead when the playhead enters another one (Classic's desk); a chip the
      // user picks stays until then
      final now = h.current;
      if (now != null && now != _followed && sel >= 0 && now < _held.length) sel = now;
      _followed = now;
    });
  }

  int? _followed;
  void _write() => widget.host?.apply(mixed ? null : sel, cur);
  final saved = <List<double>>[[.7, 0, .3, 1], [.2, .9, .6, 1.2]];
  int sel = 0; // -1 = all intervals (mixed)
  bool ghost = false;
  bool _overOK = true;
  int? drag;
  bool _moved = false;
  int? peek; // preset under the pointer or the keyboard: shown on the plot until it leaves
  int keyFocus = 2;
  final _presetFocus = FocusNode();
  /// Local preview: while pressed, the plot's playhead slides 0→1 across the current curve.
  late final AnimationController _play = AnimationController(vsync: this, duration: const Duration(milliseconds: 1400));

  void _togglePlay() {
    final p = _play;
    setState(() {
      if (p.isAnimating) p.stop();
      else p.forward(from: 0);
    });
  }
  

  @override
  void dispose() {
    widget.host?.removeListener(_absorb);
    _play.dispose();
    
    _presetFocus.dispose();
    _plotKeys.dispose();
    _typed.dispose();
    _typedFocus.dispose();
    super.dispose();
  }

  Seg get cur => segs.isEmpty ? Seg(0, 1, presets) : segs[sel < 0 ? 0 : sel];
  bool get mixed => sel < 0;
  String _n(double v) => mixed ? '—' : v.toStringAsFixed(2);

  @override
  Widget build(BuildContext context) => DeskShell(
        kind: DeskKind.ease,
        title: 'Ease',
        alwaysShowTrailing: true,
        subtitle: widget.host?.target ?? 'KEYFRAMES · MOTION',
        trailing: Row(mainAxisSize: MainAxisSize.min, children: [
          EditorSwitch(
            on: _overOK,
            glyph: _overOK ? Glyph.check_box_outlined : Glyph.check_box_outline_blank,
            label: 'Allow values outside 0–1',
            compact: true,
            tint: kBlue,
            onChanged: (v) => setState(() => _overOK = v),
          ),
          Text('Over OK', style: sans(Dn.microSize, c: Surface.muted)),
        ]),
        full: (c, s) => _body(s.height, s.width),
        strip: (c, s) => _body(s.height, s.width),
        tall: (c, s) => _body(s.height, s.width),
      );

  // Primary loop (interval -> preset -> curve -> play) always shares one viewport.
  // Precision, saved curves and options are secondary and are the only thing that scrolls.
  Widget _body(double h, double w) {
    final pad = w < 230 ? Surface.px(8.0) : Surface.px(12.0);
    if (h < 200) {
      // a short strip: the face first, everything else one scroll away
      return SingleChildScrollView(
        key: const ValueKey('ease-scroll'),
        physics: drag != null ? const NeverScrollableScrollPhysics() : null,
        padding: EdgeInsets.fromLTRB(pad, Surface.panelInset, pad, Surface.px(12)),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          SizedBox(height: math.max(56, h - 20), child: _plot(labels: false)),
          SizedBox(height: Surface.sectionGap),
          _presetRow(),
          _secondary(w),
        ]),
      );
    }
    final plotH = (h * .46).clamp(110.0, 300.0);
    return Padding(
      padding: EdgeInsets.fromLTRB(pad, Surface.panelInset, pad, 0),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        SizedBox(height: plotH, child: _plot(labels: w >= 230 && plotH >= 150)),
        SizedBox(height: Surface.px(7.5)),
        _presetRow(),
        Expanded(child: SingleChildScrollView(key: const ValueKey('ease-scroll'), padding: EdgeInsets.only(bottom: Surface.px(12)), child: _secondary(w))),
      ]),
    );
  }

  Widget _secondary(double w) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _section('VALUES', _values(w)),
        _section('SAVED', _savedRow()),
        _section('OPTIONS', _options()),
      ]);

  Widget _section(String t, Widget child) => Padding(
        padding: EdgeInsets.only(top: Surface.px(12)),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Text(t, style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w500, ls: 1.4)),
          SizedBox(height: Surface.sectionGap),
          child,
        ]),
      );

  

  /// The curve's numbers (Classic DK-018): a Bezier's four, another kind's own parameters (re-described by the host).
  List<(String, double, void Function(double))> _params() {
    if (cur.bez || widget.host == null) {
      return [
        ('X1', cur.x1, (v) => cur.x1 = clampD(v, 0, 1)),
        ('Y1', cur.y1, (v) => cur.y1 = _overOK ? v : clampD(v, 0, 1)),
        ('X2', cur.x2, (v) => cur.x2 = clampD(v, 0, 1)),
        ('Y2', cur.y2, (v) => cur.y2 = _overOK ? v : clampD(v, 0, 1)),
      ].map((e) => (e.$1, e.$2, (double v) { e.$3(v); cur.p = cur.kinds.indexWhere((k) => k.bez != null && k.name == 'Bezier').clamp(0, cur.kinds.length - 1); })).toList();
    }
    final m = cur.model ?? const <String, dynamic>{};
    return [
      for (final e in m.entries)
        if (e.value is num && e.key != 'overshoots')
          (
            e.key.replaceAllMapped(RegExp(r'([a-z])([A-Z])'), (x) => '${x[1]} ${x[2]}').toUpperCase(),
            (e.value as num).toDouble(),
            (double v) {
              final seg = cur, ask = ++_asks;
              seg.model = {...m, e.key: v};
              widget.host?.model(seg, null, null).then((next) {
                if (next == null || !mounted || ask != _asks) return;
                setState(() => seg.model = next);
              });
            },
          ),
    ];
  }

  String? _typing; // the number being typed
  final _typed = TextEditingController();
  final _typedFocus = FocusNode(debugLabel: 'ease number');

  Widget _values(double w) {
    final params = _params();
    if (params.isEmpty) return Text('This curve has no numbers.', style: sans(Dn.nameSize, c: Surface.muted));
    Widget cell((String, double, void Function(double)) p) {
      final (l, v, set) = p;
      final typing = _typing == l;
      return Expanded(
        child: GestureDetector(
          key: ValueKey('ease-value-$l'),
          behavior: HitTestBehavior.opaque,
          onTap: mixed ? null : () => setState(() {
            _typing = l;
            _typed.text = v.toStringAsFixed(2);
            _typedFocus.requestFocus();
          }),
          onHorizontalDragUpdate: mixed ? null : (d) {
            setState(() => set(v + d.delta.dx * .005));
            widget.host?.preview(sel, cur);
          },
          onHorizontalDragEnd: mixed ? null : (_) => _write(),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(l, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.microSize, c: mixed ? Surface.muted : N.inkSoft, w: FontWeight.w700, ls: .6)),
            SizedBox(height: Surface.px(2)),
            typing
                ? Focus(
                    onKeyEvent: (_, e) {
                      if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
                        setState(() => _typing = null);
                        return KeyEventResult.handled;
                      }
                      return KeyEventResult.ignored;
                    },
                    child: EditableText(
                      controller: _typed,
                      focusNode: _typedFocus,
                      style: sans(Dn.numericSize, c: N.g10, w: FontWeight.w600, ls: -.4),
                      cursorColor: N.g10,
                      backgroundCursorColor: Surface.muted,
                      onSubmitted: (t) {
                        final next = double.tryParse(t.trim());
                        setState(() {
                          _typing = null;
                          if (next != null) set(next);
                        });
                        if (next != null) _write();
                      },
                    ),
                  )
                : Text(_n(v), softWrap: false, style: sans(Dn.numericSize, c: mixed ? Surface.muted : N.g10, w: FontWeight.w600, ls: -.4)),
          ]),
        ),
      );
    }
    Widget block(List<(String, double, void Function(double))> two) => Container(
          padding: EdgeInsets.fromLTRB(Surface.panelInset, Surface.px(7), Surface.sectionGap, Surface.px(7.5)),
          decoration: BoxDecoration(color: mixed ? Surface.raised : kYellow, borderRadius: BorderRadius.circular(Surface.faceRadius)),
          child: Row(children: [for (final p in two) cell(p), if (two.length == 1) const Expanded(child: SizedBox())]),
        );
    final blocks = [for (var i = 0; i < params.length; i += 2) block(params.sublist(i, math.min(i + 2, params.length)))];
    if (w >= 230) {
      return Column(children: [
        for (var i = 0; i < blocks.length; i += 2) ...[
          if (i > 0) SizedBox(height: Surface.px(4.5)),
          Row(children: [Expanded(child: blocks[i]), SizedBox(width: Surface.px(4.5)), Expanded(child: i + 1 < blocks.length ? blocks[i + 1] : const SizedBox())]),
        ],
      ]);
    }
    return Column(children: [for (final (i, b) in blocks.indexed) ...[if (i > 0) SizedBox(height: Surface.px(4.5)), b]]);
  }

  List<int> get _shown => [for (final (i, _) in presets.indexed) i];

  // Presets: shape only. Arrow keys peek the shape on the plot; Enter or click applies.
  Widget _presetRow() => Focus(
        focusNode: _presetFocus,
        onFocusChange: (f) { if (!f) setState(() => peek = null); },
        onKeyEvent: (_, e) {
          if (e is! KeyDownEvent) return KeyEventResult.ignored;
          final s = _shown;
          if (s.isEmpty) return KeyEventResult.ignored;
          final at = (int dir) {
            var i = s.indexOf(keyFocus);
            if (i < 0) i = 0;
            return s[(i + dir) % s.length];
          };
          final k = e.logicalKey;
          if (k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowDown) { final f = at(1); setState(() { keyFocus = f; peek = f; }); }
          else if (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowUp) { final f = at(-1); setState(() { keyFocus = f; peek = f; }); }
          else if (k == LogicalKeyboardKey.home) { final f = s.first; setState(() { keyFocus = f; peek = f; }); }
          else if (k == LogicalKeyboardKey.end) { final f = s.last; setState(() { keyFocus = f; peek = f; }); }
          else if (k == LogicalKeyboardKey.enter) { setState(() { _apply(keyFocus); peek = null; }); }
          else if (k == LogicalKeyboardKey.escape) { setState(() => peek = null); }
          else { return KeyEventResult.ignored; }
          return KeyEventResult.handled;
        },
        child: Row(children: [
          _playButton(-1),
          Expanded(child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(children: [
          for (final (i, p) in presets.indexed)
            ...[
              SizedBox(
                width: Surface.faceTile + Surface.sectionGap,
                child: GestureDetector(
                  key: ValueKey('ease-preset-$i'),
                  onTap: () { _presetFocus.requestFocus(); setState(() { keyFocus = i; _apply(i); }); },
                  child: Container(
                    height: Surface.faceTile,
                    margin: EdgeInsets.only(right: Surface.sectionGap),
                    padding: EdgeInsets.all(Surface.sectionGap),
                    // a quiet tile with the curve in its colour; the chosen one takes a tint and an edge of it
                    decoration: BoxDecoration(color: !mixed && cur.p == i ? _presetColors[i % _presetColors.length].withValues(alpha: .18) : (peek == i ? N.g15 : N.g13), borderRadius: BorderRadius.circular(Surface.px(4)), border: Border.all(color: !mixed && cur.p == i ? _presetColors[i % _presetColors.length] : N.g20, width: !mixed && cur.p == i ? Surface.px(1.4) : 1)),
                    child: CustomPaint(size: Size.infinite, painter: _Icon(p.shape, _presetColors[i % _presetColors.length], 2)),
                  ),
                ),
              ),
            ],
            ]),
          )),
        ]),
      );

  /// Previews the current curve (its playhead runs 0→1).
  Widget _playButton(int i) => SizedBox(
        width: Surface.faceTile + Surface.sectionGap,
        child: ListenableBuilder(
          listenable: _play,
          builder: (context, _) {
            final on = _play.isAnimating;
            return GestureDetector(
              key: const ValueKey('ease-play'),
              onTap: _togglePlay,
              child: Container(
                height: Surface.faceTile,
                margin: EdgeInsets.only(right: Surface.sectionGap),
                padding: EdgeInsets.all(Surface.sectionGap),
                decoration: BoxDecoration(color: on ? kBlue : N.g13, borderRadius: BorderRadius.circular(Surface.px(4)), border: Border.all(color: on ? kBlue : N.g20, width: on ? Surface.px(1.4) : 1)),
                child: Icon(Glyph.play_arrow_outlined, color: on ? N.g10 : N.g76, size: Surface.px(18)),
              ),
            );
          },
        ),
      );

  void _apply(int i) {
    if (mixed) { for (final s in segs) { s.choose(i); } } else { cur.choose(i); }
    _write();
  }

  Widget _savedRow() => widget.host != null ? _keptRow(widget.host!) : Wrap(spacing: Surface.px(4), runSpacing: Surface.px(4), children: [
        for (final (i, v) in saved.indexed)
          GestureDetector(
            key: ValueKey('ease-saved-$i'),
            onTap: () {
              setState(() { if (mixed) { for (final s in segs) { s.setValues(v); } } else { cur.setValues(v); } });
              _write();
            },
            child: Container(width: Surface.px(30), height: Surface.controlHero, padding: EdgeInsets.all(Surface.sectionGap), decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(Surface.controlRadius)), child: CustomPaint(size: Size.infinite, painter: _Icon(bezierShape(v[0], v[1], v[2], v[3]), kMint, 2))),
          ),
        _chip('ease-copy', 'Copy curve', () => setState(() => saved.add(List.of(cur.values)))),
        if (saved.isNotEmpty) _chip('ease-clear', 'Clear', () => setState(saved.clear)),
      ]);

  /// Hosted: the host's kept curves (not the demo ones) and what can be done with the current one.
  Widget _keptRow(EaseHost h) => Wrap(spacing: Surface.px(4), runSpacing: Surface.px(4), children: [
        for (final (i, s) in h.saved.indexed)
          GestureDetector(
            key: ValueKey('ease-saved-$i'),
            onTap: () {
              void take(Seg to) { to.p = s.p; if (s.bez) { to.setValues(s.values); } else { to.model = s.model; } }
              setState(() { if (mixed) { for (final t in segs) { take(t); } } else { take(cur); } });
              _write();
            },
            child: Container(width: Surface.px(30), height: Surface.controlHero, padding: EdgeInsets.all(Surface.sectionGap), decoration: BoxDecoration(color: Surface.raised, borderRadius: BorderRadius.circular(Surface.controlRadius)), child: CustomPaint(size: Size.infinite, painter: _Icon(s.shape, kMint, 2))),
          ),
        _chip('ease-copy', 'Copy curve', () => h.copyCurve(cur)),
        _chip('ease-save', 'Save preset', () => h.savePreset(cur)),
        _chip('ease-new-keys', 'Use for new keys', () => h.useForNewKeys(cur)),
        if (h.canClear) _chip('ease-clear', 'Clear', h.clearSaved),
      ]);

  Widget _chip(String key, String t, VoidCallback f) => GestureDetector(
        key: ValueKey(key),
        onTap: f,
        child: Container(height: Surface.control, padding: EdgeInsets.symmetric(horizontal: Surface.panelInset), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: N.g26), borderRadius: BorderRadius.circular(Surface.controlRadius)), child: Text(t, style: sans(Dn.nameSize, c: N.g76))),
      );

  /// Hosted, the switch says whether the desk is in its ghost mode (set by what is picked); unhosted it is a toy.
  bool get _ghost => widget.host == null ? ghost : widget.host!.sequence > 1;

  Widget _options() => Row(children: [
        Expanded(child: GestureDetector(
          key: const ValueKey('ease-seq'),
          behavior: HitTestBehavior.opaque,
          onTap: widget.host == null ? () => setState(() => ghost = !ghost) : null,
          child: Row(children: [
            Container(width: Surface.px(22), height: Surface.px(13), padding: EdgeInsets.all(Surface.px(1.5)), alignment: _ghost ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: _ghost ? kBlue : N.g20, borderRadius: BorderRadius.circular(Surface.px(6.5))), child: Container(width: Surface.px(10), height: Surface.px(10), decoration: const BoxDecoration(color: Surface.ink, shape: BoxShape.circle))),
            SizedBox(width: Surface.px(7.5)),
            Flexible(child: Text('Sequence ghosts', softWrap: false, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: N.g76))),
          ]),
        )),
      ]);

  

  // Only the plot follows the playhead (its line), frame by frame; the rest of the desk is not rebuilt.
  Widget _plot({required bool labels}) {
    final listens = <Listenable>[
      if (widget.host?.frame != null) widget.host!.frame,
      _play,
    ];
    return ListenableBuilder(listenable: Listenable.merge(listens), builder: (context, _) => _plotBody(labels: labels));
  }

  Widget _plotBody({required bool labels}) => LayoutBuilder(builder: (context, box) {
        final size = Size(box.maxWidth, box.maxHeight);
        final startFrame = mixed ? 0 : segs.take(sel).fold<int>(0, (a, b) => a + b.frames);
        final shown = peek == null ? cur.shape : presets[peek!].shape;
        final painter = _PlotP(
          shown, mixed && peek == null ? segs.map((e) => e.shape).toList() : const [], cur.x1, cur.y1, cur.x2, cur.y2,
          peek == null && !mixed && cur.bez, labels, peek == null ? presets[cur.p].name : '${presets[peek!].name}  ·  peek', mixed && peek == null, startFrame,
          mixed ? segs.fold<int>(0, (a, b) => a + b.frames) : cur.frames, widget.host == null && ghost,
          // local preview: the playhead we just pressed; else the document's (static centre when unhosted)
          _play.isAnimating ? _play.value : (widget.host?.playhead(mixed ? null : sel) ?? .5),
          layers: widget.host?.sequence ?? 0,
          caption: widget.host?.caption,
          meaning: widget.host == null || peek != null ? null : curveMeaning(presets[cur.p].name),
          points: peek == null && !mixed && !cur.bez ? cur.handles : const [],
        );
        void pick(Offset p) {
          if (mixed || peek != null || (!cur.bez && widget.host == null)) return;
          final h = [for (final o in cur.handles) easeAt(size, o.dx, o.dy)];
          var best = -1;
          var bd = 28.0;
          for (var i = 0; i < h.length; i++) {
            final dd = (h[i] - p).distance;
            if (dd < bd) { bd = dd; best = i; }
          }
          if (best >= 0) {
            _before = (cur.p, List.of(cur.values));
            _beforeModel = cur.model;
            _plotKeys.requestFocus();
            setState(() => drag = best);
          }
        }
        void move(Offset p) {
          if (drag == null) return;
          final r = easePlotBox(size);
          final x = clampD((p.dx - r.left) / r.width, 0, 1);
          final y = clampD((r.bottom - p.dy) / r.height * 1.3 - .15, _overOK ? -.3 : 0, _overOK ? 1.3 : 1);
          if (!cur.bez) {
            // another kind: the host moves its handle and describes the curve again (latest answer wins)
            final seg = cur, handle = drag!, ask = ++_asks;
            _moved = true;
            _pending = widget.host?.model(seg, handle, Offset(x, y)).then((m) {
              if (m == null || !mounted || ask != _asks) return;
              setState(() => seg.model = m);
              if (drag != null) widget.host?.preview(mixed ? null : sel, seg);
            });
            return;
          }
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
            _asks++; // an answer still on its way is for the drag that was put back
            _pending = null;
            setState(() {
              if (b != null) { cur.setValues(b.$2); cur.p = b.$1; cur.model = _beforeModel; }
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
            // a quick flick of another kind's handle is written once the host has described the curve it made
            final pending = _pending;
            _pending = null;
            // the interval and curve the drag was on, even if another chip is picked before the answer lands
            final at = mixed ? null : sel, seg = cur;
            if (wrote) pending == null ? _write() : pending.then((_) { if (mounted) widget.host?.apply(at, seg); });
          },
          onPointerCancel: (_) {
            _asks++;
            _pending = null;
            if (drag != null && _moved) widget.host?.cancel();
            _moved = false;
            setState(() => drag = null);
          },
          child: CustomPaint(size: size, painter: painter),
        ));
      });

  final _plotKeys = FocusNode(debugLabel: 'ease plot');
  (int, List<double>)? _before;
  Map<String, dynamic>? _beforeModel;
  Future<void>? _pending;
  int _asks = 0;
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
    // surface: scale curve strokes with the tile, with a readable floor and a cap against crowding.
    final stroke = (w * s.shortestSide / 28).clamp(Surface.px(1), Surface.px(3)).toDouble();
    c.drawPath(path, Paint()..color = col..style = PaintingStyle.stroke..strokeWidth = stroke..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  }
  @override
  bool shouldRepaint(_Icon o) => true;
}

class _PlotP extends CustomPainter {
  _PlotP(this.f, this.all, this.x1, this.y1, this.x2, this.y2, this.editable, this.labels, this.name, this.mixed, this.f0, this.frames, this.ghost, this.head, {this.layers = 0, this.caption, this.meaning, this.points = const []});

  /// Handle points of a kind other than Bezier (0–1 space), drawn as the Bezier handles are.
  final List<Offset> points;

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

  // surface-block: painter geometry: drawn in the canvas's own pixels (method paint)
  @override
  void paint(Canvas c, Size s) {
    final r = box(s);
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(4.5)), Paint()..color = N.g07);
    // surface: plot markers shrink in compact plots without shrinking their gesture targets.
    final markerScale = (r.shortestSide / Surface.px(180)).clamp(.55, 1.3).toDouble();
    // surface: preserve the relative sizes of endpoints, handles and preview dots.
    double radius(double base) => Surface.px(base) * markerScale;
    final base = at(s, 0, 0).dy, top = at(s, 0, 1).dy;
    // only the two rails the curve travels between; no grid
    c.drawLine(Offset(r.left, base), Offset(r.right, base), Paint()..color = N.g20..strokeWidth = 1.2);
    c.drawLine(Offset(r.left, top), Offset(r.right, top), Paint()..color = N.g20..strokeWidth = 1.2);
    // the silhouette of the curve is the object: a flat colour field under it
    if (!mixed) {
      final area = _curve(s, f)..lineTo(at(s, 1, 0).dx, base)..lineTo(at(s, 0, 0).dx, base)..close();
      c.drawPath(area, Paint()..shader = const LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [kPink, kViolet, kBlue], stops: [0, .5, 1]).createShader(Rect.fromLTRB(r.left, top, r.right, base)));
    }
    if (layers > 1 && !mixed) {
      for (var i = 0; i < layers; i++) {
        final x = i / (layers - 1);
        c.drawCircle(at(s, x, f(x)), radius(4), Paint()..color = Surface.ink);
      }
    }
    if (ghost && !mixed) {
      for (var k = 3; k >= 1; k--) {
        c.drawPath(_curve(s, f, dx: .06 * k, squeeze: 1 - .06 * k), Paint()..color = Surface.ink.withValues(alpha: .40 - .09 * k)..style = PaintingStyle.stroke..strokeWidth = 1.4);
      }
    }
    if (mixed) {
      final cols = [kViolet, kMint, kPink];
      for (final (i, fn) in all.indexed) {
        final p = _curve(s, fn);
        c.drawPath(p, Paint()..color = cols[i]..style = PaintingStyle.stroke..strokeWidth = 2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
      }
    }
    // playhead: white line, flag on top
    final ph = r.left + r.width * head;
    c.drawLine(Offset(ph, r.top - 2), Offset(ph, r.bottom), Paint()..color = Surface.ink..strokeWidth = 1.6);
    c.drawPath(Path()..moveTo(ph - 6, r.top - 12)..lineTo(ph + 6, r.top - 12)..lineTo(ph, r.top - 2)..close(), Paint()..color = Surface.ink);
    // handles: yellow, the touchable thing
    if (editable) {
      final h = Paint()..color = kYellow..strokeWidth = 2;
      c.drawLine(at(s, 0, 0), at(s, x1, y1), h);
      c.drawLine(at(s, 1, 1), at(s, x2, y2), h);
      for (final o in [at(s, x1, y1), at(s, x2, y2)]) {
        c.drawCircle(o, radius(12), Paint()..color = kYellow.withValues(alpha: .22));
        c.drawCircle(o, radius(7), Paint()..color = kYellow);
        c.drawCircle(o, radius(7), Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = radius(2));
      }
    }
    if (!mixed) c.drawPath(_curve(s, f), Paint()..color = Surface.ink..style = PaintingStyle.stroke..strokeWidth = 2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
    for (final o in [for (final q in points) at(s, q.dx, q.dy)]) {
      c.drawCircle(o, radius(12), Paint()..color = kYellow.withValues(alpha: .22));
      c.drawCircle(o, radius(7), Paint()..color = kYellow);
      c.drawCircle(o, radius(7), Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = radius(2));
    }
    for (final o in [at(s, 0, 0), at(s, 1, 1)]) {
      c.drawCircle(o, radius(7), Paint()..color = Surface.ink);
    }
    final v = f(head);
    final pt = at(s, head, v);
    // motion strip: where the value lands at even steps, read off the right edge
    final cx = r.right + 16;
    for (var i = 0; i <= 8; i++) { c.drawCircle(Offset(cx, at(s, 0, f(i / 8)).dy), radius(2.8), Paint()..color = Surface.ink.withValues(alpha: .55)); }
    c.drawLine(pt, Offset(cx, pt.dy), Paint()..color = kMint..strokeWidth = 1.6);
    c.drawCircle(Offset(cx, pt.dy), radius(5.4), Paint()..color = kMint);
    if (!mixed) { c.drawCircle(pt, radius(6), Paint()..color = kMint); c.drawCircle(pt, radius(6), Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = radius(2)); }
    if (labels) {
      _text(c, mixed ? 'Mixed' : name, Offset(r.left, 9), sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600));
      _text(c, caption ?? (layers > 1 ? '$layers layers' : (mixed ? '3 intervals · $frames f' : '$frames f')), Offset(s.width - 14, 12), sans(Dn.nameSize, c: N.g69), right: true);
      if (meaning != null) _text(c, meaning!, Offset(r.left, 28), sans(Dn.labelSize, c: N.g69));
      _text(c, 'f$f0', Offset(r.left, s.height - 17), mono(Dn.microSize));
      _text(c, 'f${f0 + frames}', Offset(r.right, s.height - 17), mono(Dn.microSize), right: true);
    }
  }
  @override
  bool shouldRepaint(_PlotP o) => true;
}
