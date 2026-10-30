// Part of dock_make.dart (cut there by size, as it was).
part of 'dock_make.dart';

/// One box of the colour bento: a colour or a gradient, the group it is of (the document's, one's own, a palette, the
/// gradients), and how much of the picture it is beside the others of its group (1: as much as each).
class _Box {
  const _Box(this.group, this.i, this.weight, {this.colour, this.grad});
  final String group;
  final int i;
  final double weight;
  final Color? colour;
  final _Gradient? grad;
}

int _dice = 0;   // (the dice's last throw: the boxes' shapes)
const _rainbow = [Color(0xFFFF3B30), Color(0xFFFF9500), Color(0xFFFFCC00), Color(0xFF34C759), Color(0xFF00C7BE), Color(0xFF007AFF), Color(0xFFAF52DE), Color(0xFFFF3B30)];
final _hovered = ValueNotifier<String?>(null);   // (the group of the box the hand is on)

/// Where each box lies, in cells: its shape thrown by the dice (a colour that is more of the picture, a larger box; a
/// gradient wide), then each put where it first fits from the top left and the gaps filled by later ones (CSS Grid's
/// auto-placement, dense: CSS Grid Layout 1, 8.5). A throw takes up about a third of the boxes, the rest keep their
/// shapes (the user: all of them changing at once was too much).
List<(int, int, int, int)> _packed(List<_Box> boxes, int cols, int seed) {
  final taken = <int, Set<int>>{}, out = <(int, int, int, int)>[];
  bool free(int x, int y, int w, int h) {
    if (x + w > cols) return false;
    for (var j = y; j < y + h; j++) {
      for (var i = x; i < x + w; i++) {
        if (taken[j]?.contains(i) ?? false) return false;
      }
    }
    return true;
  }
  for (final b in boxes) {
    var thrown = 0;   // (the last throw that took this box up)
    for (var d = 1; d <= seed; d++) {
      if (math.Random(Object.hash(d, b.group, b.i, 7)).nextDouble() < .35) thrown = d;
    }
    final r = math.Random(Object.hash(thrown, b.group, b.i));
    final shapes = b.grad != null ? const [(2, 1), (3, 1), (2, 2)] : b.weight >= 1.6 ? const [(2, 2), (3, 2), (2, 3), (3, 3)] : b.weight >= .9 ? const [(2, 1), (1, 2), (2, 2), (1, 1)] : const [(1, 1), (1, 1), (1, 2), (2, 1)];
    final (w0, h) = shapes[r.nextInt(shapes.length)];
    final w = math.min(w0, cols);
    for (var y = 0;; y++) {
      final x = [for (var x = 0; x < cols; x++) if (free(x, y, w, h)) x].firstOrNull;
      if (x == null) continue;
      for (var j = y; j < y + h; j++) {
        taken.putIfAbsent(j, () => {}).addAll([for (var i = x; i < x + w; i++) i]);
      }
      out.add((x, y, w, h));
      break;
    }
  }
  return out;
}

final _boxMenu = GlobalKey<WsMenuState>();
(String, List<(int, int, int, int)>)? _bentoLaid;   // (the last laying out, by what it was laid for)

/// How large a box is, any size the maker likes (the user: not a few steps, "任意で数値を"): a size slider over the
/// grid, as the Browser study's (sets/studies/panel_browser_b_8.dart, "size"), Finder's icon size, Photoshop's
/// Swatches panel, Lightroom's thumbnails.
const _tileLeast = 16.0, _tileMost = 120.0;
double _tile = 52;
Timer? _tileKept;

/// The colours as a bento of boxes of many sizes, wide and tall, packed with no holes, as many to a row as the shelf is
/// wide (the user: a poster's colours come with the space they take, so not all squares; a dice throws the shapes
/// again). Boxes move briskly to where they now lie, without overshooting (Flutter's AnimatedPositioned, easeOutCubic).
class _Bento extends StatelessWidget {
  const _Bento(this.boxes, this.c);
  final List<_Box> boxes;
  final DockCtx c;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, box) {
      // (as many columns as boxes of the size chosen fit, the boxes stretching to fill the row: CLIP STUDIO PAINT's
      //  Color Set at a fixed tile size, as many a row as the palette is wide; CSS Grid's repeat(auto-fill, minmax()).
      //  Laid again only when its columns, its colours or the dice change, and only then do the boxes move; widened
      //  between, they only stretch, nothing laid again: the window's 60 frames a second kept)
      final cols = math.max(1, box.maxWidth ~/ Shape.z(_tile)), unit = box.maxWidth / cols, all = boxes;
      final key = '$cols/$_dice/${[for (final b in all) '${b.group}.${b.i}.${b.weight.toStringAsFixed(2)}'].join(',')}';
      final laid = _bentoLaid?.$1 == key ? _bentoLaid!.$2 : _packed(all, cols, _dice), moved = _bentoLaid?.$1 != key;
      _bentoLaid = (key, laid);
      final rows = laid.fold(0, (a, l) => math.max(a, l.$2 + l.$4)), gap = Shape.s(3);
      return WsMenu(
        key: _boxMenu,
        child: SizedBox(
          height: rows * unit,
          child: Stack(
            children: [
              for (final (k, b) in all.indexed)
                AnimatedPositioned(
                  key: ValueKey('${b.group}-${b.i}'),
                  duration: moved ? const Duration(milliseconds: 200) : Duration.zero,
                  curve: Curves.easeOutCubic,
                  left: laid[k].$1 * unit,
                  top: laid[k].$2 * unit,
                  width: laid[k].$3 * unit - gap,
                  height: laid[k].$4 * unit - gap,
                  child: b.grad != null ? _Grad('grad-${b.i}', b.grad!, c) : _ColourBox(b, c),
                ),
            ],
          ),
        ),
      );
    },
  );
}

/// A colour's box: pressed, it colours (or the stop picked, while a gradient is coloured); the hand on it rings its
/// group and tells the group in the shelf's foot; its right-click menu, for one's own, replaces or deletes it.
class _ColourBox extends StatelessWidget {
  const _ColourBox(this.b, this.c);
  final _Box b;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final col = b.colour!, own = b.group == 'mine', pal = _pals.where((p) => p.id == b.group).firstOrNull;
    return MouseRegion(
      onEnter: (_) => _named.value = _hovered.value = b.group,
      onExit: (_) {
        _named.value = _pressed;
        _hovered.value = null;
      },
      child: GestureDetector(
        onSecondaryTapDown: (d) {
          final items = [
            if (own) ...[
              WsMenuItem('Replace with ${_hex(_wheel.toColor())}', () => c.set(() {
                _mineColours[b.i] = _wheel.toColor();
                _keepMine();
              })),
              WsMenuItem('Delete', () => c.set(() {
                _mineColours.removeAt(b.i);
                _keepMine();
              })),
            ],
            if (pal != null && pal.saved) WsMenuItem('Delete this palette', () => c.set(() {
              _minePals.remove(pal);
              _named.value = _pressed = null;
              _keepMine();
            })),
          ];
          if (items.isEmpty) return;
          final box = _boxMenu.currentContext?.findRenderObject() as RenderBox?;
          _boxMenu.currentState?.show(box?.globalToLocal(d.globalPosition) ?? Offset.zero, items);
        },
        child: DockHover(
          key: ValueKey('colour-box-${b.group}-${b.i}'),
          onTap: () {
            _named.value = _pressed = b.group;
            _take(c, col);
          },
          builder: (context, h) => ValueListenableBuilder<String?>(
            valueListenable: _hovered,
            builder: (context, over, _) {   // (the hand on a box: a rainbow rim round each box of its palette, so a palette is seen whole with no gap kept round it: the user, a rim, not the others darkened)
              final box = Container(
                decoration: BoxDecoration(
                  color: col,
                  borderRadius: BorderRadius.all(Shape.r(over == b.group ? 3 : 5)),
                  border: _wheelFrom == _hex(col).substring(1).toLowerCase() ? Border.all(color: Grey.g95, width: 2) : null,
                ),
              );
              return over != b.group
                  ? box
                  : Container(
                      key: const ValueKey('colour-rim'),
                      padding: const EdgeInsets.all(2),
                      decoration: BoxDecoration(borderRadius: BorderRadius.all(Shape.r(5)), gradient: const SweepGradient(colors: _rainbow)),
                      child: box,
                    );
            },
          ),
        ),
      ),
    );
  }
}

/// The colour now kept as one's own (CLIP STUDIO PAINT's colour set: Add Color); a gradient coloured now is kept whole,
/// where its colours stand and how they mix.
void _keepNow(DockCtx c) {
  final stops = _stopsIn(_paintNow(c));
  stops.length > 1
      ? _mineGrads.insert(0, _Gradient('Gradient ${_mineGrads.length + 1}', [for (final s in stops) s.$1], at: [for (final s in stops) s.$2], blend: _blendNow(c), saved: true))
      : _mineColours.insert(0, _wheel.toColor());   // (at the top, as the newest)
  _keepMine();
}

/// The dice: a die's face of five; pressed, the boxes are thrown again.
class _Dice extends StatelessWidget {
  const _Dice(this.onTap);
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => DockHover(
    key: const ValueKey('colour-dice'),
    onTap: onTap,
    builder: (context, h) => Container(
      width: Shape.z(22),
      height: Shape.z(22),
      decoration: BoxDecoration(color: h ? WsT.raised : WsT.well, borderRadius: BorderRadius.all(Shape.r(5))),
      child: CustomPaint(painter: _DiePaint(h ? Grey.g95 : Grey.g76)),
    ),
  );
}

class _DiePaint extends CustomPainter {
  const _DiePaint(this.ink);
  final Color ink;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint()..color = ink, d = s.width / 4.5;
    for (final o in const [Offset(-1, -1), Offset(1, -1), Offset(0, 0), Offset(-1, 1), Offset(1, 1)]) {
      c.drawCircle(s.center(Offset.zero) + o * d, s.width / 14, p);
    }
  }

  @override
  bool shouldRepaint(_DiePaint o) => o.ink != ink;
}

/// A gradient's colours and where each stands, as the document says its fill ("ff3366@0.35 ..."; one with no @ stands
/// evenly), and back.
List<(Color, double)> _stopsIn(String word) {
  final parts = word.split(' ').where((w) => w.isNotEmpty).toList();
  if (parts.length < 2) return const [];
  final out = <(Color, double)>[];
  for (final (i, w) in parts.indexed) {
    final hex = w.split('@').first, v = int.tryParse(hex, radix: 16);
    if (v == null || (hex.length != 6 && hex.length != 8)) return const [];
    out.add((hex.length == 8 ? Color(((v & 0xFF) << 24) | (v >> 8)) : Color(0xFF000000 | v), w.contains('@') ? (double.tryParse(w.split('@').last) ?? 0).clamp(0.0, 1.0) : i / (parts.length - 1)));
  }
  return out;
}

String _wordOfStops(List<(Color, double)> stops) {
  String hex8(Color c) => _hex(c).substring(1).toLowerCase() + (c.a < 1 ? (c.a * 255).round().toRadixString(16).padLeft(2, '0') : '');
  return [for (final (col, at) in [...stops]..sort((a, b) => a.$2.compareTo(b.$2))) '${hex8(col)}@${at.toStringAsFixed(3)}'].join(' ');
}

int _stop = 0;   // (the stop the picker colours)

/// The gradient being coloured, its bar and its stops beneath (Figma's and Canva's gradient editors): a press on the
/// bar puts a stop there in the colour the bar has there; a stop pressed is the one the square above colours; a stop
/// carried along the bar moves; Remove takes the picked one out (two stay at least).
class _Stops extends StatefulWidget {
  const _Stops(this.stops, this.onChanged, this.onPick, {this.onHand});
  final List<(Color, double)> stops;
  final ValueChanged<List<(Color, double)>> onChanged;
  final ValueChanged<int> onPick;
  final ValueChanged<bool>? onHand;   // (a hand down on it, and up: one step)
  @override
  State<_Stops> createState() => _StopsState();
}

class _StopsState extends State<_Stops> {
  List<(Color, double)>? _held;   // (while a stop is carried: where the hand has it)

  Color _at(List<(Color, double)> s, double x) {
    final sorted = [...s]..sort((a, b) => a.$2.compareTo(b.$2));
    if (x <= sorted.first.$2) return sorted.first.$1;
    for (var i = 1; i < sorted.length; i++) {
      if (x <= sorted[i].$2) return Color.lerp(sorted[i - 1].$1, sorted[i].$1, (x - sorted[i - 1].$2) / math.max(1e-6, sorted[i].$2 - sorted[i - 1].$2))!;
    }
    return sorted.last.$1;
  }

  @override
  Widget build(BuildContext context) {
    final s = _held ?? widget.stops, sorted = [...s]..sort((a, b) => a.$2.compareTo(b.$2)), knob = Shape.z(12);
    return LayoutBuilder(
      builder: (context, box) {
        final w = box.maxWidth - knob;
        return SizedBox(
          height: Shape.z(18) + knob + Shape.s(4),
          child: Stack(
            clipBehavior: Clip.none,
            children: [
              Positioned(
                left: knob / 2,
                width: w,
                top: 0,
                height: Shape.z(18),
                child: GestureDetector(
                  key: const ValueKey('colour-stops-bar'),
                  onTapUp: (d) {
                    final x = (d.localPosition.dx / w).clamp(0.0, 1.0);
                    widget.onChanged([...s, (_at(s, x), x)]);
                    widget.onPick(s.where((e) => e.$2 <= x).length);   // (where it stands among them: the document keeps them in order)
                  },
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      borderRadius: BorderRadius.all(Shape.r(3)),
                      gradient: LinearGradient(colors: [for (final e in sorted) e.$1], stops: [for (final e in sorted) e.$2]),
                    ),
                  ),
                ),
              ),
              for (final (i, (col, at)) in s.indexed)
                Positioned(
                  left: at * w,
                  top: Shape.z(18) + Shape.s(3),
                  width: knob,
                  height: knob,
                  child: GestureDetector(
                    key: ValueKey('colour-stop-$i'),
                    onTap: () => widget.onPick(i),
                    onHorizontalDragStart: (_) {
                      widget.onPick(i);
                      widget.onHand?.call(true);
                      setState(() => _held = [...s]);
                    },
                    onHorizontalDragUpdate: (d) {
                      final held = _held!;
                      held[i] = (held[i].$1, (held[i].$2 + d.delta.dx / w).clamp(0.0, 1.0));
                      setState(() {});
                      widget.onChanged([...held]);
                    },
                    onHorizontalDragEnd: (_) {
                      final held = _held!, at = held[i].$2;
                      widget.onPick(held.where((e) => e.$2 < at).length);   // (carried past another, it is picked where it now stands)
                      setState(() => _held = null);
                      widget.onHand?.call(false);
                    },
                    child: Container(
                      decoration: BoxDecoration(
                        color: col,
                        borderRadius: BorderRadius.all(Shape.r(2)),
                        border: Border.all(color: i == _stop ? Grey.g95 : Grey.g44, width: i == _stop ? 2 : 1),
                      ),
                    ),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }
}

/// A gradient, a small tile of it: its name is said when the hand is on it, as a swatch's (the colours are what is
/// looked for, and the tiles keep to a block however many are kept).
class _Grad extends StatelessWidget {
  const _Grad(this.id, this.g, this.c);
  final String id;
  final _Gradient g;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final word = [for (final k in g.colors) _hex(k).substring(1).toLowerCase()].join(' ');
    final (look, at) = _previewOf(g.colors, g.blend, g.at);
    return Semantics(
      label: g.name,
      child: GestureDetector(
        onSecondaryTapDown: !g.saved ? null : (d) {   // (one's own gradient's menu: deleted, as one's own colour)
          final box = _boxMenu.currentContext?.findRenderObject() as RenderBox?;
          _boxMenu.currentState?.show(box?.globalToLocal(d.globalPosition) ?? Offset.zero, [
            WsMenuItem('Delete this gradient', () => c.set(() {
              _mineGrads.remove(g);
              _keepMine();
            })),
          ]);
        },
        child: DockHover(
        key: ValueKey('colour-$id'),
        onTap: () => _putGradient(c, g.colors, g.blend, g.at),   // (a press puts it on, where its colours stand and mixed as it was kept)
        builder: (context, h) => Container(
          decoration: BoxDecoration(
            gradient: LinearGradient(colors: look, stops: at),
            borderRadius: BorderRadius.all(Shape.r(5)),
            border: _wheelFrom == word ? Border.all(color: Grey.g95, width: 2) : (h ? Border.all(color: Grey.g76) : null),
          ),
        ),
      ),
      ),
    );
  }
}

// ---------------------------------------------------------------- create

class _Make {
  const _Make(this.name, this.g, this.hue, this.key, this.what, {this.recipe = false});
  final String name, key, what;
  final DockG g;
  final Color hue;

  /// One of the recipes (a picture made in one call), which may share a name with a tile of the shelf's own: AbsentMotion's
  /// Rectangle, Star, Cube, Sun beside Motolii's shape, solid and light of those names (the shelf's own were made as
  /// the recipe while it was asked first).
  final bool recipe;
  String get id => recipe ? 'recipe:$name' : name;
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
/// What a solid is made of, a material: put on the picked solid, as Repeater lays the picked thing out (a material is
/// made in Create, not chosen among colours: the user, 10/29, "マテリアルなら普通クリエイトに入れるべき").
final _materials = (
  'Materials',
  [
    _Make('Glass', DockG.radial, WsT.toneShape, '', 'Thick glass: bends and tints what lies behind it. Put on the picked solid'),
  ],
);
const _worn = {'Glass': 'glass'};   // (the tile, and the solid's fill it gives)

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
void _make(DockCtx c, _Make tile) {
  final m = c.ws.inside!, name = tile.name;
  if (tile.recipe) { // (a picture in one call: one step, and it is picked)
    c.ws.after(m.later('make', word: name), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why), (made) => c.ws.select('$made'));   // (without waiting: picked once made)
    return;
  }
  if (_worn[name] case final fill?) {
    final t = int.tryParse(c.ws.selected ?? '');
    if (t == null || m.word(t, 'kind') != 'solid') return c.set(() => c.mem.trouble[name] = 'Pick a solid to make it ${name.toLowerCase()}');
    c.ws.after(m.later('say', thing: t, name: 'fill', word: fill), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why));
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
  m.later('together', number: 1);   // (one step: the thing and its inks)
  c.ws.after(m.later('add', name: asked.$1, word: asked.$1 == 'text' ? c.ws.bodyFace : asked.$2), null, (made) => c.ws.select('$made'));   // (without waiting: picked once made; a text in the body's font)
  wsInked(c.ws, asked.$1, asked.$2);
  c.ws.after(m.later('together'), (done, why) => c.set(() => done ? c.mem.trouble.remove(name) : c.mem.trouble[name] = why));
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
      _materials,
      _lights,
    ].where((e) => e.$2.isNotEmpty).toList();
    final groups = [
      for (final (g, items) in makes)
        if (c.chip == 'All' || c.chip == g) (g, items.where((m) => c.hit(m.name)).toList()),
    ].where((e) => e.$2.isNotEmpty).toList();
    final sel = makes.expand((e) => e.$2).where((m) => m.id == c.sel).firstOrNull;
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
            action: _laid.containsKey(sel.name) ? 'Lay out' : _worn.containsKey(sel.name) ? 'Put on' : 'Create',
            fresh: !_laid.containsKey(sel.name) && !_worn.containsKey(sel.name), // (a tile makes a new thing; a tile of copies lays out the picked one, a material is put on it)
            onAction: () => _make(c, sel),
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
    final sel = c.sel == m.id;
    final inside = c.ws.inside;
    final makes = inside != null && (m.recipe ? inside.recipes.any((r) => r.$1 == m.name) : _asked.containsKey(m.name) || _laid.containsKey(m.name) || _worn.containsKey(m.name) || inside.elements.any((e) => e.$1 == m.name));
    return DockHover(
      onTap: () => c.pick(m.id),
      onDoubleTap: makes ? () => _make(c, m) : null, // (two presses make it, one step)
      picks: true,
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
