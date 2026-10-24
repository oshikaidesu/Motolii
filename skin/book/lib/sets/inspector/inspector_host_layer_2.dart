// Part of inspector_host_layer.dart (cut there by size, as it was).
part of 'inspector_host_layer.dart';

/// contract: the layer's time is read through its ancestors' order shifts (schedule_delay), then its own Loop fold (clock.rs);
/// Ghost is one delayed look of this layer, never copies — copies are the Repeater's.
class TimeCard extends StatelessWidget {
  const TimeCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      const {'time_speed': 100, 'time_loop.dur': 1.5, 'time_stagger': .5, 'time_ghost.f': 4},
      strs: const {'time_loop.dir': 'Alternate', 'time_from': 'Start'},
      flags: const {'time_loop': true, 'time_fromEnd': false, 'time_ghost': false},
    );
    final speed = doc.get('time_speed') ?? 100, loop = doc.b['time_loop'] ?? false, dur = doc.get('time_loop.dur') ?? 0;
    final dir = math.max(0, _ldLoopDirs.indexOf(doc.s2['time_loop.dir'] ?? 'Normal'));
    final stagger = doc.get('time_stagger') ?? 0, fromEnd = doc.b['time_fromEnd'] ?? false;
    final w = _ldOrderWeight(math.max(0, _ldFrom.indexOf(doc.s2['time_from'] ?? 'Start')), _ldOrderI, _ldOrderN);
    final delay = stagger <= 1e-9 ? 0.0 : (fromEnd ? -1 : 1) * w * stagger;
    final ghost = doc.b['time_ghost'] ?? false, gf = doc.get('time_ghost.f') ?? 0;
    final dirWord = const ['', ' reversed', ' alternate', ' alt. reversed'][dir];
    return Sect(
      title: 'Time',
      tone: Pop.toneTime,
      mark: CardMark.clock,
      keyIds: const ['time_speed', 'time_loop.dur', 'time_stagger'],
      brief: [
        'Speed ${brief(speed, 0)}%',
        if (loop && dur > 0) 'Loop ${brief(dur, 2)} s$dirWord',
        if (delay != 0) '${delay > 0 ? '+' : ''}${brief(delay, 2)} s',
        if (ghost) 'Ghost ${brief(gf, 0)} f',
      ],
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: SizedBox(height: 30, child: CustomPaint(painter: _LdClockPaint(speed / 100, loop ? dur : 0, dir, delay, ghost ? gf / _ldFps : null))),
        ),
        LdGrid([
          LdCell(ParamTile('time', const Param(id: 'speed', label: 'Speed', look: ParamLook.bar, type: WType.f32, min: 0, max: 400, def: 100, unit: '%'))),
          const LdCell(_LdToggle('Loop', 'time_loop')),
          if (loop) ...[
            LdCell(ldNum(doc, 'time_loop.dur', 'Length', unit: 's', dec: 2, min: 0, max: 3600, perPx: .02, step: .1, zero: 'Off')),
            const LdCell(_LdSeg('Loop direction', 'time_loop.dir', _ldLoopDirs, words: ['→', '←', '↔', '↔ rev']), span: 2),
          ],
          const LdCell(_LdToggle('Ghost', 'time_ghost')),
          if (ghost) LdCell(ldNum(doc, 'time_ghost.f', 'Ghost delay', unit: 'f', min: -120, max: 120, perPx: .25, zero: 'None', bipolar: true)),
        ]),
        if (ghost) const Cap('One delayed look of this layer. For copies use a Repeater.'),
        _LdAdvanced(
          id: 'time',
          count: 3,
          children: [
            _LdSub(
              'Order',
              note: delay == 0
                  ? 'Set on Jewel Field · 3rd of $_ldOrderN'
                  : 'Set on Jewel Field · 3rd of $_ldOrderN starts ${brief(delay.abs(), 2)} s ${delay > 0 ? 'later' : 'earlier'}',
            ),
            LdGrid([
              LdCell(ldNum(doc, 'time_stagger', 'Stagger', unit: 's', dec: 2, min: 0, max: 60, perPx: .01, step: .05, zero: 'Off')),
              const LdCell(_LdSeg('From', 'time_from', _ldFrom), span: 2),
              const LdCell(_LdToggle('From end', 'time_fromEnd')),
            ]),
          ],
        ),
      ],
    );
  }
}

/// The layer's own time against the composition's: speed is the slope, Loop folds it, Stagger shifts it, Ghost trails it.
class _LdClockPaint extends CustomPainter {
  const _LdClockPaint(this.speed, this.loop, this.dir, this.delay, this.ghost);
  final double speed, loop, delay;
  final int dir;
  final double? ghost;
  static const span = 4.0;

  double _local(double t) {
    final s = math.max(0.0, t - delay) * speed;
    if (loop <= 1e-9) return s;
    final round = (s / loop).floor(), along = s - round * loop, odd = round.isOdd;
    return switch (dir) {
      1 => loop - along,
      2 => odd ? loop - along : along,
      3 => odd ? along : loop - along,
      _ => along,
    };
  }

  @override
  void paint(Canvas c, Size s) {
    final box = Offset.zero & s;
    c.drawRRect(RRect.fromRectAndRadius(box, const Radius.circular(6)), Paint()..color = Pop.well);
    final r = box.deflate(5), tick = Paint()..color = Grey.g20;
    for (var i = 1; i < span; i++) {
      final x = r.left + r.width * i / span;
      c.drawLine(Offset(x, r.top), Offset(x, r.bottom), tick);
    }
    final top = loop > 1e-9 ? loop : math.max(span * speed, 1e-3);
    Path curve(double shift) {
      final p = Path();
      for (var k = 0; k <= 160; k++) {
        final t = span * k / 160, y = (_local(t - shift) / top).clamp(0.0, 1.0);
        final o = Offset(r.left + r.width * k / 160, r.bottom - r.height * y);
        k == 0 ? p.moveTo(o.dx, o.dy) : p.lineTo(o.dx, o.dy);
      }
      return p;
    }

    Paint line(Color col, double w) => Paint()
      ..color = col
      ..style = PaintingStyle.stroke
      ..strokeWidth = w
      ..strokeJoin = StrokeJoin.round;
    if (ghost != null) c.drawPath(curve(ghost!), line(Pop.toneTime.withValues(alpha: .35), 1.2));
    if (delay != 0) {
      final x = r.left + r.width * (delay.abs() / span).clamp(0.0, 1.0);
      c.drawLine(Offset(x, r.top), Offset(x, r.bottom), line(Grey.g56, 1));
    }
    c.drawPath(curve(0), line(Pop.toneTime, 1.6));
  }

  @override
  bool shouldRepaint(_LdClockPaint o) => o.speed != speed || o.loop != loop || o.dir != dir || o.delay != delay || o.ghost != ghost;
}

// ---- Layout ---------------------------------------------------------------------------------------------------------
const _ldDirections = ['Row', 'Column', 'Row Reverse', 'Column Reverse', 'Depth'];
const _ldJustify = ['Start', 'End', 'Center', 'Space Between', 'Space Around', 'Space Evenly'];
const _ldAlign = ['Stretch', 'Start', 'End', 'Center'];
const _ldSizing = ['Hug', 'Fill', 'Fixed'];
const _ldSelf = ['Auto', 'Stretch', 'Start', 'End', 'Center'];

/// contract: the parent's lines (Display, grid, gap, padding, alignment, sizing) as CSS flex / grid; the layer's own lines as a child sit behind Advanced.
class HostLayoutCard extends StatelessWidget {
  const HostLayoutCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(
      const {
        'hl_cols': 3,
        'hl_rows': 2,
        'hl_gap': 8,
        'hl_padx': 16,
        'hl_pady': 16,
        'hl_w': 480,
        'hl_h': 270,
        'hl_colStart': 0,
        'hl_rowStart': 0,
        'hl_colSpan': 1,
        'hl_rowSpan': 1,
        'hl_margin': 0,
        'hl_shrink': 1,
        'hl_radius': 0,
      },
      strs: const {
        'hl_display': 'Grid',
        'hl_dir': 'Row',
        'hl_wrap': 'No Wrap',
        'hl_justify': 'Start',
        'hl_align': 'Stretch',
        'hl_wsize': 'Hug',
        'hl_hsize': 'Hug',
        'hl_self': 'Auto',
        'hl_overflow': 'Visible',
      },
      flags: const {'hl_abs': false},
    );
    final display = doc.s2['hl_display'] ?? 'Grid', grid = display == 'Grid', flex = display == 'Flex';
    final cols = doc.get('hl_cols') ?? 0, rows = doc.get('hl_rows') ?? 0;
    final wFixed = doc.s2['hl_wsize'] == 'Fixed', hFixed = doc.s2['hl_hsize'] == 'Fixed';
    const shows = ['None', 'Flex', 'Grid'];
    return Sect(
      title: 'Layout',
      tone: Pop.toneLayout,
      mark: CardMark.layout,
      keyIds: const ['hl_gap', 'hl_padx', 'hl_pady', 'hl_w', 'hl_h'],
      brief: [
        if (grid) 'Grid ${brief(cols, 0)}×${rows == 0 ? 'auto' : brief(rows, 0)}' else if (flex) 'Flex ${doc.s2['hl_dir']}' else 'No layout',
        if (grid || flex) ...['Gap ${brief(doc.get('hl_gap'), 0)}', '${doc.s2['hl_justify']} · ${doc.s2['hl_align']}', doc.s2['hl_wsize'] ?? 'Hug'],
      ],
      children: [
        SizedBox(
          height: Pop.cell,
          child: Dis(
            child: Segmented(items: shows, index: math.max(0, shows.indexOf(display)), expand: true, onChanged: (i) => doc.str('hl_display', shows[i])),
          ),
        ),
        if (!grid && !flex)
          const Cap('Children keep their own positions.')
        else
          LdGrid([
            if (grid) ...[
              LdCell(
                ldNum(
                  doc,
                  'hl_cols',
                  'Columns',
                  min: 1,
                  max: 64,
                  perPx: .1,
                  glyph: Steps(id: 'hl_cols', p: _ldCount(1)),
                  owns: true,
                  tall: true,
                ),
              ),
              LdCell(
                ldNum(
                  doc,
                  'hl_rows',
                  'Rows',
                  min: 0,
                  max: 64,
                  perPx: .1,
                  zero: 'Auto',
                  glyph: Steps(id: 'hl_rows', p: _ldCount(0)),
                  owns: true,
                  tall: true,
                ),
              ),
            ],
            if (flex) ...[
              const LdCell(_LdPick('Direction', 'hl_dir', _ldDirections)),
              const LdCell(_LdSeg('Wrap', 'hl_wrap', ['No Wrap', 'Wrap', 'Wrap Reverse'], words: ['No', 'Wrap', 'Rev']), span: 2),
            ],
            LdCell(ldNum(doc, 'hl_gap', 'Gap', unit: 'px', min: 0, max: 1000, glyph: _LdMini(_LdMiniKind.gap, doc.get('hl_gap') ?? 0))),
            LdCell(ldNum(doc, 'hl_padx', 'Padding X', unit: 'px', min: 0, max: 1000, glyph: _LdMini(_LdMiniKind.padX, doc.get('hl_padx') ?? 0))),
            LdCell(ldNum(doc, 'hl_pady', 'Padding Y', unit: 'px', min: 0, max: 1000, glyph: _LdMini(_LdMiniKind.padY, doc.get('hl_pady') ?? 0))),
            const LdCell(_LdAlignPad()),
            const LdCell(_LdPick('Justify', 'hl_justify', _ldJustify)),
            const LdCell(_LdPick('Items', 'hl_align', _ldAlign)),
            const LdCell(_LdSeg('Width', 'hl_wsize', _ldSizing), span: 2),
            LdCell(ldNum(doc, 'hl_w', 'W', unit: 'px', min: 0, max: 100000, enabled: wFixed)),
            const LdCell(_LdSeg('Height', 'hl_hsize', _ldSizing), span: 2),
            LdCell(ldNum(doc, 'hl_h', 'H', unit: 'px', min: 0, max: 100000, enabled: hFixed)),
          ]),
        _LdAdvanced(
          id: 'hl',
          count: 10,
          children: [
            const _LdSub('As a child', note: 'Its own lines in Jewel Field'),
            LdGrid([
              const LdCell(_LdToggle('Ignore layout', 'hl_abs')),
              const LdCell(_LdPick('Align self', 'hl_self', _ldSelf)),
              LdCell(ldNum(doc, 'hl_margin', 'Margin', unit: 'px', min: 0, max: 100000)),
              LdCell(ldNum(doc, 'hl_colStart', 'Column start', min: 0, max: 64, perPx: .1, zero: 'Auto')),
              LdCell(ldNum(doc, 'hl_rowStart', 'Row start', min: 0, max: 64, perPx: .1, zero: 'Auto')),
              LdCell(ldNum(doc, 'hl_colSpan', 'Column span', min: 1, max: 64, perPx: .1)),
              LdCell(ldNum(doc, 'hl_rowSpan', 'Row span', min: 1, max: 64, perPx: .1)),
              LdCell(ldNum(doc, 'hl_shrink', 'Shrink', dec: 1, min: 0, max: 1000, perPx: .05, step: .1)),
            ]),
            const _LdSub('Box'),
            LdGrid([
              const LdCell(_LdSeg('Overflow', 'hl_overflow', ['Visible', 'Clip', 'Bounce']), span: 2),
              LdCell(ldNum(doc, 'hl_radius', 'Corner radius', unit: 'px', min: 0, max: 100000)),
            ]),
          ],
        ),
      ],
    );
  }
}

Param _ldCount(double min) => Param(id: '', label: '', look: ParamLook.stepper, type: WType.i32, min: min, max: 64, def: min);

enum _LdMiniKind { gap, padX, padY }

/// A small picture of a spacing value: two children apart for Gap, a box inside a box for Padding.
class _LdMini extends StatelessWidget {
  const _LdMini(this.kind, this.v);
  final _LdMiniKind kind;
  final double v;
  @override
  Widget build(BuildContext context) => CustomPaint(size: Size(Pop.glyph, Pop.glyph), painter: _LdMiniPaint(kind, v));
}

class _LdMiniPaint extends CustomPainter {
  const _LdMiniPaint(this.kind, this.v);
  final _LdMiniKind kind;
  final double v;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero), lit = Paint()..color = Grey.g76;
    final t = (v / 48).clamp(0.0, 1.0);
    final line = Paint()
      ..color = Grey.g44
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    switch (kind) {
      case _LdMiniKind.gap:
        final g = 1 + 7 * t;
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(o.dx - g / 2 - 6, o.dy - 6, o.dx - g / 2, o.dy + 6), const Radius.circular(1.5)), lit);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(o.dx + g / 2, o.dy - 6, o.dx + g / 2 + 6, o.dy + 6), const Radius.circular(1.5)), lit);
      case _LdMiniKind.padX || _LdMiniKind.padY:
        final outer = Rect.fromCenter(center: o, width: 18, height: 16), p = 1 + 4 * t;
        c.drawRRect(RRect.fromRectAndRadius(outer, const Radius.circular(2)), line);
        final inner = kind == _LdMiniKind.padX
            ? Rect.fromLTRB(outer.left + p, outer.top + 3, outer.right - p, outer.bottom - 3)
            : Rect.fromLTRB(outer.left + 3, outer.top + p, outer.right - 3, outer.bottom - p);
        c.drawRRect(RRect.fromRectAndRadius(inner, const Radius.circular(1.5)), lit);
    }
  }

  @override
  bool shouldRepaint(_LdMiniPaint o) => o.kind != kind || o.v != v;
}

/// Justify (across) and Align Items (down) as one 3×3 pick, like the anchor's pad.
/// contract: a press or drag lands on the nearest of nine; it writes Start / Center / End on both axes, the choosers beside it keep the rest (Space *, Stretch).
class _LdAlignPad extends StatelessWidget {
  const _LdAlignPad();
  static const _across = {'Start': 0, 'End': 2, 'Center': 1, 'Space Between': 0, 'Space Around': 1, 'Space Evenly': 2};
  static const _down = {'Stretch': 1, 'Start': 0, 'End': 2, 'Center': 1};
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc;
    final j = doc.s2['hl_justify'] ?? 'Start', a = doc.s2['hl_align'] ?? 'Stretch';
    final on = !x.cfg.locked && !RowOff.offOf(context);
    void pick(Offset p, Size s) {
      final cx = (p.dx / s.width * 3).floor().clamp(0, 2), cy = (p.dy / s.height * 3).floor().clamp(0, 2);
      final nj = const ['Start', 'Center', 'End'][cx], na = const ['Start', 'Center', 'End'][cy];
      if (nj != doc.s2['hl_justify'] || na != doc.s2['hl_align']) {
        doc.s2['hl_justify'] = nj;
        doc.str('hl_align', na);
      }
    }

    return LdTile(
      label: 'Align',
      glyphW: 38,
      glyph: LayoutBuilder(
        builder: (context, box) {
          final s = Size(box.maxWidth, box.maxHeight);
          return MouseRegion(
            cursor: on ? SystemMouseCursors.click : SystemMouseCursors.basic,
            child: Listener(
              onPointerDown: on ? (e) => pick(e.localPosition, s) : null,
              onPointerMove: on ? (e) => pick(e.localPosition, s) : null,
              child: CustomPaint(size: s, painter: _LdAlignPaint(_across[j] ?? 0, _down[a] ?? 1, a == 'Stretch', j.startsWith('Space'), on)),
            ),
          );
        },
      ),
      child: Padding(
        padding: const EdgeInsets.only(bottom: 3),
        child: Text('${j.replaceFirst('Space ', '')} · $a', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(on ? Grey.g91 : Grey.g56)),
      ),
    );
  }
}

class _LdAlignPaint extends CustomPainter {
  const _LdAlignPaint(this.cx, this.cy, this.stretch, this.spread, this.on);
  final int cx, cy;
  final bool stretch, spread, on;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromRectAndRadius(Offset.zero & s, const Radius.circular(5)), Paint()..color = Grey.g20);
    final dim = Paint()..color = Grey.g38, lit = Paint()..color = on ? Pop.toneLayout : Grey.g56;
    Offset at(int i, int k) => Offset(s.width * (i + .5) / 3, s.height * (k + .5) / 3);
    for (var k = 0; k < 3; k++) {
      for (var i = 0; i < 3; i++) {
        c.drawCircle(at(i, k), 1.4, dim);
      }
    }
    final p = at(cx, cy), w = spread ? s.width * .8 : 8.0, h = stretch ? s.height * .8 : 6.0;
    c.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromCenter(center: Offset(spread ? s.width / 2 : p.dx, stretch ? s.height / 2 : p.dy), width: w, height: h),
        const Radius.circular(2),
      ),
      lit,
    );
  }

  @override
  bool shouldRepaint(_LdAlignPaint o) => o.cx != cx || o.cy != cy || o.stretch != stretch || o.spread != spread || o.on != on;
}

// ---- Links ----------------------------------------------------------------------------------------------------------
const _ldTargets = ['Opacity', 'Position X', 'Position Y', 'Rotation', 'Scale', 'Blur radius'];
const _ldLayers = ['Logo', 'Title', 'Backdrop', 'Camera 1', 'Null 1'];
const _ldSources = ['Opacity', 'Position X', 'Position Y', 'Rotation', 'Scale', 'Audio amplitude'];

/// contract: a link reads another layer's property (optionally earlier or later by Delay) and maps it, value × Multiply + Offset, onto this one (PropertyLink).
class LinksCard extends StatelessWidget {
  const LinksCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(const {}, strs: const {'link_list': '1,2', 'link_2.to': 'Rotation', 'link_2.layer': 'Null 1', 'link_2.prop': 'Rotation'});
    final ids = _ldList(doc, 'link_list');
    for (final i in ids) {
      doc.seed(
        {'link_$i.mul': i == '1' ? .5 : 1, 'link_$i.add': 0, 'link_$i.time': i == '2' ? 5 : 0},
        strs: {'link_$i.to': 'Opacity', 'link_$i.layer': 'Logo', 'link_$i.prop': 'Opacity'},
        flags: {'link_$i.open': i == '1'},
      );
    }
    return Sect(
      title: 'Links',
      tone: Pop.toneLink,
      mark: CardMark.link,
      keyIds: [
        for (final i in ids) ...['link_$i.mul', 'link_$i.add'],
      ],
      brief: [
        ids.isEmpty ? 'No links' : '${ids.length} link${ids.length == 1 ? '' : 's'}',
        for (final i in ids.take(2)) '${doc.s2['link_$i.to']} ← ${doc.s2['link_$i.layer']}',
      ],
      children: [
        if (ids.isEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 2),
            child: Text('No links. A link drives a property from another layer.', style: T.label(Grey.g56)),
          ),
        for (final i in ids) _LdLink(i),
        _LdAdd('Add link', () => _ldAddTo(doc, 'link_list')),
      ],
    );
  }
}

class _LdLink extends StatelessWidget {
  const _LdLink(this.i);
  final String i;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, k = 'link_$i', open = doc.b['$k.open'] ?? true;
    final mul = doc.get('$k.mul'), add = doc.get('$k.add') ?? 0, t = doc.get('$k.time') ?? 0;
    final map =
        '× ${brief(mul, 2)}${add != 0 ? ' ${add > 0 ? '+' : '−'} ${brief(add.abs(), 1)}' : ''}${t != 0 ? ' · ${t > 0 ? '+' : ''}${brief(t, 0)} f' : ''}';
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _LdItemHead(
          open: open,
          onToggle: () => doc.flag('$k.open', !open),
          onRemove: () => _ldRemove(doc, 'link_list', i),
          title: '${doc.s2['$k.to']} ← ${doc.s2['$k.layer']} · ${doc.s2['$k.prop']}',
          note: map,
        ),
        if (open)
          LdGrid([
            LdCell(_LdPick('Drives', '$k.to', _ldTargets)),
            LdCell(_LdPick('From layer', '$k.layer', _ldLayers)),
            LdCell(_LdPick('Its property', '$k.prop', _ldSources)),
            LdCell(ParamTile('link', Param(id: '$i.mul', label: 'Multiply', look: ParamLook.bipolar, type: WType.f32, min: -4, max: 4, dec: 2, def: 1))),
            LdCell(ParamTile('link', Param(id: '$i.add', label: 'Offset', look: ParamLook.scrub, type: WType.f32, dec: 1))),
            LdCell(ldNum(doc, '$k.time', 'Delay', unit: 'f', min: -120, max: 120, perPx: .25, zero: 'None', bipolar: true)),
          ]),
      ],
    );
  }
}
