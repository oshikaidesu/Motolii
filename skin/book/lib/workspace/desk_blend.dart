// The desk's Blend tool: the layer stack, the front first. Each layer that lies on what is under it: how (its blend:
// the ways the inside names, each drawn as two discs and what their overlap becomes), how much of it is there (its
// opacity, a bar pulled across), and whether it shows only where the layer under it is (clip). A tap on a name picks it.
// What the ways are is the inside's (motolii_words); the window only draws each.
import 'package:flutter/gestures.dart' show kPrimaryButton;
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'desk_kit.dart';
import 'ws.dart';

class DeskBlend extends StatelessWidget {
  const DeskBlend({super.key});
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context);
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: WsT.gap, vertical: 6),
      children: [for (final l in ws.layers) _Row(l, picked: l.id == ws.selected)],
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(this.l, {required this.picked});
  final WsLayer l;
  final bool picked;
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), m = ws.inside, t = int.tryParse(l.id);
    final ways = m == null || t == null ? const <String>[] : m.words(t, 'blend');
    final now = ways.isEmpty ? '' : (m!.word(t!, 'blend').isEmpty ? ways.first : m.word(t, 'blend'));
    final clip = ways.isNotEmpty && (m!.value(t!, 'clip', 0) ?? 0) > 0.5;
    final opacity = ways.isEmpty ? 1.0 : (m!.value(t!, 'opacity', ws.frame / Ws.fps) ?? 1).clamp(0.0, 1.0);
    void say(String name, String word) {
      if (m != null && t != null && m.say(t, name, word)) ws.reread();
    }

    return Container(
      height: 34,
      margin: const EdgeInsets.only(bottom: 3),
      padding: const EdgeInsets.symmetric(horizontal: 6),
      decoration: BoxDecoration(color: picked ? WsT.well : null, borderRadius: BorderRadius.circular(5)),
      child: Row(
        children: [
          Container(width: 4, height: 18, decoration: BoxDecoration(color: wsTone(l.kind), borderRadius: BorderRadius.circular(2))),
          const SizedBox(width: 8),
          Expanded(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => ws.select(l.id),
              child: Text(l.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(ways.isEmpty ? Grey.g56 : Grey.g91)),
            ),
          ),
          if (ways.isNotEmpty) ...[
            for (final w in ways)
              Padding(
                padding: const EdgeInsets.only(left: 2),
                child: DeskIconButton(key: ValueKey('blend.${l.id}.$w'), size: 24, on: w == now, glyph: (ink) => _Way(w, ink), onTap: () => say('blend', w)),
              ),
            const SizedBox(width: 8),
            SizedBox(width: 64, height: 18, child: _Opacity(key: ValueKey('opacity.${l.id}'), thing: t!, value: opacity)),
            const SizedBox(width: 6),
            DeskIconButton(key: ValueKey('clip.${l.id}'), size: 24, on: clip, glyph: (ink) => _Clip(ink), onTap: () {
              if (m!.set(t, 'clip', clip ? 0 : 1)) ws.reread();
            }),
          ],
        ],
      ),
    );
  }
}

/// How much of it is there: a bar filled so far, pulled across (held and carried while pulled, one step when let go).
class _Opacity extends StatefulWidget {
  const _Opacity({super.key, required this.thing, required this.value});
  final int thing;
  final double value;
  @override
  State<_Opacity> createState() => _OpacityState();
}

class _OpacityState extends State<_Opacity> {
  double? _held;
  void _to(double x, double wide) {
    final m = WsScope.read(context).inside;
    if (m == null) return;
    final v = (x / wide).clamp(0.0, 1.0);
    if (_held == null && !m.hold(widget.thing)) return;
    setState(() => _held = v);
    m.carry('opacity', v);
  }

  void _letGo() {
    final ws = WsScope.read(context);
    if (_held == null) return;
    _held = null;
    if (ws.inside?.letGo(true) ?? false) ws.reread();
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) => Listener(
      onPointerDown: (e) => e.buttons == kPrimaryButton ? _to(e.localPosition.dx, box.maxWidth) : null,
      onPointerMove: (e) => _to(e.localPosition.dx, box.maxWidth),
      onPointerUp: (_) => _letGo(),
      onPointerCancel: (_) => _letGo(),
      child: CustomPaint(painter: _Bar(_held ?? widget.value)),
    ),
  );
}

class _Bar extends CustomPainter {
  _Bar(this.v);
  final double v;
  @override
  void paint(Canvas c, Size s) {
    final r = RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(4));
    c.drawRRect(r, Paint()..color = WsT.well);
    c.save();
    c.clipRRect(r);
    c.drawRect(Rect.fromLTWH(0, 0, s.width * v, s.height), Paint()..color = Grey.g56);
    c.restore();
  }

  @override
  bool shouldRepaint(_Bar o) => o.v != v;
}

/// A way of lying on what is under, drawn as what it does: two discs, their overlap as the way makes it.
class _Way extends CustomPainter {
  _Way(this.way, this.ink);
  final String way;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final r = s.shortestSide * .26, a = Offset(s.width * .38, s.height / 2), b = Offset(s.width * .62, s.height / 2);
    final under = Paint()..color = ink.withValues(alpha: .45), over = Paint()..color = ink.withValues(alpha: .45);
    c.drawCircle(a, r, under);
    final lens = Path.combine(PathOperation.intersect, Path()..addOval(Rect.fromCircle(center: a, radius: r)), Path()..addOval(Rect.fromCircle(center: b, radius: r)));
    switch (way) {
      case 'add': // (where they meet: brighter than either)
        c.drawCircle(b, r, over);
        c.drawPath(lens, Paint()..color = ink);
      case 'multiply': // (where they meet: darker than either)
        c.drawCircle(b, r, over);
        c.drawPath(lens, Paint()..color = Grey.g00);
      case 'screen': // (where they meet: lighter, softly)
        c.drawCircle(b, r, over);
        c.drawPath(lens, Paint()..color = ink.withValues(alpha: .75));
      default: // (normal: the one over hides the one under)
        c.drawCircle(b, r, Paint()..color = ink);
    }
  }

  @override
  bool shouldRepaint(_Way o) => o.way != way || o.ink != ink;
}

/// Shows only where the one under it is: a disc cut to a square under it.
class _Clip extends CustomPainter {
  _Clip(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final box = Rect.fromCenter(center: s.center(Offset.zero), width: s.width * .46, height: s.height * .46);
    c.drawRect(box, Paint()
      ..color = ink.withValues(alpha: .5)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1);
    c.save();
    c.clipRect(box);
    c.drawCircle(box.topRight, box.width * .7, Paint()..color = ink);
    c.restore();
  }

  @override
  bool shouldRepaint(_Clip o) => o.ink != ink;
}
