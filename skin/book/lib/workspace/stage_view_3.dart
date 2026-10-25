// Part of stage_view.dart (cut there by size, as it was).
part of 'stage_view.dart';

/// Joined: what is picked together and the relation picked, over the stage. Picked together: a ring each, and along the
/// top of the stage what they may do together. A relation picked: its shape (a ring round its middle, a row's line, how
/// far a push reaches), handles that carry its middle and size, a ring on each thing it moves with a small × that takes
/// that one out, a crown on its leader (another's crown makes that one the leader), and along the top its name and
/// Release. A thing picked: the relations it is in, beside it, as outlined tags.
class _RelationMarks extends StatelessWidget {
  const _RelationMarks({required this.scene, required this.v, required this.lasso});
  final ArtScene scene;
  final _View v;
  final List<Offset>? lasso;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: Listenable.merge([WsScope.read(context).on[WsAspect.document], WsScope.read(context).on[WsAspect.picked], WsScope.read(context).seen]),   // (where things are in the picture shown)
    builder: (context, _) {
      final ws = WsScope.read(context), m = ws.inside;
      Offset? on(String id) => scene.box(id) == null ? null : v.toScreen(WsArt.toView(scene, scene.box(id)!.c));
      final picked = [for (final id in ws.many) if (on(id) case final p?) p];
      final rel = int.tryParse(ws.selected ?? ''), kind = rel == null || m == null ? '' : m.word(rel, 'file');
      final isRelation = rel != null && m != null && m.word(rel, 'kind') == 'behaviour' && wsRelations.contains(kind);
      final members = [for (final id in wsMembersOf(ws)) if (on(id) case final p?) (id, p)];
      final now = m?.now ?? 0;   // (the moment of the picture shown)
      // (a place of the stage in its own units, on the screen; and back)
      Offset screen(double x, double y) => v.toScreen(WsArt.toView(scene, Offset(WsArt.comp.width / 2 + x * m!.point, WsArt.comp.height / 2 - y * m.point)));
      (double, double) units(Offset p) {
        final c = v.toComp(p);
        return ((c.dx - WsArt.comp.width / 2) / m!.point, (WsArt.comp.height / 2 - c.dy) / m.point);
      }

      final shape = <_Shape>[];
      final handles = <Widget>[];
      String? leader;
      if (isRelation) {
        double val(String n, [double or = 0]) => m.value(rel, n, now) ?? or;
        final off = val('visible', 1) <= .5;
        leader = val('leader') > 0 ? '${val('leader').round()}' : null;
        final lead = kind == 'around' || kind == 'row' ? int.tryParse(leader ?? '') : null;   // (a ring's or a row's middle is its leader's place and its centre a step from it)
        final lx = lead == null ? 0.0 : m.shows(lead, 'x', now)?.$1 ?? 0, ly = lead == null ? 0.0 : m.shows(lead, 'y', now)?.$1 ?? 0;
        final cx = val('centre_x') + lx, cy = val('centre_y') + ly, mid = screen(cx, cy), px = m.point * v.s;
        Widget handle(String key, Offset at, void Function((double, double) u) carry) => Positioned(   // (the middle a cross, a size an arrow out)
          left: at.dx - 7,
          top: at.dy - 7,
          child: GestureDetector(
            supportedDevices: wsPressing,
            key: ValueKey('ws-relation-$key'),
            onPanStart: (_) => m.take(rel),
            onPanUpdate: (d) {
              final box = context.findRenderObject() as RenderBox?;
              if (box != null) carry(units(box.globalToLocal(d.globalPosition)));
            },
            onPanEnd: (_) => ws.after(m.later('let_go', number: 1), (_, _) => ws.reread()),
            child: Container(width: 16, height: 16, decoration: BoxDecoration(color: WsT.onAccent, shape: BoxShape.circle, border: Border.all(color: WsT.accent, width: 2)), child: CustomPaint(painter: _GlyphPaint(key == 'centre' ? 'cross' : 'arrow', WsT.accent))),
          ),
        );
        if (kind == 'around') {
          final r = val('radius', .5), turn = val('offset') + .5 / math.max(1, members.length), tilt = val('tilt') * math.pi / 180;
          if (tilt.abs() < 1e-6 || members.length < 3) {
            shape.add(_Shape.ring(mid, r * px, off: off));
          } else {   // (tipped back, its things in depth: drawn through where the camera shows them, in its order, round)
            for (var k = 0; k < members.length; k++) {
              shape.add(_Shape.line(members[k].$2, members[(k + 1) % members.length].$2, off: off));
            }
          }
          handles
            ..addAll([if (lead == null) handle('centre', mid, (u) => m..carry('centre_x', u.$1 - lx)..carry('centre_y', u.$2 - ly))])   // (a leader is its middle: no handle over it, the leader itself is held)
            ..add(handle('size', screen(cx + r * math.sin(turn * 2 * math.pi), cy + r * math.cos(turn * 2 * math.pi) * math.cos(tilt)), (u) => m.carry('radius', math.sqrt(math.pow(u.$1 - cx, 2) + math.pow((u.$2 - cy) / math.max(.2, math.cos(tilt).abs()), 2)))));
        } else if (kind == 'row') {
          final gap = val('gap', .2), n = math.max(1, m.word(rel, 'members').split(' ').length), half = gap * (n - 1) / 2;
          shape.add(_Shape.line(screen(cx - half, cy), screen(cx + half, cy), off: off));   // (a row about its middle)
          const above = Offset(0, -30);   // (its handles over the line, not on the things in it)
          handles
            ..addAll([if (lead == null) handle('centre', mid + above, (u) {
              final (_, lift) = units(above);
              final (_, zero) = units(Offset.zero);
              m..carry('centre_x', u.$1 - lx)..carry('centre_y', u.$2 - (lift - zero) - ly);
            })])
            ..add(handle('size', screen(cx + half, cy) + above, (u) => m.carry('gap', math.max(0, u.$1 - cx) * 2 / math.max(1, n - 1))));   // (its right end out: wider both ways)
        } else if (kind == 'push' && leader != null && on(leader) != null) {
          final l = on(leader)!, r = val('radius', .5);
          shape.add(_Shape.ring(l, r * px, off: off));
          handles.add(handle('size', l + Offset(r * px, 0), (u) {
            final (lx, ly) = units(l);
            m.carry('radius', math.sqrt(math.pow(u.$1 - lx, 2) + math.pow(u.$2 - ly, 2)));
          }));
        }
      }
      final tags = wsRelationsOf(ws), sel = ws.selected == null ? null : on(ws.selected!);
      if (sel != null && m != null && !isRelation) // (a thing picked: the shape of each relation it is in, its leader's ring too, while it is carried)
        for (final r in tags.map(int.parse)) {
          double val(String n, [double or = 0]) => m.value(r, n, now) ?? or;
          final px = m.point * v.s, l = val('leader') > 0 ? on('${val('leader').round()}') : null, off = val('visible', 1) <= .5;
          switch (m.word(r, 'file')) {
            case 'around':   // (round its leader, if it has one; tipped back, through where its things show)
              final lid = val('leader') > 0 ? val('leader').round() : null, lx = lid == null ? 0.0 : m.shows(lid, 'x', now)?.$1 ?? 0, ly = lid == null ? 0.0 : m.shows(lid, 'y', now)?.$1 ?? 0;
              final pts = [for (final s in m.word(r, 'members').split(' ')) if (on(s) case final p?) p];
              if (val('tilt').abs() > 1e-6 && pts.length > 2) {
                for (var k = 0; k < pts.length; k++) {
                  shape.add(_Shape.line(pts[k], pts[(k + 1) % pts.length], off: off));
                }
              } else {
                shape.add(_Shape.ring(screen(val('centre_x') + lx, val('centre_y') + ly), val('radius', .5) * px, off: off));
              }
            case 'row':
              final half = val('gap', .2) * (m.word(r, 'members').split(' ').length - 1) / 2;
              shape.add(_Shape.line(screen(val('centre_x') - half, val('centre_y')), screen(val('centre_x') + half, val('centre_y')), off: off));
            case 'push' when l != null:
              shape.add(_Shape.ring(l, val('radius', .5) * px, off: off));
          }
        }
      return Stack(
        children: [
          IgnorePointer(child: CustomPaint(size: Size.infinite, painter: _MarksPainter(lasso, [...picked, for (final (_, p) in members) p], shape))),
          ...handles,
          for (final (id, p) in members) ...[
            Positioned(left: p.dx + 12, top: p.dy - 24, child: _Badge('×', () => wsFree(ws, id), key: ValueKey('ws-free-$id'))), // (one taken out of it where it stands: the others stay)
            if ((kind == 'follow' || kind == 'push') && id != leader) Positioned(left: p.dx - 24, top: p.dy - 24, child: _Badge('crown', () => wsLead(ws, id), faint: true, key: ValueKey('ws-lead-$id'))), // (another's crown: that one leads, nothing moving)
          ],
          if (on(leader ?? '') case final l? when leader != null) Positioned(left: l.dx - 24, top: l.dy - 24, child: _Badge('crown', () {}, key: const ValueKey('ws-leader'))), // (the leader)
          if (isRelation && (kind == 'around' || kind == 'row')) // (a ring's or a row's middle may be a thing: a faint crown on each other one, a touch puts the middle there)
            for (final l in ws.layers)
              if (l.id != leader && l.id != '$rel' && !members.any((e) => e.$1 == l.id) && m.word(int.parse(l.id), 'kind') != 'behaviour' && on(l.id) != null)
                Positioned(left: on(l.id)!.dx - 24, top: on(l.id)!.dy - 24, child: _Badge('crown', () => ws.after(m.later('set', thing: rel, name: 'leader', number: double.parse(l.id)), (_, _) => ws.reread()), faint: true, key: ValueKey('ws-lead-${l.id}'))),
          if (!isRelation && ws.manyByHand && ws.many.length > 1 && on(ws.many.first) != null) // (picked one by one: the first leads a Follow or a Push, shown before it is made)
            Positioned(left: on(ws.many.first)!.dx - 24, top: on(ws.many.first)!.dy - 24, child: _Badge('crown', () {}, faint: true, key: const ValueKey('ws-leader-to-be'))),
          if (sel != null && tags.isNotEmpty) // (the relations the picked thing is in, the one on top first: a touch picks it)
            Positioned(left: sel.dx + 20, top: sel.dy + 18, child: Row(children: [for (final r in tags) _RelateChip(m!.word(int.parse(r), 'file'), () => ws.select(r), outlined: r != wsWinner(ws), key: ValueKey('ws-in-$r'))])),   // (filled: the one that shows; the rest under it, or switched off, outlined)
          if (lasso == null && (picked.length > 1 || isRelation)) // (along the top of the stage, never over a thing)
            Positioned(
              left: v.area.left,
              right: v.area.right - v.area.width,
              top: v.area.top + 8,
              child: Center(
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: picked.length > 1
                      ? [_Said('${picked.length} picked'), for (final k in wsRelations) _RelateChip(k, () => wsRelate(ws, k))]
                      : [
                          _Said('${kind[0].toUpperCase()}${kind.substring(1)} · ${members.length}'),
                          _RelateChip('eye', () => wsSwitch(ws), glyph: (m!.value(rel!, 'visible', 0) ?? 1) > .5 ? TpGlyph.eye : TpGlyph.eyeOff, key: const ValueKey('ws-relation-switch')), // (its eye, as on its row: shut, what it is on top of shows)
                          _RelateChip('bake', () => wsRelease(ws), key: const ValueKey('ws-relate-release')),
                          _RelateChip('delete', () => ws.after(m.later('remove', thing: rel), (_, _) => ws..select(null)..reread()), key: const ValueKey('ws-relate-delete')),   // (as Blender puts Apply and Delete side by side: gone, its things back to what is under it)   // (baked: kept where they show, the relation gone; × and Delete: back to what is under it)
                        ],
                ),
              ),
            ),
        ],
      );
    },
  );
}

/// A relation's shape on the stage: a ring (its middle, how far) or a line.
class _Shape {
  _Shape.ring(this.a, this.r, {this.off = false, this.squash = 1}) : b = null;
  _Shape.line(this.a, Offset this.b, {this.off = false}) : r = 0, squash = 1;
  final Offset a;
  final Offset? b;
  final double r;
  final bool off;   // (its relation's eye shut: drawn faint)
  final double squash;   // (a ring's height to its width)
}

/// Drawn round: the line the hand drew; a ring round each thing picked together or moved by the relation picked; the
/// relation's shape, thin and broken.
class _MarksPainter extends CustomPainter {
  _MarksPainter(this.lasso, this.at, this.shape);
  final List<Offset>? lasso;
  final List<Offset> at;
  final List<_Shape> shape;
  @override
  void paint(Canvas c, Size s) {
    final ink = Paint()
      ..color = WsT.accent
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    if (lasso case final l? when l.length > 1) c.drawPath(Path()..addPolygon(l, false), ink);
    for (final p in at) {
      c.drawCircle(p, 14, ink);
    }
    final thin = Paint()
      ..color = WsT.accent.withValues(alpha: .8)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    for (final sh in shape) {
      thin.color = WsT.accent.withValues(alpha: sh.off ? .25 : .8);
      final path = sh.b == null ? (Path()..addOval(Rect.fromCenter(center: sh.a, width: 2 * sh.r, height: 2 * sh.r * sh.squash.abs()))) : (Path()..moveTo(sh.a.dx, sh.a.dy)..lineTo(sh.b!.dx, sh.b!.dy));
      for (final m in path.computeMetrics()) {
        for (var d = 0.0; d < m.length; d += 9) {
          c.drawPath(m.extractPath(d, math.min(d + 5, m.length)), thin);
        }
      }
    }
  }

  @override
  bool shouldRepaint(_MarksPainter o) => true;
}

/// The relations the picked thing is in, on top of its inspector, the one that shows first (as Blender stacks a thing's
/// constraints): each its eye, its kind and leader, × to take the thing out; pressed, that relation picked.
class WsRelationStack extends StatelessWidget {
  const WsRelationStack({super.key});
  @override
  Widget build(BuildContext context) => ListenableBuilder(listenable: Listenable.merge([WsScope.read(context).on[WsAspect.document], WsScope.read(context).on[WsAspect.picked]]), builder: (context, _) => _built(context));   // (its own: the inspector is not built again for a value of another thing)

  Widget _built(BuildContext context) {
    final ws = WsScope.read(context), m = ws.inside, t = ws.selected;
    if (m == null || t == null) return const SizedBox.shrink();
    final rels = wsRelationsOf(ws), top = wsWinner(ws);
    if (rels.isEmpty) return const SizedBox.shrink();
    String nameOf(int i) => '${ws.layers.indexWhere((l) => l.id == '$i') + 1}. ${ws.layers.where((l) => l.id == '$i').firstOrNull?.name ?? ''}';
    return Container(
      key: const ValueKey('ws-relation-stack'),
      padding: EdgeInsets.fromLTRB(Shape.s(10), Shape.s(8), Shape.s(10), Shape.s(6)),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final r in rels)
            () {
              final id = int.parse(r), kind = m.word(id, 'file'), on = (m.value(id, 'visible', 0) ?? 1) > .5;
              final lead = (m.value(id, 'leader', 0) ?? 0).round(), leads = '$lead' == t;
              final ink = r == top ? Grey.g95 : Grey.g56;
              return GestureDetector(
                key: ValueKey('ws-stack-$r'),
                behavior: HitTestBehavior.opaque,
                onTap: () => ws.select(r),
                child: Padding(
                  padding: EdgeInsets.symmetric(vertical: Shape.s(3)),
                  child: Row(
                    children: [
                      Container(width: Shape.z(6), height: Shape.z(6), decoration: BoxDecoration(color: on ? ink : const Color(0x00000000), border: Border.all(color: ink), shape: BoxShape.circle)),   // (lit: switched on; its eye is on the relation's own row and bar)
                      SizedBox(width: Shape.s(8)),
                      Text('${kind[0].toUpperCase()}${kind.substring(1)}', style: T.label(ink)),
                      SizedBox(width: Shape.s(8)),
                      Expanded(child: Text(leads ? 'leads it' : lead > 0 ? nameOf(lead) : '', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g56))),
                      if (!leads)
                        GestureDetector(
                          key: ValueKey('ws-stack-free-$r'),
                          onTap: () => ws.after(m.later('free', thing: int.parse(t), number: id.toDouble()), (_, _) => ws.reread()),
                          child: Text('×', style: T.label(Grey.g56)),
                        ),
                    ],
                  ),
                ),
              );
            }(),
        ],
      ),
    );
  }
}

/// A word in the bar over the stage, on the panel's ground (the bar lies over the stage's surround, dark or light).
class _Said extends StatelessWidget {
  const _Said(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    margin: EdgeInsets.only(right: Shape.s(6)),
    padding: EdgeInsets.symmetric(horizontal: Shape.s(8), vertical: Shape.s(5)),
    decoration: BoxDecoration(color: WsT.body, borderRadius: BorderRadius.all(Shape.r(12))),
    child: Text(text, style: T.label(Grey.g91)),
  );
}

/// What things picked together may do together, or a relation a thing is in (outlined): a word on a pill.
class _RelateChip extends StatelessWidget {
  const _RelateChip(this.kind, this.onTap, {this.outlined = false, this.glyph, super.key});
  final String kind;
  final VoidCallback onTap;
  final bool outlined;
  final TpGlyph? glyph;   // (drawn instead of its word)
  @override
  Widget build(BuildContext context) => GestureDetector(
    key: key == null ? ValueKey('ws-relate-$kind') : null,
    onTap: onTap,
    child: Container(
      margin: EdgeInsets.only(right: Shape.s(4)),
      padding: EdgeInsets.symmetric(horizontal: Shape.s(10), vertical: Shape.s(5)),
      decoration: BoxDecoration(color: outlined ? WsT.body : WsT.accent, border: outlined ? Border.all(color: WsT.accent) : null, borderRadius: BorderRadius.all(Shape.r(12))),
      child: glyph != null ? TpGlyphView(glyph!, WsT.onAccent, size: Shape.z(12)) : Text(kind[0].toUpperCase() + kind.substring(1), style: T.label(outlined ? WsT.accent : WsT.onAccent)),
    ),
  );
}

/// A small round mark on a thing a relation moves: × takes it out, a crown is its leader.
class _Badge extends StatelessWidget {
  const _Badge(this.glyph, this.onTap, {this.faint = false, super.key});
  final String glyph;
  final VoidCallback onTap;
  final bool faint;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: onTap,
    child: Container(
      width: 18,
      height: 18,
      decoration: BoxDecoration(color: faint ? WsT.body.withValues(alpha: .7) : WsT.accent, shape: BoxShape.circle),
      child: CustomPaint(painter: _GlyphPaint(glyph == '×' ? 'x' : glyph, faint ? Grey.g63 : WsT.onAccent)),
    ),
  );
}

/// A small mark drawn, not a letter (no font has to carry it): a cross (a middle), an arrow out (a size), an x (out of
/// it), a crown (the leader).
class _GlyphPaint extends CustomPainter {
  _GlyphPaint(this.what, this.ink);
  final String what;
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final m = s.center(Offset.zero), r = s.shortestSide * .28;
    final pen = Paint()
      ..color = ink
      ..strokeWidth = 1.6
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;
    switch (what) {
      case 'cross':
        c
          ..drawLine(m - Offset(r, 0), m + Offset(r, 0), pen)
          ..drawLine(m - Offset(0, r), m + Offset(0, r), pen);
      case 'arrow':
        c
          ..drawLine(m - Offset(r, 0), m + Offset(r, 0), pen)
          ..drawLine(m + Offset(r, 0), m + Offset(r * .4, -r * .6), pen)
          ..drawLine(m + Offset(r, 0), m + Offset(r * .4, r * .6), pen);
      case 'x':
        c
          ..drawLine(m - Offset(r, r), m + Offset(r, r), pen)
          ..drawLine(m - Offset(r, -r), m + Offset(r, -r), pen);
      case 'crown':
        c.drawPath(
          Path()..addPolygon([m + Offset(-r * 1.2, r * .8), m + Offset(-r * 1.2, -r * .6), m + Offset(-r * .5, 0), m + Offset(0, -r), m + Offset(r * .5, 0), m + Offset(r * 1.2, -r * .6), m + Offset(r * 1.2, r * .8)], true),
          Paint()..color = ink,
        );
    }
  }

  @override
  bool shouldRepaint(_GlyphPaint o) => o.what != what || o.ink != ink;
}

/// The pill at the bottom-left of the viewport: zoom and where the pointer is, in composition px.
class _Hud extends StatelessWidget {
  _Hud({required this.scale, required this.cursor});
  final double scale;
  final ValueNotifier<Offset?> cursor;
  @override
  Widget build(BuildContext context) => Container(
    height: _V.hud,
    padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
    decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('${(scale * 100).round()}%', style: T.value(Grey.g95)),
        Container(width: 1, height: Shape.z(10), margin: EdgeInsets.symmetric(horizontal: Shape.s(8)), color: WsT.line),
        ValueListenableBuilder<Offset?>(
          valueListenable: cursor,
          builder: (_, p, _) => Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('X ', style: T.micro(Grey.g56)),
              SizedBox(width: Shape.z(34), child: Text(p == null ? '—' : '${p.dx.round()}', style: T.value(p == null ? Grey.g56 : Grey.g91))),
              Text('Y ', style: T.micro(Grey.g56)),
              SizedBox(width: Shape.z(34), child: Text(p == null ? '—' : '${p.dy.round()}', style: T.value(p == null ? Grey.g56 : Grey.g91))),
            ],
          ),
        ),
      ],
    ),
  );
}

/// The navigator: the whole frame small, the part the viewport shows outlined in the accent. Shown only when zoomed past fit.
class _Navigator extends StatelessWidget {
  const _Navigator({required this.scene, required this.view, this.texture});
  final ArtScene scene;
  final _View view;

  /// Joined: the inside's own picture, the whole frame, under the outline (the same texture the stage shows).
  final int? texture;
  @override
  Widget build(BuildContext context) {
    final h = _V.mapW * WsArt.comp.height / WsArt.comp.width;
    return Container(
      width: _V.mapW + 8,
      height: h + 8,
      padding: EdgeInsets.all(Shape.s(4)),
      color: WsT.card,
      child: Stack(
        fit: StackFit.expand,
        children: [
          if (texture != null) Texture(textureId: texture!, filterQuality: FilterQuality.medium),
          CustomPaint(painter: _MapPainter(scene, Rect.fromPoints(view.toComp(view.area.topLeft), view.toComp(view.area.bottomRight)))),
        ],
      ),
    );
  }
}

class _MapPainter extends CustomPainter {
  _MapPainter(this.scene, this.shown);
  final ArtScene scene;
  final Rect shown;
  @override
  void paint(Canvas c, Size s) {
    final k = s.width / WsArt.comp.width;
    c.save();
    c.clipRect(Offset.zero & s);
    c.scale(k);
    c.restore();
    final r = Rect.fromLTRB(shown.left * k, shown.top * k, shown.right * k, shown.bottom * k).intersect(Offset.zero & s);
    c.drawRect(
      r.deflate(.75),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = WsT.accent,
    );
  }

  @override
  bool shouldRepaint(_MapPainter o) => o.scene != scene || o.shown != shown;
}

// ---- the picture and what sits on it ----------------------------------------------------------------------------------

Paint _line(Color c, [double w = 1]) => Paint()
  ..style = PaintingStyle.stroke
  ..strokeWidth = w
  ..color = c;

/// A line that reads on any artwork: a dark under-stroke, then the colour.
void _legible(Canvas c, Path p, Color col, [double w = 1.5]) {
  c.drawPath(p, _line(Grey.g00, w + 2));
  c.drawPath(p, _line(col, w));
}

TextPainter _tp(String t, TextStyle s) => TextPainter(
  text: TextSpan(text: t, style: s),
  textDirection: TextDirection.ltr,
  maxLines: 1,
)..layout();

/// What one painter draws: everything (the stand-in art too), or, when the inside's picture sits in the composition,
/// what lies under it (the surround, the captions) and what lies over it (grid, guides, the selection).
enum _Part { all, under, over }

class _PointsPainter extends CustomPainter {
  _PointsPainter(this.at, this.handles);
  final List<Offset> at;
  final List<(Offset, Offset)> handles;
  @override
  void paint(Canvas c, Size _) {
    final edge = Paint()..color = WsT.accent..style = PaintingStyle.stroke..strokeWidth = 1, inside = Paint()..color = const Color(0xFFFFFFFF);
    for (final (h, o) in handles) {
      if ((h - o).distance < 1) continue; // (a handle lying on its point: none to see)
      c..drawLine(o, h, edge)..drawCircle(h, 3, inside)..drawCircle(h, 3, edge);
    }
    for (final p in at) {
      final r = Rect.fromCenter(center: p, width: Shape.z(6), height: Shape.z(6));
      c..drawRect(r, inside)..drawRect(r, edge);
    }
  }

  @override
  bool shouldRepaint(_PointsPainter o) => !listEquals(o.at, at) || !listEquals(o.handles, handles);
}

class _HandlePainter extends CustomPainter {
  _HandlePainter(this.at, this.out);
  final Offset at, out;
  @override
  void paint(Canvas c, Size _) {
    final line = Paint()..color = WsT.accent..strokeWidth = 1, dot = Paint()..color = WsT.accent;
    c.drawLine(at * 2 - out, out, line);
    for (final p in [at * 2 - out, out]) c.drawCircle(p, 3, dot);
    c.drawRect(Rect.fromCenter(center: at, width: Shape.z(6), height: Shape.z(6)), dot);
  }

  @override
  bool shouldRepaint(_HandlePainter o) => o.at != at || o.out != out;
}
