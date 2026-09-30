// The Inspector body: declaration rows in, controls out. Hero in front, sections, advanced behind a fold,
// a filter for very long lists. Structure is the Classic Inspector's; only the body of each control is new.
import 'package:flutter/widgets.dart';
import '../../browser/parts.dart';
import '../../browser/search.dart';
import '../desk/common.dart' show kBlue;
import 'rows.dart';
import 'toys.dart';
import '../metrics.dart';
import 'tones.dart';
import '../neutral.dart';
import '../shell/place.dart' show H;

/// Heroes: the declared ones, else the first four that are not advanced (four in front, the rest behind).
/// A front projection, not a limit: everything else is reached through sections, the fold and the filter.
Set<String> heroIds(List<Map<String, dynamic>> rows) {
  final plain = rows.where((r) => r['advanced'] != true).toList();
  final declared = plain.where((r) => r['hero'] == true).toList();
  return {for (final r in declared.isNotEmpty ? declared : (plain.length > 4 ? plain.take(4) : const <Map<String, dynamic>>[])) '${r['id']}'};
}

bool isHalf(Map<String, dynamic> r) => const {PKind.scalar, PKind.bounded, PKind.integer, PKind.toggle}.contains(kindOf(r));

sealed class PEntry {}

class PSection extends PEntry { PSection(this.label); final String label; }
class PFold extends PEntry { PFold(this.count, this.open); final int count; final bool open; }
class PCells extends PEntry { PCells(this.rows, this.hero); final List<Map<String, dynamic>> rows; final bool hero; }

List<PEntry> layoutOf(List<Map<String, dynamic>> declared, {required bool advancedOpen, bool flat = false, bool narrow = false}) {
  final rows = foldPairs(declared);
  final out = <PEntry>[];
  void cells(List<Map<String, dynamic>> list, bool hero) {
    var i = 0;
    while (i < list.length) {
      final a = list[i];
      if (!narrow && isHalf(a) && i + 1 < list.length && isHalf(list[i + 1])) { out.add(PCells([a, list[i + 1]], hero)); i += 2; }
      else { out.add(PCells([a], hero)); i++; }
    }
  }

  if (flat) {
    String? sec;
    var run = <Map<String, dynamic>>[];
    for (final r in rows) {
      final s = r['section'] as String?;
      if (s != sec) { cells(run, false); run = []; if (s != null) out.add(PSection(s)); sec = s; }
      run.add(r);
    }
    cells(run, false);
    return out;
  }
  final heroes = heroIds(rows);
  cells([for (final r in rows) if (heroes.contains('${r['id']}')) r], true);
  String? sec;
  var run = <Map<String, dynamic>>[];
  for (final r in rows) {
    if (heroes.contains('${r['id']}') || r['advanced'] == true) continue;
    final s = r['section'] as String?;
    if (s != sec) { cells(run, false); run = []; if (s != null) out.add(PSection(s)); sec = s; }
    run.add(r);
  }
  cells(run, false);
  final adv = [for (final r in rows) if (r['advanced'] == true) r];
  if (adv.isNotEmpty) {
    out.add(PFold(adv.length, advancedOpen));
    if (advancedOpen) cells(adv, false);
  }
  return out;
}

class InspectorBody extends StatefulWidget {
  const InspectorBody({super.key, required this.store, required this.subject, this.thingId, this.search, this.advancedOpen = false, this.initialOffset = 0});
  final String? thingId; // what the colours are dealt from; the subject when not given
  final ParamStore store;
  final String subject;
  final SearchCapability? search;
  final bool advancedOpen; // starting state of the fold
  final double initialOffset;
  @override
  State<InspectorBody> createState() => _InspectorBodyState();
}

class _InspectorBodyState extends State<InspectorBody> {
  late bool advancedOpen = widget.advancedOpen;
  late final SearchCapability search = widget.search ?? SearchCapability();
  late final Tones tones = Tones(widget.thingId ?? widget.subject, foldPairs(widget.store.rows));
  late final ScrollController scroll = ScrollController(initialScrollOffset: widget.initialOffset);

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([widget.store, search]),
        builder: (context, _) => LayoutBuilder(builder: (context, box) {
          final narrow = box.maxWidth < 188;
          final rows = search.active ? search.apply(widget.store.rows, (r) => ['${r['label']}', '${r['id']}', '${r['section'] ?? ''}']) : widget.store.rows;
          final entries = layoutOf(rows, advancedOpen: advancedOpen, flat: search.active, narrow: narrow);
          return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(Surface.panelInset, Surface.sectionGap, Surface.panelInset, Surface.inlineGap),
              child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                Row(children: [
                  Expanded(child: Text(widget.subject, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600))),
                  if (widget.store.frozen) Container(margin: const EdgeInsets.only(right: Surface.sectionGap), padding: const EdgeInsets.symmetric(horizontal: Surface.sectionGap, vertical: Surface.hair), decoration: BoxDecoration(border: Border.all(color: Surface.selected), borderRadius: BorderRadius.circular(Surface.faceRadius)), child: Text('Frozen', style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w600))),
                  Text('${widget.store.rows.length}', key: const ValueKey('insp-count'), style: mono(Dn.labelSize, c: Surface.muted)),
                ]),
                const SizedBox(height: Surface.inlineGap),
                SearchField(search, 'Filter', height: Surface.control),
              ]),
            ),
            Expanded(
              child: entries.isEmpty
                  ? Padding(padding: const EdgeInsets.all(Surface.panelInset), child: Text('No parameter matches', style: sans(Dn.nameSize, c: Surface.muted)))
                  : ListView.builder(
                      key: const ValueKey('insp-list'),
                      controller: scroll,
                      padding: const EdgeInsets.fromLTRB(Surface.panelInset, 0, Surface.panelInset, Surface.panelInset),
                      itemCount: entries.length,
                      itemBuilder: (_, i) => _entry(entries[i]),
                    ),
            ),
          ]);
        }),
      );

  Widget _entry(PEntry e) => paramEntry(e, widget.store, tones, () => setState(() => advancedOpen = !advancedOpen));
}

Widget paramEntry(PEntry e, ParamStore store, Tones tones, VoidCallback toggleAdvanced, {bool inline = false}) => switch (e) {
        PSection(:final label) => Padding(padding: const EdgeInsets.only(top: Surface.sectionGap, bottom: Surface.inlineGap), child: Row(children: [Container(key: ValueKey('tone-$label'), width: Surface.focusStroke, height: Surface.mark, margin: const EdgeInsets.only(right: Surface.inlineGap), decoration: BoxDecoration(color: tones.ofGroup(label), borderRadius: BorderRadius.circular(Surface.hair))), Text(label.toUpperCase(), style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.3))])),
        PFold(:final count, :final open) => GestureDetector(
            key: const ValueKey('advanced-fold'),
            behavior: HitTestBehavior.opaque,
            onTap: () => toggleAdvanced(),
            child: Padding(padding: const EdgeInsets.only(top: Surface.sectionGap, bottom: Surface.inlineGap), child: Row(children: [Container(width: Surface.focusStroke, height: Surface.mark, margin: const EdgeInsets.only(right: Surface.inlineGap), decoration: BoxDecoration(color: tones.advanced, borderRadius: BorderRadius.circular(Surface.hair))), Text(open ? '▾' : '▸', style: sans(Dn.labelSize, c: Surface.muted)), const SizedBox(width: Surface.inlineGap), Text('ADVANCED', style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.3)), const SizedBox(width: Surface.inlineGap), Text('$count', style: mono(Dn.microSize, c: N.g44))])),
          ),
        PCells(:final rows, :final hero) => Padding(
            padding: EdgeInsets.only(bottom: inline ? 0 : Surface.cellGap),
            child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
              for (final (i, r) in rows.indexed) ...[
                Expanded(child: ParamCell(store, r, hero: hero, tone: tones.of(r), inline: inline, alone: rows.length == 1)),
                if (i < rows.length - 1) const SizedBox(width: Surface.inlineGap),
              ],
            ]),
          ),
      };

/// The rows of one ParamStore laid out as the Inspector body lays them (heroes, sections, the advanced fold, routes),
/// without the header or a scroll of its own: for a card inside a longer panel, such as one effect's parameters.
class ParamSheet extends StatefulWidget {
  const ParamSheet(this.store, {super.key, required this.thingId, this.advancedOpen = false});
  final ParamStore store;
  final String thingId; // what the colours are dealt from
  final bool advancedOpen;
  @override
  State<ParamSheet> createState() => _ParamSheetState();
}

class _ParamSheetState extends State<ParamSheet> {
  late bool advancedOpen = widget.advancedOpen;
  late Tones tones = Tones(widget.thingId, foldPairs(widget.store.rows));

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: widget.store,
        builder: (context, _) => LayoutBuilder(builder: (context, box) {
          final entries = layoutOf(widget.store.rows, advancedOpen: advancedOpen, narrow: box.maxWidth < 188);
          return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            for (final e in entries) paramEntry(e, widget.store, tones, () => setState(() => advancedOpen = !advancedOpen), inline: true),
          ]);
        }),
      );
}

/// One declared parameter: its word, its marks, its Toy. Marks: a dot when it is not at its default,
/// a diamond when it is animated (filled when keyed on this frame), and a reset when there is somewhere to go back to.
/// The label : value split of a two-up line; a line alone widens the value by a whole cell.
const _labelFlex = 4, _valueFlex = 5;

class ParamCell extends StatelessWidget {
  const ParamCell(this.store, this.row, {super.key, this.hero = false, this.tone = kBlue, this.inline = false, this.alone = false});
  final ParamStore store;
  final Map<String, dynamic> row;
  final bool hero;
  final Color tone;

  /// One line of the work: the label left, the control right, at [Surface.workRow].
  final bool inline;

  /// An inline cell that has its line to itself keeps the label column of the two-up lines, so the values align down the sheet.
  final bool alone;
  @override
  Widget build(BuildContext context) {
    final id = '${row['id']}';
    final kind = kindOf(row);
    final ids = kind == PKind.pair ? (row['ids'] as List).cast<String>() : [id];
    final mod = ids.any((i) => modified(store.row(i)));
    final label = '${row['label'] ?? id}';
    final Widget toy = switch (kind) {
      PKind.scalar || PKind.bounded || PKind.integer => ValueToy(Slot(store, id), hero: hero && !inline, tone: tone),
      PKind.toggle => ToggleToy(store, id, tone: tone, hero: hero && !inline),
      PKind.choice => ChoiceToy(store, id, tone: tone),
      PKind.vec2 => ValuesToy(store, id, 2, hero: hero && !inline, tone: tone),
      PKind.vec3 => ValuesToy(store, id, 3, hero: hero && !inline, tone: tone),
      PKind.pair => PairToy(store, ids, id, hero: hero && !inline, tone: tone),
      PKind.text => TextToy(store, id),
      PKind.reference => ReferenceToy(store, id, tone: tone),
      PKind.route => RouteToy(store, id, tone: tone),
      PKind.raw => TextToy(store, id, rawJson: true),
    };
    // a seed is a Value with a Reroll, unless the declaration already says what its actions are
    final actions = (row['actions'] as List?)?.cast<String>() ?? (kind == PKind.scalar || kind == PKind.integer ? (characterOf(row) == Character.seed ? const ['Reroll'] : const <String>[]) : const <String>[]);
    // a number with a specialist view keeps its exact editing and points to the specialist
    final accessory = kind != PKind.route ? row['route'] as String? : null;
    final linkable = row['linkable'] == true;
    final linked = store.linked.contains(id);
    final t = store.frozen ? dimTone(tone) : tone;
    // the label line: a diamond when the value is animated (where the store can key, an unkeyed value shows a faint one and any of
    // them keys this frame), its word, a dot when it is off its default, and the pills (relation, link, route) and reset
    final labelLine = Row(children: [
      if (row['animated'] == true || store.keyable)
        GestureDetector(
          key: ValueKey('key-$id'),
          behavior: HitTestBehavior.opaque,
          onTap: store.keyable && !store.frozen ? () => store.toggleKey(id) : null,
          child: Padding(
            padding: const EdgeInsets.only(right: Surface.inlineGap),
            child: SizedBox(key: ValueKey('anim-$id'), width: Surface.mark, height: Surface.mark, child: CustomPaint(painter: _Diamond(row['keyedNow'] == true, row['animated'] == true ? tone : dimTone(tone)))),
          ),
        ),
      Expanded(child: Row(children: [
        Flexible(child: Text(label, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(Dn.nameSize, c: hero ? N.g86 : N.g69, w: hero ? FontWeight.w600 : FontWeight.w500))),
        if (mod) Padding(padding: const EdgeInsets.only(left: Surface.inlineGap), child: Container(key: ValueKey('mod-$id'), width: Surface.mark, height: Surface.mark, decoration: BoxDecoration(color: t, shape: BoxShape.circle))),
      ])),
      // a relation: this value is driven by another (source), or drives others (how many)
      if (row['link'] is Map || (row['drives'] is int && row['drives'] > 0))
        GestureDetector(
          key: ValueKey('relation-$id'),
          behavior: HitTestBehavior.opaque,
          onTap: () => store.focusRelation(id),
          child: Container(
            margin: const EdgeInsets.only(left: Surface.inlineGap),
            padding: const EdgeInsets.symmetric(horizontal: Surface.inlineGap),
            decoration: BoxDecoration(color: H.relation, borderRadius: BorderRadius.circular(Surface.controlRadius)),
            child: Text(row['link'] is Map ? '◉ ${(row['link'] as Map)['name'] ?? 'Relation'}' : '◉ ${row['drives']}', style: sans(Dn.microSize, c: N.g10, w: FontWeight.w700)),
          ),
        ),
      if (linkable) GestureDetector(key: ValueKey('link-$id'), behavior: HitTestBehavior.opaque, onTap: store.frozen ? null : () => store.toggleLink(id), child: Container(margin: const EdgeInsets.only(left: Surface.inlineGap), padding: const EdgeInsets.symmetric(horizontal: Surface.inlineGap), decoration: BoxDecoration(color: linked ? t : null, border: linked ? null : Border.all(color: N.g26), borderRadius: BorderRadius.circular(Surface.controlRadius)), child: Text('Link', style: sans(Dn.microSize, c: linked ? N.g10 : Surface.muted, w: FontWeight.w700)))),
      if (accessory != null) GestureDetector(key: ValueKey('route-acc-$id'), behavior: HitTestBehavior.opaque, onTap: () => store.route(accessory, id), child: Container(margin: const EdgeInsets.only(left: Surface.inlineGap), padding: const EdgeInsets.symmetric(horizontal: Surface.inlineGap), decoration: BoxDecoration(border: Border.all(color: t.withValues(alpha: .7)), borderRadius: BorderRadius.circular(Surface.controlRadius)), child: Text('$accessory →', style: sans(Dn.microSize, c: t, w: FontWeight.w700)))),
      if (mod && !store.frozen) GestureDetector(key: ValueKey('reset-$id'), behavior: HitTestBehavior.opaque, onTap: () => store.resetMany(ids), child: Padding(padding: const EdgeInsets.only(left: Surface.sectionGap), child: Text('↺', style: sans(Dn.nameSize, c: N.g33)))),
    ]);
    final chips = actions.isEmpty ? null : Wrap(spacing: Surface.inlineGap, children: [for (final a in actions) ActionChip(store, id, a, tone: tone)]);
    // Work density: the label is the row's left, the control its right (one line of the work), not a header over a gap over a control
    if (inline) {
      return Column(key: ValueKey('cell-$id'), crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        SizedBox(
          height: Surface.workRow,
          child: Row(children: [
            Expanded(flex: _labelFlex, child: labelLine),
            const SizedBox(width: Surface.inlineGap),
            Expanded(flex: alone ? _labelFlex + 2 * _valueFlex : _valueFlex, child: Center(child: toy)),
          ]),
        ),
        if (chips != null) Padding(padding: const EdgeInsets.only(bottom: Surface.inlineGap), child: chips),
      ]);
    }
    return Column(key: ValueKey('cell-$id'), crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      SizedBox(height: Surface.labelRow, child: labelLine),
      const SizedBox(height: Surface.labelGap),
      toy,
      if (chips != null) Padding(padding: const EdgeInsets.only(top: Surface.inlineGap), child: chips),
    ]);
  }
}

class _Diamond extends CustomPainter {
  _Diamond(this.filled, this.color);
  final bool filled;
  final Color color;
  @override
  void paint(Canvas c, Size s) {
    final p = Path()..moveTo(s.width / 2, 0)..lineTo(s.width, s.height / 2)..lineTo(s.width / 2, s.height)..lineTo(0, s.height / 2)..close();
    c.drawPath(p, Paint()..color = color..style = filled ? PaintingStyle.fill : PaintingStyle.stroke..strokeWidth = 1.4);
  }
  @override
  bool shouldRepaint(_Diamond o) => o.filled != filled || o.color != color;
}
