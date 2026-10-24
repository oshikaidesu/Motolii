// Part of dose_parts.dart (cut there by size, as it was).
part of 'dose_parts.dart';

class _BadgePainter extends CustomPainter {
  const _BadgePainter(this.d, this.hue, this.shadow);
  final Dose d;
  final Color hue;
  final bool shadow;
  @override
  void paint(Canvas c, Size s) {
    final r = (Offset.zero & s).deflate(Dose.hair / 2), p = cutPath(r, d.cut), line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    if (shadow) c.drawPath(p.shift(const Offset(Dose.hair, Dose.hair)), line..color = Dose.shifted(hue, d.shadowHueShift));
    c.drawPath(p, Paint()..color = N.g13);
    c.drawPath(p, line..color = hue.withValues(alpha: d.selAlpha));
  }

  @override
  bool shouldRepaint(_BadgePainter o) => o.d != d || o.hue != hue || o.shadow != shadow;
}

class TileDef {
  const TileDef(this.name, this.category);
  final String name, category;
}

const tileDefs = [TileDef('Gaussian Blur', 'Blur'), TileDef('Glow', 'Light'), TileDef('Ripple', 'Warp')];
const tileBadgeLabels = ['New', 'GPU', '3D'];
List<Color> get tileBadgeHues => [C.mode, Fam.along.c, Fam.follow.c];

class BrowserTile extends StatelessWidget {
  const BrowserTile({super.key, required this.d, required this.def, required this.selected, required this.badges, this.hovered = false, this.onTap});
  final Dose d;
  final TileDef def;
  final bool selected, hovered;
  final int badges;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) {
    final n = math.min(badges, d.badgesMax);
    return GestureDetector(
      onTap: onTap,
      child: SelBox(
        d: d, selected: selected, hovered: hovered, hue: C.mode, w: Dose.tileW, h: Dose.tileH, tile: true,
        child: Padding(
          padding: const EdgeInsets.all(Dose.tilePad),
          child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, children: [
            SizedBox(height: Dose.kickerSlot, child: d.kickerItems ? kickerText(def.category, N.g63) : null),
            Container(
              height: Dose.tileThumbH,
              decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(Dose.radius)),
              child: const Center(child: Glyph(G.effect, size: 22, color: N.g63)),
            ),
            const SizedBox(height: Dose.gap),
            SizedBox(
              height: Dose.badgeH,
              width: Dose.tileW - 2 * Dose.tilePad,
              child: FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerLeft, child: Flex(direction: Axis.horizontal, children: [
                for (var i = 0; i < n; i++) ...[if (i > 0) const SizedBox(width: Dose.gap), DoseBadge(tileBadgeLabels[i], d: d, hue: tileBadgeHues[i], shadow: selected)],
              ])),
            ),
            const SizedBox(height: Dose.gap),
            SizedBox(
              height: Dose.tileNameCell,
              child: Align(alignment: Alignment.centerLeft, child: Text(def.name, maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: T.name(selected ? N.g95 : N.g76).copyWith(fontSize: d.selLabelPx(selected)))),
            ),
          ]),
        ),
      ),
    );
  }
}

/// Three tiles, one selected, with hover tracked per tile.
class TileRow extends StatefulWidget {
  const TileRow({super.key, required this.d, required this.badges});
  final Dose d;
  final int badges;
  @override
  State<TileRow> createState() => _TileRowState();
}

class _TileRowState extends State<TileRow> {
  int _sel = 1;
  int? _hover;
  @override
  Widget build(BuildContext context) => Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < tileDefs.length; i++) ...[
          if (i > 0) const SizedBox(width: Dose.gapL),
          MouseRegion(
            onEnter: (_) => setState(() => _hover = i),
            onExit: (_) => setState(() => _hover = null),
            child: BrowserTile(d: widget.d, def: tileDefs[i], selected: i == _sel, hovered: i == _hover, badges: widget.badges, onTap: () => setState(() => _sel = i)),
          ),
        ],
      ]);
}

// ---------------------------------------------------------------- 5 panel header strip

class _HeaderPainter extends CustomPainter {
  const _HeaderPainter(this.d);
  final Dose d;
  @override
  void paint(Canvas c, Size s) {
    final cut = d.headerCut ? d.cut : 0.0;
    final body = Path()..moveTo(0, 0)..lineTo(s.width - cut, 0)..lineTo(s.width, cut)..lineTo(s.width, s.height)..lineTo(0, s.height)..close();
    c.drawPath(body, Paint()..color = N.g13);
    final line = Paint()..style = PaintingStyle.stroke..strokeWidth = Dose.hair;
    if (cut > 0) c.drawLine(Offset(s.width - cut, Dose.hair / 2), Offset(s.width, cut + Dose.hair / 2), line..color = N.g100.withValues(alpha: Dose.leadAlpha));
    final y = s.height - Dose.hair / 2;
    c.drawLine(Offset(0, y), Offset(s.width, y), line..color = d.leadLine ? N.g100.withValues(alpha: Dose.leadAlpha) : N.rowLine);
  }

  @override
  bool shouldRepaint(_HeaderPainter o) => o.d != d;
}

/// Halftone on a 45 degree lattice: pitch and alpha from the dose; a strip of its own, never under text.
class _HalftonePainter extends CustomPainter {
  const _HalftonePainter(this.pitch, this.alpha);
  final double pitch, alpha;
  @override
  void paint(Canvas c, Size s) {
    c.clipRect(Offset.zero & s);
    final p = Paint()..color = N.g100.withValues(alpha: alpha);
    var row = 0;
    for (var y = pitch / 2; y < s.height + pitch; y += pitch / 2, row++) {
      for (var x = row.isOdd ? pitch / 2 : 0.0; x < s.width + pitch; x += pitch) {
        c.drawCircle(Offset(x, y), pitch * .22, p);
      }
    }
  }

  @override
  bool shouldRepaint(_HalftonePainter o) => o.pitch != pitch || o.alpha != alpha;
}

class PanelHeader extends StatelessWidget {
  const PanelHeader({super.key, required this.d, this.kicker = 'Inspector', this.title = 'Relations', this.number = 1});
  final Dose d;
  final String kicker, title;
  final int number;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: Dose.headerW,
        child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
          CustomPaint(
            painter: _HeaderPainter(d),
            child: SizedBox(
              height: Dose.headerH,
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: Dose.gapL + Dose.gap),
                child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                  SizedBox(
                    height: Dose.kickerSlot,
                    child: d.kickerHeader ? Flex(direction: Axis.horizontal, children: [
                      if (d.headerNumber) ...[Text(number.toString().padLeft(2, '0'), style: T.value(N.g56).copyWith(fontSize: Dose.kickerPx)), const SizedBox(width: Dose.gapL)],
                      kickerText(kicker, N.g76),
                    ]) : null,
                  ),
                  Text(title, style: T.title()),
                ]),
              ),
            ),
          ),
          if (d.halftone) SizedBox(width: Dose.headerW, height: Dose.stripH, child: CustomPaint(painter: _HalftonePainter(d.halftonePitch, d.halftoneAlpha))) else const SizedBox(height: Dose.stripH),
        ]),
      );
}
