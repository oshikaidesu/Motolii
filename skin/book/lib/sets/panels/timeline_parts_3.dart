// Part of timeline_parts.dart (cut there by size, as it was).
part of 'timeline_parts.dart';

class _TransportState extends State<_Transport> with SingleTickerProviderStateMixin {
  late final Ticker ticker = createTicker(_tick);
  late bool playing = widget.playing, loop = widget.loop;
  double frame = 0;
  Duration last = Duration.zero;

  @override
  void initState() {
    super.initState();
    if (playing) ticker.start();
  }

  void _tick(Duration d) {
    final dt = (d - last).inMicroseconds / 1e6;
    last = d;
    setState(() {
      frame += dt * widget.fps;
      if (frame >= widget.total) {
        if (loop) {
          frame = 0;
        } else {
          frame = widget.total.toDouble();
          playing = false;
          ticker.stop();
        }
      }
    });
  }

  void toggle() {
    setState(() => playing = !playing);
    if (playing) {
      if (frame >= widget.total) frame = 0;
      last = Duration.zero;
      ticker.start();
    } else {
      ticker.stop();
    }
  }

  void seek(double f) => setState(() => frame = f.clamp(0, widget.total.toDouble()));

  @override
  void dispose() {
    ticker.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6)),
        child: Column(children: [
          Row(children: [
            TpIconButton(TpGlyph.toStart, size: 24, onTap: () => seek(0)),
            TpIconButton(TpGlyph.stepBack, size: 24, onTap: () => seek(frame.roundToDouble() - 1)),
            const SizedBox(width: 4),
            TpHit(
              onTap: toggle,
              builder: (_, h, d) => AnimatedContainer(
                duration: tpFast,
                curve: Curves.easeOut,
                width: 28,
                height: 24,
                alignment: Alignment.center,
                decoration: BoxDecoration(color: playing ? (d ? N.g38 : N.g26) : C.play.withValues(alpha: d ? .1 : h ? .24 : .16), borderRadius: BorderRadius.circular(6)),
                child: TpGlyphView(playing ? TpGlyph.pause : TpGlyph.play, playing ? N.g95 : C.play),
              ),
            ),
            const SizedBox(width: 4),
            TpIconButton(TpGlyph.stepFwd, size: 24, onTap: () => seek(frame.roundToDouble() + 1)),
            TpIconButton(TpGlyph.toEnd, size: 24, onTap: () => seek(widget.total.toDouble())),
            const SizedBox(width: 8),
            TpIconButton(TpGlyph.loop, size: 24, on: loop, onTap: () => setState(() => loop = !loop)),
            const Spacer(),
            Text(tpTime(frame, widget.fps), style: T.value(N.g95).copyWith(fontSize: 14)),
            const SizedBox(width: 6),
            Text('/ ${tpTime(widget.total, widget.fps)}', style: T.value(N.g56)),
          ]),
          const SizedBox(height: 6),
          Expanded(
            child: LayoutBuilder(builder: (context, box) {
              void drag(Offset p) => seek(p.dx / box.maxWidth * widget.total);
              return MouseRegion(
                cursor: SystemMouseCursors.resizeLeftRight,
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onPanDown: (d) => drag(d.localPosition),
                  onPanUpdate: (d) => drag(d.localPosition),
                  child: CustomPaint(size: Size.infinite, painter: _Scrub(frame / widget.total, widget.total, widget.fps)),
                ),
              );
            }),
          ),
        ]),
      );
}

class _Scrub extends CustomPainter {
  const _Scrub(this.t, this.total, this.fps);
  final double t;
  final int total, fps;
  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2, p = Paint(), x = (s.width * t).roundToDouble();
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(0, cy - 3, s.width, cy + 3), const Radius.circular(3)), p..color = N.g07);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(0, cy - 3, x, cy + 3), const Radius.circular(3)), p..color = N.g26);
    for (var f = fps; f < total; f += fps) {
      c.drawRect(Rect.fromLTWH((s.width * f / total).roundToDouble(), cy + 4, 1, 3), p..color = N.g38);
    }
    c.drawRect(Rect.fromLTWH(x, cy - 7, 1, 14), p..color = C.playhead);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(x + .5, cy), width: 7, height: 12), const Radius.circular(3)), p..color = C.playhead);
  }

  @override
  bool shouldRepaint(_Scrub o) => o.t != t || o.total != total || o.fps != fps;
}

// ---------------------------------------------------------------- 13. lane header

class _HeaderStack extends StatelessWidget {
  const _HeaderStack({super.key, required this.on});
  final bool on;
  @override
  Widget build(BuildContext context) => Column(children: [
        _band(0, _LaneHeader(name: 'Position X', unit: 'px', value: 640, step: 1, on: on)),
        _band(1, _LaneHeader(name: 'Opacity', unit: '%', value: 100, step: .5, on: on)),
        _band(0, _LaneHeader(name: 'Rotation', unit: '°', value: 12.5, step: .1, on: on)),
        _band(1, _LaneHeader(name: 'Scale', unit: '%', value: 100, step: .5, on: false)),
      ]);
}

class _LaneHeader extends StatefulWidget {
  const _LaneHeader({required this.name, required this.unit, required this.value, required this.step, required this.on});
  final String name, unit;
  final double value, step;
  final bool on;
  @override
  State<_LaneHeader> createState() => _LaneHeaderState();
}

class _LaneHeaderState extends State<_LaneHeader> {
  late bool on = widget.on, key = false;
  late double v = widget.value;
  bool hoverVal = false, scrub = false;

  @override
  Widget build(BuildContext context) => TpHit(
        cursor: SystemMouseCursors.basic,
        builder: (_, h, __) => Padding(
          padding: const EdgeInsets.only(left: 22, right: 6),
          child: Row(children: [
            TpIconButton(TpGlyph.stopwatch, on: on, onTap: () => setState(() {
                  on = !on;
                  if (!on) key = false;
                })),
            const SizedBox(width: 6),
            Expanded(child: Text(widget.name, style: T.name(on ? N.g91 : N.g76), maxLines: 1, overflow: TextOverflow.ellipsis)),
            AnimatedOpacity(
              duration: tpFast,
              curve: Curves.easeOut,
              opacity: on ? 1 : 0,
              child: IgnorePointer(
                ignoring: !on,
                child: Row(children: [
                  TpIconButton(TpGlyph.stepBack, size: 16, glyph: 10, rest: N.g63),
                  TpIconButton(key ? TpGlyph.keyFilled : TpGlyph.keyOutline, size: 16, glyph: 10, rest: N.g63, on: key, onTap: () => setState(() => key = !key)),
                  TpIconButton(TpGlyph.stepFwd, size: 16, glyph: 10, rest: N.g63),
                ]),
              ),
            ),
            const SizedBox(width: 6),
            MouseRegion(
              cursor: SystemMouseCursors.resizeLeftRight,
              onEnter: (_) => setState(() => hoverVal = true),
              onExit: (_) => setState(() => hoverVal = false),
              child: GestureDetector(
                onPanStart: (_) => setState(() => scrub = true),
                onPanEnd: (_) => setState(() => scrub = false),
                onPanUpdate: (d) => setState(() => v += d.delta.dx * widget.step),
                child: AnimatedContainer(
                  duration: tpFast,
                  curve: Curves.easeOut,
                  height: 17,
                  width: 58,
                  padding: const EdgeInsets.symmetric(horizontal: 5),
                  alignment: Alignment.centerRight,
                  decoration: BoxDecoration(color: scrub ? N.g07 : hoverVal ? N.g10 : const Color(0x00131313), borderRadius: BorderRadius.circular(3)),
                  child: Text('${v.toStringAsFixed(1)}${widget.unit}', style: T.value(scrub ? N.g100 : N.g95).copyWith(fontSize: 10.5)),
                ),
              ),
            ),
          ]),
        ),
      );
}

// ---------------------------------------------------------------- 14. in context

class _Context extends StatefulWidget {
  const _Context({super.key, required this.ppf, required this.look, required this.head});
  final double ppf;
  final Look look;
  final int head;
  @override
  State<_Context> createState() => _ContextState();
}

class _ContextState extends State<_Context> {
  late int head = widget.head;
  int sel = 2;
  int? hov;
  double marker = 48;

  @override
  Widget build(BuildContext context) {
    final layers = <Layer>[...sampleLayers];
    return Padding(padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8), child: LayoutBuilder(builder: (context, box) {
      final laneW = box.maxWidth - TL.label;
      final ppf = math.min(widget.ppf, (laneW - 8) / 120);
      void scrub(Offset p) => setState(() => head = ((p.dx - TL.label) / ppf).round().clamp(0, (laneW / ppf).floor()));
      return Stack(children: [
        Column(children: [
          GestureDetector(
            onPanDown: (d) => scrub(d.localPosition),
            onPanUpdate: (d) => scrub(d.localPosition),
            child: SizedBox(
              height: TL.ruler,
              child: Row(children: [
                Container(width: TL.label, color: N.g07, padding: const EdgeInsets.only(left: 10), alignment: Alignment.centerLeft, child: Text(tpTime(head, 30), style: T.value(C.playhead))),
                Expanded(child: CustomPaint(size: Size.infinite, painter: _RulerPainter(ppf, 30, headX: head * ppf))),
              ]),
            ),
          ),
          SizedBox(
            height: 16,
            child: Row(children: [
              SizedBox(width: TL.label),
              Expanded(
                child: ClipRect(
                  child: Stack(children: [
                    Positioned(left: marker * ppf + 1, top: 0, child: _Flag(label: 'Drop', onDrag: (dx) => setState(() => marker = (marker + dx / ppf).clamp(0.0, 100.0)))),
                  ]),
                ),
              ),
            ]),
          ),
          for (var i = 0; i < layers.length; i++)
            MouseRegion(
              onEnter: (_) => setState(() => hov = i),
              onExit: (_) => setState(() => hov = null),
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => setState(() => sel = i),
                child: Row(children: [
                  SizedBox(width: TL.label, child: TpLabelCell(name: layers[i].name, fam: layers[i].fam, band: i.isEven ? N.bandA : N.bandB, selected: sel == i, hover: hov == i, caret: false)),
                  Expanded(
                    child: Container(
                      height: TL.height,
                      decoration: BoxDecoration(color: i.isEven ? N.bandA : N.bandB, border: const Border(bottom: BorderSide(color: N.rowLine))),
                      child: ClipRect(child: CustomPaint(
                        size: Size.infinite,
                        painter: TpLanePainter(fam: layers[i].fam, kind: sel == i ? TpBar.selected : TpBar.normal, start: layers[i].start, end: layers[i].end, keys: layers[i].keys, head: head, ppf: ppf, hover: hov == i, look: widget.look),
                      )),
                    ),
                  ),
                ]),
              ),
            ),
          Expanded(child: Row(children: [SizedBox(width: TL.label), Expanded(child: CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: Fam.face, showBar: false, ppf: ppf)))])),
        ]),
        Positioned(left: TL.label + marker * ppf, top: TL.ruler, bottom: 0, width: 1, child: IgnorePointer(child: ColoredBox(color: N.g63.withValues(alpha: .4)))),
        Positioned(
          left: TL.label,
          right: 0,
          top: 0,
          bottom: 0,
          child: IgnorePointer(child: CustomPaint(painter: _HeadLine(head * ppf, 0, widget.look, false))),
        ),
      ]);
    }));
  }
}
