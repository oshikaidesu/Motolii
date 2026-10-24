// set: timeline_parts. The pieces of a motion-graphics timeline, each alone and in context. Every size is a fraction of TL.* (DESIGN.md section 4).
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../parts/timeline.dart' show Layer, sampleLayers;
import '../../tokens.dart';
import 'timeline_parts_parts.dart';
part 'timeline_parts_2.dart';
part 'timeline_parts_3.dart';

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

Widget _band(int i, Widget child, {double? h}) => Container(height: h ?? TL.height, decoration: BoxDecoration(color: i.isEven ? N.bandA : N.bandB, border: const Border(bottom: BorderSide(color: N.rowLine))), child: child);

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
