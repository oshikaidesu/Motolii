// The workspace's Timeline seat: transport + timecode on top, layer labels beside the track area, a ruler with the work area, the playhead.
// The document lives in Ws; this seat keeps only how it looks at it (zoom, scroll, loop, the row toggles, the work area).
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show PointerScrollEvent, PointerSignalEvent;
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../sets/panels/timeline_parts_parts.dart' show TpGlyph, TpGlyphPainter;
import '../tokens.dart';
import 'timeline_paint.dart';
import 'ws.dart';

class WsTimeline extends StatefulWidget {
  const WsTimeline({super.key});
  @override
  State<WsTimeline> createState() => _WsTimelineState();
}

class _WsTimelineState extends State<WsTimeline> with SingleTickerProviderStateMixin {
  late final Ticker _ticker = createTicker(_tick);
  final _focus = FocusNode(debugLabel: 'timeline');
  Ws? _ws;

  bool _loop = true;
  double _zoom = 1, _scroll = 0, _vscroll = 0, _trackW = 1000, _rowsH = 200;
  int _workIn = 12, _workOut = 228;
  final _hidden = <String>{}, _locked = <String>{'bg'}, _solo = <String>{};
  static const _markers = [(36, 'Hit'), (120, 'Drop'), (204, 'Out')];

  Duration _t0 = Duration.zero;
  int _base = 0, _last = -1;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final ws = WsScope.read(context);
    if (ws != _ws) {
      _ws?.removeListener(_changed);
      _ws = ws..addListener(_changed);
      if (ws.playing) SchedulerBinding.instance.addPostFrameCallback((_) => mounted ? _changed() : null);
    }
  }

  @override
  void dispose() {
    _ws?.removeListener(_changed);
    _ticker.dispose();
    _focus.dispose();
    super.dispose();
  }

  // ---------------------------------------------------------------- play

  void _changed() {
    final ws = _ws!;
    if (ws.playing && !_ticker.isActive) {
      _ticker.start();
      _last = -1;
      if (ws.frame >= Ws.duration) ws.frame = 0;
    } else if (!ws.playing && _ticker.isActive) {
      _ticker.stop();
    }
    _follow(ws.frame);
  }

  void _tick(Duration e) {
    final ws = _ws!;
    if (ws.frame != _last) {
      _base = ws.frame;
      _t0 = e;
    }
    var f = _base + ((e - _t0).inMicroseconds * Ws.fps ~/ 1000000);
    if (f >= Ws.duration) {
      if (!_loop) {
        _last = Ws.duration;
        ws.frame = Ws.duration;
        ws.playing = false;
        return;
      }
      f -= Ws.duration;
      _base = f;
      _t0 = e;
    }
    _last = f;
    ws.frame = f;
  }

  void _toggle() {
    final ws = _ws!;
    ws.playing = !ws.playing;
  }

  void _step(int d) {
    final ws = _ws!;
    ws.playing = false;
    ws.frame = ws.frame + d;
  }

  // ---------------------------------------------------------------- view

  double get _fit => math.max(.1, (_trackW - WtM.padL - WtM.padR) / Ws.duration);
  WtView get _view => WtView(_fit * _zoom, _scroll);
  double get _visible => Ws.duration / _zoom;
  double _clampScroll(double s) => s.clamp(0.0, math.max(0.0, Ws.duration - _visible)).toDouble();

  void _setZoom(double z, {int? anchor}) {
    final nz = z.clamp(1.0, WtM.maxZoom);
    final a = anchor ?? _ws!.frame, ax = _view.x(a);
    setState(() {
      _zoom = nz;
      _scroll = _clampScroll(a - (ax - WtM.padL) / (_fit * nz));
    });
  }

  /// Keeps the playhead in view when zoomed in: the view pages along as play or a scrub carries it off the edge.
  void _follow(int f) {
    if (_zoom <= 1 || !mounted) return;
    final v = _view, x = v.x(f);
    if (x >= 0 && x <= _trackW - WtM.padR) return;
    setState(() => _scroll = _clampScroll(f - _visible * .1));
  }

  void _wheel(PointerSignalEvent e) {
    if (e is! PointerScrollEvent) return;
    final d = e.scrollDelta, sideways = HardwareKeyboard.instance.isShiftPressed || d.dx.abs() > d.dy.abs();
    if (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed) {
      _setZoom(_zoom * math.pow(1.0015, -d.dy));
    } else if (sideways) {
      setState(() => _scroll = _clampScroll(_scroll + (d.dx.abs() > d.dy.abs() ? d.dx : d.dy) / _view.ppf));
    } else {
      setState(() => _vscroll = _vscroll + d.dy);
    }
  }

  double _clampV(int rows) => _vscroll.clamp(0.0, math.max(0.0, rows * WtM.row - _rowsH)).toDouble();

  // ---------------------------------------------------------------- input

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
    final k = e.logicalKey, ws = _ws!;
    if (k == LogicalKeyboardKey.space && e is KeyDownEvent) {
      _toggle();
    } else if (k == LogicalKeyboardKey.arrowLeft) {
      _step(HardwareKeyboard.instance.isShiftPressed ? -10 : -1);
    } else if (k == LogicalKeyboardKey.arrowRight) {
      _step(HardwareKeyboard.instance.isShiftPressed ? 10 : 1);
    } else if (k == LogicalKeyboardKey.home) {
      _step(-ws.frame);
    } else if (k == LogicalKeyboardKey.end) {
      _step(Ws.duration - ws.frame);
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  void _trackDown(Offset p) {
    final ws = _ws!, v = _view;
    final i = ((p.dy + _vscroll) / WtM.row).floor();
    if (i < 0 || i >= ws.layers.length) return ws.clearKeys();
    final l = ws.layers[i];
    final cy = i * WtM.row - _vscroll + WtM.row / 2;
    if (!_locked.contains(l.id) && (p.dy - cy).abs() <= WtM.bar / 2 + 2) {
      int? best;
      var bd = WtM.keyPicked + 2;
      for (final k in l.keys) {
        final d = (v.x(k) - p.dx).abs();
        if (d <= bd) {
          bd = d;
          best = k;
        }
      }
      if (best != null) return ws.pickKey((layer: l.id, frame: best), add: HardwareKeyboard.instance.isShiftPressed);
    }
    final f = v.frameAt(p.dx);
    if (f >= l.inF && f <= l.outF) {
      ws.select(l.id);
    } else {
      ws.clearKeys();
    }
  }

  int? _workDrag; // 0 in, 1 out, 2 both, null scrub
  double _workGrab = 0;

  void _rulerDown(Offset p) {
    final v = _view, f = v.frameAt(p.dx);
    _workDrag = null;
    if (p.dy <= WtM.band + 2) {
      final a = v.x(_workIn), b = v.x(_workOut);
      if ((p.dx - a).abs() <= 6) {
        _workDrag = 0;
      } else if ((p.dx - b).abs() <= 6) {
        _workDrag = 1;
      } else if (p.dx > a && p.dx < b) {
        _workDrag = 2;
        _workGrab = f - _workIn;
      }
    }
    _rulerMove(p);
  }

  void _rulerMove(Offset p) {
    final f = _view.frameAt(p.dx).round();
    switch (_workDrag) {
      case 0:
        setState(() => _workIn = f.clamp(0, _workOut - 2));
      case 1:
        setState(() => _workOut = f.clamp(_workIn + 2, Ws.duration));
      case 2:
        final len = _workOut - _workIn, a = (_view.frameAt(p.dx) - _workGrab).round().clamp(0, Ws.duration - len);
        setState(() {
          _workIn = a;
          _workOut = a + len;
        });
      default:
        _ws!.frame = f;
    }
  }

  void _overview(Offset p, double w) {
    final f = (p.dx - 4) / (w - 8) * Ws.duration;
    setState(() => _scroll = _clampScroll(f - _visible / 2));
  }

  void _flip(Set<String> s, String id) => setState(() => s.contains(id) ? s.remove(id) : s.add(id));

  // ---------------------------------------------------------------- build

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context);
    final sel = ws.selected, picks = ws.keys;
    final rows = [
      for (final l in ws.layers)
        WtRow(
          name: l.name,
          kind: l.kind,
          inF: l.inF,
          outF: l.outF,
          keys: l.keys,
          picked: {
            for (final k in picks)
              if (k.layer == l.id) k.frame,
          },
          selected: l.id == sel,
          dim: _hidden.contains(l.id) || (_solo.isNotEmpty && !_solo.contains(l.id)),
          locked: _locked.contains(l.id),
        ),
    ];
    return Focus(
      focusNode: _focus,
      onKeyEvent: _key,
      child: Listener(
        onPointerDown: (_) => _focus.requestFocus(),
        child: ColoredBox(
          color: WsT.body,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _Header(ws: ws, loop: _loop, zoom: _zoom, onToggle: _toggle, onStep: _step, onLoop: () => setState(() => _loop = !_loop), onZoom: _setZoom),
              Expanded(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    SizedBox(width: WtM.label, child: _labels(ws, rows)),
                    SizedBox(
                      width: WtM.divider,
                      child: ColoredBox(color: WsT.ground),
                    ),
                    Expanded(child: _tracks(ws, rows)),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _labels(Ws ws, List<WtRow> rows) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _LabelHeader(count: ws.layers.length, solo: _solo.isNotEmpty),
      Expanded(
        child: Listener(
          onPointerSignal: _wheel,
          child: ClipRect(
            child: LayoutBuilder(
              builder: (context, box) {
                final top = _clampV(rows.length);
                return Stack(
                  children: [
                    for (var i = 0; i < rows.length; i++)
                      Positioned(
                        key: ValueKey('ws-tl-label-${ws.layers[i].id}'),
                        left: 0,
                        right: 0,
                        top: i * WtM.row - top,
                        height: WtM.row,
                        child: _LabelRow(
                          index: i,
                          row: rows[i],
                          hidden: _hidden.contains(ws.layers[i].id),
                          solo: _solo.contains(ws.layers[i].id),
                          onSelect: () => ws.select(ws.layers[i].id),
                          onEye: () => _flip(_hidden, ws.layers[i].id),
                          onSolo: () => _flip(_solo, ws.layers[i].id),
                          onLock: () => _flip(_locked, ws.layers[i].id),
                        ),
                      ),
                  ],
                );
              },
            ),
          ),
        ),
      ),
    ],
  );

  Widget _tracks(Ws ws, List<WtRow> rows) => LayoutBuilder(
    builder: (context, box) {
      _trackW = box.maxWidth;
      _rowsH = math.max(0, box.maxHeight - WtM.ruler - WtM.overview);
      _scroll = _clampScroll(_scroll);
      _vscroll = _clampV(rows.length);
      final v = _view;
      return Stack(
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              GestureDetector(
                key: const ValueKey('ws-tl-ruler'),
                behavior: HitTestBehavior.opaque,
                onPanDown: (d) => _rulerDown(d.localPosition),
                onPanUpdate: (d) => _rulerMove(d.localPosition),
                child: SizedBox(
                  height: WtM.ruler,
                  child: CustomPaint(
                    painter: WtRulerPainter(view: v, fps: Ws.fps, workIn: _workIn, workOut: _workOut, markers: _markers),
                  ),
                ),
              ),
              Expanded(
                child: Listener(
                  onPointerSignal: _wheel,
                  child: GestureDetector(
                    key: const ValueKey('ws-tl-tracks'),
                    behavior: HitTestBehavior.opaque,
                    onTapDown: (d) => _trackDown(d.localPosition),
                    child: RepaintBoundary(
                      child: CustomPaint(
                        size: Size.infinite,
                        painter: WtTracksPainter(rows: rows, view: v, fps: Ws.fps, vscroll: _vscroll),
                      ),
                    ),
                  ),
                ),
              ),
              _Overview(from: _scroll, to: _scroll + _visible, frame: ws.frame, onDrag: (p) => _overview(p, box.maxWidth)),
            ],
          ),
          Positioned(
            left: 0,
            right: 0,
            top: 0,
            bottom: WtM.overview,
            child: IgnorePointer(
              child: RepaintBoundary(
                child: ClipRect(
                  child: CustomPaint(
                    painter: WtPlayheadPainter(view: v, frame: ws.frame, headTop: WtM.band + 1),
                  ),
                ),
              ),
            ),
          ),
        ],
      );
    },
  );
}

// ---------------------------------------------------------------- header strip

class _Header extends StatelessWidget {
  const _Header({
    required this.ws,
    required this.loop,
    required this.zoom,
    required this.onToggle,
    required this.onStep,
    required this.onLoop,
    required this.onZoom,
  });
  final Ws ws;
  final bool loop;
  final double zoom;
  final VoidCallback onToggle, onLoop;
  final ValueChanged<int> onStep;
  final void Function(double z, {int? anchor}) onZoom;

  @override
  Widget build(BuildContext context) {
    final tc = ws.timecode().split(':');
    final big = T.value(Grey.g95).copyWith(fontSize: 17, fontWeight: FontWeight.w600, fontFeatures: const [FontFeature.tabularFigures()]);
    final picked = ws.keys.length, layer = ws.layer;
    return Container(
      height: WtM.header,
      padding: const EdgeInsets.symmetric(horizontal: WsT.inset),
      decoration: BoxDecoration(
        color: WsT.body,
        border: Border(bottom: BorderSide(color: WsT.line)),
      ),
      child: LayoutBuilder(
        builder: (context, box) {
          final roomy = box.maxWidth >= 1000, mid = box.maxWidth >= 820;
          return Row(
            children: [
              if (mid) ...[const _Chip('Main', mono: false), const SizedBox(width: 12)],
              _Btn(TpGlyph.toStart, key: const ValueKey('ws-tl-start'), onTap: () => onStep(-ws.frame)),
              const SizedBox(width: 2),
              _Btn(TpGlyph.stepBack, key: const ValueKey('ws-tl-back'), onTap: () => onStep(-1)),
              const SizedBox(width: 2),
              _Btn(ws.playing ? TpGlyph.pause : TpGlyph.play, key: const ValueKey('ws-tl-play'), width: 34, on: ws.playing, rest: WsT.accent, onTap: onToggle),
              const SizedBox(width: 2),
              _Btn(TpGlyph.stepFwd, key: const ValueKey('ws-tl-fwd'), onTap: () => onStep(1)),
              const SizedBox(width: 2),
              _Btn(TpGlyph.loop, key: const ValueKey('ws-tl-loop'), on: loop, onTap: onLoop),
              const SizedBox(width: 12),
              Text(tc[0], style: big.copyWith(color: T.ink(Grey.g56))),
              Text(':', style: big.copyWith(color: T.ink(Grey.g56))),
              Text(tc[1], style: big),
              Text(':', style: big.copyWith(color: T.ink(Grey.g56))),
              Text(tc[2], style: big.copyWith(color: T.ink(WsT.accent))),
              const SizedBox(width: 8),
              if (roomy) ...[Text('/ ${ws.timecode(Ws.duration)}', style: T.value(Grey.g56)), const SizedBox(width: 10)],
              const _Chip('${Ws.fps} fps'),
              const SizedBox(width: 12),
              if (mid && layer != null)
                Flexible(
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      WtKindChip(layer.kind),
                      const SizedBox(width: 6),
                      Flexible(
                        child: Text(
                          picked > 0 ? '${layer.name}  ·  $picked key${picked == 1 ? '' : 's'}' : '${layer.name}  ·  ${layer.keys.length} keys',
                          style: T.label(Grey.g76),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    ],
                  ),
                ),
              const Spacer(),
              _Btn(TpGlyph.minus, key: const ValueKey('ws-tl-zoom-out'), size: 22, onTap: () => onZoom(zoom / 1.5)),
              const SizedBox(width: 4),
              _ZoomSlider(zoom: zoom, onZoom: onZoom),
              const SizedBox(width: 4),
              _Btn(TpGlyph.plus, key: const ValueKey('ws-tl-zoom-in'), size: 22, onTap: () => onZoom(zoom * 1.5)),
              const SizedBox(width: 4),
              _Btn(TpGlyph.fit, key: const ValueKey('ws-tl-fit'), size: 22, width: 46, label: 'Fit', lit: zoom <= 1.001, onTap: () => onZoom(1)),
            ],
          );
        },
      ),
    );
  }
}

/// A round touchable: a well that lights on hover; [on] fills it with the accent, [lit] raises it in grey.
class _Btn extends StatefulWidget {
  const _Btn(this.g, {super.key, this.onTap, this.on = false, this.lit = false, this.size = 24, this.width, this.rest, this.label});
  final TpGlyph g;
  final VoidCallback? onTap;
  final bool on, lit;
  final double size;
  final double? width;
  final Color? rest;
  final String? label;
  @override
  State<_Btn> createState() => _BtnState();
}

class _BtnState extends State<_Btn> {
  bool _h = false, _d = false;
  @override
  Widget build(BuildContext context) {
    final on = widget.on;
    final bg = on ? WsT.accent : (_d || widget.lit ? Grey.g26 : (_h ? WsT.raised : WsT.well));
    final ink = on ? WsT.onAccent : (_h || widget.lit ? Grey.g100 : (widget.rest ?? Grey.g91));
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: (_) => setState(() => _d = true),
        onTapUp: (_) => setState(() => _d = false),
        onTapCancel: () => setState(() => _d = false),
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          width: widget.width ?? widget.size,
          height: widget.size,
          decoration: BoxDecoration(color: bg, borderRadius: BorderRadius.circular(WsT.radius)),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CustomPaint(size: const Size.square(11), painter: TpGlyphPainter(widget.g, ink)),
              if (widget.label case final l?) ...[const SizedBox(width: 4), Text(l, style: T.micro(ink).copyWith(fontWeight: FontWeight.w700))],
            ],
          ),
        ),
      ),
    );
  }
}

class _Chip extends StatelessWidget {
  const _Chip(this.text, {this.mono = true});
  final String text;
  final bool mono;
  @override
  Widget build(BuildContext context) => Container(
    height: 20,
    padding: const EdgeInsets.symmetric(horizontal: 7),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.circular(WsT.radius)),
    child: Text(text, style: mono ? T.value(Grey.g91).copyWith(fontSize: 10.5) : T.micro(Grey.g91).copyWith(fontWeight: FontWeight.w600)),
  );
}

class _ZoomSlider extends StatelessWidget {
  const _ZoomSlider({required this.zoom, required this.onZoom});
  final double zoom;
  final void Function(double z, {int? anchor}) onZoom;
  static const w = 96.0;
  @override
  Widget build(BuildContext context) {
    final t = math.log(zoom) / math.log(WtM.maxZoom);
    void at(Offset p) => onZoom(math.pow(WtM.maxZoom, ((p.dx - 6) / (w - 12)).clamp(0.0, 1.0)).toDouble());
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        key: const ValueKey('ws-tl-zoom'),
        behavior: HitTestBehavior.opaque,
        onPanDown: (d) => at(d.localPosition),
        onPanUpdate: (d) => at(d.localPosition),
        child: SizedBox(
          width: w,
          height: 22,
          child: CustomPaint(painter: _ZoomTrack(t)),
        ),
      ),
    );
  }
}

class _ZoomTrack extends CustomPainter {
  const _ZoomTrack(this.t);
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2, x = 6 + (s.width - 12) * t, p = Paint();
    c.drawRRect(RRect.fromLTRBR(0, cy - 3, s.width, cy + 3, const Radius.circular(3)), p..color = WsT.well);
    c.drawRRect(RRect.fromLTRBR(0, cy - 3, x, cy + 3, const Radius.circular(3)), p..color = Grey.g44);
    c.drawCircle(Offset(x, cy), 6, p..color = Grey.g91);
  }

  @override
  bool shouldRepaint(_ZoomTrack o) => o.t != t;
}

// ---------------------------------------------------------------- label column

class _LabelHeader extends StatelessWidget {
  const _LabelHeader({required this.count, required this.solo});
  final int count;
  final bool solo;
  @override
  Widget build(BuildContext context) => Container(
    height: WtM.ruler,
    padding: const EdgeInsets.only(left: 8, right: 6),
    decoration: BoxDecoration(
      color: Grey.g10,
      border: Border(bottom: BorderSide(color: WsT.line)),
    ),
    child: Row(
      children: [
        Text('#', style: T.micro(Grey.g56)),
        const SizedBox(width: 12),
        Text('LAYER', style: T.micro(Grey.g63).copyWith(letterSpacing: 1)),
        const SizedBox(width: 6),
        Text('$count', style: T.value(Grey.g56).copyWith(fontSize: 10)),
        const Spacer(),
        for (final g in const [TpGlyph.eye, TpGlyph.solo, TpGlyph.lock])
          SizedBox(
            width: 20,
            child: Center(
              child: CustomPaint(size: const Size.square(10), painter: TpGlyphPainter(g, g == TpGlyph.solo && solo ? WsT.accent : Grey.g56)),
            ),
          ),
      ],
    ),
  );
}

class _LabelRow extends StatefulWidget {
  const _LabelRow({
    required this.index,
    required this.row,
    required this.hidden,
    required this.solo,
    required this.onSelect,
    required this.onEye,
    required this.onSolo,
    required this.onLock,
  });
  final int index;
  final WtRow row;
  final bool hidden, solo;
  final VoidCallback onSelect, onEye, onSolo, onLock;
  @override
  State<_LabelRow> createState() => _LabelRowState();
}

class _LabelRowState extends State<_LabelRow> {
  bool _h = false;
  @override
  Widget build(BuildContext context) {
    final r = widget.row, sel = r.selected;
    final bg = sel ? wtBand(widget.index, true) : (_h ? Grey.g15 : wtBand(widget.index, false));
    final ink = r.dim ? Grey.g56 : (sel ? Grey.g100 : Grey.g91);
    return MouseRegion(
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onSelect,
        child: Container(
          color: bg,
          padding: const EdgeInsets.only(bottom: 1),
          child: Stack(
            children: [
              if (sel) const Positioned(left: 0, top: 0, bottom: 0, width: 3, child: ColoredBox(color: WsT.accent)),
              Padding(
                padding: const EdgeInsets.only(left: 8, right: 6),
                child: Row(
                  children: [
                    SizedBox(width: 14, child: Text('${widget.index + 1}', style: T.value(sel ? Grey.g91 : Grey.g56).copyWith(fontSize: 10))),
                    const SizedBox(width: 6),
                    WtKindChip(r.kind, dim: r.dim),
                    const SizedBox(width: 7),
                    Expanded(
                      child: Text(
                        r.name,
                        style: T.name(ink).copyWith(fontWeight: sel ? FontWeight.w700 : FontWeight.w500),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    _Toggle(widget.hidden ? TpGlyph.eyeOff : TpGlyph.eye, on: false, lit: !widget.hidden, onTap: widget.onEye),
                    _Toggle(TpGlyph.solo, on: widget.solo, lit: _h || sel, onTap: widget.onSolo),
                    _Toggle(r.locked ? TpGlyph.lock : TpGlyph.unlock, on: false, lit: r.locked || _h || sel, strong: r.locked, onTap: widget.onLock),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// A row toggle: a bare glyph; [on] puts it on an accent chip, [strong] on a raised grey one.
class _Toggle extends StatelessWidget {
  const _Toggle(this.g, {required this.on, required this.lit, this.strong = false, required this.onTap});
  final TpGlyph g;
  final bool on, lit, strong;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: onTap,
    child: SizedBox(
      width: 20,
      height: WtM.row - 1,
      child: Center(
        child: Container(
          width: 17,
          height: 17,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: on ? WsT.accent : (strong ? Grey.g26 : null), borderRadius: BorderRadius.circular(WsT.chipRadius)),
          child: CustomPaint(size: const Size.square(10), painter: TpGlyphPainter(g, on ? WsT.onAccent : (strong ? Grey.g95 : (lit ? Grey.g76 : Grey.g38)))),
        ),
      ),
    ),
  );
}

// ---------------------------------------------------------------- overview

class _Overview extends StatefulWidget {
  const _Overview({required this.from, required this.to, required this.frame, required this.onDrag});
  final double from, to;
  final int frame;
  final ValueChanged<Offset> onDrag;
  @override
  State<_Overview> createState() => _OverviewState();
}

class _OverviewState extends State<_Overview> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) => setState(() => _h = true),
    onExit: (_) => setState(() => _h = false),
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onPanDown: (d) => widget.onDrag(d.localPosition),
      onPanUpdate: (d) => widget.onDrag(d.localPosition),
      child: SizedBox(
        height: WtM.overview,
        child: CustomPaint(
          painter: WtOverviewPainter(from: widget.from, to: widget.to, frame: widget.frame, hot: _h),
        ),
      ),
    ),
  );
}
