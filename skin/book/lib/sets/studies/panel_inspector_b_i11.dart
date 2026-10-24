part of 'panel_inspector_b.dart';

// ---- I11: carry one aspect of a thing to another -------------------------------------------------------------------------------------

final Color _att = Fam.attach.c;

const _aspects = [
  ('Value', 'Glow Intensity 1.40'),
  ('Ease', '3 keys · Ease Out Back'),
  ('Color', 'Glow Color #EFCB4E'),
  ('Keys', '0:12  1:00  2:04'),
  ('Effects', 'Glow + Gaussian Blur'),
];

class _Target {
  const _Target(this.name, {this.glow = true, this.keys = true});
  final String name;
  final bool glow, keys;
}

const _targets = [
  _Target('title_card'),
  _Target('bg_gradient', glow: false, keys: false),
  _Target('star_burst'),
  _Target('見出し_タイトル', keys: false),
  _Target('particles_main'),
  _Target('logo_end_card_final_v2', glow: false),
];

// ---- I11-a: a clipboard that holds one aspect ----------------------------------------------------------------------------------------

class _ClipPanel extends StatefulWidget {
  const _ClipPanel({required this.h, required this.preset});
  final double h;
  final String preset; // empty | holding | pasted | partial
  @override
  State<_ClipPanel> createState() => _ClipPanelState();
}

class _ClipPanelState extends State<_ClipPanel> {
  int held = -1;
  Set<int> sel = {0, 1, 2, 3};
  Map<int, String> result = {};

  @override
  void initState() {
    super.initState();
    _preset();
  }

  @override
  void didUpdateWidget(_ClipPanel old) {
    super.didUpdateWidget(old);
    if (old.preset != widget.preset) {
      _preset();
    }
  }

  void _preset() {
    result = {};
    held = widget.preset == 'empty' ? -1 : 1;
    sel = {0, 1, 2, 3};
    if (widget.preset == 'pasted' || widget.preset == 'partial') {
      _paste();
    }
  }

  String _res(int a, _Target t) {
    if (a == 4) {
      return t.glow ? 'Glow replaced, Blur added' : 'Glow + Blur added';
    }
    if (!t.glow) {
      return 'skipped: no Glow here';
    }
    if ((a == 1 || a == 3) && !t.keys) {
      return 'skipped: no keys on Glow Intensity';
    }
    return switch (a) { 0 => 'Intensity set to 1.40', 1 => 'Ease on 3 keys', 2 => 'Color set to #EFCB4E', _ => '3 keys added' };
  }

  void _paste() {
    result = {for (final i in sel) i: _res(held, _targets[i])};
  }

  @override
  Widget build(BuildContext context) {
    final skipped = result.values.where((r) => r.startsWith('skipped')).length;
    return _Panel(
      title: 'ring_glow',
      sub: 'Copy and paste',
      h: widget.h,
      child: _Lv(children: [
        const _Sect('Copy from this layer'),
        for (var i = 0; i < _aspects.length; i++)
          Hov(
            onTap: () => setState(() {
              held = i;
              result = {};
            }),
            builder: (_, h) => _Gut(
              tick: held == i ? N.g95 : null,
              child: Container(
                height: 28,
                padding: const EdgeInsets.symmetric(horizontal: 12),
                decoration: BoxDecoration(color: held == i ? N.g20 : (h ? N.g15 : null), border: const Border(bottom: BorderSide(color: N.rowLine))),
                child: Row(children: [
                  SizedBox(width: 56, child: Text(_aspects[i].$1, maxLines: 1, style: T.name(held == i ? N.g95 : N.g76))),
                  Expanded(child: Text(_aspects[i].$2, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76))),
                  SizedBox(width: 32, child: Align(alignment: Alignment.centerRight, child: h && held != i ? Text('Copy', style: T.label(N.g95)) : null)),
                ]),
              ),
            ),
          ),
        _Band(
          tick: held < 0 ? N.g26 : N.g95,
          child: Row(children: [
            Expanded(
              child: held < 0
                  ? Text('Nothing held', style: T.label(N.g76).copyWith(fontSize: 11))
                  : Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                      Text.rich(TextSpan(children: [TextSpan(text: 'Holding  ', style: T.label(N.g76).copyWith(fontSize: 11)), TextSpan(text: _aspects[held].$1, style: T.title())]), maxLines: 1, overflow: TextOverflow.ellipsis),
                      _gap(4),
                      Text('from ring_glow', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g76)),
                    ]),
            ),
            if (held >= 0) ...[
              const SizedBox(width: 8),
              _Word('Clear', onTap: () => setState(() {
                    held = -1;
                    result = {};
                  })),
            ],
          ]),
        ),
        _Sect('Paste to', count: sel.length, trailing: _Word(sel.length == _targets.length ? 'None' : 'All', onTap: () => setState(() => sel = sel.length == _targets.length ? {} : {for (var i = 0; i < _targets.length; i++) i}))),
        for (var i = 0; i < _targets.length; i++)
          Hov(
            onTap: () => setState(() {
              sel = {...sel};
              sel.contains(i) ? sel.remove(i) : sel.add(i);
              result = {};
            }),
            builder: (_, h) {
              final on = sel.contains(i), r = result[i], bad = r != null && r.startsWith('skipped');
              return _Gut(
                tick: bad ? Role.error : (on ? N.g95 : null),
                child: Container(
                  constraints: const BoxConstraints(minHeight: 28),
                  padding: const EdgeInsets.fromLTRB(12, 4, 12, 4),
                  decoration: BoxDecoration(color: on ? N.g20 : (h ? N.g15 : null), border: const Border(bottom: BorderSide(color: N.rowLine))),
                  child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [
                    if (bad) const ErrMark(size: 10, gap: 6),
                    Expanded(flex: 4, child: Text(_targets[i].name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(on ? N.g95 : N.g76))),
                    if (r != null) Expanded(flex: 5, child: Text(r, textAlign: TextAlign.right, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(bad ? N.g91 : N.g76).copyWith(height: 1.2))),
                  ]),
                ),
              );
            },
          ),
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 12, 12, 0),
          child: Align(
            alignment: Alignment.centerLeft,
            child: _Btn(
              held < 0 ? 'Paste' : 'Paste ${_aspects[held].$1}',
              primary: held >= 0 && sel.isNotEmpty,
              onTap: held < 0 || sel.isEmpty ? null : () => setState(_paste),
            ),
          ),
        ),
        if (result.isNotEmpty) _Hint(skipped == 0 ? 'Pasted to ${result.length}' : '${result.length - skipped} of ${result.length} took it · $skipped skipped'),
      ]),
    );
  }
}

// ---- I11-b: copy with link ---------------------------------------------------------------------------------------------------------

class _LinkPanel extends StatefulWidget {
  const _LinkPanel({required this.h, required this.preset});
  final double h;
  final String preset; // linked | unlinked | deleted
  @override
  State<_LinkPanel> createState() => _LinkPanelState();
}

class _LinkPanelState extends State<_LinkPanel> {
  static const names = ['ring_glow', 'title_card', 'bg_gradient'];
  double shared = 1.4;
  final own = [1.4, 1.4, 1.4];
  Set<int> group = {0, 1, 2};
  bool deleted = false;

  @override
  void initState() {
    super.initState();
    _preset();
  }

  @override
  void didUpdateWidget(_LinkPanel old) {
    super.didUpdateWidget(old);
    if (old.preset != widget.preset) {
      _preset();
    }
  }

  void _preset() {
    shared = 1.4;
    for (var i = 0; i < 3; i++) {
      own[i] = i == 0 ? 1.4 : (i == 1 ? 1.1 : 0.7);
    }
    deleted = widget.preset == 'deleted';
    group = widget.preset == 'linked' ? {0, 1, 2} : {};
    if (deleted) {
      own[1] = own[2] = shared;
    }
  }

  void _set(int i, double d) => setState(() {
        if (group.contains(i)) {
          shared = (shared + d).clamp(0, 10);
        } else {
          own[i] = (own[i] + d).clamp(0, 10);
        }
      });

  @override
  Widget build(BuildContext context) {
    final n = group.length;
    return _Panel(
      title: 'Glow Intensity',
      sub: n > 1 ? 'Shared by $n layers' : 'Not shared',
      h: widget.h,
      child: _Lv(children: [
        _Sect('Layers', count: deleted ? 2 : 3, trailing: _Word(group.length == 3 ? 'Unlink all' : 'Link all', onTap: () => setState(() {
              if (group.length == 3) {
                for (var i = 0; i < 3; i++) {
                  own[i] = shared;
                }
                group = {};
              } else {
                group = {for (var i = deleted ? 1 : 0; i < 3; i++) i};
              }
            }))),
        for (var i = 0; i < 3; i++)
          if (!(deleted && i == 0)) ...[
            _Sect(names[i], upper: false, sub: i == 0 ? 'the source' : null, trailing: i == 0 ? _Word('Delete layer', onTap: () => setState(() {
                  deleted = true;
                  own[1] = own[2] = group.contains(1) ? shared : own[1];
                  group = group.where((g) => g != 0).toSet();
                  for (final g in group.toList()) {
                    own[g] = shared;
                  }
                  group = {};
                })) : null),
            _VRow(
              'Glow Intensity',
              _Num(value: group.contains(i) ? shared : own[i], dec: 2, step: .01, per: .02, onDelta: (d) => _set(i, d), onType: (x) => setState(() => group.contains(i) ? shared = x : own[i] = x)),
              tick: group.contains(i) ? _att : null,
              labelColor: group.contains(i) ? N.g95 : N.g63,
            ),
            if (group.contains(i))
              Padding(
                padding: const EdgeInsets.fromLTRB(12, 0, 12, 4),
                child: Row(children: [
                  Text('Shared ×${group.length}', style: T.label(N.g76)),
                  const Spacer(),
                  _Word('Unlink', onTap: () => setState(() {
                        own[i] = shared;
                        group = {...group}..remove(i);
                      })),
                ]),
              ),
            _VRow('Glow Radius', _Num(value: 25.0 + i * 5, unit: 'px', onDelta: (_) {})),
            _VRow('Glow Threshold', _Num(value: 60.0, unit: '%', onDelta: (_) {})),
          ],
        if (deleted) const _Hint('ring_glow deleted', color: N.g63),
      ]),
    );
  }
}

// ---- I11-c: eyedropper ----------------------------------------------------------------------------------------------------------------------

const _dropLayers = ['ring_glow', 'bg_gradient', '見出し_タイトル', 'particles_main', 'star_burst', 'kick_visual', 'logo_end_card_final_v2', 'flare_01'];

class _DropProp {
  const _DropProp(this.id, this.label, this.unit, this.dec, this.kind, this.vals, this.mine);
  final String id, label, unit;
  final int dec;
  final _K kind;
  final List<double?> vals;
  final double mine;
}

const _dropProps = [
  _DropProp('gi', 'Glow Intensity', '', 2, _K.num, [1.8, null, 2.2, .4, 1.4, null, .9, 3.1], .8),
  _DropProp('gr', 'Glow Radius', 'px', 0, _K.num, [32, null, 12, 80, 25, 48, null, 160], 40),
  _DropProp('gc', 'Glow Color', '', 0, _K.color, [4, null, 1, 3, 4, null, 5, 2], 0),
  _DropProp('bm', 'Blending Mode', '', 0, _K.choice, [1, 0, 0, 3, 1, 2, 0, 3], 0),
];

class _DropPanel extends StatefulWidget {
  const _DropPanel({required this.h, required this.start});
  final double h;
  final String start; // none | gi | gc
  @override
  State<_DropPanel> createState() => _DropPanelState();
}

class _DropPanelState extends State<_DropPanel> {
  String? picking;
  int? peek;
  final mine = {for (final p in _dropProps) p.id: p.mine};
  final focus = FocusNode();

  @override
  void initState() {
    super.initState();
    picking = widget.start == 'none' ? null : widget.start;
  }

  @override
  void didUpdateWidget(_DropPanel old) {
    super.didUpdateWidget(old);
    if (old.start != widget.start) {
      picking = widget.start == 'none' ? null : widget.start;
      peek = null;
    }
  }

  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  String _show(_DropProp p, double v) => switch (p.kind) {
        _K.num => '${_f(v, p.dec)}${p.unit.isEmpty ? '' : ' ${p.unit}'}',
        _K.color => hexOf(_swatch[v.round()]),
        _ => _bl[v.round()],
      };

  /// The value cell as every Inspector cell is drawn: a number has the grip ticks and its unit, a colour and a choice end in the chevron.
  Widget _cell(_DropProp p, double v, {bool hot = false, bool dim = false}) {
    final num = p.kind == _K.num;
    return Container(
      height: 20,
      padding: EdgeInsets.only(left: num ? 8 : 7, right: num ? 8 : 4),
      decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: hot ? Role.selected : N.g20)),
      child: Stack(clipBehavior: Clip.none, children: [
        Positioned.fill(
          child: Row(children: [
            if (p.kind == _K.color) Padding(padding: const EdgeInsets.only(right: 6), child: Container(width: 16, height: 10, decoration: BoxDecoration(color: _swatch[v.round()], borderRadius: BorderRadius.circular(2)))),
            Expanded(child: Text(_show(p, v), maxLines: 1, overflow: TextOverflow.ellipsis, textAlign: num ? TextAlign.right : TextAlign.left, style: num ? T.value(dim ? N.g63 : N.g95) : T.value(dim ? N.g63 : N.g95).copyWith(fontFamily: p.kind == _K.color ? T.mono : T.sans))),
            if (!num) ...[const SizedBox(width: 4), Text('›', style: T.name(N.g63))],
          ]),
        ),
        if (num) Positioned(left: -5, top: 0, bottom: 0, child: Center(child: CustomPaint(size: const Size(5, 8), painter: _GripTicks(hot ? N.g76 : N.g38)))),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) {
    final pk = picking == null ? null : _dropProps.firstWhere((p) => p.id == picking);
    return _Panel(
      title: 'title_card',
      sub: pk == null ? 'Eyedropper' : 'Picking ${pk.label}',
      h: widget.h,
      child: Focus(
        focusNode: focus,
        autofocus: true,
        onKeyEvent: (n, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && picking != null) {
            setState(() {
              picking = null;
              peek = null;
            });
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: _Lv(children: [
          const _Sect('Glow on this layer'),
          for (final p in _dropProps)
            Hov(
              cursor: SystemMouseCursors.basic,
              builder: (_, h) {
                final on = picking == p.id;
                final shown = on && peek != null && p.vals[peek!] != null ? p.vals[peek!]! : mine[p.id]!;
                return _Gut(
                  tick: on ? N.g95 : null,
                  child: Container(
                    constraints: const BoxConstraints(minHeight: 28),
                    padding: const EdgeInsets.fromLTRB(12, 4, 12, 4),
                    color: on ? N.g20 : (h ? N.g15 : null),
                    child: Row(children: [
                      Expanded(flex: 5, child: _Mid(p.label, T.label(on ? N.g95 : N.g63).copyWith(fontSize: 11, height: 1.15))),
                      const SizedBox(width: 8),
                      Expanded(flex: 6, child: _cell(p, shown, hot: on && peek != null)),
                      const SizedBox(width: 8),
                      SizedBox(width: 64, child: Align(alignment: Alignment.centerRight, child: QuietButton(on ? 'Cancel' : 'Pick', primary: on, onTap: () => setState(() {
                            picking = on ? null : p.id;
                            peek = null;
                          })))),
                    ]),
                  ),
                );
              },
            ),
          if (pk != null) ...[
            _Sect('Read ${pk.label} from', count: _dropLayers.length),
            for (var i = 0; i < _dropLayers.length; i++)
              Hov(
                onTap: pk.vals[i] == null ? null : () => setState(() {
                      mine[pk.id] = pk.vals[i]!;
                      picking = null;
                      peek = null;
                    }),
                cursor: pk.vals[i] == null ? SystemMouseCursors.forbidden : SystemMouseCursors.click,
                builder: (_, h) {
                  final none = pk.vals[i] == null;
                  if (h && peek != i && !none) {
                    WidgetsBinding.instance.addPostFrameCallback((_) => mounted ? setState(() => peek = i) : null);
                  } else if (!h && peek == i) {
                    WidgetsBinding.instance.addPostFrameCallback((_) => mounted && peek == i ? setState(() => peek = null) : null);
                  }
                  return _Gut(
                    tick: h && !none ? N.g95 : null,
                    child: Container(
                      height: 28,
                      padding: const EdgeInsets.symmetric(horizontal: 12),
                      color: h && !none ? N.g20 : null,
                      child: Row(children: [
                        Expanded(child: Text(_dropLayers[i], maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(none ? N.g63 : (h ? N.g95 : N.g76)))),
                        Text(none ? 'no ${pk.id.startsWith('g') ? 'Glow' : 'value'}' : _show(pk, pk.vals[i]!), maxLines: 1, style: T.value(none ? N.g63 : N.g91).copyWith(fontFamily: none ? T.sans : T.mono)),
                      ]),
                    ),
                  );
                },
              ),
          ],
        ]),
      ),
    );
  }
}

// ---- I11-d: a shelf of aspects ('Mine') -----------------------------------------------------------------------------------------------------

class _ShelfItem {
  const _ShelfItem(this.name, this.meta, this.seed);
  final String name, meta;
  final int seed;
}

class _ShelfPanel extends StatefulWidget {
  const _ShelfPanel({required this.h, required this.count});
  final double h;
  final int count;
  @override
  State<_ShelfPanel> createState() => _ShelfPanelState();
}
