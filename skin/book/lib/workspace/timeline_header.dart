// timeline_header.dart: the row above the tracks: the document's name, the transport, the time, the zoom.
part of 'timeline.dart';

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
    final big = T.value(Grey.g76).copyWith(fontSize: T.px(14), fontWeight: FontWeight.w500, fontFeatures: const [FontFeature.tabularFigures()]);
    final picked = ws.keys.length, layer = ws.layer;
    return Container(
      height: WtM.header,
      padding: EdgeInsets.symmetric(horizontal: WsT.inset),
      decoration: BoxDecoration(
        color: WsT.body,
        border: Border(bottom: BorderSide(color: WsT.line)),
      ),
      child: LayoutBuilder(
        builder: (context, box) {
          final roomy = box.maxWidth >= 1000, mid = box.maxWidth >= 820, slide = box.maxWidth >= 600;   // (narrower: the zoom by its buttons alone, as the document's name and length go)
          return Row(
            children: [
              if (mid) ...[_Chip(ws.document?.replaceFirst(RegExp(r'\.motolii$'), '') ?? 'Main', mono: false), SizedBox(width: Shape.s(12))], // (joined: the document's name)
              _Btn(TpGlyph.toStart, key: const ValueKey('ws-tl-start'), onTap: () => onStep(-ws.frame)),
              SizedBox(width: Shape.s(2)),
              _Btn(TpGlyph.stepBack, key: const ValueKey('ws-tl-back'), onTap: () => onStep(-1)),
              SizedBox(width: Shape.s(2)),
              ValueListenableBuilder<int>(   // (only the button follows play: the rows are not built again)
                valueListenable: ws.on[WsAspect.playing]!,
                builder: (_, _, _) => _Btn(ws.playing ? TpGlyph.pause : TpGlyph.play, key: const ValueKey('ws-tl-play'), width: Shape.z(34), on: ws.playing, rest: WsT.accent, onTap: onToggle),
              ),
              SizedBox(width: Shape.s(2)),
              _Btn(TpGlyph.stepFwd, key: const ValueKey('ws-tl-fwd'), onTap: () => onStep(1)),
              SizedBox(width: Shape.s(2)),
              _Btn(TpGlyph.loop, key: const ValueKey('ws-tl-loop'), on: loop, onTap: onLoop),
              SizedBox(width: Shape.s(12)),
              ValueListenableBuilder<int>(valueListenable: ws.moment, builder: (_, _, _) => Text(ws.timecode(), style: big)),   // (one number, one weight: not three colours)
              SizedBox(width: Shape.s(8)),
              if (roomy) ...[Text('/ ${ws.timecode(Ws.duration)}', style: T.value(Grey.g56)), SizedBox(width: Shape.s(10))],
              _Chip('${Ws.fps} fps'),
              SizedBox(width: Shape.s(12)),
              if (mid && layer != null)
                Flexible(
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      WtKindChip(layer.kind, tone: layer.tone),
                      SizedBox(width: Shape.s(6)),
                      Flexible(
                        child: FittedBox( // (drawn smaller, never cut)
                          fit: BoxFit.scaleDown,
                          alignment: Alignment.centerLeft,
                          child: Text(
                            picked > 0 ? '${layer.name}  ·  $picked key${picked == 1 ? '' : 's'}' : '${layer.name}  ·  ${layer.keys.length} keys',
                            style: T.label(Grey.g76),
                            maxLines: 1,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              const Spacer(),
              _Btn(TpGlyph.minus, key: const ValueKey('ws-tl-zoom-out'), size: Shape.z(22), onTap: () => onZoom(zoom / 1.5)),
              SizedBox(width: Shape.s(4)),
              if (slide) ...[_ZoomSlider(zoom: zoom, onZoom: onZoom), SizedBox(width: Shape.s(4))],
              _Btn(TpGlyph.plus, key: const ValueKey('ws-tl-zoom-in'), size: Shape.z(22), onTap: () => onZoom(zoom * 1.5)),
              SizedBox(width: Shape.s(4)),
              _Btn(TpGlyph.fit, key: const ValueKey('ws-tl-fit'), size: Shape.z(22), width: Shape.z(46), label: 'Fit', lit: zoom <= 1.001, onTap: () => onZoom(1)),
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
    final bg = on ? WsT.accent : (_d || widget.lit ? Grey.g26 : (_h ? WsT.raised : const Color(0x00000000)));   // (flat at rest: a button shows itself when the hand is on it)
    final ink = on ? WsT.onAccent : (_h || widget.lit ? Grey.g100 : (widget.rest ?? Grey.g56));
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
          decoration: BoxDecoration(color: bg, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CustomPaint(size: const Size.square(11), painter: TpGlyphPainter(widget.g, ink)),
              if (widget.label case final l?) ...[SizedBox(width: Shape.s(4)), Text(l, style: T.micro(ink).copyWith(fontWeight: FontWeight.w700))],
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
    height: Shape.z(20),
    padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
    child: Text(text, style: mono ? T.value(Grey.g91).copyWith(fontSize: T.px(10.5)) : T.micro(Grey.g91).copyWith(fontWeight: FontWeight.w600)),
  );
}

class _ZoomSlider extends StatelessWidget {
  const _ZoomSlider({required this.zoom, required this.onZoom});
  final double zoom;
  final void Function(double z, {int? anchor}) onZoom;
  static double get w => Shape.z(96.0);
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
          height: Shape.z(22),
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
    c.drawRRect(RRect.fromLTRBR(0, cy - 3, s.width, cy + 3, Shape.r(3)), p..color = WsT.well);
    c.drawRRect(RRect.fromLTRBR(0, cy - 3, x, cy + 3, Shape.r(3)), p..color = Grey.g44);
    c.drawCircle(Offset(x, cy), 6, p..color = Grey.g91);
  }

  @override
  bool shouldRepaint(_ZoomTrack o) => o.t != t;
}
