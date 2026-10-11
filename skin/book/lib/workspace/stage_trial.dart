// The bar over the viewport while an effect is tried on: what is tried on which layer, what the try-on can't show by itself,
// and the two ways out (Add keeps it, Cancel drops it; Enter and Esc anywhere in the window do the same).
import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'fx_catalog.dart';
import 'ws.dart';

class StageTrialBar extends StatelessWidget {
  const StageTrialBar({super.key});
  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context), f = fxNamed(ws.trial), l = ws.layer;
    if (f == null || l == null) return const SizedBox.shrink();
    final note = f.trialNote;
    return Container(
      key: const ValueKey('ws-stage-trial'),
      padding: const EdgeInsets.fromLTRB(10, 6, 6, 6),
      decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.circular(WsT.radius + 2)),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                height: 16,
                padding: const EdgeInsets.symmetric(horizontal: 5),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: WsT.accent, borderRadius: BorderRadius.circular(WsT.chipRadius)),
                child: Text('TRYING', style: T.micro(WsT.onAccent).copyWith(fontWeight: FontWeight.w800, letterSpacing: .6, height: 1)),
              ),
              const SizedBox(width: 8),
              Text(f.name, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
              Text('  on  ', style: T.label(Grey.g56)),
              Container(
                width: 7,
                height: 7,
                decoration: BoxDecoration(color: wsTone(l.kind), borderRadius: BorderRadius.circular(1.5)),
              ),
              const SizedBox(width: 4),
              Text(l.name, style: T.name(Grey.g91)),
              const SizedBox(width: 14),
              _Btn('Cancel', 'Esc', onTap: ws.dropFx),
              const SizedBox(width: WsT.gap),
              _Btn('Add', 'Enter', accent: true, onTap: ws.keepFx),
            ],
          ),
          if (note != null)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(note, style: T.label(Grey.g76)),
            ),
        ],
      ),
    );
  }
}

class _Btn extends StatefulWidget {
  const _Btn(this.label, this.hint, {required this.onTap, this.accent = false});
  final String label, hint;
  final VoidCallback onTap;
  final bool accent;
  @override
  State<_Btn> createState() => _BtnState();
}

class _BtnState extends State<_Btn> {
  bool _h = false;
  @override
  Widget build(BuildContext context) {
    final a = widget.accent, ink = a ? WsT.onAccent : Grey.g91;
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: GestureDetector(
        key: ValueKey('ws-trial-${widget.label.toLowerCase()}'),
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          height: 22,
          padding: const EdgeInsets.symmetric(horizontal: 9),
          decoration: BoxDecoration(color: a ? (_h ? WsT.accentHot : WsT.accent) : (_h ? WsT.raised : WsT.well), borderRadius: BorderRadius.circular(11)),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(widget.label, style: T.name(ink).copyWith(fontWeight: FontWeight.w700)),
              const SizedBox(width: 5),
              Text(widget.hint, style: T.micro(a ? WsT.onAccent.withValues(alpha: .6) : Grey.g56)),
            ],
          ),
        ),
      ),
    );
  }
}
