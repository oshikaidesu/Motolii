// The Inspector body: declaration rows in, controls out. Hero in front, sections, advanced behind a fold,
// a filter for very long lists. Structure is the Classic Inspector's; only the body of each control is new.
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../bp/search.dart';
import '../desk/common.dart' show kBlue, kInk;
import 'rows.dart';
import 'toys.dart';
import 'tones.dart';

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
          final narrow = box.maxWidth < 250;
          final rows = search.active ? search.apply(widget.store.rows, (r) => ['${r['label']}', '${r['id']}', '${r['section'] ?? ''}']) : widget.store.rows;
          final entries = layoutOf(rows, advancedOpen: advancedOpen, flat: search.active, narrow: narrow);
          return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 10, 12, 6),
              child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                Row(children: [
                  Expanded(child: Text(widget.subject, softWrap: false, overflow: TextOverflow.clip, style: sans(13.5, c: kInk, w: FontWeight.w600))),
                  if (widget.store.frozen) Container(margin: const EdgeInsets.only(right: 8), padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3), decoration: BoxDecoration(border: Border.all(color: const Color(0xFF45464C)), borderRadius: BorderRadius.circular(10)), child: Text('Frozen', style: sans(9.5, c: kMuted, w: FontWeight.w600))),
                  Text('${widget.store.rows.length}', key: const ValueKey('insp-count'), style: mono(10, c: kMuted)),
                ]),
                const SizedBox(height: 6),
                SearchField(search, 'Filter', height: 24),
              ]),
            ),
            Expanded(
              child: entries.isEmpty
                  ? Padding(padding: const EdgeInsets.all(20), child: Text('No parameter matches', style: sans(12, c: kMuted)))
                  : ListView.builder(
                      key: const ValueKey('insp-list'),
                      controller: scroll,
                      padding: const EdgeInsets.fromLTRB(12, 2, 12, 20),
                      itemCount: entries.length,
                      itemBuilder: (_, i) => _entry(entries[i]),
                    ),
            ),
          ]);
        }),
      );

  Widget _entry(PEntry e) => paramEntry(e, widget.store, tones, () => setState(() => advancedOpen = !advancedOpen));
}

Widget paramEntry(PEntry e, ParamStore store, Tones tones, VoidCallback toggleAdvanced) => switch (e) {
        PSection(:final label) => Padding(padding: const EdgeInsets.only(top: 12, bottom: 5), child: Row(children: [Container(key: ValueKey('tone-$label'), width: 3, height: 9, margin: const EdgeInsets.only(right: 6), decoration: BoxDecoration(color: tones.ofGroup(label), borderRadius: BorderRadius.circular(1.5))), Text(label.toUpperCase(), style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.3))])),
        PFold(:final count, :final open) => GestureDetector(
            key: const ValueKey('advanced-fold'),
            behavior: HitTestBehavior.opaque,
            onTap: () => toggleAdvanced(),
            child: Padding(padding: const EdgeInsets.only(top: 12, bottom: 5), child: Row(children: [Container(width: 3, height: 9, margin: const EdgeInsets.only(right: 6), decoration: BoxDecoration(color: tones.advanced, borderRadius: BorderRadius.circular(1.5))), Text(open ? '▾' : '▸', style: sans(10, c: kMuted)), const SizedBox(width: 6), Text('ADVANCED', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.3)), const SizedBox(width: 6), Text('$count', style: mono(9.5, c: const Color(0xFF6E6F76)))])),
          ),
        PCells(:final rows, :final hero) => Padding(
            padding: const EdgeInsets.only(bottom: 7),
            child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
              for (final (i, r) in rows.indexed) ...[
                Expanded(child: ParamCell(store, r, hero: hero, tone: tones.of(r))),
                if (i < rows.length - 1) const SizedBox(width: 6),
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
          final entries = layoutOf(widget.store.rows, advancedOpen: advancedOpen, narrow: box.maxWidth < 250);
          return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            for (final e in entries) paramEntry(e, widget.store, tones, () => setState(() => advancedOpen = !advancedOpen)),
          ]);
        }),
      );
}

/// One declared parameter: its word, its marks, its Toy. Marks: a dot when it is not at its default,
/// a diamond when it is animated (filled when keyed on this frame), and a reset when there is somewhere to go back to.
class ParamCell extends StatelessWidget {
  const ParamCell(this.store, this.row, {super.key, this.hero = false, this.tone = kBlue});
  final ParamStore store;
  final Map<String, dynamic> row;
  final bool hero;
  final Color tone;
  @override
  Widget build(BuildContext context) {
    final id = '${row['id']}';
    final kind = kindOf(row);
    final ids = kind == PKind.pair ? (row['ids'] as List).cast<String>() : [id];
    final mod = ids.any((i) => modified(store.row(i)));
    final label = '${row['label'] ?? id}';
    final Widget toy = switch (kind) {
      PKind.scalar || PKind.bounded || PKind.integer => ValueToy(Slot(store, id), hero: hero, tone: tone),
      PKind.toggle => ToggleToy(store, id, tone: tone, hero: hero),
      PKind.choice => ChoiceToy(store, id, tone: tone),
      PKind.vec2 => ValuesToy(store, id, 2, hero: hero, tone: tone),
      PKind.vec3 => ValuesToy(store, id, 3, hero: hero, tone: tone),
      PKind.pair => PairToy(store, ids, id, hero: hero, tone: tone),
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
    return Column(key: ValueKey('cell-$id'), crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      SizedBox(
        height: 15,
        child: Row(children: [
          // a diamond when the value is animated; where the store can key, an unkeyed value shows a faint one and any of them keys this frame
          if (row['animated'] == true || store.keyable)
            GestureDetector(
              key: ValueKey('key-$id'),
              behavior: HitTestBehavior.opaque,
              onTap: store.keyable && !store.frozen ? () => store.toggleKey(id) : null,
              child: Padding(
                padding: const EdgeInsets.only(right: 4),
                child: SizedBox(key: ValueKey('anim-$id'), width: 7, height: 7, child: CustomPaint(painter: _Diamond(row['keyedNow'] == true, row['animated'] == true ? tone : dimTone(tone)))),
              ),
            ),
          Expanded(child: Row(children: [
            Flexible(child: Text(label, softWrap: false, overflow: TextOverflow.ellipsis, style: sans(hero ? 11.5 : 11, c: hero ? kInk : const Color(0xFFB4B6BB), w: hero ? FontWeight.w600 : FontWeight.w500))),
            if (mod) Padding(padding: const EdgeInsets.only(left: 5), child: Container(key: ValueKey('mod-$id'), width: 4, height: 4, decoration: BoxDecoration(color: t, shape: BoxShape.circle))),
          ])),
          if (linkable) GestureDetector(key: ValueKey('link-$id'), behavior: HitTestBehavior.opaque, onTap: store.frozen ? null : () => store.toggleLink(id), child: Container(margin: const EdgeInsets.only(left: 6), padding: const EdgeInsets.symmetric(horizontal: 6), decoration: BoxDecoration(color: linked ? t : null, border: linked ? null : Border.all(color: const Color(0xFF45464C)), borderRadius: BorderRadius.circular(6)), child: Text('Link', style: sans(8.5, c: linked ? const Color(0xFF1B1B1D) : kMuted, w: FontWeight.w700)))),
          if (accessory != null) GestureDetector(key: ValueKey('route-acc-$id'), behavior: HitTestBehavior.opaque, onTap: () => store.route(accessory, id), child: Container(margin: const EdgeInsets.only(left: 6), padding: const EdgeInsets.symmetric(horizontal: 6), decoration: BoxDecoration(border: Border.all(color: t.withValues(alpha: .7)), borderRadius: BorderRadius.circular(6)), child: Text('$accessory →', style: sans(8.5, c: t, w: FontWeight.w700)))),
          if (mod && !store.frozen) GestureDetector(key: ValueKey('reset-$id'), behavior: HitTestBehavior.opaque, onTap: () { for (final i in ids) { store.reset(i); } }, child: Padding(padding: const EdgeInsets.only(left: 8), child: Text('↺', style: sans(11, c: const Color(0xFF55565C))))),
        ]),
      ),
      const SizedBox(height: 3),
      toy,
      if (actions.isNotEmpty) Padding(padding: const EdgeInsets.only(top: 4), child: Wrap(spacing: 5, children: [for (final a in actions) ActionChip(store, id, a, tone: tone)])),
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
