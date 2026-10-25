// timeline_labels.dart: the names column: its head and a row each, with eye, solo and lock.
part of 'timeline.dart';

class _LabelHeader extends StatelessWidget {
  const _LabelHeader({required this.count, required this.solo});
  final int count;
  final bool solo;
  @override
  Widget build(BuildContext context) => Container(
    height: WtM.ruler,
    padding: EdgeInsets.only(left: Shape.s(8), right: Shape.s(6)),
    decoration: BoxDecoration(
      color: Grey.g10,
      border: Border(bottom: BorderSide(color: WsT.line)),
    ),
    child: Row(
      children: [
        Text('#', style: T.micro(Grey.g56)),
        SizedBox(width: Shape.s(12)),
        Text('LAYER', style: T.micro(Grey.g63).copyWith(letterSpacing: 1)),
        SizedBox(width: Shape.s(6)),
        Text('$count', style: T.value(Grey.g56).copyWith(fontSize: T.px(10))),
        const Spacer(),
        for (final g in const [TpGlyph.eye, TpGlyph.solo, TpGlyph.lock])
          SizedBox(
            width: Shape.z(20),
            child: Center(
              child: CustomPaint(size: const Size.square(10), painter: TpGlyphPainter(g, g == TpGlyph.solo && solo ? WsT.accent : Grey.g56)),
            ),
          ),
      ],
    ),
  );
}

class _LabelRow extends StatefulWidget {
  const _LabelRow({
    required this.index,
    required this.row,
    required this.hidden,
    required this.solo,
    required this.onSelect,
    required this.onEye,
    required this.onSolo,
    required this.onLock,
  });
  final int index;
  final WtRow row;
  final bool hidden, solo;
  final VoidCallback onSelect, onEye, onSolo, onLock;
  @override
  State<_LabelRow> createState() => _LabelRowState();
}

class _LabelRowState extends State<_LabelRow> {
  bool _h = false;
  @override
  Widget build(BuildContext context) {
    final r = widget.row, sel = r.selected;
    return MouseRegion(
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: Forms.worn.track(
        context,
        TrackSpec(
          index: widget.index,
          name: r.name,
          selected: sel,
          dim: r.dim,
          hot: _h,
          onSelect: widget.onSelect,
          kind: WtKindChip(r.kind, dim: r.dim, tone: r.tone),
          eye: _Toggle(widget.hidden ? TpGlyph.eyeOff : TpGlyph.eye, on: false, lit: !widget.hidden, onTap: widget.onEye),
          solo: _Toggle(TpGlyph.solo, on: widget.solo, lit: _h || sel, onTap: widget.onSolo),
          lock: _Toggle(r.locked ? TpGlyph.lock : TpGlyph.unlock, on: false, lit: r.locked || _h || sel, strong: r.locked, onTap: widget.onLock),
        ),
      ),
    );
  }
}

/// A row toggle: a bare glyph; [on] puts it on an accent chip, [strong] on a raised grey one.
class _Toggle extends StatelessWidget {
  const _Toggle(this.g, {required this.on, required this.lit, this.strong = false, required this.onTap});
  final TpGlyph g;
  final bool on, lit, strong;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: onTap,
    child: SizedBox(
      width: Shape.z(20),
      height: WtM.row - 1,
      child: Center(
        child: Container(
          width: Shape.z(17),
          height: Shape.z(17),
          alignment: Alignment.center,
          decoration: BoxDecoration(color: on ? WsT.accent : (strong ? Grey.g26 : null), borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
          child: CustomPaint(size: const Size.square(10), painter: TpGlyphPainter(g, on ? WsT.onAccent : (strong ? Grey.g95 : (lit ? Grey.g76 : Grey.g38)))),
        ),
      ),
    ),
  );
}
