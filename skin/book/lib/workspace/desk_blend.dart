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

class _Row extends StatefulWidget {
  const _Row(this.l, {required this.picked});
  final WsLayer l;
  final bool picked;
  @override
  State<_Row> createState() => _RowState();
}

class _RowState extends State<_Row> {
  bool _open = false;
  String? _hover;
  @override
  Widget build(BuildContext context) {
    final l = widget.l, ws = WsScope.of(context), m = ws.inside, t = int.tryParse(l.id);
    final ways = m == null || t == null ? const <String>[] : m.words(t, 'blend');
    final now = ways.isEmpty ? '' : (m!.word(t!, 'blend').isEmpty ? ways.first : m.word(t, 'blend'));
    final clip = ways.isNotEmpty && (m!.value(t!, 'clip', 0) ?? 0) > 0.5;
    final opacity = ways.isEmpty ? 1.0 : (m!.value(t!, 'opacity', ws.frame / Ws.fps) ?? 1).clamp(0.0, 1.0);
    void say(String name, String word) {
      if (m != null && t != null && m.say(t, name, word)) ws.reread();
    }

    final row = Container(
      height: 34,
      padding: const EdgeInsets.symmetric(horizontal: 6),
      decoration: BoxDecoration(color: widget.picked ? WsT.well : null, borderRadius: BorderRadius.circular(5)),
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
            SizedBox(width: 74, child: Text(now.replaceAll('_', ' '), maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: TextAlign.right, style: T.micro(now == 'normal' ? Grey.g56 : Grey.g91))),   // (its way, named: two ways can look alike small)
            const SizedBox(width: 4),
            DeskIconButton(key: ValueKey('blend.${l.id}'), size: 24, on: _open, glyph: (ink) => _Way(now, ink), onTap: () => setState(() => _open = !_open)),
            const SizedBox(width: 8),
            SizedBox(width: 64, height: 18, child: _Opacity(key: ValueKey('opacity.${l.id}'), thing: t!, value: opacity)),
            const SizedBox(width: 6),
            if (l == ws.layers.last) const SizedBox(width: 24)   // (the back-most: nothing under it to clip to)
            else
              DeskIconButton(key: ValueKey('clip.${l.id}'), size: 24, on: clip, glyph: (ink) => _Clip(ink), onTap: () {
                if (m!.set(t, 'clip', clip ? 0 : 1)) ws.reread();
              }),
          ],
        ],
      ),
    );
    if (!_open || ways.isEmpty) return Padding(padding: const EdgeInsets.only(bottom: 3), child: row);
    // (every way, in After Effects' groups: normal; darker; lighter; contrast; difference; colour. The way under the
    //  hand, or the one it lies, named once under them)
    final groups = <List<String>>[[], [], [], [], [], []];
    for (final w in ways) {
      groups[_group(w)].add(w);
    }
    for (final (k, g) in groups.indexed) {
      g.sort((x, y) => _order[k].indexOf(x).compareTo(_order[k].indexOf(y)));
    }
    return Padding(
      padding: const EdgeInsets.only(bottom: 3),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          row,
          Padding(
            padding: const EdgeInsets.fromLTRB(18, 4, 6, 2),
            child: Wrap(
              spacing: 2,
              runSpacing: 2,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: [
                for (final (k, g) in groups.indexed)
                  if (g.isNotEmpty) ...[
                    if (k > 0) const SizedBox(width: 8),
                    for (final w in g)
                      MouseRegion(
                        onEnter: (_) => setState(() => _hover = w),
                        onExit: (_) => setState(() => _hover = null),
                        child: DeskIconButton(key: ValueKey('blend.${l.id}.$w'), size: 26, on: w == now, glyph: (ink) => _Way(w, ink), onTap: () => say('blend', w)),
                      ),
                  ],
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 2, 6, 4),
            child: Text((_hover ?? now).replaceAll('_', ' '), style: T.micro(Grey.g76)),
          ),
        ],
      ),
    );
  }
}

/// Which of After Effects' groups a way is in (0 normal, 1 darker, 2 lighter, 3 contrast, 4 difference, 5 colour).
int _group(String w) => _order.indexWhere((g) => g.contains(w)).clamp(0, 5);

/// After Effects' groups and its order within each (a way the inside names that is in none: with normal).
const _order = [
  ['normal'],
  ['darken', 'multiply', 'color_burn', 'linear_burn', 'darker_color'],
  ['add', 'lighten', 'screen', 'color_dodge', 'lighter_color'],
  ['overlay', 'soft_light', 'hard_light', 'linear_light', 'vivid_light', 'pin_light'],
  ['difference', 'exclusion', 'subtract', 'divide'],
  ['hue', 'saturation', 'color', 'luminosity'],
];

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
      onPointerMove: (e) => _held != null ? _to(e.localPosition.dx, box.maxWidth) : null,   // (only a drag that began with the first button)
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

/// A way of lying on what is under, drawn as what it does: a warm disc under, a cool one over, where they meet the
/// colour the inside draws for that way (check/blend_ways: the same two colours, as drawn).
class _Way extends CustomPainter {
  _Way(this.way, this.ink);
  final String way;
  final Color ink;
  static const _under = Color(0xFFCC4D33), _over = Color(0xFF3380E6);
  static const _meets = <String, Color>{
    'normal': Color(0xFF3380E6), 'add': Color(0xFFD193EA), 'multiply': Color(0xFF27222D), 'screen': Color(0xFFCE8FE7),
    'darken': Color(0xFF334D33), 'lighten': Color(0xFFCC80E6), 'overlay': Color(0xFFAD4D5C), 'soft_light': Color(0xFFB34D66),
    'hard_light': Color(0xFF524ED7), 'color_dodge': Color(0xFFFF9BFF), 'color_burn': Color(0xFF00001D), 'difference': Color(0xFF9933B3),
    'exclusion': Color(0xFFAD80BD), 'hue': Color(0xFF397AD2), 'saturation': Color(0xFFDC4729), 'color': Color(0xFF2F7CE2), 'luminosity': Color(0xFFD05137),
    'linear_burn': Color(0xFF00001A), 'linear_light': Color(0xFF334EFF), 'vivid_light': Color(0xFF804DFF), 'pin_light': Color(0xFF664DCD),
    'darker_color': Color(0xFFCC4D33), 'lighter_color': Color(0xFF3380E6), 'subtract': Color(0xFF990000), 'divide': Color(0xFFFF9939),
  };
  @override
  void paint(Canvas c, Size s) {
    final r = s.shortestSide * .27, a = Offset(s.width * .38, s.height / 2), b = Offset(s.width * .62, s.height / 2);
    c.drawCircle(a, r, Paint()..color = _under);
    c.drawCircle(b, r, Paint()..color = _over);
    final lens = Path.combine(PathOperation.intersect, Path()..addOval(Rect.fromCircle(center: a, radius: r)), Path()..addOval(Rect.fromCircle(center: b, radius: r)));
    c.drawPath(lens, Paint()..color = _meets[way] ?? _over);
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
