part of 'inspector_parts.dart';

// ---- 3. small things ------------------------------------------------------------------------------------------------
/// A contract written as a 10px caption under a control.
class Cap extends StatelessWidget {
  const Cap(this.text, {super.key, this.color});
  final String text;
  final Color? color;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: 6),
    child: Text(text, style: T.label(color ?? Grey.g56).copyWith(height: 1.3)),
  );
}

class Hov extends StatefulWidget {
  const Hov({super.key, required this.builder, this.cursor = SystemMouseCursors.click, this.onTap});
  final Widget Function(BuildContext, bool) builder;
  final MouseCursor cursor;
  final VoidCallback? onTap;
  @override
  State<Hov> createState() => _HovState();
}

class _HovState extends State<Hov> {
  bool h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: widget.cursor,
    onEnter: (_) => setState(() => h = true),
    onExit: (_) => setState(() => h = false),
    child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(context, h)),
  );
}

String trimZeros(String s) => s.contains('.') ? s.replaceFirst(RegExp(r'\.?0+$'), '') : s;

/// A value as a folded card shows it: short, trailing zeros dropped, — when mixed.
String brief(double? v, [int dec = 1]) => v == null ? '—' : trimZeros(v.toStringAsFixed(dec));

String fmt(double v, int dec, {bool tight = false}) {
  var d = tight ? 0 : dec;
  var s = v.toStringAsFixed(d);
  while (v != 0 && double.parse(s) == 0 && d < 4) {
    d++;
    s = v.toStringAsFixed(d);
  }
  return s == '-0' ? '0' : s;
}

/// The key mark. Three states differ by SHAPE, not by tone alone (D1, do-not-repeat 11): off = hollow diamond; animated = diamond with its link line; keyed here = filled playhead diamond.
class KeyMark extends StatelessWidget {
  const KeyMark({super.key, required this.state, this.onTap, this.enabled = true});
  final KeyS state;
  final VoidCallback? onTap;
  final bool enabled;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: enabled ? onTap : null,
    builder: (_, h) => SizedBox(
      width: _keyW,
      height: _keyW,
      child: CustomPaint(painter: _KeyPaint(state, h && enabled, enabled)),
    ),
  );
}

class _KeyPaint extends CustomPainter {
  const _KeyPaint(this.s, this.hover, this.enabled);
  final KeyS s;
  final bool hover, enabled;
  @override
  void paint(Canvas c, Size z) {
    final o = Offset(z.width / 2, z.height / 2), r = s == KeyS.at ? 4.6 : 3.8;
    final dia = Path()
      ..moveTo(o.dx, o.dy - r)
      ..lineTo(o.dx + r, o.dy)
      ..lineTo(o.dx, o.dy + r)
      ..lineTo(o.dx - r, o.dy)
      ..close();
    final a = enabled ? 1.0 : .4;
    Paint line(Color col, double w) => Paint()
      ..color = col.withValues(alpha: col.a * a)
      ..style = PaintingStyle.stroke
      ..strokeWidth = w;
    switch (s) {
      case KeyS.off:
        c.drawPath(dia, line(hover ? Role.of(Grey.g76, Role.still) : Role.of(Grey.g44, Role.still), 1));
      case KeyS.anim:
        c.drawPath(dia, line(hover ? Role.of(Grey.g95, Role.key) : Role.of(Grey.g91, Role.key), 1));
        c.drawLine(Offset(o.dx - r - 4, o.dy), Offset(o.dx - r, o.dy), line(Role.of(Grey.g91, Role.key), 1));
        c.drawLine(Offset(o.dx + r, o.dy), Offset(o.dx + r + 4, o.dy), line(Role.of(Grey.g91, Role.key), 1));
      case KeyS.at:
        c.drawPath(dia, Paint()..color = C.playhead.withValues(alpha: a));
        c.drawPath(dia, line(Grey.g100, 1.5));
    }
  }

  @override
  bool shouldRepaint(_KeyPaint o) => o.s != s || o.hover != hover || o.enabled != enabled;
}

/// The reset mark: appears only when the value differs (C7, E4). [open] the arrow is a text character, not a new glyph.
class ResetMark extends StatelessWidget {
  const ResetMark({super.key, required this.show, this.onTap});
  final bool show;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => SizedBox(
    width: _resetW,
    child: show
        ? Hov(
            onTap: onTap,
            builder: (_, h) => Center(child: Text('↺', style: T.label(h ? Grey.g95 : Grey.g56).copyWith(fontSize: 12))),
          )
        : null,
  );
}

/// The link between two fields: closed chain = linked, broken chain = free. Shape carries the state, not tone alone.
class LinkMark extends StatelessWidget {
  const LinkMark({super.key, required this.on, this.onTap, this.enabled = true});
  final bool on, enabled;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    toggled: on,
    label: 'Link',
    child: Hov(
      onTap: enabled ? onTap : null,
      builder: (_, h) => SizedBox(
        width: 18,
        height: _cellH,
        child: CustomPaint(painter: _LinkPaint(on, h && enabled, enabled)),
      ),
    ),
  );
}

class _LinkPaint extends CustomPainter {
  const _LinkPaint(this.on, this.hover, this.enabled);
  final bool on, hover, enabled;
  @override
  void paint(Canvas c, Size s) {
    final o = s.center(Offset.zero);
    final ink = !enabled ? Grey.g38 : (on ? (hover ? Grey.g100 : Grey.g91) : (hover ? Grey.g76 : Grey.g44));
    final st = Paint()
      ..color = ink
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round;
    final gap = on ? 0.0 : 1.5;
    const link = Size(7, 4.5);
    for (final dir in const [-1.0, 1.0]) {
      final r = Rect.fromCenter(center: o + Offset(dir * (link.width / 2 - 1.2 + gap), 0), width: link.width, height: link.height);
      c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(2.25)), st);
    }
    if (!on) {
      c.drawLine(o + const Offset(-1, -4.5), o + const Offset(0, -3), st);
      c.drawLine(o + const Offset(1, 4.5), o + const Offset(0, 3), st);
    }
  }

  @override
  bool shouldRepaint(_LinkPaint o) => o.on != on || o.hover != hover || o.enabled != enabled;
}

/// The word-and-switch for On/Off (G2): the state is a word, never colour alone.
class OnOff extends StatelessWidget {
  const OnOff({super.key, required this.on, required this.onChanged, this.enabled = true});
  final bool on, enabled;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) {
    final live = enabled && !RowOff.offOf(context) && !Ctx.of(context).cfg.locked;
    return IgnorePointer(
      ignoring: !live,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(
            width: 20,
            child: Text(on ? 'On' : 'Off', textAlign: TextAlign.right, style: T.label(live && on ? Grey.g95 : Grey.g56)),
          ),
          const SizedBox(width: 8),
          Opacity(
            opacity: live ? 1 : .5,
            child: Pop.on(context) ? _PopSwitch(on: on, onChanged: onChanged) : PillSwitch(on: on, onChanged: onChanged),
          ),
        ],
      ),
    );
  }
}

class _PopSwitch extends StatelessWidget {
  const _PopSwitch({required this.on, required this.onChanged});
  final bool on;
  final ValueChanged<bool> onChanged;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: () => onChanged(!on),
    child: AnimatedContainer(
      duration: const Duration(milliseconds: 120),
      curve: Curves.easeOut,
      width: 28,
      height: 16,
      padding: const EdgeInsets.all(2),
      alignment: on ? Alignment.centerRight : Alignment.centerLeft,
      decoration: BoxDecoration(color: on ? Pop.accent : Grey.g26, borderRadius: BorderRadius.circular(8)),
      child: Container(
        width: 12,
        height: 12,
        decoration: BoxDecoration(color: on ? Pop.onAccent : Grey.g63, shape: BoxShape.circle),
      ),
    ),
  );
}

/// A choice chip in the lab's choice-control style: a g20 pill, g95 text, a 1px accent underline.
class Chip extends StatelessWidget {
  const Chip(this.text, {super.key, required this.on, this.onTap, this.enabled = true});
  final String text;
  final bool on, enabled;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => Hov(
    onTap: enabled ? onTap : null,
    builder: (_, h) => Opacity(
      opacity: enabled ? 1 : .4,
      child: Container(
        height: 20,
        padding: const EdgeInsets.symmetric(horizontal: 7),
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: on ? Grey.g20 : (h ? Grey.g15 : null),
          borderRadius: BorderRadius.circular(5),
          border: Border(bottom: BorderSide(color: on ? Role.selected : const Color(0x00000000))),
        ),
        child: Text(text, style: T.label(on ? Grey.g95 : Grey.g63)),
      ),
    ),
  );
}
