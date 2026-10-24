// The browser's media & assets. Direction from concept/browser-panels-concept.png: the work (thumbnails) is the brightest thing,
// the chrome is grey; one selection colour (C.mode); hover is a grey step; 120ms ease-out.
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../kit.dart';
import '../../parts/controls.dart';
import '../../tokens.dart';
import 'media_parts.dart';
part 'media_set_2.dart';
part 'media_set_3.dart';

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
