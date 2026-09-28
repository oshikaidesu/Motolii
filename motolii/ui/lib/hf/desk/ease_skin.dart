import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import '../../foundation/ease_view.dart';
import '../../foundation/panel_controls.dart' show EditorNumericField;
import '../bp/common.dart';
import 'common.dart';
import '../neutral.dart';

const _tileColors = [kBlue, kViolet, kPink, kMint, kYellow];
const _segColors = [kViolet, kMint, kPink];
const _dark = N.g10;

/// The finished Ease desk drawn from what the desk knows ([EaseView]): the curve the runtime models (its samples and
/// handles), the shelf of curves it offers, the intervals or the layers it is for, and the actions it takes.
/// Nothing here is a curve of its own: every shape, parameter and preview comes from the view.
class EaseSkin extends StatefulWidget {
  const EaseSkin(this.view, {super.key});
  final EaseView view;
  @override
  State<EaseSkin> createState() => _EaseSkinState();
}

class _EaseSkinState extends State<EaseSkin> {
  EaseView get v => widget.view;
  final _presetFocus = FocusNode();
  int _key = 0;
  int? _drag;

  @override
  void dispose() {
    _presetFocus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => DeskShell(
        kind: DeskKind.ease,
        title: 'Ease',
        subtitle: 'KEYFRAMES · MOTION',
        full: (c, s) => _body(s.height, s.width),
        strip: (c, s) => _body(s.height, s.width),
        tall: (c, s) => _body(s.height, s.width),
      );

  Widget _body(double h, double w) {
    final pad = w < 230 ? 8.0 : 12.0;
    final plotH = h < 200 ? (h - 20).clamp(56.0, 300.0) : (h * .42).clamp(110.0, 300.0);
    return SingleChildScrollView(
      key: const ValueKey('ease-scroll'),
      physics: _drag != null ? const NeverScrollableScrollPhysics() : null,
      padding: EdgeInsets.fromLTRB(pad, 12, pad, 16),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _head(),
        const SizedBox(height: 8),
        SizedBox(height: plotH, child: _plot(labels: w >= 230 && plotH >= 150)),
        const SizedBox(height: 10),
        _segments(),
        _section('CURVES', _presets(w)),
        if (v.params.isNotEmpty) _section('VALUES', _params(w)),
        _section('KEEP', _keep()),
        const SizedBox(height: 14),
        _apply(),
      ]),
    );
  }

  Widget _head() => Row(children: [
        if (v.leading != null) Padding(padding: const EdgeInsets.only(right: 8), child: v.leading!),
        Expanded(child: Text(v.title, key: const ValueKey('ease-title'), softWrap: false, overflow: TextOverflow.ellipsis, style: sans(12.5, c: kInk, w: FontWeight.w600))),
        if (v.sequence > 0) Container(margin: const EdgeInsets.only(left: 6), padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3), decoration: BoxDecoration(color: kBlue, borderRadius: BorderRadius.circular(9)), child: Text('ghosts', style: sans(10, c: kInk, w: FontWeight.w700))),
        if (!v.canApply) Padding(padding: const EdgeInsets.only(left: 6), child: Text('read only', style: sans(10, c: kMuted))),
      ]);

  Widget _section(String t, Widget child) => Padding(
        padding: const EdgeInsets.only(top: 16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          Text(t, style: sans(9.5, c: kMuted, w: FontWeight.w500, ls: 1.4)),
          const SizedBox(height: 8),
          child,
        ]),
      );

  // The intervals between the selected keys, width = duration, the active one ringed; or the layers a Sequence spreads over.
  Widget _segments() {
    if (v.sequence > 0) {
      return Text('${v.sequence} layers · the curve spreads their delay', style: sans(11, c: kMuted));
    }
    if (v.segments.isEmpty) return Text('Select keys to shape the curve between them.', style: sans(11, c: kMuted));
    return SizedBox(
      height: 30,
      child: Row(children: [
        for (final (i, s) in v.segments.indexed)
          Expanded(
            flex: (s.end - s.frame).clamp(1, 1 << 20),
            child: Container(
              key: ValueKey('ease-seg-$i'),
              margin: EdgeInsets.only(right: i == v.segments.length - 1 ? 0 : 4),
              padding: const EdgeInsets.symmetric(horizontal: 5),
              alignment: Alignment.centerLeft,
              decoration: BoxDecoration(color: _segColors[i % 3], borderRadius: BorderRadius.circular(5), border: s.active ? Border.all(color: kInk, width: 2.4) : null),
              child: Text('${s.end - s.frame}f', softWrap: false, overflow: TextOverflow.clip, style: sans(10, c: _dark, w: FontWeight.w700)),
            ),
          ),
      ]),
    );
  }

  Widget _presets(double w) {
    final cols = ((w - 24 + 5) / 76).floor().clamp(2, 6);
    final tile = (w - 24 - 5 * (cols - 1)) / cols;
    return Focus(
      focusNode: _presetFocus,
      onFocusChange: (f) {
        if (!f) v.endPeek();
      },
      onKeyEvent: (_, e) {
        if (e is! KeyDownEvent || v.presets.isEmpty) return KeyEventResult.ignored;
        final k = e.logicalKey, n = v.presets.length;
        if (k == LogicalKeyboardKey.enter) {
          v.choose(_key.clamp(0, n - 1));
          return KeyEventResult.handled;
        }
        final next = k == LogicalKeyboardKey.arrowRight
            ? _key + 1
            : k == LogicalKeyboardKey.arrowLeft
                ? _key - 1
                : k == LogicalKeyboardKey.arrowDown
                    ? _key + cols
                    : k == LogicalKeyboardKey.arrowUp
                        ? _key - cols
                        : k == LogicalKeyboardKey.home
                            ? 0
                            : k == LogicalKeyboardKey.end
                                ? n - 1
                                : null;
        if (next == null) return KeyEventResult.ignored;
        setState(() => _key = next.clamp(0, n - 1));
        v.peek(_key);
        return KeyEventResult.handled;
      },
      child: Wrap(spacing: 5, runSpacing: 5, children: [
        for (final (i, p) in v.presets.indexed)
          MouseRegion(
            onEnter: (_) => v.peek(i),
            onExit: (_) => v.endPeek(),
            child: GestureDetector(
              key: ValueKey('ease-preset-$i'),
              onTap: () {
                _presetFocus.requestFocus();
                setState(() => _key = i);
                v.choose(i);
              },
              child: Container(
                width: tile,
                height: 62,
                padding: const EdgeInsets.fromLTRB(8, 7, 8, 5),
                decoration: BoxDecoration(color: _tileColors[i % 5], borderRadius: BorderRadius.circular(5), border: p.selected ? Border.all(color: kInk, width: 2.4) : (_presetFocus.hasFocus && _key == i ? Border.all(color: kInk.withValues(alpha: .6), width: 1.4) : null)),
                child: Column(children: [
                  Expanded(child: CustomPaint(size: Size.infinite, painter: _Icon(p.shape))),
                  const SizedBox(height: 3),
                  Text(_name(p.kind), softWrap: false, overflow: TextOverflow.clip, style: sans(9.5, c: _dark, w: FontWeight.w600)),
                ]),
              ),
            ),
          ),
      ]),
    );
  }

  static String _name(String kind) => kind.replaceAllMapped(RegExp(r'([a-z])([A-Z])'), (m) => '${m[1]} ${m[2]}');

  // The parameters the runtime's model gives this curve: scrubbed or typed as previews, one commit.
  Widget _params(double w) => Wrap(spacing: 8, runSpacing: 6, children: [
        for (final p in v.params)
          Container(
            key: ValueKey('ease-param-${p.name}'),
            width: (w - 24 - 8) / 2,
            height: 44,
            padding: const EdgeInsets.fromLTRB(10, 5, 6, 5),
            decoration: BoxDecoration(color: kYellow, borderRadius: BorderRadius.circular(6)),
            child: Row(children: [
              Expanded(child: Text(p.name.replaceAll('_', ' ').toUpperCase(), softWrap: false, overflow: TextOverflow.clip, style: sans(9.5, c: N.inkSoft, w: FontWeight.w700, ls: .6))),
              SizedBox(
                width: 52,
                child: EditorNumericField(value: p.value, label: p.name, enabled: v.canApply || true, speed: .005, onPreview: p.preview, onCommit: p.commit, onFinish: p.finish, onCancel: p.cancel),
              ),
            ]),
          ),
      ]);

  Widget _chip(String key, String t, VoidCallback f) => GestureDetector(
        key: ValueKey(key),
        onTap: f,
        child: Container(height: 34, padding: const EdgeInsets.symmetric(horizontal: 12), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: N.g26), borderRadius: BorderRadius.circular(4)), child: Text(t, style: sans(11, c: N.g76))),
      );

  Widget _keep() => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Wrap(spacing: 5, runSpacing: 5, children: [
          _chip('ease-copy', 'Copy curve', v.copy),
          _chip('ease-save', 'Save preset', v.save),
          _chip('ease-new', 'New keys: ${v.newKeyKind}', v.useForNew),
          if (v.hasSaved) _chip('ease-clear', 'Clear saved', v.clearSaved),
        ]),
        if (v.notice != null) Padding(padding: const EdgeInsets.only(top: 8), child: Text(v.notice!, style: sans(10.5, c: kMuted))),
      ]);

  Widget _apply() => Row(children: [
        Expanded(
          child: GestureDetector(
            key: const ValueKey('ease-overshoot'),
            behavior: HitTestBehavior.opaque,
            onTap: () => v.setFree(!v.free),
            child: Row(children: [
              Container(width: 34, height: 20, padding: const EdgeInsets.all(2), alignment: v.free ? Alignment.centerRight : Alignment.centerLeft, decoration: BoxDecoration(color: v.free ? kBlue : N.g20, borderRadius: BorderRadius.circular(10)), child: Container(width: 16, height: 16, decoration: const BoxDecoration(color: kInk, shape: BoxShape.circle))),
              const SizedBox(width: 10),
              Flexible(child: Text('Overshoot', softWrap: false, overflow: TextOverflow.ellipsis, style: sans(11.5, c: N.g76))),
            ]),
          ),
        ),
        GestureDetector(
          key: const ValueKey('ease-apply'),
          onTap: v.canApply ? v.apply : null,
          child: Container(height: 34, padding: const EdgeInsets.symmetric(horizontal: 14), margin: const EdgeInsets.only(left: 8), alignment: Alignment.center, decoration: BoxDecoration(color: v.canApply ? kMint : kRaised, borderRadius: BorderRadius.circular(5)), child: Text('Apply', style: sans(12, c: v.canApply ? _dark : kMuted, w: FontWeight.w700))),
        ),
      ]);

  Widget _plot({required bool labels}) => LayoutBuilder(builder: (context, box) {
        final size = Size(box.maxWidth, box.maxHeight);
        final painter = _PlotP(v, labels);
        int? near(Offset p) {
          var best = -1;
          var bd = 24.0;
          for (final (i, h) in v.handles.indexed) {
            final d = (painter.toPixel(h, size) - p).distance;
            if (d < bd) {
              bd = d;
              best = i;
            }
          }
          return best < 0 ? null : best;
        }
        // Raw pointers: a handle drag must not lose to the panel's own vertical scroll.
        return Listener(
          key: const ValueKey('ease-plot'),
          onPointerDown: (e) {
            if (e.buttons != 1 || !v.canApply) return;
            final h = near(e.localPosition);
            if (h == null) return;
            setState(() => _drag = h);
            v.grab(h);
          },
          onPointerMove: (e) {
            if (_drag != null) v.drag(_drag!, painter.toCurve(e.localPosition, size));
          },
          onPointerUp: (_) {
            if (_drag == null) return;
            setState(() => _drag = null);
            v.release();
          },
          onPointerCancel: (_) {
            if (_drag == null) return;
            setState(() => _drag = null);
            v.cancel();
          },
          child: ListenableBuilder(listenable: Listenable.merge([v.frame, v.motion]), builder: (_, __) => CustomPaint(size: size, painter: _PlotP(v, labels))),
        );
      });
}

List<Offset> _samples(Map<String, dynamic> shape) {
  if (shape['kind'] == 'Hold') return const [Offset(0, 0), Offset(1, 0), Offset(1, 1)];
  final s = [for (final e in (shape['samples'] as List? ?? const []).whereType<List>()) Offset((e[0] as num).toDouble(), (e[1] as num).toDouble())];
  return s.length < 2 ? const [Offset(0, 0), Offset(1, 1)] : s;
}

double _valueAt(Map<String, dynamic> shape, double x) {
  if (shape['kind'] == 'Hold') return x < 1 ? 0 : 1;
  final s = _samples(shape);
  for (var i = 0; i + 1 < s.length; i++) {
    if (x <= s[i + 1].dx) {
      final t = s[i + 1].dx == s[i].dx ? 0.0 : ((x - s[i].dx) / (s[i + 1].dx - s[i].dx)).clamp(0.0, 1.0);
      return s[i].dy + (s[i + 1].dy - s[i].dy) * t;
    }
  }
  return s.last.dy;
}

class _Icon extends CustomPainter {
  _Icon(this.shape);
  final Map<String, dynamic> shape;
  @override
  void paint(Canvas c, Size s) {
    final path = Path();
    final pts = _samples(shape);
    var lo = 0.0, hi = 1.0;
    for (final p in pts) {
      if (p.dy < lo) lo = p.dy;
      if (p.dy > hi) hi = p.dy;
    }
    for (final (i, p) in pts.indexed) {
      final o = Offset(s.width * p.dx.clamp(0, 1), s.height - s.height * (p.dy - lo) / (hi - lo));
      i == 0 ? path.moveTo(o.dx, o.dy) : path.lineTo(o.dx, o.dy);
    }
    c.drawPath(path, Paint()..color = _dark..style = PaintingStyle.stroke..strokeWidth = 2..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
  }
  @override
  bool shouldRepaint(_Icon o) => o.shape != shape;
}

class _PlotP extends CustomPainter {
  _PlotP(this.v, this.labels);
  final EaseView v;
  final bool labels;

  Rect plot(Size s) => (Offset.zero & s).deflate(labels ? 18 : 8);
  Offset toPixel(Offset p, Size s) {
    final r = plot(s);
    return Offset(r.left + p.dx * r.width, r.top + (v.hi - p.dy) / (v.hi - v.lo) * r.height);
  }

  Offset toCurve(Offset p, Size s) {
    final r = plot(s);
    return Offset(((p.dx - r.left) / r.width).clamp(-.2, 1.2), v.hi - (p.dy - r.top) / r.height * (v.hi - v.lo));
  }

  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(6)), Paint()..color = N.g07);
    Offset p(double x, double y) => toPixel(Offset(x, y), s);
    final rail = Paint()..color = N.g20..strokeWidth = 1.2;
    c.drawLine(p(0, 0), p(1, 0), rail);
    c.drawLine(p(0, 1), p(1, 1), rail);
    Path curve(Map<String, dynamic> shape) {
      final path = Path();
      for (final (i, q) in _samples(shape).indexed) {
        final o = toPixel(q, s);
        i == 0 ? path.moveTo(o.dx, o.dy) : path.lineTo(o.dx, o.dy);
      }
      return path;
    }
    final shown = curve(v.shown);
    c.save();
    c.clipRect(Offset.zero & s);
    // the silhouette of the curve is the object: a flat colour field under it
    c.drawPath(Path.from(shown)..lineTo(p(1, 0).dx, p(0, 0).dy)..lineTo(p(0, 0).dx, p(0, 0).dy)..close(), Paint()..shader = LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [kPink.withAlpha(0x66), kViolet.withAlpha(0x66), kBlue.withAlpha(0x66)]).createShader(Offset.zero & s));
    c.drawPath(shown, Paint()..color = kInk..style = PaintingStyle.stroke..strokeWidth = 3.4..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
    if (v.sequence > 1) {
      for (var i = 0; i < v.sequence; i++) {
        final x = i / (v.sequence - 1);
        c.drawCircle(p(x, _valueAt(v.shape, x)), 4.5, Paint()..color = kYellow);
      }
    }
    final head = v.playhead;
    if (head != null) {
      final x = head.clamp(0.0, 1.0);
      final inside = head >= 0 && head <= 1;
      c.drawLine(p(x, v.lo), p(x, v.hi), Paint()..color = kInk.withValues(alpha: inside ? 1 : .4)..strokeWidth = 1.6);
      if (inside) {
        final o = p(x, _valueAt(v.shown, x));
        c.drawCircle(o, 6, Paint()..color = kMint);
        c.drawCircle(o, 6, Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = 2);
      }
    }
    // the audition: where the value lands as the motion runs
    if (v.motion.value > 0) {
      final x = v.motion.value;
      final o = p(x, _valueAt(v.shown, x));
      c.drawCircle(o, 6, Paint()..color = kMint);
    }
    c.restore();
    // handles: yellow, the touchable thing
    if (v.handles.isNotEmpty && v.shown == v.shape) {
      final ends = [p(0, 0), p(1, 1)];
      final h = Paint()..color = kYellow..strokeWidth = 2;
      for (final (i, q) in v.handles.indexed) {
        final o = toPixel(q, s);
        if (i < 2) c.drawLine(ends[i], o, h);
        c.drawCircle(o, 12, Paint()..color = kYellow.withValues(alpha: .22));
        c.drawCircle(o, 7, Paint()..color = kYellow);
        c.drawCircle(o, 7, Paint()..color = N.g07..style = PaintingStyle.stroke..strokeWidth = 2);
      }
    }
    for (final o in [p(0, 0), p(1, 1)]) {
      c.drawCircle(o, 6, Paint()..color = kInk);
    }
    if (labels) {
      final tp = TextPainter(text: TextSpan(text: _EaseSkinState._name('${v.shown['kind']}'), style: sans(14, c: kInk, w: FontWeight.w600)), textDirection: TextDirection.ltr)..layout();
      tp.paint(c, const Offset(20, 6));
    }
  }

  @override
  bool shouldRepaint(_PlotP o) => true;
}
