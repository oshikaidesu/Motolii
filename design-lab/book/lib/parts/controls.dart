// Controls. One rule: the value is the loudest thing in a control, the chrome around it is grey, the family colour marks only what it belongs to.
import 'package:flutter/widgets.dart';

import '../tokens.dart';

/// The three looks an open design question is shown in; the owner picks one in the real window (DESIGN.md section 9).
enum Look { concept, quiet, glow }

/// A bounded value as a well filled up to the value, left-anchored, in its family's colour; the number sits on the right.
class ValueWell extends StatelessWidget {
  const ValueWell({super.key, required this.label, required this.value, this.min = 0, this.max = 1, this.fam = Fam.scatter, this.decimals = 2, this.onChanged});
  final String label;
  final double value, min, max;
  final Fam fam;
  final int decimals;
  final ValueChanged<double>? onChanged;

  @override
  Widget build(BuildContext context) {
    final t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    return LayoutBuilder(builder: (context, box) {
      void set(Offset p) => onChanged?.call(min + (max - min) * (p.dx / box.maxWidth).clamp(0.0, 1.0));
      return GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanDown: (d) => set(d.localPosition),
        onPanUpdate: (d) => set(d.localPosition),
        child: SizedBox(
          height: 22,
          child: Stack(children: [
            Positioned.fill(child: DecoratedBox(decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)))),
            Positioned(left: 0, top: 0, bottom: 0, width: box.maxWidth * t, child: DecoratedBox(decoration: BoxDecoration(
              gradient: LinearGradient(colors: [fam.c.withValues(alpha: .28), fam.c.withValues(alpha: .55)]),
              borderRadius: BorderRadius.circular(4),
            ))),
            Positioned(left: 0, top: 4, bottom: 4, width: 2, child: DecoratedBox(decoration: BoxDecoration(color: fam.c, borderRadius: BorderRadius.circular(1)))),
            Positioned.fill(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 9), child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Text(label, style: T.label(N.g76)),
              const Spacer(),
              Text(value.toStringAsFixed(decimals), style: T.value(N.g95)),
            ]))),
          ]),
        ),
      );
    });
  }
}

class PillSwitch extends StatelessWidget {
  const PillSwitch({super.key, required this.on, this.fam = Fam.scatter, this.onChanged});
  final bool on;
  final Fam fam;
  final ValueChanged<bool>? onChanged;

  /// Fine, not heavy (critique, 4 passes): a 28x16 track; off = a grey thumb on g20; on = an accent TINT with a 1px accent edge and a light thumb, never a solid accent block.
  @override
  Widget build(BuildContext context) => GestureDetector(
        onTap: () => onChanged?.call(!on),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          width: 28,
          height: 16,
          padding: const EdgeInsets.all(2),
          alignment: on ? Alignment.centerRight : Alignment.centerLeft,
          decoration: BoxDecoration(
            color: on ? C.mode.withValues(alpha: .30) : N.g20,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: on ? C.mode.withValues(alpha: .85) : N.g26, width: 1),
          ),
          child: Container(width: 10, height: 10, decoration: BoxDecoration(color: on ? N.g95 : N.g56, shape: BoxShape.circle)),
        ),
      );
}

class Segmented extends StatelessWidget {
  const Segmented({super.key, required this.items, required this.index, this.onChanged, this.accent = C.mode, this.expand = false});
  final List<String> items;
  final int index;
  final Color accent;
  final bool expand;
  final ValueChanged<int>? onChanged;

  /// One selected style across the lab: a g20 pill with g95 text and a 1px accent underline (critique 2026-10-02: four selected styles were found).
  @override
  Widget build(BuildContext context) {
    Widget seg(int i) => GestureDetector(
          onTap: () => onChanged?.call(i),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 120),
            curve: Curves.easeOut,
            padding: EdgeInsets.symmetric(horizontal: expand ? 6 : 12),
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: i == index ? N.g20 : null,
              borderRadius: BorderRadius.circular(5),
              border: Border(bottom: BorderSide(color: i == index ? accent : const Color(0x00000000), width: 1)),
            ),
            child: FittedBox(fit: BoxFit.scaleDown, child: Text(items[i].toUpperCase(), maxLines: 1, softWrap: false, style: T.micro(i == index ? N.g95 : N.g63).copyWith(letterSpacing: .4))),
          ),
        );
    return Container(
      height: 26,
      padding: const EdgeInsets.all(2),
      decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(7)),
      child: Flex(direction: Axis.horizontal, mainAxisSize: expand ? MainAxisSize.max : MainAxisSize.min, children: [
        for (var i = 0; i < items.length; i++) expand ? Expanded(child: seg(i)) : seg(i),
      ]),
    );
  }
}

/// A small round or square button, the kind the transport uses.
class Tool extends StatelessWidget {
  const Tool({super.key, required this.child, this.fill, this.size = 28, this.round = false, this.on = false, this.onTap});
  final Widget child;
  final Color? fill;
  final double size;
  final bool round, on;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
        onTap: onTap,
        child: Container(
          width: size,
          height: size,
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: fill ?? (on ? N.g20 : N.g13),
            shape: round ? BoxShape.circle : BoxShape.rectangle,
            borderRadius: round ? null : BorderRadius.circular(6),
            border: Border.all(color: on ? N.g44 : N.g20, width: 1),
          ),
          child: child,
        ),
      );
}
