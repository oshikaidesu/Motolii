part of 'panel_inspector_b.dart';

// ---- I7: a long property list, one engine, four options ---------------------------------------------------------------------------
// _ParamPanel is one effect's property list. The options differ by flags: fold (a), find (b), stars (c), recent (d).

class _ParamPanel extends StatefulWidget {
  const _ParamPanel({
    required this.fx,
    required this.h,
    this.w = 282,
    this.lang = 0,
    this.layer = 'grain_overlay.mov',
    this.find = false,
    this.fold = false,
    this.stars = false,
    this.recent = false,
    this.presetQuery = '',
    this.presetOpen = false,
    this.presetStars = 0,
  });
  final _Fx fx;
  final double h, w;
  final int lang;
  final String layer, presetQuery;
  final bool find, fold, stars, recent, presetOpen;
  final int presetStars;
  @override
  State<_ParamPanel> createState() => _ParamPanelState();
}

class _ParamPanelState extends State<_ParamPanel> {
  final vals = _Vals();
  final ctrl = TextEditingController();
  bool foldOpen = false, recentOn = true;
  final List<String> stars = [];

  @override
  void initState() {
    super.initState();
    vals.addListener(() => setState(() {}));
    ctrl.addListener(() => setState(() {}));
    _preset(true);
  }

  void _preset(bool all, [_ParamPanel? old]) {
    final w = widget;
    if (all || w.presetQuery != old!.presetQuery) {
      ctrl.text = w.presetQuery;
    }
    if (all || w.presetOpen != old!.presetOpen) {
      foldOpen = w.presetOpen;
    }
    if (all || w.presetStars != old!.presetStars) {
      stars
        ..clear()
        ..addAll(w.fx.params.where((p) => p.major).take(w.presetStars).map((p) => p.id));
    }
    if (all && w.recent) {
      vals.touched.addAll(const ['fn.contrast', 'fn.evo', 'fn.scale']);
    }
  }

  @override
  void didUpdateWidget(_ParamPanel old) {
    super.didUpdateWidget(old);
    if (old.fx.id != widget.fx.id) {
      vals.clear();
      _preset(true);
    } else {
      _preset(false, old);
      if (old.layer != widget.layer) {
        vals.clear();
      }
    }
  }

  @override
  void dispose() {
    ctrl.dispose();
    vals.dispose();
    super.dispose();
  }

  _P _byId(String id) => widget.fx.params.firstWhere((p) => p.id == id);

  Widget _row(_P p, {Widget Function(bool)? trailH, Color? tick}) => _PRow(p, vals, key: ValueKey('${widget.fx.id}.${p.id}'), lang: widget.lang, jp: widget.fx.jpFx, query: ctrl.text.trim(), trailH: trailH, tick: tick);

  Widget _star(_P p, bool hover) => SizedBox(
        width: 24,
        child: (stars.contains(p.id) || hover)
            ? StarButton(on: stars.contains(p.id), size: 12, onChanged: (v) => setState(() => v ? stars.add(p.id) : stars.remove(p.id)))
            : null,
      );

  @override
  Widget build(BuildContext context) {
    final w = widget, fx = w.fx;
    final q = ctrl.text.trim();
    final all = fx.params;
    final list = <Widget>[];

    if (q.isNotEmpty) {
      final hits = all.where((p) => _match(p, q)).toList();
      if (hits.isEmpty) {
        list.add(_NoMatch(what: 'No property "$q"', next: 'Show all ${all.length}', onNext: ctrl.clear));
      } else {
        list.addAll(hits.map((p) => _row(p, trailH: w.stars ? (h) => _star(p, h) : null)));
      }
    } else {
      var order = [...all];
      if (w.recent && recentOn) {
        final t = vals.touched;
        order.sort((a, b) {
          final ia = t.indexOf(a.id), ib = t.indexOf(b.id);
          if (ia < 0 && ib < 0) {
            return 0;
          }
          return ia < 0 ? 1 : (ib < 0 ? -1 : ia.compareTo(ib));
        });
      }
      final pinned = w.stars ? stars.where((id) => all.any((p) => p.id == id)).map(_byId).toList() : <_P>[];
      final rest = order.where((p) => !pinned.contains(p)).toList();
      if (w.stars) {
        list.add(_Sect('Pinned', count: pinned.length));
        if (pinned.isEmpty) {
          list.add(const _Hint('None'));
        }
        list.addAll(pinned.map((p) => _row(p, trailH: (h) => _star(p, h), tick: N.g95)));
        list.add(_Sect('All', count: rest.length));
      }
      if (w.fold) {
        final main = rest.where((p) => p.major).toList(), adv = rest.where((p) => !p.major).toList();
        list.addAll(main.map(_row));
        if (adv.isNotEmpty) {
          list.add(_FoldRow(count: adv.length, open: foldOpen, onTap: () => setState(() => foldOpen = !foldOpen)));
          if (foldOpen) {
            list.addAll(adv.map(_row));
          }
        }
      } else if (w.recent) {
        final nTouched = rest.where((p) => vals.touched.contains(p.id)).length;
        for (var k = 0; k < rest.length; k++) {
          final p = rest[k], i = vals.touched.indexOf(p.id);
          if (recentOn && k == 0 && nTouched > 0) {
            list.add(_Sect('Touched', count: nTouched));
          }
          if (recentOn && k == nTouched && nTouched < rest.length) {
            list.add(_Sect('Not touched', count: rest.length - nTouched));
          }
          list.add(_row(p, tick: !recentOn && i >= 0 && i < 5 ? Color.lerp(N.g76, N.g26, i / 4) : null));
        }
      } else {
        String? g;
        for (final p in rest) {
          if (!w.stars && p.grp(w.lang) != g) {
            g = p.grp(w.lang);
            list.add(_Sect(g, count: rest.where((x) => x.grp(w.lang) == g).length));
          }
          list.add(_row(p, trailH: w.stars ? (h) => _star(p, h) : null));
        }
      }
    }

    final hitCount = q.isEmpty ? null : '${all.where((p) => _match(p, q)).length} of ${all.length}';
    return _Panel(
      title: w.layer,
      sub: 'Footage layer',
      w: w.w,
      h: w.h,
      child: Column(mainAxisSize: _FitScope.of(context) ? MainAxisSize.min : MainAxisSize.max, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        _FxHead(_fxName(fx, w.lang), w.fold || w.find || w.stars || w.recent ? '' : '${all.length} properties', true, (_) {}), // the total is said once: by the fold, the field's count, or the section counts
        if (w.find)
          Padding(padding: const EdgeInsets.fromLTRB(12, 8, 12, 8), child: _Field(ctrl: ctrl, hint: 'Find a property', count: hitCount)),
        if (w.recent)
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
            child: Segmented(items: const ['Recent first', 'Fixed order'], index: recentOn ? 0 : 1, expand: true, onChanged: (i) => setState(() => recentOn = i == 0)),
          ),
        _grow(context, _Lv(children: list)),
      ]),
    );
  }
}

/// "Advanced  17 more": the fold of a long tail; closed it says how many rows it hides (the one place that count is stated, in the same "N more" form everywhere), open it says nothing more.
class _FoldRow extends StatelessWidget {
  const _FoldRow({required this.count, required this.open, required this.onTap, this.pose});
  final _Pose? pose;
  final int count;
  final bool open;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => _Hv(
        pose: pose,
        onTap: onTap,
        builder: (_, h, d) => _Fcs(
          onTap: onTap,
          pose: pose,
          child: Container(
            height: 28,
            margin: const EdgeInsets.symmetric(vertical: 4),
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(color: d ? N.g26 : (h ? N.g15 : null), border: const Border(top: BorderSide(color: N.g20), bottom: BorderSide(color: N.g20))),
            child: Row(children: [
              _Chev(open),
              const SizedBox(width: 4),
              Flexible(
                child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.baseline, textBaseline: TextBaseline.alphabetic, children: [
                  Flexible(child: Text(open ? 'Hide advanced' : 'Advanced', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(N.g91))),
                  if (!open) ...[const SizedBox(width: 8), Text('$count more', maxLines: 1, style: _cnt(N.g76).copyWith(fontWeight: FontWeight.w500))],
                ]),
              ),
            ]),
          ),
        ),
      );
}

/// An empty or no-result state that says why and offers the next step (X6).
class _NoMatch extends StatelessWidget {
  const _NoMatch({required this.what, required this.next, required this.onNext});
  final String what, next;
  final VoidCallback onNext;
  @override
  Widget build(BuildContext context) => Padding(
        padding: const EdgeInsets.fromLTRB(12, 12, 12, 12),
        child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text(what, style: T.name(N.g91).copyWith(height: 1.3)),
          _gap(12),
          QuietButton(next, onTap: onNext),
        ]),
      );
}

List<WidgetbookUseCase> _i7Cases() {
  final fxs = {'Fractal Noise (${_fractal.params.length})': _fractal, 'Glow (${_glow.params.length})': _glow, '色相/彩度 (${_hue.params.length})': _hue, 'Gaussian Blur (${_blur.params.length})': _blur};
  _Fx fxKnob(BuildContext c) => fxs[c.knobs.object.dropdown<String>(label: 'Effect', options: fxs.keys.toList(), initialOption: fxs.keys.first, labelBuilder: (s) => s)]!;
  String layerKnob(BuildContext c) => c.knobs.object.dropdown<String>(label: 'Layer', options: const ['grain_overlay.mov', 'flicker_light_long_name_final_v3.mov'], initialOption: 'grain_overlay.mov', labelBuilder: (s) => s);
  const q = ['(type here)', 'sca', 'offset', 'ノイズ', 'bloom'];
  String queryKnob(BuildContext c) => c.knobs.object.dropdown<String>(label: 'Start with query', options: q, initialOption: q.first, labelBuilder: (s) => s == 'bloom' ? 'bloom  (no match)' : s).replaceFirst('(type here)', '');

  return [
    _story('I7-a Advanced fold', 'I7-a', 'Main properties first, the long tail folded with a count', _Habit.habit,
        'Contract: the fold says how many rows it hides; its state is kept across layers (switch Layer in the knobs, the fold stays); Effects with few properties show no fold.',
        (c, h) => _ParamPanel(fx: fxKnob(c), h: h, lang: _lang(c), layer: layerKnob(c), fold: true, presetOpen: c.knobs.boolean(label: 'Advanced open at start', initialValue: false))),
    _story('I7-a Advanced fold in a dock', 'I7-a', 'The same panel docked at 200 px', _Habit.habit,
        'Contract: below 220 px each label stacks over its value (one narrowing rule); the fold row and its count do not move.',
        (c, h) => _ParamPanel(fx: fxKnob(c), h: h, w: 200, lang: _lang(c), layer: layerKnob(c), fold: true, presetOpen: c.knobs.boolean(label: 'Advanced open at start', initialValue: true))),
    _story('I7-b Find and narrow', 'I7-b', 'A field on top narrows the rows as you type (groups flatten)', _Habit.habit,
        'Contract: updates per character; Esc or Clear returns to everything; no match says so and offers "Show all". Try the knob: sca / offset / ノイズ / bloom.',
        (c, h) => _ParamPanel(fx: fxKnob(c), h: h, lang: _lang(c), layer: layerKnob(c), find: true, presetQuery: queryKnob(c))),
    _story('I7-b Find with no match', 'I7-b', 'Nothing matches: say so, and give the way back', _Habit.habit,
        'Contract: an empty result is a sentence plus one button that returns to the full list (X6). The query stays in the field so it can be corrected.',
        (c, h) => _ParamPanel(fx: fxKnob(c), h: h, lang: _lang(c), find: true, presetQuery: 'bloom')),
    _story('I7-b Find in a dock', 'I7-b', 'Search in a 200 px dock', _Habit.habit,
        'Contract: the field keeps its count ("n of 26") and Clear; the highlighted match stays readable when the label wraps to two lines.',
        (c, h) => _ParamPanel(fx: fxKnob(c), h: h, w: 200, lang: _lang(c), find: true, presetQuery: 'offset')),
    _story('I7-c Pin with a star', 'I7-c', 'Star the rows you always touch: they sit on top, for every layer', _Habit.addition,
        'Contract: stars are remembered per effect type; rows without a star do not move; the star shows only on hover or when set. Knob sets how many are starred at start (0 = empty state).',
        (c, h) => _ParamPanel(fx: fxKnob(c), h: h, lang: _lang(c), layer: layerKnob(c), stars: true, presetStars: c.knobs.int.slider(label: 'Starred at start', initialValue: 3, min: 0, max: 6))),
    _story('I7-d Recent first', 'I7-d', 'Rows you touched last rise to the top (or keep their place)', _Habit.departs,
        'Contract: order follows touching; "Fixed order" keeps positions and shows recency as a grey tick. Departs from habit: a row\'s place changes under the hand (D-I7 does not adopt it). Scrub a row to see it jump.',
        (c, h) => _ParamPanel(fx: _fractal, h: h, lang: _lang(c), layer: layerKnob(c), recent: true)),
    _story('I7-ab Standard main and Find', 'I7-a + I7-b', 'The adopted pair (D-I7): main first, fold, and one search across everything', _Habit.habit,
        'Contract: the field is always one row; typing flattens the fold (Advanced rows appear in the result); empty search returns to the folded view. This is the draft to compare the others with.',
        (c, h) => _ParamPanel(fx: fxKnob(c), h: h, lang: _lang(c), layer: layerKnob(c), find: true, fold: true, presetQuery: queryKnob(c), presetOpen: c.knobs.boolean(label: 'Advanced open at start', initialValue: false))),
  ];
}
