// Part of panel_inspector_a_space.dart (cut there by size, as it was).
part of 'panel_inspector_a.dart';

class _UnfoldRow extends StatelessWidget {
  const _UnfoldRow({required this.title, required this.row, required this.m, required this.open, required this.onToggle, required this.instrument, this.dock = false, this.hotArrow, this.focusArrow, this.pressArrow});
  final String title;
  final _R row;
  final _M m;
  final bool open, dock;
  final bool? hotArrow, focusArrow, pressArrow; // pinned states of the fold arrow (Parts)
  final VoidCallback onToggle;
  final Widget instrument;

  /// The fold arrow sits in the right column (the left edge stays one line): g63 at rest, g95 on hover, hover plate g15 / focus 1 px g76 edge / pressed g26 plate around it. Open turns it down; the open row is selected (g20 + tick).
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
        _prow(
          row, m,
          dock: dock,
          selected: open,
          trail: _Ctl(
            onTap: onToggle,
            hover: hotArrow,
            focus: focusArrow,
            pressed: pressArrow,
            builder: (_, h, f, p) => Container(
              width: _keyW,
              height: _cellH,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: p ? N.g26 : (h ? N.g15 : null), borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g76 : const Color(0x00000000))),
              child: Transform.rotate(angle: open ? math.pi / 2 : 0, child: Text('›', style: T.name(h || f || p || open ? N.g95 : N.g63))),
            ),
          ),
        ),
        AnimatedSize(
          duration: Mo.dur,
          curve: Mo.ease,
          alignment: Alignment.topCenter,
          child: open ? Padding(padding: const EdgeInsets.only(top: 4, bottom: 8), child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [instrument])) : const SizedBox(width: double.infinity),
        ),
      ]);
}
