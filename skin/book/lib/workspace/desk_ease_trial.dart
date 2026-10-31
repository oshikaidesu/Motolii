// A way tried with no stretch to go by: it runs once when it changes, and over and over while the hand is on the graph, as
// easings.net's curves do under the hand (the maker, 10/31: "何がどう動くのかっていう試しができない"). It stops of itself: a panel
// left alone, as it opens with nothing picked, spends nothing.
import 'package:flutter/widgets.dart';

class EaseTrial extends StatefulWidget {
  const EaseTrial({super.key, required this.way, required this.builder});
  final String way;
  final Widget Function(double? at) builder;   // (0 to 1 along the way; none: not running)
  @override
  State<EaseTrial> createState() => _EaseTrialState();
}

class _EaseTrialState extends State<EaseTrial> with SingleTickerProviderStateMixin {
  late final _run = AnimationController(vsync: this, duration: const Duration(milliseconds: 1800))..addStatusListener(_ended);
  bool _over = false;

  void _ended(AnimationStatus s) {
    if (s == AnimationStatus.completed && _over) _run.forward(from: 0);
  }

  void _start() => _run.forward(from: 0);

  @override
  void didUpdateWidget(EaseTrial old) {
    super.didUpdateWidget(old);
    if (old.way != widget.way) _start();
  }

  @override
  void dispose() {
    _run.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) {
      _over = true;
      if (!_run.isAnimating) _start();
    },
    onExit: (_) => _over = false,
    child: AnimatedBuilder(
      animation: _run,
      builder: (_, _) => widget.builder(_run.isAnimating ? (_run.value * 1.2).clamp(0.0, 1.0) : null),   // (a beat at the end, as it has arrived)
    ),
  );
}
