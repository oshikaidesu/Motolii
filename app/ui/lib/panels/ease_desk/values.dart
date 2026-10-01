part of '../ease_desk.dart';

String _curveName(String kind) => kind.replaceAllMapped(
  RegExp(r'([a-z])([A-Z])'),
  (match) => '${match[1]} ${match[2]}',
);

const _easeIconBox = BoxConstraints.tightFor(
  width: EditorMetrics.control,
  height: EditorMetrics.control,
);

/// 静かな道具の粒。Ink の波紋を持たないので Reduce Motion で走る animation が残らない。
class _EaseIcon extends StatefulWidget {
  const _EaseIcon({
    required this.tooltip,
    required this.icon,
    required this.onPressed,
    this.color,
  });
  final String tooltip;
  final IconData icon;
  final VoidCallback onPressed;
  final Color? color;
  @override
  State<_EaseIcon> createState() => _EaseIconState();
}

class _EaseIconState extends State<_EaseIcon> {
  bool _over = false;
  @override
  Widget build(BuildContext context) => EditorTooltip(
    message: widget.tooltip,
    child: Semantics(
      label: widget.tooltip,
      button: true,
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        onEnter: (_) => setState(() => _over = true),
        onExit: (_) => setState(() => _over = false),
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: widget.onPressed,
          child: Container(
            constraints: _easeIconBox,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: _over ? EditorTheme.of(context).hover : EditorTheme.clear,
              borderRadius: BorderRadius.circular(EditorMetrics.s4),
            ),
            child: Icon(
              widget.icon,
              size: EditorMetrics.s16,
              color: _over
                  ? EditorTheme.of(context).ink
                  : widget.color ?? EditorTheme.of(context).muted,
            ),
          ),
        ),
      ),
    ),
  );
}

String _curveMeaning(String kind) => switch (kind) {
  'Hold' => 'Wait, then change in one jump.',
  'Linear' => 'Move at a constant pace.',
  'Bezier' => 'Shape the start and finish.',
  'Bounce' => 'Reach the end, then rebound.',
  'Elastic' => 'Pass the end and spring back.',
  'Cyclic' => 'Repeat a wave as time advances.',
  'Random' => 'Vary the pace irregularly.',
  'Steps' => 'Move through distinct levels.',
  'ElasticSteps' => 'Spring into each new level.',
  _ => 'Preview the change from start to finish.',
};

double easeValueAt(Map<String, dynamic> shape, double x) {
  if (shape['kind'] == 'Hold') return x < 1 ? 0 : 1;
  final samples = (shape['samples'] as List? ?? []).whereType<List>().toList();
  if (samples.length < 2) return x;
  for (var i = 0; i + 1 < samples.length; i++) {
    final a = samples[i], b = samples[i + 1];
    final ax = (a[0] as num).toDouble(), bx = (b[0] as num).toDouble();
    if (x <= bx) {
      final t = bx == ax ? 0.0 : ((x - ax) / (bx - ax)).clamp(0.0, 1.0);
      final ay = (a[1] as num).toDouble(), by = (b[1] as num).toDouble();
      return ay + (by - ay) * t;
    }
  }
  return (samples.last[1] as num).toDouble();
}
