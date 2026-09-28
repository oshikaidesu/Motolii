import 'dart:math' as math;
import 'package:flutter/widgets.dart';
import '../glyphs.dart';
import 'classify.dart';
import 'create_board.dart';
import 'common.dart';
import 'effects.dart';
import 'faces.dart';
import 'search.dart';
import 'seat.dart';
import 'shell.dart';
import 'things.dart';

enum Mk { text, rect, rounded, ellipse, star, polygon, line, arrow, path, blob, pen, pencil, spray, eraser, nul, camera, light, particles, stage, cube, sphere, torus, cylinder, cone, pyramid, plane }

class CreatePanel extends StatefulWidget {
  const CreatePanel({super.key, required this.catalog, this.user, this.scene, this.search, this.classify});
  final Catalog catalog;
  final UserViews? user;
  final EffectScene? scene;
  final SearchCapability? search;
  final ClassifyCapability? classify;
  @override
  State<CreatePanel> createState() => _CreatePanelState();
}

class _CreatePanelState extends State<CreatePanel> with WithDiscovery<CreatePanel> {
  @override
  SearchCapability? get injectedSearch => widget.search;
  @override
  ClassifyCapability? get injectedClassify => widget.classify;
  late final views = ThingViews(widget.catalog.registry, widget.catalog.things, widget.catalog.registry.panels['create']!, widget.user ?? UserViews());

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: discovery,
        builder: (_, __) {
          final q = ThingQuery.parse(search.query);
          final found = q.isEmpty ? views.scope : [for (final t in views.scope) if (q.matches(t, widget.catalog.registry)) t];
          final shown = [for (final t in found) if (views.contains(classify.selected, t)) t];
          final sections = <String, List<Thing>>{};
          for (final t in shown) {
            (sections[views.sectionOf(t)] ??= []).add(t);
          }
          return PanelShell(
            title: 'Create',
            icon: const GlyphBox(HG.plus, size: 22, color: Color(0xFFF2F2F4)),
            search: search,
            classify: classify,
            groups: views.groups(found),
            hint: 'Search create',
            count: _count(shown.length),
            classStrip: true,
            wide: (c, s) => shown.isEmpty ? emptyBody('No mark matches "${search.query}".') : SymbolMatrix(sections: sections, recent: search.active ? const [] : _recent(), width: s.width, scene: widget.scene),
            narrow: (c, s) => shown.isEmpty ? emptyBody('No mark matches.') : SymbolMatrix(sections: sections, recent: search.active ? const [] : _recent(), width: s.width, scene: widget.scene),
            strip: (c, s) => _strip(shown, s.height),
          );
        },
      );

  static String _count(int n) => n >= 1000 ? '${n ~/ 1000},${(n % 1000).toString().padLeft(3, '0')}' : '$n';

  /// The quick shelf: what was taken last (the user's Recent), a few, newest first.
  List<Thing> _recent() {
    final byId = {for (final t in views.scope) t.id: t};
    return [for (final id in (widget.user?.recent ?? const <String>[])) if (byId[id] case final t?) t].take(8).toList();
  }

  Widget _strip(List<Thing> shown, double h) => ListView.builder(
        scrollDirection: Axis.horizontal,
        physics: const ClampingScrollPhysics(),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 10),
        itemCount: shown.length,
        itemBuilder: (c, i) => Padding(padding: const EdgeInsets.only(right: 5), child: SizedBox(width: math.min(h - 20, 54), child: seated(c, shown[i], _Tile(shown[i], widget.scene, false)))),
      );
}

/// One key of the board: the face (at most 40 px) and its name under it. No ground; a housing shows under the pointer.
class _Tile extends StatefulWidget {
  const _Tile(this.thing, this.scene, this.caption);
  final Thing thing;
  final EffectScene? scene;
  final bool caption;
  @override
  State<_Tile> createState() => _TileState();
}

class _TileState extends State<_Tile> {
  bool _hover = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: DecoratedBox(
          decoration: BoxDecoration(color: _hover ? kRaisedHi : null, borderRadius: BorderRadius.circular(3)),
          child: LayoutBuilder(builder: (context, box) {
            final face = (box.maxWidth * .78).clamp(0.0, 40.0);
            return Column(mainAxisAlignment: MainAxisAlignment.center, children: [
              SizedBox(width: face, height: face, child: ThingFace(widget.thing, scene: widget.scene)),
              if (widget.caption)
                Padding(
                  padding: const EdgeInsets.only(top: 3),
                  child: Text(widget.thing.name, maxLines: 1, softWrap: false, overflow: TextOverflow.fade, textAlign: TextAlign.center, style: sans(9.5, c: const Color(0xFFD6D7DA), w: FontWeight.w500)),
                ),
            ]);
          }),
        ),
      );
}

class MarkPainter extends CustomPainter {
  MarkPainter(this.mk, this.color);
  final Mk mk;
  final Color color;
  @override
  void paint(Canvas cv, Size sz) {
    final s = sz.shortestSide;
    final c = Offset(sz.width / 2, sz.height / 2);
    Color tone(double t) => Color.lerp(color, t > 0 ? const Color(0xFFFFFFFF) : const Color(0xFF000000), t.abs())!;
    final fill = Paint()..color = color;
    Paint stroke([double w = 1.7, Color? k]) => Paint()..color = k ?? color..style = PaintingStyle.stroke..strokeWidth = w..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    Rect box(double f, [double dy = 0]) => Rect.fromCenter(center: c.translate(0, dy * s), width: s * f, height: s * f);
    Path poly(int n, double r, double rot) {
      final p = Path();
      for (var i = 0; i < n; i++) {
        final a = rot + i * 2 * math.pi / n;
        final q = c + Offset(math.cos(a), math.sin(a)) * r;
        i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
      }
      return p..close();
    }
    switch (mk) {
      case Mk.text:
        final p = stroke(s * .17);
        cv.drawLine(c + Offset(-s * .42, -s * .38), c + Offset(s * .42, -s * .38), p);
        cv.drawLine(c + Offset(0, -s * .38), c + Offset(0, s * .46), p);
      case Mk.rect:
        cv.drawRect(box(.84), Paint()..shader = LinearGradient(begin: Alignment.topLeft, end: Alignment.bottomRight, colors: [tone(.25), tone(-.1)]).createShader(box(.84)));
      case Mk.rounded:
        cv.drawRRect(RRect.fromRectAndRadius(box(.84), Radius.circular(s * .24)), fill);
      case Mk.ellipse:
        // a flat 2D oval, so it is never read as the shaded 3D sphere next to it
        cv.drawOval(Rect.fromCenter(center: c, width: s * .92, height: s * .7), fill);
      case Mk.star:
        final p = Path();
        for (var i = 0; i < 10; i++) {
          final r = i.isEven ? s * .48 : s * .21;
          final a = -math.pi / 2 + i * math.pi / 5;
          final q = c + Offset(math.cos(a), math.sin(a)) * r;
          i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
        }
        cv.drawPath(p..close(), fill);
      case Mk.polygon:
        cv.drawPath(poly(5, s * .46, -math.pi / 2), Paint()..color = tone(.05));
      case Mk.line:
        cv.drawLine(c + Offset(-s * .44, s * .4), c + Offset(s * .44, -s * .4), stroke(s * .07));
      case Mk.arrow:
        final a = c + Offset(-s * .42, s * .42), b = c + Offset(s * .4, -s * .4);
        cv.drawLine(a, b, stroke(s * .08));
        cv.drawLine(b, b + Offset(-s * .34, s * .02), stroke(s * .08));
        cv.drawLine(b, b + Offset(-s * .02, s * .34), stroke(s * .08));
      case Mk.path:
        final p = Path()..moveTo(c.dx - s * .44, c.dy + s * .3)..cubicTo(c.dx - s * .3, c.dy - s * .75, c.dx + s * .3, c.dy - s * .75, c.dx + s * .44, c.dy + s * .3);
        cv.drawPath(p, stroke(s * .08));
        cv.drawCircle(c + Offset(-s * .44, s * .3), s * .07, fill);
        cv.drawCircle(c + Offset(s * .44, s * .3), s * .07, fill);
      case Mk.blob:
        final p = Path()..moveTo(c.dx - s * .3, c.dy - s * .38)..cubicTo(c.dx, c.dy - s * .6, c.dx + s * .5, c.dy - s * .3, c.dx + s * .42, c.dy + s * .06)
          ..cubicTo(c.dx + s * .5, c.dy + s * .5, c.dx - s * .05, c.dy + s * .55, c.dx - s * .3, c.dy + s * .36)..cubicTo(c.dx - s * .6, c.dy + s * .2, c.dx - s * .55, c.dy - s * .2, c.dx - s * .3, c.dy - s * .38)..close();
        cv.drawPath(p, Paint()..shader = LinearGradient(begin: Alignment.topLeft, end: Alignment.bottomRight, colors: [tone(.35), color]).createShader(p.getBounds()));
      case Mk.pen:
        final p = Path()..moveTo(c.dx, c.dy - s * .46)..cubicTo(c.dx + s * .3, c.dy - s * .1, c.dx + s * .3, c.dy + s * .2, c.dx, c.dy + s * .3)..cubicTo(c.dx - s * .3, c.dy + s * .2, c.dx - s * .3, c.dy - s * .1, c.dx, c.dy - s * .46)..close();
        cv.drawPath(p, fill);
        cv.drawLine(c + Offset(0, s * .3), c + Offset(0, s * .47), stroke(s * .07, tone(.1)));
        cv.drawCircle(c.translate(0, -s * .08), s * .05, Paint()..color = kTile);
      case Mk.pencil:
        cv.save();
        cv.translate(c.dx, c.dy);
        cv.rotate(math.pi / 4);
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(0, -s * .06), width: s * .22, height: s * .7), Radius.circular(s * .03)), fill);
        cv.drawPath(Path()..moveTo(-s * .11, s * .29)..lineTo(s * .11, s * .29)..lineTo(0, s * .47)..close(), Paint()..color = tone(-.3));
        cv.restore();
      case Mk.spray:
        final rnd = math.Random(2);
        for (var i = 0; i < 26; i++) {
          final a = rnd.nextDouble() * math.pi * 2, r = math.sqrt(rnd.nextDouble()) * s * .44;
          cv.drawCircle(c + Offset(math.cos(a), math.sin(a)) * r, s * (.018 + rnd.nextDouble() * .04), fill);
        }
      case Mk.eraser:
        cv.save();
        cv.translate(c.dx, c.dy);
        cv.rotate(-math.pi / 5);
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: s * .78, height: s * .4), Radius.circular(s * .06)), fill);
        cv.drawRect(Rect.fromLTWH(-s * .39, -s * .2, s * .26, s * .4), Paint()..color = tone(-.35));
        cv.restore();
      case Mk.nul:
        final p = stroke(s * .07);
        cv.drawLine(c + Offset(-s * .42, 0), c + Offset(s * .42, 0), p);
        cv.drawLine(c + Offset(0, -s * .42), c + Offset(0, s * .42), p);
        cv.drawCircle(c, s * .13, p);
      case Mk.camera:
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: c.translate(0, s * .06), width: s * .9, height: s * .58), Radius.circular(s * .1)), fill);
        cv.drawRect(Rect.fromCenter(center: c.translate(0, -s * .28), width: s * .32, height: s * .14), fill);
        cv.drawCircle(c.translate(0, s * .06), s * .17, Paint()..color = kTile);
        cv.drawCircle(c.translate(0, s * .06), s * .17, stroke(s * .06));
      case Mk.light:
        cv.drawCircle(c, s * .2, fill);
        for (var i = 0; i < 8; i++) {
          final a = i * math.pi / 4;
          cv.drawLine(c + Offset(math.cos(a), math.sin(a)) * s * .32, c + Offset(math.cos(a), math.sin(a)) * s * .47, stroke(s * .07));
        }
      case Mk.particles:
        final rnd = math.Random(4);
        for (var i = 0; i < 10; i++) {
          final a = rnd.nextDouble() * math.pi * 2, r = math.sqrt(rnd.nextDouble()) * s * .42;
          cv.drawCircle(c + Offset(math.cos(a), math.sin(a)) * r, s * (0.03 + rnd.nextDouble() * .05), fill);
        }
      case Mk.stage:
        final r = box(.86);
        for (final m in (Path()..addRect(r)).computeMetrics()) {
          var d = 0.0;
          while (d < m.length) {
            cv.drawPath(m.extractPath(d, d + s * .09), stroke(s * .06));
            d += s * .17;
          }
        }
        cv.drawRect(box(.42), stroke(s * .06));
      case Mk.cube:
        final top = Path()..moveTo(c.dx, c.dy - s * .46)..lineTo(c.dx + s * .4, c.dy - s * .23)..lineTo(c.dx, c.dy)..lineTo(c.dx - s * .4, c.dy - s * .23)..close();
        final left = Path()..moveTo(c.dx - s * .4, c.dy - s * .23)..lineTo(c.dx, c.dy)..lineTo(c.dx, c.dy + s * .46)..lineTo(c.dx - s * .4, c.dy + s * .23)..close();
        final right = Path()..moveTo(c.dx + s * .4, c.dy - s * .23)..lineTo(c.dx, c.dy)..lineTo(c.dx, c.dy + s * .46)..lineTo(c.dx + s * .4, c.dy + s * .23)..close();
        cv.drawPath(top, Paint()..color = tone(.4));
        cv.drawPath(left, Paint()..color = color);
        cv.drawPath(right, Paint()..color = tone(-.3));
      case Mk.sphere:
        cv.drawCircle(c, s * .43, Paint()..shader = RadialGradient(center: const Alignment(-.4, -.45), radius: .95, colors: [tone(.55), color, tone(-.35)]).createShader(Rect.fromCircle(center: c, radius: s * .43)));
      case Mk.torus:
        cv.drawOval(Rect.fromCenter(center: c, width: s * .94, height: s * .56), stroke(s * .16, tone(-.05)));
        cv.drawOval(Rect.fromCenter(center: c.translate(0, -s * .02), width: s * .94, height: s * .56), stroke(s * .05, tone(.4)));
      case Mk.cylinder:
        final body = RRect.fromRectAndCorners(Rect.fromCenter(center: c.translate(0, s * .04), width: s * .6, height: s * .7), bottomLeft: Radius.elliptical(s * .3, s * .12), bottomRight: Radius.elliptical(s * .3, s * .12));
        cv.drawRRect(body, Paint()..shader = LinearGradient(colors: [tone(.25), color, tone(-.3)]).createShader(body.outerRect));
        cv.drawOval(Rect.fromCenter(center: c.translate(0, -s * .31), width: s * .6, height: s * .22), Paint()..color = tone(.45));
      case Mk.cone:
        final p = Path()..moveTo(c.dx, c.dy - s * .46)..lineTo(c.dx + s * .36, c.dy + s * .3)..quadraticBezierTo(c.dx, c.dy + s * .48, c.dx - s * .36, c.dy + s * .3)..close();
        cv.drawPath(p, Paint()..shader = LinearGradient(colors: [tone(.3), color, tone(-.3)]).createShader(p.getBounds()));
      case Mk.pyramid:
        final l = Path()..moveTo(c.dx, c.dy - s * .46)..lineTo(c.dx - s * .44, c.dy + s * .34)..lineTo(c.dx + s * .1, c.dy + s * .14)..close();
        final r = Path()..moveTo(c.dx, c.dy - s * .46)..lineTo(c.dx + s * .44, c.dy + s * .34)..lineTo(c.dx + s * .1, c.dy + s * .14)..close();
        cv.drawPath(l, Paint()..color = tone(.2));
        cv.drawPath(r, Paint()..color = tone(-.3));
      case Mk.plane:
        final p = Path()..moveTo(c.dx - s * .22, c.dy - s * .24)..lineTo(c.dx + s * .48, c.dy - s * .24)..lineTo(c.dx + s * .22, c.dy + s * .24)..lineTo(c.dx - s * .48, c.dy + s * .24)..close();
        cv.drawPath(p, Paint()..shader = LinearGradient(begin: Alignment.topRight, end: Alignment.bottomLeft, colors: [tone(.35), tone(-.15)]).createShader(p.getBounds()));
    }
  }
  @override
  bool shouldRepaint(MarkPainter o) => o.mk != mk || o.color != color;
}
