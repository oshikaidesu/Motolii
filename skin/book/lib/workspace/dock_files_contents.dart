// dock_files_contents.dart: what is in the open folder: tiles (a film looked through by the hand) or rows, the in-use mark, the foot that says the picked one.
part of 'dock_files.dart';

class _Contents extends StatelessWidget {
  const _Contents(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final path = _pathOf(c, c.mem.folder), here = path.last;
    final root = here.disk ? here : path.first; // (on the disk a search stays in the open folder and the folders already read below it)
    final searching = c.query.isNotEmpty;
    final List<FileNode> items;
    if (searching) {
      final found = <FileNode>[];
      void walk(FileNode n) {
        for (final k in n.kids) {
          if (k.folder) {
            if (k.listed) walk(k);
          } else if (DockTags.hit(c.query, k.name, DockTags.of('Media', k.id, {_kindOf(k).toLowerCase(), if (k.id.contains('.')) k.id.split('.').last.toLowerCase()}))) {
            found.add(k);
          }
        }
      }

      walk(root);
      items = found;
    } else {
      if (c.mem.mediaDeep) {   // (the folder and every folder below it, their files together: a folder looked at as one pile)
        final all = <FileNode>[];
        void below(FileNode f, int depth) {
          if (f.disk && !f.listed && depth > 0) _read(f.id);
          for (final k in f.kids) {
            if (k.folder) {
              if (depth < 4) below(k, depth + 1);
            } else {
              all.add(k);
            }
          }
        }

        below(here, 0);
        items = [...here.kids.where((k) => k.folder), ...all];
      } else {
        items = [...here.kids.where((k) => k.folder), ...here.kids.where((k) => !k.folder)];
      }
    }
    final kinds = <String, int>{};
    for (final n in items) {
      kinds.update(_kindOf(n), (v) => v + 1, ifAbsent: () => 1);
    }
    final chip = kinds.containsKey(c.chip) ? c.chip : 'All';
    final shown = chip == 'All' ? items : items.where((n) => _kindOf(n) == chip).toList();
    final sel = shown.where((n) => n.id == c.sel && !n.folder).firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _Crumbs(path, c, searching: searching),
        DockChips([('All', items.length), for (final e in kinds.entries) (e.key, e.value)], active: chip, onPick: c.pickChip),
        Expanded(
          child: shown.isEmpty
              ? DockEmpty(c.query)
              : c.mem.mediaList
              ? ListView(
                  padding: EdgeInsets.zero,
                  children: [for (final n in shown) _Row(n, c, where: searching ? (n.disk ? _base(_dir(n.id)) : '') : null)],
                )
              : LayoutBuilder(
                  builder: (context, box) {
                    final cols = (box.maxWidth / 78).floor().clamp(2, 8), w = (box.maxWidth - WsT.gutter * (cols - 1)) / cols;
                    return ListView(
                      padding: EdgeInsets.only(top: WsT.gutter),
                      children: [
                        Wrap(
                          spacing: WsT.gutter,
                          runSpacing: WsT.gutter,
                          children: [for (final n in shown) SizedBox(width: w, child: _Tile(n, c))],
                        ),
                      ],
                    );
                  },
                ),
        ),
        if (sel != null) _Footer(sel, c),
      ],
    );
  }
}

/// A file's picture: its thumbnail if it is media, otherwise a flat tile with its kind's mark.
class _Art extends StatelessWidget {
  const _Art(this.n, {this.step = 0});
  final FileNode n;
  final int step;   // (a film: the picture this far along it, of eight)
  @override
  Widget build(BuildContext context) {
    if (n.disk && (n.g == DockG.video || n.g == DockG.image)) {
      // A file of the disk shows its own picture: a film its first one (the inside makes it), a photo itself (Flutter
      // reads it); what has none to show (a vector picture, until it is made) is a flat tile with its kind's mark.
      final flat = Container(color: WsT.well, alignment: Alignment.center, child: DockBadge(n.g, Grey.g63, size: Shape.z(26)));
      if (n.g == DockG.video) {
        return FutureBuilder<String?>(
          future: _thumbnailAtOf?.call(n.id, step, 8),
          builder: (context, made) => made.data == null ? flat : ColoredBox(color: WsT.well, child: Image.file(File(made.data!), fit: BoxFit.contain, gaplessPlayback: true)),   // (the whole picture, not cut to a square)
        );
      }
      final photo = RegExp(r'\.(png|jpe?g|webp)$', caseSensitive: false).hasMatch(n.id);
      return photo ? ColoredBox(color: WsT.well, child: Image.file(File(n.id), fit: BoxFit.contain, cacheWidth: 240, errorBuilder: (_, _, _) => flat)) : flat;
    }
    return switch (n.g) {
      DockG.font => Container(
        color: WsT.toneText,
        alignment: Alignment.center,
        child: Text('Aa', style: T.title(WsT.onAccent).copyWith(fontSize: T.px(18), fontWeight: FontWeight.w700)),
      ),
      DockG.folder => Container(
        color: WsT.well,
        alignment: Alignment.center,
        child: DockGlyph(DockG.folder, size: Shape.z(26), color: WsT.accentInk),
      ),
      _ => Container(color: WsT.well, alignment: Alignment.center, child: DockBadge(n.g, n.g == DockG.comp ? WsT.accent : Grey.g63, size: Shape.z(26))),
    };
  }
}

class _Tile extends StatefulWidget {
  const _Tile(this.n, this.c);
  final FileNode n;
  final DockCtx c;
  @override
  State<_Tile> createState() => _TileState();
}

/// A tile: a film is looked along by the hand's place across it, or played in the tile by the mark at its corner (eight
/// pictures along it, round and round); a file's kind is told by a small tag only where the hand is or it is picked.
class _TileState extends State<_Tile> {
  int _step = 0;
  bool _playing = false;
  Timer? _clock;

  FileNode get n => widget.n;
  bool get _film => n.disk && n.g == DockG.video;

  void _play() {
    setState(() => _playing = !_playing);
    _clock?.cancel();
    if (_playing) _clock = Timer.periodic(const Duration(milliseconds: 280), (_) => mounted ? setState(() => _step = (_step + 1) % 8) : null);
    if (!_playing) setState(() => _step = 0);
  }

  @override
  void dispose() {
    _clock?.cancel();
    super.dispose();
  }

  // (the kind's own tag; a vector picture says what it is by its file's end: a .json is a Lottie)
  String _badge(MediaType t) {
    if (t != MediaType.image || !n.disk || RegExp(r'\.(png|jpe?g|webp)$', caseSensitive: false).hasMatch(n.id)) return t.badge;
    final end = n.id.split('.').last.toLowerCase();
    return end == 'json' ? 'LOTTIE' : end.toUpperCase();
  }

  @override
  Widget build(BuildContext context) {
    final c = widget.c, sel = c.sel == n.id || _filesPicked.contains(n.id), t = _typeOf(n.g), can = _kindOfFile(n.id).isNotEmpty;
    return DockHover(
      key: ValueKey('browser:file:${n.id}'),
      onTap: () => _open(n, c),
      onDoubleTap: can && !n.folder ? () => _place(c, n) : null,   // (two presses place it; carried to the stage it does the same)
      builder: (context, h) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          AspectRatio(
            aspectRatio: 16 / 9,   // (a film's own shape: the whole of it seen, not a square cut out of it)
            child: MouseRegion(
              onHover: !_film || _playing ? null : (e) {
                final w = (context.size?.width ?? 1) * 1.0, k = (e.localPosition.dx / math.max(1, w) * 8).floor().clamp(0, 7);
                if (k != _step) setState(() => _step = k);
              },
              onExit: (_) => !_film || _playing || _step == 0 ? null : setState(() => _step = 0),
              child: Stack(
                fit: StackFit.expand,
                children: [
                  _Art(n, step: _step),
                  if (_film)
                    Positioned(
                      left: 4,
                      top: 4,
                      child: GestureDetector(
                        onTap: _play,
                        child: Container(
                          width: Shape.z(16),
                          height: Shape.z(16),
                          decoration: BoxDecoration(color: const Color(0xAA000000), borderRadius: BorderRadius.all(Shape.r(8))),
                          child: Center(child: Text(_playing ? '■' : '▶', style: TextStyle(fontSize: Shape.z(8), height: 1, color: const Color(0xFFFFFFFF)))),
                        ),
                      ),
                    )
                  else if (t != null && (h || sel))
                    Positioned(left: 3, top: 3, child: DockTag(_badge(t), hue: t.hue, edge: true)),
                  if (_film && (h || sel) && t != null) Positioned(right: 3, top: 3, child: DockTag(_badge(t), hue: t.hue, edge: true)),
                  if (n.folder && n.listed) Positioned(right: 4, bottom: 3, child: Text('${n.kids.length}', style: T.value(Grey.g63).copyWith(fontSize: T.px(10)))),
                  if (_inUse(n, c)) const Positioned(right: 3, bottom: 3, child: _InUse()),
                  if (sel || h)
                    DecoratedBox(
                      decoration: BoxDecoration(
                        border: Border.all(color: sel ? WsT.accentInk : Grey.g76, width: sel ? 2 : 1),
                      ),
                    ),
                ],
              ),
            ),
          ),
          AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: Shape.z(18),
            padding: EdgeInsets.symmetric(horizontal: Shape.s(4)),
            alignment: Alignment.centerLeft,
            color: dockFill(sel, h, WsT.card),
            child: FittedBox( // (a name wider than its tile is drawn smaller, never cut)
              fit: BoxFit.scaleDown,
              child: Text(
                n.name,
                maxLines: 1,
                style: T.micro(dockInk(sel, n.folder ? Grey.g91 : Grey.g76)).copyWith(fontWeight: sel || n.folder ? FontWeight.w700 : FontWeight.w500),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(this.n, this.c, {this.where});
  final FileNode n;
  final DockCtx c;

  /// The folder a search result sits in.
  final String? where;
  @override
  Widget build(BuildContext context) {
    final sel = c.sel == n.id;
    final meta = where == null ? _metaOf(n) : '$where · ${_metaOf(n)}';
    return DockHover(
      key: ValueKey('browser:file:${n.id}'),
      onTap: () => _open(n, c),
      builder: (context, h) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: Shape.z(34),
        margin: EdgeInsets.only(top: WsT.gutter),
        padding: EdgeInsets.only(left: Shape.s(4), right: WsT.inset),
        color: dockFill(sel, h, WsT.body),
        child: Row(
          children: [
            SizedBox(width: Shape.z(40), height: Shape.z(28), child: _Art(n)),
            SizedBox(width: Shape.s(8)),
            Expanded(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(n.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(dockInk(sel))),
                  SizedBox(height: Shape.s(4)),
                  Text(meta, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(dockInk(sel, Grey.g63))),
                ],
              ),
            ),
            if (_inUse(n, c)) ...[const _InUse(), SizedBox(width: Shape.s(6))],
          ],
        ),
      ),
    );
  }
}

class _InUse extends StatelessWidget {
  const _InUse();
  @override
  Widget build(BuildContext context) => Container(
    width: Shape.z(14),
    height: Shape.z(14),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.accent, borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
    child: DockGlyph(DockG.link, size: Shape.z(10), color: WsT.onAccent),
  );
}

/// What the picked file does here: one the inside can show is placed; a folder is gone into.
class _Footer extends StatelessWidget {
  const _Footer(this.n, this.c);
  final FileNode n;
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    // A file the inside can show is placed (as often as wanted, each a new thing from the present moment); one in use says so.
    if (_filesPicked.length > 1) {   // (several picked: placed together)
      return DockFooter(title: '${_filesPicked.length} files', meta: 'Placed one after another, in the order picked', action: 'Place ${_filesPicked.length}', fresh: true, onAction: () => _placeAll(c, _filesPicked.toList()));
    }
    final layer = _layerOf(n, c), can = _kindOfFile(n.id).isNotEmpty, m = c.ws.inside;
    // (a shape picked and a still chosen: it fills that shape, Alight Motion's Fill > Media; the button says so)
    final shape = int.tryParse(c.ws.selected ?? ''), fills = m != null && shape != null && _kindOfFile(n.id) == 'picture' && m.word(shape, 'kind') == 'shape' && m.word(shape, 'shape') != 'null';
    return FutureBuilder<String>(   // (what the media is, as GStreamer's discoverer tells it: Alight Motion's Media Info; its file's kind and size until then)
      future: can && m != null ? mediaInfoOf(m.libPath, n.id) : null,
      builder: (context, info) => DockFooter(
      preview: n.disk && (n.g == DockG.audio || n.g == DockG.video && (info.data ?? '×').isNotEmpty && !(info.data ?? '×').contains('×')) ? _Listen(n) : _Art(n),   // (sound alone, no picture in it: heard before it is placed)
      title: n.name,
      meta: c.mem.trouble[n.id] ?? [if (layer != null) 'In use', if (info.data case final i? when i.isNotEmpty) i, _metaOf(n)].join(' · '),
      action: fills ? 'Fill shape' : can ? 'Place' : 'Select',
      done: !can && layer == null,
      doneLabel: 'Folder', // (the shelf lists only folders and files the inside can place: what cannot be placed is a folder)
      fresh: can, // (placed, it is a new thing)
      lasts: switch (_kindOfFile(n.id)) { 'picture' => _StillSeconds(c), 'film' || 'sound' when m != null => _FilmRange(c, n.id, mediaSecondsOf(m.libPath, n.id)), _ => null },
      onAction: () => fills ? c.ws.after(m.later('say', thing: shape, name: 'fill_media', word: n.id)) : can ? _place(c, n) : c.ws.select(layer),
      ),
    );
  }
}

/// How long a still lasts, chosen before it is placed: Alight Motion's ruler under a photo (2 seconds as it comes);
/// End, as long as the document goes on. Kept for the next still.
class _StillSeconds extends StatelessWidget {
  const _StillSeconds(this.c);
  final DockCtx c;
  @override
  Widget build(BuildContext context) {
    final at = _stillSteps.indexOf(_stillSeconds), s = _stillSeconds;
    Widget step(String word, int by) => DockHover(
      key: ValueKey('media-still-$word'),
      onTap: at + by < 0 || at + by >= _stillSteps.length ? null : () => c.set(() => _stillSeconds = _stillSteps[at + by]),
      builder: (_, h) => Padding(padding: EdgeInsets.symmetric(horizontal: Shape.s(3)), child: Text(word == 'less' ? '−' : '+', style: T.label(h ? Grey.g95 : Grey.g63))),
    );
    return Row(mainAxisSize: MainAxisSize.min, children: [
      Text('for', style: T.label(Grey.g56)),
      step('less', -1),
      Text(s == 0 ? 'End' : '${s % 1 == 0 ? s.round() : s} s', key: const ValueKey('media-still-seconds'), style: T.label(Grey.g91)),
      step('more', 1),
    ]);
  }
}

/// The part of a film used, cut before it is placed: Alight Motion's band under a video, its two ends drawn in; the
/// stretch said in seconds of the film. Kept for that file while the shelf is open.
class _FilmRange extends StatelessWidget {
  const _FilmRange(this.c, this.file, this.seconds);
  final DockCtx c;
  final String file;
  final Future<double> seconds;
  @override
  Widget build(BuildContext context) => FutureBuilder<double>(
    future: seconds,
    builder: (context, got) {
      final len = got.data ?? 0;
      if (len <= 0) return const SizedBox.shrink();   // (a length not known: all of it, as before)
      final (a, b) = _filmRanges[file] ?? (0.0, 1.0);
      final w = Shape.z(84);
      String t(double f) => '${(f * len ~/ 60)}:${(f * len % 60).toStringAsFixed(1).padLeft(4, '0')}';
      int? end;   // (the end a drag holds: the nearer when it began)
      void to(double x) {
        final f = (x / w).clamp(0.0, 1.0);
        final (p, q) = _filmRanges[file] ?? (0.0, 1.0);
        c.set(() => _filmRanges[file] = end == 0 ? (math.min(f, q - .01), q) : (p, math.max(f, p + .01)));
      }
      return Row(mainAxisSize: MainAxisSize.min, children: [
        Text('use', style: T.label(Grey.g56)),
        SizedBox(width: Shape.s(4)),
        GestureDetector(
          key: const ValueKey('media-film-range'),
          onHorizontalDragStart: (d) => end = ((d.localPosition.dx / w) - a).abs() <= ((d.localPosition.dx / w) - b).abs() ? 0 : 1,
          onHorizontalDragUpdate: (d) => to(d.localPosition.dx),
          child: CustomPaint(size: Size(w, Shape.z(10)), painter: _RangePaint(a, b)),
        ),
        SizedBox(width: Shape.s(4)),
        Text('${t(a)}–${t(b)}', key: const ValueKey('media-film-said'), style: T.label(Grey.g91)),
      ]);
    },
  );
}

/// The film's whole length as a dim track, the part used bright between its two ends.
class _RangePaint extends CustomPainter {
  _RangePaint(this.a, this.b);
  final double a, b;
  @override
  void paint(Canvas c, Size s) {
    final p = Paint();
    c.drawRRect(RRect.fromLTRBR(0, s.height / 2 - 2, s.width, s.height / 2 + 2, const Radius.circular(2)), p..color = Grey.g26);
    c.drawRect(Rect.fromLTRB(a * s.width, s.height / 2 - 2, b * s.width, s.height / 2 + 2), p..color = WsT.accent);
    for (final f in [a, b]) {
      c.drawRRect(RRect.fromLTRBR(f * s.width - 1.5, 0, f * s.width + 1.5, s.height, const Radius.circular(1)), p..color = Grey.g95);
    }
  }
  @override
  bool shouldRepaint(_RangePaint o) => o.a != a || o.b != b;
}

final _filmRanges = <String, (double, double)>{};

const _stillSteps = [0.5, 1.0, 2.0, 3.0, 5.0, 10.0, 0.0];   // (0: to the document's end)
var _stillSeconds = 2.0;

/// A sound file heard before it is placed (Alight Motion plays a song before it is added): pressed, the Mac's own
/// player plays it (afplay); pressed again, or another file picked, it stops.
class _Listen extends StatefulWidget {
  const _Listen(this.n);
  final FileNode n;
  @override
  State<_Listen> createState() => _ListenState();
}

class _ListenState extends State<_Listen> {
  Process? _playing;

  Future<void> _toggle() async {
    if (_playing case final p?) {
      p.kill();
      return setState(() => _playing = null);
    }
    final p = await Process.start('afplay', [widget.n.id]);
    if (!mounted) {
      p.kill();
      return;
    }
    setState(() => _playing = p);
    p.exitCode.then((_) => mounted && identical(_playing, p) ? setState(() => _playing = null) : null);   // (to its end: still again)
  }

  @override
  void didUpdateWidget(_Listen old) {
    super.didUpdateWidget(old);
    if (old.n.id != widget.n.id) _playing?.kill();
  }

  @override
  void dispose() {
    _playing?.kill();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => GestureDetector(
    key: const ValueKey('media-listen'),
    onTap: _toggle,
    child: Stack(fit: StackFit.expand, children: [
      _Art(widget.n),
      Center(child: Text(_playing == null ? '▶' : '■', style: T.name(Grey.g95))),
    ]),
  );
}
