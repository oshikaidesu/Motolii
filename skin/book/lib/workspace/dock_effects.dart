// The Effects shelf: what the inside does, in three plain lists: the effects a thing may be given (picked, its foot puts
// it on or takes it off), the whole-screen effects (on the stage, in their order), the motions (one at a time).
import 'dart:io';

import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'dock_parts.dart';
import 'ws.dart';

class EffectsShelf extends StatelessWidget {
  const EffectsShelf(this.c, {super.key});
  final DockCtx c;

  @override
  Widget build(BuildContext context) {
    final ws = c.ws, m = ws.inside;
    final blocks = <Widget>[], weights = <double>[];
    void section(String title, Color hue, List<Widget> rows, {double? weight}) {
      if (rows.isEmpty) return;
      final folded = c.query.isEmpty && c.mem.folded.contains(title);
      blocks.add(
        Padding(
          padding: EdgeInsets.only(bottom: WsT.gutter),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DockSection(title, count: rows.length, hue: hue, folded: folded, onTap: () => c.toggle(c.mem.folded, title)),
              if (!folded) ...rows,
            ],
          ),
        ),
      );
      weights.add(1.0 + (folded ? 0 : weight ?? rows.length));
    }

    // (one list: with a thing picked, every effect goes on it, as the camera sees it; with none, the shaders go on the whole stage)
    final on = ws.fxOf(ws.selected), things = [for (final (name, _) in m?.thingEffects ?? const <(String, String)>[]) if (c.hit(name)) name];
    final screen = ws.screenFx, picked = int.tryParse(ws.selected ?? '') != null;
    final shaders = [for (final e in m?.effects ?? const <(String, String)>[]) if (c.hit(e.$1)) e];   // (effects/*.wgsl, their knobs on the card of what they are on)
    section(picked ? 'Effects' : 'Screen', picked ? Pop.toneBlend : Pop.toneOutput, [
      if (picked) for (final n in things) _FxRow(n, c, on: on.contains(n)),
      if (shaders.isNotEmpty)
        LayoutBuilder(
          builder: (context, box) {
            final cols = wsCols(box.maxWidth, Shape.z(84)), w = wsCell(box.maxWidth, cols);
            return Wrap(
              spacing: WsT.gutter,
              runSpacing: WsT.gutter,
              children: [for (final e in shaders) SizedBox(width: w, child: _ScreenTile(e.$1, e.$2, on: screen.indexOf(e.$2), onTap: () => ws.screenToggle(e.$2)))],
            );
          },
        ),
    ], weight: (picked ? things.length : 0) + shaders.length * 1.4);
    // (the inside's motions, Cavalry's Animators, one at a time on the picked thing: animators/*.js)
    if (ws.selected != null) {
      section('Motion', Pop.toneOutput, [
        for (final e in m?.animators ?? const <(String, String)>[])
          if (c.hit(e.$1)) _ScreenRow(e.$1, of: 'motion', on: ws.motion == e.$2 ? 0 : -1, onTap: () => ws.motionToggle(e.$2)),
      ]);
    }
    final sel = things.contains(c.sel) ? c.sel : null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(child: blocks.isEmpty ? DockEmpty(c.query) : DockMasonry(minColumn: 260, weights: weights, children: blocks)),
        if (sel != null) _FxFooter(sel, ws: ws, done: on.contains(sel)),
      ],
    );
  }
}

/// A row is words only: the name, who does it, ON when the picked thing carries it, the star.
class _FxRow extends StatelessWidget {
  const _FxRow(this.name, this.c, {required this.on});
  final String name;
  final DockCtx c;
  final bool on;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == name, star = c.starred(name);
    return DockHover(
      onTap: () => c.pick(name),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: Shape.z(24),
        padding: EdgeInsets.only(left: Shape.s(8), right: Shape.s(4)),
        color: dockFill(sel, h),
        child: Row(
          children: [
            CustomPaint(size: const Size(18, 16), painter: _FxLook(name)), // (what it does, drawn: found with the words hidden)
            SizedBox(width: Shape.s(6)),
            Flexible(
              child: FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerLeft, child: Text(name, maxLines: 1, style: T.name(dockInk(sel)).copyWith(fontWeight: sel ? FontWeight.w700 : FontWeight.w500))),
            ),
            SizedBox(width: Shape.s(6)),
            Expanded(child: Text('ThorVG', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g56)))),
            if (on) ...[DockTag('ON', hue: sel ? null : WsT.accent), SizedBox(width: Shape.s(2))],
            if (star || h || sel) DockStar(on: star, ink: sel ? WsT.onAccent : null, onTap: () => c.star(name)) else SizedBox(width: Shape.s(20)),
          ],
        ),
      ),
    );
  }
}

/// A whole-screen effect: its name, and its place among those on the stage when it is on (1 nearest the picture).
class _ScreenRow extends StatelessWidget {
  const _ScreenRow(this.name, {required this.on, required this.onTap, this.of = 'screen'});
  final String name, of;
  final int on; // (its place among those on, from 0; -1: off; a motion is one alone, ON)
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => DockHover(
    key: ValueKey('ws-$of-$name'),
    onTap: onTap,
    builder: (context, h) => AnimatedContainer(
      duration: Mo.dur,
      curve: Mo.ease,
      height: Shape.z(24),
      padding: EdgeInsets.only(left: Shape.s(22), right: Shape.s(4)),
      color: dockFill(false, h),
      child: Row(
        children: [
          Expanded(child: FittedBox(fit: BoxFit.scaleDown, alignment: Alignment.centerLeft, child: Text(name, maxLines: 1, style: T.name(dockInk(false)).copyWith(fontWeight: FontWeight.w500)))),
          if (on >= 0) ...[DockTag(of == 'screen' ? 'ON ${on + 1}' : 'ON', hue: WsT.accent), SizedBox(width: Shape.s(2))],
        ],
      ),
    ),
  );
}

/// A whole-screen effect as a tile: what it does to the Poster recipe (the .png beside its .wgsl, made by
/// filament/fx_tiles.sh; none: a plain tile), its name, and its place among those on (1 nearest the picture).
class _ScreenTile extends StatelessWidget {
  const _ScreenTile(this.name, this.path, {required this.on, required this.onTap});
  final String name, path;
  final int on;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => DockHover(
    key: ValueKey('ws-screen-$name'),
    onTap: onTap,
    builder: (context, h) => Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        AspectRatio(
          aspectRatio: 1,
          child: Stack(
            fit: StackFit.expand,
            children: [
              Image.file(File(path.replaceFirst(RegExp(r'\.wgsl$'), '.png')), fit: BoxFit.cover, cacheWidth: 256, errorBuilder: (_, _, _) => ColoredBox(color: WsT.card)),
              if (on >= 0) Positioned(left: 3, top: 3, child: DockTag('ON ${on + 1}', hue: WsT.accent, edge: true)),
              if (on >= 0 || h) DecoratedBox(decoration: BoxDecoration(border: Border.all(color: on >= 0 ? WsT.accentInk : Grey.g76, width: on >= 0 ? 2 : 1))),
            ],
          ),
        ),
        Container(
          height: Shape.z(18),
          padding: EdgeInsets.symmetric(horizontal: Shape.s(4)),
          alignment: Alignment.centerLeft,
          color: dockFill(false, h, WsT.card),
          child: FittedBox(fit: BoxFit.scaleDown, child: Text(name, maxLines: 1, style: T.micro(dockInk(false, Grey.g76)))), // (a name wider than its tile is drawn smaller, never cut)
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
