// Part of ws.dart (cut there by size, as it was).
part of 'ws.dart';

/// A seat's header: title on the left, [trailing] on the right, a hairline under it.
class WsHeader extends StatelessWidget {
  const WsHeader(this.title, {super.key, this.trailing = const [], this.leading});
  final String title;
  final List<Widget> trailing;
  final Widget? leading;
  @override
  Widget build(BuildContext context) => Container(
    height: WsT.header,
    padding: EdgeInsets.symmetric(horizontal: WsT.inset),
    decoration: BoxDecoration(
      color: WsT.body,
      border: Border(bottom: BorderSide(color: WsT.line)),
    ),
    child: Row(
      children: [
        if (leading case final l?) ...[l, SizedBox(width: Shape.s(6))],
        if (title.isNotEmpty) Text(title, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
        const Spacer(),
        ...trailing,
      ],
    ),
  );
}

/// A seat: a square tile of [WsT.body] that clips what it holds.
class WsSeat extends StatelessWidget {
  const WsSeat({super.key, required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) => ClipRect(
    child: ColoredBox(color: WsT.body, child: child),
  );
}

/// How many cells of at least [cell] px fit across [width] with [gap] between them, kept within [min]..[max].
/// Every grid in the window reads this, so a pane made wider gains columns instead of fatter tiles.
int wsCols(double width, double cell, {double gap = WsT.gutter, int min = 1, int max = 8}) => ((width + gap) / (cell + gap)).floor().clamp(min, max);

/// The width of one of [cols] cells across [width].
double wsCell(double width, int cols, {double gap = WsT.gutter}) => (width - gap * (cols - 1)) / cols;
