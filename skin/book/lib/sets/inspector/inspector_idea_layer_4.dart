// Part of inspector_idea_layer.dart (cut there by size, as it was).
part of 'inspector_idea_layer.dart';

class _LiScrub extends StatefulWidget {
  const _LiScrub(this.id, this.text, {required this.perPx, required this.min, required this.max, required this.dec, required this.onTap});
  final String id, text;
  final double perPx, min, max;
  final int dec;
  final VoidCallback onTap;
  @override
  State<_LiScrub> createState() => _LiScrubState();
}

class _LiScrubState extends State<_LiScrub> {
  double _start = 0, _acc = 0;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, on = !x.cfg.locked;
    return MouseRegion(
      cursor: on ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.basic,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onTap,
        onHorizontalDragStart: on
            ? (_) {
                _start = doc.get(widget.id) ?? 0;
                _acc = 0;
                doc.beginGesture();
                widget.onTap();
              }
            : null,
        onHorizontalDragUpdate: on
            ? (e) {
                _acc += e.delta.dx * x.cfg.mult();
                doc.set(widget.id, _liRound((_start + _acc * widget.perPx).clamp(widget.min, widget.max), widget.dec));
              }
            : null,
        onHorizontalDragEnd: on ? (_) => doc.endGesture() : null,
        onHorizontalDragCancel: on ? doc.cancelGesture : null,
        child: Center(child: Text(widget.text, maxLines: 1, style: T.value(Grey.g95).copyWith(fontSize: 10))),
      ),
    );
  }
}

class _LiWirePaint extends CustomPainter {
  const _LiWirePaint(this.wires, this.live);
  final List<((Offset, Offset, Offset, Offset), bool)> wires;
  final (Offset, Offset)? live;
  @override
  void paint(Canvas c, Size s) {
    Path bez((Offset, Offset, Offset, Offset) w) => Path()
      ..moveTo(w.$1.dx, w.$1.dy)
      ..cubicTo(w.$2.dx, w.$2.dy, w.$3.dx, w.$3.dy, w.$4.dx, w.$4.dy);
    for (final (w, sel) in [...wires]..sort((a, b) => a.$2 ? 1 : (b.$2 ? -1 : 0))) {
      c.drawPath(bez(w), _liLine(sel ? Pop.toneLink : Pop.toneLink.withValues(alpha: .5), sel ? 2 : 1.3));
    }
    final l = live;
    if (l != null) _liDash(c, bez(_liWire(l.$1, l.$2)), _liLine(Grey.g95, 1.5));
  }

  @override
  bool shouldRepaint(_LiWirePaint o) => true;
}

// ---- D. Matte at a glance -------------------------------------------------------------------------------------------
const _liMatteModes = ['Alpha', 'Inverted Alpha', 'Luma', 'Inverted Luma'], _liMatteWords = ['Alpha', 'Inv α', 'Luma', 'Inv L'];

/// Top to bottom; index 1 is this layer, 2 is the layer below (the clip base).
const _liStack = ['Title', 'Jewel', 'Card'];
const _liLuma = ColorFilter.matrix([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, .2126, .7152, .0722, 0, 0]);

void _liLayer(Canvas c, Rect r, int i) {
  switch (i) {
    case 0:
      final tp = TextPainter(
        text: TextSpan(
          text: 'JEWEL',
          style: TextStyle(
            fontFamily: T.sans,
            fontSize: r.height * .4,
            fontWeight: FontWeight.w900,
            height: 1,
            foreground: Paint()..shader = LinearGradient(colors: [Grey.g100, Grey.g26]).createShader(r.deflate(r.width * .12)),
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(c, r.center - Offset(tp.width / 2, tp.height / 2));
    case 1:
      final cols = [Pop.toneTime, Pop.tonePlace, Pop.toneTextAnim, Pop.tonePath, Pop.toneSolid];
      final w = r.width / 6;
      c.save();
      c.clipRect(r);
      for (var k = -4; k < 8; k++) {
        final x = r.left + k * w;
        c.drawPath(
          Path()
            ..moveTo(x, r.bottom)
            ..lineTo(x + w, r.bottom)
            ..lineTo(x + w + r.height, r.top)
            ..lineTo(x + r.height, r.top)
            ..close(),
          Paint()..color = cols[(k + 4) % cols.length],
        );
      }
      c.restore();
    default:
      c.drawRRect(
        RRect.fromRectAndRadius(Rect.fromCenter(center: r.center, width: r.width * .66, height: r.height * .7), Radius.circular(r.height * .14)),
        Paint()..color = const Color(0xFF55627A),
      );
  }
}

class _LiMatteCard extends StatelessWidget {
  const _LiMatteCard();
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(const {}, strs: const {'li_d.src': 'Title', 'li_d.mode': 'Luma'}, flags: const {'li_d.clip': false});
    final src = doc.s2['li_d.src'] ?? 'None', none = src == 'None', clip = doc.b['li_d.clip'] ?? false;
    final mode = math.max(0, _liMatteModes.indexOf(doc.s2['li_d.mode'] ?? 'Alpha'));
    final where = const ['is opaque', 'is clear', 'is bright', 'is dark'][mode];
    final say = none
        ? (clip ? 'Jewel shows only inside Card, the layer below.' : 'Jewel draws whole. Click Title or Card to cut it with that layer.')
        : 'Jewel shows where $src $where${clip ? ', and only inside Card' : ''}. $src itself is not drawn.';
    return Sect(
      title: 'Matte & clip',
      tone: Pop.toneBlend,
      mark: CardMark.overlay,
      brief: [none ? 'No matte' : 'Matte $src', if (!none) _liMatteModes[mode], if (clip) 'Clip to below'],
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: SizedBox(
            height: 124,
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                SizedBox(
                  width: 128,
                  child: Column(
                    children: [
                      for (var i = 0; i < _liStack.length; i++) ...[if (i > 0) const SizedBox(height: 4), Expanded(child: _LiStackTile(i))],
                    ],
                  ),
                ),
                const SizedBox(width: 6),
                const Expanded(child: _LiMattePreview(key: ValueKey('li_matte_preview'))),
              ],
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.only(top: 4),
          child: Row(
            children: [
              Expanded(
                child: SizedBox(
                  height: Pop.cell,
                  child: Dis(
                    child: Opacity(
                      opacity: none ? .5 : 1,
                      child: Segmented(
                        items: _liMatteWords,
                        index: none ? -1 : mode,
                        expand: true,
                        onChanged: (i) {
                          if (none) doc.s2['li_d.src'] = 'Title';
                          doc.str('li_d.mode', _liMatteModes[i]);
                        },
                      ),
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 6),
              _LiChip('Clip to below', on: clip, tone: Pop.toneBlend, onTap: () => doc.flag('li_d.clip', !clip)),
            ],
          ),
        ),
        Cap(say),
      ],
    );
  }
}

class _LiStackTile extends StatelessWidget {
  const _LiStackTile(this.i);
  final int i;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, name = _liStack[i], me = i == 1, used = doc.s2['li_d.src'] == name;
    final clip = doc.b['li_d.clip'] ?? false, on = !me && !x.cfg.locked;
    final role = me ? 'This layer' : (used ? 'Matte · hidden' : (i == 2 && clip ? 'Clip base' : (i == 0 ? 'Above' : 'Below')));
    return Hov(
      onTap: on ? () => doc.str('li_d.src', used ? 'None' : name) : null,
      cursor: on ? SystemMouseCursors.click : SystemMouseCursors.basic,
      builder: (_, h) => Container(
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(
          color: used ? Grey.g20 : (h && on ? Grey.g15 : Pop.well),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(color: used ? Pop.toneBlend : (me ? Grey.g38 : const Color(0x00000000))),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(width: 44, child: CustomPaint(painter: _LiThumb(i, used))),
            const SizedBox(width: 6),
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(name, maxLines: 1, style: T.name(me || used || h ? Grey.g100 : Grey.g76).copyWith(fontWeight: FontWeight.w600)),
                  const SizedBox(height: 3),
                  Text(role, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(used ? Pop.toneBlend : Grey.g56)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _LiThumb extends CustomPainter {
  const _LiThumb(this.i, this.dim);
  final int i;
  final bool dim;
  @override
  void paint(Canvas c, Size s) {
    final r = Offset.zero & s;
    _liChecker(c, r, 4);
    c.save();
    c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
    c.saveLayer(r, Paint()..color = Color.fromRGBO(0, 0, 0, dim ? .55 : 1));
    _liLayer(c, r, i);
    c.restore();
    c.restore();
  }

  @override
  bool shouldRepaint(_LiThumb o) => o.i != i || o.dim != dim;
}

/// contract: a click on the picture steps the matte mode (with no matte it takes the layer above, as a matte usually is).
class _LiMattePreview extends StatelessWidget {
  const _LiMattePreview({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, src = doc.s2['li_d.src'] ?? 'None', clip = doc.b['li_d.clip'] ?? false;
    final mode = math.max(0, _liMatteModes.indexOf(doc.s2['li_d.mode'] ?? 'Alpha'));
    return Hov(
      onTap: x.cfg.locked ? null : () => src == 'None' ? doc.str('li_d.src', 'Title') : doc.str('li_d.mode', _liMatteModes[(mode + 1) % _liMatteModes.length]),
      builder: (_, h) => CustomPaint(painter: _LiMattePaint(_liStack.indexOf(src), mode, clip, h)),
    );
  }
}

class _LiMattePaint extends CustomPainter {
  const _LiMattePaint(this.src, this.mode, this.clip, this.hot);
  final int src, mode;
  final bool clip, hot;
  @override
  void paint(Canvas c, Size s) {
    final box = Offset.zero & s, r = box.deflate(5);
    c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(6)), Paint()..color = hot ? Grey.g20 : Pop.well);
    _liChecker(c, r, 6);
    c.save();
    c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)));
    if (src != 2) _liLayer(c, r, 2);
    c.saveLayer(r, Paint());
    _liLayer(c, r, 1);
    if (clip) {
      c.saveLayer(r, Paint()..blendMode = BlendMode.dstIn);
      _liLayer(c, r, 2);
      c.restore();
    }
    if (src >= 0) {
      final p = Paint()..blendMode = mode.isOdd ? BlendMode.dstOut : BlendMode.dstIn;
      if (mode >= 2) p.colorFilter = _liLuma;
      c.saveLayer(r, p);
      _liLayer(c, r, src);
      c.restore();
    }
    c.restore();
    if (src != 0) _liLayer(c, r, 0);
    c.restore();
    final tag = [if (src >= 0) '${_liMatteModes[mode]} · ${_liStack[src]}', if (clip) 'Clip'].join('  ·  ');
    if (tag.isNotEmpty)
      _liText(c, tag, Offset(r.left + 6, r.top + 5), T.label(Grey.g95).copyWith(fontWeight: FontWeight.w600), bg: Grey.g07.withValues(alpha: .8));
    final hint = src < 0 ? 'Click: matte with Title' : 'Click: next mode';
    if (hot) _liText(c, hint, Offset(r.right - 6, r.bottom - 15), T.label(Grey.g95), ax: 1, bg: Grey.g07.withValues(alpha: .8));
  }

  @override
  bool shouldRepaint(_LiMattePaint o) => o.src != src || o.mode != mode || o.clip != clip || o.hot != hot;
}
