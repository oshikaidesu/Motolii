part of 'inspector_parts.dart';

/// Values that seldom change (seeds, point types, limits): folded at the card's foot, never removed.
/// contract: nothing the host can set is hidden from the user — what is rarely touched goes here instead.
class AdvancedFold extends StatelessWidget {
  const AdvancedFold(this.id, this.children, {super.key, this.summary});
  final String id;
  final List<Widget> children;

  /// What the folded row says on the right; defaults to how many rows it holds.
  final String? summary;

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed), adv = doc.b[id] ?? false;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 6),
          child: Hov(
            key: ValueKey('adv:$id'),
            onTap: () => doc.flag(id, !adv),
            builder: (_, h) => Container(
              height: 24,
              decoration: BoxDecoration(
                border: Border(top: BorderSide(color: ed.rule)),
              ),
              child: Row(
                children: [
                  FoldMark(open: adv, hot: h),
                  const SizedBox(width: 6),
                  Text('Advanced', style: ed.labStyle(h || adv ? Grey.g95 : Grey.g63)),
                  const Spacer(),
                  if (!adv) Text(summary ?? '${children.length} more', style: T.label(Grey.g56)),
                ],
              ),
            ),
          ),
        ),
        if (adv) ...[const SizedBox(height: 4), ...children],
      ],
    );
  }
}
