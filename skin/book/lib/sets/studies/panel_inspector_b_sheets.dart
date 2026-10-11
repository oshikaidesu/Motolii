part of 'panel_inspector_b.dart';

// ---- Parts @200%: the small parts of each option group, every state at once, drawn at twice the size -------------------------
// A cell lays its part out at 1x and paints it through a 2x canvas transform (vector, no filter): lines, paddings and text sizes stay sharp and keep their real proportions.

class _Cell {
  const _Cell(this.cap, this.child, {this.w = 140, this.h = 28});
  final String cap;
  final Widget child;
  final double w, h;
}

/// One plate: the caption (10 px, g76), then the part at 2x on the panel's own ground g10 with a hairline. [h] is the plate height at 1x: every plate of one row has the same width and the same height.
class _Z extends StatelessWidget {
  const _Z(this.c, this.h);
  final _Cell c;
  final double h;
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        SizedBox(width: c.w * 2 + 2, height: 24, child: Align(alignment: Alignment.topLeft, child: Text(c.cap, maxLines: 2, overflow: TextOverflow.ellipsis, softWrap: true, style: T.label(N.g76).copyWith(height: 1.2)))),
        _gap(4),
        Container(
          width: c.w * 2 + 2,
          height: h * 2 + 2,
          decoration: BoxDecoration(color: N.g10, border: Border.all(color: N.g20)),
          child: ClipRect(
            child: OverflowBox(
              alignment: Alignment.topLeft,
              minWidth: c.w,
              maxWidth: c.w,
              minHeight: h,
              maxHeight: h,
              child: Transform.scale(scale: 2, alignment: Alignment.topLeft, child: SizedBox(width: c.w, height: h, child: c.child)),
            ),
          ),
        ),
      ]);
}

class _Grp {
  const _Grp(this.title, this.cells, {this.note});
  final String title;
  final List<_Cell> cells;
  final String? note; // what this group does not have, or a rule that holds for all of it
}

/// A group's plates in rows of one width: consecutive cells of the same width share a row (and its height).
List<Widget> _runs(List<_Cell> cells) {
  final out = <Widget>[];
  var i = 0;
  while (i < cells.length) {
    var j = i;
    while (j < cells.length && cells[j].w == cells[i].w) {
      j++;
    }
    final run = cells.sublist(i, j), h = run.map((x) => x.h).fold<double>(0, math.max);
    if (out.isNotEmpty) {
      out.add(_gap(16));
    }
    out.add(Wrap(spacing: 16, runSpacing: 16, children: [for (final x in run) _Z(x, h)]));
    i = j;
  }
  return out;
}

class _PartsSheet extends StatefulWidget {
  const _PartsSheet(this.heading, this.groups);
  final String heading;
  final List<_Grp> groups;
  @override
  State<_PartsSheet> createState() => _PartsSheetState();
}

/// The sheet scrolls both ways with visible bars, PageUp / PageDown / Home / End / the arrows work, and a JUMP TO bar goes to a group: nothing is cut unseen, every lower row is reachable.
class _PartsSheetState extends State<_PartsSheet> {
  final v = ScrollController(), h = ScrollController();
  late final keys = [for (final _ in widget.groups) GlobalKey()];

  @override
  void dispose() {
    v.dispose();
    h.dispose();
    super.dispose();
  }

  void _go(int i) {
    final c = keys[i].currentContext;
    if (c != null) {
      Scrollable.ensureVisible(c, duration: Mo.dur, curve: Mo.ease, alignment: 0);
    }
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (!(e is KeyDownEvent || e is KeyRepeatEvent) || !v.hasClients) {
      return KeyEventResult.ignored;
    }
    final vp = v.position.viewportDimension, k = e.logicalKey;
    double? to;
    if (k == LogicalKeyboardKey.pageDown || k == LogicalKeyboardKey.space) {
      to = v.offset + vp * .9;
    } else if (k == LogicalKeyboardKey.pageUp) {
      to = v.offset - vp * .9;
    } else if (k == LogicalKeyboardKey.arrowDown) {
      to = v.offset + 80;
    } else if (k == LogicalKeyboardKey.arrowUp) {
      to = v.offset - 80;
    } else if (k == LogicalKeyboardKey.home) {
      to = 0;
    } else if (k == LogicalKeyboardKey.end) {
      to = v.position.maxScrollExtent;
    }
    if (to == null) {
      return KeyEventResult.ignored;
    }
    v.jumpTo(to.clamp(0.0, v.position.maxScrollExtent));
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g07,
        child: Focus(
          autofocus: true,
          onKeyEvent: _key,
          child: LayoutBuilder(builder: (ctx, box) {
            // The sheet is as wide as the viewport, never narrower than the widest plate; wider than the viewport it scrolls sideways.
            final widest = [for (final g in widget.groups) for (final x in g.cells) x.w * 2 + 2].fold<double>(0, math.max);
            final inner = math.max(box.maxWidth - 32 - 14, widest);
            return RawScrollbar(
              controller: v,
              thumbVisibility: true,
              thumbColor: N.g38,
              radius: const Radius.circular(3),
              thickness: 6,
              child: RawScrollbar(
                controller: h,
                thumbVisibility: true,
                notificationPredicate: (n) => n.depth == 1,
                scrollbarOrientation: ScrollbarOrientation.bottom,
                thumbColor: N.g38,
                radius: const Radius.circular(3),
                thickness: 6,
                child: SingleChildScrollView(
                  controller: v,
                  child: SingleChildScrollView(
                    controller: h,
                    scrollDirection: Axis.horizontal,
                    child: Padding(
                      padding: const EdgeInsets.fromLTRB(16, 16, 16 + 14, 16 + 14),
                      child: SizedBox(
                        width: inner,
                        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                          Row(children: [
                            Flexible(child: Text(widget.heading, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title())),
                            const SizedBox(width: 8),
                            Text('@200%', style: T.micro(N.g76)),
                          ]),
                          _gap(8),
                          Wrap(spacing: 12, runSpacing: 8, crossAxisAlignment: WrapCrossAlignment.center, children: [
                            Text('JUMP TO', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                            for (var i = 0; i < widget.groups.length; i++) _Word(widget.groups[i].title, onTap: () => _go(i)),
                            Text('PageDown, End and the arrows scroll', style: T.label(N.g76)),
                          ]),
                          for (var i = 0; i < widget.groups.length; i++) ...[
                            _gap(24),
                            KeyedSubtree(key: keys[i], child: Text(widget.groups[i].title.toUpperCase(), style: T.micro(N.g76).copyWith(letterSpacing: .6))),
                            if (widget.groups[i].note != null) ...[_gap(4), SizedBox(width: 640, child: Text(widget.groups[i].note!, style: T.label(N.g76).copyWith(height: 1.4)))],
                            _gap(8),
                            ..._runs(widget.groups[i].cells),
                          ],
                          _gap(24),
                          Text('END OF SHEET', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                        ]),
                      ),
                    ),
                  ),
                ),
              ),
            );
          }),
        ),
      );
}

WidgetbookUseCase _parts(String name, String heading, List<_Grp> groups) => WidgetbookUseCase(name: name, builder: (c) => _PartsSheet(heading, groups));

_P _pp(String id) => [_fractal, _glow, _hue, _blur, _echo, _turb, _vig].expand((f) => f.params).firstWhere((p) => p.id == id);

_Vals _vals([Map<String, double> set = const {}]) {
  final v = _Vals();
  set.forEach((k, x) => v.v[k] = x);
  return v;
}

/// One property row at 282 px (or [w]) as a cell.
_Cell _row(String cap, String id, {Map<String, double> set = const {}, _Pose? pose, int lang = 0, bool jp = false, double w = 282, String query = '', Color? tick, double h = 28}) {
  final p = _pp(id);
  return _Cell(cap, _PRow(p, _vals(set), lang: lang, jp: jp, pose: pose, query: query, tick: tick), w: w, h: h);
}

Widget _pad(Widget c) => Padding(padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4), child: c);

/// The number cell in every state, in one group.
List<_Cell> _numCells() => [
      _Cell('rest (grip ticks on the left)', _pad(_Num(value: 960, unit: 'px', onDelta: (_) {})), w: 140, h: 28),
      _Cell('hover', _pad(_Num(value: 960, unit: 'px', pose: _Pose.hover, onDelta: (_) {})), w: 140, h: 28),
      _Cell('pressed (drag)', _pad(_Num(value: 960, unit: 'px', pose: _Pose.pressed, onDelta: (_) {})), w: 140, h: 28),
      _Cell('focused', _pad(_Num(value: 960, unit: 'px', pose: _Pose.focused, onDelta: (_) {})), w: 140, h: 28),
      _Cell('typing (click)', _pad(_Num(value: 960, unit: 'px', typing: true, onDelta: (_) {})), w: 140, h: 28),
      _Cell('disabled (no well, no grip)', _pad(_Num(value: 960, unit: 'px', enabled: false, onDelta: (_) {})), w: 140, h: 28),
      _Cell('mixed (dash + word)', _pad(_Num(value: 0, mixed: true, onDelta: (_) {})), w: 140, h: 28),
      _Cell('changed (edge under)', _pad(_Num(value: 960, unit: 'px', changed: true, onDelta: (_) {})), w: 140, h: 28),
      _Cell('locked word', _pad(_Num(value: 100, word: 'Locked', enabled: false, onDelta: (_) {})), w: 140, h: 28),
      _Cell('negative, 3 decimals', _pad(_Num(value: -.033, dec: 3, onDelta: (_) {})), w: 140, h: 28),
      _Cell('narrow 96: no unit written', _pad(_Num(value: 960, unit: 'px', onDelta: (_) {})), w: 96, h: 28),
      _Cell('long value, ellipsis', _pad(_Num(value: 1234567890.12, dec: 2, onDelta: (_) {})), w: 96, h: 28),
    ];

List<_Cell> _wordCells() => [
      _Cell('rest', _pad(_Word('Unlink', onTap: () {})), w: 96, h: 28),
      _Cell('hover', _pad(_Word('Unlink', pose: _Pose.hover, onTap: () {})), w: 96, h: 28),
      _Cell('pressed', _pad(_Word('Unlink', pose: _Pose.pressed, onTap: () {})), w: 96, h: 28),
      _Cell('focused', _pad(_Word('Unlink', pose: _Pose.focused, onTap: () {})), w: 96, h: 28),
      _Cell('selected', _pad(_Word('Solo', on: true, onTap: () {})), w: 96, h: 28),
      _Cell('disabled (no action)', _pad(const _Word('Unlink')), w: 96, h: 28),
      _Cell('long EN, long JP, ellipsis', _pad(_words([const _Word('Show each layer', onTap: _noop), const _Word('全レイヤーを表示', onTap: _noop)])), w: 160, h: 28),
    ];

List<_Cell> _pickCells() => [
      _Cell('rest', _pad(_Pick(text: 'Soft Linear', onTap: () {})), w: 140, h: 28),
      _Cell('focused', _pad(_Pick(text: 'Soft Linear', pose: _Pose.focused, onTap: () {})), w: 140, h: 28),
      _Cell('hover', _pad(_Pick(text: 'Soft Linear', pose: _Pose.hover, onTap: () {})), w: 140, h: 28),
      _Cell('pressed', _pad(_Pick(text: 'Soft Linear', pose: _Pose.pressed, onTap: () {})), w: 140, h: 28),
      _Cell('long text, ellipsis', _pad(_Pick(text: 'Allow HDR Results (Clamp Later)', onTap: () {})), w: 140, h: 28),
      _Cell('JP', _pad(_Pick(text: '色相を反転', onTap: () {})), w: 140, h: 28),
    ];

List<_Cell> _colorCells() => [
      _Cell('rest', _pad(_ColorCell(index: 4, onTap: () {})), w: 140, h: 28),
      _Cell('hover', _pad(_ColorCell(index: 4, pose: _Pose.hover, onTap: () {})), w: 140, h: 28),
      _Cell('pressed', _pad(_ColorCell(index: 4, pose: _Pose.pressed, onTap: () {})), w: 140, h: 28),
      _Cell('focused', _pad(_ColorCell(index: 4, pose: _Pose.focused, onTap: () {})), w: 140, h: 28),
      _Cell('dark swatch', _pad(_ColorCell(index: 3, onTap: () {})), w: 140, h: 28),
    ];

List<_Cell> _sectCells() => [
      _Cell('closed (rest)', _Sect('Transform', count: 14, open: false, onTap: () {}), w: 280),
      _Cell('open', _Sect('Transform', count: 14, open: true, onTap: () {}), w: 280),
      _Cell('hover', _Sect('Transform', count: 14, open: false, pose: _Pose.hover, onTap: () {}), w: 280),
      _Cell('pressed', _Sect('Transform', count: 14, open: false, pose: _Pose.pressed, onTap: () {}), w: 280),
      _Cell('selected: g20 + gutter tick', _Sect('Transform', count: 14, open: false, selected: true, onTap: () {}), w: 280),
      _Cell('focused', _Sect('Transform', count: 14, open: false, pose: _Pose.focused, onTap: () {}), w: 280),
      _Cell('plain, not foldable (no hover)', const _Sect('Transform', count: 14), w: 280),
      _Cell('long EN + sub, ellipsis', const _Sect('Transform Anchor Point Offset', count: 14, sub: 'scaled by layer distance'), w: 280),
      _Cell('JP', const _Sect('変形 と アンカーポイント', count: 7, upper: false), w: 280),
      _Cell('dock 200, long, ellipsis', _Sect('Transform Anchor Point Offset', count: 14, open: false, onTap: () {}), w: 200),
      _Cell('dock 200, open, hover', _Sect('Transform', count: 14, open: true, pose: _Pose.hover, onTap: () {}), w: 200),
    ];

List<_Cell> _foldCells() => [
      _Cell('closed: "14 more" (the one count)', _FoldRow(count: 14, open: false, onTap: () {}), w: 280, h: 36),
      _Cell('open: the count is gone', _FoldRow(count: 14, open: true, onTap: () {}), w: 280, h: 36),
      _Cell('hover', _FoldRow(count: 14, open: false, pose: _Pose.hover, onTap: () {}), w: 280, h: 36),
      _Cell('pressed', _FoldRow(count: 14, open: false, pose: _Pose.pressed, onTap: () {}), w: 280, h: 36),
      _Cell('focused', _FoldRow(count: 14, open: false, pose: _Pose.focused, onTap: () {}), w: 280, h: 36),
      _Cell('open, hover', _FoldRow(count: 14, open: true, pose: _Pose.hover, onTap: () {}), w: 280, h: 36),
      _Cell('open, pressed', _FoldRow(count: 14, open: true, pose: _Pose.pressed, onTap: () {}), w: 280, h: 36),
      _Cell('open, focused', _FoldRow(count: 14, open: true, pose: _Pose.focused, onTap: () {}), w: 280, h: 36),
      _Cell('3-digit count', _FoldRow(count: 128, open: false, onTap: () {}), w: 280, h: 36),
      _Cell('dock 200', _FoldRow(count: 14, open: false, onTap: () {}), w: 200, h: 36),
      _Cell('narrow 120, ellipsis', _FoldRow(count: 14, open: true, onTap: () {}), w: 120, h: 36),
    ];

List<_Cell> _rowCells() => [
      _row('rest', 'fn.contrast'),
      _row('changed: bright label, edge under the cell, reset', 'fn.contrast', set: {'fn.contrast': 140}),
      _row('changed toggle: underlined label', 'gb.edge', set: {'gb.edge': 0}),
      _row('changed choice: edge under the cell', 'fn.type', set: {'fn.type': 2}),
      _row('hover', 'fn.contrast', pose: _Pose.hover),
      _row('number, px', 'fn.ox'),
      _row('choice', 'fn.type'),
      _row('choice, long text, ellipsis', 'fn.overflow', set: {'fn.overflow': 3}, w: 200, h: 52),
      _row('colour', 'gl.ca'),
      _row('toggle, off', 'gb.edge', set: {'gb.edge': 0}),
      _row('toggle, on', 'gb.edge'),
      _row('number with unit', 'fn.scale'),
      _row('long EN label, ellipsis', 'fn.anchor'),
      _row('long JP label, ellipsis', 'fn.anchor', lang: 2),
      _row('narrow 200: label stacks', 'fn.contrast', set: {'fn.contrast': 140}, w: 200, h: 52),
    ];

_Cell _hint(String cap, String text, {Color color = N.g76, double w = 282}) => _Cell(cap, _Hint(text, color: color), w: w, h: 28);

// ---- the sheets --------------------------------------------------------------------------------------------------------------

Widget _lead(Widget w) => Padding(padding: const EdgeInsets.fromLTRB(12, 4, 12, 4), child: Align(alignment: Alignment.centerLeft, child: w));

/// One row of the pick-whip panel at 1x: ring, label, value cell; [bound] = followed (orange tick, "← source"), [target] = lit, [grab] = being dragged, [over] = under the pointer.
Widget _ringRow(String label, {bool bound = false, bool target = false, bool grab = false, bool over = false, bool dim = false, bool far = false, _Pose? pose}) => Opacity(
      opacity: dim ? .3 : 1,
      child: _Gut(
        tick: bound ? _fol : (target ? _fol.withValues(alpha: .6) : null),
        child: Container(
          constraints: const BoxConstraints(minHeight: 28),
          padding: const EdgeInsets.fromLTRB(12, 4, 12, 4),
          color: over ? _fol.withValues(alpha: .16) : (grab ? N.g20 : null),
          child: Row(children: [
            _Ring(bound: bound, active: grab, pose: pose, onStart: far ? null : (_) {}),
            Expanded(flex: 5, child: far ? _FarLabel(label.split(' ').first, label.split(' ').skip(1).join(' ')) : _Mid(label, T.label(bound || target || grab ? N.g95 : N.g63).copyWith(fontSize: 11, height: 1.2))),
            const SizedBox(width: 8),
            Expanded(flex: 6, child: bound ? _BoundCell(text: 'ring_glow · Opacity', onUnlink: () {}) : _Num(value: 100, unit: '%', enabled: !far, onDelta: (_) {})),
          ]),
        ),
      ),
    );

WidgetbookUseCase _sheetFold() => _parts('Parts @200% Folds and rows', 'Inspector I7 Fold: headings, fold row, property rows', [
      _Grp('Group heading', _sectCells(), note: 'Rest g10 (the plate), hover g15, selected g20 + gutter tick, pressed g26, focus a 1 px g63 ring. Not designed: disabled (a heading is never inert; a locked layer greys its rows, not its headings).'),
      _Grp('Fold row', _foldCells(), note: 'Closed says how many rows it hides, open says nothing (the rows are there). Not designed: selected, disabled (a toggle, not a choice; it exists only when there is something to fold). Closed and open chevrons share one box and one left edge.'),
      _Grp('Property row', _rowCells(), note: 'Not designed: pressed or focused on the row itself (the row is not a control; its cell is). Choice cells align left with a chevron (a menu, a word); number cells align right (a quantity). The unit is written only in a cell 96 px or wider, at 10 px, deg like % and px.'),
    ]);

WidgetbookUseCase _sheetFind() => _parts('Parts @200% Search field', 'Inspector I7 Find: field, match, empty result', [
      _Grp('Field', [
        _Cell('empty, rest', _pad(_Field(ctrl: TextEditingController(), hint: 'Find a property')), w: 260, h: 36),
        _Cell('empty, hover', _pad(_Field(ctrl: TextEditingController(), hint: 'Find a property', pose: _Pose.hover)), w: 260, h: 36),
        _Cell('empty, focused', _pad(_Field(ctrl: TextEditingController(), hint: 'Find a property', pose: _Pose.focused)), w: 260, h: 36),
        _Cell('typed, count + Clear', _pad(_Field(ctrl: TextEditingController(text: 'offset'), hint: 'Find a property', count: '4 of 26')), w: 260, h: 36),
        _Cell('typed JP', _pad(_Field(ctrl: TextEditingController(text: 'ノイズ'), hint: 'Find a property', count: '2 of 26')), w: 260, h: 36),
        _Cell('typed, long, ends at Clear', _pad(_Field(ctrl: TextEditingController(text: 'transform anchor point offset scaled'), hint: 'Find a property', count: '1 of 26')), w: 260, h: 36),
        _Cell('typed, focused', _pad(_Field(ctrl: TextEditingController(text: 'offset'), hint: 'Find a property', count: '4 of 26', pose: _Pose.focused)), w: 260, h: 36),
        _Cell('dock 200, empty', _pad(_Field(ctrl: TextEditingController(), hint: 'Find a property')), w: 200, h: 36),
      ], note: 'Edge: rest g20, hover g38, focus g63. Not designed: disabled (Find is always on) and error (any text is a valid query).'),
      _Grp('Match highlight in a row', [
        _row('query sca', 'fn.scale', query: 'sca'),
        _row('query offset, ellipsis', 'fn.anchor', query: 'offset'),
        _row('query ノイズ (JP name)', 'fn.noise', query: 'ノイズ', lang: 2),
        _row('dock 200, stacked', 'fn.ox', query: 'offset', w: 200, h: 52),
      ]),
      _Grp('No match', [
        _Cell('sentence + way back', _NoMatch(what: 'No property "bloom"', next: 'Show all 26', onNext: () {}), w: 282, h: 60),
        _Cell('dock 200', _NoMatch(what: 'No property "bloom"', next: 'Show all 26', onNext: () {}), w: 200, h: 60),
      ], note: 'The button is a quiet button (rest and primary are in the Words sheet; its hover and focus belong to the shared part).'),
    ]);

WidgetbookUseCase _sheetPin() => _parts('Parts @200% Star and tick', 'Inspector I7 Pin and Recent: star, tick, row', [
      _Grp('Star', [
        _Cell('off (shown while hovering the row)', const Center(child: StarButton(on: false)), w: 48, h: 28),
        _Cell('on (always shown)', const Center(child: StarButton(on: true)), w: 48, h: 28),
        _Cell('row, rest: no star yet (label x is the same)', _PRow(_pp('fn.contrast'), _vals(), trailH: (h) => SizedBox(width: 24, child: h ? const StarButton(on: false, size: 12) : null)), w: 282),
        _Cell('row, hover: the star appears', _PRow(_pp('fn.contrast'), _vals(), pose: _Pose.hover, trailH: (h) => SizedBox(width: 24, child: h ? const StarButton(on: false, size: 12) : null)), w: 282),
        _Cell('row, starred', _PRow(_pp('fn.contrast'), _vals(), tick: N.g95, trailH: (h) => const SizedBox(width: 24, child: StarButton(on: true, size: 12))), w: 282),
      ], note: 'The star glyph\'s own hover, focus and pressed are the shared StarButton\'s (Media sheet). Not designed: disabled.'),
      _Grp('Row with a tick', [
        _row('starred row: tick g95', 'fn.contrast', tick: N.g95),
        _row('recent (fixed order): grey tick', 'fn.contrast', tick: N.g63, set: {'fn.contrast': 140}),
        _row('no tick', 'fn.contrast'),
        _row('hover', 'fn.contrast', pose: _Pose.hover, tick: N.g95),
      ]),
      _Grp('Group heading', _sectCells().take(6).toList()),
    ]);

WidgetbookUseCase _sheetClip() => _parts('Parts @200% Words and tags', 'Inspector I11 Clipboard: word buttons, tag, hint, tick', [
      _Grp('Word button', _wordCells(), note: 'g76 at rest, g95 + underline on hover, g26 fill on pressed, 1 px g63 ring on focus, selected = g20 + accent line, disabled = 50 %. The text sits on the content edge.'),
      _Grp('Tag and button', [
        _Cell('tag', _lead(const Pill('Holding Ease')), w: 140),
        _Cell('tag, JP', _lead(const Pill('イージングを保持')), w: 140),
        _Cell('button, rest', _lead(QuietButton('Paste Ease', onTap: () {})), w: 140),
        _Cell('button, primary', _lead(QuietButton('Paste Ease', primary: true, onTap: () {})), w: 140),
        _Cell('button, disabled', _lead(const _Btn('Paste')), w: 140),
      ], note: 'A tag is a label, not a control: no hover, pressed or disabled. The button\'s hover and focus are the shared QuietButton\'s (Media sheet).'),
      _Grp('Hint and status', [
        _hint('rest', 'Pasted to 4'),
        _hint('error (danger)', 'Skipped 2: no keys, no Glow.', color: Role.error),
        _hint('partial', '2 of 4 took it · 2 skipped', color: N.g76),
      ]),
      _Grp('Shared row', [
        _row('linked: attach tick', 'gl.i', tick: Role.linkedFor(Fam.attach), set: {'gl.i': 2}),
        _row('hover', 'gl.i', tick: Role.linkedFor(Fam.attach), pose: _Pose.hover),
        _row('failed: danger tick', 'gl.i', tick: Role.error),
      ]),
    ]);

WidgetbookUseCase _sheetPickShelf() => _parts('Parts @200% Value cells', 'Inspector I11 Eyedropper and Shelf: choice, colour, number cells', [
      _Grp('Choice cell', _pickCells(), note: 'Left-aligned word and chevron on purpose: it opens a menu. Not designed: disabled.'),
      _Grp('Colour cell', _colorCells(), note: 'Not designed: disabled.'),
      _Grp('Number cell', _numCells(), note: 'Right-aligned on purpose: it is a quantity. Not designed: error (a number clamps to its range, so nothing can be invalid).'),
      _Grp('Word button', _wordCells()),
    ]);

WidgetbookUseCase _sheetWhip() => _parts('Parts @200% Ring and bound cell', 'Inspector I9 Pick-whip: ring, bound cell', [
      _Grp('Ring (22 px hit, 8 px dot)', [
        _Cell('rest', _Ring(bound: false, active: false, onStart: (_) {}), w: 56, h: 28),
        _Cell('hover', _Ring(bound: false, active: false, pose: _Pose.hover, onStart: (_) {}), w: 56, h: 28),
        _Cell('active: grabbing (pressed)', const _Ring(bound: false, active: true), w: 56, h: 28),
        _Cell('bound', const _Ring(bound: true, active: false), w: 56, h: 28),
        _Cell('not a target / far value', const _Ring(bound: false, active: false), w: 56, h: 28),
      ], note: 'A ring is a pointer gesture. Not designed: keyboard focus.'),
      _Grp('Bound cell', [
        _Cell('rest', _pad(_BoundCell(text: 'Glow Intensity', onUnlink: () {})), w: 200),
        _Cell('hover', _pad(_BoundCell(text: 'Glow Intensity', pose: _Pose.hover, onUnlink: () {})), w: 200),
        _Cell('long, ellipsis', _pad(_BoundCell(text: 'Letter Spacing (title, large display)', onUnlink: () {})), w: 200),
        _Cell('JP', _pad(_BoundCell(text: 'グロー強度', onUnlink: () {})), w: 200),
      ], note: 'Unlink is always written, so hovering moves nothing. Not designed: disabled.'),
      _Grp('Row with a ring', [
        _Cell('rest', _ringRow('Opacity'), w: 282),
        _Cell('grabbed (g20)', _ringRow('Opacity', grab: true), w: 282),
        _Cell('same kind: lit tick', _ringRow('Scale', target: true), w: 282),
        _Cell('under the pointer: orange tint', _ringRow('Scale', target: true, over: true), w: 282),
        _Cell('other kind: 30 %', _ringRow('Position X', dim: true), w: 282),
        _Cell('followed', _ringRow('Scale', bound: true), w: 282),
        _Cell('far value: layer over property, no ring to pull', _ringRow('ring_glow Opacity', far: true), w: 282, h: 40),
      ]),
    ]);

WidgetbookUseCase _sheetLasso() => _parts('Parts @200% Range and follow', 'Inspector I9 Lasso: range cells, follow button, hint', [
      _Grp('Range number cells', _numCells().take(6).toList()),
      _Grp('Buttons', [
        _Cell('Follow, primary', _lead(QuietButton('Follow', primary: true, onTap: () {})), w: 140),
        _Cell('Follow, disabled (nothing selected)', _lead(const _Btn('Follow')), w: 140),
        _Cell('= now: rest', _pad(_Word('= now', onTap: () {})), w: 140),
        _Cell('= now: hover', _pad(_Word('= now', pose: _Pose.hover, onTap: () {})), w: 140),
        _Cell('= now: pressed', _pad(_Word('= now', pose: _Pose.pressed, onTap: () {})), w: 140),
        _Cell('= now: focused', _pad(_Word('= now', pose: _Pose.focused, onTap: () {})), w: 140),
      ]),
      _Grp('Status', [
        _hint('empty (nothing drawn)', '0 of 18'),
        _hint('selected', '12 of 18'),
        _hint('dock 224', '12 of 18', w: 224),
      ]),
    ]);

WidgetbookUseCase _sheetMacro() => _parts('Parts @200% Macro row', 'Inspector I9 Macro and One-line: macro row, bound cell, range', [
      _Grp('Macro row', [
        _Cell('rest', _MacroRow(c: _pool[0], lo: .4, hi: 2.4, t: .5, onLo: (_) {}, onHi: (_) {}, onUnlink: () {}), w: 282, h: 60),
        _Cell('hover', _MacroRow(c: _pool[0], lo: .4, hi: 2.4, t: .5, pose: _Pose.hover, onLo: (_) {}, onHi: (_) {}, onUnlink: () {}), w: 282, h: 60),
        _Cell('long name, middle ellipsis', _MacroRow(c: _pool[8], lo: -2, hi: 24, t: .75, onLo: (_) {}, onHi: (_) {}, onUnlink: () {}), w: 282, h: 60),
        _Cell('dock 200', _MacroRow(c: _pool[8], lo: -2, hi: 24, t: .25, onLo: (_) {}, onHi: (_) {}, onUnlink: () {}), w: 200, h: 60),
      ], note: 'Unlink is always written. Not designed: selected, disabled.'),
      _Grp('Not linked row', [
        _Cell('rest', Container(constraints: const BoxConstraints(minHeight: 28), padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4), child: Row(children: [Expanded(child: Text('Blur Radius', style: T.label(N.g63).copyWith(fontSize: 11))), _Word('Link', onTap: () {})])), w: 282),
        _Cell('hover', Container(constraints: const BoxConstraints(minHeight: 28), padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4), color: N.g15, child: Row(children: [Expanded(child: Text('Blur Radius', style: T.label(N.g63).copyWith(fontSize: 11))), _Word('Link', pose: _Pose.hover, onTap: () {})])), w: 282),
      ]),
      _Grp('Bound cell', [
        _Cell('rest', _pad(_BoundCell(text: 'Macro 1', onUnlink: () {})), w: 200),
        _Cell('hover', _pad(_BoundCell(text: 'Macro 1', pose: _Pose.hover, onUnlink: () {})), w: 200),
      ]),
      _Grp('Number cell', _numCells().take(6).toList()),
    ]);

/// A specimen of one card header of the stack (I8): number, grip, chevron, name, status word, switch.
Widget _cardHead(String name, {bool sel = false, bool on = true, String? status, Color statusColor = N.g76, bool muted = false, bool failed = false, bool open = false}) => Padding(
      padding: const EdgeInsets.fromLTRB(12, 0, 12, 0),
      child: Container(
        decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(4)),
        child: _Gut(
          tick: failed ? Role.error : (sel ? N.g95 : null),
          child: Opacity(
            opacity: muted ? .6 : 1,
            child: SizedBox(
              height: status == null ? 32 : 40,
              child: Row(children: [
                const SizedBox(width: 6),
                SizedBox(width: 20, child: Center(child: Text('02', style: _cnt(N.g76)))),
                SizedBox(width: 12, height: 12, child: CustomPaint(painter: _GripPaint())),
                const SizedBox(width: 4),
                _Chev(open),
                const SizedBox(width: 4),
                Expanded(
                  child: Column(mainAxisAlignment: MainAxisAlignment.center, crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Text(name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g63).copyWith(decoration: on ? null : TextDecoration.lineThrough)),
                    if (status != null) ...[const SizedBox(height: 4), statusColor == Role.error ? Row(mainAxisSize: MainAxisSize.min, children: [const ErrMark(size: 10, gap: 4), Text(status, style: T.label(statusColor))]) : Text(status, style: T.label(statusColor))],
                  ]),
                ),
                PillSwitch(on: on),
                const SizedBox(width: 8),
              ]),
            ),
          ),
        ),
      ),
    );

WidgetbookUseCase _sheetStack() => _parts('Parts @200% Card header', 'Inspector I8 Stack: card header in every state, effect head, words', [
      _Grp('Card header', [
        _Cell('rest, folded', _cardHead('Gaussian Blur'), w: 282, h: 40),
        _Cell('open', _cardHead('Gaussian Blur', open: true), w: 282, h: 40),
        _Cell('selected: gutter tick g95', _cardHead('Gaussian Blur', sel: true), w: 282, h: 40),
        _Cell('bypassed: struck + word', _cardHead('Gaussian Blur', on: false, status: 'Bypassed'), w: 282, h: 40),
        _Cell('failed: danger tick + word', _cardHead('Glow', failed: true, status: 'Failed', statusColor: Role.error), w: 282, h: 40),
        _Cell('muted by solo', _cardHead('Fractal Noise', muted: true, status: 'Muted by solo'), w: 282, h: 40),
        _Cell('long name, ellipsis', _cardHead('Transform Anchor Point Offset (Scaled by Layer Distance)'), w: 282, h: 40),
        _Cell('JP', _cardHead('ガウスブラー'), w: 282, h: 40),
        _Cell('dock 224', _cardHead('Transform Anchor Point Offset'), w: 224, h: 40),
      ], note: 'Hover and focus of the chevron and name, the grip (grab cursor) and the switch are the live panel\'s: they cannot be pinned here. Not designed: disabled (a bypassed card is still editable).'),
      _Grp('Effect head (inside the Inspector)', [
        _Cell('on', _FxHead('Glow', '11 properties', true, (_) {}), w: 282, h: 40),
        _Cell('off', _FxHead('Glow', '11 properties', false, (_) {}), w: 282, h: 40),
        _Cell('long, ellipsis', _FxHead('Transform Anchor Point Offset (Scaled by Layer Distance)', '11 properties', true, (_) {}), w: 282, h: 40),
        _Cell('JP, long, ellipsis', _FxHead('タービュレントディスプレイス (長い名前のテスト)', '6 properties', true, (_) {}), w: 282, h: 40),
      ], note: 'Not designed: hover, pressed, focus on the head itself (only its switch acts).'),
      _Grp('Word buttons (Solo, Swap, Retry)', _wordCells().take(7).toList()),
    ]);

WidgetbookUseCase _sheetMixed() => _parts('Parts @200% Mixed cells', 'Inspector I12 Mixed: number cell states, mixed word, locked word', [
      _Grp('Number cell', _numCells()),
      _Grp('Label and control rows', [
        _Cell('rest', _VRow('Position X', _Num(value: 960, unit: 'px', onDelta: (_) {})), w: 282),
        _Cell('mixed', _VRow('Position X', _Num(value: 0, mixed: true, onDelta: (_) {})), w: 282),
        _Cell('mixed + grey tick', _VRow('Position X', _Num(value: 0, mixed: true, onDelta: (_) {}), tick: N.g44), w: 282),
        _Cell('locked layer', _VRow('Position X', _Num(value: 960, word: 'Locked', enabled: false, onDelta: (_) {})), w: 282),
        _Cell('long JP label', _VRow('変形アンカーポイントのオフセット', _Num(value: 0, mixed: true, onDelta: (_) {})), w: 282, h: 44),
        _Cell('dock 200 stacked', _VRow('Position X', _Num(value: 0, mixed: true, onDelta: (_) {})), w: 200, h: 52),
      ], note: 'A row is not a control: hover, pressed and focus belong to its cell.'),
      _Grp('Status', [
        _hint('range under Mixed', '120 to 1,840'),
        _hint('error (danger)', 'Could not write 2 layers.', color: Role.error),
      ]),
    ]);

WidgetbookUseCase _sheetStagger() => _parts('Parts @200% Step and basis', 'Inspector I12 Stagger, Falloff, Grab: number, word choice, status', [
      _Grp('Step number cell', _numCells().take(6).toList()),
      _Grp('Order and shape (the one segmented bar of the Inspector)', [
        _Cell('Stage selected', _pad(Segmented(items: const ['Stage', 'Timeline', 'Picked'], index: 0, expand: true)), w: 280, h: 36),
        _Cell('Timeline selected', _pad(Segmented(items: const ['Stage', 'Timeline', 'Picked'], index: 1, expand: true)), w: 280, h: 36),
        _Cell('Picked selected', _pad(Segmented(items: const ['Stage', 'Timeline', 'Picked'], index: 2, expand: true)), w: 280, h: 36),
        _Cell('Falloff shape, Linear', _pad(Segmented(items: const ['Linear', 'Ease', 'Step'], index: 0, expand: true)), w: 280, h: 36),
        _Cell('Grab property, long names', _pad(Segmented(items: const ['Position X', 'Position Y', 'Scale'], index: 2, expand: true)), w: 280, h: 36),
      ], note: 'Selected = g20 pill + accent underline; hover, pressed and focus are the shared Segmented\'s (Controls sheet). Not designed: disabled.'),
      _Grp('Grab row, layer rows', [
        _row('all layers: row stands for 6', 'fn.contrast'),
        _row('mixed after grab', 'fn.contrast', set: {'fn.contrast': 140}),
        _Cell('locked layer skipped', _VRow('title_card', _Num(value: 100, word: 'Locked', enabled: false, onDelta: (_) {})), w: 282),
      ]),
      _Grp('Status', [
        _hint('rest', 'title_card  f14'),
        _hint('off the Stage', '3 off Stage', color: N.g91),
      ]),
    ]);

WidgetbookUseCase _sheetControls() => _parts('Parts @200% Number choice colour', 'Inspector X controls: number grammar, choice, colour, row', [
      _Grp('Number cell', _numCells()),
      _Grp('Choice cell', _pickCells(), note: 'Left-aligned word and chevron on purpose: it opens a menu. Number cells are right-aligned.'),
      _Grp('Colour cell', _colorCells()),
      _Grp('Property row', _rowCells().take(7).toList()),
    ]);

WidgetbookUseCase _sheetStates() => _parts('Parts @200% State block', 'Inspector X states: dead-end block, heading, switch, hint', [
      _Grp('State block', [
        _Cell('neutral', const _StateBlock('Nothing is selected', ['Select the top layer', 'Open Browser']), w: 282, h: 76),
        _Cell('error (danger tick) + one reason', const _StateBlock('Glow could not run', ['Retry', 'Open source', 'Bypass'], why: 'Shader error, line 14', bad: true), w: 282, h: 92),
        _Cell('long JP', const _StateBlock('タービュレントディスプレイスを実行できません', ['再試行', 'ソースを開く'], why: 'シェーダーのエラー (14 行目)', bad: true), w: 282, h: 104),
        _Cell('dock 200', const _StateBlock('city_pass.mov is missing', ['Relink', 'Replace with solid'], bad: true), w: 200, h: 88),
      ], note: 'A block is a title and the words that act (and at most one short reason), no sentences; it is not a control: no hover or pressed (its words have their own, see Words). Not designed: disabled.'),
      _Grp('Heading', _sectCells().take(6).toList()),
      _Grp('Switch and word', [
        _Cell('on (switch first, word to its right)', _pad(_Tog(true, (_) {})), w: 140),
        _Cell('off', _pad(_Tog(false, (_) {})), w: 140),
        _Cell('Locked number', _VRow('Opacity', _Num(value: 100, unit: '%', enabled: false, word: 'Locked', onDelta: (_) {})), w: 282),
      ], note: 'The switch\'s hover, pressed and focus are the shared PillSwitch\'s (Controls sheet).'),
      _Grp('Hint', [
        _hint('rest', 'Saved'),
        _hint('error', 'Glow failed on 1 layer.', color: Role.error),
      ]),
    ]);
