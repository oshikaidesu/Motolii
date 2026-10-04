part of 'inspector_parts.dart';

/// The panels that own lists (fonts, colours, eases). The Inspector shows what is set and sends you there; it never holds the list.
enum LinkTo {
  fonts('Browser', 'Fonts'),
  colors('Browser', 'Colors'),
  ease('Ease', 'Ease');

  const LinkTo(this.panel, this.shelf);
  final String panel, shelf;
  String get route => panel == shelf ? panel : '$panel › $shelf';
}

/// One cell: what is set now, and where it is chosen.
/// contract: a tap only asks for the panel (`route` in Doc, the lab's stand-in); the choice made there comes back as the value.
class PanelLink extends StatelessWidget {
  const PanelLink({super.key, required this.to, required this.value, this.lead, this.label});
  final LinkTo to;
  final String value;
  final String? label;
  final Widget? lead;

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), off = x.cfg.locked;
    return Hov(
      key: ValueKey('link:${to.name}:${label ?? ''}'),
      cursor: off ? SystemMouseCursors.basic : SystemMouseCursors.click,
      onTap: off ? null : () => x.doc.str('route', to.route),
      builder: (_, h) => Container(
        height: Pop.cell,
        padding: const EdgeInsets.only(left: 8, right: 6),
        decoration: BoxDecoration(color: h && !off ? Grey.g20 : Pop.well, borderRadius: BorderRadius.circular(6)),
        child: Row(
          children: [
            if (label case final l?) ...[Text(l, style: T.label(Grey.g63)), const SizedBox(width: 8)],
            if (lead case final w?) ...[w, const SizedBox(width: 6)],
            Expanded(
              child: Text(value, style: T.value(Grey.g95), maxLines: 1, overflow: TextOverflow.ellipsis),
            ),
            if (to.shelf != label) ...[Text(to.shelf, style: T.label(h && !off ? Grey.g95 : Grey.g56)), const SizedBox(width: 2)],
            CustomPaint(size: const Size(8, 8), painter: _LinkArrowPaint(h && !off ? Grey.g95 : Grey.g56)),
          ],
        ),
      ),
    );
  }
}

/// The value line of a block tile that is chosen elsewhere: the value, then where it is chosen.
class LinkLine extends StatelessWidget {
  const LinkLine({super.key, required this.to, required this.value, this.id = ''});
  final LinkTo to;
  final String value, id;
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), off = x.cfg.locked;
    return Hov(
      key: ValueKey('link:${to.name}:$id'),
      cursor: off ? SystemMouseCursors.basic : SystemMouseCursors.click,
      onTap: off ? null : () => x.doc.str('route', to.route),
      builder: (_, h) => SizedBox(
        height: 18,
        child: Row(
          children: [
            Expanded(
              child: Text(value, maxLines: 1, overflow: TextOverflow.ellipsis, style: _blockValue(off ? Grey.g56 : Grey.g95)),
            ),
            Text(to.shelf, style: T.micro(h && !off ? Grey.g95 : Grey.g56)),
            const SizedBox(width: 2),
            CustomPaint(size: const Size(7, 7), painter: _LinkArrowPaint(h && !off ? Grey.g95 : Grey.g56)),
          ],
        ),
      ),
    );
  }
}

/// A small swatch for a colour link.
class LinkSwatch extends StatelessWidget {
  const LinkSwatch(this.hex, {super.key});
  final String hex;
  @override
  Widget build(BuildContext context) => Container(
    width: 14,
    height: 14,
    decoration: BoxDecoration(
      color: _hex(hex) ?? Grey.g95,
      borderRadius: BorderRadius.circular(3),
      border: Border.all(color: Grey.g38),
    ),
  );
}

/// The arrow that says "opens elsewhere".
class _LinkArrowPaint extends CustomPainter {
  const _LinkArrowPaint(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    final p = Paint()
      ..color = c
      ..strokeWidth = 1.3
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    canvas
      ..drawLine(Offset(1, s.height - 1), Offset(s.width - 1, 1), p)
      ..drawPath(
        Path()
          ..moveTo(s.width * .35, 1)
          ..lineTo(s.width - 1, 1)
          ..lineTo(s.width - 1, s.height * .65),
        p,
      );
  }

  @override
  bool shouldRepaint(_LinkArrowPaint o) => o.c != c;
}
