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
              child: Text(f.desc, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g56))),
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

/// The foot: the picked effect, what the analysis says about it, and Add (keeps the try-on, or adds straight away).
class _FxFooter extends StatelessWidget {
  const _FxFooter(this.f, {required this.ws, required this.done});
  final FxDef f;
  final Ws ws;
  final bool done;
  @override
  Widget build(BuildContext context) => DockFooter(
    title: f.name,
    meta: [f.family.label, f.seat, f.time, f.inputs, ...f.looks].join(' · '),
    action: 'Add',
    done: done,
    onAction: ws.canTry
        ? () {
            ws.tryFx(f.name);
            ws.keepFx();
          }
        : null,
  );
}
