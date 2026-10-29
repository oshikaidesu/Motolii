// Part of dock_make.dart (cut there by size, as it was).
part of 'dock_make.dart';

/// The gradient being made: its stops (a colour put in with Add stop; a stop pressed is taken out), the ramp they
/// make in the space chosen, and Save (kept with the maker's own).
class _GradMaker extends StatelessWidget {
  const _GradMaker(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final picked = _pickedColour(c), ramp = _stopsOf(_tray, _trayOklab);
    return Container(
      margin: EdgeInsets.only(top: WsT.gutter),
      color: WsT.card,
      padding: EdgeInsets.all(WsT.inset),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(child: Text('Make a gradient', style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700))),
              _Pill(_trayOklab ? 'Mix: CAM16' : 'Mix: sRGB', () => c.set(() => _trayOklab = !_trayOklab), on: _trayOklab),
            ],
          ),
          SizedBox(height: Shape.s(6)),
          DockHover(
            onTap: _tray.length > 1 ? () => c.pick('maker-0') : null,
            builder: (context, h) => Container(
              height: Shape.z(24),
              decoration: BoxDecoration(
                color: _tray.length > 1 ? null : WsT.well,
                gradient: _tray.length > 1 ? LinearGradient(colors: ramp) : null,
                borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius)),
                border: c.sel == 'maker-0' ? Border.all(color: Grey.g95, width: 2) : null,
              ),
              alignment: Alignment.center,
              child: _tray.length > 1 ? null : Text('Add two colours or more', style: T.label(Grey.g56)),
            ),
          ),
          SizedBox(height: Shape.s(6)),
          Row(
            children: [
              for (final (i, col) in _tray.indexed) ...[
                DockHover(
                  onTap: () => c.set(() => _tray.removeAt(i)),   // (pressed: taken out)
                  builder: (context, h) => Container(width: Shape.z(18), height: Shape.z(18), decoration: BoxDecoration(color: col, border: h ? Border.all(color: Grey.g95, width: 2) : null, borderRadius: BorderRadius.all(Shape.r(3)))),
                ),
                SizedBox(width: Shape.s(4)),
              ],
              _Pill('Add stop', () => c.set(() => _tray.add(picked!)), enabled: picked != null && _tray.length < 8),
              SizedBox(width: Shape.s(4)),
              _Pill('Save', () => c.set(() {
                _mineGrads.insert(0, _Gradient('Gradient ${_mineGrads.length + 1}', List.of(_tray), oklab: _trayOklab, saved: true));
                _tray.clear();
                _keepMine();
              }), enabled: _tray.length > 1),
              SizedBox(width: Shape.s(4)),
              _Pill('Clear', () => c.set(_tray.clear), enabled: _tray.isNotEmpty),
            ],
          ),
        ],
      ),
    );
  }
}

/// The colour picked on this shelf now, if one is picked.
Color? _pickedColour(DockCtx c) {
  final id = c.sel;
  if (id == null) return null;
  final parts = id.split('-'), i = int.tryParse(parts.last);
  if (i == null) return null;
  final key = parts.sublist(0, parts.length - 1).join('-');
  if (key == 'wheel') return _wheel.toColor();
  if (key == 'found' && i < _found.length) return _found[i];
  if (key == 'used') return usedColours(c.ws).values.elementAtOrNull(i);
  if (key.startsWith('field') && i < 12) return _field[int.parse(key.split('-').last)][i];
  if (key == 'tray') return i < _tray.length ? _tray[i] : null;
  final p = _pals.where((p) => p.id == key).firstOrNull;
  return p != null && i < p.colors.length ? p.colors[i] : null;
}

class _PalCard extends StatelessWidget {
  const _PalCard(this.p, this.c);
  final _Pal p;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final ref = p.reference, open = _refOpen.contains(p.id);
    return Container(
      margin: EdgeInsets.only(top: WsT.gutter),
      color: WsT.card,
      padding: EdgeInsets.fromLTRB(WsT.inset, Shape.s(6), WsT.inset, WsT.inset),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(p.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
              ),
              if (p.saved) ...[const DockTag('SAVED'), SizedBox(width: Shape.s(6))],
              Text('${p.colors.length}', style: T.value(Grey.g56).copyWith(fontSize: T.px(10))),
            ],
          ),
          SizedBox(height: Shape.s(6)),
          Row(
            children: [
              if (ref != null && File(ref).existsSync()) ...[   // (the photo it was taken from: pressed, it opens large and stays)
                DockHover(
                  onTap: () => c.set(() => open ? _refOpen.remove(p.id) : _refOpen.add(p.id)),
                  builder: (context, h) => ClipRRect(
                    borderRadius: BorderRadius.all(Shape.r(3)),
                    child: SizedBox(width: Shape.z(40), height: Shape.z(26), child: Image.file(File(ref), fit: BoxFit.cover, cacheWidth: 120)),
                  ),
                ),
                SizedBox(width: Shape.s(6)),
              ],
              Expanded(child: _Strip(p.id, p.colors, c, height: Shape.z(26), pad: false, shares: p.shares)),
            ],
          ),
          if (open && ref != null && File(ref).existsSync()) ...[
            SizedBox(height: Shape.s(6)),
            ClipRRect(
              borderRadius: BorderRadius.all(Shape.r(3)),
              child: ConstrainedBox(constraints: BoxConstraints(maxHeight: Shape.z(160)), child: Image.file(File(ref), fit: BoxFit.contain, cacheWidth: 480)),
            ),
          ],
        ],
      ),
    );
  }
}

/// Flat swatches edge to edge (each as wide as its share of a photo, if it has one); the picked one gets a light frame with a dark keyline inside, readable on any hue.
class _Strip extends StatelessWidget {
  const _Strip(this.id, this.colors, this.c, {this.names, this.height = 24, this.gap = WsT.gutter, this.pad = true, this.shares});
  final String id;
  final List<Color> colors;
  final DockCtx c;
  final List<String>? names;
  final double height, gap;
  final bool pad;
  final List<double>? shares;
  @override
  Widget build(BuildContext context) => Padding(
    padding: EdgeInsets.fromLTRB(pad ? WsT.inset : Shape.s(0), pad ? gap : Shape.s(0), pad ? WsT.inset : Shape.s(0), Shape.s(0)),
    child: SizedBox(
      height: height,
      child: Row(
        children: [
          for (final (i, col) in colors.indexed) ...[
            if (i > 0) SizedBox(width: gap),
            Expanded(
              flex: shares == null ? 1000 : math.max(60, (shares![i] * 1000).round()),
              child: DockHover(
                onTap: () => c.pick('$id-$i'),
                onDoubleTap: () => _fill(c, _hex(col).substring(1).toLowerCase()),   // (two presses fill; carried to the stage it does the same)
                builder: (context, h) {
                  final sel = c.sel == '$id-$i';
                  return Container(
                    decoration: BoxDecoration(
                      color: col,
                      borderRadius: BorderRadius.circular(sel || h ? 3 : 0),
                      border: sel ? Border.all(color: Grey.g95, width: 2) : (h ? Border.all(color: Grey.g76) : null),
                    ),
                    foregroundDecoration: sel
                        ? BoxDecoration(
                            border: Border.all(color: Grey.g07),
                            borderRadius: BorderRadius.all(Shape.r(1)),
                          )
                        : null,
                  );
                },
              ),
            ),
          ],
        ],
      ),
    ),
  );
}

class _Grad extends StatelessWidget {
  const _Grad(this.id, this.g, this.c);
  final String id;
  final _Gradient g;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == id;
    return DockHover(
      onTap: () => c.pick(id),
      onDoubleTap: () => _fill(c, [for (final k in _stopsOf(g.colors, g.oklab)) _hex(k).substring(1).toLowerCase()].join(' ')),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: Shape.z(28),
        margin: EdgeInsets.only(top: WsT.gutter),
        padding: EdgeInsets.symmetric(horizontal: WsT.inset),
        color: dockFill(sel, h, WsT.card),
        child: Row(
          children: [
            Container(
              width: Shape.z(64),
              height: Shape.z(16),
              decoration: BoxDecoration(
                gradient: LinearGradient(colors: _stopsOf(g.colors, g.oklab)),
                borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius)),
              ),
            ),
            SizedBox(width: Shape.s(8)),
            Expanded(
              child: Text(g.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(dockInk(sel))),
            ),
            Text('${g.colors.length} stops · ${g.oklab ? 'CAM16' : 'sRGB'}', style: T.label(dockInk(sel, Grey.g56))),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------- create

class _Make {
  const _Make(this.name, this.g, this.hue, this.key, this.what);
  final String name, key, what;
  final DockG g;
  final Color hue;
}

final _makes = <(String, List<_Make>)>[
  (
    'Layers',
    [
      _Make('Text', DockG.text, WsT.toneText, 'T', 'Type layer'),
      _Make('Camera', DockG.camera, WsT.toneCamera, '⇧C', '3D camera, 50 mm'),
      _Make('Null', DockG.nullObj, Grey.g76, '⇧N', 'Invisible parent'),
    ],
  ),
  (
    'Shapes',
    [
      _Make('Rectangle', DockG.rect, WsT.toneShape, 'R', 'Rounded corners'),
      _Make('Ellipse', DockG.ellipse, WsT.toneShape, 'E', 'Circle with ⇧'),
      _Make('Polygon', DockG.polygon, WsT.toneShape, '', 'Sides · Roundness'),
      _Make('Star', DockG.star, WsT.toneShape, '', 'Points · Inner radius'),
      _Make('Line', DockG.line, WsT.toneShape, 'L', 'Open path'),
    ],
  ),
  (
    'Copies',
    [
      _Make('Repeater', DockG.copies, Pop.tonePlace, '', 'Copies of the picked thing: how many, in how many rows, how far bent round a centre'),
    ],
  ),
];

/// The inside's own 3D shapes, and how a tile is asked of it (kind, then its file or shape).
/// What a solid is made of (a colour, glass) is not a tile here: it is its fill, given on the Colors shelf.
final _solids = (
  '3D',
  [
    _Make('Ring', DockG.radial, WsT.toneShape, '', 'A solid ring'),
    _Make('Gem', DockG.polygon, WsT.toneShape, '', 'A cut solid'),
    _Make('Cube', DockG.rect, WsT.toneShape, '', 'A round-edged cube'),
    _Make('Ball', DockG.ellipse, WsT.toneShape, '', 'A ball'),
  ],
);

/// Filament's own lights, things of the document: placed and turned as any thing, they light the solid things.
final _lights = (
  'Light',
  [
    _Make('Sun', DockG.light, WsT.toneCamera, '', 'Light from one way, everywhere'),
    _Make('Point', DockG.light, WsT.toneCamera, '', 'Light from a point, all round'),
    _Make('Spot', DockG.light, WsT.toneCamera, '', 'A cone of light'),
  ],
);
final _asked = {
  'Text': ('text', wsTextFont),
  'Solid': ('shape', 'solid'), // (a flat colour as large as the stage: the inside makes it so)
  'Rectangle': ('shape', 'rect'),
  'Ellipse': ('shape', 'circle'),
  'Polygon': ('shape', 'polygon'),
  'Star': ('shape', 'star'),
  'Line': ('shape', 'line'),
  'Null': ('shape', 'null'), // (nothing to see: a thing to go with)
  'Camera': ('camera', 'camera'), // (the nearest one that is on is the stage's eye)
  'Ring': ('solid', 'ring'),
  'Gem': ('solid', 'gem'),
  'Cube': ('solid', 'cube'),
  'Ball': ('solid', 'ball'),
  'Sun': ('light', 'sun'),
  'Point': ('light', 'point'),
  'Spot': ('light', 'spot'),
};

/// The tiles that lay the picked thing out in copies (the document's copies, as After Effects' repeater), and
/// what each tells it: in a row, side by side and a little smaller each time; round a centre below it.
const _laid = <String, Map<String, double>>{
  'Repeater': {'copies': 5, 'copy_x': 1.3, 'copy_size': .92},
};

/// A tile made on the stage as a new thing, picked; a tile of copies lays the picked thing out. Not done: the foot of
/// the shelf says why.
void _make(DockCtx c, String name) {
  final m = c.ws.inside!;
  if (m.recipes.any((r) => r.$1 == name)) { // (a picture in one call: one step, and it is picked)
    c.ws.after(m.later('make', word: name), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why), (made) => c.ws.select('$made'));   // (without waiting: picked once made)
    return;
  }
  if (_laid[name] case final told?) {
    final t = int.tryParse(c.ws.selected ?? '');
    if (t == null) return c.set(() => c.mem.trouble[name] = 'Pick a thing to lay out');
    m.later('together', number: 1);   // (one step, without waiting)
    for (final e in told.entries) {
      m.later('set', thing: t, name: e.key, number: e.value.toDouble());
    }
    c.ws.after(m.later('together'), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why));
    return;
  }
  if (!_asked.containsKey(name)) {
    if (m.elements.where((e) => e.$1 == name).firstOrNull case final e?) {   // (at the playhead, in the middle, picked: AbsentMotion's addElement)
      c.ws.after(m.later('element', word: e.$2, number: c.ws.frame / Ws.fps), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why), (made) => c.ws.select('$made'));
    }
    return;
  }
  final asked = _asked[name]!;
  c.ws.after(m.later('add', name: asked.$1, word: asked.$2), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why), (made) => c.ws.select('$made'));   // (without waiting: picked once made)
}

class CreateShelf extends StatelessWidget {
  const CreateShelf(this.c, {super.key});
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final makes = [
      _recipesOf(c.ws),
      _makes.first,
      _elementsOf(c.ws),
      ..._makes.skip(1),
      _solids,
      _lights,
    ].where((e) => e.$2.isNotEmpty).toList();
    final groups = [
      for (final (g, items) in makes)
        if (c.chip == 'All' || c.chip == g) (g, items.where((m) => c.hit(m.name)).toList()),
    ].where((e) => e.$2.isNotEmpty).toList();
    final sel = makes.expand((e) => e.$2).where((m) => m.name == c.sel).firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        DockChips(
          [('All', makes.fold<int>(0, (s, e) => s + e.$2.length)), for (final (g, items) in makes) (g, items.length)],
          active: c.chip,
          onPick: c.pickChip,
        ),
        Expanded(
          child: groups.isEmpty
              ? DockEmpty(c.query)
              : DockMasonry(
                  minColumn: 300,
                  weights: [for (final (_, items) in groups) 1 + items.length / 3],
                  children: [
                    for (final (g, items) in groups)
                      Padding(
                        padding: EdgeInsets.only(bottom: WsT.gutter),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            DockSection(g, count: items.length, hue: items.first.hue, folded: c.query.isEmpty && c.mem.createFolded.contains(g), onTap: () => c.set(() => c.mem.createFolded.contains(g) ? c.mem.createFolded.remove(g) : c.mem.createFolded.add(g))),
                            if (!(c.query.isEmpty && c.mem.createFolded.contains(g))) ...[
                            const SizedBox(height: WsT.gutter),
                            LayoutBuilder(
                              builder: (context, box) {
                                final cols = wsCols(box.maxWidth, 74, min: 2), w = wsCell(box.maxWidth, cols);
                                return Wrap(
                                  spacing: WsT.gutter,
                                  runSpacing: WsT.gutter,
                                  children: [for (final m in items) SizedBox(width: w, height: Shape.z(70), child: _MakeTile(m, c))],
                                );
                              },
                            ),
                            ],
                          ],
                        ),
                      ),
                  ],
                ),
        ),
        if (sel != null && sel.name == 'Repeater') _CopiesPanel(c),
        if (sel != null && sel.name != 'Repeater')
          DockFooter(
            preview: DockBadge(sel.g, sel.hue, size: Shape.z(34)),
            title: '${sel.name} layer',
            meta: c.mem.trouble[sel.name] ?? 'At ${c.ws.timecode()} · ${sel.what}',
            action: _laid.containsKey(sel.name) ? 'Lay out' : 'Create',
            fresh: !_laid.containsKey(sel.name), // (a tile makes a new thing; a tile of copies lays out the picked one)
            onAction: () => _make(c, sel.name),
          ),
      ],
    );
  }
}

class _MakeTile extends StatelessWidget {
  const _MakeTile(this.m, this.c);
  final _Make m;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == m.name;
    final inside = c.ws.inside;
    final makes = inside != null && (_asked.containsKey(m.name) || _laid.containsKey(m.name) || inside.recipes.any((r) => r.$1 == m.name) || inside.elements.any((e) => e.$1 == m.name));
    return DockHover(
      onTap: () => c.pick(m.name),
      onDoubleTap: makes ? () => _make(c, m.name) : null, // (two presses make it, one step)
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        padding: EdgeInsets.all(Shape.s(7)),
        color: sel ? WsT.accent : (h ? WsT.raised : WsT.card),
        child: Stack(
          children: [
            if (_isRecipe(c, m.name))   // (a recipe: its motion as the inside reckons it, drawn by the window, the whole list moving at once)
              Positioned.fill(child: RecipeMotionView(m.name))
            else
              Align(
                alignment: Alignment.topLeft,
                child: DockGlyph(m.g, size: Shape.z(26), color: sel ? WsT.onAccent : m.hue),
              ),
            if (m.key.isNotEmpty)
              Align(
                alignment: Alignment.topRight,
                child: Text(m.key, style: T.micro(dockInk(sel, Grey.g56))),
              ),
            Align(
              alignment: Alignment.bottomLeft,
              child: AutoSizeText( // (a long name on two lines of whole words, as Finder's icons have it; smaller rather than a word broken)
                m.name,
                maxLines: 2,
                wrapWords: false,
                minFontSize: 6,
                stepGranularity: .5,
                style: T.name(dockInk(sel, Grey.g95)).copyWith(fontWeight: FontWeight.w700),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ---- the copies, as values (not as three kinds): how many, how many rows, how far bent round a centre. Copies are the
// document's own numbers (copies, copy_x, copy_y, copy_columns, copy_rotate, anchor_y), each keyable, so a row
// can be bent into a ring, a grid into a row, by keys. The sliders write those numbers on the picked thing; a hand's
// whole pull is one step.
double _copyCount = 5, _copyColumns = 1, _copyBend = 0;

void _lay(DockCtx c) {
  final t = int.tryParse(c.ws.selected ?? ''), m = c.ws.inside;
  if (t == null || m == null) return;
  final n = _copyCount.round(), cols = _copyColumns.round().clamp(1, n), bend = _copyBend, spacing = 1.3;
  final ring = n / (2 * math.pi) * spacing;   // (the radius at which n copies a step apart lie round a centre)
  final values = {
    'copies': n.toDouble(),
    'copy_columns': cols.toDouble(),
    'copy_x': spacing * (1 - bend),
    'copy_y': cols > 1 ? -spacing * (1 - bend) : 0.0,
    'copy_rotate': bend * 360 / n,
    'anchor_y': -bend * ring,
  };
  for (final e in values.entries) {
    m.later('set', thing: t, name: e.key, number: e.value);
  }
}
