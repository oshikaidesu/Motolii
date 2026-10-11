// The browser's media & assets. Direction from concept/browser-panels-concept.png: the work (thumbnails) is the brightest thing,
// the chrome is grey; one selection colour (C.mode); hover is a grey step; 120ms ease-out.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'media_parts.dart';

// [open] tile radius 6 / row radius 4 are from the brief; the sizes below are not in any token.
const _rowH = 36.0; // [open] list row height
const _rowHCompact = 28.0; // [open] compact list row height
const _chipH = 22.0; // [open] source chip height
const _barH = 28.0; // [open] search bar height
const _presetH = 40.0; // [open] preset row height
const _fontH = 56.0; // [open] font row height
const _ghostW = 96.0; // [open] drag ghost width
const _gridGap = 8.0; // [open] default grid gap (Browser panel uses 8)

Asset _byKind(Kind k) => sampleAssets.firstWhere((a) => a.kind == k);

WidgetbookComponent mediaSetSet() => WidgetbookComponent(name: 'media_set', useCases: [
      uc('Asset tile', _tileCase, width: 160, height: 160),
      uc('Asset list row', _rowCase, width: 520, height: 5 * _rowH + 8),
      uc('Thumbnail grid / masonry', _gridCase, width: 520, height: 440),
      uc('Source chips', _sourceCase, width: 440),
      uc('Type filter strip', _typeCase, width: 440),
      uc('Search + filter bar', _searchCase, width: 440),
      uc('Folder tree', _treeCase, width: 260),
      uc('Preset row', _presetCase, width: 360),
      uc('Font row', _fontCase, width: 560),
      uc('Colour palette strip', _paletteCase, width: 360),
      uc('Drag ghost', _dragCase, width: 520, height: 280),
      uc('Empty state', _emptyCase, width: 320, height: 240),
      uc('Import drop zone', _dropCase, width: 360, height: 140),
      uc('Asset details card', _detailsCase, width: 280),
    ]);

// ---------------------------------------------------------------- asset tile

class _Tile extends StatelessWidget {
  const _Tile(this.a, {required this.width, this.aspect = .625, this.selected = false, this.look = Look.concept, this.onTap});
  final Asset a;
  final double width, aspect;
  final bool selected;
  final Look look;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Press(
        onTap: onTap,
        builder: (context, hover, focus, pos) {
          final h = width * aspect;
          final scrub = (a.kind.timed && a.dur > 0 && hover && pos != null) ? (pos.dx / width).clamp(0.0, 1.0) : null;
          final ring = selected ? Role.selected : (hover || focus ? N.g56 : N.g20);
          final overlayName = look == Look.quiet;
          return SizedBox(
            width: width,
            child: Flex(direction: Axis.vertical, crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
              Stack(clipBehavior: Clip.none, children: [
                AnimatedContainer(
                  duration: kFast,
                  curve: kEase,
                  width: width,
                  height: h,
                  foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: ring)),
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(6),
                    child: Stack(fit: StackFit.expand, children: [
                      Art(a),
                      if (overlayName) Positioned(left: 0, right: 0, bottom: 0, height: 32, child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.bottomCenter, end: Alignment.topCenter, colors: [N.g00.withValues(alpha: .75), N.g00.withValues(alpha: 0)])))),
                      if (a.kind != Kind.image) Positioned(left: 4, top: 4, child: TypeBadge(a.kind)),
                      if (overlayName) Positioned(left: 6, bottom: 6, right: a.dur > 0 ? 48 : 6, child: Text(a.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g95))),
                      if (a.dur > 0 || scrub != null) Positioned(right: 4, bottom: 4, child: TimePill(scrub != null ? clock(scrub * a.dur) : clock(a.dur))),
                      if (scrub != null) Positioned(left: pos!.dx.clamp(0.0, width - 1), top: 0, bottom: 0, width: 1, child: const ColoredBox(color: N.g100)),
                    ]),
                  ),
                ),
                if (look == Look.glow && selected) Positioned(left: -3, top: -3, right: -3, bottom: -3, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(9), border: Border.all(color: Role.selected.withValues(alpha: .35), width: 2))))),
              ]),
              if (!overlayName) ...[
                const SizedBox(height: 6),
                Text(a.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91)),
                const SizedBox(height: 3),
                Text('${a.kind.label} · ${a.dims}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56)),
              ],
            ]),
          );
        },
      );
}

Kind _kindKnob(BuildContext c, [Kind init = Kind.video]) => c.knobs.object.dropdown<Kind>(label: 'Type', options: Kind.values, initialOption: init, labelBuilder: (k) => k.label);

Widget _tileCase(BuildContext c) {
  final base = _byKind(_kindKnob(c));
  final dur = c.knobs.int.slider(label: 'Duration (s)', initialValue: base.dur.round(), min: 0, max: 600);
  final a = Asset(c.knobs.string(label: 'Name', initialValue: base.name), base.kind, c.knobs.int.slider(label: 'Art seed', initialValue: base.seed, min: 0, max: 30), style: base.style, dur: dur.toDouble(), dims: base.dims);
  final selected = c.knobs.boolean(label: 'Selected', initialValue: false);
  return _Tile(a, width: c.knobs.double.slider(label: 'Width', initialValue: 128, min: 72, max: 160), selected: selected, look: lookKnob(c));
}

// ------------------------------------------------------------- asset list row

class _Row extends StatelessWidget {
  const _Row(this.a, {this.selected = false, this.compact = false, this.showTags = true, this.showMeta = true, this.meta, this.onTap});
  final Asset a;
  final String? meta;
  final bool selected, compact, showTags, showMeta;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Press(
        onTap: onTap,
        builder: (context, hover, focus, _) {
          final h = compact ? _rowHCompact : _rowH, th = h - 8;
          return AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: h,
            padding: const EdgeInsets.symmetric(horizontal: 6),
            decoration: BoxDecoration(color: selected ? N.g20 : (hover ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))),
            child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Tick(selected, height: th),
              SizedBox(width: th, height: th, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: Art(a))),
              const SizedBox(width: 8),
              Expanded(flex: 3, child: Text(a.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91))),
              if (showMeta) SizedBox(width: meta == null ? 92 : 120, child: Text(meta ?? a.dims, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.value(N.g56).copyWith(fontSize: 10))),
              if (showTags)
                Expanded(flex: 2, child: ClipRect(child: SingleChildScrollView(scrollDirection: Axis.horizontal, physics: const NeverScrollableScrollPhysics(), child: Flex(direction: Axis.horizontal, children: [for (final t in a.tags) Padding(padding: const EdgeInsets.only(right: 4), child: Pill(t, fill: selected ? N.g26 : N.g20))])))),
              SizedBox(width: 40, child: Text(a.dur > 0 ? clock(a.dur) : a.size, textAlign: TextAlign.right, style: T.value(N.g56).copyWith(fontSize: 10))),
            ]),
          );
        },
      );
}

Widget _rowCase(BuildContext c) {
  final compact = c.knobs.boolean(label: 'Compact', initialValue: false);
  final tags = c.knobs.boolean(label: 'Show tags', initialValue: true);
  final meta = c.knobs.boolean(label: 'Show meta', initialValue: true);
  return Hold<int>(
    initial: 1,
    builder: (context, sel, set) => Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
      for (var i = 0; i < 5; i++) _Row(sampleAssets[i], selected: i == sel, compact: compact, showTags: tags, showMeta: meta, onTap: () => set(i)),
    ]),
  );
}

// ------------------------------------------------------- grid & masonry

enum _Layout { grid, masonry }

Widget _gridCase(BuildContext c) {
  final layout = c.knobs.object.dropdown<_Layout>(label: 'Layout', options: _Layout.values, initialOption: _Layout.masonry, labelBuilder: (l) => l.name);
  final cols = c.knobs.int.slider(label: 'Columns', initialValue: 4, min: 2, max: 6);
  final gap = c.knobs.double.slider(label: 'Gap', initialValue: _gridGap, min: 4, max: 16);
  final look = lookKnob(c);
  final items = [...sampleAssets, ...sampleAssets.reversed];
  return Hold<Set<int>>(
    initial: {1, 6},
    builder: (context, sel, set) {
      void toggle(int i) => set(sel.contains(i) ? ({...sel}..remove(i)) : {...sel, i});
      return Padding(padding: const EdgeInsets.fromLTRB(4, 4, 4, 0), child: LayoutBuilder(builder: (context, box) {
        final w = ((box.maxWidth - gap * (cols - 1)) / cols).floorToDouble();
        final Widget body;
        if (layout == _Layout.grid) {
          body = Wrap(spacing: gap, runSpacing: gap, children: [for (var i = 0; i < items.length; i++) _Tile(items[i], width: w, look: look, selected: sel.contains(i), onTap: () => toggle(i))]);
        } else {
          final columns = List.generate(cols, (_) => <int>[]), heights = List.filled(cols, 0.0);
          for (var i = 0; i < items.length; i++) {
            var m = 0;
            for (var k = 1; k < cols; k++) {
              if (heights[k] < heights[m]) m = k;
            }
            columns[m].add(i);
            heights[m] += items[i].aspect + (look == Look.quiet ? 0 : .3);
          }
          body = Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.start, children: [
            for (var k = 0; k < cols; k++) ...[
              if (k > 0) SizedBox(width: gap),
              SizedBox(width: w, child: Flex(direction: Axis.vertical, children: [
                for (final i in columns[k]) Padding(padding: EdgeInsets.only(bottom: gap), child: _Tile(items[i], width: w, aspect: items[i].aspect, look: look, selected: sel.contains(i), onTap: () => toggle(i))),
              ])),
            ],
          ]);
        }
        return Stack(children: [
          Positioned.fill(child: SingleChildScrollView(padding: const EdgeInsets.only(bottom: 24), child: body)),
          Positioned(left: 0, right: 0, bottom: 0, height: 24, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.bottomCenter, end: Alignment.topCenter, colors: [N.g07, N.g07.withValues(alpha: 0)]))))),
        ]);
      }));
    },
  );
}

// ------------------------------------------------------- filter chips & strip

class _Chip extends StatelessWidget {
  const _Chip(this.label, {this.count, this.selected = false, this.look = Look.concept, this.onTap});
  final String label;
  final int? count;
  final bool selected;
  final Look look;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => Press(
        onTap: onTap,
        builder: (context, hover, focus, _) {
          final bg = selected ? N.g20 : (hover ? N.g15 : (look == Look.quiet ? const Color(0x00000000) : N.g13));
          return AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: _chipH,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(color: bg, borderRadius: BorderRadius.circular(look == Look.glow ? 11 : 4), border: Border.all(color: focus ? N.g56 : (look == Look.quiet ? const Color(0x00000000) : N.g20))),
            child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Text(label, style: T.label(selected ? N.g95 : N.g76).copyWith(fontWeight: FontWeight.w500)),
              if (count != null) Padding(padding: const EdgeInsets.only(left: 6), child: Text('$count', style: T.value(N.g56).copyWith(fontSize: 10))),
            ]),
          );
        },
      );
}

Widget _sourceCase(BuildContext c) {
  final multi = c.knobs.boolean(label: 'Multi-select', initialValue: false);
  final counts = c.knobs.boolean(label: 'Show counts', initialValue: true);
  final look = lookKnob(c);
  const names = ['This project', 'Bundled', 'Favorites', 'Recent'], n = [42, 318, 9, 24];
  return Hold<Set<int>>(
    initial: {0},
    builder: (context, sel, set) => Wrap(spacing: 4, runSpacing: 4, children: [
      for (var i = 0; i < names.length; i++)
        _Chip(names[i], count: counts ? n[i] : null, look: look, selected: sel.contains(i), onTap: () {
          if (multi) {
            final next = sel.contains(i) ? ({...sel}..remove(i)) : {...sel, i};
            set(next.isEmpty ? {i} : next);
          } else {
            set({i});
          }
        }),
    ]),
  );
}

Widget _typeCase(BuildContext c) {
  final counts = c.knobs.boolean(label: 'Show counts', initialValue: true);
  final look = lookKnob(c);
  final labels = ['All', for (final k in Kind.values) k.label];
  int count(int i) => i == 0 ? sampleAssets.length : sampleAssets.where((a) => a.kind == Kind.values[i - 1]).length;
  return Hold<int>(
    initial: 0,
    builder: (context, sel, set) {
      final items = [
        for (var i = 0; i < labels.length; i++)
          Press(
            onTap: () => set(i),
            builder: (context, hover, focus, _) {
              final on = i == sel;
              final txt = Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
                Text(labels[i], style: T.label(on ? N.g95 : (hover ? N.g91 : N.g63)).copyWith(fontWeight: FontWeight.w500)),
                if (counts) Padding(padding: const EdgeInsets.only(left: 4), child: Text('${count(i)}', style: T.value(N.g44).copyWith(fontSize: 10))),
              ]);
              return AnimatedContainer(duration: kFast, curve: kEase, height: 22, padding: const EdgeInsets.symmetric(horizontal: 10), alignment: Alignment.center, decoration: BoxDecoration(color: on ? N.g20 : (hover ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))), child: txt);
            },
          ),
      ];
      final row = Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: items);
      return look == Look.glow ? Container(padding: const EdgeInsets.all(2), decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6)), child: row) : (look == Look.concept ? Container(padding: const EdgeInsets.only(bottom: 4), decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: N.g20))), child: row) : row);
    },
  );
}

// ------------------------------------------------------- search + filter bar

class _SearchBar extends StatefulWidget {
  const _SearchBar({required this.library, required this.filters, required this.preview});
  final int library, filters;
  final bool preview;
  @override
  State<_SearchBar> createState() => _SearchBarState();
}

class _SearchBarState extends State<_SearchBar> {
  final _ctl = TextEditingController(text: 'loop');
  final _focus = FocusNode();
  bool _hover = false;

  @override
  void initState() {
    super.initState();
    _focus.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _ctl.dispose();
    _focus.dispose();
    super.dispose();
  }

  List<Asset> get _hits {
    final q = _ctl.text.trim().toLowerCase();
    return [for (final a in sampleAssets) if (q.isEmpty || a.name.toLowerCase().contains(q) || a.tags.any((t) => t.contains(q))) a];
  }

  @override
  Widget build(BuildContext context) {
    final hits = _hits;
    final shown = widget.library > sampleAssets.length ? (hits.length * widget.library / sampleAssets.length).round() : hits.length;
    return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      MouseRegion(
        onEnter: (_) => setState(() => _hover = true),
        onExit: (_) => setState(() => _hover = false),
        child: AnimatedContainer(
          duration: kFast,
          curve: kEase,
          height: _barH,
          padding: const EdgeInsets.only(left: 8, right: 4),
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: _focus.hasFocus ? N.g56 : (_hover ? N.g26 : N.g20))),
          child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
            glyph(12, (cv, s) {
              final p = Paint()..color = N.g56..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round;
              cv.drawCircle(Offset(s.width * .42, s.height * .42), s.width * .3, p);
              cv.drawLine(Offset(s.width * .65, s.height * .65), Offset(s.width * .92, s.height * .92), p);
            }),
            const SizedBox(width: 8),
            Expanded(child: Stack(alignment: Alignment.centerLeft, children: [
              if (_ctl.text.isEmpty) IgnorePointer(child: Text('Search media, tags', style: T.name(N.g44))),
              EditableText(controller: _ctl, focusNode: _focus, style: T.name(N.g95), cursorColor: N.g95, backgroundCursorColor: N.g20, selectionColor: Role.selected.withValues(alpha: .4), maxLines: 1, onChanged: (_) => setState(() {})),
            ])),
            if (_ctl.text.isNotEmpty)
              Press(
                onTap: () => setState(_ctl.clear),
                builder: (context, hover, focus, _) => SizedBox(width: 20, height: 20, child: Center(child: glyph(8, (cv, s) {
                  final p = Paint()..color = hover || focus ? N.g95 : N.g56..strokeWidth = 1.3..strokeCap = StrokeCap.round;
                  cv.drawLine(Offset.zero, Offset(s.width, s.height), p);
                  cv.drawLine(Offset(s.width, 0), Offset(0, s.height), p);
                }, [hover, focus]))),
              ),
            const SizedBox(width: 4),
            Text(hits.isEmpty ? 'No results' : '$shown ${shown == 1 ? 'result' : 'results'}', style: T.value(hits.isEmpty ? N.g44 : N.g56).copyWith(fontSize: 10)),
            Container(width: 1, height: 14, margin: const EdgeInsets.symmetric(horizontal: 6), color: N.g20),
            Press(
              builder: (context, hover, focus, _) => AnimatedContainer(
                duration: kFast,
                curve: kEase,
                height: 20,
                padding: const EdgeInsets.symmetric(horizontal: 6),
                alignment: Alignment.center,
                decoration: BoxDecoration(color: hover ? N.g20 : (widget.filters > 0 ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(3), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))),
                child: Flex(direction: Axis.horizontal, mainAxisSize: MainAxisSize.min, children: [
                  glyph(10, (cv, s) {
                    final p = Paint()..color = widget.filters > 0 ? N.g95 : N.g63..strokeWidth = 1.2..strokeCap = StrokeCap.round;
                    for (var i = 0; i < 3; i++) {
                      final y = s.height * (.2 + i * .3), x = s.width * (.1 + .25 * i);
                      cv.drawLine(Offset(0, y), Offset(s.width, y), p);
                      cv.drawCircle(Offset(s.width - x, y), 1.6, Paint()..color = N.g15);
                      cv.drawCircle(Offset(s.width - x, y), 1.6, p..style = PaintingStyle.stroke);
                    }
                  }, widget.filters),
                  if (widget.filters > 0) Padding(padding: const EdgeInsets.only(left: 4), child: Text('${widget.filters}', style: T.value(N.g95).copyWith(fontSize: 10))),
                ]),
              ),
            ),
          ]),
        ),
      ),
      if (widget.preview) ...[
        const SizedBox(height: 8),
        if (hits.isEmpty)
          Text('Nothing matches "${_ctl.text}"', style: T.label(N.g56))
        else
          for (final a in hits.take(5)) _Row(a, compact: true, showTags: false, meta: '${a.kind.label} · ${a.size}'),
      ],
    ]);
  }
}

Widget _searchCase(BuildContext c) => _SearchBar(
      library: c.knobs.int.slider(label: 'Library size', initialValue: 12, min: 12, max: 2000),
      filters: c.knobs.int.slider(label: 'Active filters', initialValue: 1, min: 0, max: 4),
      preview: c.knobs.boolean(label: 'Show result thumbs', initialValue: true),
    );

// ------------------------------------------------------------------ folder tree

class _Node {
  const _Node(this.name, this.count, [this.kids = const []]);
  final String name;
  final int count;
  final List<_Node> kids;
}

const _tree = [
  _Node('Project', 142, [
    _Node('Footage', 58, [_Node('Day 1', 24), _Node('Day 2', 34)]),
    _Node('Audio', 31, [_Node('Music', 12), _Node('Foley', 19)]),
    _Node('Textures', 40),
    _Node('3D', 13),
  ]),
  _Node('Bundled', 318, [_Node('Skies', 64), _Node('Gradients', 120)]),
  _Node('Favorites', 9),
];

Widget _treeCase(BuildContext c) {
  final indent = c.knobs.double.slider(label: 'Indent', initialValue: 14, min: 10, max: 22);
  final rowH = c.knobs.double.slider(label: 'Row height', initialValue: 24, min: 20, max: 32);
  final counts = c.knobs.boolean(label: 'Show counts', initialValue: true);
  final guides = c.knobs.boolean(label: 'Indent guides', initialValue: true);
  return Hold<({Set<String> open, String sel})>(
    initial: (open: {'Project', 'Project/Footage'}, sel: 'Project/Footage/Day 1'),
    builder: (context, st, set) {
      final out = <Widget>[];
      void walk(List<_Node> ns, int depth, String parent) {
        for (final n in ns) {
          final path = parent.isEmpty ? n.name : '$parent/${n.name}';
          final open = st.open.contains(path), sel = st.sel == path, hasKids = n.kids.isNotEmpty;
          out.add(Press(
            onTap: () => set((open: hasKids && sel ? ({...st.open}..toggle(path)) : st.open, sel: path)),
            builder: (context, hover, focus, _) => AnimatedContainer(
              duration: kFast,
              curve: kEase,
              height: rowH,
              padding: const EdgeInsets.only(left: 4, right: 8),
              decoration: BoxDecoration(color: sel ? N.g20 : (hover ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))),
              child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
                Tick(sel, height: rowH - 10),
                for (var d = 0; d < depth; d++) SizedBox(width: indent, height: rowH, child: Center(child: Container(width: 1, color: guides ? N.g20 : const Color(0x00000000)))),
                Press(
                  onTap: hasKids ? () => set((open: {...st.open}..toggle(path), sel: st.sel)) : null,
                  builder: (context, h2, f2, _) => SizedBox(width: 16, height: rowH, child: hasKids ? Center(child: AnimatedRotation(turns: open ? .25 : 0, duration: kFast, curve: kEase, child: glyph(8, (cv, s) {
                    final p = Path()..moveTo(s.width * .3, 0)..lineTo(s.width * .8, s.height / 2)..lineTo(s.width * .3, s.height);
                    cv.drawPath(p, Paint()..color = h2 ? N.g95 : N.g56..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round);
                  }, h2))) : null),
                ),
                glyph(14, (cv, s) => drawFolder(cv, s, sel ? N.g95 : N.g56, fill: sel), sel),
                const SizedBox(width: 6),
                Expanded(child: Text(n.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(sel ? N.g100 : N.g91))),
                if (counts) Text('${n.count}', style: T.value(N.g56).copyWith(fontSize: 10)),
              ]),
            ),
          ));
          if (open && hasKids) walk(n.kids, depth + 1, path);
        }
      }

      walk(_tree, 0, '');
      return Container(padding: const EdgeInsets.all(4), decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)), child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: out));
    },
  );
}

extension on Set<String> {
  void toggle(String v) => contains(v) ? remove(v) : add(v);
}

// ------------------------------------------------------------------ preset row

enum _Motion { bounce, fade, spin, pop }

class _PresetRow extends StatefulWidget {
  const _PresetRow({required this.name, required this.meta, required this.fam, required this.motion, required this.always, required this.fav, required this.onFav, required this.selected, required this.onTap});
  final String name, meta;
  final Fam fam;
  final _Motion motion;
  final bool always, fav, selected;
  final ValueChanged<bool> onFav;
  final VoidCallback onTap;
  @override
  State<_PresetRow> createState() => _PresetRowState();
}

class _PresetRowState extends State<_PresetRow> with SingleTickerProviderStateMixin {
  static const _rest = .62;
  late final _ctl = AnimationController(vsync: this, duration: const Duration(milliseconds: 1400), value: _rest);
  bool _hover = false;

  void _sync() {
    if (widget.always || _hover) {
      if (!_ctl.isAnimating) _ctl.repeat();
    } else {
      _ctl.stop();
      _ctl.value = _rest;
    }
  }

  @override
  void initState() {
    super.initState();
    _sync();
  }

  @override
  void didUpdateWidget(_PresetRow o) {
    super.didUpdateWidget(o);
    _sync();
  }

  @override
  void dispose() {
    _ctl.dispose();
    super.dispose();
  }

  void _paint(Canvas cv, Size s, double t) {
    final col = widget.fam.c;
    final fill = Paint()..color = col;
    switch (widget.motion) {
      case _Motion.bounce:
        final x = 8 + (s.width - 16) * (.5 - .5 * math.cos(2 * math.pi * Curves.easeInOut.transform(t)));
        cv.drawCircle(Offset(x, s.height / 2), 5, fill);
      case _Motion.fade:
        final a = .15 + .85 * (.5 - .5 * math.cos(2 * math.pi * t));
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: s.center(Offset.zero), width: 18, height: 12), const Radius.circular(3)), Paint()..color = col.withValues(alpha: a));
      case _Motion.spin:
        cv.save();
        cv.translate(s.width / 2, s.height / 2);
        cv.rotate(2 * math.pi * Curves.easeInOut.transform(t));
        cv.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 20, height: 6), const Radius.circular(3)), fill);
        cv.restore();
      case _Motion.pop:
        final k = t < .5 ? Curves.easeOutBack.transform(t * 2) : 1 - Curves.easeIn.transform((t - .5) * 2);
        cv.drawCircle(s.center(Offset.zero), 2 + 8 * k.clamp(0.0, 1.3), fill);
    }
  }

  @override
  Widget build(BuildContext context) => MouseRegion(
        onEnter: (_) => setState(() {
          _hover = true;
          _sync();
        }),
        onExit: (_) => setState(() {
          _hover = false;
          _sync();
        }),
        child: Press(
          onTap: widget.onTap,
          builder: (context, hover, focus, _) => AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: _presetH,
            padding: const EdgeInsets.fromLTRB(6, 0, 2, 0),
            decoration: BoxDecoration(color: widget.selected ? N.g20 : (hover ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))),
            child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Tick(widget.selected, height: 24),
              Container(
                width: 56,
                height: 28,
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
                child: AnimatedBuilder(animation: _ctl, builder: (context, _) => CustomPaint(painter: Draw((cv, s) => _paint(cv, s, _ctl.value), _ctl.value))),
              ),
              const SizedBox(width: 10),
              Expanded(child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(widget.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(widget.selected ? N.g95 : N.g91)),
                const SizedBox(height: 4),
                Text(widget.meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56)),
              ])),
              StarButton(on: widget.fav, onChanged: widget.onFav),
            ]),
          ),
        ),
      );
}

Widget _presetCase(BuildContext c) {
  final fam = c.knobs.object.dropdown<Fam>(label: 'Family', options: Fam.all, initialOption: Fam.stagger, labelBuilder: (f) => f.name);
  final motion = c.knobs.object.dropdown<_Motion>(label: 'Motion', options: _Motion.values, initialOption: _Motion.bounce, labelBuilder: (m) => m.name);
  final always = c.knobs.boolean(label: 'Always animate', initialValue: false);
  final name = c.knobs.string(label: 'Name', initialValue: 'Soft bounce');
  return Hold<({int sel, Set<int> fav})>(
    initial: (sel: 0, fav: {0}),
    builder: (context, st, set) {
      final specs = [(name, '${fam.name} · 0.6 s', fam, motion), ('Slow fade', 'Stagger · 1.2 s', Fam.stagger, _Motion.fade), ('Whip spin', 'Face · 0.4 s', Fam.face, _Motion.spin), ('Pop in', 'Scatter · 0.3 s', Fam.scatter, _Motion.pop)];
      return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < specs.length; i++)
          _PresetRow(
            name: specs[i].$1,
            meta: specs[i].$2,
            fam: specs[i].$3,
            motion: specs[i].$4,
            always: always,
            fav: st.fav.contains(i),
            selected: st.sel == i,
            onTap: () => set((sel: i, fav: st.fav)),
            onFav: (v) => set((sel: st.sel, fav: v ? {...st.fav, i} : ({...st.fav}..remove(i)))),
          ),
      ]);
    },
  );
}

// ------------------------------------------------------------------- font row

class _FontRow extends StatelessWidget {
  const _FontRow({required this.family, required this.meta, required this.sample, required this.size, required this.selected, required this.onTap});
  final String family, meta, sample;
  final double size;
  final bool selected;
  final VoidCallback onTap;

  TextStyle _style(double s, Color col) => TextStyle(fontFamily: family, fontSize: s, color: T.hidden.value ? const Color(0x00000000) : col, height: 1.1, decoration: TextDecoration.none);

  @override
  Widget build(BuildContext context) => Hold<({double size, bool fav})>(
        key: ValueKey(size),
        initial: (size: size, fav: false),
        builder: (context, st, set) => Press(
          onTap: onTap,
          builder: (context, hover, focus, _) => AnimatedContainer(
            duration: kFast,
            curve: kEase,
            height: _fontH,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(color: selected ? N.g20 : (hover ? N.g15 : const Color(0x00000000)), borderRadius: BorderRadius.circular(4), border: Border.all(color: focus ? N.g56 : const Color(0x00000000))),
            child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Tick(selected, height: 32),
              Container(width: 40, height: 40, alignment: Alignment.center, decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)), child: Text('Aa', style: _style(22, N.g95))),
              const SizedBox(width: 12),
              SizedBox(width: 108, child: Flex(direction: Axis.vertical, mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text(family, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(selected ? N.g95 : N.g91)),
                const SizedBox(height: 4),
                Text(meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g56)),
              ])),
              Expanded(child: ClipRect(child: Text(sample, maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: _style(st.size, N.g95)))),
              const SizedBox(width: 12),
              MiniSlider(value: st.size, min: 10, max: 32, width: 72, track: selected ? N.g38 : N.g26, onChanged: (v) => set((size: v, fav: st.fav))),
              SizedBox(width: 28, child: Text('${st.size.round()}', textAlign: TextAlign.right, style: T.value(N.g56).copyWith(fontSize: 10))),
              StarButton(on: st.fav, onChanged: (v) => set((size: st.size, fav: v))),
            ]),
          ),
        ),
      );
}

Widget _fontCase(BuildContext c) {
  const fams = ['Inter', 'Menlo', 'Georgia', 'Courier New', 'Times New Roman', 'Helvetica Neue'];
  final fam = c.knobs.object.dropdown<String>(label: 'Family', options: fams, initialOption: 'Inter', labelBuilder: (s) => s);
  final sample = c.knobs.string(label: 'Specimen', initialValue: 'Quick brown fox jumps');
  final size = c.knobs.double.slider(label: 'Initial size', initialValue: 16, min: 10, max: 32);
  return Hold<int>(
    initial: 0,
    builder: (context, sel, set) {
      final rows = [(fam, 'Sans · 18 styles'), ('Georgia', 'Serif · 8 styles'), ('Menlo', 'Mono · 4 styles')];
      return Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
        for (var i = 0; i < rows.length; i++) _FontRow(family: rows[i].$1, meta: rows[i].$2, sample: sample, size: size, selected: sel == i, onTap: () => set(i)),
      ]);
    },
  );
}

// -------------------------------------------------------------- palette strip

Widget _paletteCase(BuildContext c) {
  final look = lookKnob(c);
  final size = c.knobs.double.slider(label: 'Swatch size', initialValue: 24, min: 16, max: 36);
  final gap = c.knobs.double.slider(label: 'Gap', initialValue: 4, min: 4, max: 12);
  final title = c.knobs.string(label: 'Title', initialValue: 'Used here');
  final gradients = c.knobs.boolean(label: 'Show gradients', initialValue: true);
  final colours = [for (final f in Fam.all) f.c, N.g95, N.g56, N.g20];
  return Hold<int>(
    initial: 0,
    builder: (context, sel, set) => Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
      Flex(direction: Axis.horizontal, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
        Text(title.toUpperCase(), style: T.micro(N.g56).copyWith(letterSpacing: .8)),
        Text(hexOf(colours[sel]), style: T.value(N.g76).copyWith(fontSize: 10)),
      ]),
      const SizedBox(height: 8),
      Wrap(spacing: gap, runSpacing: gap, children: [
        for (var i = 0; i < colours.length; i++)
          Press(
            onTap: () => set(i),
            builder: (context, hover, focus, _) => AnimatedContainer(
              duration: kFast,
              curve: kEase,
              padding: const EdgeInsets.all(2),
              decoration: BoxDecoration(borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 6), border: Border.all(color: i == sel ? N.g95 : (hover || focus ? N.g56 : const Color(0x00000000)))),
              child: Container(width: size, height: look == Look.glow ? size * 1.5 : size, decoration: BoxDecoration(color: colours[i], borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 4))),
            ),
          ),
        Press(
          builder: (context, hover, focus, _) => Padding(
            padding: const EdgeInsets.all(2),
            child: SizedBox(width: size, height: look == Look.glow ? size * 1.5 : size, child: DecoratedBox(decoration: BoxDecoration(color: hover ? N.g15 : const Color(0x00000000), borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 4), border: Border.all(color: focus ? N.g56 : N.g20)), child: Center(child: glyph(10, (cv, s) {
              final p = Paint()..color = hover ? N.g95 : N.g63..strokeWidth = 1.3..strokeCap = StrokeCap.round;
              cv.drawLine(Offset(0, s.height / 2), Offset(s.width, s.height / 2), p);
              cv.drawLine(Offset(s.width / 2, 0), Offset(s.width / 2, s.height), p);
            }, hover)))),
          ),
        ),
      ]),
      if (gradients) ...[
        const SizedBox(height: 14),
        Text('GRADIENTS', style: T.micro(N.g56).copyWith(letterSpacing: .8)),
        const SizedBox(height: 8),
        Wrap(spacing: gap, runSpacing: gap, children: [
          for (var i = 0; i < 2; i++) Container(width: size * 2, height: size * .7, decoration: BoxDecoration(borderRadius: BorderRadius.circular(look == Look.quiet ? 99 : 4), gradient: LinearGradient(colors: [Fam.all[i].c, Fam.all[(i + 2) % 6].c]))),
        ]),
      ],
    ]),
  );
}

// ------------------------------------------------------------------ drag ghost

class _DragScene extends StatefulWidget {
  const _DragScene({required this.count, required this.accepts, required this.opacity});
  final int count;
  final bool accepts;
  final double opacity;
  @override
  State<_DragScene> createState() => _DragSceneState();
}

class _DragSceneState extends State<_DragScene> {
  static const _slotX = 300.0, _slotW = 196.0, _slotH = 56.0, _slotY = 24.0, _slotGap = 16.0;
  Offset _pos = const Offset(440, 140);

  Rect _slot(int i) => Rect.fromLTWH(_slotX, _slotY + i * (_slotH + _slotGap), _slotW, _slotH);
  int? get _over {
    for (var i = 0; i < 3; i++) {
      if (_slot(i).contains(_pos)) return i;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final over = _over;
    final a = sampleAssets[1];
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onPanDown: (d) => setState(() => _pos = d.localPosition),
      onPanUpdate: (d) => setState(() => _pos = d.localPosition),
      child: MouseRegion(
        cursor: SystemMouseCursors.grabbing,
        child: Stack(clipBehavior: Clip.none, children: [
          Positioned.fill(child: DecoratedBox(decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)))),
          Positioned(left: 24, top: 24, child: Opacity(opacity: .4, child: _Tile(a, width: _ghostW))),
          Positioned(left: 24, top: 130, child: _Tile(sampleAssets[0], width: _ghostW)),
          Positioned(left: 144, top: 24, child: _Tile(sampleAssets[5], width: _ghostW)),
          for (var i = 0; i < 3; i++)
            Positioned.fromRect(
              rect: _slot(i),
              child: AnimatedContainer(
                duration: kFast,
                curve: kEase,
                alignment: Alignment.topLeft,
                padding: const EdgeInsets.fromLTRB(12, 10, 0, 0),
                decoration: BoxDecoration(color: over == i && widget.accepts ? Role.selected.withValues(alpha: .12) : N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: over == i ? (widget.accepts ? Role.selected : N.g44) : N.g20)),
                child: Text(over == i ? (widget.accepts ? 'Drop to add to Layer ${i + 1}' : 'Layer ${i + 1} is locked') : 'Layer ${i + 1}', style: T.label(over == i && widget.accepts ? N.g95 : N.g56)),
              ),
            ),
          Positioned(left: 24, bottom: 16, child: Text('Press and drag anywhere', style: T.label(N.g44))),
          Positioned(
            left: _pos.dx + 8,
            top: _pos.dy + 8,
            child: IgnorePointer(
              child: Opacity(
                opacity: widget.opacity,
                child: Transform.rotate(
                  angle: -.04,
                  child: Stack(clipBehavior: Clip.none, children: [
                    if (widget.count > 2) Positioned(left: 8, top: 8, child: _ghostCard(a, _ghostW, flat: true)),
                    if (widget.count > 1) Positioned(left: 4, top: 4, child: _ghostCard(a, _ghostW, flat: true)),
                    _ghostCard(a, _ghostW),
                    if (widget.count > 1) Positioned(right: -8, top: -8, child: Container(constraints: const BoxConstraints(minWidth: 18), height: 18, padding: const EdgeInsets.symmetric(horizontal: 5), alignment: Alignment.center, decoration: BoxDecoration(color: C.mode, borderRadius: BorderRadius.circular(9)), child: Text('${widget.count}', style: T.value(N.g100).copyWith(fontSize: 10)))),
                  ]),
                ),
              ),
            ),
          ),
        ]),
      ),
    );
  }
}

Widget _ghostCard(Asset a, double w, {bool flat = false}) => Container(
      width: w,
      height: w * .625,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: flat ? N.g38 : N.g63),
        color: N.g13,
        boxShadow: flat ? null : [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))], // floating: the one shadow
      ),
      child: ClipRRect(borderRadius: BorderRadius.circular(5), child: flat ? null : Stack(fit: StackFit.expand, children: [Art(a), Positioned(left: 4, top: 4, child: TypeBadge(a.kind))])),
    );

Widget _dragCase(BuildContext c) => _DragScene(
      count: c.knobs.int.slider(label: 'Items dragged', initialValue: 3, min: 1, max: 12),
      accepts: c.knobs.boolean(label: 'Target accepts', initialValue: true),
      opacity: c.knobs.double.slider(label: 'Ghost opacity', initialValue: .92, min: .5, max: 1),
    );

// ------------------------------------------------------------------ empty state

enum _Empty { folder, search, favorites }

Widget _emptyCase(BuildContext c) {
  final v = c.knobs.object.dropdown<_Empty>(label: 'Variant', options: _Empty.values, initialOption: _Empty.folder, labelBuilder: (e) => e.name);
  final folder = c.knobs.string(label: 'Folder', initialValue: 'Day 3');
  final action = c.knobs.boolean(label: 'Show action', initialValue: true);
  final (title, hint, button) = switch (v) {
    _Empty.folder => ('"$folder" is empty', 'Drop files here or import them from disk.', 'Import…'),
    _Empty.search => ('No matches', 'Try a shorter search, or clear the filters.', 'Clear filters'),
    _Empty.favorites => ('No favorites yet', 'Star an asset or preset and it will show up here.', 'Browse bundled'),
  };
  return Container(
    decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
    alignment: Alignment.center,
    child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
      glyph(40, (cv, s) {
        switch (v) {
          case _Empty.folder:
            drawFolder(cv, s, N.g38);
          case _Empty.search:
            final p = Paint()..color = N.g38..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round;
            cv.drawCircle(Offset(s.width * .44, s.height * .44), s.width * .28, p);
            cv.drawLine(Offset(s.width * .66, s.height * .66), Offset(s.width * .88, s.height * .88), p);
          case _Empty.favorites:
            drawStar(cv, s, N.g38, false);
        }
      }, v),
      const SizedBox(height: 14),
      Text(title, style: T.title(N.g91)),
      const SizedBox(height: 8),
      Padding(padding: const EdgeInsets.symmetric(horizontal: 32), child: Text(hint, textAlign: TextAlign.center, style: T.label(N.g56).copyWith(height: 1.4))),
      if (action) ...[const SizedBox(height: 16), QuietButton(button, primary: v == _Empty.folder)],
    ]),
  );
}

// ------------------------------------------------------------ import drop zone

class _DropZone extends StatelessWidget {
  const _DropZone({required this.dragging, required this.look, required this.files});
  final bool dragging;
  final Look look;
  final int files;

  @override
  Widget build(BuildContext context) => Press(
        builder: (context, hover, focus, _) {
          final active = dragging;
          final lit = active || hover || focus;
          final col = active ? C.mode : (lit ? N.g56 : N.g38);
          return AnimatedContainer(
            duration: kFast,
            curve: kEase,
            decoration: BoxDecoration(color: active ? Role.selected.withValues(alpha: .08) : (hover ? N.g13 : const Color(0x00000000)), borderRadius: BorderRadius.circular(6), border: Border.all(color: active && look != Look.quiet ? const Color(0x00000000) : (active ? Role.selected : (focus ? N.g56 : (hover ? N.g26 : N.g20))))),
            child: CustomPaint(
              painter: !active ? null : (look == Look.concept ? DashedBox(col) : (look == Look.glow ? _Corners(col) : null)),
              child: Center(
                child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, children: [
                  glyph(24, (cv, s) {
                    final p = Paint()..color = active ? Role.selected : (lit ? N.g76 : N.g56)..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
                    final x = s.width / 2;
                    cv.drawLine(Offset(x, 2), Offset(x, s.height * .62), p);
                    cv.drawPath(Path()..moveTo(x - 5, s.height * .46)..lineTo(x, s.height * .64)..lineTo(x + 5, s.height * .46), p);
                    cv.drawPath(Path()..moveTo(3, s.height * .74)..lineTo(3, s.height - 2)..lineTo(s.width - 3, s.height - 2)..lineTo(s.width - 3, s.height * .74), p);
                  }, [active, lit]),
                  const SizedBox(height: 10),
                  Text(active ? 'Release to import' : 'Drop files to import', style: T.name(active ? N.g100 : N.g91)),
                  const SizedBox(height: 6),
                  Text(active ? '$files files · ${files * 42} MB' : 'or click to browse · MP4  MOV  WAV  GLB  EXR', style: T.value(N.g56).copyWith(fontSize: 10)),
                ]),
              ),
            ),
          );
        },
      );
}

class _Corners extends CustomPainter {
  const _Corners(this.color);
  final Color color;
  @override
  void paint(Canvas canvas, Size size) {
    final p = Paint()..color = color..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round;
    const l = 14.0, r = 6.0, o = 1.0;
    final w = size.width, h = size.height;
    canvas.drawPath(Path()..moveTo(o, l)..lineTo(o, r)..quadraticBezierTo(o, o, r, o)..lineTo(l, o), p);
    canvas.drawPath(Path()..moveTo(w - l, o)..lineTo(w - r, o)..quadraticBezierTo(w - o, o, w - o, r)..lineTo(w - o, l), p);
    canvas.drawPath(Path()..moveTo(w - o, h - l)..lineTo(w - o, h - r)..quadraticBezierTo(w - o, h - o, w - r, h - o)..lineTo(w - l, h - o), p);
    canvas.drawPath(Path()..moveTo(l, h - o)..lineTo(r, h - o)..quadraticBezierTo(o, h - o, o, h - r)..lineTo(o, h - l), p);
  }

  @override
  bool shouldRepaint(_Corners o) => o.color != color;
}

Widget _dropCase(BuildContext c) => _DropZone(
      dragging: c.knobs.boolean(label: 'Dragging over', initialValue: false),
      look: lookKnob(c),
      files: c.knobs.int.slider(label: 'Files', initialValue: 3, min: 1, max: 40),
    );

// --------------------------------------------------------------- details card

Widget _detailsCase(BuildContext c) {
  final kind = _kindKnob(c, Kind.video);
  final base = _byKind(kind);
  final name = c.knobs.string(label: 'Name', initialValue: base.name);
  final tags = c.knobs.boolean(label: 'Show tags', initialValue: true);
  final a = Asset(name, base.kind, base.seed, style: base.style, dur: base.dur, tags: base.tags, dims: base.dims, size: base.size);
  final extra = switch (kind) {
    Kind.image => [('Colour space', 'sRGB'), ('Alpha', 'No')],
    Kind.video => [('Frame rate', '29.97 fps'), ('Codec', 'H.264')],
    Kind.audio => [('Channels', 'Stereo'), ('Bit depth', '24-bit')],
    Kind.model => [('Format', 'glTF 2.0'), ('Materials', '3')],
    Kind.pano => [('Projection', 'Equirect'), ('Frame rate', '30 fps')],
  };
  final rows = [('Type', kind.label), (kind == Kind.audio ? 'Sample rate' : 'Dimensions', a.dims), if (a.dur > 0) ('Duration', clock(a.dur)), ...extra, ('Size', a.size), ('Modified', '12 Sep 2026')];
  return Hold<bool>(
    initial: false,
    builder: (context, fav, set) => Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
      child: Flex(direction: Axis.vertical, mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        AspectRatio(
          aspectRatio: 16 / 10,
          child: Container(
            foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
            child: ClipRRect(borderRadius: BorderRadius.circular(6), child: Stack(fit: StackFit.expand, children: [
              Art(a),
              if (kind != Kind.image) Positioned(left: 6, top: 6, child: TypeBadge(kind)),
              if (a.dur > 0) Positioned(right: 6, bottom: 6, child: TimePill(clock(a.dur))),
            ])),
          ),
        ),
        const SizedBox(height: 12),
        Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
          Expanded(child: Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
          StarButton(on: fav, onChanged: set),
        ]),
        const SizedBox(height: 4),
        Text('Project / Footage / Day 1', style: T.label(N.g56)),
        const SizedBox(height: 12),
        for (var i = 0; i < rows.length; i++)
          Container(
            height: 24,
            decoration: BoxDecoration(border: Border(top: BorderSide(color: i == 0 ? N.g20 : N.rowLine))),
            child: Flex(direction: Axis.horizontal, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Text(rows[i].$1, style: T.label(N.g56)),
              const Spacer(),
              Text(rows[i].$2, style: T.value(N.g91)),
            ]),
          ),
        if (tags) ...[
          const SizedBox(height: 12),
          Wrap(spacing: 4, runSpacing: 4, children: [
            for (final t in a.tags) Pill(t),
            Press(
              builder: (context, hover, focus, _) => Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
                decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: focus ? N.g56 : (hover ? N.g26 : N.g20))),
                child: Text('+ Tag', style: T.micro(hover ? N.g91 : N.g56)),
              ),
            ),
          ]),
        ],
        const SizedBox(height: 12),
        Flex(direction: Axis.horizontal, mainAxisAlignment: MainAxisAlignment.start, children: [QuietButton('Add to project', primary: true), const SizedBox(width: 8), const QuietButton('Reveal')]),
      ]),
    ),
  );
}
