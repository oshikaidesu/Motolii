import 'dart:math' as math;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:lab_book/theme.dart';
import 'package:lab_book/sets/inspector/inspector_rows_prop_row.dart';
import 'package:lab_book/sets/inspector/inspector_rows_sect.dart';
import '../../parts/controls.dart';
import '../../parts/glyphs.dart';
import '../../tokens.dart';
import 'inspector_parts.dart';
import '../foundations/text_hand.dart';

// ---- 6. choosers and hand-overs -------------------------------------------------------------------------------------
/// A short list lives in a segmented control; a long one opens a searchable list (G2, G3). Never prev/next cycling (do-not-repeat 6, 7).
class Chooser extends StatefulWidget {
  const Chooser({super.key, required this.id, required this.options, this.searchable = false, this.enabled = true});
  final String id;
  final List<String> options;
  final bool searchable, enabled;
  @override
  State<Chooser> createState() => ChooserState();
}

class ChooserState extends State<Chooser> {
  /// Opened from outside: a tile around it, tapped anywhere (its glyph, its name), opens its list.
  void open() {
    if (widget.enabled && !_open) _set(true);
  }

  bool _open = false, _up = false;
  final _q = TextEditingController();
  late final _qf = FocusNode(debugLabel: 'chooser-q', onKeyEvent: _key);
  final _node = FocusNode(debugLabel: 'chooser');
  final _list = ScrollController();
  int _at = -1;   // (the one the keys are on)
  String _typed = '';   // (what has been typed to go to one, and when: a pause starts it again)
  DateTime _typedAt = DateTime(0);
  void Function(String)? _pick;
  final _link = LayerLink();
  final _portal = OverlayPortalController();

  static const _itemH = 22.0, _maxItems = 8;

  @override
  void dispose() {
    _q.dispose();
    _qf.dispose();
    _node.dispose();
    _list.dispose();
    super.dispose();
  }

  List<String> get _hits => [
    for (final o in widget.options)
      if (o.toLowerCase().contains(_q.text.toLowerCase())) o,
  ];

  void _go(int to) {
    final n = _hits.length;
    if (n == 0) return;
    setState(() => _at = to.clamp(0, n - 1));
    if (_list.hasClients) {
      final top = _at * _itemH, seen = _maxItems * _itemH;
      if (top < _list.offset) _list.jumpTo(top);
      if (top + _itemH > _list.offset + seen) _list.jumpTo(top + _itemH - seen);
    }
  }

  // (a select's keys, W3C's ARIA Authoring Practices, Select-Only Combobox: Down, Up, Return or Space open it; the arrows go
  //  along it, Return takes the one the keys are on, Esc leaves it; a letter typed goes to the first that starts so)
  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
    final k = e.logicalKey, down = k == LogicalKeyboardKey.arrowDown, up = k == LogicalKeyboardKey.arrowUp;
    final enter = k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter;
    final letter = !widget.searchable && (e.character?.trim().length ?? 0) == 1 && !HardwareKeyboard.instance.isMetaPressed && !HardwareKeyboard.instance.isControlPressed ? e.character! : null;
    if (!_open) {
      if (!down && !up && !enter && k != LogicalKeyboardKey.space && letter == null) return KeyEventResult.ignored;
      _set(true);
      if (letter == null) return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.escape) _set(false);
    else if (down) _go(_at + 1);
    else if (up) _go(_at - 1);
    else if (enter) {
      if (_at >= 0 && _at < _hits.length) _pick?.call(_hits[_at]);
    } else if (letter != null) {
      final now = DateTime.now();
      _typed = (now.difference(_typedAt) > const Duration(seconds: 1) ? '' : _typed) + letter.toLowerCase();
      _typedAt = now;
      final to = _hits.indexWhere((o) => o.toLowerCase().startsWith(_typed));
      if (to >= 0) _go(to);
    } else {
      return KeyEventResult.ignored;
    }
    return KeyEventResult.handled;
  }

  void _set(bool open) {
    if (open) {
      // contract: the list floats over the panel and never pushes the rows below it; it opens upward when there is no room underneath.
      final box = context.findRenderObject() as RenderBox?;
      final overlay = Overlay.maybeOf(context, rootOverlay: true)?.context.findRenderObject() as RenderBox?;
      if (box != null && overlay != null) {
        final bottom = box.localToGlobal(Offset(0, box.size.height), ancestor: overlay).dy;
        final need = math.min(widget.options.length, _maxItems) * _itemH + (widget.searchable ? 24 : 0) + 12;
        _up = overlay.size.height - bottom < need && bottom - box.size.height > need;
      }
    }
    setState(() => _open = open);
    _q.clear();
    open ? _portal.show() : _portal.hide();
    if (open) {
      _at = widget.options.indexOf(Ctx.read(context).doc.s2[widget.id] ?? widget.options.first);
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        (widget.searchable ? _qf : _node).requestFocus();
        _go(_at);   // (what it is now in sight, the keys on it)
      });
    }
  }

  Widget _menu(BuildContext context, String cur, void Function(String) pick) {
    final hits = _hits;
    return Container(
      padding: EdgeInsets.all(Shape.s(3)),
      decoration: BoxDecoration(
        color: Grey.g15,
        borderRadius: BorderRadius.all(Shape.r(8)),
        border: Border.all(color: Grey.g26),
        boxShadow: [BoxShadow(color: ColorTheme.of('Shadow', const Color(0x99000000)), blurRadius: 16, offset: const Offset(0, 6))],
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (widget.searchable)
            Container(
              height: Shape.z(24),
              margin: EdgeInsets.only(bottom: Shape.s(3)),
              padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
              decoration: BoxDecoration(color: Grey.g10, borderRadius: BorderRadius.all(Shape.r(5))),
              alignment: Alignment.centerLeft,
              child: Stack(
                alignment: Alignment.centerLeft,
                children: [
                  if (_q.text.isEmpty) Text('Search', style: T.label(Grey.g56)),
                  TextHand(field: (k) => EditableText(key: k, rendererIgnoresPointer: true,
                    controller: _q,
                    focusNode: _qf,
                    style: T.name(Grey.g95),
                    cursorColor: Grey.g95,
                    backgroundCursorColor: Grey.g20,
                    maxLines: 1,
                    onChanged: (_) => setState(() => _at = 0),
                  )),
                ],
              ),
            ),
          SizedBox(
            height: math.min(math.max(hits.length, 1), _maxItems) * _itemH,
            child: hits.isEmpty
                ? Padding(
                    padding: EdgeInsets.all(Shape.s(6)),
                    child: Text('No match', style: T.label(Grey.g56)),
                  )
                : ListView.builder(
                    controller: _list,
                    padding: EdgeInsets.zero,
                    itemExtent: _itemH,
                    itemCount: hits.length,
                    itemBuilder: (_, i) {
                      final o = hits[i], sel = o == cur;
                      return Hov(
                        onTap: () => pick(o),
                        builder: (_, h) => Container(
                          padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
                          decoration: BoxDecoration(color: h || i == _at ? Grey.g26 : (sel ? Grey.g20 : null), borderRadius: BorderRadius.all(Shape.r(5))),
                          alignment: Alignment.centerLeft,
                          child: Row(
                            children: [
                              Expanded(
                                child: Text(o, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(sel || h ? Grey.g95 : Grey.g76)),
                              ),
                              if (sel)
                                Container(
                                  width: Shape.z(5),
                                  height: Shape.z(5),
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

    _pick = pick;
    final field = Hov(
      onTap: () => _set(!_open),
      builder: (_, h) => Container(
        height: Pop.on(context) ? Pop.cell : cellH,
        padding: EdgeInsets.symmetric(horizontal: Shape.s(8)),
        decoration: Pop.on(context)
            ? BoxDecoration(
                color: h || _open ? Grey.g10 : Pop.well,
                borderRadius: BorderRadius.all(Shape.r(6)),
                border: Border.all(color: _open ? Grey.g63 : const Color(0x00000000)),
              )
            : BoxDecoration(
                color: !on ? Grey.g10 : (x.cfg.look == Look.quiet ? Grey.g13 : Grey.g07),
                borderRadius: BorderRadius.all(Shape.r(4)),
                border: Border.all(
                  color: _open ? Grey.g63 : (h && on ? Grey.g38 : (x.cfg.look == Look.quiet && x.cfg.ed == Editorial.a ? const Color(0x00000000) : ed.rule)),
                ),
              ),
        // (a menu says so by its mark, not a word)
        child: Row(
          children: [
            // the default value (the first option) is dim; a chosen one is bright (dim = default)
            Expanded(
              child: Text(cur, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(!on || cur == widget.options.first ? Grey.g56 : Grey.g91)),
            ),
            FoldMark(open: true, hot: _open),
          ],
        ),
      ),
    );

    return Focus(
      focusNode: _node,
      onKeyEvent: _key,
      child: IgnorePointer(
        ignoring: !on,
        child: CompositedTransformTarget(
          link: _link,
          // (the window's own overlay, not the nearest one: a list that hangs out of its pane is still under the hand)
          child: OverlayPortal.targetsRootOverlay(
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
    final w = doc.kindWord; // (what the host calls it, where the kinds have no word)
    final (word, glyph) = switch (doc.kind) {
      Kind.shape when w != null => (w, w == 'Film' || w == 'Picture' ? G.image : G.shape),
      Kind.shape => ('Shape', G.shape),
      Kind.text => ('Text', G.text),
      Kind.camera => ('Camera', G.camera),
      Kind.group => ('Group', G.grid),
      Kind.several => ('Layers', null),
      Kind.none => ('', null),
    };
    if (Pop.on(context)) {
      return Container(
        height: Shape.z(34),
        padding: EdgeInsets.symmetric(horizontal: Pop.inset),
        color: Pop.card,
        child: Row(
          children: [
            Container(
              width: Shape.z(22),
              height: Shape.z(22),
              alignment: Alignment.center,
              decoration: BoxDecoration(color: x.cfg.locked ? Grey.g26 : Pop.accent, borderRadius: BorderRadius.all(Shape.r(6))),
              child: glyph == null ? null : Glyph(glyph, size: Shape.z(13), color: x.cfg.locked ? Grey.g76 : Pop.onAccent),
            ),
            SizedBox(width: Shape.s(8)),
            Flexible(
              child: FittedBox( // (drawn smaller, never cut)
                fit: BoxFit.scaleDown,
                alignment: Alignment.centerLeft,
                child: Text(
                  doc.name,
                  maxLines: 1,
                  style: T.title(x.cfg.locked ? Grey.g76 : Grey.g100).copyWith(fontSize: T.px(13), fontWeight: FontWeight.w700, height: 1),
                ),
              ),
            ),
            SizedBox(width: Shape.s(6)),
            Text((doc.kind == Kind.several ? 'Several' : word) + (x.cfg.locked ? ' · Locked' : ''), maxLines: 1, style: T.label(Grey.g56)),
            const Spacer(),
            Text('Animate all', style: T.label(Grey.g63)),
            SizedBox(width: Shape.s(6)),
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
      padding: EdgeInsets.only(bottom: Shape.s(4)),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          if (glyph != null) ...[Glyph(glyph, size: Shape.z(16), color: Grey.g76), SizedBox(width: Shape.s(8))],
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
                SizedBox(height: Shape.s(4)),
                Text(
                  doc.name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: ed.nameStyle().copyWith(color: x.cfg.locked ? Grey.g76 : null),
                ),
              ],
            ),
          ),
          SizedBox(width: Shape.s(8)),
          // contract: while on, editing an unkeyed value writes its first key at the playhead (D2).
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('Animate all', style: T.label(x.cfg.locked ? Grey.g56 : (doc.animateAll ? Grey.g95 : Grey.g63))),
              SizedBox(height: Shape.s(4)),
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
          padding: EdgeInsets.only(top: Shape.s(4), bottom: Shape.s(8)),
          child: Text('Nothing selected', style: T.label(Grey.g56).copyWith(height: 1.4)),
        ),
        for (final c in x.doc.cards) ValuesCard(c), // (the host's own: the whole picture's values)
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
          SizedBox(height: Shape.s(1)),
        if (!x.cfg.maybe) Container(height: 1, color: ed.rule),
      ],
    );
  }
}
