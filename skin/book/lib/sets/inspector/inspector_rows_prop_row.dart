// One property: label, its value cells, the key mark, the reset mark. The columns are the same on every row (B1) so X/Y/Z line up.
import 'package:flutter/foundation.dart' show ValueListenable, listEquals;
import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_value_marks.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

/// One property: label, its value cells, the key mark, the reset mark. The columns are the same on every row (B1) so X/Y/Z line up.
/// contract: right-click a row for Key this frame / Remove key / Remove animation; unavailable ones say why (D3).
class PropRow extends StatefulWidget {
  const PropRow({
    super.key,
    required this.id,
    required this.label,
    required this.cells,
    this.ids,
    this.keyable = true,
    this.enabled = true,
    this.under,
    this.lead,
    this.trail,
    this.note,
    this.rel,
    this.relName,
  });
  final String id, label;
  final List<Widget> cells;
  final List<String>? ids;
  final bool keyable, enabled;
  final Widget? under, lead, trail;
  final String? note, relName;
  final Fam? rel;
  @override
  State<PropRow> createState() => _PropRowState();
}

class _PropRowState extends State<PropRow> {
  bool _menu = false;

  // (its key mark and whether it differs from as it comes, at a new moment: told alone, as its fields are)
  final _watching = <ValueListenable<int>>[];
  void _moved() {
    if (mounted) setState(() {});
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final doc = Ctx.read(context).doc, now = [doc.watch('key:${widget.id}'), for (final i in widget.ids ?? [widget.id]) doc.watch(i)];
    if (listEquals(now, _watching)) return;
    for (final w in _watching) {
      w.removeListener(_moved);
    }
    _watching
      ..clear()
      ..addAll(now);
    for (final w in now) {
      w.addListener(_moved);
    }
  }

  @override
  void dispose() {
    for (final w in _watching) {
      w.removeListener(_moved);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), cfg = x.cfg, doc = x.doc, ed = Ed(cfg.ed);
    final ids = widget.ids ?? [widget.id];
    final changed = ids.any((i) => doc.changed(i) && (Role.grey || doc.get(i) != null)),
        mixedAny = ids.any((i) => doc.get(i) == null),
        ks = doc.keys[widget.id] ?? KeyS.off;
    final active = doc.directHover == widget.id;
    final mark = widget.lead ?? RowMark.of(context, widget.id) ?? (valueMarks[widget.id] == null ? null : ValueMarkGlyph(valueMarks[widget.id]!, doc.get(widget.id)));
    final pop = Pop.on(context);
    final units = widget.cells.whereType<NumField>().map((f) => f.unit).toSet();
    final shared = pop && units.length == 1 ? units.single : null;
    final unit = shared != null && _fits('${ed.lab(widget.label)} $shared', ed.labStyle(Grey.g63), labelWPop - 34 - 16) ? shared : null;
    final locked = cfg.locked;
    final showRel = cfg.maybe && widget.rel != null;
    final cellsOn = widget.enabled && !showRel && !locked;
    return LayoutBuilder(
      builder: (context, box) {
        final lw = pop ? labelWPop : (box.maxWidth < 300 ? labelWNarrow : labelW);
        final slots = (pop ? 0 : keyW) + (cfg.resetArrow ? resetW : 0);
        final cellsW = box.maxWidth - lw - slots - 8;
        final stack = widget.cells.length == 3 && (cellsW - 8) / 3 < (pop ? 56 : stack3);
        Widget gap(double w) => SizedBox(width: w);
        Widget cellsRow(List<Widget> cs) => Row(
          children: [
            for (var i = 0; i < cs.length; i++) ...[if (i > 0) gap(4), Expanded(child: cs[i])],
          ],
        );
        final cells = stack
            ? Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  cellsRow(widget.cells.take(2).toList()),
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      Expanded(child: widget.cells[2]),
                      gap(4),
                      const Expanded(child: SizedBox()),
                    ],
                  ),
                ],
              )
            : cellsRow(widget.cells);
        final label = Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                if (mark != null)
                  _Scrub( // (the row's mark is its handle too: pulled sideways, its number moves, as Figma's field glyphs)
                    field: cellsOn ? widget.cells.whereType<NumField>().where((f) => f.id == widget.id).firstOrNull : null,
                    child: Padding(
                    padding: EdgeInsets.only(right: Shape.s(6)),
                    child: Opacity(
                      opacity: cellsOn ? 1 : .4,
                      child: pop
                          ? Container(
                              width: 28,
                              height: Pop.cell,
                              padding: EdgeInsets.all(Shape.s(2)),
                              decoration: BoxDecoration(color: active ? Grey.g20 : null, borderRadius: BorderRadius.all(Shape.r(7))),
                              child: FittedBox(child: mark),
                            )
                          : mark,
                    ),
                  ),
                  )
                else if (pop)
                  const SizedBox(width: 34),
                (unit != null ? Expanded.new : Flexible.new)(
                  child: Text(
                    ed.lab(widget.label),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: ed.labStyle(
                      !cellsOn ? Grey.g56 : (changed ? Role.of(Grey.g95, Role.changed) : (mixedAny ? Role.of(Grey.g63, Role.info) : Grey.g63)),
                    ),
                  ),
                ),
                ?widget.trail,
                if (unit != null) ...[
                  Padding(
                    padding: EdgeInsets.only(right: Shape.s(8)),
                    child: Text(unit, style: T.label(Grey.g44)),
                  ),
                ],
                if (showRel)
                  Padding(
                    padding: EdgeInsets.only(left: Shape.s(4)),
                    child: Container(
                      width: 5,
                      height: 5,
                      decoration: BoxDecoration(color: widget.rel!.c, shape: BoxShape.circle),
                    ),
                  ),
              ],
            ),
            if (widget.under != null) Padding(padding: EdgeInsets.only(top: Shape.s(4), bottom: Shape.s(2)), child: widget.under),
          ],
        );
        Widget item(String s, bool ok, String why, VoidCallback go) => Hov(
          cursor: ok ? SystemMouseCursors.click : SystemMouseCursors.basic,
          onTap: ok && !locked
              ? () {
                  go();
                  setState(() => _menu = false);
                }
              : null,
          builder: (_, h) => Container(
            height: 22,
            padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
            color: h && ok ? Grey.g15 : null,
            child: Row(
              children: [
                Text(s, style: T.name(ok ? Grey.g91 : Grey.g56)),
                const Spacer(),
                if (!ok) Text(why, style: T.label(Grey.g56)),
              ],
            ),
          ),
        );
        return Listener(
          behavior: HitTestBehavior.translucent,
          onPointerDown: (e) {
            if (e.buttons == kSecondaryButton && widget.keyable) setState(() => _menu = !_menu);
          },
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Container(
                constraints: BoxConstraints(minHeight: pop ? Pop.row : ed.row),
                padding: EdgeInsets.symmetric(vertical: Shape.s(1)),
                decoration: pop
                    ? BoxDecoration(color: active ? Grey.g15 : null, borderRadius: BorderRadius.all(Shape.r(6)))
                    : BoxDecoration(
                        color: active ? Grey.g20 : null,
                        border: active ? Border(left: BorderSide(color: Grey.g95, width: 2)) : null,
                      ),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    SizedBox(width: lw, child: label),
                    Expanded(
                      child: RowOff(
                        off: !cellsOn,
                        narrow: box.maxWidth < 300,
                        child: IgnorePointer(
                          ignoring: !cellsOn,
                          child: pop ? RowInfo(unit: unit, keys: widget.keyable ? ks : KeyS.off, child: cells) : cells,
                        ),
                      ),
                    ),
                    // the key column is reserved on every row so every field ends at one edge
                    if (pop)
                      const SizedBox.shrink()
                    else if (widget.keyable)
                      KeyMark(
                        state: ks,
                        enabled: !locked,
                        // contract: click = key at the playhead; on a key here it removes that key (the track stays animated).
                        onTap: () => doc.setKey(widget.id, ks == KeyS.at ? KeyS.anim : KeyS.at),
                      )
                    else
                      const SizedBox(width: keyW),
                    if (cfg.resetArrow) ResetMark(show: (changed || mixedAny) && !locked, onTap: () => doc.resetIds(ids)),
                  ],
                ),
              ),
              if (widget.note != null)
                Padding(
                  padding: EdgeInsets.only(left: lw, bottom: Shape.s(2)),
                  child: Text(widget.note!, style: T.label(Grey.g56)),
                ),
              if (showRel)
                Padding(
                  padding: EdgeInsets.only(left: lw, bottom: Shape.s(2)),
                  child: Text(
                    'Driven by ${widget.rel!.name}${widget.relName != null ? ' · ${widget.relName}' : ''}. Edit it in Relations.',
                    style: T.label(Role.linkedFor(widget.rel)),
                  ),
                ),
              if (_menu)
                Container(
                  margin: EdgeInsets.only(left: Shape.s(0), top: Shape.s(2), bottom: Shape.s(4)),
                  decoration: BoxDecoration(
                    color: Grey.g13,
                    borderRadius: BorderRadius.all(Shape.r(4)),
                    border: Border.all(color: ed.rule),
                  ),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      item('Key this frame', ks != KeyS.at, 'Already keyed', () => doc.setKey(widget.id, KeyS.at)),
                      item('Remove key', ks == KeyS.at, 'No key here', () => doc.setKey(widget.id, KeyS.anim)),
                      item('Remove animation', ks != KeyS.off, 'Not animated', () => doc.setKey(widget.id, KeyS.off)),
                      if (cfg.maybe) item('Relate…', true, '', () {}),
                    ],
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}

bool _fits(String s, TextStyle style, double w) =>
    (TextPainter(
      text: TextSpan(text: s, style: style),
      maxLines: 1,
      textDirection: TextDirection.ltr,
    )..layout()).width <=
    w;

/// A row's mark pulled sideways scrubs the row's own number, as its field does (nothing, when the row has none).
class _Scrub extends StatelessWidget {
  const _Scrub({required this.field, required this.child});
  final NumField? field;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final f = field;
    if (f == null) return child;
    final doc = Ctx.read(context).doc;
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onHorizontalDragStart: (_) => doc.beginGesture(),
        onHorizontalDragUpdate: (d) {
          final v = (doc.get(f.id) ?? 0) + d.delta.dx * f.perPx;
          doc.set(f.id, v.clamp(f.min ?? -double.infinity, f.max ?? double.infinity));
        },
        onHorizontalDragEnd: (_) => doc.endGesture(),
        child: child,
      ),
    );
  }
}
