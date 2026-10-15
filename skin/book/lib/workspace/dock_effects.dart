// The Effects shelf: a plain list by family, narrowed by the filter band (what the shader declares, what the author tagged).
// No thumbnails: picking a row tries the effect on the selected layer, on the stage, through the real renderer. Add keeps it.
import 'package:flutter/widgets.dart';

import '../sets/inspector/inspector_parts.dart' show Pop;
import '../tokens.dart';
import 'dock_filters.dart';
import 'dock_parts.dart';
import 'fx_catalog.dart';
import 'ws.dart';

const _collection = 'Collection', _starred = 'Starred';

/// From this shelf width the filter band stands beside the list, full height, instead of above it.
const _besideFrom = 560.0;

Color fxHue(FxFamily f) => switch (f) {
  FxFamily.blur => Pop.toneLayout,
  FxFamily.light => Pop.toneOutput,
  FxFamily.color => Pop.toneFill,
  FxFamily.stylize => Pop.toneBlend,
  FxFamily.distort => Pop.toneLink,
  FxFamily.space => Pop.toneSolid,
  FxFamily.path => Pop.tonePath,
  FxFamily.place => Pop.tonePlace,
  FxFamily.time => Pop.toneTime,
};

/// How many filters are on in the Effects band (for the toggle beside the search).
int fxActiveFilters(DockMem m) => m.filter.values.where((s) => s.isNotEmpty).length;

class EffectsShelf extends StatelessWidget {
  const EffectsShelf(this.c, {super.key});
  final DockCtx c;

  bool _has(FxDef f, String g, String t) => g == _collection ? c.starred(f.name) : fxHas(f, g, t);

  bool _keep(FxDef f, {String? except}) =>
      c.hit(f.name) && c.mem.filter.entries.every((e) => e.key == except || e.value.isEmpty || e.value.any((t) => _has(f, e.key, t)));

  void _toggle(String g, String t, bool add) => c.set(() {
    final s = c.mem.filter.putIfAbsent(g, () => <String>{});
    if (add) {
      s.contains(t) ? s.remove(t) : s.add(t);
    } else if (s.length == 1 && s.contains(t)) {
      s.clear();
    } else {
      s
        ..clear()
        ..add(t);
    }
  });

  void _pick(Ws ws, String name) {
    if (c.sel == name && ws.trial == name) return ws.dropFx();
    c.pick(name);
    ws.tryFx(name);
  }

  @override
  Widget build(BuildContext context) {
    final ws = c.ws, shown = fxCatalog.where((f) => _keep(f)).toList();
    final blocks = <Widget>[], weights = <double>[];
    // (joined: the inside's whole-screen effects first, each on the stage or not: effects/*.wgsl, its knobs on the stage's card)
    final screen = [for (final e in ws.inside?.effects ?? const <(String, String)>[]) if (c.hit(e.$1)) e];
    if (screen.isNotEmpty) {
      final folded = c.query.isEmpty && c.mem.folded.contains('Screen');
      final on = ws.screenFx;
      blocks.add(
        Padding(
          padding: const EdgeInsets.only(bottom: WsT.gutter),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DockSection('Screen', count: screen.length, hue: Pop.toneOutput, folded: folded, onTap: () => c.toggle(c.mem.folded, 'Screen')),
              if (!folded)
                for (final e in screen) _ScreenRow(e.$1, on: on.indexOf(e.$2), onTap: () => ws.screenToggle(e.$2)),
            ],
          ),
        ),
      );
      weights.add(1.0 + (folded ? 0 : screen.length));
    }
    // (joined: the inside's motions, Cavalry's Animators, one at a time on the picked thing: animators/*.js)
    final motions = [for (final e in ws.inside?.animators ?? const <(String, String)>[]) if (c.hit(e.$1)) e];
    if (motions.isNotEmpty && ws.selected != null) {
      final folded = c.query.isEmpty && c.mem.folded.contains('Motion');
      final has = ws.motion;
      blocks.add(
        Padding(
          padding: const EdgeInsets.only(bottom: WsT.gutter),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DockSection('Motion', count: motions.length, hue: Pop.toneOutput, folded: folded, onTap: () => c.toggle(c.mem.folded, 'Motion')),
              if (!folded)
                for (final e in motions) _ScreenRow(e.$1, of: 'motion', on: has == e.$2 ? 0 : -1, onTap: () => ws.motionToggle(e.$2)),
            ],
          ),
        ),
      );
      weights.add(1.0 + (folded ? 0 : motions.length));
    }
    for (final fam in FxFamily.values) {
      final items = shown.where((f) => f.family == fam).toList();
      if (items.isEmpty) continue;
      final folded = c.query.isEmpty && c.mem.folded.contains(fam.label);
      blocks.add(
        Padding(
          padding: const EdgeInsets.only(bottom: WsT.gutter),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              DockSection(fam.label, count: items.length, hue: fxHue(fam), folded: folded, onTap: () => c.toggle(c.mem.folded, fam.label)),
              if (!folded)
                for (final f in items) _FxRow(f, c, on: ws.fxOf(ws.selected).contains(f.name), trying: ws.trial == f.name, onTap: () => _pick(ws, f.name)),
            ],
          ),
        ),
      );
      weights.add(1.0 + (folded ? 0 : items.length));
    }
    final sel = shown.where((f) => f.name == c.sel).firstOrNull;
    final groups = [
      (_collection, const [_starred]),
      for (final g in fxGroups) (g, fxTagsOf(g)),
    ];
    final list = blocks.isEmpty ? DockEmpty(c.query.isEmpty ? 'these filters' : c.query) : DockMasonry(minColumn: 260, weights: weights, children: blocks);
    final footer = sel == null ? null : _FxFooter(sel, ws: ws, done: ws.fxOf(ws.selected).contains(sel.name));
    return LayoutBuilder(
      builder: (context, box) {
        final band = DockFilterBand(
          groups: groups,
          chosen: c.mem.filter,
          folded: c.mem.filterFolds,
          results: shown.length,
          count: (g, t) => fxCatalog.where((f) => _keep(f, except: g) && _has(f, g, t)).length,
          onFold: (g) => c.toggle(c.mem.filterFolds, g),
          onToggle: _toggle,
          onClear: () => c.set(c.mem.filter.clear),
        );
        if (c.mem.filtersOpen && box.maxWidth >= _besideFrom) {
          return Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Expanded(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    SizedBox(width: 250, child: band),
                    ColoredBox(
                      color: WsT.ground,
                      child: SizedBox(width: WsT.gutter),
                    ),
                    Expanded(child: list),
                  ],
                ),
              ),
              ?footer,
            ],
          );
        }
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            if (c.mem.filtersOpen)
              DockResizable(height: c.mem.filterHeight, max: box.maxHeight - 96, onHeight: (h) => c.set(() => c.mem.filterHeight = h), child: band),
            Expanded(child: list),
            ?footer,
          ],
        );
      },
    );
  }
}

/// A row is words only: the name, what it does, ON when the selected layer carries it, the star.
class _FxRow extends StatelessWidget {
  const _FxRow(this.f, this.c, {required this.on, required this.trying, required this.onTap});
  final FxDef f;
  final DockCtx c;
  final bool on, trying;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == f.name, star = c.starred(f.name);
    return DockHover(
      onTap: onTap,
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 24,
        padding: const EdgeInsets.only(left: 22, right: 4),
        color: dockFill(sel, h),
        child: Row(
          children: [
            Flexible(
              child: Text(
                f.name,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.name(dockInk(sel)).copyWith(fontWeight: sel ? FontWeight.w700 : FontWeight.w500),
              ),
            ),
            const SizedBox(width: 6),
            Expanded(
              // (joined: who does it: ThorVG, for the six the inside has; the sample's analysis says nothing of the document)
              child: Text(c.ws.inside == null ? f.desc : (fxValues.containsKey(f.name) ? 'ThorVG' : 'No seat yet'), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g56))),
            ),
            if (trying) ...[DockTag('TRY', hue: sel ? null : WsT.accent), const SizedBox(width: 2)],
            if (on) ...[DockTag('ON', hue: sel ? null : WsT.accent), const SizedBox(width: 2)],
            if (star || h || sel) DockStar(on: star, ink: sel ? WsT.onAccent : null, onTap: () => c.star(f.name)) else const SizedBox(width: 20),
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
      height: 24,
      padding: const EdgeInsets.only(left: 22, right: 4),
      color: dockFill(false, h),
      child: Row(
        children: [
          Expanded(child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(dockInk(false)).copyWith(fontWeight: FontWeight.w500))),
          if (on >= 0) ...[DockTag(of == 'screen' ? 'ON ${on + 1}' : 'ON', hue: WsT.accent), const SizedBox(width: 2)],
        ],
      ),
    ),
  );
}

/// Joined: the document's value an effect is, and what the picked thing has for it now.
String _said(Ws ws, String name) {
  final v = fxValues[name]?.$1, t = int.tryParse(ws.selected ?? ''), m = ws.inside!;
  if (v == null) return 'No seat yet';
  final now = t == null ? '' : m.word(t, v);
  return now.isEmpty ? '$v · none on it' : '$v · $now';
}

/// The foot: the picked effect, what the analysis says about it, and Add (keeps the try-on, or adds straight away).
class _FxFooter extends StatelessWidget {
  const _FxFooter(this.f, {required this.ws, required this.done});
  final FxDef f;
  final Ws ws;
  final bool done;
  @override
  Widget build(BuildContext context) => DockFooter(
    title: f.name,
    meta: ws.inside == null ? [f.family.label, f.seat, f.time, f.inputs, ...f.looks].join(' · ') : _said(ws, f.name),
    // (joined: an effect the inside has goes on or comes off the picked thing; the others have no seat yet)
    action: ws.inside != null && done ? 'Remove' : 'Add',
    done: ws.inside != null ? !fxValues.containsKey(f.name) || ws.selected == null : done,
    doneLabel: ws.inside != null ? (fxValues.containsKey(f.name) ? 'Pick a thing' : 'No seat yet') : 'Added',
    onAction: ws.inside != null
        ? () => ws.fxToggle(f.name)
        : ws.canTry
        ? () {
            ws.tryFx(f.name);
            ws.keepFx();
          }
        : null,
  );
}
