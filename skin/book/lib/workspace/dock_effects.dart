// The Effects shelf: all the inside does to a picture, as tiles sorted by what each is for (the inside's labels as
// chips, not who draws it): the effects a thing may be given (picked: its foot puts it on or takes it off), the
// whole-screen effects (on the picked thing as the camera sees it, or on the stage, in their order), the motions.
import 'dart:async';
import 'dart:io';

import 'package:flutter/widgets.dart';

import '../tokens.dart';
import 'dock_parts.dart';
import 'effect_tiles.dart';
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
    // (each: what it is told by (a thing's own by its name; a file's by its file: two files may share a name, Halftone's
    //  three.js .wgsl and an AviUtl2 script), its name, what it is for, its tile, whether it is on, what two presses do
    //  (also what a drop on the stage does), what it says of itself)
    final all = <({String id, String name, String label, Widget tile, bool on, VoidCallback apply, String said})>[
      if (picked)
        for (final (n, _) in m.thingEffects)
          (id: n, name: n, label: m.label(n), on: on.contains(n), apply: () => ws.fxToggle(n), said: _said(ws, n),
           tile: _Tile(n, file: m.picture(n), look: _FxLook(n), on: on.contains(n) ? 'ON' : null, picked: c.sel == n, onTap: () => c.pick(n), onApply: () => ws.fxToggle(n), star: c)),
      for (final (n, f) in m.effects)
        (id: f, name: n, label: m.label(n), on: screen.contains(f), apply: () => ws.screenToggle(f), said: 'On the picked thing',
         tile: _Tile(n, key: ValueKey('ws-screen-$f'), file: m.picture(f), same: m.tileSame(f), on: screen.contains(f) ? 'ON ${screen.indexOf(f) + 1}' : null, picked: c.sel == f, onTap: () => c.pick(f), onApply: () => ws.screenToggle(f))),
      if (picked)
        for (final (n, f) in m.animators)
          (id: f, name: n, label: 'Motion', on: ws.motion == f, apply: () => ws.motionToggle(f), said: 'A motion for the picked thing',
           tile: _Tile(n, key: ValueKey('ws-motion-$n'), on: ws.motion == f ? 'ON' : null, picked: c.sel == f, onTap: () => c.pick(f), onApply: () => ws.motionToggle(f))),
    ].where((e) => c.hit(e.name)).toList();
    final kinds = <String, int>{};
    for (final e in all) {
      if (e.label.isNotEmpty) kinds[e.label] = (kinds[e.label] ?? 0) + 1;
    }
    final chip = kinds.containsKey(c.chip) ? c.chip : 'All', shown = [for (final e in all) if (chip == 'All' || e.label == chip) e.tile];
    final sel = all.where((e) => e.id == c.sel).firstOrNull;
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
        if (sel != null) _FxFooter(sel.name, kind: sel.label, said: sel.said, done: sel.on, ws: ws, onAction: sel.apply, needs: !picked),   // (an effect is always a thing's: none picked, none put on)
      ],
    );
  }
}

/// One tile, whatever draws it: what it does to a sample (the picture the inside says, drawn by effect_tiles.dart and
/// moving while the hand is on it, as Alight Motion's tiles move; or made by filament/fx_tiles.sh; without one, a thing's
/// effect its small drawing, else a plain tile), its name, ON (and its place among those on the stage) when it is on,
/// NO CHANGE when it is seen to change nothing on the sample, the star for a thing's effect.
class _Tile extends StatelessWidget {
  const _Tile(this.name, {super.key, this.file, this.look, required this.on, required this.onTap, this.onApply, this.picked = false, this.same = false, this.star});
  final String name;
  final String? file, on;
  final CustomPainter? look;
  final bool picked, same;
  final VoidCallback onTap;
  final VoidCallback? onApply;   // (two presses: it is put on, or taken off)
  final DockCtx? star;
  @override
  Widget build(BuildContext context) => DockHover(
    onTap: onTap,
    onDoubleTap: onApply,
    builder: (context, h) => Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        AspectRatio(
          aspectRatio: 1,
          child: Stack(
            fit: StackFit.expand,
            children: [
              ColoredBox(color: WsT.card),
              if (file case final f? when f.isNotEmpty)
                _Moving(f, playing: h)
              else if (look != null)
                FittedBox(child: CustomPaint(size: const Size(18, 16), painter: look)),
              if (on case final o?) Positioned(left: 3, top: 3, child: DockTag(o, hue: WsT.accent, edge: true)),
              if (same) Positioned(left: 3, bottom: 3, child: DockTag('NO CHANGE', hue: Grey.g63, edge: true)),   // (the sample as it is: its defaults show nothing on a photo, or it helps other scripts)
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

/// A tile's picture, its eight moving ones played round and round while [playing] (the media shelf's 280 ms a picture,
/// dock_files_contents.dart); its still otherwise, and for a picture with none beside it.
class _Moving extends StatefulWidget {
  const _Moving(this.file, {required this.playing});
  final String file;
  final bool playing;
  @override
  State<_Moving> createState() => _MovingState();
}

class _MovingState extends State<_Moving> {
  int? _step;
  Timer? _clock;

  void _follow() {
    final go = widget.playing && File(EffectTiles.frame(widget.file, 0)).existsSync();
    if (go == (_clock != null)) return;
    _clock?.cancel();
    _clock = go ? Timer.periodic(const Duration(milliseconds: 280), (_) => mounted ? setState(() => _step = ((_step ?? EffectTiles.still) + 1) % EffectTiles.moments.length) : null) : null;
    if (!go) _step = null;
  }

  @override
  void initState() {
    super.initState();
    _follow();
  }

  @override
  void didUpdateWidget(_Moving old) {
    super.didUpdateWidget(old);
    _follow();
  }

  @override
  void dispose() {
    _clock?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Image.file(File(_step == null ? widget.file : EffectTiles.frame(widget.file, _step!)), fit: BoxFit.cover, cacheWidth: 256, gaplessPlayback: true, errorBuilder: (_, _, _) => const SizedBox());
}

/// Joined: the document's value an effect is, and what the picked thing has for it now.
String _said(Ws ws, String name) {
  final m = ws.inside!, v = m.thingEffects.firstWhere((e) => e.$1 == name).$2, t = int.tryParse(ws.selected ?? ''); // (the shelf shows only what the inside does)
  final now = t == null ? '' : m.word(t, v);
  return now.isEmpty ? '$v · none on it' : '$v · $now';
}

/// The foot: the picked effect, what it is for and what it does, and Add (or Remove): one press picks, two put it on.
class _FxFooter extends StatelessWidget {
  const _FxFooter(this.name, {required this.kind, required this.said, required this.ws, required this.done, required this.onAction, required this.needs});
  final String name, kind, said;
  final Ws ws;
  final bool done, needs;   // (needs: an effect of a thing, with no thing picked)
  final VoidCallback onAction;
  @override
  Widget build(BuildContext context) => DockFooter(
    title: name,
    meta: kind.isEmpty ? said : '$kind · $said · double-click to add',
    action: done ? 'Remove' : 'Add',
    done: needs,
    doneLabel: 'Pick a thing',
    onAction: onAction,
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
