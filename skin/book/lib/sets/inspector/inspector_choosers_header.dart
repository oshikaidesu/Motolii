part of 'inspector_parts.dart';

// ---- 6. choosers and hand-overs -------------------------------------------------------------------------------------
/// A short list lives in a segmented control; a long one opens a searchable list (G2, G3). Never prev/next cycling (do-not-repeat 6, 7).
class Chooser extends StatefulWidget {
  const Chooser({super.key, required this.id, required this.options, this.searchable = false, this.word = 'Choose', this.enabled = true});
  final String id, word;
  final List<String> options;
  final bool searchable, enabled;
  @override
  State<Chooser> createState() => _ChooserState();
}

class _ChooserState extends State<Chooser> {
  bool _open = false, _up = false;
  final _q = TextEditingController();
  final _qf = FocusNode(debugLabel: 'chooser-q');
  final _link = LayerLink();
  final _portal = OverlayPortalController();

  static const _itemH = 22.0, _maxItems = 8;

  @override
  void dispose() {
    _q.dispose();
    _qf.dispose();
    super.dispose();
  }

  void _set(bool open) {
    if (open) {
      // contract: the list floats over the panel and never pushes the rows below it; it opens upward when there is no room underneath.
      final box = context.findRenderObject() as RenderBox?;
      final overlay = Overlay.maybeOf(context)?.context.findRenderObject() as RenderBox?;
      if (box != null && overlay != null) {
        final bottom = box.localToGlobal(Offset(0, box.size.height), ancestor: overlay).dy;
        final need = math.min(widget.options.length, _maxItems) * _itemH + (widget.searchable ? 24 : 0) + 12;
        _up = overlay.size.height - bottom < need && bottom - box.size.height > need;
      }
    }
    setState(() => _open = open);
    _q.clear();
    open ? _portal.show() : _portal.hide();
    if (open && widget.searchable) WidgetsBinding.instance.addPostFrameCallback((_) => _qf.requestFocus());
  }

  Widget _menu(BuildContext context, String cur, void Function(String) pick) {
    final hits = [
      for (final o in widget.options)
        if (o.toLowerCase().contains(_q.text.toLowerCase())) o,
    ];
    return Container(
      padding: const EdgeInsets.all(3),
      decoration: BoxDecoration(
        color: Grey.g15,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Grey.g26),
        boxShadow: const [BoxShadow(color: Color(0x99000000), blurRadius: 16, offset: Offset(0, 6))],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (widget.searchable)
            Container(
              height: 24,
              margin: const EdgeInsets.only(bottom: 3),
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(color: Grey.g10, borderRadius: BorderRadius.circular(5)),
              alignment: Alignment.centerLeft,
              child: Stack(
                alignment: Alignment.centerLeft,
                children: [
                  if (_q.text.isEmpty) Text('Search', style: T.label(Grey.g56)),
                  EditableText(
                    controller: _q,
                    focusNode: _qf,
                    style: T.name(Grey.g95),
                    cursorColor: Grey.g95,
                    backgroundCursorColor: Grey.g20,
                    maxLines: 1,
                    onChanged: (_) => setState(() {}),
                    onSubmitted: (_) {
                      if (hits.isNotEmpty) pick(hits.first);
                    },
                  ),
                ],
              ),
            ),
          SizedBox(
            height: math.min(math.max(hits.length, 1), _maxItems) * _itemH,
            child: hits.isEmpty
                ? Padding(
                    padding: const EdgeInsets.all(6),
                    child: Text('No match', style: T.label(Grey.g56)),
                  )
                : ListView.builder(
                    padding: EdgeInsets.zero,
                    itemExtent: _itemH,
                    itemCount: hits.length,
                    itemBuilder: (_, i) {
                      final o = hits[i], sel = o == cur;
                      return Hov(
                        onTap: () => pick(o),
                        builder: (_, h) => Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8),
                          decoration: BoxDecoration(color: h ? Grey.g26 : (sel ? Grey.g20 : null), borderRadius: BorderRadius.circular(5)),
                          alignment: Alignment.centerLeft,
                          child: Row(
                            children: [
                              Expanded(
                                child: Text(o, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(sel || h ? Grey.g95 : Grey.g76)),
                              ),
                              if (sel)
                                Container(
                                  width: 5,
                                  height: 5,
                                  decoration: BoxDecoration(color: Grey.g95, shape: BoxShape.circle),
                                ),
                            ],
                          ),
                        ),
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed), cur = x.doc.s2[widget.id] ?? widget.options.first;
    final on = widget.enabled && !x.cfg.locked && !RowOff.offOf(context);
    void pick(String o) {
      x.doc.str(widget.id, o);
      _set(false);
    }

    final field = Hov(
      onTap: () => _set(!_open),
      builder: (_, h) => Container(
        height: Pop.on(context) ? Pop.cell : _cellH,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: Pop.on(context)
            ? BoxDecoration(
                color: h || _open ? Grey.g10 : Pop.well,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: _open ? Grey.g63 : const Color(0x00000000)),
              )
            : BoxDecoration(
                color: !on ? Grey.g10 : (x.cfg.look == Look.quiet ? Grey.g13 : Grey.g07),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(
                  color: _open ? Grey.g63 : (h && on ? Grey.g38 : (x.cfg.look == Look.quiet && x.cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule)),
                ),
              ),
        // the value wins a narrow field: the word only shows when there is room for both
        child: LayoutBuilder(
          builder: (context, box) => Row(
            children: [
              // the default value (the first option) is dim; a chosen one is bright (dim = default)
              Expanded(
                child: Text(cur, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(!on || cur == widget.options.first ? Grey.g56 : Grey.g91)),
              ),
              if (box.maxWidth >= 120) Text(_open ? 'Close' : widget.word, style: T.label(Grey.g56)),
            ],
          ),
        ),
      ),
    );

    return Focus(
      onKeyEvent: (n, e) {
        if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _open) {
          _set(false);
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: IgnorePointer(
        ignoring: !on,
        child: CompositedTransformTarget(
          link: _link,
          child: OverlayPortal(
            controller: _portal,
            overlayChildBuilder: (_) {
              final w = (context.findRenderObject() as RenderBox?)?.size.width ?? 160;
              return Stack(
                children: [
                  Positioned.fill(
                    child: GestureDetector(behavior: HitTestBehavior.translucent, onTap: () => _set(false)),
                  ),
                  CompositedTransformFollower(
                    link: _link,
                    showWhenUnlinked: false,
                    targetAnchor: _up ? Alignment.topLeft : Alignment.bottomLeft,
                    followerAnchor: _up ? Alignment.bottomLeft : Alignment.topLeft,
                    offset: Offset(0, _up ? -4 : 4),
                    child: Align(
                      alignment: _up ? Alignment.bottomLeft : Alignment.topLeft,
                      child: SizedBox(width: math.max(w, 160), child: _menu(context, cur, pick)),
                    ),
                  ),
                ],
              );
            },
            child: field,
          ),
        ),
      ),
    );
  }
}

// ---- 7. header and empty ----------------------------------------------------------------------------------------------
class InspHeader extends StatelessWidget {
  const InspHeader({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), doc = x.doc, ed = Ed(x.cfg.ed);
    final (word, glyph) = switch (doc.kind) {
      Kind.shape => ('Shape', G.shape),
      Kind.text => ('Text', G.text),
      Kind.camera => ('Camera', G.camera),
      Kind.group => ('Group', G.grid),
      Kind.several => ('Layers', null),
      Kind.none => ('', null),
    };
    if (Pop.on(context)) {
      return Container(
        height: 34,
        padding: const EdgeInsets.symmetric(horizontal: Pop.inset),
        color: Pop.card,
        child: Row(
          children: [
            Container(
              width: 22,
              height: 22,
              alignment: Alignment.center,
              decoration: BoxDecoration(color: x.cfg.locked ? Grey.g26 : Pop.accent, borderRadius: BorderRadius.circular(6)),
              child: glyph == null ? null : Glyph(glyph, size: 13, color: x.cfg.locked ? Grey.g76 : Pop.onAccent),
            ),
            const SizedBox(width: 8),
            Flexible(
              child: Text(
                doc.name,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: T.title(x.cfg.locked ? Grey.g76 : Grey.g100).copyWith(fontSize: 13, fontWeight: FontWeight.w700, height: 1),
              ),
            ),
            const SizedBox(width: 6),
            Text((doc.kind == Kind.several ? 'Several' : word) + (x.cfg.locked ? ' · Locked' : ''), maxLines: 1, style: T.label(Grey.g56)),
            const Spacer(),
            Text('Animate all', style: T.label(Grey.g63)),
            const SizedBox(width: 6),
            OnOff(
              on: doc.animateAll,
              enabled: !x.cfg.locked,
              onChanged: (v) {
                doc.animateAll = v;
                doc.poke();
              },
            ),
          ],
        ),
      );
    }
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          if (glyph != null) ...[Glyph(glyph, size: 16, color: Grey.g76), const SizedBox(width: 8)],
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  ed.kick(doc.kind == Kind.several ? 'Several selected' : word) + (x.cfg.locked ? (ed.e == Editorial.b ? ' · Locked' : ' · LOCKED') : ''),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: ed.kickStyle(Grey.g56),
                ),
                const SizedBox(height: 4),
                Text(
                  doc.name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: ed.nameStyle().copyWith(color: x.cfg.locked ? Grey.g76 : null),
                ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          // contract: while on, editing an unkeyed value writes its first key at the playhead (D2).
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('Animate all', style: T.label(x.cfg.locked ? Grey.g56 : (doc.animateAll ? Grey.g95 : Grey.g63))),
              const SizedBox(height: 4),
              OnOff(
                on: doc.animateAll,
                enabled: !x.cfg.locked,
                onChanged: (v) {
                  doc.animateAll = v;
                  doc.poke();
                },
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class EmptyState extends StatelessWidget {
  const EmptyState({super.key});
  @override
  Widget build(BuildContext context) {
    final x = Ctx.of(context), ed = Ed(x.cfg.ed);
    // contract: one plain muted line, no controls, no stale values (A6). With the maybe-knob on, the composition's own settings take its place.
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 4, bottom: 8),
          child: Text('Nothing selected. Pick a layer on the Stage or in the Timeline.', style: T.label(Grey.g56).copyWith(height: 1.4)),
        ),
        if (x.cfg.maybe)
          Sect(
            title: 'Composition',
            first: true,
            children: [
              PropRow(
                id: 'comp.w',
                label: 'Width',
                keyable: false,
                cells: const [NumField(id: 'comp.w', unit: 'px', min: 16, max: 8192)],
              ),
              PropRow(
                id: 'comp.h',
                label: 'Height',
                keyable: false,
                cells: const [NumField(id: 'comp.h', unit: 'px', min: 16, max: 8192)],
              ),
              PropRow(
                id: 'comp.fps',
                label: 'Frame rate',
                keyable: false,
                cells: const [NumField(id: 'comp.fps', unit: 'fps', min: 1, max: 240)],
              ),
            ],
          )
        else
          const SizedBox(height: 1),
        if (!x.cfg.maybe) Container(height: 1, color: ed.rule),
      ],
    );
  }
}
