import 'dart:math' as math;
import 'inspector_studies.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_effects_effect_card.dart';
import 'package:lab_book/sets/inspector/inspector_param_controls_tile.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';
part 'inspector_host_layer_2.dart';

// ---- Layer data: masks, the layer's clock, layout, property links ---------------------------------------------------
// Sources: motolii-doc store/mask.rs, attrs.rs (matte, ghost), layout/clock.rs + time.rs, layout.rs (GROUP_ROWS / ITEM_ROWS), slot.rs (PropertyLink).

/// An On/Off block over doc.b.
class _LdToggle extends StatelessWidget {
  const _LdToggle(this.label, this.id);
  final String label, id;
  @override
  Widget build(BuildContext context) {
    final doc = Ctx.of(context).doc, b = doc.b[id] ?? false;
    final on = !Ctx.of(context).cfg.locked && !RowOff.offOf(context);
    return LdTile(
      label: label,
      onTap: () => doc.flag(id, !b),
      child: Padding(
        padding: const EdgeInsets.only(bottom: 3),
        child: Row(
          children: [
            IgnorePointer(
              child: Opacity(
                opacity: on ? 1 : .4,
                child: PopSwitch(on: b, onChanged: (_) {}),
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
    return LdTile(
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
  const _LdPick(this.label, this.id, this.options);
  final String label, id;
  final List<String> options;
  @override
  Widget build(BuildContext context) => LdTile(
    label: label,
    child: SizedBox(
      height: 22,
      child: Chooser(id: id, options: options),
    ),
  );
}

/// A number block (NumField caption form) carrying the row's key dot.
Widget ldNum(
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
}) => RowInfo(
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
        LdGrid([
          const LdCell(_LdPick('Matte layer', 'mask_matte', _ldMatteLayers)),
          if (matte != 'None') const LdCell(_LdSeg('Use', 'mask_matte.mode', _ldMatteModes, words: ['Alpha', 'Inv α', 'Luma', 'Inv L']), span: 2),
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
            child: Chooser(id: '$k.mode', options: _ldMaskModes),
          ),
        ),
        if (open)
          LdGrid([
            LdCell(ParamTile('mask', Param(id: '$i.feather', label: 'Feather', look: ParamLook.bar, type: WType.f32, min: 0, max: 500, unit: 'px'))),
            LdCell(ParamTile('mask', Param(id: '$i.exp', label: 'Expansion', look: ParamLook.bipolar, type: WType.f32, min: -200, max: 200, unit: 'px'))),
            LdCell(ParamTile('mask', Param(id: '$i.op', label: 'Opacity', look: ParamLook.percent, type: WType.f32, min: 0, max: 100, unit: '%'))),
            LdCell(
              ParamTile(
                'mask',
                Param(id: '$i.frame', label: 'Frame', look: ParamLook.choice, type: WType.u32, min: 0, max: 1, options: const ['Layer', 'Box']),
              ),
              span: 2,
            ),
            LdCell(ParamTile('mask', Param(id: '$i.inv', label: 'Inverted', look: ParamLook.toggle, type: WType.u32, min: 0, max: 1))),
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
