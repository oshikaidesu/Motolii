// Part of dock_files.dart (cut there by size, as it was).
part of 'dock_files.dart';

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
    final c = widget.c, sel = c.sel == n.id, t = _typeOf(n.g), can = _kindOfFile(n.id).isNotEmpty;
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
    final layer = _layerOf(n, c), can = _kindOfFile(n.id).isNotEmpty;
    return DockFooter(
      preview: _Art(n),
      title: n.name,
      meta: c.mem.trouble[n.id] ?? (layer != null ? 'In use · ${_metaOf(n)}' : _metaOf(n)),
      action: can ? 'Place' : 'Select',
      done: !can && layer == null,
      doneLabel: 'Folder', // (the shelf lists only folders and files the inside can place: what cannot be placed is a folder)
      fresh: can, // (placed, it is a new thing)
      onAction: () => can ? _place(c, n) : c.ws.select(layer),
    );
  }
}
