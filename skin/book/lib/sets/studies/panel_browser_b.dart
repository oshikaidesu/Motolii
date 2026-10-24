// Panel drafts, Browser side, group B (research/options-browser.md): B9 hot-swap, B8 replace vs add, B7 click grammar, B10 favourites,
// B11 own presets, B12 large libraries, B13 context filter. Every use case is ONE panel at realistic size (320 wide Browser / 282 wide
// Inspector region / 232 narrow dock) with real-looking content. Two knobs on every one: Panel width, and Panel height (Fit list = the
// frame is as tall as its content, at most 700 / 700 tall = the stress frame). State is local to the panel (setState); a "stand-in" tray inside
// a panel plays the part of another panel, because cross-panel session state and drag and drop are out of scope for a draft.
// Colour: grey, one accent (C.mode). Text is g76 (10 px) / g63 (11 px) at the least. One left and right inset (10 px), the selection tick sits in the gutter (x 2-4).
import 'dart:async';
import 'dart:math' as math;
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart' show PointerScrollEvent;
import 'package:flutter/material.dart' show Tooltip;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../../parts/controls.dart';
import '../../tokens.dart';
import '../foundations/inputs_parts.dart' show TextBox;
import '../panels/media_parts.dart';
import '../foundations/navigation_parts.dart' as nav show G, Glyph;
import '../panels/transport_parts.dart' as tp show G, Glyph;
part 'panel_browser_b_2.dart';
part 'panel_browser_b_3.dart';
part 'panel_browser_b_4.dart';
part 'panel_browser_b_5.dart';
part 'panel_browser_b_6.dart';
part 'panel_browser_b_7.dart';
part 'panel_browser_b_8.dart';
part 'panel_browser_b_9.dart';
part 'panel_browser_b_10.dart';

// ================================================================ shared: ground, caption, frame

enum _Hab { habit, addition, departs, counter }

(String, String) _habWords(_Hab h) => switch (h) {
      _Hab.habit => ('HABIT', '手癖'),
      _Hab.addition => ('ADDITION', '追加'),
      _Hab.departs => ('DEPARTS', '手癖から外れる'),
      _Hab.counter => ('COUNTER', '反例'),
    };

const _clear = Color(0x00000000);
Color get _accent => Role.selected;

/// The one accent line weight of the book: panel frame, linked card, picker, filter chip all use this single alpha.
Color get _accentLine => _accent.withValues(alpha: .7);

/// The one left and right inset of a panel (and of every row, header and card in it). The selection tick is the only thing in the gutter (x 2-4).
const _px = 10.0;

/// The one small number style (counts, indices, clocks): mono 10 px, g76.
TextStyle _num([Color c = N.g76]) => T.value(c).copyWith(fontSize: 10);

/// Caption for a prop that stands in for another panel: one style (micro caps, same as a section title).
Widget _standIn(String s) => _t(s.toUpperCase(), T.micro(N.g76).copyWith(letterSpacing: .8));
const _tabs = ['Objects', 'Relations', 'Effects', 'Media'];

/// One use case: a ground, a one-line story caption outside the frame, and the panel. `make` builds the content for the chosen width.
/// Knobs: Panel width (the home width / narrow dock 232) and Panel height (Fit list = as tall as its content / 700 tall).
WidgetbookUseCase _uc(String name, String id, _Hab hab, String line, Widget Function(BuildContext c, double w) make, {double w = 320, double narrow = 232}) => WidgetbookUseCase(
      name: name,
      builder: (c) {
        final ww = c.knobs.object.dropdown<double>(label: 'Panel width', options: [w, narrow], initialOption: w, labelBuilder: (v) => v == narrow ? 'Narrow dock ${v.toInt()}' : (v > 300 ? 'Browser ${v.toInt()}' : 'Inspector ${v.toInt()}'));
        final fit = c.knobs.object.dropdown<String>(label: 'Panel height', options: const ['Fit list (default)', '700 tall'], initialOption: 'Fit list (default)', labelBuilder: (v) => v) != '700 tall';
        return _Ground(key: ValueKey(name), id: id, hab: hab, line: line, fit: fit, child: make(c, ww));
      },
    );

/// Whether the frame is only as tall as its content ("Fit list") or the 700 px stress frame. Lists and the column of the panel read it.
class _PanelH extends InheritedWidget {
  const _PanelH(this.fit, {required super.child});
  final bool fit;
  static bool fitOf(BuildContext c) => c.dependOnInheritedWidgetOfExactType<_PanelH>()?.fit ?? true;
  @override
  bool updateShouldNotify(_PanelH o) => o.fit != fit;
}

class _Ground extends StatelessWidget {
  const _Ground({super.key, required this.id, required this.hab, required this.line, required this.fit, required this.child});
  final String id, line;
  final _Hab hab;
  final bool fit;
  final Widget child;
  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g07,
        child: LayoutBuilder(builder: (c, box) => SingleChildScrollView(
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: ConstrainedBox(
                  constraints: BoxConstraints(minWidth: box.maxWidth, minHeight: box.hasBoundedHeight ? box.maxHeight : 0),
                  child: Padding(
                    padding: const EdgeInsets.all(12),
                    child: Center(
                      child: Column(mainAxisSize: MainAxisSize.min, children: [
                        ConstrainedBox(constraints: const BoxConstraints(maxWidth: 800), child: _Cap(id, hab, line)),
                        const SizedBox(height: 10),
                        _PanelH(fit, child: child),
                      ]),
                    ),
                  ),
                ),
              ),
            )),
      );
}

class _Cap extends StatelessWidget {
  const _Cap(this.id, this.hab, this.line);
  final String id, line;
  final _Hab hab;
  @override
  Widget build(BuildContext context) {
    final (word, jp) = _habWords(hab);
    return Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
      Text(id, style: T.name(N.g95)),
      const SizedBox(width: 8),
      _Tag('$word $jp', hi: hab == _Hab.departs),
      const SizedBox(width: 8),
      Expanded(child: Text(line, style: T.label(N.g76).copyWith(height: 1.35))),
    ]);
  }
}

/// The panel frame: g10 ground, one hairline, 6 px corner. Fit list: as tall as the content (max 700), so there is no empty band under the foot. 700 tall: exactly 700.
class _Frame extends StatelessWidget {
  const _Frame({required this.w, required this.child, this.accent = false});
  final double w;
  final Widget child;
  final bool accent;
  @override
  Widget build(BuildContext context) {
    final fit = _PanelH.fitOf(context);
    return Container(
      width: w,
      height: fit ? null : 700,
      constraints: fit ? const BoxConstraints(maxHeight: 700) : null,
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: accent ? _accentLine : N.g20)),
      child: child,
    );
  }
}

/// The column of a panel: shrink-wraps in Fit list, fills the frame in 700 tall.
class _Col extends StatelessWidget {
  const _Col(this.children);
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Column(mainAxisSize: _PanelH.fitOf(context) ? MainAxisSize.min : MainAxisSize.max, crossAxisAlignment: CrossAxisAlignment.stretch, children: children);
}

/// The one flexible region of a panel (where an Expanded child sits). Fit list: only as tall as its content (the list inside shrink-wraps), capped by the frame. 700 tall: it takes what is left, top-aligned.
class _Grow extends StatelessWidget {
  const _Grow({required this.child});
  final Widget child;
  @override
  Widget build(BuildContext context) => _PanelH.fitOf(context) ? Flexible(child: child) : Expanded(child: child);
}

/// The header block of a panel: the one inset, a column of stretched children.
class _Head extends StatelessWidget {
  const _Head(this.children, {this.bottom = 0});
  final List<Widget> children;
  final double bottom;
  @override
  Widget build(BuildContext context) => Padding(padding: EdgeInsets.fromLTRB(_px, 10, _px, bottom), child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: children));
}

/// Props that live outside the panel (a disk, a layer) so the panel itself stays honest.
class _Side extends StatelessWidget {
  const _Side(this.title, this.children);
  final String title;
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Container(
        width: 200,
        padding: const EdgeInsets.all(_px),
        decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
          Text(title.toUpperCase(), style: T.micro(N.g76).copyWith(letterSpacing: .8)),
          const SizedBox(height: 8),
          for (final c in children) Padding(padding: const EdgeInsets.only(bottom: 6), child: c),
        ]),
      );
}

Widget _withSide(Widget frame, Widget? side) => side == null ? frame : Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.start, children: [frame, const SizedBox(width: 14), side]);

// ================================================================ shared: small widgets

class _Tag extends StatelessWidget {
  const _Tag(this.text, {this.hi = false, this.warn = false});
  final String text;
  final bool hi, warn;
  @override
  Widget build(BuildContext context) => Container(
        height: 16,
        padding: const EdgeInsets.symmetric(horizontal: 5),
        decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(3), border: Border.all(color: hi ? _accent : (warn && !Role.grey ? Role.warning : _clear))),
        child: Center(widthFactor: 1, child: Text(text, maxLines: 1, style: T.micro(hi ? N.g95 : N.g76).copyWith(letterSpacing: .2))),
      );
}

/// A key mark: 16 px tall, 6 px sides, g20; text g76.
class _Kbd extends StatelessWidget {
  const _Kbd(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
        height: 16,
        constraints: const BoxConstraints(minWidth: 18),
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(3)),
        child: Center(widthFactor: 1, child: Text(text, style: _num())),
      );
}

class _Sec extends StatelessWidget {
  const _Sec(this.text, {this.n, this.trailing});
  final String text;
  final String? n;
  final Widget? trailing;
  @override
  Widget build(BuildContext context) => SizedBox(
        height: 22,
        child: Row(children: [
          Expanded(
            child: Row(children: [
              Flexible(child: Text(text.toUpperCase(), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g76).copyWith(letterSpacing: .8))),
              if (n != null) ...[const SizedBox(width: 6), Flexible(child: Text(n!, maxLines: 1, overflow: TextOverflow.ellipsis, style: _num()))],
            ]),
          ),
          if (trailing != null) ...[const SizedBox(width: 8), trailing!],
        ]),
      );
}

/// A 2 px selection tick inset in the left gutter (x 2 to 4, never on the panel edge); everything else in a row starts at x 10.
class _GutterTick extends StatelessWidget {
  const _GutterTick(this.on, {this.height = 16});
  final bool on;
  final double height;
  @override
  Widget build(BuildContext context) => Positioned(left: 2, top: 0, bottom: 0, width: 2, child: Center(child: AnimatedContainer(duration: Mo.dur, curve: Mo.ease, width: 2, height: height, decoration: BoxDecoration(color: on ? N.g95 : _clear, borderRadius: BorderRadius.circular(1)))));
}

/// The one list line: full width, the one inset, hover g15, selected g20 plus the gutter tick. `band` = a row band fill (layer rows); `line` = a hairline under it.
class _Line extends StatelessWidget {
  const _Line({required this.height, required this.child, this.sel = false, this.hover = false, this.band, this.line = false, this.tick = 16});
  final double height;
  final Widget child;
  final bool sel, hover, line;
  final Color? band;
  final double tick;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: height,
        decoration: BoxDecoration(color: sel ? N.g20 : (hover ? N.g15 : (band ?? _clear)), border: line ? const Border(bottom: BorderSide(color: N.rowLine)) : null),
        child: Stack(children: [
          Positioned.fill(child: Padding(padding: const EdgeInsets.symmetric(horizontal: _px), child: child)),
          _GutterTick(sel, height: tick),
        ]),
      );
}

/// Hover, click and a real double click (second tap inside 320 ms), without delaying the first click.
class _Hit extends StatefulWidget {
  const _Hit({required this.builder, this.onTap, this.onDouble, this.onSecondary, this.onHover, this.cursor = SystemMouseCursors.click});
  final Widget Function(BuildContext c, bool hover, Offset? pos) builder;
  final VoidCallback? onTap, onDouble, onSecondary;
  final ValueChanged<bool>? onHover;
  final MouseCursor cursor;
  @override
  State<_Hit> createState() => _HitState();
}

class _HitState extends State<_Hit> {
  bool _h = false;
  Offset? _p;
  DateTime? _last;
  void _tap() {
    final now = DateTime.now();
    if (widget.onDouble != null && _last != null && now.difference(_last!) < const Duration(milliseconds: 320)) {
      _last = null;
      widget.onDouble!();
    } else {
      _last = now;
      widget.onTap?.call();
    }
  }

  @override
  Widget build(BuildContext context) => MouseRegion(
        cursor: widget.cursor,
        onEnter: (_) {
          setState(() => _h = true);
          widget.onHover?.call(true);
        },
        onExit: (_) {
          setState(() {
            _h = false;
            _p = null;
          });
          widget.onHover?.call(false);
        },
        child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: _tap, onSecondaryTap: widget.onSecondary, child: widget.builder(context, _h, _p)),
      );
}

/// Hover, keyboard focus and pressed in one shell for the small buttons and chips. hover / focus / pressed pin a state (Parts); [onTap] null = disabled: no state shows, arrow cursor.
/// The look every one of them shares: hover = one grey step up, pressed = two, focus = a 1 px g63 edge.
class _Ctl extends StatefulWidget {
  const _Ctl({required this.builder, this.onTap, this.hover, this.focus, this.pressed});
  final Widget Function(BuildContext, bool hover, bool focus, bool pressed) builder;
  final VoidCallback? onTap;
  final bool? hover, focus, pressed;
  @override
  State<_Ctl> createState() => _CtlState();
}

class _CtlState extends State<_Ctl> {
  bool _h = false, _f = false, _p = false;
  @override
  Widget build(BuildContext context) {
    final off = widget.onTap == null;
    return FocusableActionDetector(
      enabled: !off,
      onShowHoverHighlight: (v) => setState(() => _h = v),
      onShowFocusHighlight: (v) => setState(() => _f = v),
      mouseCursor: off ? SystemMouseCursors.basic : SystemMouseCursors.click,
      actions: {ActivateIntent: CallbackAction<ActivateIntent>(onInvoke: (_) {
        widget.onTap?.call();
        return null;
      })},
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onTap,
        onTapDown: off ? null : (_) => setState(() => _p = true),
        onTapUp: (_) => setState(() => _p = false),
        onTapCancel: () => setState(() => _p = false),
        child: widget.builder(context, !off && (widget.hover ?? _h), !off && (widget.focus ?? _f), !off && (widget.pressed ?? _p)),
      ),
    );
  }
}

void _noop() {}

/// The quiet text button of these panels, 24 px: rest g13, hover g20, pressed g38, focus = a g63 edge; [primary] is one grey step up; onTap null = disabled (g10 ground, g63 word, no hover).
class _Btn extends StatelessWidget {
  const _Btn(this.text, {this.onTap, this.primary = false, this.hover, this.focus, this.pressed});
  final String text;
  final VoidCallback? onTap;
  final bool primary;
  final bool? hover, focus, pressed;
  @override
  Widget build(BuildContext context) {
    final off = onTap == null;
    return _Ctl(
      onTap: onTap,
      hover: hover,
      focus: focus,
      pressed: pressed,
      builder: (c, h, f, p) {
        final fill = off ? N.g10 : (p ? N.g38 : (primary ? (h ? N.g26 : N.g20) : (h ? N.g20 : N.g13)));
        return AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          height: 24,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          decoration: BoxDecoration(color: fill, borderRadius: BorderRadius.circular(4), border: Border.all(color: off ? _clear : (f ? N.g63 : (primary ? N.g26 : N.g20)))),
          child: CustomPaint(painter: off ? const DashedBox(N.g26, radius: 4, dash: 3, gap: 2) : null, child: Center(widthFactor: 1, child: Text(text, maxLines: 1, style: T.label(off ? N.g63 : N.g91).copyWith(fontWeight: FontWeight.w500)))),
        );
      },
    );
  }
}

/// A filter / sort / action chip, 22 px: rest g13, hover g20, pressed g38, on = g26 plus an accent underline, focus = a g63 edge; [enabled] false = g10 ground and g63 word.
class _Chip extends StatelessWidget {
  const _Chip(this.text, {this.n, this.on = false, this.onTap, this.hover, this.focus, this.pressed, this.enabled = true});
  final String text;
  final int? n;
  final bool on, enabled;
  final bool? hover, focus, pressed;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => _Ctl(
        onTap: enabled ? (onTap ?? _noop) : null,
        hover: hover,
        focus: focus,
        pressed: pressed,
        builder: (c, h, f, p) => Stack(children: [
          AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            height: 22,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            decoration: BoxDecoration(color: !enabled ? N.g10 : (p ? N.g38 : (on ? N.g26 : (h ? N.g20 : N.g13))), borderRadius: BorderRadius.circular(4), border: Border.all(color: f ? N.g63 : _clear)),
              child: CustomPaint(painter: enabled ? null : const DashedBox(N.g26, radius: 4, dash: 3, gap: 2), child: Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.center, children: [
              Text(text, maxLines: 1, style: T.label(!enabled ? N.g63 : (on ? N.g95 : N.g76)).copyWith(fontWeight: FontWeight.w500)),
              if (n != null) ...[const SizedBox(width: 5), Text('$n', style: _num())],
            ])),
          ),
          Positioned(left: 4, right: 4, bottom: 0, height: 1, child: AnimatedContainer(duration: Mo.dur, color: on ? _accent : _clear)),
        ]),
      );
}

/// Keyboard for one panel: it takes focus when the pointer goes down in it, and it never steals keys from a text field.
class _Keys extends StatefulWidget {
  const _Keys({required this.onKey, required this.child});
  final bool Function(KeyEvent e) onKey;
  final Widget child;
  @override
  State<_Keys> createState() => _KeysState();
}

class _KeysState extends State<_Keys> {
  final _n = FocusNode(debugLabel: 'panel keys');
  @override
  void dispose() {
    _n.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
        focusNode: _n,
        autofocus: true,
        onKeyEvent: (node, e) {
          if (FocusManager.instance.primaryFocus?.context?.findAncestorWidgetOfExactType<EditableText>() != null) return KeyEventResult.ignored;
          if (e is KeyUpEvent) {
            widget.onKey(e);
            return KeyEventResult.ignored;
          }
          return widget.onKey(e) ? KeyEventResult.handled : KeyEventResult.ignored;
        },
        child: Listener(behavior: HitTestBehavior.translucent, onPointerDown: (_) => _n.requestFocus(), child: widget.child),
      );
}

bool _isEnter(LogicalKeyboardKey k) => k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.numpadEnter;

/// A ghost that follows the pointer while something is carried (a stand-in for a real drag, which would cross panels).
class _Follow extends StatefulWidget {
  const _Follow({required this.child, this.ghost});
  final Widget child;
  final Widget? ghost;
  @override
  State<_Follow> createState() => _FollowState();
}

class _FollowState extends State<_Follow> {
  Offset? _p;
  @override
  Widget build(BuildContext context) => MouseRegion(
        onHover: widget.ghost == null ? null : (e) => setState(() => _p = e.localPosition),
        onExit: (_) => setState(() => _p = null),
        child: LayoutBuilder(builder: (c, box) {
          final flip = _p != null && _p!.dx > box.maxWidth * .5;
          return Stack(children: [
            widget.child,
            if (widget.ghost != null && _p != null)
              Positioned(
                left: flip ? null : _p!.dx + 14,
                right: flip ? box.maxWidth - _p!.dx + 14 : null,
                top: _p!.dy + 12,
                child: IgnorePointer(child: widget.ghost!),
              ),
          ]);
        }),
      );
}
