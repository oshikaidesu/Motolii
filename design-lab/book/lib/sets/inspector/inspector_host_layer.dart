part of 'inspector_parts.dart';

// ---- Layer data: masks, the layer's clock, layout, property links ---------------------------------------------------
// Sources: motolii-doc store/mask.rs, attrs.rs (matte, ghost), layout/clock.rs + time.rs, layout.rs (GROUP_ROWS / ITEM_ROWS), slot.rs (PropertyLink).

/// One block in a [_LdGrid]: what it shows and how many columns it takes.
class _LdCell {
  const _LdCell(this.child, {this.span = 1});
  final Widget child;
  final int span;
}

/// The same flowing block grid as [ParamGrid], for blocks that are not shader parameters (choices, switches, pads).
class _LdGrid extends StatelessWidget {
  const _LdGrid(this.cells);
  final List<_LdCell> cells;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      const g = Pop.tileGap;
      final cols = math.max(1, ((box.maxWidth + g) / (Pop.tileMin + g)).floor());
      final rows = <List<_LdCell>>[[]];
      var used = 0;
      for (final c in cells) {
        final s = math.min(c.span, cols);
        if (used + s > cols) {
          rows.add([]);
          used = 0;
        }
        rows.last.add(c);
        used += s;
      }
      Widget row(List<_LdCell> cs, bool last) {
        final filled = cs.fold<int>(0, (a, c) => a + math.min(c.span, cols));
        return Row(
          children: [
            for (var i = 0; i < cs.length; i++) ...[if (i > 0) const SizedBox(width: g), Expanded(flex: math.min(cs[i].span, cols), child: cs[i].child)],
            if (last && filled < cols) ...[const SizedBox(width: g), Spacer(flex: cols - filled)],
          ],
        );
      }

      return Padding(
        padding: const EdgeInsets.only(top: 4),
        child: Column(
          children: [
            for (var i = 0; i < rows.length; i++) ...[if (i > 0) const SizedBox(height: g), row(rows[i], i == rows.length - 1)],
          ],
        ),
      );
    },
  );
}

/// A block's skeleton: name top-left, the control along the bottom, an optional picture top-right (as [ParamTile]).
class _LdTile extends StatelessWidget {
  const _LdTile({required this.label, required this.child, this.glyph, this.onTap, this.glyphW = Pop.glyph});
  final String label;
  final Widget child;
  final Widget? glyph;
  final VoidCallback? onTap;
  final double glyphW;
  @override
  Widget build(BuildContext context) {
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return Hov(
      onTap: on ? onTap : null,
      cursor: onTap != null && on ? SystemMouseCursors.click : SystemMouseCursors.basic,
      builder: (_, h) => Container(
        height: Pop.tile,
        decoration: BoxDecoration(color: h && onTap != null && on ? Grey.g20 : Pop.well, borderRadius: BorderRadius.circular(6)),
        child: Stack(
          children: [
            if (glyph != null)
              Positioned(
                right: Pop.tilePad - 3,
                top: 4,
                bottom: glyphW > Pop.glyph ? 4 : null,
                height: glyphW > Pop.glyph ? null : Pop.glyph,
                width: glyphW,
                child: glyph!,
              ),
            Padding(
              padding: EdgeInsets.fromLTRB(Pop.tilePad, 6, glyph != null ? glyphW + Pop.tilePad : Pop.tilePad, 4),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(on ? Grey.g63 : Grey.g44).copyWith(height: 1.1)),
                  child,
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// An On/Off block over doc.b.
class _LdToggle extends StatelessWidget {
  const _LdToggle(this.label, this.id);
  final String label, id;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, b = doc.b[id] ?? false;
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return _LdTile(
      label: label,
      onTap: () => doc.flag(id, !b),
      child: Padding(
        padding: const EdgeInsets.only(bottom: 3),
        child: Row(
          children: [
            IgnorePointer(
              child: Opacity(
                opacity: on ? 1 : .4,
                child: _PopSwitch(on: b, onChanged: (_) {}),
              ),
            ),
            const SizedBox(width: 6),
            Text(b ? 'On' : 'Off', style: T.label(on ? Grey.g76 : Grey.g56)),
          ],
        ),
      ),
    );
  }
}

/// A short choice (<=4) as a segmented block over doc.s2.
class _LdSeg extends StatelessWidget {
  const _LdSeg(this.label, this.id, this.items, {this.words});
  final String label, id;
  final List<String> items;

  /// Shorter words drawn in the segments; [items] are what doc.s2 stores.
  final List<String>? words;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    return _LdTile(
      label: label,
      child: SizedBox(
        height: 22,
        child: Dis(
          child: Segmented(
            items: words ?? items,
            index: math.max(0, items.indexOf(doc.s2[id] ?? items.first)),
            expand: true,
            onChanged: (i) => doc.str(id, items[i]),
          ),
        ),
      ),
    );
  }
}

/// A long choice as a block whose field opens the floating [Chooser].
class _LdPick extends StatelessWidget {
  const _LdPick(this.label, this.id, this.options, {this.word = 'Pick'});
  final String label, id, word;
  final List<String> options;
  @override
  Widget build(BuildContext context) => _LdTile(
    label: label,
    child: SizedBox(
      height: 22,
      child: Chooser(id: id, options: options, word: word),
    ),
  );
}

/// A number block (NumField caption form) carrying the row's key dot.
Widget _ldNum(
  Doc doc,
  String id,
  String label, {
  String unit = '',
  int dec = 0,
  double? min,
  double? max,
  double perPx = 1,
  double step = 1,
  String? zero,
  bool bipolar = false,
  bool enabled = true,
  Widget? glyph,
  bool owns = false,
  bool tall = false,
}) => _RowInfo(
  unit: null,
  keys: doc.keys[id] ?? KeyS.off,
  child: NumField(
    id: id,
    caption: label,
    unit: unit,
    decimals: dec,
    min: min,
    max: max,
    perPx: perPx,
    step: step,
    zeroWord: zero,
    bipolar: bipolar,
    enabled: enabled,
    glyph: glyph,
    glyphOwnsPointer: owns,
    glyphTall: tall,
  ),
);

/// A small caption that starts a group of blocks inside a card.
class _LdSub extends StatelessWidget {
  const _LdSub(this.text, {this.note});
  final String text;
  final String? note;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 8, bottom: 0),
    child: Row(
      children: [
        Text(text, style: T.micro(Grey.g76).copyWith(fontWeight: FontWeight.w700)),
        if (note != null) ...[
          const SizedBox(width: 6),
          Expanded(
            child: Text(note!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g56)),
          ),
        ],
      ],
    ),
  );
}

/// The Advanced fold as in [EffectCard]: a ruled line that says how much is behind it.
class _LdAdvanced extends StatelessWidget {
  const _LdAdvanced({required this.id, required this.count, required this.children});
  final String id;
  final int count;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), adv = doc.b['$id.adv'] ?? false;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 6),
          child: Hov(
            onTap: () => doc.flag('$id.adv', !adv),
            builder: (_, h) => Container(
              height: 24,
              decoration: BoxDecoration(
                border: Border(top: BorderSide(color: ed.rule)),
              ),
              child: Row(
                children: [
                  Text('Advanced', style: ed.labStyle(h || adv ? Grey.g95 : Grey.g63)),
                  const Spacer(),
                  Text(adv ? 'Fold' : '$count more', style: T.label(Grey.g56)),
                ],
              ),
            ),
          ),
        ),
        if (adv) ...children,
      ],
    );
  }
}

/// The add row at the foot of a list card.
class _LdAdd extends StatelessWidget {
  const _LdAdd(this.label, this.onTap);
  final String label;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) {
    final locked = Ctx.of(context).cfg.locked;
    return Padding(
      padding: const EdgeInsets.only(top: 6),
      child: Hov(
        onTap: locked ? null : onTap,
        builder: (_, h) => Container(
          height: Pop.cell,
          alignment: Alignment.center,
          decoration: BoxDecoration(color: h && !locked ? Grey.g20 : Pop.well, borderRadius: BorderRadius.circular(6)),
          child: Text('+  $label', style: T.label(locked ? Grey.g44 : (h ? Grey.g95 : Grey.g76))),
        ),
      ),
    );
  }
}

/// One item of a list card (a mask, a link): fold mark, picture, name, what it holds when folded, its own controls, remove.
class _LdItemHead extends StatelessWidget {
  const _LdItemHead({required this.open, required this.onToggle, required this.title, required this.onRemove, this.lead, this.note, this.trail});
  final bool open;
  final VoidCallback onToggle, onRemove;
  final String title;
  final String? note;
  final Widget? lead, trail;
  @override
  Widget build(BuildContext context) {
    final locked = Ctx.of(context).cfg.locked;
    return Padding(
      padding: const EdgeInsets.only(top: 6),
      child: SizedBox(
        height: Pop.cell,
        child: Row(
          children: [
            Expanded(
              child: Hov(
                onTap: onToggle,
                builder: (_, h) => Row(
                  children: [
                    FoldMark(open: open, hot: h),
                    const SizedBox(width: 6),
                    if (lead != null) ...[lead!, const SizedBox(width: 6)],
                    Flexible(
                      child: Text(
                        title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: T.name(h ? Grey.g100 : Grey.g91).copyWith(fontWeight: FontWeight.w600),
                      ),
                    ),
                    if (!open && note != null) ...[
                      const SizedBox(width: 8),
                      Flexible(
                        child: Text(note!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g56)),
                      ),
                    ],
                  ],
                ),
              ),
            ),
            if (trail != null) ...[const SizedBox(width: 6), trail!],
            Hov(
              onTap: locked ? null : onRemove,
              builder: (_, h) => Container(
                width: 20,
                height: Pop.cell,
                alignment: Alignment.center,
                child: Text('×', style: T.name(h && !locked ? Grey.g95 : Grey.g56).copyWith(fontSize: 13)),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// A list of items kept as comma-separated ids in doc.s2[id], so add and remove are plain string edits.
List<String> _ldList(Doc doc, String id) => (doc.s2[id] ?? '').split(',').where((e) => e.isNotEmpty).toList();

void _ldAddTo(Doc doc, String id) {
  final l = _ldList(doc, id);
  final next = l.map(int.parse).fold(0, math.max) + 1;
  doc.str(id, [...l, '$next'].join(','));
}

void _ldRemove(Doc doc, String id, String item) => doc.str(id, (_ldList(doc, id)..remove(item)).join(','));

// ---- Masks ----------------------------------------------------------------------------------------------------------
const _ldMaskModes = ['Add', 'Subtract', 'Intersect', 'Lighten', 'Darken', 'Difference'];
const _ldMatteLayers = ['None', 'Logo', 'Title', 'Backdrop'];
const _ldMatteModes = ['Alpha', 'Inverted Alpha', 'Luma', 'Inverted Luma'];

/// contract: masks combine top to bottom by their mode; Frame says whether a mask moves with the layer (Layer) or stays on the box that clips it (Box).
class MasksCard extends StatelessWidget {
  const MasksCard({super.key});
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc;
    doc.seed(const {}, strs: const {'mask_list': '1,2', 'mask_matte': 'None', 'mask_matte.mode': 'Alpha', 'mask_2.mode': 'Subtract'});
    final ids = _ldList(doc, 'mask_list');
    for (final i in ids) {
      doc.seed(
        {'mask_$i.feather': i == '1' ? 12 : 0, 'mask_$i.exp': 0, 'mask_$i.op': 100, 'mask_$i.inv': 0, 'mask_$i.frame': 0},
        strs: {'mask_$i.name': 'Mask $i', 'mask_$i.mode': 'Add'},
        flags: {'mask_$i.open': i == '1'},
      );
    }
    final matte = doc.s2['mask_matte'] ?? 'None';
    String mode(String i) => '${doc.s2['mask_$i.mode']}${(doc.get('mask_$i.inv') ?? 0) > 0 ? ' inv' : ''}';
    return Sect(
      title: 'Masks',
      tone: Pop.toneMask,
      mark: CardMark.mask,
      keyIds: [
        for (final i in ids) ...['mask_$i.feather', 'mask_$i.exp', 'mask_$i.op'],
      ],
      brief: [
        ids.isEmpty ? 'No masks' : '${ids.length} mask${ids.length == 1 ? '' : 's'}',
        for (final i in ids.take(3)) mode(i),
        if (matte != 'None') 'Matte $matte',
      ],
      children: [
        if (ids.isEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 2),
            child: Text('No masks. Draw one on the Stage or add one here.', style: T.label(Grey.g56)),
          ),
        for (final i in ids) _LdMask(i),
        _LdAdd('Add mask', () => _ldAddTo(doc, 'mask_list')),
        const _LdSub('Track matte', note: 'Another layer cuts this one'),
        _LdGrid([
          const _LdCell(_LdPick('Matte layer', 'mask_matte', _ldMatteLayers, word: 'Layers')),
          if (matte != 'None') const _LdCell(_LdSeg('Use', 'mask_matte.mode', _ldMatteModes, words: ['Alpha', 'Inv α', 'Luma', 'Inv L']), span: 2),
        ]),
      ],
    );
  }
}

class _LdMask extends StatelessWidget {
  const _LdMask(this.i);
  final String i;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, k = 'mask_$i';
    final open = doc.b['$k.open'] ?? true, inv = (doc.get('$k.inv') ?? 0) > 0;
    final m = _ldMaskModes.indexOf(doc.s2['$k.mode'] ?? 'Add');
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _LdItemHead(
          open: open,
          onToggle: () => doc.flag('$k.open', !open),
          onRemove: () => _ldRemove(doc, 'mask_list', i),
          lead: CustomPaint(size: const Size(14, 14), painter: _LdMaskGlyph(math.max(0, m), inv)),
          title: doc.s2['$k.name'] ?? 'Mask $i',
          note: 'Feather ${brief(doc.get('$k.feather'), 0)} · ${brief(doc.get('$k.op'), 0)}%${inv ? ' · Inverted' : ''}',
          trail: SizedBox(
            width: 128,
            child: Chooser(id: '$k.mode', options: _ldMaskModes, word: 'Mode'),
          ),
        ),
        if (open)
          _LdGrid([
            _LdCell(ParamTile('mask', Param(id: '$i.feather', label: 'Feather', look: ParamLook.bar, type: WType.f32, min: 0, max: 500, unit: 'px'))),
            _LdCell(ParamTile('mask', Param(id: '$i.exp', label: 'Expansion', look: ParamLook.bipolar, type: WType.f32, min: -200, max: 200, unit: 'px'))),
            _LdCell(ParamTile('mask', Param(id: '$i.op', label: 'Opacity', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%'))),
            _LdCell(
              ParamTile(
                'mask',
                Param(id: '$i.frame', label: 'Frame', look: ParamLook.choice, type: WType.u32, min: 0, max: 1, options: const ['Layer', 'Box']),
              ),
              span: 2,
            ),
            _LdCell(ParamTile('mask', Param(id: '$i.inv', label: 'Inverted', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1))),
          ]),
      ],
    );
  }
}

/// What the mask leaves visible: the layer as the square, the mask as the circle, the lit area is what remains.
class _LdMaskGlyph extends CustomPainter {
  const _LdMaskGlyph(this.mode, this.inv);
  final int mode;
  final bool inv;
  @override
  void paint(Canvas c, Size s) {
    final sq = Path()..addRRect(RRect.fromRectAndRadius(Rect.fromLTWH(1, 1, s.width - 2, s.height - 2), const Radius.circular(2)));
    final o = s.center(Offset.zero);
    Path circ(double dx) => Path()..addOval(Rect.fromCircle(center: o + Offset(dx, 0), radius: 4.2));
    var lit = switch (mode) {
      1 => Path.combine(PathOperation.difference, sq, circ(0)),
      2 => Path.combine(PathOperation.intersect, circ(-1.8), circ(1.8)),
      5 => Path.combine(PathOperation.xor, circ(-1.8), circ(1.8)),
      _ => circ(0),
    };
    if (inv) lit = Path.combine(PathOperation.difference, sq, lit);
    c.drawPath(
      sq,
      Paint()
        ..color = Grey.g44
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1,
    );
    c.drawPath(lit, Paint()..color = Pop.toneMask.withValues(alpha: mode == 4 ? .45 : 1));
    if (mode == 3) {
      c.drawPath(
        circ(0),
        Paint()
          ..color = Grey.g100
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(_LdMaskGlyph o) => o.mode != mode || o.inv != inv;
}

// ---- Time -----------------------------------------------------------------------------------------------------------
const _ldLoopDirs = ['Normal', 'Reverse', 'Alternate', 'Alternate Reverse'];
const _ldFrom = ['Start', 'Center', 'End', 'Edges'];

/// The mock layer's place among its siblings, for the order delay it receives.
const _ldOrderI = 2, _ldOrderN = 5, _ldFps = 30.0;

/// time.rs `order_weight`: how far along the order member i of n sits, seen from [from].
double _ldOrderWeight(int from, int i, int n) {
  final along = n > 1 ? i / (n - 1) : 0.0, fromCentre = (along - .5).abs() * 2;
  return switch (from) {
    1 => fromCentre,
    2 => 1 - along,
    3 => 1 - fromCentre,
    _ => along,
  };
}

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
        _LdGrid([
          _LdCell(ParamTile('time', const Param(id: 'speed', label: 'Speed', look: ParamLook.bar, type: WType.f32, min: 0, max: 400, def: 100, unit: '%'))),
          const _LdCell(_LdToggle('Loop', 'time_loop')),
          if (loop) ...[
            _LdCell(_ldNum(doc, 'time_loop.dur', 'Length', unit: 's', dec: 2, min: 0, max: 3600, perPx: .02, step: .1, zero: 'Off')),
            const _LdCell(_LdSeg('Loop direction', 'time_loop.dir', _ldLoopDirs, words: ['→', '←', '↔', '↔ rev']), span: 2),
          ],
          const _LdCell(_LdToggle('Ghost', 'time_ghost')),
          if (ghost) _LdCell(_ldNum(doc, 'time_ghost.f', 'Ghost delay', unit: 'f', min: -120, max: 120, perPx: .25, zero: 'None', bipolar: true)),
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
            _LdGrid([
              _LdCell(_ldNum(doc, 'time_stagger', 'Stagger', unit: 's', dec: 2, min: 0, max: 60, perPx: .01, step: .05, zero: 'Off')),
              const _LdCell(_LdSeg('From', 'time_from', _ldFrom), span: 2),
              const _LdCell(_LdToggle('From end', 'time_fromEnd')),
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
          _LdGrid([
            if (grid) ...[
              _LdCell(
                _ldNum(
                  doc,
                  'hl_cols',
                  'Columns',
                  min: 1,
                  max: 64,
                  perPx: .1,
                  glyph: _Steps(id: 'hl_cols', p: _ldCount(1)),
                  owns: true,
                  tall: true,
                ),
              ),
              _LdCell(
                _ldNum(
                  doc,
                  'hl_rows',
                  'Rows',
                  min: 0,
                  max: 64,
                  perPx: .1,
                  zero: 'Auto',
                  glyph: _Steps(id: 'hl_rows', p: _ldCount(0)),
                  owns: true,
                  tall: true,
                ),
              ),
            ],
            if (flex) ...[
              const _LdCell(_LdPick('Direction', 'hl_dir', _ldDirections, word: '')),
              const _LdCell(_LdSeg('Wrap', 'hl_wrap', ['No Wrap', 'Wrap', 'Wrap Reverse'], words: ['No', 'Wrap', 'Rev']), span: 2),
            ],
            _LdCell(_ldNum(doc, 'hl_gap', 'Gap', unit: 'px', min: 0, max: 1000, glyph: _LdMini(_LdMiniKind.gap, doc.get('hl_gap') ?? 0))),
            _LdCell(_ldNum(doc, 'hl_padx', 'Padding X', unit: 'px', min: 0, max: 1000, glyph: _LdMini(_LdMiniKind.padX, doc.get('hl_padx') ?? 0))),
            _LdCell(_ldNum(doc, 'hl_pady', 'Padding Y', unit: 'px', min: 0, max: 1000, glyph: _LdMini(_LdMiniKind.padY, doc.get('hl_pady') ?? 0))),
            const _LdCell(_LdAlignPad()),
            const _LdCell(_LdPick('Justify', 'hl_justify', _ldJustify, word: '')),
            const _LdCell(_LdPick('Items', 'hl_align', _ldAlign, word: '')),
            const _LdCell(_LdSeg('Width', 'hl_wsize', _ldSizing), span: 2),
            _LdCell(_ldNum(doc, 'hl_w', 'W', unit: 'px', min: 0, max: 100000, enabled: wFixed)),
            const _LdCell(_LdSeg('Height', 'hl_hsize', _ldSizing), span: 2),
            _LdCell(_ldNum(doc, 'hl_h', 'H', unit: 'px', min: 0, max: 100000, enabled: hFixed)),
          ]),
        _LdAdvanced(
          id: 'hl',
          count: 10,
          children: [
            const _LdSub('As a child', note: 'Its own lines in Jewel Field'),
            _LdGrid([
              const _LdCell(_LdToggle('Ignore layout', 'hl_abs')),
              const _LdCell(_LdPick('Align self', 'hl_self', _ldSelf, word: '')),
              _LdCell(_ldNum(doc, 'hl_margin', 'Margin', unit: 'px', min: 0, max: 100000)),
              _LdCell(_ldNum(doc, 'hl_colStart', 'Column start', min: 0, max: 64, perPx: .1, zero: 'Auto')),
              _LdCell(_ldNum(doc, 'hl_rowStart', 'Row start', min: 0, max: 64, perPx: .1, zero: 'Auto')),
              _LdCell(_ldNum(doc, 'hl_colSpan', 'Column span', min: 1, max: 64, perPx: .1)),
              _LdCell(_ldNum(doc, 'hl_rowSpan', 'Row span', min: 1, max: 64, perPx: .1)),
              _LdCell(_ldNum(doc, 'hl_shrink', 'Shrink', dec: 1, min: 0, max: 1000, perPx: .05, step: .1)),
            ]),
            const _LdSub('Box'),
            _LdGrid([
              const _LdCell(_LdSeg('Overflow', 'hl_overflow', ['Visible', 'Clip', 'Bounce']), span: 2),
              _LdCell(_ldNum(doc, 'hl_radius', 'Corner radius', unit: 'px', min: 0, max: 100000)),
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
  Widget build(BuildContext context) => CustomPaint(size: const Size(Pop.glyph, Pop.glyph), painter: _LdMiniPaint(kind, v));
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

    return _LdTile(
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
          _LdGrid([
            _LdCell(_LdPick('Drives', '$k.to', _ldTargets, word: '')),
            _LdCell(_LdPick('From layer', '$k.layer', _ldLayers, word: '')),
            _LdCell(_LdPick('Its property', '$k.prop', _ldSources, word: '')),
            _LdCell(ParamTile('link', Param(id: '$i.mul', label: 'Multiply', look: ParamLook.bipolar, type: WType.f32, min: -4, max: 4, dec: 2, def: 1))),
            _LdCell(ParamTile('link', Param(id: '$i.add', label: 'Offset', look: ParamLook.scrub, type: WType.f32, dec: 1))),
            _LdCell(_ldNum(doc, '$k.time', 'Delay', unit: 'f', min: -120, max: 120, perPx: .25, zero: 'None', bipolar: true)),
          ]),
      ],
    );
  }
}
