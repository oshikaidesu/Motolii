// Part of panel_browser_b.dart (cut there by size, as it was).
part of 'panel_browser_b.dart';

class _SaveToShelfState extends State<_SaveToShelf> {
  late final List<_Mine> _mine = widget.empty ? [] : _seedMine();
  String? _carry, _msg;
  bool _over = false;
  static const _cards = [('Effects', 'Glow 40% + Levels'), ('Effects', 'Gaussian Blur 12'), ('Curve', 'custom ease · overshoot 1.2'), ('Layer', 'タイトル with 2 effects')];

  void _drop() {
    if (_carry == null) return;
    final c = _cards.firstWhere((e) => e.$2 == _carry);
    setState(() {
      _mine.insert(0, _Mine(c.$2, c.$1, 'Everywhere', naming: true));
      _msg = 'Saved to Mine';
      _carry = null;
      _over = false;
    });
  }

  @override
  Widget build(BuildContext context) => _Keys(
          onKey: (e) {
            if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _carry != null) {
              setState(() {
                _carry = null;
                _msg = 'Cancelled';
              });
              return true;
            }
            return false;
          },
          child: _Follow(
            ghost: _carry == null ? null : _Ghost(_carry!),
            child: _withSide(_Frame(
              w: widget.w,
              child: _Col([
              _Head([
                const IgnorePointer(child: _Tabs(2)),
                const SizedBox(height: 8),
                _Sec('Mine', n: '${_mine.length}'),
              ], bottom: 4),
              _Grow(
                child: MouseRegion(
                  onEnter: (_) => setState(() => _over = true),
                  onExit: (_) => setState(() => _over = false),
                  cursor: _carry == null ? MouseCursor.defer : SystemMouseCursors.copy,
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTap: _drop,
                    child: _well(
                      edge: _carry != null ? (_over ? _accent : N.g56) : N.g20,
                      child: _mine.isEmpty
                          ? Center(heightFactor: 1, child: _empty('自分の棚はまだ空です', ''))
                          : _Fade(child: ListView.builder(
                              shrinkWrap: _PanelH.fitOf(context),
                              padding: const EdgeInsets.symmetric(vertical: 4),
                              itemCount: _mine.length + 1,
                              itemBuilder: (c, i) {
                                if (i == _mine.length) {
                                  return Padding(padding: const EdgeInsets.fromLTRB(_px, 6, _px, 6), child: SizedBox(height: 34, child: _DropZone('Save', on: _carry != null)));
                                }
                                final m = _mine[i];
                                if (!m.naming) return _Hit(onTap: _drop, cursor: _carry == null ? SystemMouseCursors.basic : SystemMouseCursors.copy, builder: (c, h, _) => _mineRow(m, hover: h && _carry == null));
                                return Container(
                                  margin: const EdgeInsets.symmetric(vertical: 2, horizontal: _px),
                                  padding: const EdgeInsets.all(6),
                                  decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(5)),
                                  child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                                    Row(children: [_Tag(m.kind), const SizedBox(width: 6), Expanded(child: _t('NEW', T.micro(N.g95)))]),
                                    const SizedBox(height: 5),
                                    _NameEdit(initial: m.name, onDone: (s) => setState(() {
                                          m.name = s;
                                          m.naming = false;
                                          _msg = 'Named "$s"';
                                        })),
                                    const SizedBox(height: 5),
                                    Segmented(items: const ['Everywhere', 'Project'], index: m.scope == 'Everywhere' ? 0 : 1, expand: true, onChanged: (j) => setState(() => m.scope = j == 0 ? 'Everywhere' : 'This project')),
                                  ]),
                                );
                              },
                            )),
                    ),
                  ),
                ),
              ),
              _Foot(msg: _msg, hints: const [('drop', 'save a copy'), ('Esc', 'cancel')]),
            ]),
            ),
            _Side('Inspector (outside)', [
              for (final c in _cards)
                _Hit(
                  onTap: () => setState(() => _carry = _carry == c.$2 ? null : c.$2),
                  builder: (cx, h, _) => AnimatedContainer(duration: Mo.dur, height: 26, padding: const EdgeInsets.symmetric(horizontal: 6), decoration: BoxDecoration(color: _carry == c.$2 ? N.g20 : (h ? N.g15 : N.g10), borderRadius: BorderRadius.circular(4), border: Border.all(color: _carry == c.$2 ? _accent : N.g20)), child: Row(children: [SizedBox(width: 50, child: Align(alignment: Alignment.centerLeft, child: _Tag(c.$1))), const SizedBox(width: 8), Expanded(child: _t(c.$2, T.name(N.g91)))])),
                ),
            ]),
            ),
          ),
        );
}

class _Well extends StatelessWidget {
  const _Well(this.label, this.v, this.onChanged, {this.changed = false});
  final String label;
  final bool changed;
  final double v;
  final ValueChanged<double> onChanged;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (c, box) {
        void set(Offset p) => onChanged((p.dx / box.maxWidth).clamp(0.0, 1.0));
        return MouseRegion(
          cursor: SystemMouseCursors.resizeLeftRight,
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onPanDown: (d) => set(d.localPosition),
            onPanUpdate: (d) => set(d.localPosition),
            child: _Param(label, v, hot: true, grip: true, changed: changed),
          ),
        );
      });
}

/// B11-b: shipped presets cannot change; the first edit makes a copy on "Mine".
class _CopyOnEdit extends StatefulWidget {
  const _CopyOnEdit({required this.w});
  final double w;
  @override
  State<_CopyOnEdit> createState() => _CopyOnEditState();
}

class _CopyOnEditState extends State<_CopyOnEdit> {
  static const _bundled = ['Soft Glow', 'Neon Edge', 'Film Dust', 'Dream Haze', 'Hard Light Pop', 'Cinematic Teal & Orange', 'Vintage VHS', 'Clean Shadow'];
  static const _bundledJp = ['やわらかい光', 'ネオンの縁', 'フィルムの埃', '夢のもや', 'ハードライト', 'シネマ調', 'ビデオ風', 'きれいな影'];
  final _vals = <String, List<double>>{};
  final _copies = <String>[];
  final _from = <String, String>{};
  String _sel = 'Soft Glow';
  String? _msg;

  bool _isCopy(String n) => _copies.contains(n);
  List<double> _v(String n) => _vals.putIfAbsent(n, () => [(n.length % 5) / 6 + .2, (n.length % 3) / 4 + .3, .5]);

  void _edit(int k, double v) {
    setState(() {
      if (!_isCopy(_sel)) {
        final base = _sel;
        var name = '$base (copy)';
        var i = 2;
        while (_copies.contains(name)) {
          name = '$base (copy $i)';
          i++;
        }
        _vals[name] = [..._v(base)];
        _from[name] = base;
        _copies.insert(0, name);
        _sel = name;
        _msg = '複製 "$name" → Mine';
      }
      _v(_sel)[k] = v;
    });
  }

  @override
  Widget build(BuildContext context) {
    final v = _v(_sel), copy = _isCopy(_sel);
    return _Frame(
      w: widget.w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(2)),
          const SizedBox(height: 8),
          _Sec('Mine', n: '${_copies.length}'),
        ], bottom: 4),
        _well(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 112),
            child: _copies.isEmpty
                ? _empty('まだ空です', '')
                : ListView(shrinkWrap: true, padding: const EdgeInsets.symmetric(vertical: 4), children: [for (final c in _copies) _Hit(onTap: () => setState(() => _sel = c), builder: (cx, h, _) => _mineRow(_Mine(c, 'Effects', 'Everywhere'), hover: h, sel: _sel == c, kind: false))]),
          ),
        ),
        Padding(padding: const EdgeInsets.fromLTRB(_px, 8, _px, 0), child: _Sec('Bundled', n: '${_bundled.length}')),
        _Grow(
          child: _Fade(child: ListView.builder(
            shrinkWrap: _PanelH.fitOf(context),
            padding: const EdgeInsets.only(bottom: 12),
            itemExtent: 28,
            itemCount: _bundled.length,
            itemBuilder: (c, i) {
              final n = _bundled[i];
              return _Hit(
                onTap: () => setState(() => _sel = n),
                builder: (c, h, _) => _Line(height: 28, sel: _sel == n, hover: h, child: Align(alignment: Alignment.centerLeft, child: _NameJp(n, _bundledJp[i], T.name(_sel == n ? N.g95 : N.g91), T.label(N.g76), whole: true))),
              );
            },
          )),
        ),
        Container(
          padding: const EdgeInsets.fromLTRB(_px, 8, _px, 8),
          decoration: const BoxDecoration(color: N.g13, border: Border(top: BorderSide(color: N.g20))),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Row(children: [Expanded(child: _t(_sel, T.name(N.g95))), _Tag(copy ? 'YOUR COPY' : 'BUNDLED')]),
            const SizedBox(height: 6),
            for (final (k, n) in ['Intensity', 'Radius', 'Threshold'].indexed) ...[
              if (k > 0) const SizedBox(height: 4),
              _Well(n, v[k], (x) => _edit(k, x), changed: copy && _from[_sel] != null && (v[k] - _v(_from[_sel]!)[k]).abs() > .001),
            ],
          ]),
        ),
        _Foot(msg: _msg, hints: const [('drag', 'a value = auto copy')]),
      ]),
    );
  }
}

/// One History row, 36 px: time, what the layer looked like, and the Name… button that makes it permanent.
class _HistRow extends StatelessWidget {
  const _HistRow(this.time, this.what, {this.hover = false, this.onName});
  final String time, what;
  final bool hover;
  final VoidCallback? onName;
  @override
  Widget build(BuildContext context) => _Line(
        height: 36,
        hover: hover,
        child: Row(children: [
          SizedBox(width: 56, child: Text(time, style: _num())),
          Expanded(child: _t(what, T.name(N.g91))),
          _Btn('Name…', onTap: onName ?? _noop),
        ]),
      );
}

/// B11-c: the layer's effect settings are snapshotted by themselves; naming one keeps it.
class _Snaps extends StatefulWidget {
  const _Snaps({required this.w});
  final double w;
  @override
  State<_Snaps> createState() => _SnapsState();
}

class _SnapsState extends State<_Snaps> {
  static const _cap = 20;
  final _named = <String>['Soft Glow + Levels (client A)', 'Hero Title Stack', '夜の青 Night Blue'];
  late final List<(String, String)> _hist = [
    ('14:02:11', 'Glow 40 · Blur 12 · Levels'),
    ('14:01:48', 'Glow 40 · Blur 10 · Levels'),
    ('14:01:20', 'Glow 35 · Blur 10 · Levels'),
    ('13:58:02', 'Glow 35 · Blur 10'),
    ('13:57:40', 'Blur 10'),
    ('13:55:09', '(empty)'),
  ];
  int _naming = -1, _tick = 0;
  String? _msg;

  void _change(String what) {
    final t = '14:${(3 + _tick ~/ 3).toString().padLeft(2, '0')}:${((_tick * 17) % 60).toString().padLeft(2, '0')}';
    _tick++;
    setState(() {
      _hist.insert(0, (t, what));
      if (_naming >= 0) _naming++;
      while (_hist.length > _cap) {
        _hist.removeLast();
      }
      _msg = 'Edit: $what · snapshot taken';
    });
  }

  @override
  Widget build(BuildContext context) {
    final w = widget.w;
    final frame = _Frame(
      w: w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(2)),
          const SizedBox(height: 8),
          _Sec('Mine', n: '${_named.length}'),
        ], bottom: 4),
        _well(child: Column(children: [for (final n in _named) _Hit(builder: (c, h, _) => _mineRow(_Mine(n, 'Effects', 'Everywhere'), hover: h, kind: false))])),
        Padding(padding: const EdgeInsets.fromLTRB(_px, 10, _px, 0), child: _Sec('History', n: '${_hist.length} / $_cap')),
        _Grow(
          child: _Fade(child: ListView.builder(
            shrinkWrap: _PanelH.fitOf(context),
            padding: const EdgeInsets.only(bottom: 12),
            itemCount: _hist.length,
            itemBuilder: (c, i) {
              final (t, d) = _hist[i];
              if (i == _naming) {
                return Container(
                  margin: const EdgeInsets.symmetric(vertical: 2, horizontal: _px),
                  padding: const EdgeInsets.all(6),
                  decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(5)),
                  child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                    _t('$t  ·  $d', T.label(N.g76)),
                    const SizedBox(height: 5),
                    _NameEdit(initial: 'Untitled $t', onDone: (s) => setState(() {
                          _named.insert(0, s);
                          _hist.removeAt(_naming);
                          _naming = -1;
                          _msg = 'Kept as "$s"';
                        })),
                  ]),
                );
              }
              return _Hit(builder: (c, h, _) => _HistRow(t, d, hover: h, onName: () => setState(() => _naming = i)));
            },
          )),
        ),
        _Foot(msg: _msg),
      ]),
    );
    final side = _Side('The layer (outside)', [
      _Btn('Blur +2', onTap: () => _change('Glow 40 · Blur ${12 + _tick} · Levels')),
      _Btn('Glow −5', onTap: () => _change('Glow ${40 - _tick * 5} · Blur 12 · Levels')),
      _Btn('Change colour', onTap: () => _change('Tint #${(0x4781E5 + _tick * 997).toRadixString(16).toUpperCase()} · Glow 40')),
      _Btn('Add Levels', onTap: () => _change('+ Levels · Glow 40 · Blur 12')),
    ]);
    return _withSide(frame, side);
  }
}

// ================================================================ B12 large libraries

const _bases = <(String, String, Kind, ArtStyle, double)>[
  ('dusk_ridge', 'jpg', Kind.image, ArtStyle.sky, 0),
  ('lens_bloom', 'mp4', Kind.video, ArtStyle.abstract, 12),
  ('wind_pad', 'wav', Kind.audio, ArtStyle.wave, 94),
  ('gem_cut', 'glb', Kind.model, ArtStyle.solid, 0),
  ('studio_hdri', 'exr', Kind.pano, ArtStyle.gradient, 0),
  ('violet_fade', 'png', Kind.image, ArtStyle.gradient, 0),
  ('city_pass', 'mov', Kind.video, ArtStyle.sky, 47),
  ('haze', 'png', Kind.image, ArtStyle.abstract, 0),
  ('夕焼け_空撮', 'mov', Kind.video, ArtStyle.sky, 31),
  ('背景_和紙テクスチャ', 'png', Kind.image, ArtStyle.abstract, 0),
  ('Interview_Tanaka-san_Cam-B_take-07_final_v2', 'mov', Kind.video, ArtStyle.sky, 312),
  ('kick_loop', 'wav', Kind.audio, ArtStyle.wave, 8),
];

List<Asset> _gen(int n) => [
      for (var i = 0; i < n; i++)
        () {
          final b = _bases[i % _bases.length];
          return Asset('${b.$1}_${(i ~/ _bases.length + 1).toString().padLeft(3, '0')}.${b.$2}', b.$3, i, style: b.$4, dur: b.$5 == 0 ? 0 : b.$5 + (i % 7) * 3, dims: b.$3 == Kind.audio ? '48 kHz' : (b.$3 == Kind.model ? '${12 + i % 40}k tris' : (i % 3 == 0 ? '3840×2160' : '1920×1080')), size: '${(i % 90) + 2} MB');
        }(),
    ];

String _fmt(int n) => n.toString().replaceAllMapped(RegExp(r'(\d)(?=(\d{3})+$)'), (m) => '${m[1]},');

class _AssetCell extends StatelessWidget {
  const _AssetCell(this.a, this.w, {this.sel = false, this.hover = false, this.badge, this.names = true});
  final Asset a;
  final double w;
  final bool sel, hover, names;
  final String? badge;
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Stack(children: [
          AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            width: w,
            height: w * .62,
            foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: sel ? _accent : (hover ? N.g56 : N.g20))),
            child: ClipRRect(borderRadius: BorderRadius.circular(6), child: _Art(a)),
          ),
          if (w >= 70 && a.kind != Kind.image) Positioned(left: 4, top: 4, child: TypeBadge(a.kind)),
          if ((badge ?? (a.dur > 0 ? clock(a.dur) : null)) != null) Positioned(right: 4, bottom: 4, child: TimePill(badge ?? clock(a.dur))),
        ]),
        if (names) ...[const SizedBox(height: 4), _Mid(a.name, T.name(sel ? N.g95 : N.g91)), const SizedBox(height: 3), _t(a.dims, T.label(N.g76))],
      ]);
}

class _AssetRow extends StatelessWidget {
  const _AssetRow(this.a, {this.sel = false, this.hover = false});
  final Asset a;
  final bool sel, hover;
  @override
  Widget build(BuildContext context) => _Line(
        height: 28,
        sel: sel,
        hover: hover,
        child: Row(children: [
          SizedBox(width: 24, height: 18, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Art(a))),
          const SizedBox(width: 8),
          Expanded(child: _t(a.name, T.name(sel ? N.g95 : N.g91))),
          Text(a.dur > 0 ? clock(a.dur) : a.kind.badge, style: _num()),
        ]),
      );
}

/// A thumbnail that has not arrived yet: same footprint as an asset cell (thumb 6 px corner, name bar, kind bar) so nothing jumps.
class _Skel extends StatelessWidget {
  const _Skel(this.w);
  final double w;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: w,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Container(height: w * .62, decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6))),
          const SizedBox(height: 4),
          Container(height: 11, width: w * .7, decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(3))),
          const SizedBox(height: 3),
          Container(height: 10, width: w * .45, decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(3))),
        ]),
      );
}

class _Pulse extends StatefulWidget {
  const _Pulse({required this.child});
  final Widget child;
  @override
  State<_Pulse> createState() => _PulseState();
}

class _PulseState extends State<_Pulse> with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(vsync: this, duration: const Duration(milliseconds: 1100))..repeat(reverse: true);
  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(animation: _c, builder: (c, w) => Opacity(opacity: .45 + .55 * Curves.easeInOut.transform(_c.value), child: w), child: widget.child);
}

/// B12-a: list and thumbnails are one thing with two faces; the selection survives the switch; the count is always there.
class _Library extends StatefulWidget {
  const _Library({super.key, required this.w, required this.n, this.view = 0, this.state = 0, this.query = ''});
  final double w;
  final int n, view;
  final int state; // 0 normal, 1 loading, 2 folder unreadable
  final String query;
  @override
  State<_Library> createState() => _LibraryState();
}
