// Part of panel_inspector_b_parts.dart (cut there by size, as it was).
part of 'panel_inspector_b.dart';

/// A bare label + control row for values that are not [_P]s.
class _VRow extends StatelessWidget {
  const _VRow(this.label, this.control, {this.tick, this.labelColor = N.g63});
  final String label;
  final Widget control;
  final Color? tick;
  final Color labelColor;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        final narrow = box.maxWidth < 220;
        final l = _Mid(label, T.label(labelColor).copyWith(fontSize: 11, height: 1.2));
        return _Gut(
          tick: tick,
          child: Container(
          constraints: const BoxConstraints(minHeight: 28),
          padding: const EdgeInsets.only(left: 12, right: 12, top: 4, bottom: 4),
          child: Row(children: [
            Expanded(
              child: narrow
                  ? Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [l, const SizedBox(height: 4), control])
                  : Row(children: [Expanded(flex: 5, child: l), const SizedBox(width: 8), Expanded(flex: 6, child: control)]),
            ),
          ]),
        ));
      });
}

/// A thin hairline between blocks.
class _Hair extends StatelessWidget {
  const _Hair();
  @override
  Widget build(BuildContext context) => const SizedBox(height: 1, child: ColoredBox(color: N.g07));
}

/// Space between blocks: 4, 8, 12 only.
Widget _gap([double h = 8]) => SizedBox(height: h);

/// A label that keeps both ends: when it does not fit, the middle becomes an ellipsis (55 % head, tail kept) so "Offset Turbulence X" and "... Y" stay apart. A query is highlighted when it survives the cut.
class _Mid extends StatelessWidget {
  const _Mid(this.text, this.style, {this.query = ''});
  final String text, query;
  final TextStyle style;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        final tp = TextPainter(textDirection: TextDirection.ltr, maxLines: 1);
        bool fits(String s) {
          tp.text = TextSpan(text: s, style: style);
          tp.layout();
          return tp.width <= box.maxWidth;
        }

        var t = text;
        if (box.maxWidth.isFinite && !fits(text)) {
          String cut(int k) {
            final head = (k * .55).ceil(), tail = k - head;
            return '${text.substring(0, head).trimRight()}…${text.substring(text.length - tail).trimLeft()}';
          }

          var lo = 2, hi = text.length - 1;
          while (lo < hi) {
            final mid = (lo + hi + 1) ~/ 2;
            if (fits(cut(mid))) {
              lo = mid;
            } else {
              hi = mid - 1;
            }
          }
          t = cut(lo);
        }
        tp.dispose();
        return Text.rich(_hl(t, query, style), maxLines: 1, softWrap: false, overflow: TextOverflow.clip);
      });
}
