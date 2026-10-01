// set: timeline_parts. The pieces of a motion-graphics timeline, each alone and in context. Every size is a fraction of TL.* (DESIGN.md section 4).
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../kit.dart';
import '../parts/controls.dart';
import '../parts/timeline.dart' show Layer, sampleLayers;
import '../tokens.dart';
import 'timeline_parts_parts.dart';

WidgetbookComponent timelinePartsSet() => WidgetbookComponent(name: 'timeline_parts', useCases: [
      uc('Layer label cell', (c) {
        final state = c.knobs.object.dropdown<CellState>(label: 'State', options: CellState.values, initialOption: CellState.normal, labelBuilder: (s) => s.name);
        final fam = _fam(c, 'Colour', Fam.stagger);
        final name = c.knobs.string(label: 'Name', initialValue: fam.name);
        final caret = c.knobs.boolean(label: 'Expand caret', initialValue: true);
        return TpLabelCell(key: ValueKey('$state$caret'), name: name, fam: fam, state: state, caret: caret);
      }, width: TL.label, height: TL.height),
      uc('Label cell / all states', (c) {
        final fam = _fam(c, 'Colour', Fam.along);
        return Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
          for (var i = 0; i < CellState.values.length; i++)
            Row(children: [
              SizedBox(width: TL.label, child: TpLabelCell(name: fam.name, fam: fam, state: CellState.values[i], band: i.isEven ? N.bandA : N.bandB)),
              const SizedBox(width: 12),
              Text(CellState.values[i].name, style: T.label(N.g56)),
            ]),
        ]);
      }, width: 300, height: 5 * TL.height),
      uc('Keyframe diamonds', (c) {
        final r = c.knobs.double.slider(label: 'Large key radius', initialValue: 15, min: 6, max: 24);
        final fam = _fam(c, 'Bar colour', Fam.stagger);
        final link = c.knobs.boolean(label: 'Link line', initialValue: true);
        return _KeyShowcase(r: r, fam: fam, link: link);
      }, width: 680, height: 290),
      uc('Layer bars', (c) {
        final fam = _fam(c, 'Colour', Fam.scatter);
        final look = lookKnob(c);
        final ppf = c.knobs.double.slider(label: 'Pixels per frame', initialValue: 6, min: 2, max: 12);
        return _BarGallery(fam: fam, look: look, ppf: ppf);
      }, width: 680, height: 7 * TL.height),
      uc('Playhead', (c) {
        final head = c.knobs.int.slider(label: 'Frame', initialValue: 42, min: 0, max: 118);
        final fps = c.knobs.object.dropdown<int>(label: 'fps', options: const [24, 30, 60], initialOption: 30, labelBuilder: (v) => '$v');
        final ppf = c.knobs.double.slider(label: 'Pixels per frame', initialValue: 6, min: 3, max: 10);
        final grab = c.knobs.boolean(label: 'Held', initialValue: false);
        return _PlayheadDemo(key: ValueKey('$head$fps$ppf'), head: head, fps: fps, ppf: ppf, look: lookKnob(c), held: grab);
      }, width: 680, height: 16 + TL.ruler + 3 * TL.height),
      uc('Marker flag', (c) {
        final sel = c.knobs.boolean(label: 'Selected', initialValue: false);
        final label = c.knobs.string(label: 'Label', initialValue: 'Drop');
        final frame = c.knobs.int.slider(label: 'Frame', initialValue: 40, min: 0, max: 100);
        return _MarkerDemo(key: ValueKey('$frame'), selected: sel, label: label, frame: frame.toDouble());
      }, width: 680, height: 120),
      uc('Work area bar', (c) {
        final a = c.knobs.int.slider(label: 'In', initialValue: 20, min: 0, max: 110);
        final b = c.knobs.int.slider(label: 'Out', initialValue: 92, min: 10, max: 118);
        return _WorkArea(key: ValueKey('$a$b'), a: a, b: math.max(b, a + 4));
      }, width: 680, height: 150),
      uc('Ruler / zoom-adaptive ticks', (c) {
        final ppf = c.knobs.double.slider(label: 'Pixels per frame', initialValue: 6, min: .3, max: 60);
        final fps = c.knobs.object.dropdown<int>(label: 'fps', options: const [24, 30, 60], initialOption: 30, labelBuilder: (v) => '$v');
        final cmp = c.knobs.boolean(label: 'Show 1/4x and 4x', initialValue: true);
        final grid = c.knobs.boolean(label: 'Grid below', initialValue: true);
        return _RulerDemo(ppf: ppf, fps: fps, compare: cmp, grid: grid);
      }, width: 720, height: 230),
      uc('Mini curve editor', (c) {
        final p = c.knobs.object.dropdown<String>(label: 'Preset', options: _presets.keys.toList(), initialOption: 'ease in-out', labelBuilder: (s) => s);
        final t = c.knobs.double.slider(label: 'Time t', initialValue: .5, min: 0, max: 1);
        return _CurveEditor(key: ValueKey(p), preset: p, t: t);
      }, width: 260, height: 300),
      uc('Property lane / Position X Y', (c) {
        final head = c.knobs.int.slider(label: 'Playhead frame', initialValue: 50, min: 0, max: 118);
        final ppf = c.knobs.double.slider(label: 'Pixels per frame', initialValue: 4.5, min: 2, max: 8);
        final ease = c.knobs.object.dropdown<TpInterp>(label: 'Interpolation', options: TpInterp.values, initialOption: TpInterp.eased, labelBuilder: (k) => k.name);
        return _PropLanes(key: ValueKey('$ease'), head: head, ppf: ppf, kind: ease);
      }, width: 700, height: 20 + 2 * TL.height),
      uc('Time zoom slider', (c) {
        final fit = c.knobs.double.slider(label: 'Fit zoom (px/frame)', initialValue: 5.2, min: .6, max: 30);
        final start = c.knobs.double.slider(label: 'Start zoom (px/frame)', initialValue: 8, min: .6, max: 30);
        return _ZoomSlider(key: ValueKey('$fit$start'), fit: fit, start: start);
      }, width: 340, height: 28),
      uc('Scrub / transport strip', (c) {
        final playing = c.knobs.boolean(label: 'Playing', initialValue: false);
        final loop = c.knobs.boolean(label: 'Loop', initialValue: true);
        final fps = c.knobs.object.dropdown<int>(label: 'fps', options: const [24, 30, 60], initialOption: 30, labelBuilder: (v) => '$v');
        final total = c.knobs.int.slider(label: 'Duration (frames)', initialValue: 240, min: 30, max: 900);
        return _Transport(key: ValueKey('$playing$loop$fps$total'), playing: playing, loop: loop, fps: fps, total: total);
      }, width: 640, height: 68),
      uc('Lane header', (c) {
        final w = c.knobs.double.slider(label: 'Width', initialValue: 220, min: 180, max: 320);
        final on = c.knobs.boolean(label: 'Stopwatch on', initialValue: true);
        return SizedBox(width: w, child: _HeaderStack(key: ValueKey('$on'), on: on));
      }, width: 320, height: 4 * TL.height),
      uc('In context / full timeline', (c) {
        final ppf = c.knobs.double.slider(label: 'Pixels per frame', initialValue: 6, min: 3, max: 8);
        final look = lookKnob(c);
        final head = c.knobs.int.slider(label: 'Frame', initialValue: 60, min: 0, max: 100);
        return _Context(key: ValueKey('$head'), ppf: ppf, look: look, head: head);
      }, width: 720, height: 16 + TL.ruler + 6 * TL.height + 24),
    ]);

Fam _fam(BuildContext c, String label, Fam initial) => c.knobs.object.dropdown<Fam>(label: label, options: Fam.all, initialOption: initial, labelBuilder: (f) => f.name);

Widget _band(int i, Widget child, {double h = TL.height}) => Container(height: h, decoration: BoxDecoration(color: i.isEven ? N.bandA : N.bandB, border: const Border(bottom: BorderSide(color: N.rowLine))), child: child);

Size _measure(String t, TextStyle s) {
  final tp = TextPainter(text: TextSpan(text: t, style: s), textDirection: TextDirection.ltr)..layout();
  final z = tp.size;
  tp.dispose();
  return z;
}

// ---------------------------------------------------------------- 1. label cell

enum CellState { normal, hover, selected, hidden, locked }

class TpLabelCell extends StatefulWidget {
  const TpLabelCell({super.key, required this.name, required this.fam, this.state = CellState.normal, this.caret = true, this.band = N.bandA, this.selected, this.hover});
  final String name;
  final Fam fam;
  final CellState state;
  final bool caret;
  final Color band;
  final bool? selected, hover; // when the row owns these
  @override
  State<TpLabelCell> createState() => _TpLabelCellState();
}

class _TpLabelCellState extends State<TpLabelCell> {
  late bool vis = widget.state != CellState.hidden, locked = widget.state == CellState.locked, solo = false, open = true;

  @override
  Widget build(BuildContext context) => TpHit(
        cursor: SystemMouseCursors.basic,
        builder: (_, h, __) {
          final sel = widget.selected ?? widget.state == CellState.selected;
          final hot = h || (widget.hover ?? widget.state == CellState.hover);
          final ink = !vis ? N.g56 : sel ? N.g100 : locked ? N.g76 : N.g91;
          final cell = AnimatedContainer(
            duration: tpFast,
            curve: Curves.easeOut,
            height: TL.height,
            padding: const EdgeInsets.only(left: 4, right: 6),
            decoration: BoxDecoration(color: sel ? N.g20 : hot ? N.g15 : widget.band, border: const Border(bottom: BorderSide(color: N.rowLine))),
            child: Row(children: [
              widget.caret ? TpIconButton(open ? TpGlyph.caretD : TpGlyph.caretR, size: 16, glyph: 10, onTap: () => setState(() => open = !open)) : const SizedBox(width: 16),
              const SizedBox(width: 4),
              AnimatedContainer(duration: tpFast, width: 8, height: 8, decoration: BoxDecoration(color: widget.fam.c.withValues(alpha: vis ? 1 : .3), borderRadius: BorderRadius.circular(2))),
              const SizedBox(width: 6),
              Expanded(child: Text(widget.name, style: T.name(ink), maxLines: 1, overflow: TextOverflow.ellipsis)),
              TpIconButton(vis ? TpGlyph.eye : TpGlyph.eyeOff, dim: !vis, onTap: () => setState(() => vis = !vis)),
              const SizedBox(width: 2),
              TpIconButton(TpGlyph.solo, on: solo, dim: !hot && !sel, onTap: () => setState(() => solo = !solo)),
              const SizedBox(width: 2),
              TpIconButton(locked ? TpGlyph.lock : TpGlyph.unlock, on: locked, dim: !hot && !sel && !locked, onTap: () => setState(() => locked = !locked)),
            ]),
          );
          return Stack(children: [
            cell,
            Positioned(left: 0, top: 0, bottom: 1, width: 2, child: AnimatedOpacity(duration: tpFast, curve: Curves.easeOut, opacity: sel ? 1 : 0, child: const ColoredBox(color: N.g95))),
          ]);
        },
      );
}

// ---------------------------------------------------------------- 3. keys

class _KeyShowcase extends StatelessWidget {
  const _KeyShowcase({required this.r, required this.fam, required this.link});
  final double r;
  final Fam fam;
  final bool link;
  @override
  Widget build(BuildContext context) {
    const states = TpKey.values, interps = TpInterp.values;
    Widget caps(List<String> t, double lead) => Row(children: [SizedBox(width: lead), for (final k in t) SizedBox(width: 130, child: Center(child: Text(k, style: T.label(N.g63))))]);
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text('KEY STATES', style: T.micro(N.g56)),
      const SizedBox(height: 6),
      SizedBox(height: 70, child: CustomPaint(size: Size.infinite, painter: _BigKeys(r, fam, link, false))),
      caps([for (final k in states) k.name], 0),
      const SizedBox(height: 16),
      Text('SEGMENT MARKS (on the link line, between two keys)', style: T.micro(N.g56)),
      const SizedBox(height: 6),
      SizedBox(height: 70, child: CustomPaint(size: Size.infinite, painter: _BigKeys(r, fam, link, true))),
      caps([for (final k in interps) k.name], 65),
      const SizedBox(height: 16),
      Text('AT ROW SCALE (23 px)', style: T.micro(N.g56)),
      const SizedBox(height: 6),
      SizedBox(
        height: TL.height,
        child: CustomPaint(
          size: Size.infinite,
          painter: TpLanePainter(fam: fam, start: 0, end: 650, ppf: 1, grid: false, link: link, keys: [for (var i = 0; i < 5; i++) 65.0 + 130 * i], interps: interps),
        ),
      ),
    ]);
  }
}

class _BigKeys extends CustomPainter {
  const _BigKeys(this.r, this.fam, this.link, this.segs);
  final double r;
  final Fam fam;
  final bool link, segs;
  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2;
    final h = math.min(r * 2 + 10, 44.0), bar = Rect.fromLTRB(0, cy - h / 2, 650, cy + h / 2);
    tpBar(c, bar, fam, TpBar.normal);
    final n = segs ? 5 : TpKey.values.length;
    if (link) tpLink(c, [for (var i = 0; i < n; i++) 65.0 + 130 * i], cy, segs ? {} : {for (var i = 0; i < n; i++) if (TpKey.values[i] == TpKey.ghost) i}, r, .7);
    if (segs) {
      for (var i = 0; i < 4; i++) {
        tpSeg(c, Offset(130 + 130.0 * i, cy), TpInterp.values[i], r / 7);
      }
    }
    for (var i = 0; i < n; i++) {
      tpKey(c, Offset(65.0 + 130 * i, cy), r, segs ? TpKey.normal : TpKey.values[i]);
    }
  }

  @override
  bool shouldRepaint(_BigKeys o) => o.r != r || o.fam != fam || o.link != link || o.segs != segs;
}

// ---------------------------------------------------------------- 4. bars

class _BarGallery extends StatelessWidget {
  const _BarGallery({required this.fam, required this.look, required this.ppf});
  final Fam fam;
  final Look look;
  final double ppf;
  @override
  Widget build(BuildContext context) => Column(children: [
        for (var i = 0; i < TpBar.values.length; i++)
          _band(
            i,
            TpHit(
              cursor: SystemMouseCursors.basic,
              builder: (_, h, __) {
                final k = TpBar.values[i];
                return Row(children: [
                  SizedBox(width: 80, child: Padding(padding: const EdgeInsets.only(left: 8), child: Text(k.name, style: T.label(N.g63)))),
                  Expanded(
                    child: CustomPaint(
                      size: Size.infinite,
                      painter: TpLanePainter(
                        fam: fam,
                        kind: k,
                        start: 8,
                        end: k == TpBar.trimmed ? 70 : 90,
                        ppf: ppf,
                        hover: h,
                        look: look,
                        keys: k == TpBar.normal || k == TpBar.selected || k == TpBar.trimmed ? const [8, 40, 70] : const [],
                      ),
                    ),
                  ),
                ]);
              },
            ),
          ),
      ]);
}

// ---------------------------------------------------------------- shared ruler painter

class _RulerPainter extends CustomPainter {
  const _RulerPainter(this.ppf, this.fps, {this.headX});
  final double ppf;
  final int fps;
  final double? headX;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = N.g07);
    final major = tpMajor(ppf, fps), fine = tpFine(ppf, fps, major), p = Paint();
    for (var f = 0.0; f * ppf < s.width; f += fine) {
      final x = (f * ppf).roundToDouble();
      if (f % major == 0) {
        c.drawRect(Rect.fromLTWH(x, s.height - 6, 1, 6), p..color = N.g44);
        final label = major >= fps ? tpTime(f, fps).substring(0, 5) : tpTime(f, fps);
        final tp = TextPainter(text: TextSpan(text: label, style: T.value(N.g63).copyWith(fontSize: 10)), textDirection: TextDirection.ltr)..layout();
        if (headX == null || x + 3 + tp.width < headX! - 8 || x + 3 > headX! + 8) tp.paint(c, Offset(x + 3, 3));
        tp.dispose();
      } else {
        c.drawRect(Rect.fromLTWH(x, s.height - 3, 1, 3), p..color = N.g38);
      }
    }
  }

  @override
  bool shouldRepaint(_RulerPainter o) => o.ppf != ppf || o.fps != fps || o.headX != headX;
}

// ---------------------------------------------------------------- 5. playhead

class _PlayheadDemo extends StatelessWidget {
  const _PlayheadDemo({super.key, required this.head, required this.fps, required this.ppf, required this.look, required this.held});
  final int head, fps;
  final double ppf;
  final Look look;
  final bool held;
  @override
  Widget build(BuildContext context) => TpValue<int>(
        initial: head,
        builder: (context, f, set) => LayoutBuilder(builder: (context, box) {
          const flagH = 16.0;
          final x = (f * ppf).roundToDouble();
          void drag(Offset p) => set((p.dx / ppf).round().clamp(0, (box.maxWidth / ppf).floor()));
          final lit = held;
          return MouseRegion(
            cursor: SystemMouseCursors.resizeLeftRight,
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanDown: (d) => drag(d.localPosition),
              onPanUpdate: (d) => drag(d.localPosition),
              child: Stack(children: [
                Positioned(top: flagH, left: 0, right: 0, height: TL.ruler, child: CustomPaint(painter: _RulerPainter(ppf, fps, headX: x))),
                Positioned(
                  top: flagH + TL.ruler,
                  left: 0,
                  right: 0,
                  child: Column(children: [
                    for (var i = 0; i < 3; i++)
                      _band(i, CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: sampleLayers[i].fam, start: sampleLayers[i].start, end: sampleLayers[i].end, keys: sampleLayers[i].keys, ppf: ppf, fps: fps))),
                  ]),
                ),
                Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _HeadLine(x, flagH, look, lit)))),
                AnimatedPositioned(
                  duration: const Duration(milliseconds: 1),
                  left: x - 31,
                  top: 0,
                  child: IgnorePointer(child: _TimeFlag(text: tpTime(f, fps), look: look, lit: lit)),
                ),
              ]),
            ),
          );
        }),
      );
}

class _TimeFlag extends StatelessWidget {
  const _TimeFlag({required this.text, required this.look, required this.lit});
  final String text;
  final Look look;
  final bool lit;
  @override
  Widget build(BuildContext context) {
    final concept = look == Look.concept, glow = look == Look.glow;
    return AnimatedContainer(
      duration: tpFast,
      curve: Curves.easeOut,
      width: 62,
      height: 16,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: concept ? C.playhead : glow ? C.playhead.withValues(alpha: .22) : C.playhead.withValues(alpha: .85),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: lit ? N.g100 : glow ? C.playhead : const Color(0x00000000), width: 1),
      ),
      child: Text(text, style: T.value(glow ? N.g100 : N.g10).copyWith(fontSize: 10, fontWeight: FontWeight.w600)),
    );
  }
}

class _HeadLine extends CustomPainter {
  const _HeadLine(this.x, this.top, this.look, this.lit);
  final double x, top;
  final Look look;
  final bool lit;
  @override
  void paint(Canvas c, Size s) {
    if (look == Look.glow) c.drawRect(Rect.fromLTWH(x - 2, top, 5, s.height - top), Paint()..color = C.playhead.withValues(alpha: .35)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3));
    c.drawRect(Rect.fromLTWH(x, top, 1, s.height - top), Paint()..color = C.playhead);
    c.save();
    c.translate(0, top);
    tpHead(c, x + .5, lit: lit);
    c.restore();
  }

  @override
  bool shouldRepaint(_HeadLine o) => o.x != x || o.look != look || o.lit != lit;
}

// ---------------------------------------------------------------- 6. marker

class _Flag extends StatefulWidget {
  const _Flag({required this.label, this.selected = false, this.onDrag});
  final String label;
  final bool selected;
  final ValueChanged<double>? onDrag;
  @override
  State<_Flag> createState() => _FlagState();
}

class _FlagState extends State<_Flag> {
  bool h = false, d = false;
  @override
  Widget build(BuildContext context) {
    final w = _measure(widget.label, _flagText(false)).width + 14;
    return MouseRegion(
      cursor: SystemMouseCursors.grab,
      onEnter: (_) => setState(() => h = true),
      onExit: (_) => setState(() => h = false),
      child: GestureDetector(
        onPanDown: (_) => setState(() => d = true),
        onPanEnd: (_) => setState(() => d = false),
        onPanCancel: () => setState(() => d = false),
        onPanUpdate: (u) => widget.onDrag?.call(u.delta.dx),
        child: CustomPaint(
          size: Size(w + 5, 16),
          painter: _FlagPainter(widget.selected, h || d),
          child: SizedBox(width: w + 5, height: 16, child: Padding(padding: const EdgeInsets.only(left: 7), child: Align(alignment: Alignment.centerLeft, child: Text(widget.label, style: _flagText(widget.selected))))),
        ),
      ),
    );
  }
}

TextStyle _flagText(bool selected) => T.label(selected ? N.g10 : N.g95).copyWith(fontWeight: FontWeight.w500);

class _FlagPainter extends CustomPainter {
  const _FlagPainter(this.selected, this.hot);
  final bool selected, hot;
  @override
  void paint(Canvas c, Size s) {
    final p = Path()..moveTo(0, 0)..lineTo(s.width, 0)..lineTo(s.width - 5, s.height / 2)..lineTo(s.width, s.height)..lineTo(0, s.height)..close();
    c.drawPath(p, Paint()..color = selected ? tpAccent : hot ? N.g26 : N.g20);
  }

  @override
  bool shouldRepaint(_FlagPainter o) => o.hot != hot || o.selected != selected;
}

class _MarkerDemo extends StatelessWidget {
  const _MarkerDemo({super.key, required this.selected, required this.label, required this.frame});
  final bool selected;
  final String label;
  final double frame;
  @override
  Widget build(BuildContext context) => TpValue<double>(
        initial: frame,
        builder: (context, f, set) {
          const ppf = 6.0, fps = 30;
          List<Widget> marker(double fr, bool on, String t, [ValueChanged<double>? move]) => [
                Positioned(left: fr * ppf, top: 0, bottom: 0, width: 1, child: ColoredBox(color: on ? tpAccent.withValues(alpha: .7) : N.g63.withValues(alpha: .5))),
                Positioned(left: fr * ppf + 1, top: 0, child: _Flag(label: t, selected: on, onDrag: move == null ? null : (dx) => move(dx / ppf))),
              ];
          return ClipRect(
            child: Stack(children: [
              Positioned(top: 16, left: 0, right: 0, height: TL.ruler, child: CustomPaint(painter: _RulerPainter(ppf, fps))),
              Positioned(top: 16 + TL.ruler, bottom: 0, left: 0, right: 0, child: _band(0, CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: Fam.stagger, start: 4, end: 118, keys: const [4, 36, 60], ppf: ppf)), h: 64)),
              ...marker(88, false, 'Hit'),
              ...marker(f, selected, label, (d) => set((f + d).clamp(0.0, 100.0))),
            ]),
          );
        },
      );
}

// ---------------------------------------------------------------- 7. work area

class _WorkArea extends StatelessWidget {
  const _WorkArea({super.key, required this.a, required this.b});
  final int a, b;
  @override
  Widget build(BuildContext context) => TpValue<(int, int)>(
        initial: (a, b),
        builder: (context, v, set) {
          const ppf = 6.0, fps = 30, grab = 8.0, top = TL.ruler;
          final x0 = v.$1 * ppf, x1 = v.$2 * ppf;
          int fr(double dx) => (dx / ppf).round();
          Widget handle(double x, bool left) => Positioned(
                left: left ? x - grab : x,
                top: top,
                width: grab,
                height: 14,
                child: MouseRegion(
                  cursor: SystemMouseCursors.resizeLeftRight,
                  child: GestureDetector(
                    onPanUpdate: (d) => left ? set((math.min(v.$1 + fr(d.delta.dx), v.$2 - 2).clamp(0, 118), v.$2)) : set((v.$1, math.max(v.$2 + fr(d.delta.dx), v.$1 + 2).clamp(0, 118))),
                    child: _Grip(left: left),
                  ),
                ),
              );
          return Stack(children: [
            Positioned(top: 0, left: 0, right: 0, height: TL.ruler, child: CustomPaint(painter: const _RulerPainter(ppf, fps))),
            Positioned(top: top, left: 0, right: 0, height: 14, child: const ColoredBox(color: N.g13)),
            Positioned(
              top: top,
              left: x0,
              width: x1 - x0,
              height: 14,
              child: MouseRegion(
                cursor: SystemMouseCursors.grab,
                child: GestureDetector(
                  onPanUpdate: (d) {
                    final n = fr(d.delta.dx);
                    final len = v.$2 - v.$1, s = (v.$1 + n).clamp(0, 118 - len);
                    set((s, s + len));
                  },
                  child: const ColoredBox(color: N.g26),
                ),
              ),
            ),
            Positioned(
              top: top + 14,
              left: 0,
              right: 0,
              child: Stack(children: [
                Column(children: [
                  for (var i = 0; i < 3; i++) _band(i, CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: sampleLayers[i + 1].fam, start: sampleLayers[i + 1].start, end: sampleLayers[i + 1].end, keys: sampleLayers[i + 1].keys, ppf: ppf))),
                ]),
                Positioned(left: 0, top: 0, width: x0, height: 3 * TL.height, child: const IgnorePointer(child: ColoredBox(color: Color(0x80000000)))),
                Positioned(left: x1, top: 0, right: 0, height: 3 * TL.height, child: const IgnorePointer(child: ColoredBox(color: Color(0x80000000)))),
              ]),
            ),
            handle(x0, true),
            handle(x1, false),
            Positioned(
              left: 0,
              right: 0,
              top: top + 14 + 3 * TL.height + 8,
              child: Row(children: [
                Text('IN ', style: T.micro(N.g56)),
                Text(tpTime(v.$1, fps), style: T.value()),
                const SizedBox(width: 16),
                Text('OUT ', style: T.micro(N.g56)),
                Text(tpTime(v.$2, fps), style: T.value()),
                const SizedBox(width: 16),
                Text('DUR ', style: T.micro(N.g56)),
                Text(tpTime(v.$2 - v.$1, fps), style: T.value(N.g95)),
              ]),
            ),
          ]);
        },
      );
}

class _Grip extends StatelessWidget {
  const _Grip({required this.left});
  final bool left;
  @override
  Widget build(BuildContext context) => TpHit(
        cursor: SystemMouseCursors.resizeLeftRight,
        builder: (_, h, d) => AnimatedContainer(
          duration: tpFast,
          curve: Curves.easeOut,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: d ? N.g100 : h ? N.g91 : N.g76, borderRadius: BorderRadius.horizontal(left: Radius.circular(left ? 3 : 0), right: Radius.circular(left ? 0 : 3))),
          child: Container(width: 1, height: 6, color: N.g38),
        ),
      );
}

// ---------------------------------------------------------------- 8. ruler

class _RulerDemo extends StatelessWidget {
  const _RulerDemo({required this.ppf, required this.fps, required this.compare, required this.grid});
  final double ppf;
  final int fps;
  final bool compare, grid;
  @override
  Widget build(BuildContext context) {
    Widget one(String cap, double z, {bool g = false}) {
      final major = tpMajor(z, fps), fine = tpFine(z, fps, major);
      return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          Text(cap, style: T.micro(N.g63)),
          const SizedBox(width: 10),
          Text('labelled every ${major.round()} f   fine every ${fine.round()} f   ${z.toStringAsFixed(1)} px/f', style: T.value(N.g56).copyWith(fontSize: 10)),
        ]),
        const SizedBox(height: 4),
        SizedBox(height: TL.ruler, child: CustomPaint(size: Size.infinite, painter: _RulerPainter(z, fps))),
        if (g) SizedBox(height: 2 * TL.height, child: ColoredBox(color: N.bandA, child: CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: Fam.stagger, showBar: false, ppf: z, fps: fps)))),
        const SizedBox(height: 12),
      ]);
    }

    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      if (compare) one('1/4x', ppf / 4),
      one('NOW', ppf, g: grid),
      if (compare) one('4x', ppf * 4),
    ]);
  }
}

// ---------------------------------------------------------------- 9. curve

const _presets = <String, List<double>>{
  'linear': [0, 0, 1, 1],
  'ease': [.25, .1, .25, 1],
  'ease in': [.42, 0, 1, 1],
  'ease out': [0, 0, .58, 1],
  'ease in-out': [.42, 0, .58, 1],
  'overshoot': [.34, 1.56, .64, 1],
  'anticipate': [.36, -.56, .64, 1],
};

class _CurveEditor extends StatefulWidget {
  const _CurveEditor({super.key, required this.preset, required this.t});
  final String preset;
  final double t;
  @override
  State<_CurveEditor> createState() => _CurveEditorState();
}

class _CurveEditorState extends State<_CurveEditor> {
  late Offset p1 = Offset(_presets[widget.preset]![0], _presets[widget.preset]![1]), p2 = Offset(_presets[widget.preset]![2], _presets[widget.preset]![3]);
  int? drag, hover;
  static const box = 220.0, pad = 20.0;
  static const ymin = -.5, ymax = 1.5;
  Rect get g => const Rect.fromLTWH(pad, pad, box, box);
  Offset px(Offset v) => Offset(g.left + v.dx * g.width, g.bottom - (v.dy - ymin) / (ymax - ymin) * g.height);
  Offset val(Offset p) => Offset(((p.dx - g.left) / g.width).clamp(0.0, 1.0), ymin + (g.bottom - p.dy) / g.height * (ymax - ymin));
  int? near(Offset p) {
    final d1 = (px(p1) - p).distance, d2 = (px(p2) - p).distance;
    final m = math.min(d1, d2);
    return m > 16 ? null : (d1 <= d2 ? 0 : 1);
  }

  void move(Offset p) {
    final v = val(p);
    setState(() => drag == 0 ? p1 = v : p2 = v);
  }

  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        MouseRegion(
          cursor: hover != null || drag != null ? SystemMouseCursors.grab : SystemMouseCursors.basic,
          onHover: (e) => setState(() => hover = near(e.localPosition)),
          onExit: (_) => setState(() => hover = null),
          child: GestureDetector(
            onPanDown: (d) {
              drag = near(d.localPosition);
              if (drag != null) move(d.localPosition);
            },
            onPanUpdate: (d) {
              if (drag != null) move(d.localPosition);
            },
            onPanEnd: (_) => setState(() => drag = null),
            child: Container(
              width: box + 2 * pad,
              height: box + 2 * pad,
              decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(4)),
              child: CustomPaint(painter: _CurvePainter(p1, p2, widget.t, drag ?? hover, drag != null, px)),
            ),
          ),
        ),
        const SizedBox(height: 10),
        Text('cubic-bezier(${p1.dx.toStringAsFixed(2)}, ${p1.dy.toStringAsFixed(2)}, ${p2.dx.toStringAsFixed(2)}, ${p2.dy.toStringAsFixed(2)})', style: T.value(N.g76).copyWith(fontSize: 10)),
      ]);
}

double _bezierY(Offset p1, Offset p2, double x) {
  double bx(double u) => 3 * (1 - u) * (1 - u) * u * p1.dx + 3 * (1 - u) * u * u * p2.dx + u * u * u;
  var lo = 0.0, hi = 1.0;
  for (var i = 0; i < 24; i++) {
    final m = (lo + hi) / 2;
    if (bx(m) < x) {
      lo = m;
    } else {
      hi = m;
    }
  }
  final u = (lo + hi) / 2;
  return 3 * (1 - u) * (1 - u) * u * p1.dy + 3 * (1 - u) * u * u * p2.dy + u * u * u;
}

class _CurvePainter extends CustomPainter {
  const _CurvePainter(this.p1, this.p2, this.t, this.active, this.dragging, this.px);
  final Offset p1, p2;
  final double t;
  final int? active;
  final bool dragging;
  final Offset Function(Offset) px;
  @override
  void paint(Canvas c, Size s) {
    final fill = Paint();
    final a = px(const Offset(0, 0)), b = px(const Offset(1, 1));
    for (var i = 0; i <= 4; i++) {
      final x = (a.dx + (b.dx - a.dx) * i / 4).roundToDouble(), y = (a.dy + (b.dy - a.dy) * i / 4).roundToDouble();
      final edge = i == 0 || i == 4;
      fill.color = edge ? N.glaze15 : N.glaze9;
      c.drawRect(Rect.fromLTWH(x, math.min(a.dy, b.dy) - 20, 1, (a.dy - b.dy).abs() + 40), fill);
      c.drawRect(Rect.fromLTWH(a.dx, y, b.dx - a.dx, 1), fill);
    }
    final stroke = Paint()..style = PaintingStyle.stroke..strokeWidth = 1;
    // handle lines, then the curve over them
    c.drawLine(a, px(p1), stroke..color = N.g56);
    c.drawLine(b, px(p2), stroke..color = N.g56);
    final path = Path()..moveTo(a.dx, a.dy);
    for (var i = 1; i <= 64; i++) {
      final u = i / 64.0;
      final x = 3 * (1 - u) * (1 - u) * u * p1.dx + 3 * (1 - u) * u * u * p2.dx + u * u * u;
      final y = 3 * (1 - u) * (1 - u) * u * p1.dy + 3 * (1 - u) * u * u * p2.dy + u * u * u;
      final o = px(Offset(x, y));
      path.lineTo(o.dx, o.dy);
    }
    c.drawPath(path, stroke..strokeWidth = 1.75..strokeCap = StrokeCap.round..color = tpAccent);
    // the time cursor and the value under it
    final tp = px(Offset(t, _bezierY(p1, p2, t)));
    c.drawRect(Rect.fromLTWH(tp.dx.roundToDouble(), a.dy - 0, 1, 0), fill);
    c.drawLine(Offset(tp.dx, a.dy), Offset(tp.dx, tp.dy), stroke..strokeWidth = 1..color = C.playhead.withValues(alpha: .6));
    c.drawCircle(tp, 2.5, fill..color = C.playhead);
    tpKey(c, a, TL.key / 2 * 1.1, TpKey.normal);
    tpKey(c, b, TL.key / 2 * 1.1, TpKey.normal);
    for (var i = 0; i < 2; i++) {
      final o = px(i == 0 ? p1 : p2), on = active == i;
      c.drawCircle(o, on ? 3.5 : 3, fill..color = on ? tpAccent : N.g95);
      if (dragging && on) c.drawCircle(o, 3.5, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = N.g100);
    }
  }

  @override
  bool shouldRepaint(_CurvePainter o) => true;
}

// ---------------------------------------------------------------- 10. property lanes

class _PropLanes extends StatefulWidget {
  const _PropLanes({super.key, required this.head, required this.ppf, required this.kind});
  final int head;
  final double ppf;
  final TpInterp kind;
  @override
  State<_PropLanes> createState() => _PropLanesState();
}

class _PropLanesState extends State<_PropLanes> {
  final names = const ['Position X', 'Position Y'];
  final vals = const [[120.0, 480.0, 300.0, 640.0], [80.0, 540.0, 200.0]];
  late final keys = [[8.0, 34.0, 66.0, 104.0], [8.0, 50.0, 82.0]];
  final anim = [true, true];
  (int, int)? sel, hov;

  double valueAt(int l, double f) {
    final k = keys[l], v = vals[l];
    if (!anim[l] || f <= k.first) return v.first;
    if (f >= k.last) return v.last;
    var i = 0;
    while (k[i + 1] < f) {
      i++;
    }
    var u = (f - k[i]) / (k[i + 1] - k[i]);
    if (widget.kind == TpInterp.hold) u = 0;
    if (widget.kind == TpInterp.eased || widget.kind == TpInterp.bezier) u = u * u * (3 - 2 * u);
    return v[i] + (v[i + 1] - v[i]) * u;
  }

  (int, int)? hit(int l, Offset p) {
    (int, int)? best;
    var bd = 9.0;
    for (var i = 0; i < keys[l].length; i++) {
      final d = (keys[l][i] * widget.ppf - p.dx).abs();
      if (d < bd) {
        bd = d;
        best = (l, i);
      }
    }
    return anim[l] ? best : null;
  }

  @override
  Widget build(BuildContext context) => Column(children: [
        Row(children: [
          const SizedBox(width: TL.label),
          Expanded(child: SizedBox(height: TL.ruler, child: CustomPaint(painter: _RulerPainter(widget.ppf, 30)))),
        ]),
        for (var l = 0; l < 2; l++)
          _band(
            l,
            Row(children: [
              SizedBox(
                width: TL.label,
                child: Padding(
                  padding: const EdgeInsets.only(left: 22, right: 8),
                  child: Row(children: [
                    TpIconButton(TpGlyph.stopwatch, on: anim[l], onTap: () => setState(() => anim[l] = !anim[l])),
                    const SizedBox(width: 6),
                    Expanded(child: Text(names[l], style: T.name(N.g76), maxLines: 1)),
                    Text(valueAt(l, widget.head.toDouble()).toStringAsFixed(1), style: T.value(N.g95)),
                  ]),
                ),
              ),
              Expanded(
                child: LayoutBuilder(builder: (context, box) {
                  void moveSel(Offset p) {
                    if (sel == null || sel!.$1 != l) return;
                    final i = sel!.$2, k = keys[l];
                    final lo = i == 0 ? 0.0 : k[i - 1] + 1, hi = i == k.length - 1 ? 118.0 : k[i + 1] - 1;
                    setState(() => k[i] = (p.dx / widget.ppf).roundToDouble().clamp(lo, hi));
                  }

                  return MouseRegion(
                    cursor: hov?.$1 == l ? SystemMouseCursors.grab : SystemMouseCursors.basic,
                    onHover: (e) => setState(() => hov = hit(l, e.localPosition)),
                    onExit: (_) => setState(() => hov = null),
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onPanDown: (d) => setState(() => sel = hit(l, d.localPosition) ?? sel),
                      onPanUpdate: (d) => moveSel(d.localPosition),
                      child: CustomPaint(
                        size: Size.infinite,
                        painter: _PropLanePainter(keys[l], widget.kind, anim[l], sel?.$1 == l ? sel?.$2 : null, hov?.$1 == l ? hov?.$2 : null, widget.head, widget.ppf),
                      ),
                    ),
                  );
                }),
              ),
            ]),
          ),
      ]);
}

class _PropLanePainter extends CustomPainter {
  const _PropLanePainter(this.keys, this.kind, this.on, this.sel, this.hov, this.head, this.ppf);
  final List<double> keys;
  final TpInterp kind;
  final bool on;
  final int? sel, hov;
  final int head;
  final double ppf;
  @override
  void paint(Canvas c, Size s) {
    tpGrid(c, s, ppf, 30);
    final cy = s.height / 2;
    if (on) {
      c.drawRect(Rect.fromLTRB(keys.first * ppf, cy - .5, keys.last * ppf, cy + .5), Paint()..color = N.g100.withValues(alpha: .7));
      for (var i = 0; i < keys.length - 1; i++) {
        if ((keys[i + 1] - keys[i]) * ppf > 24) tpSeg(c, Offset((keys[i] + keys[i + 1]) / 2 * ppf, cy), kind);
      }
      for (var i = 0; i < keys.length; i++) {
        final k = keys[i].round() == head ? TpKey.atHead : i == sel ? TpKey.selected : TpKey.normal;
        tpKey(c, Offset(keys[i] * ppf, cy), TL.key / 2 * (i == hov ? 1.2 : 1), k);
      }
    }
    c.drawRect(Rect.fromLTWH(head * ppf, 0, 1, s.height), Paint()..color = C.playhead);
  }

  @override
  bool shouldRepaint(_PropLanePainter o) => true;
}

// ---------------------------------------------------------------- 11. zoom

class _ZoomSlider extends StatelessWidget {
  const _ZoomSlider({super.key, required this.fit, required this.start});
  final double fit, start;
  static double toPpf(double t) => .5 * math.pow(80, t);
  static double fromPpf(double p) => (math.log(p / .5) / math.log(80)).clamp(0.0, 1.0);
  @override
  Widget build(BuildContext context) => TpValue<double>(
        initial: fromPpf(start),
        builder: (context, t, set) {
          final fitT = fromPpf(fit);
          final atFit = (t - fitT).abs() < .01;
          return Row(children: [
            TpIconButton(TpGlyph.minus, size: 22, onTap: () => set((t - .08).clamp(0.0, 1.0))),
            const SizedBox(width: 2),
            Expanded(
              child: LayoutBuilder(builder: (context, box) {
                void drag(Offset p) => set(((p.dx - 6) / (box.maxWidth - 12)).clamp(0.0, 1.0));
                return MouseRegion(
                  cursor: SystemMouseCursors.resizeLeftRight,
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onPanDown: (d) => drag(d.localPosition),
                    onPanUpdate: (d) => drag(d.localPosition),
                    child: SizedBox(height: 22, child: CustomPaint(painter: _ZoomTrack(t, fitT))),
                  ),
                );
              }),
            ),
            const SizedBox(width: 2),
            TpIconButton(TpGlyph.plus, size: 22, onTap: () => set((t + .08).clamp(0.0, 1.0))),
            const SizedBox(width: 8),
            SizedBox(width: 58, child: Text('${toPpf(t).toStringAsFixed(1)} px/f', style: T.value(N.g76).copyWith(fontSize: 10), maxLines: 1)),
            const SizedBox(width: 6),
            TpHit(
              onTap: () => set(fitT),
              builder: (_, h, d) => AnimatedContainer(
                duration: tpFast,
                curve: Curves.easeOut,
                height: 22,
                padding: const EdgeInsets.symmetric(horizontal: 6),
                decoration: BoxDecoration(color: d ? N.g20 : h ? N.g15 : const Color(0x00262626), borderRadius: BorderRadius.circular(4)),
                child: Row(mainAxisSize: MainAxisSize.min, children: [
                  TpGlyphView(TpGlyph.fit, atFit ? tpAccent : h ? N.g95 : N.g63, size: 11),
                  const SizedBox(width: 4),
                  Text('FIT', style: T.micro(atFit ? N.g95 : N.g76)),
                ]),
              ),
            ),
          ]);
        },
      );
}

class _ZoomTrack extends CustomPainter {
  const _ZoomTrack(this.t, this.fitT);
  final double t, fitT;
  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2, w = s.width - 12, x = 6 + w * t, p = Paint();
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(0, cy - 1, s.width, cy + 1), const Radius.circular(1)), p..color = N.g20);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(0, cy - 1, x, cy + 1), const Radius.circular(1)), p..color = N.g56);
    c.drawCircle(Offset(x, cy), 6, p..color = N.g95);
  }

  @override
  bool shouldRepaint(_ZoomTrack o) => o.t != t || o.fitT != fitT;
}

// ---------------------------------------------------------------- 12. transport

class _Transport extends StatefulWidget {
  const _Transport({super.key, required this.playing, required this.loop, required this.fps, required this.total});
  final bool playing, loop;
  final int fps, total;
  @override
  State<_Transport> createState() => _TransportState();
}

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
              const SizedBox(width: TL.label),
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
          Expanded(child: Row(children: [const SizedBox(width: TL.label), Expanded(child: CustomPaint(size: Size.infinite, painter: TpLanePainter(fam: Fam.face, showBar: false, ppf: ppf)))])),
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
