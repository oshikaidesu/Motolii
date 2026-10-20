// The Effects shelf: all the inside does to a picture, as tiles sorted by what each is for (the inside's labels as
// chips, not who draws it): the effects a thing may be given (picked: its foot puts it on or takes it off), the
// whole-screen effects (on the picked thing as the camera sees it, or on the stage, in their order), the motions.
import 'dart:io';

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_parts.dart';
import 'ws.dart';

class EffectsShelf extends StatelessWidget {
  const EffectsShelf(this.c, {super.key});
  final DockCtx c;

  @override
  Widget build(BuildContext context) {
    final ws = c.ws, m = ws.inside;
    if (m == null) return DockEmpty(c.query);
    final on = ws.fxOf(ws.selected), screen = ws.screenFx, picked = int.tryParse(ws.selected ?? '') != null;
    // (a thing's own effects and motions go on a picked thing only; the whole-screen ones on it, or on the stage)
    final all = [
      if (picked) for (final (n, _) in m.thingEffects) (n, m.label(n), _Tile(n, look: _FxLook(n), on: on.contains(n) ? 'ON' : null, picked: c.sel == n, onTap: () => c.pick(n), star: c)),
      for (final (n, f) in m.effects)
        (n, m.label(n), _Tile(n, key: ValueKey('ws-screen-$n'), file: f, on: screen.contains(f) ? 'ON ${screen.indexOf(f) + 1}' : null, onTap: () => ws.screenToggle(f))),
      if (picked)
        for (final (n, f) in m.animators) (n, 'Motion', _Tile(n, key: ValueKey('ws-motion-$n'), on: ws.motion == f ? 'ON' : null, onTap: () => ws.motionToggle(f))),
    ].where((e) => c.hit(e.$1)).toList();
    final kinds = <String, int>{};
    for (final e in all) {
      if (e.$2.isNotEmpty) kinds[e.$2] = (kinds[e.$2] ?? 0) + 1;
    }
    final chip = kinds.containsKey(c.chip) ? c.chip : 'All', shown = [for (final e in all) if (chip == 'All' || e.$2 == chip) e.$3];
    final sel = m.thingEffects.any((e) => e.$1 == c.sel) && picked ? c.sel : null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips([('All', all.length), for (final k in kinds.entries) (k.key, k.value)], active: chip, onPick: c.pickChip),
        Expanded(
          child: shown.isEmpty
              ? DockEmpty(c.query)
              : SingleChildScrollView(
                  child: LayoutBuilder(
                    builder: (context, box) {
                      final cols = wsCols(box.maxWidth, Shape.z(84)), w = wsCell(box.maxWidth, cols);
                      return Wrap(spacing: WsT.gutter, runSpacing: WsT.gutter, children: [for (final t in shown) SizedBox(width: w, child: t)]);
                    },
                  ),
                ),
        ),
        if (sel != null) _FxFooter(sel, ws: ws, done: on.contains(sel)),
      ],
    );
  }
}

/// One tile, whatever draws it: what it does to a sample (a whole-screen effect's .png beside its .wgsl, made by
/// filament/fx_tiles.sh; a thing's effect its small drawing; none: a plain tile), its name, ON (and its place among
/// those on the stage) when it is on, the star for a thing's effect.
class _Tile extends StatelessWidget {
  const _Tile(this.name, {super.key, this.file, this.look, required this.on, required this.onTap, this.picked = false, this.star});
  final String name;
  final String? file, on;
  final CustomPainter? look;
  final bool picked;
  final VoidCallback onTap;
  final DockCtx? star;
  @override
  Widget build(BuildContext context) => DockHover(
    onTap: onTap,
    builder: (context, h) => Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        AspectRatio(
          aspectRatio: 1,
          child: Stack(
            fit: StackFit.expand,
            children: [
              ColoredBox(color: WsT.card),
              if (file != null)
                Image.file(File(file!.replaceFirst(RegExp(r'\.wgsl$'), '.png')), fit: BoxFit.cover, cacheWidth: 256, errorBuilder: (_, _, _) => const SizedBox())
              else if (look != null)
                FittedBox(child: CustomPaint(size: const Size(18, 16), painter: look)),
              if (on case final o?) Positioned(left: 3, top: 3, child: DockTag(o, hue: WsT.accent, edge: true)),
              if (star case final s? when s.starred(name) || h) Positioned(right: 0, top: 0, child: DockStar(on: s.starred(name), onTap: () => s.star(name))),
              if (on != null || picked || h) DecoratedBox(decoration: BoxDecoration(border: Border.all(color: on != null || picked ? WsT.accentInk : Grey.g76, width: on != null || picked ? 2 : 1))),
            ],
          ),
        ),
        Container(
          height: Shape.z(18),
          padding: EdgeInsets.symmetric(horizontal: Shape.s(4)),
          alignment: Alignment.centerLeft,
          color: dockFill(picked, h, WsT.card),
          child: FittedBox(fit: BoxFit.scaleDown, child: Text(name, maxLines: 1, style: T.micro(dockInk(picked, Grey.g76)))), // (a name wider than its tile is drawn smaller, never cut)
        ),
      ],
    ),
  );
}

/// Joined: the document's value an effect is, and what the picked thing has for it now.
String _said(Ws ws, String name) {
  final m = ws.inside!, v = m.thingEffects.firstWhere((e) => e.$1 == name).$2, t = int.tryParse(ws.selected ?? ''); // (the shelf shows only what the inside does)
  final now = t == null ? '' : m.word(t, v);
  return now.isEmpty ? '$v · none on it' : '$v · $now';
}

/// The foot: the picked effect, what the analysis says about it, and Add (keeps the try-on, or adds straight away).
class _FxFooter extends StatelessWidget {
  const _FxFooter(this.name, {required this.ws, required this.done});
  final String name;
  final Ws ws;
  final bool done;
  @override
  Widget build(BuildContext context) => DockFooter(
    title: name,
    meta: _said(ws, name),
    // (an effect goes on or comes off the picked thing)
    action: done ? 'Remove' : 'Add',
    done: ws.selected == null,
    doneLabel: 'Pick a thing',
    onAction: () => ws.fxToggle(name),
  );
}

/// What each of the inside's own effects does to a square, drawn small: a blur's soft edge, a shadow under it, a glow
/// round it, a path half drawn, two tones, a ramp of colours. (An effect the window does not know has no picture.)
class _FxLook extends CustomPainter {
  const _FxLook(this.name);
  final String name;
  @override
  void paint(Canvas c, Size s) {
    const ink = Color(0xFFE0A35A), r = Rect.fromLTWH(4, 3, 10, 10);
    switch (name) {
      case 'Gaussian Blur':
        c.drawRect(r, Paint()
          ..color = ink
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3.5)); // (soft all round: nothing sharp left)
      case 'Drop Shadow':
        c.drawRect(r.shift(const Offset(2, 2)), Paint()
          ..color = const Color(0xFF000000)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.2));
        c.drawRect(r, Paint()..color = ink);
      case 'Glow':
        c.drawRect(r, Paint()
          ..color = const Color(0xFFFFE6A8)
          ..maskFilter = const MaskFilter.blur(BlurStyle.outer, 3));
        c.drawRect(r, Paint()..color = ink);
      case 'Trim Paths':
        c.drawArc(Rect.fromCircle(center: r.center, radius: 5.5), -1.6, 3.8, false, Paint()
          ..color = ink
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.8
          ..strokeCap = StrokeCap.round);
      case 'Duotone':
        c.drawRect(Rect.fromLTWH(r.left, r.top, r.width / 2, r.height), Paint()..color = const Color(0xFF2B3A67));
        c.drawRect(Rect.fromLTWH(r.center.dx, r.top, r.width / 2, r.height), Paint()..color = const Color(0xFFF4C27A));
      case 'Gradient Map':
        c.drawRect(r, Paint()..shader = const LinearGradient(colors: [Color(0xFF2B3A67), Color(0xFFE0607E), Color(0xFFF4C27A)]).createShader(r));
    }
  }

  @override
  bool shouldRepaint(_FxLook o) => o.name != name;
}
