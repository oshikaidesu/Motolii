// The numbers.

import 'package:flutter/widgets.dart';
import 'package:lab_book/tokens.dart';
import 'package:lab_book/workspace/desk_ease.dart';
import 'package:lab_book/workspace/desk_kit.dart';
import 'package:lab_book/workspace/ws.dart';

/// The numbers: AE-style influence and speed for each end, then the four cubic-bezier numbers you can scrub.
class EaseNumbers extends StatelessWidget {
  const EaseNumbers({super.key, required this.ease, required this.span, required this.onChanged});
  final Ease ease;
  final EaseSpan? span;
  final ValueChanged<Ease> onChanged;
  @override
  Widget build(BuildContext context) {
    final bez = ease.kind == EaseKind.bezier && !easeOther(ease), e = ease;   // (a way defined in the document: no handles of a bezier to scrub)
    String slope(Offset h, Offset k) => (h.dx - k.dx).abs() < 1e-3 ? '∞' : ((h.dy - k.dy) / (h.dx - k.dx)).abs().toStringAsFixed(2);
    Widget end(String title, double influence, String speed, {String? caption}) => Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        DeskCaption(title, trailing: caption),
        const SizedBox(height: 4),
        Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            DeskReadout(value: bez ? (influence * 100).toStringAsFixed(0) : '—', unit: bez ? '%' : '', ink: bez ? Grey.g95 : Grey.g56),
            const Spacer(),
            Text(bez ? '$speed×' : '—', style: T.value(Grey.g76).copyWith(fontSize: 11)),
          ],
        ),
      ],
    );
    String f(double v) => v.toStringAsFixed(2);
    Widget pair(DeskScrub a, DeskScrub b) => Row(
      children: [
        Expanded(child: a),
        const SizedBox(width: 2),
        Expanded(child: b),
      ],
    );
    // (a kind with no seat here: each of its own numbers to scrub, as far as a hand takes it; said back as the kind and all of them)
    final words = easeOther(e) ? e.name!.split(' ') : const <String>[], inside = WsScope.of(context).inside;
    final own = words.isEmpty || inside == null ? const <(String, double, double)>[] : inside.wayNumbers(words.first);
    if (own.isNotEmpty) {
      final now = words.length == own.length + 1 ? [for (final w in words.skip(1)) double.tryParse(w) ?? 0] : [for (final n in own) (n.$2 + n.$3) / 2];
      return ColoredBox(
        color: WsT.card,
        child: SingleChildScrollView(
          padding: const EdgeInsets.fromLTRB(WsT.inset, 7, WsT.inset, WsT.inset),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              for (final (j, (name, lo0, hi0)) in own.indexed)
                if ((now[j] < lo0 ? now[j] : lo0, now[j] > hi0 ? now[j] : hi0) case (final lo, final hi)) ...[   // (a number said beyond a hand's reach stays as it is until moved)
                DeskScrub(
                  key: ValueKey('ease.number.$name'),
                  label: name.replaceAll('_', ' '),
                  value: now[j],
                  min: lo,
                  max: hi,
                  perPx: (hi - lo) / 200,
                  pad: 5,
                  format: (v) => hi - lo > 50 ? v.round().toString() : v.toStringAsFixed(2),
                  onChanged: (v) => onChanged(Ease(e.kind, e.p1, e.p2, [words.first, for (final (k, x) in now.indexed) k == j ? (hi - lo > 50 ? v.roundToDouble() : v) : x].join(' '))),
                ),
                const SizedBox(height: 2),
              ],
            ],
          ),
        ),
      );
    }
    return ColoredBox(
      color: WsT.card,
      child: LayoutBuilder(
        builder: (context, box) => SingleChildScrollView(
          physics: const NeverScrollableScrollPhysics(),
          padding: const EdgeInsets.fromLTRB(WsT.inset, 7, WsT.inset, WsT.inset),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              end('Out', e.p1.dx, slope(e.p1, Offset.zero), caption: 'infl · speed'),
              const SizedBox(height: 10),
              end('In', 1 - e.p2.dx, slope(e.p2, const Offset(1, 1))),
              if (box.maxHeight >= 170) ...[
                const SizedBox(height: 10),
                const DeskCaption('Bezier'),
                const SizedBox(height: 4),
                pair(
                  DeskScrub(
                    label: 'x1',
                    value: e.p1.dx,
                    min: 0,
                    max: 1,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(Offset(v, e.p1.dy), e.p2)),
                  ),
                  DeskScrub(
                    label: 'y1',
                    value: e.p1.dy,
                    min: -.4,
                    max: 1.6,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(Offset(e.p1.dx, v), e.p2)),
                  ),
                ),
                const SizedBox(height: 2),
                pair(
                  DeskScrub(
                    label: 'x2',
                    value: e.p2.dx,
                    min: 0,
                    max: 1,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(e.p1, Offset(v, e.p2.dy))),
                  ),
                  DeskScrub(
                    label: 'y2',
                    value: e.p2.dy,
                    min: -.4,
                    max: 1.6,
                    perPx: .01,
                    pad: 5,
                    format: f,
                    enabled: bez,
                    onChanged: (v) => onChanged(e.withHandles(e.p1, Offset(e.p2.dx, v))),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
