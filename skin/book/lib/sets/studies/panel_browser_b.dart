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

class _Ghost extends StatelessWidget {
  const _Ghost(this.text, {this.thumb});
  final String text;
  final Widget? thumb;
  @override
  Widget build(BuildContext context) => Container(
        height: 26,
        padding: const EdgeInsets.fromLTRB(3, 3, 8, 3),
        decoration: BoxDecoration(color: N.g20, borderRadius: BorderRadius.circular(5), border: Border.all(color: N.g44), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))]),
        child: Row(mainAxisSize: MainAxisSize.min, children: [
          if (thumb != null) ...[SizedBox(width: 26, height: 20, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: thumb)), const SizedBox(width: 6)],
          Text(text, style: T.name(N.g95)),
        ]),
      );
}

/// What just happened (up to two lines). The keys that work here are a hover tooltip only (never text in the panel). Without a message or an action it takes no room.
class _Foot extends StatelessWidget {
  const _Foot({this.msg, this.hints = const [], this.trailing});
  final String? msg;
  final Widget? trailing;
  final List<(String, String)> hints;
  @override
  Widget build(BuildContext context) {
    if (msg == null && trailing == null) return const SizedBox.shrink();
    final bar = Container(
      padding: const EdgeInsets.fromLTRB(_px, 8, _px, 8),
      decoration: const BoxDecoration(color: N.g13, border: Border(top: BorderSide(color: N.g20))),
      child: Row(children: [if (msg != null && (msg!.startsWith('Added') || msg!.startsWith('Saved'))) const OkMark(size: 10, gap: 6), Expanded(child: Text(msg ?? '', maxLines: 2, overflow: TextOverflow.ellipsis, style: T.label(N.g91).copyWith(height: 1.25))), ...?(trailing == null ? null : [const SizedBox(width: 8), trailing!])]),
    );
    return hints.isEmpty ? bar : Tooltip(message: hints.map((h) => '${h.$1} ${h.$2}').join('  ·  '), waitDuration: const Duration(milliseconds: 500), child: bar);
  }
}

/// A read-only parameter row, grey only (a Fam hue would claim a relation).
class _Param extends StatelessWidget {
  const _Param(this.label, this.v, {this.hot = false, this.grip = false, this.changed = false});
  final String label;
  final double v;
  final bool hot, grip, changed;
  @override
  Widget build(BuildContext context) => Container(
        height: 20,
        decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)),
        child: Stack(children: [
          Positioned.fill(child: Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: v.clamp(0.0, 1.0), heightFactor: 1, child: DecoratedBox(decoration: BoxDecoration(color: hot ? N.g38 : N.g26, borderRadius: BorderRadius.circular(3)))))),
          if (grip) Positioned.fill(child: Align(alignment: Alignment.centerLeft, child: FractionallySizedBox(widthFactor: v.clamp(0.0, 1.0), heightFactor: 1, child: Align(alignment: Alignment.centerRight, child: Padding(padding: const EdgeInsets.only(right: 3), child: SizedBox(width: 5, height: 8, child: CustomPaint(painter: _GripP(hot ? N.g95 : N.g76)))))))),
          Positioned.fill(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: Row(children: [
            Expanded(child: Text(label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(N.g91).copyWith(decoration: changed ? TextDecoration.underline : null, decorationColor: N.g76, decorationThickness: 1))),
            Text((v * 40).toStringAsFixed(1), style: _num(N.g95)),
          ]))),
        ]),
      );
}

/// Two 1 px grip lines, 5 x 8: the mark of a field that is dragged.
class _GripP extends CustomPainter {
  const _GripP(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size size) {
    final p = Paint()..color = c..strokeWidth = 1;
    canvas.drawLine(const Offset(1, 0), Offset(1, size.height), p);
    canvas.drawLine(Offset(size.width - 1, 0), Offset(size.width - 1, size.height), p);
  }

  @override
  bool shouldRepaint(_GripP o) => o.c != c;
}

class _Tabs extends StatelessWidget {
  const _Tabs(this.i, {this.onChanged});
  final int i;
  final ValueChanged<int>? onChanged;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (c, box) => Segmented(items: box.maxWidth < 250 ? const ['Obj', 'Rel', 'Fx', 'Media'] : _tabs, index: i, expand: true, onChanged: onChanged));
}

/// The undo count: nothing at 0, otherwise the undo glyph and the number (hover says what it is).
Widget _undoMark(int n) => n <= 0
    ? const SizedBox.shrink()
    : Tooltip(message: 'Undo', waitDuration: const Duration(milliseconds: 500), child: Row(mainAxisSize: MainAxisSize.min, children: [const tp.Glyph(tp.G.undo, size: 12, color: N.g76), const SizedBox(width: 3), Text('$n', style: _num())]));

Widget _t(String s, TextStyle st, {int lines = 1}) => Text(s, maxLines: lines, overflow: TextOverflow.ellipsis, style: st);

double _tw(String s, TextStyle st) => (TextPainter(text: TextSpan(text: s, style: st), maxLines: 1, textDirection: TextDirection.ltr)..layout()).width;

/// The one truncation rule for every tile/row name in this file: a tail "…"; only a counter or extension suffix ("dusk_ridge_00…2.png") is kept.
String _fitMid(String s, TextStyle st, double maxW) {
  if (!maxW.isFinite || _tw(s, st) <= maxW - .5) return s;
  var tail = RegExp(r'\.\w{2,4}$').hasMatch(s) ? 8 : (RegExp(r'\d$').hasMatch(s) ? 3 : 0);
  tail = math.min(tail, s.length ~/ 3);
  final t = s.substring(s.length - tail);
  var lo = 0, hi = s.length - tail;
  while (lo < hi) {
    final m = (lo + hi + 1) >> 1;
    if (_tw('${s.substring(0, m)}…$t', st) <= maxW - .5) {
      lo = m;
    } else {
      hi = m - 1;
    }
  }
  return '${s.substring(0, lo)}…$t';
}

/// A one-line name with the one truncation rule ([_fitMid]).
class _Mid extends StatelessWidget {
  const _Mid(this.text, this.style);
  final String text;
  final TextStyle style;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (c, box) => Text(_fitMid(text, style, box.maxWidth), maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: style));
}

/// English name then Japanese name on one line. `whole`: the Japanese part is shown only when all of it fits (a cut "カラー化の…" tells nothing); otherwise the English name alone, with the one truncation rule.
class _NameJp extends StatelessWidget {
  const _NameJp(this.en, this.jp, this.enStyle, this.jpStyle, {this.whole = false});
  final String en, jp;
  final TextStyle enStyle, jpStyle;
  final bool whole;
  @override
  Widget build(BuildContext context) {
    final both = TextSpan(children: [TextSpan(text: en, style: enStyle), TextSpan(text: '  $jp', style: jpStyle)]);
    if (!whole) return Text.rich(both, maxLines: 1, overflow: TextOverflow.ellipsis);
    return LayoutBuilder(builder: (c, box) => _tw(en, enStyle) + _tw('  $jp', jpStyle) <= box.maxWidth - .5 ? Text.rich(both, maxLines: 1, softWrap: false, overflow: TextOverflow.clip) : Text(_fitMid(en, enStyle, box.maxWidth), maxLines: 1, softWrap: false, overflow: TextOverflow.clip, style: enStyle));
  }
}

/// The Inspector-region header of a layer (B9-b, B9-c): a plain glyph (never a plate that reads as a button), the kind, the name.
Widget _layerHead() => Row(children: [
      const nav.Glyph(nav.G.text, size: 14, color: N.g76),
      const SizedBox(width: 8),
      Expanded(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [_t('Text layer', T.label(N.g76)), const SizedBox(height: 3), _t('タイトル — Intro v3', T.title())])),
    ]);

// ================================================================ shared: the effect library (real names, Japanese and English)

enum _Sel { none, shape, text, key, many }

const _cats = ['Blur & Sharpen', 'Color', 'Distort', 'Stylize', 'Time', 'Generate', 'Transition', 'Text', 'Ease (key)'];
const _catShort = ['ぼかし', 'カラー', 'ゆがみ', '装飾', '時間', '生成', 'ワイプ', 'テキスト', 'ease'];
const _catJp = ['ぼかし・シャープ', 'カラー', 'ゆがみ', 'スタイライズ', '時間', '生成', 'トランジション', 'テキスト', 'イージング'];

const _raw = <(String, String, int)>[
  ('Gaussian Blur', 'ガウスぼかし', 0), ('Fast Box Blur', 'ボックスぼかし(高速)', 0), ('Directional Blur', '方向ぼかし', 0), ('Radial Blur', '放射ぼかし', 0), ('Lens Blur', 'レンズぼかし', 0), ('Sharpen', 'シャープ', 0), ('Unsharp Mask', 'アンシャープマスク', 0),
  ('Hue/Saturation', '色相・彩度', 1), ('Levels', 'レベル補正', 1), ('Curves', 'トーンカーブ', 1), ('Tint', '色合い', 1), ('Fill', '塗り', 1), ('Gradient Map', 'グラデーションマップ', 1), ('Exposure', '露光量', 1), ('Vibrance', '自然な彩度', 1), ('Color Balance', 'カラーバランス', 1),
  ('Wave Warp', '波形ワープ', 2), ('Twirl', '渦巻き', 2), ('Bulge', '膨張', 2), ('Turbulent Displace', 'タービュレント ディスプレイス', 2), ('Ripple', '波紋', 2), ('Corner Pin', 'コーナーピン', 2), ('Polar Coordinates', '極座標', 2), ('Mesh Warp', 'メッシュワープ', 2),
  ('Glow', 'グロー', 3), ('Drop Shadow', 'ドロップシャドウ', 3), ('Mosaic', 'モザイク', 3), ('Halftone', 'ハーフトーン', 3), ('Posterize', 'ポスタライズ', 3), ('Find Edges', '輪郭検出', 3), ('Vignette', 'ビネット', 3),
  ('Chromatic Aberration with Lens Distortion', '色収差+レンズ歪み(広角レンズ向け)', 3), ('Glitch RGB Split', 'グリッチ RGB ずらし', 3), ('Film Grain', 'フィルムグレイン', 3), ('Scanlines', '走査線', 3),
  ('Echo', 'エコー', 4), ('Posterize Time', 'コマ落とし', 4), ('Time Displacement', 'タイムディスプレイスメント', 4), ('Frame Blend', 'フレームブレンド', 4),
  ('Fractal Noise', 'フラクタルノイズ', 5), ('Gradient Ramp', 'グラデーションランプ', 5), ('Grid', 'グリッド', 5), ('Lens Flare', 'レンズフレア', 5), ('Light Sweep', '光の走査', 5), ('4-Color Gradient', '4色グラデーション', 5),
  ('Linear Wipe', 'リニアワイプ', 6), ('Radial Wipe', '時計ワイプ', 6), ('Venetian Blinds', 'ブラインド', 6), ('Iris Wipe', 'アイリスワイプ', 6), ('Block Dissolve', 'ブロックディゾルブ', 6),
  ('Typewriter', '文字送り', 7), ('Per-character Fade', '文字ごとフェード', 7), ('Tracking Reveal', '字間アニメ', 7), ('Text Scramble', '文字スクランブル', 7),
  ('Ease Out Expo', '急停止(Expo)', 8), ('Ease In-Out Cubic', 'なめらか(Cubic)', 8), ('Overshoot', '行き過ぎ', 8), ('Bounce', 'バウンス', 8), ('Elastic', 'ゴム', 8), ('Anticipate', '予備動作', 8), ('Hold', '保持', 8), ('Snap', 'スナップ', 8), ('Slow-Mo Ease', 'スローモーション', 8),
];

class _Fx {
  const _Fx(this.en, this.jp, this.cat, this.seed);
  final String en, jp;
  final int cat, seed;
  bool get isKey => cat == 8;
  bool get isText => cat == 7;
}

final List<_Fx> _fxs = [for (final (i, e) in _raw.indexed) _Fx(e.$1, e.$2, e.$3, i)];
final List<_Fx> _layerFx = _fxs.where((f) => !f.isKey && !f.isText).toList();

bool _fits(_Fx f, _Sel s) => switch (s) {
      _Sel.none => true,
      _Sel.key => f.isKey,
      _Sel.text => !f.isKey,
      _Sel.shape || _Sel.many => !f.isKey && !f.isText,
    };

String _why(_Fx f, _Sel s) => f.isKey ? 'Key only' : (s == _Sel.key ? 'Layer only' : 'Text only');
String _selName(_Sel s) => switch (s) {
      _Sel.none => 'Nothing selected',
      _Sel.shape => 'Layer 4 · Logo Mark (Shape)',
      _Sel.text => 'Layer 3 · タイトル (Text)',
      _Sel.key => '2 keys · Position X',
      _Sel.many => '5 layers (mixed)',
    };

(String, String) _params(_Fx f) => switch (f.cat) {
      0 => ('Blurriness', 'Repeat Edge'),
      1 => ('Amount', 'Channel'),
      2 => ('Amount', 'Size'),
      3 => ('Intensity', 'Radius'),
      4 => ('Offset', 'Count'),
      5 => ('Scale', 'Seed'),
      _ => ('Completion', 'Feather'),
    };


double _ease(String n, double t) {
  double bounce(double x) {
    const n1 = 7.5625, d1 = 2.75;
    if (x < 1 / d1) return n1 * x * x;
    if (x < 2 / d1) return n1 * (x -= 1.5 / d1) * x + .75;
    if (x < 2.5 / d1) return n1 * (x -= 2.25 / d1) * x + .9375;
    return n1 * (x -= 2.625 / d1) * x + .984375;
  }

  switch (n) {
    case 'Ease Out Expo':
      return t >= 1 ? 1 : 1 - math.pow(2, -10 * t).toDouble();
    case 'Ease In-Out Cubic':
      return t < .5 ? 4 * t * t * t : 1 - math.pow(-2 * t + 2, 3) / 2;
    case 'Overshoot':
      const c1 = 1.70158, c3 = c1 + 1;
      return 1 + c3 * math.pow(t - 1, 3) + c1 * math.pow(t - 1, 2);
    case 'Bounce':
      return bounce(t);
    case 'Elastic':
      return t <= 0 ? 0 : (t >= 1 ? 1 : math.pow(2, -10 * t) * math.sin((t * 10 - .75) * (2 * math.pi / 3)) + 1);
    case 'Anticipate':
      const c = 1.70158 * 1.2;
      return (c + 1) * t * t * t - c * t * t;
    case 'Hold':
      return t < 1 ? 0 : 1;
    case 'Snap':
      return t < .25 ? 0 : (t < .3 ? (t - .25) / .05 : 1);
    default:
      return t * t * (3 - 2 * t) * .6 + t * .4 * t;
  }
}

class _CurveP extends CustomPainter {
  const _CurveP(this.name);
  final String name;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..color = N.g07);
    final g = Paint()..color = N.g20..strokeWidth = 1;
    for (var i = 1; i < 4; i++) {
      c.drawLine(Offset(s.width * i / 4, 0), Offset(s.width * i / 4, s.height), g);
      c.drawLine(Offset(0, s.height * i / 4), Offset(s.width, s.height * i / 4), g);
    }
    final p = Path();
    const n = 48;
    for (var i = 0; i <= n; i++) {
      final t = i / n, y = _ease(name, t).clamp(-.4, 1.4);
      final o = Offset(s.width * (.1 + .8 * t), s.height * (.82 - .64 * y));
      i == 0 ? p.moveTo(o.dx, o.dy) : p.lineTo(o.dx, o.dy);
    }
    c.drawPath(p, Paint()..color = N.g95..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeJoin = StrokeJoin.round);
  }

  @override
  bool shouldRepaint(_CurveP o) => o.name != name;
}


// ================================================================ thumbnails: grey shapes, one shape per kind

/// The shape a thumbnail draws: one per effect family and one per media kind, so a row tells its kind without colour.
enum _Sh { blur, color, distort, stylize, time, generate, wipe, text, image, video, audio, model, pano }

_Sh _shOfFx(int cat) => const [_Sh.blur, _Sh.color, _Sh.distort, _Sh.stylize, _Sh.time, _Sh.generate, _Sh.wipe, _Sh.text][cat.clamp(0, 7)];
_Sh _shOfKind(Kind k) => switch (k) { Kind.image => _Sh.image, Kind.video => _Sh.video, Kind.audio => _Sh.audio, Kind.model => _Sh.model, Kind.pano => _Sh.pano };

/// A soft mid-grey tile with light-grey ink (never a near-black smear, never a hue). The seed nudges the size so siblings are not clones.
class _Ink extends CustomPainter {
  const _Ink(this.sh, this.seed);
  final _Sh sh;
  final int seed;
  @override
  void paint(Canvas c, Size s) {
    c.drawRect(Offset.zero & s, Paint()..shader = ui.Gradient.linear(Offset.zero, Offset(0, s.height), const [Color(0xFF6A6A6A), Color(0xFF585858)]));
    final w = s.width, h = s.height, u = h, m = Offset(w / 2, h / 2), k = .9 + (seed % 5) * .05, v = seed % 3;
    final ink = Paint()..color = N.g91.withValues(alpha: .8), soft = Paint()..color = N.g91.withValues(alpha: .45);
    final line = Paint()..color = N.g91.withValues(alpha: .85)..style = PaintingStyle.stroke..strokeWidth = math.max(1, u / 14)..strokeJoin = StrokeJoin.round..strokeCap = StrokeCap.round;
    switch (sh) {
      case _Sh.blur:
        if (v == 0) {
          for (var i = 2; i >= 0; i--) {
            c.drawCircle(m, u * (.14 + .11 * i) * k, Paint()..color = N.g91.withValues(alpha: [.8, .45, .22][i]));
          }
        } else if (v == 1) {
          for (var i = 0; i < 5; i++) {
            final y = h * (.26 + i * .12), a = [.25, .5, .85, .5, .25][i];
            c.drawLine(Offset(w * (.2 + (i % 2) * .06), y), Offset(w * (.8 - (i % 2) * .06), y), line..color = N.g91.withValues(alpha: a));
          }
          line.color = N.g91.withValues(alpha: .85);
        } else {
          for (var i = 2; i >= 0; i--) {
            c.drawCircle(m, u * (.12 + .12 * i) * k, line..color = N.g91.withValues(alpha: [.9, .55, .3][i]));
          }
          line.color = N.g91.withValues(alpha: .85);
        }
      case _Sh.color:
        final r = RRect.fromRectAndRadius(Rect.fromLTWH(w * .14, h * .26, w * .72, h * .48), Radius.circular(u * .08));
        final shader = v == 0
            ? ui.Gradient.linear(Offset(w * .14, 0), Offset(w * .86, 0), [N.g95.withValues(alpha: .9), N.g38.withValues(alpha: .9)])
            : (v == 1 ? ui.Gradient.linear(Offset(0, h * .26), Offset(0, h * .74), [N.g38.withValues(alpha: .9), N.g95.withValues(alpha: .9)]) : ui.Gradient.linear(Offset(w * .14, h * .26), Offset(w * .86, h * .74), [N.g95.withValues(alpha: .9), N.g38.withValues(alpha: .9), N.g95.withValues(alpha: .9)], const [0, .5, 1]));
        c.drawRRect(r, Paint()..shader = shader);
      case _Sh.distort:
        final p = Path();
        for (var i = 0; i <= 20; i++) {
          final t = i / 20, x = w * (.12 + .76 * t), y = v == 1 ? m.dy + (((t * 4 * k) % 1) < .5 ? -1 : 1) * u * .16 * (1 - ((t * 8 * k) % 1) * .0) : m.dy + math.sin(t * math.pi * 2 * k) * u * .2;
          i == 0 ? p.moveTo(x, y) : p.lineTo(x, y);
        }
        c.drawPath(p, line);
        if (v == 2) c.drawPath(p.shift(Offset(0, u * .2)), line..color = N.g91.withValues(alpha: .45));
        line.color = N.g91.withValues(alpha: .85);
      case _Sh.stylize:
        final q = u * .17;
        for (var i = 0; i < 3; i++) {
          for (var j = 0; j < 3; j++) {
            final o = Offset(m.dx + (i - 1) * q * 1.4, m.dy + (j - 1) * q * 1.4);
            if (v == 0) {
              c.drawRect(Rect.fromCenter(center: o, width: q, height: q), (i + j + seed) % 3 == 0 ? ink : soft);
            } else if (v == 1) {
              c.drawCircle(o, q * (.25 + .2 * ((i + j) % 3)), ink);
            } else {
              c.drawRect(Rect.fromCenter(center: o, width: q * (i == 1 ? 1 : .45), height: q * (j == 1 ? 1 : .45)), soft);
            }
          }
        }
      case _Sh.time:
        for (var i = 2; i >= 0; i--) {
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .2 + i * u * .13, h * .24 + i * u * .06, w * .42, h * .42), Radius.circular(u * .06)), Paint()..color = N.g91.withValues(alpha: [.85, .5, .25][i]));
        }
      case _Sh.generate:
        final r = math.Random(seed * 17 + 3);
        for (var i = 0; i < 16; i++) {
          c.drawCircle(Offset(w * (.14 + .72 * r.nextDouble()), h * (.16 + .68 * r.nextDouble())), u * (.04 + .04 * r.nextDouble()), i.isEven ? ink : soft);
        }
      case _Sh.wipe:
        c.drawPath(Path()..moveTo(0, 0)..lineTo(w * (.5 + (seed % 3) * .05), 0)..lineTo(w * (.36 + (seed % 3) * .05), h)..lineTo(0, h)..close(), Paint()..color = N.g91.withValues(alpha: .7));
      case _Sh.text:
        for (var i = 0; i < 3; i++) {
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .18, h * (.28 + i * .2), w * [.64, .5, .36][i] * k, u * .1), Radius.circular(u * .05)), i == 0 ? ink : soft);
        }
      case _Sh.image:
        c.drawPath(Path()..moveTo(w * .08, h * .82)..lineTo(w * .38, h * .36)..lineTo(w * .62, h * .82)..close(), ink);
        c.drawPath(Path()..moveTo(w * .46, h * .82)..lineTo(w * .7, h * .5)..lineTo(w * .94, h * .82)..close(), soft);
        c.drawCircle(Offset(w * .74, h * .26), u * .1 * k, ink);
      case _Sh.video:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(w * .16, h * .2, w * .68, h * .6), Radius.circular(u * .08)), line);
        c.drawPath(Path()..moveTo(m.dx - u * .1, m.dy - u * .15)..lineTo(m.dx + u * .17, m.dy)..lineTo(m.dx - u * .1, m.dy + u * .15)..close(), ink);
      case _Sh.audio:
        for (var i = 0; i < 9; i++) {
          final a = u * (.1 + .3 * (.5 + .5 * math.sin(i * 1.3 + seed))) * k;
          c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(w * (.16 + .085 * i), m.dy), width: math.max(1.5, w * .04), height: a * 2), const Radius.circular(1)), ink);
        }
      case _Sh.model:
        final r = u * .34 * k;
        final pts = [for (var i = 0; i < 6; i++) m + Offset(math.cos(math.pi / 3 * i - math.pi / 2) * r, math.sin(math.pi / 3 * i - math.pi / 2) * r)];
        c.drawPath(Path()..addPolygon(pts, true), line);
        for (var i = 1; i < 6; i += 2) {
          c.drawLine(m, pts[i], line);
        }
      case _Sh.pano:
        c.drawOval(Rect.fromLTWH(w * .1, h * .24, w * .8, h * .52), line);
        c.drawLine(Offset(w * .1, m.dy), Offset(w * .9, m.dy), line);
        c.drawCircle(Offset(w * .66, h * .38), u * .06 * k, ink);
    }
  }

  @override
  bool shouldRepaint(_Ink o) => o.sh != sh || o.seed != seed;
}

/// A media thumbnail: a grey tile whose shape says the kind (mountain = image, frame = video, bars = audio, cube = 3D, horizon = 360).
class _Art extends StatelessWidget {
  const _Art(this.asset);
  final Asset asset;
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _Ink(_shOfKind(asset.kind), asset.seed), size: Size.infinite);
}

class _Thumb extends StatelessWidget {
  const _Thumb(this.fx);
  final _Fx fx;
  @override
  Widget build(BuildContext context) => fx.isKey ? CustomPaint(painter: _CurveP(fx.en), size: Size.infinite) : CustomPaint(painter: _Ink(_shOfFx(fx.cat), fx.seed), size: Size.infinite);
}

/// The artist's own frame with the effect laid over it: a stand-in for "preview on my frame" (B5-c); the lab fakes the look per family, in grey.
class _Shot extends StatelessWidget {
  const _Shot(this.fx);
  final _Fx? fx;
  static const _base = _Art(Asset('shot', Kind.image, 3));
  @override
  Widget build(BuildContext context) {
    final f = fx;
    Widget body;
    if (f == null) {
      body = _base;
    } else if (f.cat == 0) {
      body = ImageFiltered(imageFilter: ui.ImageFilter.blur(sigmaX: 3, sigmaY: 3), child: _base);
    } else if (f.cat == 1) {
      body = Stack(fit: StackFit.expand, children: [_base, ColoredBox(color: N.g100.withValues(alpha: .1 + (f.seed % 3) * .05))]);
    } else {
      body = Stack(fit: StackFit.expand, children: [_base, Opacity(opacity: .45, child: _Thumb(f))]);
    }
    return ClipRRect(borderRadius: BorderRadius.circular(4), child: body);
  }
}

// ================================================================ shared: rows, tiles, grid, empty card

class _FxRow extends StatelessWidget {
  const _FxRow(this.fx, {this.sel = false, this.hover = false, this.tags = const [], this.whole = false});
  final _Fx fx;
  final bool sel, hover, whole;
  final List<Widget> tags;
  @override
  Widget build(BuildContext context) => _Line(
        height: 28,
        sel: sel,
        hover: hover,
        child: Row(children: [
          SizedBox(width: 22, height: 18, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(fx))),
          const SizedBox(width: 8),
          Expanded(child: _NameJp(fx.en, fx.jp, T.name(sel ? N.g95 : N.g91), T.label(N.g76), whole: whole)),
          for (final t in tags) Padding(padding: const EdgeInsets.only(left: 4), child: t),
        ]),
      );
}

class _FxTile extends StatelessWidget {
  const _FxTile(this.fx, this.width, {this.sel = false, this.hover = false, this.dim = false, this.reason});
  final _Fx fx;
  final double width;
  final bool sel, hover, dim;
  final String? reason;
  @override
  Widget build(BuildContext context) {
    final h = width * .62;
    return SizedBox(
      width: width,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
        Stack(children: [
          AnimatedContainer(
            duration: Mo.dur,
            curve: Mo.ease,
            width: width,
            height: h,
            foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: sel ? _accent : (hover ? N.g56 : N.g20))),
            child: ClipRRect(borderRadius: BorderRadius.circular(6), child: Opacity(opacity: dim ? .35 : 1, child: _Thumb(fx))),
          ),
          if (reason != null) Positioned(left: 4, right: 4, bottom: 4, child: Center(child: Container(padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 3), decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(3), border: Border.all(color: N.g26)), child: Text(reason!, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(N.g76).copyWith(letterSpacing: .1))))),
        ]),
        const SizedBox(height: 5),
        _Mid(fx.en, T.name(sel ? N.g95 : (dim ? N.g63 : N.g91))),
        const SizedBox(height: 3),
        _t(fx.jp, T.label(N.g76)),
      ]),
    );
  }
}

/// Columns by width: the tile never goes under 92 px.
class _Grid extends StatelessWidget {
  const _Grid({required this.n, required this.item});
  final int n;
  final Widget Function(int i, double w) item;
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (c, box) {
        final cols = math.max(2, ((box.maxWidth + 8) / 92).floor());
        final tw = (box.maxWidth - 8 * (cols - 1)) / cols;
        return Wrap(spacing: 8, runSpacing: 10, children: [for (var i = 0; i < n; i++) item(i, tw)]);
      });
}

/// A scrolling region that is cut by the frame: a 16 px fade into the panel ground at its foot says "there is more", so a half tile never looks like a bug.
Widget _fadeFoot(Widget child, {bool on = true}) => !on
    ? child
    : Stack(children: [
        child,
        Positioned(left: 0, right: 0, bottom: 0, height: 16, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [N.g10.withValues(alpha: 0), N.g10]))))),
      ]);

/// The foot fade only where the list really is cut: it reads the scroll metrics, so a list that fits shows no fade and a cut last row always has one.
class _Fade extends StatefulWidget {
  const _Fade({required this.child});
  final Widget child;
  @override
  State<_Fade> createState() => _FadeState();
}

class _FadeState extends State<_Fade> {
  bool _more = false;
  bool _note(Notification n) {
    final m = n is ScrollNotification ? n.metrics : (n is ScrollMetricsNotification ? n.metrics : null);
    if (m != null && m.axis == Axis.vertical) {
      final v = m.maxScrollExtent - m.pixels > 1;
      if (v != _more) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && v != _more) setState(() => _more = v);
        });
      }
    }
    return false;
  }

  @override
  Widget build(BuildContext context) => NotificationListener<Notification>(
        onNotification: _note,
        child: Stack(children: [
          widget.child,
          if (_more) Positioned(left: 0, right: 0, bottom: 0, height: 20, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, stops: const [0, .75, 1], colors: [N.g10.withValues(alpha: 0), N.g10.withValues(alpha: .85), N.g10])))))
        ]),
      );
}

/// A strip that is cut at its right edge: a 20 px fade into the panel ground (a half item is a scroll cue, not a cut word).
Widget _fadeSide(Widget child) => Stack(children: [
      child,
      Positioned(top: 0, bottom: 0, right: 0, width: 32, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(gradient: LinearGradient(begin: Alignment.centerLeft, end: Alignment.centerRight, stops: const [0, .7, 1], colors: [N.g10.withValues(alpha: 0), N.g10.withValues(alpha: .9), N.g10])))))
    ]);

/// An empty slot: a dashed outline with the one-word state in it (the outline says "a place that fills"), a reason line only when there is a fact to give, and the way out as a button.
Widget _empty(String title, String hint, {Widget? action, Color? tone}) => Padding(
      padding: const EdgeInsets.fromLTRB(_px, 12, _px, 12),
      child: Column(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        if (title.isNotEmpty) SizedBox(height: 56, child: CustomPaint(painter: DashedBox(tone != null && !Role.grey ? tone : N.g38), child: Center(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: Row(mainAxisSize: MainAxisSize.min, children: [if (tone != null) ErrMark(gap: 6, color: tone), Flexible(child: Text(title, textAlign: TextAlign.center, maxLines: 2, overflow: TextOverflow.ellipsis, style: T.name(N.g76)))]))))),
        if (hint.isNotEmpty) ...[const SizedBox(height: 8), Text(hint, textAlign: TextAlign.center, style: T.label(N.g63).copyWith(height: 1.35))],
        if (action != null) ...[const SizedBox(height: 10), Center(child: action)],
      ]),
    );

// ================================================================ B9-a Hot-swap: link mode (Q)

class _HotSwap extends StatefulWidget {
  const _HotSwap({required this.w, this.mode = 0, this.startLinked = false});
  final double w;
  final int mode; // 0 normal, 1 no effect on the layer yet, 2 layer locked
  final bool startLinked;
  @override
  State<_HotSwap> createState() => _HotSwapState();
}

class _HotSwapState extends State<_HotSwap> {
  bool _fam = false, _linked = false;
  int _cur = -1, _undo = 0;
  _Fx? _before, _now;
  String? _msg;
  final _sc = ScrollController();

  List<_Fx> get _vis => _fam ? _layerFx.where((f) => f.cat == 0).toList() : _layerFx;
  bool get _locked => widget.mode == 2;
  bool get _can => !_locked && _now != null;

  @override
  void initState() {
    super.initState();
    if (widget.mode == 0) _before = _now = _layerFx[0];
    if (widget.startLinked && _can) {
      _linked = true;
      _cur = 2;
      _now = _vis[2];
      _msg = 'Linked · Effect 2';
    }
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  void _scroll(int i) {
    if (_sc.hasClients) _sc.animateTo((i * 28.0 - 84).clamp(0.0, _sc.position.maxScrollExtent), duration: Mo.dur, curve: Mo.ease);
  }

  void _link() {
    if (_locked) {
      setState(() => _msg = 'Layer 3 is locked.');
      return;
    }
    if (_now == null) {
      setState(() => _msg = 'No effect on Layer 3.');
      return;
    }
    setState(() {
      _linked = true;
      _cur = math.max(0, _vis.indexOf(_before!));
      _msg = 'Linked · Effect 2';
    });
  }

  void _commit() => setState(() {
        _linked = false;
        _before = _now;
        _undo++;
        _msg = 'Kept ${_now!.en}';
      });

  void _cancel() => setState(() {
        _linked = false;
        _now = _before;
        _msg = 'Reverted to ${_before!.en}';
      });

  void _pick(int i) {
    setState(() {
      _cur = i.clamp(0, _vis.length - 1);
      if (_linked) {
        _now = _vis[_cur];
        _msg = 'Trying ${_now!.en}';
      }
    });
    _scroll(_cur);
  }

  void _add(int i) => setState(() {
        _before = _now = _vis[i];
        _undo++;
        _msg = 'Added ${_now!.en} to Layer 3';
      });

  bool _key(KeyEvent e) {
    if (e is KeyUpEvent) return false;
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.keyQ) {
      _linked ? _cancel() : _link();
      return true;
    }
    if (k == LogicalKeyboardKey.arrowDown) {
      _pick(_cur + 1);
      return true;
    }
    if (k == LogicalKeyboardKey.arrowUp) {
      _pick(_cur <= 0 ? 0 : _cur - 1);
      return true;
    }
    if (_isEnter(k) && _linked) {
      _commit();
      return true;
    }
    if (k == LogicalKeyboardKey.escape && _linked) {
      _cancel();
      return true;
    }
    return false;
  }

  Widget _target(double iw) {
    final shotW = (iw - 22 - 8) / 2;
    final Widget body;
    if (_locked) {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [Expanded(child: _t('Layer 3', T.title())), const _Tag('LOCKED · ロック中', hi: true)]),
        const SizedBox(height: 10),
        const _Btn('Link'),
      ]);
    } else if (_now == null) {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [Expanded(child: _t('Layer 3', T.title())), const _Tag('NO EFFECT · なし')]),
        const SizedBox(height: 10),
        const _Btn('Link'),
      ]);
    } else if (!_linked) {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text.rich(TextSpan(children: [TextSpan(text: _now!.en, style: T.title()), TextSpan(text: '  ${_now!.jp}', style: T.label(N.g76))]), maxLines: 1, overflow: TextOverflow.ellipsis),
        const SizedBox(height: 4),
        Text('2 / 3', style: _num()),
        const SizedBox(height: 10),
        Row(children: [_Btn('Link', onTap: _link), const SizedBox(width: 6), const _Kbd('Q')]),
      ]);
    } else {
      body = Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        _t('LINKED · not final', T.micro(N.g95)),
        const SizedBox(height: 6),
        SizedBox(height: 40, child: Row(children: [
          SizedBox(width: shotW, child: _Shot(_before)),
          const SizedBox(width: 8),
          SizedBox(width: shotW, child: DecoratedBox(position: DecorationPosition.foreground, decoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: _accent)), child: _Shot(_now))),
        ])),
        const SizedBox(height: 4),
        Row(children: [
          SizedBox(width: shotW, child: _t('BEFORE · ${_before!.en}', T.label(N.g76))),
          const SizedBox(width: 8),
          SizedBox(width: shotW, child: _t('NOW · ${_now!.en}', T.label(N.g95))),
        ]),
        const SizedBox(height: 10),
        Row(children: [_Btn('Keep', primary: true, onTap: _commit), const SizedBox(width: 4), const _Kbd('↵'), const SizedBox(width: 10), _Btn('Revert', onTap: _cancel), const SizedBox(width: 4), const _Kbd('Esc')]),
      ]);
    }
    return AnimatedSize(
      duration: Mo.dur,
      curve: Mo.ease,
      alignment: Alignment.topCenter,
      child: Container(padding: const EdgeInsets.all(_px), decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: _linked ? _accentLine : N.g20)), child: body),
    );
  }

  @override
  Widget build(BuildContext context) {
    final vis = _vis;
    return _Frame(
      w: widget.w,
      accent: _linked,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 10),
            LayoutBuilder(builder: (c, box) => _target(box.maxWidth)),
            const SizedBox(height: 10),
            Row(children: [
              _Chip('All', n: _layerFx.length, on: !_fam, onTap: () => setState(() => _fam = false)),
              const SizedBox(width: 6),
              _Chip('Blur family', n: 7, on: _fam, onTap: () => setState(() => _fam = true)),
              Expanded(child: Align(alignment: Alignment.centerRight, child: _undoMark(_undo))),
            ]),
            const SizedBox(height: 6),
          ]),
          _Grow(
            child: ListView.builder(
              controller: _sc,
              shrinkWrap: _PanelH.fitOf(context),
              padding: const EdgeInsets.only(bottom: 4),
              itemExtent: 28,
              itemCount: vis.length,
              itemBuilder: (c, i) {
                final f = vis[i];
                final isBefore = _before != null && f.en == _before!.en;
                return _Hit(
                  onTap: () => _pick(i),
                  onDouble: () => _linked ? _commit() : (_now == null && !_locked ? _add(i) : null),
                  builder: (c, h, _) => _FxRow(f, sel: i == _cur, hover: h, tags: [if (isBefore && !_linked) const _Tag('ON LAYER')]),
                );
              },
            ),
          ),
          _Foot(msg: _msg, hints: const [('↑↓', 'next / prev')]),
        ]),
      ),
    );
  }
}

/// An effect card of the Inspector's Effects region: thumb, name, two parameter bars. `replace` = a carried tile is over it.
class _CardFace extends StatelessWidget {
  const _CardFace(this.c, {this.replace = false, this.number = 1, this.kept = 0});
  final _Card c;
  final bool replace;
  final int number, kept;
  @override
  Widget build(BuildContext context) {
    final p = _params(c.fx);
    return AnimatedContainer(
      duration: Mo.dur,
      curve: Mo.ease,
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: replace ? N.g15 : N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: replace ? _accent : N.g20)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          SizedBox(width: 22, height: 16, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(c.fx))),
          const SizedBox(width: 8),
          Expanded(child: _NameJp(c.fx.en, c.fx.jp, T.name(N.g95), T.label(N.g76), whole: true)),
          const SizedBox(width: 4),
          if (replace) _Tag('REPLACE $kept/2', hi: true) else Text('$number', style: _num()),
        ]),
        const SizedBox(height: 6),
        _Param(p.$1, c.a, hot: replace),
        const SizedBox(height: 4),
        _Param(p.$2, c.b, hot: replace),
      ]),
    );
  }
}

/// A stand-in Browser tile in a tray: thumb and name. `on` = picked up.
class _TrayChip extends StatelessWidget {
  const _TrayChip(this.fx, {this.on = false, this.hover = false});
  final _Fx fx;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 24,
        padding: const EdgeInsets.fromLTRB(4, 0, 8, 0),
        decoration: BoxDecoration(color: on ? N.g26 : (hover ? N.g20 : N.g13), borderRadius: BorderRadius.circular(4), border: Border.all(color: on ? _accent : N.g20)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [SizedBox(width: 22, height: 16, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(fx))), const SizedBox(width: 8), Text(fx.en, style: T.label(N.g91))]),
      );
}

/// The 2 px insertion line under a card: Alt held while a tile is over it means "add a new card here".
class _AddLine extends StatelessWidget {
  const _AddLine(this.on);
  final bool on;
  @override
  Widget build(BuildContext context) => SizedBox(height: 8, child: Center(child: AnimatedContainer(duration: Mo.dur, height: 2, margin: const EdgeInsets.symmetric(horizontal: 4), decoration: BoxDecoration(color: on ? _accent : _clear, borderRadius: BorderRadius.circular(1)))));
}

// ================================================================ B9-b Drop replaces (Alt adds): the Inspector's Effects region

class _Card {
  _Card(this.fx, this.a, this.b);
  _Fx fx;
  double a, b;
}

List<_Card> _seedCards() => [
      _Card(_fxs[0], .30, .55),
      _Card(_fxs[24], .45, .20),
      _Card(_fxs[8], .60, .35),
      _Card(_fxs[7], .25, .70),
      _Card(_fxs[25], .50, .40),
    ];

class _DropCards extends StatefulWidget {
  const _DropCards({required this.w, this.altSim = false});
  final double w;
  final bool altSim;
  @override
  State<_DropCards> createState() => _DropCardsState();
}

class _DropCardsState extends State<_DropCards> {
  List<_Card> _cards = _seedCards();
  final _history = <List<_Card>>[];
  _Fx? _carry;
  int _hov = -1;
  bool _alt = false;
  String? _msg;
  static final _tray = [_fxs[2], _fxs[26], _fxs[33], _fxs[17], _fxs[38]];

  bool get _isAlt => _alt || widget.altSim;

  List<_Card> _copy() => [for (final c in _cards) _Card(c.fx, c.a, c.b)];

  void _drop(int i) {
    final fx = _carry;
    if (fx == null) return;
    _history.add(_copy());
    setState(() {
      if (_isAlt) {
        _cards.insert(i + 1, _Card(fx, .4, .4));
        _msg = 'Added ${fx.en} below card ${i + 1}';
      } else {
        final old = _cards[i], p0 = _params(old.fx), p1 = _params(fx);
        final keep = (p0.$1 == p1.$1 ? 1 : 0) + (p0.$2 == p1.$2 ? 1 : 0);
        _cards[i] = _Card(fx, p0.$1 == p1.$1 ? old.a : .4, p0.$2 == p1.$2 ? old.b : .4);
        _msg = 'Replaced ${old.fx.en} with ${fx.en} · $keep/2 values kept';
      }
      _carry = null;
    });
  }

  bool _key(KeyEvent e) {
    setState(() => _alt = HardwareKeyboard.instance.isAltPressed);
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _carry != null) {
      setState(() {
        _carry = null;
        _msg = 'Cancelled';
      });
      return true;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed) && _history.isNotEmpty) {
      setState(() {
        _cards = _history.removeLast();
        _msg = 'Undone.';
      });
      return true;
    }
    return false;
  }

  Widget _cardView(int i) {
    final c = _cards[i];
    final target = _carry != null && _hov == i;
    final replace = target && !_isAlt, add = target && _isAlt;
    final kept = replace ? () {
      final p0 = _params(c.fx), p1 = _params(_carry!);
      return (p0.$1 == p1.$1 ? 1 : 0) + (p0.$2 == p1.$2 ? 1 : 0);
    }() : 0;
    return _Hit(
      cursor: _carry != null ? SystemMouseCursors.copy : SystemMouseCursors.basic,
      onHover: (v) => setState(() => _hov = v ? i : (_hov == i ? -1 : _hov)),
      onTap: () => _drop(i),
      builder: (cx, h, _) => Column(children: [
        _CardFace(c, replace: replace, number: i + 1, kept: kept),
        _AddLine(add),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) => _Keys(
        onKey: _key,
        child: _Follow(
          ghost: _carry == null ? null : _Ghost(_isAlt ? '${_carry!.en}  + Add' : '${_carry!.en}  = Replace', thumb: _Thumb(_carry!)),
          child: _withSide(
            _Frame(
              w: widget.w,
              child: _Col([
                _Head([_layerHead(), const SizedBox(height: 10), const IgnorePointer(child: Segmented(items: ['Transform', 'Effects', 'Material'], index: 1, expand: true))], bottom: 8),
                Padding(padding: const EdgeInsets.fromLTRB(_px, 0, _px, 0), child: _Sec('Effects', n: '${_cards.length}', trailing: _undoMark(_history.length))),
                _Grow(child: _Fade(child: ListView.builder(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.fromLTRB(_px, 0, _px, 12), itemCount: _cards.length, itemBuilder: (c, i) => _cardView(i)))),
                _Foot(msg: _msg, hints: const [('drop', 'replace'), ('Alt', 'add instead'), ('Esc', 'cancel'), ('⌘Z', 'undo')]),
              ]),
            ),
            _Side('Browser (outside)', [
              for (final f in _tray) _Hit(onTap: () => setState(() => _carry = _carry == f ? null : f), builder: (c, h, _) => _TrayChip(f, on: _carry == f, hover: h)),
            ]),
          ),
        ),
      );
}

// ================================================================ B9-c ↑↓ on the effect card (same family, in place)

/// An effect card of the arrow-step panel: thumb, name, the family position, the two neighbours (when selected) and two parameter bars.
class _StepFace extends StatelessWidget {
  const _StepFace(this.f, this.v, {this.sel = false, this.hover = false, this.hoverUp = false, this.onStep});
  final _Fx f;
  final (double, double) v;
  final bool sel, hover, hoverUp;
  final ValueChanged<int>? onStep;
  @override
  Widget build(BuildContext context) {
    final fam = _fxs.where((x) => x.cat == f.cat).toList(), at = fam.indexWhere((x) => x.en == f.en), p = _params(f);
    Widget nb(String dir, _Fx? n, int d, bool pin) => Expanded(
          child: _Hit(
            onTap: n == null ? null : () => onStep?.call(d),
            cursor: n == null ? SystemMouseCursors.basic : SystemMouseCursors.click,
            builder: (c, h, _) => AnimatedContainer(duration: Mo.dur, height: 22, padding: const EdgeInsets.symmetric(horizontal: 6), alignment: Alignment.centerLeft, decoration: BoxDecoration(color: n != null && (h || pin) ? N.g20 : N.g10, borderRadius: BorderRadius.circular(4), border: Border.all(color: N.g20)), child: Row(children: [Text(dir, style: T.value(N.g76)), const SizedBox(width: 5), Expanded(child: _t(n?.en ?? 'end of family', T.label(n == null ? N.g63 : N.g76)))])),
          ),
        );
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: sel ? N.g15 : (hover ? N.g13 : N.g10), borderRadius: BorderRadius.circular(6), border: Border.all(color: sel ? _accentLine : N.g20)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [
          SizedBox(width: 22, height: 16, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: _Thumb(f))),
          const SizedBox(width: 8),
          Expanded(child: _NameJp(f.en, f.jp, T.name(N.g95), T.label(N.g76), whole: true)),
          const SizedBox(width: 4),
          _Tag('${_catShort[f.cat]} ${at + 1}/${fam.length}'),
        ]),
        if (sel) ...[
          const SizedBox(height: 6),
          Row(children: [nb('↑', at > 0 ? fam[at - 1] : null, -1, hoverUp), const SizedBox(width: 4), nb('↓', at < fam.length - 1 ? fam[at + 1] : null, 1, false)]),
        ],
        const SizedBox(height: 6),
        _Param(p.$1, v.$1, hot: sel),
        const SizedBox(height: 4),
        _Param(p.$2, v.$2, hot: sel),
      ]),
    );
  }
}

class _CardStep extends StatefulWidget {
  const _CardStep({required this.w});
  final double w;
  @override
  State<_CardStep> createState() => _CardStepState();
}

class _CardStepState extends State<_CardStep> {
  late List<_Fx> _fx = [_fxs[0], _fxs[7], _fxs[24], _fxs[26], _fxs[35], _fxs[42]];
  final _stack = <(int, _Fx)>[];
  int _sel = 0;
  String? _msg;
  final _vals = [(.30, .55), (.25, .70), (.45, .20), (.50, .40), (.35, .60), (.55, .30)];

  List<_Fx> _family(int cat) => _fxs.where((f) => f.cat == cat).toList();

  void _step(int d, [int? card]) {
    final i = card ?? _sel;
    final fam = _family(_fx[i].cat);
    final at = fam.indexWhere((f) => f.en == _fx[i].en), to = (at + d).clamp(0, fam.length - 1);
    if (to == at) {
      setState(() => _msg = '${_catJp[_fx[i].cat]} の${d > 0 ? '最後' : '最初'}です。');
      return;
    }
    setState(() {
      _sel = i;
      _stack.add((i, _fx[i]));
      final from = _fx[i];
      _fx = [..._fx]..[i] = fam[to];
      _msg = '${from.en} → ${fam[to].en}';
    });
  }

  void _undo() {
    if (_stack.isEmpty) return;
    final (i, f) = _stack.last;
    setState(() {
      _stack.removeLast();
      _fx = [..._fx]..[i] = f;
      _msg = 'Undone: back to ${f.en}.';
    });
  }

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return false;
    if (e.logicalKey == LogicalKeyboardKey.arrowDown) {
      _step(1);
      return true;
    }
    if (e.logicalKey == LogicalKeyboardKey.arrowUp) {
      _step(-1);
      return true;
    }
    if (e.logicalKey == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      _undo();
      return true;
    }
    return false;
  }

  Widget _card(int i) {
    final f = _fx[i], sel = i == _sel;
    return Listener(
      onPointerSignal: (s) {
        if (s is PointerScrollEvent && sel) _step(s.scrollDelta.dy > 0 ? 1 : -1);
      },
      child: _Hit(
        onTap: () => setState(() => _sel = i),
        cursor: SystemMouseCursors.basic,
        builder: (c, h, _) => _StepFace(f, _vals[i], sel: sel, hover: h, onStep: (d) => _step(d, i)),
      ),
    );
  }

  @override
  Widget build(BuildContext context) => _Frame(
        w: widget.w,
        child: _Keys(
          onKey: _key,
          child: _Col([
            _Head([
              _layerHead(),
              const SizedBox(height: 10),
              const IgnorePointer(child: Segmented(items: ['Transform', 'Effects', 'Material'], index: 1, expand: true)),
              const SizedBox(height: 10),
              _Sec('Effects', n: '${_fx.length}', trailing: _undoMark(_stack.length)),
            ], bottom: 0),
            _Grow(child: _Fade(child: ListView(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.fromLTRB(_px, 0, _px, 8), children: [for (var i = 0; i < _fx.length; i++) _card(i)]))),
            _Foot(msg: _msg, hints: const [('↑↓', 'same family'), ('wheel', 'on a card'), ('⌘Z', 'one step back')]),
          ]),
        ),
      );
}

// ================================================================ B9-d A/B: two frames, now and candidate

class _Ab extends StatefulWidget {
  const _Ab({required this.w, this.startOpen = true});
  final double w;
  final bool startOpen;
  @override
  State<_Ab> createState() => _AbState();
}

class _AbState extends State<_Ab> {
  _Fx _now = _layerFx[0];
  int _cand = -1, _undo = 0;
  String? _msg;
  final _sc = ScrollController();

  @override
  void initState() {
    super.initState();
    if (widget.startOpen) _cand = 10;
  }

  @override
  void dispose() {
    _sc.dispose();
    super.dispose();
  }

  void _keep() {
    if (_cand < 0) return;
    setState(() {
      _now = _layerFx[_cand];
      _msg = 'Kept ${_now.en}';
      _cand = -1;
      _undo++;
    });
  }

  void _close() => setState(() {
        _cand = -1;
        _msg = 'Closed';
      });

  void _set(int i) {
    setState(() {
      _cand = i.clamp(0, _layerFx.length - 1);
      _msg = 'Candidate: ${_layerFx[_cand].en}';
    });
    if (_sc.hasClients) _sc.animateTo((_cand * 28.0 - 56).clamp(0.0, _sc.position.maxScrollExtent), duration: Mo.dur, curve: Mo.ease);
  }

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return false;
    final k = e.logicalKey;
    if (k == LogicalKeyboardKey.arrowDown) {
      _set(_cand + 1);
      return true;
    }
    if (k == LogicalKeyboardKey.arrowUp) {
      _set(_cand < 0 ? 0 : _cand - 1);
      return true;
    }
    if (_isEnter(k)) {
      _keep();
      return true;
    }
    if (k == LogicalKeyboardKey.escape && _cand >= 0) {
      _close();
      return true;
    }
    return false;
  }

  Widget _pane(String cap, _Fx fx, bool cand) => Expanded(
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          AspectRatio(aspectRatio: 16 / 10, child: DecoratedBox(position: DecorationPosition.foreground, decoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: cand ? _accent : N.g20)), child: _Shot(fx))),
          const SizedBox(height: 5),
          _t(cap, T.micro(cand ? N.g95 : N.g76).copyWith(letterSpacing: .5)),
          const SizedBox(height: 3),
          _Mid(fx.en, T.name(cand ? N.g95 : N.g76)),
        ]),
      );

  Widget _compare() => AnimatedSize(
        duration: Mo.dur,
        curve: Mo.ease,
        alignment: Alignment.topCenter,
        child: _cand < 0
            ? SizedBox(height: 52, child: Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Row(children: [Expanded(child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Shot(_now))), const SizedBox(width: 8), Expanded(child: CustomPaint(painter: DashedBox(N.g38), child: const SizedBox.expand()))])))
            : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                const SizedBox(height: 4),
                Row(crossAxisAlignment: CrossAxisAlignment.start, children: [_pane('NOW', _now, false), const SizedBox(width: 8), _pane('CANDIDATE', _layerFx[_cand], true)]),
                const SizedBox(height: 8),
                Row(children: [_Btn('Keep', primary: true, onTap: _keep), const SizedBox(width: 4), const _Kbd('↵'), const SizedBox(width: 10), _Btn('Close', onTap: _close), const SizedBox(width: 4), const _Kbd('Esc')]),
              ]),
      );

  @override
  Widget build(BuildContext context) => _Frame(
        w: widget.w,
        accent: _cand >= 0,
        child: _Keys(
          onKey: _key,
          child: _Col([
            _Head([
              const IgnorePointer(child: _Tabs(2)),
              const SizedBox(height: 8),
              _Sec('Compare · Layer 3, Effect 2', trailing: _cand >= 0 ? const _Tag('PREVIEW · not written', hi: true) : _undoMark(_undo)),
              _compare(),
              const SizedBox(height: 6),
              _Sec('Candidates', n: '${_layerFx.length}'),
            ]),
            _Grow(
              child: _Fade(child: ListView.builder(
                controller: _sc,
                shrinkWrap: _PanelH.fitOf(context),
                padding: const EdgeInsets.only(bottom: 12),
                itemExtent: 28,
                itemCount: _layerFx.length,
                itemBuilder: (c, i) {
                  final f = _layerFx[i];
                  return _Hit(onTap: () => _set(i), onDouble: () {
                    _set(i);
                    _keep();
                  }, builder: (c, h, _) => _FxRow(f, sel: i == _cand, hover: h, whole: true, tags: [if (f.en == _now.en) const _Tag('NOW')]));
                },
              )),
            ),
            _Foot(msg: _msg, hints: const [('click', 'compare'), ('↑↓', 'next')]),
          ]),
        ),
      );
}

// ================================================================ B8 replace vs add: dropping an asset on a layer (the Timeline's layer column)

class _Ly {
  _Ly(this.name, this.src, this.keys);
  String name, src;
  int keys;
}

List<_Ly> _seedLayers() => [
      _Ly('タイトル / Title', 'Text', 5),
      _Ly('Subtitle サブタイトル', 'Text', 3),
      _Ly('BG Gradient', 'Solid', 0),
      _Ly('Logo Mark', 'logo_mark.png', 4),
      _Ly('City B-roll', 'city_pass.mov', 2),
      _Ly('Lens Bloom', 'lens_bloom.mp4', 6),
      _Ly('Particles 01', 'Shape', 8),
      _Ly('Particles 02', 'Shape', 8),
      _Ly('Light Leak', 'light_leak_03.mov', 1),
      _Ly('Caption JP 字幕(日本語)', 'Text', 0),
      _Ly('Adjustment 調整レイヤー', '—', 2),
      _Ly('Interview_Tanaka-san_Cam-B_take-07_final_v2', 'interview_b_07.mov', 0),
      _Ly('Camera 1', 'Camera', 12),
      _Ly('Null: Rig Controller', 'Null', 4),
    ];

/// One layer line of the B8 panel: index, name, source, key count. Bands alternate; the selected line is g20 with the gutter tick.
class _LayerRow extends StatelessWidget {
  const _LayerRow(this.l, this.i, {this.sel = false, this.hover = false, this.showSrc = true, this.height = 23});
  final _Ly l;
  final int i;
  final bool sel, hover, showSrc;
  final double height;
  @override
  Widget build(BuildContext context) => _Line(
        height: height,
        sel: sel,
        hover: hover,
        band: i.isEven ? N.bandA : N.bandB,
        line: true,
        tick: 13,
        child: Row(children: [
          SizedBox(width: 22, child: Text('${i + 1}', style: _num())),
          Expanded(child: _t(l.name, T.name(sel ? N.g95 : N.g91))),
          if (showSrc) SizedBox(width: 104, child: Padding(padding: const EdgeInsets.only(left: 8), child: _t(l.src, _num()))),
          SizedBox(width: 44, child: Text(l.keys == 0 ? '—' : '${l.keys} keys', textAlign: TextAlign.right, maxLines: 1, style: T.label(N.g76))),
        ]),
      );
}

/// One of the two faces of B8-d (Replace | Add). Lit = the pointer is over it.
class _Face extends StatelessWidget {
  const _Face(this.en, this.jp, this.on);
  final String en, jp;
  final bool on;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        margin: const EdgeInsets.all(2),
        decoration: BoxDecoration(color: on ? N.g26 : N.g20, borderRadius: BorderRadius.circular(5), border: Border.all(color: on ? _accent : N.g44)),
        child: Column(mainAxisAlignment: MainAxisAlignment.center, children: [Text(en.toUpperCase(), style: T.micro(N.g95)), const SizedBox(height: 4), _t(jp, T.label(N.g76))]),
      );
}

const _dropAssets = [
  Asset('lens_bloom.mp4', Kind.video, 7, dur: 12),
  Asset('夕焼け_空撮.mov', Kind.video, 14, dur: 47),
  Asset('logo_mark.png', Kind.image, 9),
  Asset('dusk_ridge.jpg', Kind.image, 3),
];

/// The dashed drop target (B8 new-layer area, B11 save area): lit when something is carried over it.
class _DropZone extends StatelessWidget {
  const _DropZone(this.text, {this.on = false});
  final String text;
  final bool on;
  @override
  Widget build(BuildContext context) => CustomPaint(painter: DashedBox(on ? _accent : N.g38), child: Center(child: Text(text, style: T.label(on ? N.g95 : N.g76))));
}

class _DropLab extends StatefulWidget {
  const _DropLab({required this.w, required this.mode, this.dwell = 1000, this.altSim = false});
  final double w;
  final int mode; // 0 add by default (Alt replaces), 1 replace by default (Alt adds), 2 layers refuse drops, 3 two faces after a dwell
  final int dwell;
  final bool altSim;
  @override
  State<_DropLab> createState() => _DropLabState();
}

class _DropLabState extends State<_DropLab> {
  static const _rh = 23.0, _voidH = 46.0;
  final List<_Ly> _ly = _seedLayers();
  final _hist = <List<_Ly>>[];
  Asset? _carry;
  int _hov = -2, _sel = 3, _face = -1, _half = -1;
  bool _alt = false;
  String? _msg;
  Timer? _timer;
  double _aw = 320;

  bool get _isAlt => _alt || widget.altSim;
  int get _n => _ly.length;

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  List<_Ly> _copy() => [for (final l in _ly) _Ly(l.name, l.src, l.keys)];
  int _rowAt(double y) => y < 0 ? -1 : (y < _n * _rh ? (y / _rh).floor() : _n);
  Rect _facesRect(int i) => Rect.fromLTWH(8, (i + 1) * _rh + 2, _aw - 16, 44);
  String _base(Asset a) => a.name.replaceAll(RegExp(r'\.[a-z0-9]+$'), '');

  void _do(int i, bool replace) {
    final a = _carry!;
    _hist.add(_copy());
    setState(() {
      if (replace) {
        final l = _ly[i];
        l.src = a.name;
        _msg = 'Replaced "${l.name}" · bar and ${l.keys} keys kept';
      } else {
        _ly.insert(i, _Ly(_base(a), a.name, 0));
        _sel = i;
        _msg = 'Added "${_base(a)}" above layer ${i + 1}';
      }
      _carry = null;
      _face = -1;
      _hov = -2;
    });
  }

  void _newTop() {
    final a = _carry!;
    _hist.add(_copy());
    setState(() {
      _ly.insert(0, _Ly(_base(a), a.name, 0));
      _sel = 0;
      _msg = 'New layer "${_base(a)}" on top';
      _carry = null;
      _hov = -2;
    });
  }

  void _hover(Offset p) {
    if (_carry == null) {
      final i = _rowAt(p.dy);
      if (i != _hov) setState(() => _hov = i);
      return;
    }
    if (widget.mode == 3 && _face >= 0 && _facesRect(_face).contains(p)) {
      setState(() => _half = p.dx < _aw / 2 ? 0 : 1);
      return;
    }
    final i = _rowAt(p.dy);
    if (i != _hov || _face >= 0) {
      _timer?.cancel();
      setState(() {
        _hov = i;
        _face = -1;
        _half = -1;
      });
      if (widget.mode == 3 && i >= 0 && i < _n) {
        _timer = Timer(Duration(milliseconds: widget.dwell), () {
          if (mounted && _hov == i) setState(() => _face = i);
        });
      }
    }
  }

  void _tap(Offset p) {
    final i = _rowAt(p.dy);
    if (_carry == null) {
      if (i >= 0 && i < _n) setState(() => _sel = i);
      return;
    }
    if (widget.mode == 3) {
      if (_face >= 0 && _facesRect(_face).contains(p)) {
        _do(_face, p.dx < _aw / 2);
      } else {
        setState(() {
          _carry = null;
          _face = -1;
          _msg = 'Cancelled';
        });
      }
      return;
    }
    if (i == _n) {
      _newTop();
    } else if (i >= 0) {
      switch (widget.mode) {
        case 0:
          _do(i, _isAlt);
        case 1:
          _do(i, !_isAlt);
        default:
          setState(() => _msg = 'ここには落とせません');
      }
    }
  }

  bool _key(KeyEvent e) {
    setState(() => _alt = HardwareKeyboard.instance.isAltPressed);
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _carry != null) {
      setState(() {
        _carry = null;
        _face = -1;
        _msg = 'Cancelled';
      });
      return true;
    }
    return false;
  }

  Widget _row(int i, bool showSrc) => _LayerRow(_ly[i], i, sel: i == _sel, hover: _carry == null && i == _hov, showSrc: showSrc, height: _rh);

  Widget _faceBox(String en, String jp, bool on) => Expanded(child: _Face(en, jp, on));

  Widget _overlay() {
    final h = _hov;
    final out = <Widget>[];
    if (_carry == null) return const SizedBox.shrink();
    if (h == _n) {
      out.add(Positioned(left: _px, right: _px, top: _n * _rh + 4, height: _voidH - 8, child: IgnorePointer(child: CustomPaint(painter: DashedBox(_accent), child: Center(child: Text('new layer on top', style: T.label(N.g95)))))));
    } else if (h >= 0 && h < _n) {
      final String? kind = switch (widget.mode) {
        0 => _isAlt ? 'replace' : 'add',
        1 => _isAlt ? 'add' : 'replace',
        2 => 'refuse',
        _ => _face < 0 ? 'dwell' : null,
      };
      if (kind == 'add') {
        out.add(Positioned(left: 4, right: 4, top: h * _rh - 1, height: 2, child: IgnorePointer(child: ColoredBox(color: _accent))));
        out.add(Positioned(right: _px, top: h * _rh + 3, child: const IgnorePointer(child: _Tag('ADD ABOVE', hi: true))));
      } else if (kind == 'replace') {
        out.add(Positioned(left: 1, right: 1, top: h * _rh, height: _rh, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(color: _accent.withValues(alpha: .12), border: Border.all(color: _accent))))));
        out.add(Positioned(right: _px, top: h * _rh + 3, child: const IgnorePointer(child: _Tag('REPLACE', hi: true))));
      } else if (kind == 'refuse') {
        out.add(Positioned(left: 0, right: 0, top: h * _rh, height: _rh, child: IgnorePointer(child: ColoredBox(color: N.g00.withValues(alpha: .55), child: Center(child: Text('ここには落とせません', style: T.label(N.g91)))))));
      } else if (kind == 'dwell') {
        out.add(Positioned(left: 1, right: 1, top: h * _rh, height: _rh, child: IgnorePointer(child: DecoratedBox(decoration: BoxDecoration(border: Border.all(color: N.g56))))));
      }
      if (widget.mode == 3 && _face >= 0) {
        final r = _facesRect(_face);
        out.add(Positioned(left: r.left, top: r.top, width: r.width, height: r.height, child: IgnorePointer(child: Container(decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(7), border: Border.all(color: N.g26), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 16, offset: const Offset(0, 6))]), child: Row(children: [_faceBox('Replace', '絵だけ差し替え', _half == 0), _faceBox('Add', '新規 layer', _half == 1)])))));
      }
    }
    return Stack(children: out);
  }

  @override
  Widget build(BuildContext context) {
    final m = widget.mode;
    final hints = switch (m) {
      0 => const [('drop', 'add'), ('Alt', 'replace'), ('Esc', 'cancel')],
      1 => const [('drop', 'replace'), ('Alt', 'add'), ('Esc', 'cancel')],
      2 => const [('drop', 'empty space or Source'), ('Esc', 'cancel')],
      _ => [('hold', '${(widget.dwell / 1000).toStringAsFixed(1)} s: two faces'), ('drop', 'on a face'), ('Esc', 'cancel')],
    };
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Follow(
          ghost: _carry == null ? null : _Ghost(_carry!.name, thumb: _Art(_carry!)),
          child: _Col([
            _Head([
              Row(children: [Expanded(child: _t('Layers · Intro v3', T.title())), _undoMark(_hist.length)]),
              const SizedBox(height: 8),
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g20)),
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  Row(children: [Expanded(child: _standIn('Browser')), if (_carry != null) const _Tag('CARRYING', hi: true)]),
                  const SizedBox(height: 6),
                  Row(children: [
                    for (final a in _dropAssets)
                      Expanded(
                        child: _Hit(
                          onTap: () => setState(() => _carry = _carry == a ? null : a),
                          builder: (c, h, _) => Padding(padding: const EdgeInsets.only(right: 4), child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                            AnimatedContainer(duration: Mo.dur, height: 34, foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: _carry == a ? _accent : (h ? N.g56 : N.g20))), child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Art(a))),
                            const SizedBox(height: 3),
                            _t(a.name, T.label(N.g76)),
                          ])),
                        ),
                      ),
                  ]),
                ]),
              ),
            ], bottom: 8),
            _Grow(
              child: LayoutBuilder(builder: (c, box) {
                _aw = box.maxWidth;
                final showSrc = box.maxWidth >= 300;
                return SingleChildScrollView(
                  physics: _carry == null ? null : const NeverScrollableScrollPhysics(),
                  child: MouseRegion(
                    onHover: (e) => _hover(e.localPosition),
                    onExit: (_) {
                      _timer?.cancel();
                      if (_hov != -2 || _face >= 0) {
                        setState(() {
                          _hov = -2;
                          _face = -1;
                        });
                      }
                    },
                    cursor: _carry == null ? MouseCursor.defer : (m == 2 && _hov >= 0 && _hov < _n ? SystemMouseCursors.forbidden : SystemMouseCursors.copy),
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTapUp: (d) => _tap(d.localPosition),
                      child: Stack(children: [
                        Column(children: [
                          for (var i = 0; i < _n; i++) _row(i, showSrc),
                          SizedBox(height: _voidH, child: Padding(padding: const EdgeInsets.symmetric(horizontal: _px, vertical: 4), child: _DropZone('New layer on top'))),
                        ]),
                        Positioned.fill(child: _overlay()),
                      ]),
                    ),
                  ),
                );
              }),
            ),
            if (m == 2)
              _Hit(
                onTap: () {
                  if (_carry != null) _do(_sel, true);
                },
                cursor: _carry == null ? SystemMouseCursors.basic : SystemMouseCursors.copy,
                builder: (c, h, _) => Container(
                  margin: const EdgeInsets.fromLTRB(_px, 8, _px, 8),
                  padding: const EdgeInsets.all(_px),
                  decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: _carry != null ? _accent : N.g20)),
                  child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Row(children: [Expanded(child: _t('INSPECTOR · LAYER ${_sel + 1}', T.micro(N.g76).copyWith(letterSpacing: .5))), if (_carry != null) const _Tag('DROP HERE', hi: true)]),
                    const SizedBox(height: 6),
                    Row(children: [Text('Source', style: T.label(N.g76)), const SizedBox(width: 10), Expanded(child: _t(_ly[_sel].src, T.value(N.g95)))]),
                  ]),
                ),
              ),
            _Foot(msg: _msg, hints: hints),
          ]),
        ),
      ),
    );
  }
}

// ================================================================ B7 click grammar: what click / double-click / Enter / drag mean on a shelf

class _MediaTile extends StatelessWidget {
  const _MediaTile(this.a, this.width, {this.sel = false, this.hover = false});
  final Asset a;
  final double width;
  final bool sel, hover;
  @override
  Widget build(BuildContext context) => SizedBox(
        width: width,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
          Stack(children: [
            AnimatedContainer(
              duration: Mo.dur,
              curve: Mo.ease,
              width: width,
              height: width * .62,
              foregroundDecoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: sel ? _accent : (hover ? N.g56 : N.g20))),
              child: ClipRRect(borderRadius: BorderRadius.circular(6), child: _Art(a)),
            ),
            if (a.kind != Kind.image) Positioned(left: 4, top: 4, child: TypeBadge(a.kind)),
            if (a.dur > 0) Positioned(right: 4, bottom: 4, child: TimePill(clock(a.dur))),
          ]),
          const SizedBox(height: 5),
          _Mid(a.name, T.name(sel ? N.g95 : N.g91)),
          const SizedBox(height: 3),
          _t(a.dims, T.label(N.g76)),
        ]),
      );
}

const _shelfIdx = [0, 2, 24, 25, 8, 7, 16, 17, 26, 31, 32, 34];
final List<_Fx> _shelf = [for (final i in _shelfIdx) _fxs[i]];
final List<Asset> _mediaShelf = [
  ...sampleAssets.take(7),
  const Asset('夕焼け_空撮_001.mov', Kind.video, 14, dur: 47, dims: '3840×2160'),
  const Asset('Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', Kind.video, 6, dur: 312),
  const Asset('背景_和紙テクスチャ.png', Kind.image, 21, dims: '4096×4096'),
];

/// The target line of B7-d: what a click will act on, in one line.
class _TargetLine extends StatelessWidget {
  const _TargetLine(this.sel);
  final _Sel sel;
  @override
  Widget build(BuildContext context) {
    final none = sel == _Sel.none;
    return Container(
      height: 30,
      padding: const EdgeInsets.symmetric(horizontal: _px),
      decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: none ? N.g20 : N.g38)),
      child: Row(children: [
        Text('TARGET', style: T.micro(N.g76).copyWith(letterSpacing: .6)),
        const SizedBox(width: 8),
        Expanded(child: _t(_selName(sel), T.name(none ? N.g63 : N.g95))),
      ]),
    );
  }
}

class _GrammarLab extends StatefulWidget {
  const _GrammarLab({required this.w, required this.mode, this.target = _Sel.text, this.startTab = 0});
  final double w;
  final int mode; // 0 select + dbl/Enter use, 1 two shelves with two meanings (counter-example), 2 single click uses, 3 browser shows its target
  final _Sel target;
  final int startTab;
  @override
  State<_GrammarLab> createState() => _GrammarLabState();
}

class _GrammarLabState extends State<_GrammarLab> {
  late int _tab = widget.startTab;
  final _sel = [-1, -1];
  final _stack = <String>['Gaussian Blur', 'Glow'];
  int _layers = 14, _cols = 3;
  final _undo = <(int, String)>[]; // (0 = effect added, 1 = layer added, name)
  String? _msg;

  _Sel get _target => widget.mode >= 2 ? widget.target : _Sel.text;
  int get _count => _tab == 0 ? _shelf.length : _mediaShelf.length;
  String _name(int tab, int i) => tab == 0 ? _shelf[i].en : _mediaShelf[i].name;

  /// Whether a gesture on this shelf means "use". This is the whole point of the case: one rule everywhere, or two.
  bool _singleUses(int tab) => switch (widget.mode) {
        0 => false,
        1 => tab == 0,
        _ => true,
      };

  void _use(int tab, int i) {
    final n = _name(tab, i);
    if (widget.mode >= 2 && _target == _Sel.none) {
      setState(() => _msg = widget.mode == 2 ? 'Select a layer' : 'Select a layer first');
      return;
    }
    if (tab == 0 && widget.mode >= 2 && !_fits(_shelf[i], _target)) {
      setState(() => _msg = '$n needs a Layer');
      return;
    }
    setState(() {
      if (tab == 0) {
        if (_target == _Sel.many) {
          _msg = 'Applied $n to 5 layers';
          _undo.add((2, n));
        } else {
          _stack.add(n);
          _msg = 'Added $n to Layer 3';
          _undo.add((0, n));
        }
      } else {
        _layers++;
        _msg = 'Added "$n" as layer $_layers';
        _undo.add((1, n));
      }
    });
  }

  void _doUndo() {
    if (_undo.isEmpty) return;
    final (k, n) = _undo.removeLast();
    setState(() {
      if (k == 0) _stack.removeLast();
      if (k == 1) _layers--;
      _msg = 'Undone: $n';
    });
  }

  void _select(int tab, int i) => setState(() {
        _sel[tab] = i;
        _msg = widget.mode == 0 ? 'Peek: ${_name(tab, i)}' : 'Selected ${_name(tab, i)}';
      });

  void _click(int tab, int i) => _singleUses(tab) ? _use(tab, i) : _select(tab, i);

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return false;
    final k = e.logicalKey;
    final cur = _sel[_tab];
    int? to;
    if (k == LogicalKeyboardKey.arrowRight) to = cur + 1;
    if (k == LogicalKeyboardKey.arrowLeft) to = cur - 1;
    if (k == LogicalKeyboardKey.arrowDown) to = cur + _cols;
    if (k == LogicalKeyboardKey.arrowUp) to = cur - _cols;
    if (to != null) {
      _select(_tab, to.clamp(0, _count - 1));
      return true;
    }
    if (_isEnter(k) && cur >= 0) {
      _use(_tab, cur);
      return true;
    }
    if (k == LogicalKeyboardKey.keyZ && (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed)) {
      _doUndo();
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final m = widget.mode;
    final hints = switch (m) {
      0 => const [('click', 'select · peek'), ('dbl / ↵', 'use'), ('drag', 'place'), ('⌘Z', 'undo')],
      1 => const [('click', 'effects: USE · media: select'), ('dbl', 'media: use')],
      2 => const [('click', 'use')],
      _ => const [('click / ↵', 'apply to the target only')],
    };
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            _Tabs(_tab + 2, onChanged: (i) {
              if (i >= 2) setState(() => _tab = i - 2);
            }),
            const SizedBox(height: 8),
            if (m == 3) ...[_TargetLine(_target), const SizedBox(height: 8)],
          ]),
          _Grow(
            child: _Fade(child: SingleChildScrollView(
              padding: const EdgeInsets.fromLTRB(_px, 0, _px, 12),
              child: LayoutBuilder(builder: (c, box) {
                _cols = math.max(2, ((box.maxWidth + 8) / 92).floor());
                return _Grid(
                  n: _count,
                  item: (i, w) => _Hit(
                    onTap: () => _click(_tab, i),
                    onDouble: _singleUses(_tab) ? null : () => _use(_tab, i),
                    builder: (c, h, _) => _tab == 0 ? _FxTile(_shelf[i], w, sel: _sel[0] == i, hover: h) : _MediaTile(_mediaShelf[i], w, sel: _sel[1] == i, hover: h),
                  ),
                );
              }),
            )),
          ),
          Container(
            padding: const EdgeInsets.fromLTRB(_px, 6, _px, 6),
            decoration: const BoxDecoration(color: N.g13, border: Border(top: BorderSide(color: N.g20))),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, mainAxisSize: MainAxisSize.min, children: [
              _Sec(_target == _Sel.many ? '5 layers' : 'Layer 3 · タイトル', n: '${_stack.length} effects', trailing: Text('comp $_layers layers', style: _num())),
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 66),
                child: ListView(shrinkWrap: true, padding: EdgeInsets.zero, children: [for (var i = _stack.length - 1; i >= 0; i--) SizedBox(height: 22, child: Row(children: [SizedBox(width: 20, child: Text('${i + 1}', style: _num())), Expanded(child: _t(_stack[i], T.name(i == _stack.length - 1 ? N.g95 : N.g76)))]))]),
              ),
            ]),
          ),
          _Foot(msg: _msg, hints: hints, trailing: m == 2 && _undo.isNotEmpty ? _Btn('Undo', onTap: _doUndo) : null),
        ]),
      ),
    );
  }
}

// ================================================================ B10 favourites and recent

const _collNames = ['Titles', 'Glitch', 'Soft light', 'Wipes 切替', 'Camera', 'Looks ルック', 'Client A', 'Night 夜', 'Scratch'];
final List<_Fx> _plainFx = _fxs.where((f) => !f.isKey).toList();

/// B10-a: nine named collections reached by the number keys 1-9; 0 clears. Names, not colours (D-8).
class _Collections extends StatefulWidget {
  const _Collections({required this.w, this.empty = false});
  final double w;
  final bool empty;
  @override
  State<_Collections> createState() => _CollectionsState();
}

class _CollectionsState extends State<_Collections> {
  final _mem = <String, Set<int>>{};
  int _sel = -1, _filter = 0; // 0 = all, 1..9 = a collection
  String? _msg;
  final _recent = [_fxs[24], _fxs[48], _fxs[0], _fxs[26], _fxs[8], _fxs[32], _fxs[16], _fxs[3], _fxs[27], _fxs[25], _fxs[7], _fxs[2]];

  @override
  void initState() {
    super.initState();
    if (!widget.empty) {
      void put(String en, List<int> c) => _mem[en] = {...c};
      put('Typewriter', [1]);
      put('Per-character Fade', [1]);
      put('Tracking Reveal', [1, 6]);
      put('Glitch RGB Split', [2]);
      put('Chromatic Aberration with Lens Distortion', [2, 8]);
      put('Scanlines', [2]);
      put('Glow', [3, 6]);
      put('Vignette', [3, 8]);
      put('Film Grain', [3, 8]);
      put('Linear Wipe', [4]);
      put('Radial Wipe', [4]);
      put('Block Dissolve', [4]);
      put('Lens Blur', [5]);
      put('Hue/Saturation', [6]);
      put('Levels', [6]);
      put('Gradient Map', [6, 8]);
    }
  }

  int _count(int c) => _mem.values.where((s) => s.contains(c)).length;
  List<_Fx> get _vis => _filter == 0 ? _plainFx : _plainFx.where((f) => _mem[f.en]?.contains(_filter) ?? false).toList();

  void _toggle(int c) {
    if (_sel < 0) {
      setState(() => _msg = 'Select an effect first');
      return;
    }
    final f = _plainFx[_sel], s = _mem.putIfAbsent(f.en, () => {});
    setState(() {
      if (!s.add(c)) s.remove(c);
      _msg = s.contains(c) ? '${f.en} → $c ${_collNames[c - 1]}' : '${f.en} removed from $c ${_collNames[c - 1]}';
    });
  }

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent) return false;
    final ch = e.character;
    if (ch != null && ch.length == 1 && '123456789'.contains(ch)) {
      _toggle(int.parse(ch));
      return true;
    }
    if (ch == '0' && _sel >= 0) {
      setState(() {
        _mem.remove(_plainFx[_sel].en);
        _msg = '${_plainFx[_sel].en}  removed from every collection';
      });
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final vis = _vis;
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 8),
            _Sec('Collections', trailing: _filter == 0 ? null : _Chip('Show all', onTap: () => setState(() => _filter = 0))),
            LayoutBuilder(builder: (c, box) {
              final cols = box.maxWidth >= 280 ? 3 : 2, cw = ((box.maxWidth - 4 * (cols - 1)) / cols).floorToDouble();
              return Wrap(spacing: 4, runSpacing: 4, children: [
                for (var n = 1; n <= 9; n++)
                  SizedBox(
                    width: cw,
                    child: _Hit(
                      onTap: () => setState(() => _filter = _filter == n ? 0 : n),
                      builder: (c, h, _) => _CollChip(n, _collNames[n - 1], _count(n), on: _filter == n, hover: h),
                    ),
                  ),
              ]);
            }),
            const SizedBox(height: 8),
            const _Sec('Recent'),
            SizedBox(
              height: 58,
              child: _fadeSide(ListView.separated(
                scrollDirection: Axis.horizontal,
                itemCount: _recent.length,
                separatorBuilder: (c, i) => const SizedBox(width: 6),
                itemBuilder: (c, i) => SizedBox(width: 66, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  SizedBox(height: 38, child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Thumb(_recent[i]))),
                  const SizedBox(height: 4),
                  _Mid(_recent[i].en, T.label(N.g76)),
                ])),
              )),
            ),
            const SizedBox(height: 4),
            _Sec(_filter == 0 ? 'All' : 'Matches', n: _filter == 0 ? '${vis.length}' : '${vis.length} of ${_plainFx.length}'),
          ]),
          _Grow(
            child: vis.isEmpty
                ? SingleChildScrollView(child: _empty('$_filter ${_collNames[_filter - 1]} はまだ空です', ''))
                : _Fade(child: ListView.builder(
                    shrinkWrap: _PanelH.fitOf(context),
                    padding: const EdgeInsets.only(bottom: 12),
                    itemExtent: 28,
                    itemCount: vis.length,
                    itemBuilder: (c, i) {
                      final f = vis[i], at = _plainFx.indexOf(f), m = [for (final n in (_mem[f.en] ?? <int>{}).toList()..sort()) if (n != _filter) n];
                      return _Hit(onTap: () => setState(() => _sel = at), builder: (c, h, _) => _FxRow(f, sel: _sel == at, hover: h, whole: true, tags: [for (final n in m.take(3)) Tooltip(message: '$n ${_collNames[n - 1]}', waitDuration: const Duration(milliseconds: 400), child: _Tag('$n')), if (m.length > 3) const _Tag('…')]));
                    },
                  )),
          ),
          _Foot(msg: _msg, hints: const [('1-9', 'add / remove'), ('0', 'clear'), ('click', 'filter')]),
        ]),
      ),
    );
  }
}

/// A numbered collection button (1-9): key number, name, how many it holds.
class _CollChip extends StatelessWidget {
  const _CollChip(this.n, this.name, this.count, {this.on = false, this.hover = false});
  final int n, count;
  final String name;
  final bool on, hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 24,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        decoration: BoxDecoration(color: on ? N.g26 : (hover ? N.g20 : N.g13), borderRadius: BorderRadius.circular(4), border: Border.all(color: on ? _accent : N.g20)),
        child: Row(children: [
          Text('$n', style: _num(N.g95)),
          const SizedBox(width: 4),
          Expanded(child: _t(name, T.label(on ? N.g95 : N.g76))),
          const SizedBox(width: 4),
          Text(count == 0 ? '–' : '$count', style: _num(count == 0 ? N.g63 : N.g76)),
        ]),
      );
}

/// B10-b: one star (F), recents and frequents fill themselves.
class _StarAuto extends StatefulWidget {
  const _StarAuto({required this.w, this.start = 0, this.fresh = false});
  final double w;
  final int start;
  final bool fresh;
  @override
  State<_StarAuto> createState() => _StarAutoState();
}

class _StarAutoState extends State<_StarAuto> {
  late int _view = widget.start;
  final _stars = <String>{};
  final _use = <String, int>{};
  final _recent = <String>[];
  int _sel = -1;
  String? _msg;

  @override
  void initState() {
    super.initState();
    if (!widget.fresh) {
      _stars.addAll(['Glow', 'Gaussian Blur', 'Vignette', 'Typewriter']);
      const u = {'Gaussian Blur': 41, 'Glow': 33, 'Levels': 19, 'Drop Shadow': 17, 'Hue/Saturation': 12, 'Linear Wipe': 9, 'Fractal Noise': 6, 'Echo': 3};
      _use.addAll(u);
      _recent.addAll(['Levels', 'Glow', 'Linear Wipe', 'Gaussian Blur', 'Echo', 'Drop Shadow', 'Fractal Noise', 'Hue/Saturation']);
    }
  }

  List<_Fx> get _vis {
    final byName = {for (final f in _plainFx) f.en: f};
    switch (_view) {
      case 1:
        return [for (final f in _plainFx) if (_stars.contains(f.en)) f];
      case 2:
        return [for (final n in _recent) byName[n]!];
      case 3:
        final k = _use.keys.toList()..sort((a, b) => _use[b]!.compareTo(_use[a]!));
        return [for (final n in k) byName[n]!];
      default:
        return _plainFx;
    }
  }

  void _star(_Fx f) => setState(() {
        _stars.contains(f.en) ? _stars.remove(f.en) : _stars.add(f.en);
        _msg = _stars.contains(f.en) ? 'Starred ${f.en}' : 'Unstarred ${f.en}';
      });

  void _useIt(_Fx f) => setState(() {
        _use[f.en] = (_use[f.en] ?? 0) + 1;
        _recent.remove(f.en);
        _recent.insert(0, f.en);
        if (_recent.length > 12) _recent.removeLast();
        _msg = 'Used ${f.en} (${_use[f.en]}×)';
      });

  bool _key(KeyEvent e) {
    if (e is! KeyDownEvent) return false;
    if (e.logicalKey == LogicalKeyboardKey.keyF && _sel >= 0 && _sel < _vis.length) {
      _star(_vis[_sel]);
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final vis = _vis;
    final emptyMsg = switch (_view) {
      1 => ('星はまだありません', ''),
      2 => ('まだ何も使っていません', ''),
      3 => ('まだ回数がありません', ''),
      _ => ('', ''),
    };
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 8),
            LayoutBuilder(builder: (c, box) => Segmented(items: box.maxWidth < 250 ? const ['All', 'Stars', 'Recent', 'Often'] : const ['All', 'Starred', 'Recent', 'Frequent'], index: _view, expand: true, onChanged: (i) => setState(() {
                  _view = i;
                  _sel = -1;
                }))),
            const SizedBox(height: 6),
            _Sec(const ['By family', 'By family', 'Latest first', 'By use count'][_view], n: '${vis.length}'),
          ]),
          _Grow(
            child: vis.isEmpty
                ? SingleChildScrollView(child: _empty(emptyMsg.$1, emptyMsg.$2))
                : _Fade(child: ListView.builder(
                    shrinkWrap: _PanelH.fitOf(context),
                    padding: const EdgeInsets.only(bottom: 12),
                    itemExtent: 28,
                    itemCount: vis.length,
                    itemBuilder: (c, i) {
                      final f = vis[i], on = _stars.contains(f.en);
                      return _Hit(
                        onTap: () => setState(() => _sel = i),
                        onDouble: () => _useIt(f),
                        builder: (c, h, _) => _FxRow(f, sel: _sel == i, hover: h, whole: true, tags: [
                          if (_view == 3) Text('${_use[f.en]}×', style: _num()),
                          StarButton(on: on, onChanged: (_) => _star(f), size: 14),
                        ]),
                      );
                    },
                  )),
          ),
          _Foot(msg: _msg, hints: const [('F', 'star'), ('dbl', 'use'), ('click', 'select')]),
        ]),
      ),
    );
  }
}

/// B10-c: a pile of what was used; the same thing folds to the front; the oldest falls off.
class _Pile extends StatefulWidget {
  const _Pile({super.key, required this.w, this.cap = 12});
  final double w;
  final int cap;
  @override
  State<_Pile> createState() => _PileState();
}

class _PileState extends State<_Pile> {
  final List<(String, int)> _pile = [('Glow', 33), ('Levels', 19), ('Gaussian Blur', 41), ('Drop Shadow', 17), ('Hue/Saturation', 12)];
  String? _flash, _msg;
  Timer? _tick2;

  @override
  void dispose() {
    _tick2?.cancel();
    super.dispose();
  }

  void _use(_Fx f) {
    final at = _pile.indexWhere((e) => e.$1 == f.en);
    final n = at < 0 ? 1 : _pile[at].$2 + 1;
    setState(() {
      if (at >= 0) _pile.removeAt(at);
      _pile.insert(0, (f.en, n));
      String? dropped;
      while (_pile.length > widget.cap) {
        dropped = _pile.removeLast().$1;
      }
      _flash = f.en;
      _msg = at >= 0 ? '${f.en} moved to the front' : '${f.en} on top${dropped == null ? '' : ' · $dropped dropped'}';
    });
    _tick2?.cancel();
    _tick2 = Timer(const Duration(milliseconds: 900), () {
      if (mounted) setState(() => _flash = null);
    });
  }

  @override
  Widget build(BuildContext context) {
    final byName = {for (final f in _plainFx) f.en: f};
    final rows = math.min(_pile.length, 7);
    return _Frame(
      w: widget.w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(2)),
          const SizedBox(height: 8),
          _Sec('Used', n: '${_pile.length} / ${widget.cap}'),
        ]),
        Container(
          height: _pile.isEmpty ? 56 : rows * 28.0 + 8,
          padding: const EdgeInsets.symmetric(vertical: 4),
          decoration: const BoxDecoration(color: N.g07, border: Border.symmetric(horizontal: BorderSide(color: N.g20))),
          child: _pile.isEmpty
              ? Padding(padding: const EdgeInsets.symmetric(horizontal: _px), child: CustomPaint(painter: DashedBox(N.g38), child: Center(child: Text('まだ空です', style: T.label(N.g76)))))
              : _Fade(child: ListView.builder(
                  itemExtent: 28,
                  itemCount: _pile.length,
                  itemBuilder: (c, i) {
                    final (n, k) = _pile[i];
                    return _Hit(onDouble: () => _use(byName[n]!), builder: (c, h, _) => _FxRow(byName[n]!, hover: h, sel: _flash == n, whole: true, tags: [Text('$k×', style: _num())]));
                  },
                )),
        ),
        Padding(padding: const EdgeInsets.fromLTRB(_px, 8, _px, 0), child: _Sec('All', n: '${_plainFx.length}')),
        _Grow(
          child: _Fade(child: ListView.builder(
            shrinkWrap: _PanelH.fitOf(context),
            padding: const EdgeInsets.only(bottom: 12),
            itemExtent: 28,
            itemCount: _plainFx.length,
            itemBuilder: (c, i) {
              final f = _plainFx[i];
              return _Hit(onDouble: () => _use(f), builder: (c, h, _) => _FxRow(f, hover: h, whole: true));
            },
          )),
        ),
        _Foot(msg: _msg, hints: const [('dbl', 'use')]),
      ]),
    );
  }
}

// ================================================================ B11 own presets on the shelf

class _NameEdit extends StatefulWidget {
  const _NameEdit({required this.initial, required this.onDone, this.autofocus = true});
  final String initial;
  final ValueChanged<String> onDone;
  final bool autofocus;
  @override
  State<_NameEdit> createState() => _NameEditState();
}

class _NameEditState extends State<_NameEdit> {
  late final _c = TextEditingController(text: widget.initial);
  final _n = FocusNode();
  bool _done = false;
  @override
  void initState() {
    super.initState();
    if (!widget.autofocus) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      _n.requestFocus();
      _c.selection = TextSelection(baseOffset: 0, extentOffset: _c.text.length);
    });
  }

  @override
  void dispose() {
    _c.dispose();
    _n.dispose();
    super.dispose();
  }

  void _finish(String s) {
    if (_done) return;
    _done = true;
    widget.onDone(s.trim().isEmpty ? widget.initial : s.trim());
  }

  @override
  Widget build(BuildContext context) => Focus(
        onKeyEvent: (node, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
            _finish(widget.initial);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: Container(
          height: 24,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          alignment: Alignment.centerLeft,
          decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(4), border: Border.all(color: _accent)),
          child: EditableText(controller: _c, focusNode: _n, style: T.name(N.g95), cursorColor: _accent, backgroundCursorColor: N.g26, selectionColor: _accent.withValues(alpha: .35), maxLines: 1, onSubmitted: _finish),
        ),
      );
}

class _Mine {
  _Mine(this.name, this.kind, this.scope, {this.naming = false});
  String name;
  final String kind;
  String scope;
  bool naming;
}

List<_Mine> _seedMine() => [
      _Mine('Soft Glow + Levels (client A)', 'Effects', 'This project'),
      _Mine('Overshoot 少し強め', 'Curve', 'Everywhere'),
      _Mine('夜の青 Night Blue', 'Colour', 'Everywhere'),
      _Mine('Hero Title Stack', 'Layer', 'This project'),
      _Mine('Handheld Shake 弱', 'Effects', 'Everywhere'),
      _Mine('Bounce 0.8 / 重め', 'Curve', 'Everywhere'),
    ];

/// One Mine row, 28 px: name, the kind tag only when it differs from the tab (the Effects tab never says "Effects"), and the scope word only when it is not "Everywhere".
Widget _mineRow(_Mine m, {bool hover = false, bool sel = false, bool kind = true, Widget? trailing}) => _Line(
      height: 28,
      hover: hover,
      sel: sel,
      child: Row(children: [
        Expanded(child: _Mid(m.name, T.name(sel ? N.g95 : N.g91))),
        if (kind && m.kind != 'Effects') Padding(padding: const EdgeInsets.only(left: 6), child: _Tag(m.kind)),
        if (trailing != null) trailing else if (m.scope == 'This project') Padding(padding: const EdgeInsets.only(left: 6), child: Text('Project', style: T.label(N.g76))),
      ]),
    );

/// A recessed well for a short list: g07, a hairline above and below, rows full width so their text starts at the one inset.
Widget _well({required Widget child, Color edge = N.g20}) => DecoratedBox(decoration: BoxDecoration(color: N.g07, border: Border.symmetric(horizontal: BorderSide(color: edge))), child: child);

/// B11-a: carry a thing from the Inspector onto "Mine"; it is saved, the original stays, and a name box opens in place.
class _SaveToShelf extends StatefulWidget {
  const _SaveToShelf({required this.w, this.empty = false});
  final double w;
  final bool empty;
  @override
  State<_SaveToShelf> createState() => _SaveToShelfState();
}

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

class _LibraryState extends State<_Library> {
  late int _view = widget.view;
  late String _q = widget.query;
  double _size = 84;
  int _sort = 1, _sel = -1;
  String? _note;
  List<Asset>? _all;
  List<Asset> _cache = const [];
  String _key = '';

  List<Asset> get _items {
    final all = _all ??= _gen(widget.n);
    final k = '$_q|$_sort|${widget.n}';
    if (k != _key) {
      _key = k;
      final q = _q.toLowerCase();
      final l = [for (final a in all) if (q.isEmpty || a.name.toLowerCase().contains(q)) a];
      if (_sort == 0) l.sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
      if (_sort == 1) l.sort((a, b) => b.seed.compareTo(a.seed));
      if (_sort == 2) l.sort((a, b) => a.kind.index != b.kind.index ? a.kind.index.compareTo(b.kind.index) : a.name.compareTo(b.name));
      _cache = l;
    }
    return _cache;
  }

  @override
  Widget build(BuildContext context) {
    final w = widget.w, fit = _PanelH.fitOf(context);
    final items = widget.state == 0 ? _items : const <Asset>[];
    const gutter = EdgeInsets.symmetric(horizontal: _px);
    Widget body;
    if (widget.state == 1) {
      body = Padding(
        padding: gutter,
        child: LayoutBuilder(builder: (c, box) {
          final cols = math.max(2, ((box.maxWidth + 8) / (_size + 8)).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols;
          return _Pulse(child: SingleChildScrollView(physics: const NeverScrollableScrollPhysics(), child: Wrap(spacing: 8, runSpacing: 10, children: [for (var i = 0; i < cols * 5; i++) _Skel(tw)])));
        }),
      );
    } else if (widget.state == 2) {
      body = SingleChildScrollView(child: _empty('このフォルダを読めません', 'Permission denied\n/Volumes/RAID_A/Footage/2026/Day3_Shoot', tone: Role.error, action: Row(mainAxisSize: MainAxisSize.min, children: [_Btn('Retry', onTap: () => setState(() => _note = 'Still unreadable')), const SizedBox(width: 6), _Btn('Locate…', onTap: () => setState(() => _note = 'Folder picker'))])));
    } else if (items.isEmpty) {
      body = SingleChildScrollView(child: _empty('"$_q" に一致する素材はありません', '', tone: Role.warning, action: _Btn('Clear search', onTap: () => setState(() => _q = ''))));
    } else if (_view == 0) {
      body = Padding(
        padding: gutter,
        child: LayoutBuilder(builder: (c, box) {
          final cols = math.max(2, ((box.maxWidth + 8) / (_size + 8)).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols, names = tw >= 70;
          return GridView.builder(
            shrinkWrap: fit,
            padding: const EdgeInsets.only(bottom: 10),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: 10, crossAxisSpacing: 8, mainAxisExtent: tw * .62 + (names ? 33 : 0)),
            itemCount: items.length,
            itemBuilder: (c, i) {
              final a = items[i];
              return _Hit(onTap: () => setState(() => _sel = a.seed), builder: (c, h, _) => _AssetCell(a, tw, sel: _sel == a.seed, hover: h, names: names));
            },
          );
        }),
      );
    } else {
      body = ListView.builder(
        shrinkWrap: fit,
        padding: const EdgeInsets.only(bottom: 4),
        itemExtent: 28,
        itemCount: items.length,
        itemBuilder: (c, i) {
          final a = items[i];
          return _Hit(onTap: () => setState(() => _sel = a.seed), builder: (c, h, _) => _AssetRow(a, sel: _sel == a.seed, hover: h));
        },
      );
    }
    final selName = _sel >= 0 && _all != null ? _all![_sel].name : null;
    final count = widget.state == 1 ? 'reading…' : (widget.state == 2 ? 'offline' : (items.length == widget.n ? _fmt(widget.n) : '${_fmt(items.length)} of ${_fmt(widget.n)}'));
    return _Frame(
      w: w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(3)),
          const SizedBox(height: 8),
          TextBox(text: _q, hint: 'Search items', look: Look.quiet, clearable: true, leading: const nav.Glyph(nav.G.search, color: N.g76), onChanged: (s) => setState(() => _q = s)),
          const SizedBox(height: 8),
          Row(children: [
            Segmented(items: const ['Grid', 'List'], index: _view, onChanged: (i) => setState(() => _view = i)),
            const Spacer(),
            if (_view == 0) ...[Text('size', style: T.label(N.g76)), const SizedBox(width: 4), MiniSlider(value: _size, min: 44, max: 140, width: w < 280 ? 64 : 92, onChanged: (v) => setState(() => _size = v))],
          ]),
          const SizedBox(height: 6),
          Row(children: [for (final (i, s) in ['Name', 'Date', 'Type'].indexed) ...[_Chip(s, on: _sort == i, onTap: () => setState(() => _sort = i)), const SizedBox(width: 4)]]),
          const SizedBox(height: 4),
          _Sec('Items', n: count),
        ]),
        _Grow(child: _fadeFoot(body, on: widget.state == 0 && items.length > 24)),
        _Foot(msg: widget.state == 2 ? _note : selName),
      ]),
    );
  }
}

/// B12-b: "find similar" turns the list into a ranking by a made-up 8-number description; the anchor stays first.
double _dist(int a, int b) {
  List<double> v(int i) {
    final k = _bases[i % _bases.length];
    final r = math.Random(i * 7919 + 13);
    return [for (var d = 0; d < 8; d++) ((k.$3.index * 3 + k.$4.index + d * 2) % 7) / 7 + (r.nextDouble() - .5) * .35];
  }

  final x = v(a), y = v(b);
  var s = 0.0;
  for (var d = 0; d < 8; d++) {
    s += (x[d] - y[d]) * (x[d] - y[d]);
  }
  return math.sqrt(s);
}

/// The removable filter chip "Similar to NAME": g20, hover g26, an accent line, and the close glyph. [w] is the width it may take.
class _SimChip extends StatelessWidget {
  const _SimChip(this.name, this.w, {this.hover = false});
  final String name;
  final double w;
  final bool hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        height: 24,
        padding: const EdgeInsets.only(left: 8, right: 6),
        decoration: BoxDecoration(color: hover ? N.g26 : N.g20, borderRadius: BorderRadius.circular(4), border: Border.all(color: _accentLine)),
        child: Row(mainAxisSize: MainAxisSize.min, children: [Text('Similar to', style: T.label(N.g76)), const SizedBox(width: 5), ConstrainedBox(constraints: BoxConstraints(maxWidth: math.max(40.0, w - 84)), child: _Mid(name, T.name(N.g95))), const SizedBox(width: 6), const nav.Glyph(nav.G.close, color: N.g76, size: 10)]),
      );
}

/// The context menu of a tile: Find similar is live, the rest are disabled in this draft (g63 word, arrow cursor, no hover).
class _CtxMenu extends StatelessWidget {
  const _CtxMenu({required this.onFind});
  final VoidCallback onFind;
  @override
  Widget build(BuildContext context) => Container(
        width: 160,
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(6), border: Border.all(color: N.g26), boxShadow: [BoxShadow(color: N.g00.withValues(alpha: .5), blurRadius: 20, offset: const Offset(0, 8))]),
        child: Column(mainAxisSize: MainAxisSize.min, children: [
          for (final (label, key, act) in <(String, String, VoidCallback?)>[('Find similar', 'S', onFind), ('Reveal in Finder', '', null), ('Rename…', '↵', null), ('Remove from project', '⌫', null)])
            _Hit(onTap: act, cursor: act == null ? SystemMouseCursors.basic : SystemMouseCursors.click, builder: (c, h, _) => Container(height: 26, padding: const EdgeInsets.symmetric(horizontal: 8), decoration: BoxDecoration(color: h && act != null ? N.g26 : _clear, borderRadius: BorderRadius.circular(4)), child: Row(children: [Expanded(child: _t(label, T.name(act == null ? N.g63 : N.g95))), if (key.isNotEmpty) Text(key, style: _num())]))),
        ]),
      );
}

class _Similar extends StatefulWidget {
  const _Similar({required this.w, this.start = -1});
  final double w;
  final int start; // asset index the panel starts "similar to"
  @override
  State<_Similar> createState() => _SimilarState();
}

class _SimilarState extends State<_Similar> {
  final _all = _gen(240);
  late int _sel = widget.start, _anchor = widget.start;
  Offset _ptr = Offset.zero;
  int _menu = -1;

  void _similar(int i) => setState(() {
        _anchor = i;
        _sel = i;
        _menu = -1;
      });

  bool _key(KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.keyS && _sel >= 0) {
      _similar(_sel);
      return true;
    }
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
      setState(() {
        _menu = -1;
        _anchor = -1;
      });
      return true;
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    final items = _anchor < 0 ? _all : ([..._all]..sort((a, b) => _dist(_anchor, a.seed).compareTo(_dist(_anchor, b.seed))));
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: Listener(
          onPointerDown: (e) => _ptr = e.localPosition,
          child: Stack(children: [
            _Col([
              _Head([
                const IgnorePointer(child: _Tabs(3)),
                const SizedBox(height: 8),
                if (_anchor >= 0) ...[
                  Align(alignment: Alignment.centerLeft, child: _Hit(onTap: () => setState(() => _anchor = -1), builder: (c, h, _) => _SimChip(_all[_anchor].name, widget.w - 2 * _px, hover: h))),
                  const SizedBox(height: 6),
                ],
                _Sec(_anchor < 0 ? 'All' : 'Closest first', n: _anchor < 0 ? '${items.length}' : null),
              ]),
              _Grow(
                child: _Fade(child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: _px),
                  child: LayoutBuilder(builder: (c, box) {
                    final cols = math.max(2, ((box.maxWidth + 8) / 96).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols;
                    return GridView.builder(
                      shrinkWrap: _PanelH.fitOf(context),
                      padding: const EdgeInsets.only(bottom: 16),
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: 10, crossAxisSpacing: 8, mainAxisExtent: tw * .62 + 33),
                      itemCount: items.length,
                      itemBuilder: (c, i) {
                        final a = items[i];
                        final pct = _anchor < 0 ? null : (a.seed == _anchor ? 'source' : '${(100 - _dist(_anchor, a.seed) * 38).clamp(3, 99).round()}%');
                        return _Hit(
                          onTap: () => setState(() {
                            _sel = a.seed;
                            _menu = -1;
                          }),
                          onSecondary: () => setState(() {
                            _sel = a.seed;
                            _menu = a.seed;
                          }),
                          builder: (c, h, _) => _AssetCell(a, tw, sel: _sel == a.seed, hover: h, badge: pct),
                        );
                      },
                    );
                  }),
                )),
              ),
              const _Foot(hints: [('right-click', 'menu'), ('S', 'find similar'), ('Esc', 'clear')]),
            ]),
            if (_menu >= 0)
              Positioned(
                left: _ptr.dx.clamp(8.0, widget.w - 168),
                top: _ptr.dy.clamp(8.0, 560),
                child: _CtxMenu(onFind: () => _similar(_menu)),
              ),
          ]),
        ),
      ),
    );
  }
}

/// A demo histogram for the Parts sheet.
const _demoHist = [2, 4, 7, 12, 9, 5, 3, 2, 1, 1, 2, 4, 8, 14, 20, 16, 9, 5, 3, 2, 1, 1, 0, 1];

/// The band of B12-c: a histogram of the library on one axis, the kept range over it, and labels where their values sit (the duration axis is logarithmic).
class _Band extends StatelessWidget {
  const _Band({required this.hist, required this.axis, required this.bw, this.r});
  final List<int> hist;
  final int axis;
  final double bw;
  final (double, double)? r;
  @override
  Widget build(BuildContext context) {
    final peak = hist.reduce(math.max), bins = hist.length, rr = r;
    return SizedBox(
      width: bw,
      height: 112,
      child: Stack(children: [
        Positioned.fill(child: DecoratedBox(decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(5), border: Border.all(color: N.g20)))),
        Positioned(left: 0, right: 0, top: 6, bottom: 22, child: Row(crossAxisAlignment: CrossAxisAlignment.end, children: [
          for (var i = 0; i < bins; i++)
            Expanded(child: Padding(padding: const EdgeInsets.symmetric(horizontal: .5), child: FractionallySizedBox(heightFactor: math.max(.03, hist[i] / peak), child: DecoratedBox(decoration: BoxDecoration(color: rr == null || ((i + .5) / bins >= rr.$1 && (i + .5) / bins <= rr.$2) ? N.g91 : N.g38, borderRadius: BorderRadius.circular(1)))))),
        ])),
        if (rr != null) Positioned(left: bw * rr.$1, width: math.max(2, bw * (rr.$2 - rr.$1)), top: 0, bottom: 0, child: DecoratedBox(decoration: BoxDecoration(color: _accent.withValues(alpha: .12), border: Border.symmetric(vertical: BorderSide(color: _accent))))),
        if (rr != null) for (final x in [rr.$1, rr.$2]) Positioned(left: (bw * x - 3).clamp(0.0, bw - 5), top: 52, width: 5, height: 8, child: const CustomPaint(painter: _GripP(N.g95))),
        Positioned(left: 0, right: 0, bottom: 5, height: 12, child: _labels()),
      ]),
    );
  }

  Widget _labels() {
    if (axis == 0) {
      return Padding(padding: const EdgeInsets.symmetric(horizontal: 4), child: DecoratedBox(decoration: BoxDecoration(borderRadius: BorderRadius.circular(2), gradient: LinearGradient(colors: [for (var i = 0; i <= 12; i++) HSVColor.fromAHSV(1, i * 30.0, .5, .72).toColor()]))));
    }
    final labels = axis == 1 ? const [('still', .04), ('10 s', .40), ('1 min', .686), ('5 min', .953)] : const [('Image', .1), ('Video', .3), ('Audio', .5), ('3D', .7), ('360°', .9)];
    return Stack(children: [for (final (l, f) in labels) Align(alignment: Alignment(f * 2 - 1, 0), child: Text(l, style: T.label(N.g76)))]);
  }
}

/// B12-c: one axis, one band. Drag across the band to keep a range.
class _AxisStrip extends StatefulWidget {
  const _AxisStrip({required this.w, this.axis = 0});
  final double w;
  final int axis;
  @override
  State<_AxisStrip> createState() => _AxisStripState();
}

class _AxisStripState extends State<_AxisStrip> {
  static const _bins = 24;
  final _all = _gen(400);
  late int _axis = widget.axis;
  double? _a, _b;

  /// Position of an asset on the axis, 0..1.
  double _pos(Asset a) {
    switch (_axis) {
      case 0:
        final k = a.kind.index * .19, r = math.Random(a.seed * 31 + 5).nextDouble();
        return (k + (r - .5) * .5 + (a.style.index * .07)) % 1;
      case 1:
        return a.dur == 0 ? 0 : (math.log(a.dur + 1) / math.log(400)).clamp(.04, 1.0);
      default:
        return (a.kind.index + .5) / 5;
    }
  }

  List<int> _hist() {
    final h = List.filled(_bins, 0);
    for (final a in _all) {
      h[(_pos(a) * _bins).floor().clamp(0, _bins - 1)]++;
    }
    return h;
  }

  (double, double)? get _range => _a == null || _b == null ? null : (math.min(_a!, _b!), math.max(_a!, _b!));

  @override
  Widget build(BuildContext context) {
    final hist = _hist();
    final r = _range;
    final items = r == null ? _all : [for (final a in _all) if (_pos(a) >= r.$1 && _pos(a) <= r.$2) a];
    return _Frame(
      w: widget.w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(3)),
          const SizedBox(height: 8),
          Segmented(items: const ['Colour', 'Duration', 'Type'], index: _axis, expand: true, onChanged: (i) => setState(() {
            _axis = i;
            _a = _b = null;
          })),
          const SizedBox(height: 8),
          _Sec('Range', trailing: r == null ? null : _Chip('Clear range', onTap: () => setState(() => _a = _b = null))),
          LayoutBuilder(builder: (c, box) {
            final bw = box.maxWidth;
            return MouseRegion(cursor: SystemMouseCursors.resizeLeftRight, child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onPanStart: (d) => setState(() => _a = _b = (d.localPosition.dx / bw).clamp(0.0, 1.0)),
              onPanUpdate: (d) => setState(() => _b = (d.localPosition.dx / bw).clamp(0.0, 1.0)),
              onTapDown: (d) => setState(() => _a = _b = null),
              child: _Band(hist: hist, axis: _axis, bw: bw, r: r),
            ));
          }),
          const SizedBox(height: 8),
          _Sec(r == null ? 'All' : 'In range', n: r == null ? '${items.length}' : '${items.length} of ${_all.length}'),
        ]),
        _Grow(
          child: _Fade(child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: _px),
            child: items.isEmpty
                ? SingleChildScrollView(child: _empty('この範囲に素材はありません', ''))
                : LayoutBuilder(builder: (c, box) {
                    final cols = math.max(2, ((box.maxWidth + 8) / 92).floor()), tw = (box.maxWidth - 8 * (cols - 1)) / cols;
                    return GridView.builder(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.only(bottom: 16), gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(crossAxisCount: cols, mainAxisSpacing: 10, crossAxisSpacing: 8, mainAxisExtent: tw * .62 + 33), itemCount: items.length, itemBuilder: (c, i) => _AssetCell(items[i], tw));
                  }),
          )),
        ),
        const _Foot(hints: [('drag', 'keep a range'), ('click', 'clear')]),
      ]),
    );
  }
}

/// B12-d: a folder on disk is a live source. A file that disappears stays as a missing row.
class _DiskFile {
  _DiskFile(this.name, {this.added});
  final String name;
  bool onDisk = true;
  DateTime? added;
}

/// A file row of the live folder. `gone` (missing on disk, or the whole drive offline) is shown by a flat thumb and g63 name, never by opacity.
class _LiveRow extends StatelessWidget {
  const _LiveRow(this.name, this.asset, {this.sel = false, this.hover = false, this.fresh = false, this.onDisk = true, this.offline = false, this.onLocate});
  final String name;
  final Asset asset;
  final bool sel, hover, fresh, onDisk, offline;
  final VoidCallback? onLocate;
  @override
  Widget build(BuildContext context) {
    final gone = !onDisk || offline;
    return _Line(
      height: 28,
      sel: sel,
      hover: hover || fresh,
      child: Row(children: [
        SizedBox(width: 24, height: 18, child: ClipRRect(borderRadius: BorderRadius.circular(3), child: gone ? const ColoredBox(color: N.g20) : _Art(asset))),
        const SizedBox(width: 8),
        Expanded(child: _Mid(name, T.name(gone ? N.g63 : N.g91))),
        if (fresh) const _Tag('NEW', hi: true),
        if (!onDisk && !offline) ...[const _Tag('Missing', warn: true), const SizedBox(width: 4), _Btn('Locate…', onTap: onLocate ?? _noop)],
      ]),
    );
  }
}

class _LiveFolder extends StatefulWidget {
  const _LiveFolder({required this.w});
  final double w;
  @override
  State<_LiveFolder> createState() => _LiveFolderState();
}

class _LiveFolderState extends State<_LiveFolder> {
  final _files = <_DiskFile>[
    for (final n in ['A001_C003_0612_R2X5.mov', 'A001_C004_0612_R2X5.mov', 'A001_C005_0612_R2X5.mov', 'メイキング_001.mov', 'メイキング_002.mov', 'Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', 'drone_north_ridge.mp4', 'drone_north_ridge_b.mp4', 'BTS_stills_0412.jpg', 'BTS_stills_0413.jpg', 'ambience_wind.wav', 'slate_scene12.png']) _DiskFile(n),
  ];
  bool _offline = false;
  int _n = 0, _sel = -1;
  Timer? _tick1;

  @override
  void initState() {
    super.initState();
    _tick1 = Timer.periodic(const Duration(seconds: 1), (_) {
      if (mounted) setState(() {});
    });
  }

  @override
  void dispose() {
    _tick1?.cancel();
    super.dispose();
  }

  bool _fresh(_DiskFile f) => f.added != null && DateTime.now().difference(f.added!) < const Duration(seconds: 4);

  void _add() => setState(() {
        _n++;
        _files.insert(0, _DiskFile('new_clip_${_n.toString().padLeft(3, '0')}.mov', added: DateTime.now()));
      });

  void _remove() => setState(() {
        final f = _files.where((f) => f.onDisk && !_fresh(f)).toList();
        if (f.isNotEmpty) f[(_n * 5 + 3) % f.length].onDisk = false;
        _n++;
      });

  @override
  Widget build(BuildContext context) {
    final missing = _files.where((f) => !f.onDisk).length, shown = _files.length;
    final frame = _Frame(
      w: widget.w,
      child: _Col([
        _Head([
          const IgnorePointer(child: _Tabs(3)),
          const SizedBox(height: 8),
          _Sec('Places'),
          Container(
            padding: const EdgeInsets.all(_px),
            decoration: BoxDecoration(color: N.g13, borderRadius: BorderRadius.circular(6), border: Border.all(color: _offline ? N.g38 : N.g20)),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              Row(children: [
                Expanded(child: _t('Footage / Day3_Shoot', T.title())),
                _Tag(_offline ? 'OFFLINE' : 'LIVE', hi: !_offline),
              ]),
              const SizedBox(height: 4),
              _t('/Volumes/RAID_A/Footage/2026/Day3_Shoot', _num()),
              if (_offline) ...[
                const SizedBox(height: 6),
                Text('見つかりません · $shown 件を保持', style: T.label(N.g91)),
                const SizedBox(height: 6),
                Row(children: [_Btn('Locate folder…', onTap: () => setState(() => _offline = false)), const SizedBox(width: 6), _Btn('Retry', onTap: () => setState(() => _offline = false))]),
              ],
            ]),
          ),
          const SizedBox(height: 8),
          _Sec('Files', n: missing > 0 ? '$shown · $missing missing' : '$shown'),
        ], bottom: 0),
        _Grow(
          child: _Fade(child: ListView.builder(
            shrinkWrap: _PanelH.fitOf(context),
            padding: const EdgeInsets.only(bottom: 12),
            itemExtent: 28,
            itemCount: _files.length,
            itemBuilder: (c, i) {
              final f = _files[i];
              final a = Asset(f.name, f.name.endsWith('.jpg') || f.name.endsWith('.png') ? Kind.image : (f.name.endsWith('.wav') ? Kind.audio : Kind.video), i + 3);
              return _Hit(
                onTap: () => setState(() => _sel = i),
                builder: (c, h, _) => _LiveRow(f.name, a, sel: _sel == i, hover: h, fresh: _fresh(f), onDisk: f.onDisk, offline: _offline, onLocate: () => setState(() => f.onDisk = true)),
              );
            },
          )),
        ),
      ]),
    );
    final side = _Side('Disk (outside the panel)', [
      _Btn('Drop a new file in', onTap: _offline ? null : _add),
      _Btn('Delete a file on disk', onTap: _offline ? null : _remove),
      _Btn(_offline ? 'Reconnect the drive' : 'Unplug the drive', onTap: () => setState(() => _offline = !_offline)),
    ]);
    return _withSide(frame, side);
  }
}

// ================================================================ B13 context filter: what to do with things that do not fit the selection

String _need(_Fx f) => f.isKey ? 'key' : (f.isText ? 'Text layer' : 'layer');

/// A layer card on the stand-in Stage (B13-c): `ok` = this layer fits what was pressed.
class _PickCard extends StatelessWidget {
  const _PickCard(this.name, this.kind, {required this.ok, this.hover = false});
  final String name, kind;
  final bool ok, hover;
  @override
  Widget build(BuildContext context) => AnimatedContainer(
        duration: Mo.dur,
        curve: Mo.ease,
        height: 44,
        decoration: BoxDecoration(color: hover && ok ? N.g20 : N.g13, borderRadius: BorderRadius.circular(5), border: ok ? Border.all(color: _accent) : null),
        child: CustomPaint(
          painter: ok ? null : DashedBox(N.g38),
          child: Padding(padding: const EdgeInsets.all(8), child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.center, children: [_t(name, T.name(ok ? N.g95 : N.g63)), const SizedBox(height: 4), _t(kind, T.label(N.g76))])),
        ),
      );
}

class _Context extends StatefulWidget {
  const _Context({required this.w, required this.mode, this.sel = _Sel.text});
  final double w;
  final int mode; // 0 misfits are not shown, 1 misfits stay, dim, with a reason word, 2 press a misfit then pick a layer
  final _Sel sel;
  @override
  State<_Context> createState() => _ContextState();
}

class _ContextState extends State<_Context> {
  late _Sel _s = widget.sel;
  bool _all = false;
  _Fx? _pending;
  String? _msg;

  @override
  void didUpdateWidget(_Context o) {
    super.didUpdateWidget(o);
    if (o.sel != widget.sel) {
      _s = widget.sel;
      _pending = null;
    }
  }

  void _apply(_Fx f, String where) => setState(() {
        _pending = null;
        _msg = 'Applied ${f.en} to $where';
      });

  void _press(_Fx f) {
    final ok = _fits(f, _s);
    if (ok && _s != _Sel.none) {
      _apply(f, _selName(_s));
      return;
    }
    if (_s == _Sel.none) {
      setState(() => _msg = 'Select a target');
      return;
    }
    switch (widget.mode) {
      case 1:
        setState(() => _msg = '${f.en}: ${_why(f, _s)}');
      case 2:
        setState(() {
          _pending = f;
          _msg = 'Pick a ${_need(f)}';
        });
      default:
        break;
    }
  }

  bool _key(KeyEvent e) {
    if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape && _pending != null) {
      setState(() {
        _pending = null;
        _msg = 'Cancelled';
      });
      return true;
    }
    return false;
  }

  Widget _stage() {
    final f = _pending!;
    const layers = [('タイトル', 'Text', _Sel.text), ('Subtitle', 'Text', _Sel.text), ('Logo Mark', 'Shape', _Sel.shape), ('BG Gradient', 'Solid', _Sel.shape)];
    return Container(
      margin: const EdgeInsets.only(top: 8),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(color: N.g07, borderRadius: BorderRadius.circular(6), border: Border.all(color: _accentLine)),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Row(children: [Expanded(child: _t('Pick a ${_need(f)} for ${f.en}', T.name(N.g95))), const _Kbd('Esc')]),
        const SizedBox(height: 6),
        Row(children: [
          for (final l in layers)
            Expanded(
              child: Padding(
                padding: const EdgeInsets.only(right: 4),
                child: Builder(builder: (c) {
                  final ok = !f.isKey && (f.isText ? l.$3 == _Sel.text : true);
                  final card = _Hit(
                    onTap: () {
                      if (ok) {
                        _apply(f, '${l.$1} (${l.$2})');
                      } else {
                        setState(() => _msg = '${l.$1}: ${l.$2}');
                      }
                    },
                    builder: (c, h, _) => _PickCard(l.$1, l.$2, ok: ok, hover: h),
                  );
                  return ok ? _Pulse(child: card) : card;
                }),
              ),
            ),
        ]),
        const SizedBox(height: 4),
        _standIn('Stage stand-in'),
      ]),
    );
  }

  @override
  Widget build(BuildContext context) {
    final m = widget.mode;
    final fitting = _fxs.where((f) => _fits(f, _s)).toList();
    final hidden = _fxs.length - fitting.length;
    final showFits = m == 0 && !_all && _s != _Sel.none;
    final list = showFits ? fitting : _fxs;
    final cats = [for (final c in const [7, 8, 0, 1, 2, 3, 4, 5, 6]) if (list.any((f) => f.cat == c)) c];
    final none = _s == _Sel.none;
    return _Frame(
      w: widget.w,
      child: _Keys(
        onKey: _key,
        child: _Col([
          _Head([
            const IgnorePointer(child: _Tabs(2)),
            const SizedBox(height: 8),
            _TargetLine(_s),
            if (!none && m == 0 && hidden > 0) ...[
              const SizedBox(height: 8),
              SizedBox(
                height: 22,
                child: Row(children: [Text('$hidden hidden', style: T.label(N.g76)), const Spacer(), _Chip(_all ? 'Hide misfits' : 'Show all', on: _all, onTap: () => setState(() => _all = !_all))]),
              ),
            ],
            if (_pending != null) _stage(),
            const SizedBox(height: 6),
          ]),
          _Grow(
            child: _fadeFoot(ListView(shrinkWrap: _PanelH.fitOf(context), padding: const EdgeInsets.fromLTRB(_px, 0, _px, 16), children: [
              for (final c in cats) ...[
                _Sec(_cats[c], n: '${list.where((f) => f.cat == c).length}'),
                _Grid(
                  n: list.where((f) => f.cat == c).length,
                  item: (i, w) {
                    final f = list.where((f) => f.cat == c).toList()[i];
                    final miss = !_fits(f, _s) && !none;
                    return _Hit(
                      cursor: miss && m == 1 ? SystemMouseCursors.basic : SystemMouseCursors.click,
                      onTap: () => _press(f),
                      builder: (cx, h, _) => _FxTile(f, w, hover: h, dim: miss && m == 1, reason: miss && m == 1 ? _why(f, _s) : null),
                    );
                  },
                ),
                const SizedBox(height: 10),
              ],
            ]), on: list.length > 12)),
          _Foot(msg: _msg, hints: switch (m) {
            0 => const [('click', 'apply')],
            1 => const [('click', 'apply to a tile that fits')],
            _ => const [('Esc', 'cancel')],
          }),
        ]),
      ),
    );
  }
}

// ================================================================ Parts @200%: the small parts of each option group, large, every state side by side

/// The parts are drawn by the very widgets the panels use (never a copy), laid out at their real size and then scaled 2x as vectors
/// (Transform.scale without a filter layer), so a 1 px line is a crisp 2 px line and a 4 px padding is a measurable 8 px.
const _zk = 2.0;

/// One labelled cell: a 10 px caption, then the part on its panel ground inside a 4 px margin, at 200 %. Every cell of a group is drawn on a plate of the same size ([sized]).
class _Cell extends StatelessWidget {
  const _Cell(this.cap, this.w, this.h, this.child, {this.ground = N.g10, this.keepH = false});
  final String cap;
  final double w, h;
  final Widget child;
  final Color ground;

  /// The plate keeps the height of its own content (cards: no empty band under the last bar); only the width is shared.
  final bool keepH;
  _Cell sized(double nw, double nh) => _Cell(cap, nw, keepH ? h : nh, child, ground: ground, keepH: keepH);
  @override
  Widget build(BuildContext context) {
    final lw = math.max(w, 56.0) + 8, lh = h + 8;
    return SizedBox(
      width: lw * _zk,
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        ConstrainedBox(constraints: const BoxConstraints(minHeight: 26), child: Padding(padding: const EdgeInsets.only(bottom: 2), child: Text(cap, softWrap: true, style: T.label(N.g76).copyWith(height: 1.25)))),
        DecoratedBox(
          position: DecorationPosition.foreground,
          decoration: BoxDecoration(border: Border.all(color: N.g20)),
          child: SizedBox(
            width: lw * _zk,
            height: lh * _zk,
            child: ClipRect(
              child: OverflowBox(
                alignment: Alignment.topLeft,
                minWidth: lw,
                maxWidth: lw,
                minHeight: lh,
                maxHeight: lh,
                child: Transform.scale(scale: _zk, alignment: Alignment.topLeft, child: ColoredBox(color: ground, child: Padding(padding: const EdgeInsets.all(4), child: Align(alignment: Alignment.topLeft, child: child)))),
              ),
            ),
          ),
        ),
      ]),
    );
  }
}

Widget _fixed(double w, Widget child) => SizedBox(width: w, child: child);

/// Every plate of a group the same size: the widest and the tallest cell set it, so a row of plates lines up.
List<Widget> _plates(List<Widget> cells) {
  final cs = cells.whereType<_Cell>().toList();
  if (cs.isEmpty) return cells;
  final w = cs.map((c) => c.w).reduce(math.max), h = cs.map((c) => c.h).reduce(math.max);
  return [for (final c in cells) c is _Cell ? c.sized(w, h) : c];
}

/// A Parts sheet: groups of cells, each group a titled wrap of equal plates. [absent] says which states do not exist for these parts (never silently left out).
WidgetbookUseCase _parts(String name, String line, List<(String, List<Widget>)> groups, {required String absent}) => WidgetbookUseCase(
      name: 'Parts @200% · $name',
      builder: (c) => ColoredBox(color: N.g07, child: _PartsSheet(key: ValueKey(name), name: name, line: line, absent: absent, groups: [for (final (t, cs) in groups) (t, _plates(cs))])),
    );

/// The sheet scrolls both ways with visible bars and by keyboard (PageUp / PageDown / Home / End / arrows); the JUMP TO bar goes to a group. A cell wider than the window is reached by the horizontal bar, never lost.
class _PartsSheet extends StatefulWidget {
  const _PartsSheet({super.key, required this.name, required this.line, required this.absent, required this.groups});
  final String name, line, absent;
  final List<(String, List<Widget>)> groups;
  @override
  State<_PartsSheet> createState() => _PartsSheetState();
}

class _PartsSheetState extends State<_PartsSheet> {
  final _v = ScrollController(), _h = ScrollController();
  late final List<GlobalKey> _keys = [for (final _ in widget.groups) GlobalKey()];
  @override
  void dispose() {
    _v.dispose();
    _h.dispose();
    super.dispose();
  }

  void _go(int i) {
    final c = _keys[i].currentContext;
    if (c != null) Scrollable.ensureVisible(c, duration: Mo.dur, curve: Mo.ease, alignment: 0, alignmentPolicy: ScrollPositionAlignmentPolicy.explicit);
  }

  KeyEventResult _key(FocusNode n, KeyEvent e) {
    if (!(e is KeyDownEvent || e is KeyRepeatEvent) || !_v.hasClients) return KeyEventResult.ignored;
    final vp = _v.position.viewportDimension, k = e.logicalKey;
    double? to;
    if (k == LogicalKeyboardKey.pageDown || k == LogicalKeyboardKey.space) to = _v.offset + vp * .9;
    if (k == LogicalKeyboardKey.pageUp) to = _v.offset - vp * .9;
    if (k == LogicalKeyboardKey.arrowDown) to = _v.offset + 80;
    if (k == LogicalKeyboardKey.arrowUp) to = _v.offset - 80;
    if (k == LogicalKeyboardKey.home) to = 0;
    if (k == LogicalKeyboardKey.end) to = _v.position.maxScrollExtent;
    if (to == null) return KeyEventResult.ignored;
    _v.jumpTo(to.clamp(0.0, _v.position.maxScrollExtent));
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) => Focus(
        autofocus: true,
        onKeyEvent: _key,
        child: LayoutBuilder(builder: (c, box) {
          // The sheet is as wide as the window, or as the widest plate if that is wider: then it scrolls sideways instead of being cut.
          final widest = [for (final g in widget.groups) for (final cell in g.$2.whereType<_Cell>()) (math.max(cell.w, 56.0) + 8) * _zk].fold<double>(0, math.max);
          final inner = math.max(box.maxWidth - 32 - 12, math.max(widest, 680.0));
          return RawScrollbar(
            controller: _v,
            thumbVisibility: true,
            thickness: 6,
            radius: const Radius.circular(3),
            thumbColor: N.g44,
            child: RawScrollbar(
              controller: _h,
              thumbVisibility: true,
              thickness: 6,
              radius: const Radius.circular(3),
              thumbColor: N.g44,
              notificationPredicate: (n) => n.depth == 1,
              scrollbarOrientation: ScrollbarOrientation.bottom,
              child: SingleChildScrollView(
                controller: _v,
                child: SingleChildScrollView(
                  controller: _h,
                  scrollDirection: Axis.horizontal,
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(16, 16, 28, 28),
                    child: SizedBox(
                      width: inner,
                      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Row(children: [Text('PARTS', style: T.name(N.g95)), const SizedBox(width: 8), Text('@200%', style: T.micro(N.g76)), const SizedBox(width: 8), Flexible(child: Text(widget.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.title()))]),
                        const SizedBox(height: 8),
                        ConstrainedBox(constraints: const BoxConstraints(maxWidth: 720), child: Text(widget.line, style: T.label(N.g76).copyWith(height: 1.35))),
                        const SizedBox(height: 4),
                        ConstrainedBox(constraints: const BoxConstraints(maxWidth: 720), child: Text('Not drawn, because they do not exist: ${widget.absent}', style: T.label(N.g76).copyWith(height: 1.35))),
                        const SizedBox(height: 10),
                        Wrap(spacing: 12, runSpacing: 6, crossAxisAlignment: WrapCrossAlignment.center, children: [
                          Text('JUMP TO', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                          for (var i = 0; i < widget.groups.length; i++) _Hit(onTap: () => _go(i), builder: (c, h, _) => Text(widget.groups[i].$1, maxLines: 1, style: T.label(h ? N.g95 : N.g76).copyWith(decoration: h ? TextDecoration.underline : TextDecoration.none, decorationColor: N.g95))),
                          Text('(PageDown, End and the arrows scroll)', style: T.label(N.g76)),
                        ]),
                        for (var i = 0; i < widget.groups.length; i++) ...[
                          const SizedBox(height: 24),
                          KeyedSubtree(key: _keys[i], child: Text(widget.groups[i].$1.toUpperCase(), style: T.micro(N.g76).copyWith(letterSpacing: .8))),
                          const SizedBox(height: 8),
                          Wrap(spacing: 12, runSpacing: 12, children: widget.groups[i].$2),
                        ],
                        const SizedBox(height: 24),
                        Text('END OF SHEET', style: T.micro(N.g76).copyWith(letterSpacing: .8)),
                      ]),
                    ),
                  ),
                ),
              ),
            ),
          );
        }),
      );
}

_Fx _fx(String en) => _fxs.firstWhere((f) => f.en == en);
Asset _asset(Kind k) => [..._mediaShelf, ...sampleAssets].firstWhere((a) => a.kind == k, orElse: () => _mediaShelf.first);

const _noTag = 'tag and key mark are labels, so they have no hover, pressed, focus or disabled look';

// ---- B9 swap
WidgetbookUseCase _partsSwapChips() => _parts('chips, tags, keys, buttons', 'B9-a small parts: the chip (22 px), the tag (16 px), the key mark and the 24 px button, each in every state it has.', [
      ('CHIP · filter, sort, action', [
        _Cell('rest', 96, 22, const _Chip('All', n: 63)),
        _Cell('hover', 96, 22, const _Chip('All', n: 63, hover: true)),
        _Cell('pressed', 96, 22, const _Chip('All', n: 63, pressed: true)),
        _Cell('keyboard focus', 96, 22, const _Chip('All', n: 63, focus: true)),
        _Cell('on (selected)', 96, 22, const _Chip('All', n: 63, on: true)),
        _Cell('on + hover', 96, 22, const _Chip('All', n: 63, on: true, hover: true)),
        _Cell('disabled', 96, 22, const _Chip('All', n: 63, enabled: false)),
        _Cell('long name, rest', 96, 22, const _Chip('Blur family', n: 7)),
        _Cell('long name, on', 96, 22, const _Chip('Blur family', n: 7, on: true)),
        _Cell('no count (sort), rest', 96, 22, const _Chip('Name')),
        _Cell('no count, on', 96, 22, const _Chip('Name', on: true)),
        _Cell('action chip', 96, 22, const _Chip('Show all')),
      ]),
      ('TAG · 16 px', [
        _Cell('plain', 120, 16, const _Tag('ON LAYER')),
        _Cell('plain, short', 120, 16, const _Tag('NOW')),
        _Cell('hi (accent edge)', 120, 16, const _Tag('REPLACE', hi: true)),
        _Cell('hi, long', 120, 16, const _Tag('PREVIEW · not written', hi: true)),
        _Cell('number', 120, 16, const _Tag('3')),
        _Cell('plain, long', 120, 16, const _Tag('Missing')),
      ]),
      ('KEY MARK · 16 px', [
        _Cell('letter', 40, 16, const _Kbd('Q')),
        _Cell('arrow', 40, 16, const _Kbd('↵')),
        _Cell('word', 40, 16, const _Kbd('Esc')),
        _Cell('chord', 40, 16, const _Kbd('⌘Z')),
      ]),
      ('BUTTON · 24 px', [
        _Cell('rest', 100, 24, const _Btn('Link', onTap: _noop)),
        _Cell('hover', 100, 24, const _Btn('Link', onTap: _noop, hover: true)),
        _Cell('pressed', 100, 24, const _Btn('Link', onTap: _noop, pressed: true)),
        _Cell('keyboard focus', 100, 24, const _Btn('Link', onTap: _noop, focus: true)),
        _Cell('disabled', 100, 24, const _Btn('Link')),
        _Cell('primary, rest', 100, 24, const _Btn('Keep', onTap: _noop, primary: true)),
        _Cell('primary, hover', 100, 24, const _Btn('Keep', onTap: _noop, primary: true, hover: true)),
        _Cell('primary, pressed', 100, 24, const _Btn('Keep', onTap: _noop, primary: true, pressed: true)),
        _Cell('button + key mark', 100, 24, const Row(mainAxisSize: MainAxisSize.min, children: [_Btn('Revert', onTap: _noop), SizedBox(width: 4), _Kbd('Esc')])),
      ]),
    ], absent: '$_noTag; error and empty states do not apply to them.');

WidgetbookUseCase _partsSwapRows() => _parts('rows, foot', 'B9-a list rows (28 px: gutter tick, thumb, English name + Japanese label, tags) and the foot (what just happened + key hints).', [
      ('EFFECT ROW · 28 px', [
        _Cell('rest', 280, 28, _fixed(280, _FxRow(_fxs[0]))),
        _Cell('hover', 280, 28, _fixed(280, _FxRow(_fxs[0], hover: true))),
        _Cell('selected', 280, 28, _fixed(280, _FxRow(_fxs[0], sel: true))),
        _Cell('selected + ON LAYER tag', 280, 28, _fixed(280, _FxRow(_fxs[1], sel: true, tags: const [_Tag('ON LAYER')]))),
        _Cell('hover + ON LAYER tag', 280, 28, _fixed(280, _FxRow(_fxs[1], hover: true, tags: const [_Tag('ON LAYER')]))),
        _Cell('selected + NOW tag (hi)', 280, 28, _fixed(280, _FxRow(_fxs[2], sel: true, tags: const [_Tag('NOW', hi: true)]))),
        _Cell('long JP + EN name (ellipsis)', 280, 28, _fixed(280, _FxRow(_fxs[31]))),
        _Cell('long name, narrow dock 232', 280, 28, _fixed(230, _FxRow(_fxs[31], sel: true))),
        _Cell('curve row (key effect)', 280, 28, _fixed(280, _FxRow(_fx('Ease Out Expo')))),
        _Cell('text effect', 280, 28, _fixed(280, _FxRow(_fx('Typewriter')))),
      ]),
      ('FOOT · what happened + key hints', [
        _Cell('nothing happened yet (takes no room)', 296, 8, _fixed(296, const _Foot(hints: [('Q', 'link / revert'), ('↑↓', 'next / prev')]))),
        _Cell('message, one line (hover lists the keys)', 296, 92, _fixed(296, const _Foot(msg: 'Linked · Effect 2', hints: [('↑↓', 'next / prev')]))),
        _Cell('message, two lines (then ellipsis)', 296, 92, _fixed(296, const _Foot(msg: 'Added Gaussian Blur to Layer 3 and applied the same Glow to the five other layers in this composition', hints: [('click', 'select')]))),
        _Cell('message only', 296, 92, _fixed(296, const _Foot(msg: 'Selected: dusk_ridge_001.jpg'))),
        _Cell('message + action', 296, 92, _fixed(296, _Foot(msg: 'Applied Glow to 5 layers', trailing: const _Btn('Undo', onTap: _noop)))),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring on rows (keys act on the panel), and disabled or error rows. The foot has one look.');

// ---- B9 cards, A/B
WidgetbookUseCase _partsCards() => _parts('cards, ghost, tray', 'B9-b/c/d small parts: the effect card in the drop panel and in the arrow-step panel, the carried ghost, the tray tile, the insertion line, the preview frame and the compare region.', [
      ('EFFECT CARD · drop panel', [
        _Cell('rest', 258, 84, _fixed(258, _CardFace(_Card(_fxs[0], .30, .55))), keepH: true),
        _Cell('a tile is over it: REPLACE', 258, 84, _fixed(258, _CardFace(_Card(_fxs[0], .30, .55), replace: true, kept: 1)), keepH: true),
        _Cell('long JP + EN name', 258, 84, _fixed(258, _CardFace(_Card(_fxs[31], .45, .2), number: 2)), keepH: true),
        _Cell('Alt held: add line under the card', 258, 92, _fixed(258, Column(mainAxisSize: MainAxisSize.min, children: [_CardFace(_Card(_fxs[8], .60, .35), number: 3), const _AddLine(true)])), keepH: true),
        _Cell('no Alt: line hidden', 258, 92, _fixed(258, Column(mainAxisSize: MainAxisSize.min, children: [_CardFace(_Card(_fxs[8], .60, .35), number: 3), const _AddLine(false)])), keepH: true),
      ]),
      ('EFFECT CARD · arrow-step panel', [
        _Cell('rest', 258, 92, _fixed(258, _StepFace(_fxs[0], (.30, .55))), keepH: true),
        _Cell('hover', 258, 92, _fixed(258, _StepFace(_fxs[0], (.30, .55), hover: true)), keepH: true),
        _Cell('selected (neighbours shown)', 258, 120, _fixed(258, _StepFace(_fxs[2], (.30, .55), sel: true)), keepH: true),
        _Cell('selected, neighbour hover', 258, 120, _fixed(258, _StepFace(_fxs[2], (.30, .55), sel: true, hoverUp: true)), keepH: true),
        _Cell('selected, first of its family', 258, 120, _fixed(258, _StepFace(_fxs[0], (.30, .55), sel: true)), keepH: true),
        _Cell('selected, last of its family', 258, 120, _fixed(258, _StepFace(_fxs[6], (.30, .55), sel: true)), keepH: true),
      ]),
      ('PARAMETER BAR · 20 px', [
        _Cell('30 %', 242, 20, _fixed(242, const _Param('Blurriness', .30))),
        _Cell('hot (target)', 242, 20, _fixed(242, const _Param('Blurriness', .30, hot: true))),
        _Cell('0 %', 242, 20, _fixed(242, const _Param('Amount', 0))),
        _Cell('100 %', 242, 20, _fixed(242, const _Param('Amount', 1))),
        _Cell('long label', 242, 20, _fixed(242, const _Param('Very long parameter name パラメータ名の長いもの', .5))),
      ]),
      ('TRAY TILE · GHOST', [
        _Cell('tray rest', 196, 26, _TrayChip(_fxs[0])),
        _Cell('tray hover', 196, 26, _TrayChip(_fxs[0], hover: true)),
        _Cell('tray picked up', 196, 26, _TrayChip(_fxs[0], on: true)),
        _Cell('ghost: replace', 196, 26, _Ghost('Gaussian Blur  = Replace', thumb: _Thumb(_fxs[0]))),
        _Cell('ghost: add', 196, 26, _Ghost('Glow  + Add', thumb: _Thumb(_fxs[24]))),
      ]),
      ('SECTION HEADER · 22 px', [
        _Cell('title + count + undo mark (shown only above 0)', 258, 22, _fixed(258, _Sec('Effects', n: '5', trailing: _undoMark(3)))),
        _Cell('title + preview tag', 258, 22, _fixed(258, const _Sec('Compare · Layer 3, Effect 2', trailing: _Tag('PREVIEW · not written', hi: true)))),
        _Cell('title only', 258, 22, _fixed(258, const _Sec('Compare · Layer 3, Effect 2'))),
        _Cell('long title (ellipsis)', 258, 22, _fixed(258, const _Sec('Candidates for the second effect on this layer', n: '63'))),
      ]),
      ('PREVIEW FRAME · compare region', [
        _Cell('no effect', 124, 78, const SizedBox(width: 124, height: 78, child: _Shot(null))),
        _Cell('blur family', 124, 78, SizedBox(width: 124, height: 78, child: _Shot(_fxs[0]))),
        _Cell('colour family', 124, 78, SizedBox(width: 124, height: 78, child: _Shot(_fxs[8]))),
        _Cell('other family', 124, 78, SizedBox(width: 124, height: 78, child: _Shot(_fxs[24]))),
        _Cell('closed: now beside an empty slot', 258, 78, _fixed(258, SizedBox(height: 52, child: Padding(padding: const EdgeInsets.symmetric(vertical: 4), child: Row(children: [Expanded(child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Shot(_fxs[0]))), const SizedBox(width: 8), Expanded(child: CustomPaint(painter: DashedBox(N.g38), child: const SizedBox.expand()))]))))),
      ]),
    ], absent: 'a pressed look on cards (a click selects at once), a focus ring on cards (keys act on the panel), disabled cards and error cards. The "end of family" button is the only disabled part.');

// ---- B8 drop on layer
WidgetbookUseCase _partsDrop() => _parts('layer rows, faces, drop zones', 'B8 small parts: the 23 px layer row (two row bands), the Replace | Add faces, the dashed new-layer zone and the carried asset row.', [
      ('LAYER ROW · 23 px', [
        _Cell('band A', 300, 23, _fixed(300, _LayerRow(_seedLayers()[0], 0))),
        _Cell('band B', 300, 23, _fixed(300, _LayerRow(_seedLayers()[1], 1))),
        _Cell('hover', 300, 23, _fixed(300, _LayerRow(_seedLayers()[1], 1, hover: true))),
        _Cell('selected', 300, 23, _fixed(300, _LayerRow(_seedLayers()[3], 3, sel: true))),
        _Cell('no keys', 300, 23, _fixed(300, _LayerRow(_seedLayers()[2], 2))),
        _Cell('long name (ellipsis)', 300, 23, _fixed(300, _LayerRow(_seedLayers()[11], 11))),
        _Cell('narrow dock 232, source hidden', 300, 23, _fixed(230, _LayerRow(_seedLayers()[11], 11, sel: true, showSrc: false))),
      ]),
      ('FACES · 48 px', [
        _Cell('neither lit', 240, 48, SizedBox(width: 240, height: 48, child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: const [Expanded(child: _Face('Replace', '絵だけ差し替え', false)), Expanded(child: _Face('Add', '新規 layer', false))]))),
        _Cell('Replace lit', 240, 48, SizedBox(width: 240, height: 48, child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: const [Expanded(child: _Face('Replace', '絵だけ差し替え', true)), Expanded(child: _Face('Add', '新規 layer', false))]))),
        _Cell('Add lit', 240, 48, SizedBox(width: 240, height: 48, child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: const [Expanded(child: _Face('Replace', '絵だけ差し替え', false)), Expanded(child: _Face('Add', '新規 layer', true))]))),
      ]),
      ('DROP ZONE · 38 px', [
        _Cell('idle', 296, 38, const SizedBox(width: 296, height: 38, child: _DropZone('new layer on top'))),
        _Cell('carrying, over it', 296, 38, const SizedBox(width: 296, height: 38, child: _DropZone('New layer', on: true))),
      ]),
      ('CARRIED ASSET ROW · 28 px', [
        _Cell('rest', 280, 28, _fixed(280, _AssetRow(_dropAssets[0]))),
        _Cell('hover', 280, 28, _fixed(280, _AssetRow(_dropAssets[0], hover: true))),
        _Cell('selected', 280, 28, _fixed(280, _AssetRow(_dropAssets[0], sel: true))),
        _Cell('JP name, video clock', 280, 28, _fixed(280, _AssetRow(_dropAssets[1]))),
        _Cell('image (kind badge word)', 280, 28, _fixed(280, _AssetRow(_dropAssets[3]))),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring (keys act on the panel), and disabled rows (a layer that refuses a drop says so on the row in B8-c, it is not a row state). Error does not apply.');

// ---- B7 click grammar
WidgetbookUseCase _partsTiles() => _parts('tabs, tiles, target line', 'B7 small parts: the tab strip (full and narrow), the effect tile and the media tile at the 92 px grid minimum and at 140 px, and the target line.', [
      ('TABS · 24 px', [
        _Cell('Effects selected', 296, 28, _fixed(296, const IgnorePointer(child: _Tabs(2)))),
        _Cell('Media selected', 296, 28, _fixed(296, const IgnorePointer(child: _Tabs(3)))),
        _Cell('narrow dock 232 (short names)', 296, 28, _fixed(232, const IgnorePointer(child: _Tabs(2)))),
      ]),
      ('TARGET LINE · 30 px', [
        _Cell('one layer', 296, 30, _fixed(296, const _TargetLine(_Sel.text))),
        _Cell('several layers', 296, 30, _fixed(296, const _TargetLine(_Sel.many))),
        _Cell('keys', 296, 30, _fixed(296, const _TargetLine(_Sel.key))),
        _Cell('nothing selected', 296, 30, _fixed(296, const _TargetLine(_Sel.none))),
      ]),
      ('EFFECT TILE · 92 px', [
        _Cell('rest', 92, 88, _FxTile(_fxs[0], 92)),
        _Cell('hover', 92, 88, _FxTile(_fxs[0], 92, hover: true)),
        _Cell('selected', 92, 88, _FxTile(_fxs[0], 92, sel: true)),
        _Cell('dim + reason', 92, 88, _FxTile(_fx('Typewriter'), 92, dim: true, reason: 'Text only')),
        _Cell('dim + reason (Key only)', 92, 88, _FxTile(_fx('Bounce'), 92, dim: true, reason: 'Key only')),
        _Cell('long EN + JP name', 92, 88, _FxTile(_fxs[31], 92)),
        _Cell('key tile (curve)', 92, 88, _FxTile(_fx('Elastic'), 92)),
      ]),
      ('MEDIA TILE · 92 px', [
        _Cell('image, rest', 92, 88, _MediaTile(_asset(Kind.image), 92)),
        _Cell('image, hover', 92, 88, _MediaTile(_asset(Kind.image), 92, hover: true)),
        _Cell('image, selected', 92, 88, _MediaTile(_asset(Kind.image), 92, sel: true)),
        _Cell('video: type badge + time pill', 92, 88, _MediaTile(_mediaShelf[7], 92)),
        _Cell('video, selected', 92, 88, _MediaTile(_mediaShelf[7], 92, sel: true)),
        _Cell('long EN name + 5:12', 92, 88, _MediaTile(_mediaShelf[8], 92)),
        _Cell('long JP name', 92, 88, _MediaTile(_mediaShelf[9], 92)),
        _Cell('audio', 92, 88, _MediaTile(_asset(Kind.audio), 92)),
      ]),
      ('TILES · 140 px', [
        _Cell('effect, rest', 140, 126, _FxTile(_fxs[0], 140)),
        _Cell('effect, selected', 140, 126, _FxTile(_fxs[0], 140, sel: true)),
        _Cell('media, rest', 140, 126, _MediaTile(_mediaShelf[7], 140)),
        _Cell('media, selected', 140, 126, _MediaTile(_mediaShelf[7], 140, sel: true)),
      ]),
    ], absent: 'a pressed look on tiles (a click selects at once or uses at once, by the case), a focus ring on tiles (keys act on the panel), and disabled tiles (a tile that does not fit is the dim tile of B13). The tab strip is a prop here and has its own sheet in the Inputs set.');

// ---- B10 favourites
WidgetbookUseCase _partsFav() => _parts('collections, tagged rows, recent', 'B10 small parts: the numbered collection button, effect rows that carry collection marks, a star, the Recent thumb and the empty-collection card.', [
      ('COLLECTION BUTTON · 24 px', [
        _Cell('rest', 92, 24, _fixed(92, const _CollChip(1, 'Titles', 12))),
        _Cell('hover', 92, 24, _fixed(92, const _CollChip(1, 'Titles', 12, hover: true))),
        _Cell('on (filter)', 92, 24, _fixed(92, const _CollChip(1, 'Titles', 12, on: true))),
        _Cell('empty (count 0)', 92, 24, _fixed(92, const _CollChip(9, 'Scratch', 0))),
        _Cell('long JP + EN name', 92, 24, _fixed(92, const _CollChip(4, 'Wipes 切替 長い名前', 7))),
        _Cell('3 columns (panel 280 and up)', 92, 24, _fixed(84, const _CollChip(6, 'Looks ルック', 128))),
      ]),
      ('ROWS WITH COLLECTION MARKS · 28 px', [
        _Cell('one number', 280, 28, _fixed(280, _FxRow(_fxs[24], tags: const [_Tag('1')]))),
        _Cell('three numbers + more', 280, 28, _fixed(280, _FxRow(_fxs[24], hover: true, tags: const [_Tag('1'), _Tag('4'), _Tag('7'), _Tag('…')]))),
        _Cell('selected, use count', 280, 28, _fixed(280, _FxRow(_fxs[24], sel: true, tags: [Text('12×', style: _num())]))),
        _Cell('long name + tags', 280, 28, _fixed(280, _FxRow(_fxs[31], tags: const [_Tag('2'), _Tag('5')]))),
        _Cell('star off / on', 280, 28, _fixed(280, _FxRow(_fxs[24], tags: [StarButton(on: false, size: 14), StarButton(on: true, size: 14)]))),
      ]),
      ('RECENT THUMB', [
        _Cell('recent', 66, 62, SizedBox(width: 66, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(height: 38, child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Thumb(_fxs[0]))), const SizedBox(height: 4), _t(_fxs[0].en, T.label(N.g76))]))),
        _Cell('long name (ellipsis)', 66, 62, SizedBox(width: 66, child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(height: 38, child: ClipRRect(borderRadius: BorderRadius.circular(4), child: _Thumb(_fxs[31]))), const SizedBox(height: 4), _t(_fxs[31].en, T.label(N.g76))]))),
      ]),
      ('SECTION HEADER · 22 px', [
        _Cell('section + action chip', 296, 22, _fixed(296, _Sec('Collections', trailing: const _Chip('Show all')))),
        _Cell('section + count', 296, 22, _fixed(296, const _Sec('Recent', n: '12'))),
      ]),
      ('EMPTY COLLECTION CARD', [
        _Cell('empty', 296, 124, _fixed(296, _empty('9 Scratch はまだ空です', ''))),
      ]),
    ], absent: 'a pressed look on the collection button (a press switches the filter on at once, which is the "on" look), a focus ring on rows and buttons (keys act on the panel), and disabled or error parts. An empty collection is drawn as the empty card.');

// ---- B11 own presets
WidgetbookUseCase _partsMine() => _parts('Mine rows, bars, zone, name box', 'B11 small parts: the Mine row for each kind, the history row, the name box, the parameter bar being dragged, the save zone and the empty-shelf card.', [
      ('MINE ROW · 28 px', [
        _Cell('rest (kind = the tab: no tag)', 280, 28, _fixed(280, _mineRow(_seedMine()[0]))),
        _Cell('hover', 280, 28, _fixed(280, _mineRow(_seedMine()[0], hover: true))),
        _Cell('selected (the one being edited)', 280, 28, _fixed(280, _mineRow(_seedMine()[0], sel: true))),
        _Cell('kind: Curve', 280, 28, _fixed(280, _mineRow(_seedMine()[1]))),
        _Cell('kind: Colour, JP + EN', 280, 28, _fixed(280, _mineRow(_seedMine()[2]))),
        _Cell('kind: Layer + scope word', 280, 28, _fixed(280, _mineRow(_seedMine()[3]))),
        _Cell('long name (ellipsis)', 280, 28, _fixed(280, _mineRow(_Mine('Interview_Tanaka-san_Cam-B_take-07 色補正セット final v2', 'Curve', 'This project')))),
      ]),
      ('HISTORY ROW · 36 px', [
        _Cell('rest', 296, 36, _fixed(296, const _HistRow('14:02:11', 'Glow 40 · Blur 12 · Levels'))),
        _Cell('hover', 296, 36, _fixed(296, const _HistRow('14:02:11', 'Glow 40 · Blur 12 · Levels', hover: true))),
        _Cell('empty state of a layer', 296, 36, _fixed(296, const _HistRow('13:55:09', '(empty)'))),
        _Cell('long description (ellipsis)', 296, 36, _fixed(296, const _HistRow('13:58:02', 'Glow 35 · Blur 10 · Levels · Tint #4781E5 · Echo 3'))),
      ]),
      ('NAME BOX · 24 px', [
        _Cell('focused (selected text)', 280, 24, _fixed(280, _NameEdit(initial: 'Glow 40% + Levels', onDone: (_) {}, autofocus: false))),
      ]),
      ('PARAMETER BAR (edited copy) · 20 px', [
        _Cell('value 30 %', 242, 20, _fixed(242, const _Param('Intensity', .30, hot: true))),
        _Cell('value 80 %', 242, 20, _fixed(242, const _Param('Radius', .80, hot: true))),
        _Cell('untouched (not hot)', 242, 20, _fixed(242, const _Param('Radius', .80))),
      ]),
      ('SAVE ZONE · EMPTY SHELF', [
        _Cell('zone idle', 296, 128, const SizedBox(width: 296, height: 38, child: _DropZone('carry here to save'))),
        _Cell('zone, carrying over it', 296, 128, const SizedBox(width: 296, height: 38, child: _DropZone('carry here to save', on: true))),
        _Cell('empty shelf card', 296, 128, _fixed(296, _empty('自分の棚はまだ空です', '')), ground: N.g07),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring on rows (keys act on the panel; only the name box takes focus), and disabled or error rows.');

// ---- B12 library
WidgetbookUseCase _partsCells() => _parts('thumbnails, cells, list rows', 'B12-a small parts: the grey thumbnail shape of every kind and family, the asset cell and the 28 px list row, in every state.', [
      ('THUMBNAIL SHAPE · one per kind and family', [
        for (final (cap, sh) in const [('image', _Sh.image), ('video', _Sh.video), ('audio', _Sh.audio), ('3D', _Sh.model), ('360°', _Sh.pano), ('blur', _Sh.blur), ('colour', _Sh.color), ('distort', _Sh.distort), ('stylize', _Sh.stylize), ('time', _Sh.time), ('generate', _Sh.generate), ('transition', _Sh.wipe), ('text', _Sh.text)])
          _Cell(cap, 92, 57, SizedBox(width: 92, height: 57, child: ClipRRect(borderRadius: BorderRadius.circular(6), child: CustomPaint(painter: _Ink(sh, 3), size: Size.infinite)))),
        _Cell('ease (key effect)', 92, 57, SizedBox(width: 92, height: 57, child: ClipRRect(borderRadius: BorderRadius.circular(6), child: _Thumb(_fx('Elastic'))))),
      ]),
      ('ASSET CELL · 92 px', [
        _Cell('image, rest', 92, 88, _AssetCell(_asset(Kind.image), 92)),
        _Cell('hover', 92, 88, _AssetCell(_asset(Kind.image), 92, hover: true)),
        _Cell('selected', 92, 88, _AssetCell(_asset(Kind.image), 92, sel: true)),
        _Cell('video: badge', 92, 88, _AssetCell(_mediaShelf[7], 92)),
        _Cell('audio', 92, 88, _AssetCell(_asset(Kind.audio), 92)),
        _Cell('long JP + EN name', 92, 88, _AssetCell(_mediaShelf[8], 92)),
        _Cell('long JP name', 92, 88, _AssetCell(_mediaShelf[9], 92)),
        _Cell('thumbnail only (cell under 70)', 92, 88, _AssetCell(_asset(Kind.video), 56, names: false)),
      ]),
      ('ASSET CELL · 140 px', [
        _Cell('rest', 140, 120, _AssetCell(_mediaShelf[7], 140)),
        _Cell('selected', 140, 120, _AssetCell(_mediaShelf[7], 140, sel: true)),
      ]),
      ('LIST ROW · 28 px', [
        _Cell('rest', 280, 28, _fixed(280, _AssetRow(_asset(Kind.image)))),
        _Cell('hover', 280, 28, _fixed(280, _AssetRow(_asset(Kind.image), hover: true))),
        _Cell('selected', 280, 28, _fixed(280, _AssetRow(_asset(Kind.image), sel: true))),
        _Cell('video clock', 280, 28, _fixed(280, _AssetRow(_mediaShelf[7]))),
        _Cell('long EN name (ellipsis)', 280, 28, _fixed(280, _AssetRow(_mediaShelf[8]))),
        _Cell('long JP name, selected', 280, 28, _fixed(280, _AssetRow(_mediaShelf[9], sel: true))),
        _Cell('audio', 280, 28, _fixed(280, _AssetRow(_asset(Kind.audio)))),
      ]),
    ], absent: 'a pressed look on cells and rows (a click selects at once), a focus ring (keys act on the panel), and disabled or error cells (an unreadable folder is the error card in the next sheet).');

WidgetbookUseCase _partsLibStates() => _parts('search, controls, states', 'B12-a small parts: the search field, view switch, size slider and the count header; then the loading skeleton, the no-results card and the unreadable-folder card.', [
      ('SEARCH FIELD · 28 px', [
        _Cell('empty (hint)', 296, 28, _fixed(296, const TextBox(text: '', hint: 'Search items', look: Look.quiet, clearable: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('typed, clear shown', 296, 28, _fixed(296, const TextBox(text: 'hypno', hint: 'Search items', look: Look.quiet, clearable: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('focus ring', 296, 28, _fixed(296, const TextBox(text: 'hyp', hint: 'Search items', look: Look.quiet, clearable: true, ring: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('error', 296, 28, _fixed(296, const TextBox(text: '[unclosed', hint: 'Search items', look: Look.quiet, invalid: true, leading: nav.Glyph(nav.G.search, color: N.g76)))),
        _Cell('disabled (offline)', 296, 28, _fixed(296, const TextBox(text: '', hint: 'Search items', look: Look.quiet, enabled: false, leading: nav.Glyph(nav.G.search, color: N.g76)))),
      ]),
      ('VIEW SWITCH · SIZE SLIDER · 24 px', [
        _Cell('view: Grid', 140, 24, IgnorePointer(child: Segmented(items: const ['Grid', 'List'], index: 0, onChanged: (_) {}))),
        _Cell('view: List', 140, 24, IgnorePointer(child: Segmented(items: const ['Grid', 'List'], index: 1, onChanged: (_) {}))),
        _Cell('size slider (grid only)', 140, 24, Row(mainAxisSize: MainAxisSize.min, children: [Text('size', style: T.label(N.g76)), const SizedBox(width: 4), MiniSlider(value: 84, min: 44, max: 140, width: 92, onChanged: (_) {})])),
      ]),
      ('COUNT HEADER · 22 px', [
        _Cell('all items', 296, 22, _fixed(296, const _Sec('Items', n: '5,000'))),
        _Cell('filtered', 296, 22, _fixed(296, const _Sec('Items', n: '37 of 5,000'))),
        _Cell('loading', 296, 22, _fixed(296, const _Sec('Items', n: 'reading…'))),
        _Cell('error', 296, 22, _fixed(296, const _Sec('Items', n: 'offline'))),
      ]),
      ('LOADING · 92 px', [
        _Cell('skeleton (static)', 92, 88, const _Skel(92)),
        _Cell('skeleton (pulsing, as shown)', 92, 88, const _Pulse(child: _Skel(92))),
        _Cell('asset cell for comparison', 92, 88, _AssetCell(_asset(Kind.image), 92)),
      ]),
      ('EMPTY · ERROR', [
        _Cell('no results', 296, 168, _fixed(296, _empty('"hypno" に一致する素材はありません', '', action: const _Btn('Clear search', onTap: _noop)))),
        _Cell('folder unreadable', 296, 168, _fixed(296, _empty('このフォルダを読めません', 'Permission denied\n/Volumes/RAID_A/Footage/2026/Day3_Shoot', action: const Row(mainAxisSize: MainAxisSize.min, children: [_Btn('Retry', onTap: _noop), SizedBox(width: 6), _Btn('Locate…', onTap: _noop)])))),
      ]),
    ], absent: 'a pressed or focus look on the view switch and the slider (they are shared Inputs parts with their own sheets), and a disabled grid / list (the two faces are always both there).');

// ---- B12 similar, axis, live folder
WidgetbookUseCase _partsLive() => _parts('similar, band, live rows', 'B12-b/c/d small parts: the closeness badge, the Similar chip and the context menu, the range band on each axis, and the live-folder row in every state.', [
      ('CLOSENESS BADGE · 92 px', [
        _Cell('source (selected)', 92, 88, _AssetCell(_asset(Kind.image), 92, sel: true, badge: 'source')),
        _Cell('92 %', 92, 88, _AssetCell(_mediaShelf[7], 92, badge: '92%')),
        _Cell('hover + 41 %', 92, 88, _AssetCell(_mediaShelf[9], 92, hover: true, badge: '41%')),
        _Cell('3 % (floor)', 92, 88, _AssetCell(_asset(Kind.audio), 92, badge: '3%')),
      ]),
      ('SIMILAR CHIP · 24 px', [
        _Cell('rest', 200, 24, const _SimChip('dusk_ridge_001.jpg', 200)),
        _Cell('hover', 200, 24, const _SimChip('dusk_ridge_001.jpg', 200, hover: true)),
        _Cell('long name (ellipsis)', 200, 24, const _SimChip('Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', 200)),
      ]),
      ('CONTEXT MENU', [
        _Cell('one live item, three disabled', 168, 120, _CtxMenu(onFind: () {})),
      ]),
      ('RANGE BAND · 112 px', [
        _Cell('colour axis', 296, 112, const _Band(hist: _demoHist, axis: 0, bw: 296)),
        _Cell('duration axis, a range kept', 296, 112, const _Band(hist: _demoHist, axis: 1, bw: 296, r: (.3, .7))),
        _Cell('type axis', 296, 112, const _Band(hist: _demoHist, axis: 2, bw: 296)),
      ]),
      ('LIVE FOLDER ROW · 28 px', [
        _Cell('rest', 296, 28, _fixed(296, _LiveRow('A001_C003_0612_R2X5.mov', _dropAssets[0]))),
        _Cell('hover', 296, 28, _fixed(296, _LiveRow('A001_C003_0612_R2X5.mov', _dropAssets[0], hover: true))),
        _Cell('selected', 296, 28, _fixed(296, _LiveRow('A001_C003_0612_R2X5.mov', _dropAssets[0], sel: true))),
        _Cell('NEW (4 s)', 296, 28, _fixed(296, _LiveRow('new_clip_001.mov', _dropAssets[0], fresh: true))),
        _Cell('missing on disk + Locate', 296, 28, _fixed(296, _LiveRow('slate_scene12.png', _dropAssets[2], onDisk: false))),
        _Cell('drive offline', 296, 28, _fixed(296, _LiveRow('メイキング_001.mov', _dropAssets[1], offline: true))),
        _Cell('long name', 296, 28, _fixed(296, _LiveRow('Interview_Tanaka-san_Cam-B_take-07_final_v2.mov', _dropAssets[0]))),
      ]),
      ('PLACE TAG · 16 px', [
        _Cell('LIVE', 120, 16, const _Tag('LIVE', hi: true)),
        _Cell('OFFLINE', 120, 16, const _Tag('OFFLINE')),
      ]),
    ], absent: 'a pressed look on rows (a click selects at once), a focus ring (keys act on the panel), and disabled cells. The disabled items of the context menu are drawn; the Locate button has the button states in the first sheet.');

// ---- B13 context filter
WidgetbookUseCase _partsContext() => _parts('tiles, pick card, lines', 'B13 small parts: the tile that fits and the tile that does not (dim + reason chip), the layer card on the stand-in Stage, the selection chips and the shown / hidden line.', [
      ('TILE: FITS vs DOES NOT FIT · 92 px', [
        _Cell('fits, rest', 92, 88, _FxTile(_fxs[0], 92)),
        _Cell('fits, hover', 92, 88, _FxTile(_fxs[0], 92, hover: true)),
        _Cell('dim: reason word', 92, 88, _FxTile(_fx('Typewriter'), 92, dim: true, reason: 'Text only')),
        _Cell('dim: Key を選ぶ', 92, 88, _FxTile(_fx('Bounce'), 92, dim: true, reason: 'Key を選ぶ')),
        _Cell('dim: Layer を選ぶ', 92, 88, _FxTile(_fxs[0], 92, dim: true, reason: 'Layer を選ぶ')),
        _Cell('dim, long name', 92, 88, _FxTile(_fxs[31], 92, dim: true, reason: 'Text only')),
      ]),
      ('LAYER CARD ON THE STAND-IN STAGE · 44 px', [
        _Cell('fits (blinks)', 70, 44, _fixed(70, const _PickCard('タイトル', 'Text', ok: true))),
        _Cell('fits, hover', 70, 44, _fixed(70, const _PickCard('タイトル', 'Text', ok: true, hover: true))),
        _Cell('does not fit', 70, 44, _fixed(70, const _PickCard('Logo Mark', 'Shape', ok: false))),
        _Cell('does not fit, hover (no change)', 70, 44, _fixed(70, const _PickCard('Logo Mark', 'Shape', ok: false, hover: true))),
        _Cell('long name', 70, 44, _fixed(70, const _PickCard('BG Gradient 長い名前', 'Solid', ok: true))),
      ]),
      ('SELECTION CHIPS · 22 px', [
        _Cell('Text on', 244, 22, const Row(mainAxisSize: MainAxisSize.min, children: [_Chip('None'), SizedBox(width: 4), _Chip('Shape'), SizedBox(width: 4), _Chip('Text', on: true), SizedBox(width: 4), _Chip('Key'), SizedBox(width: 4), _Chip('Mixed')])),
        _Cell('Key hover', 244, 22, const Row(mainAxisSize: MainAxisSize.min, children: [_Chip('None'), SizedBox(width: 4), _Chip('Shape'), SizedBox(width: 4), _Chip('Text'), SizedBox(width: 4), _Chip('Key', hover: true), SizedBox(width: 4), _Chip('Mixed')])),
        _Cell('None on', 244, 22, const Row(mainAxisSize: MainAxisSize.min, children: [_Chip('None', on: true), SizedBox(width: 4), _Chip('Shape'), SizedBox(width: 4), _Chip('Text'), SizedBox(width: 4), _Chip('Key'), SizedBox(width: 4), _Chip('Mixed')])),
      ]),
      ('COUNT LINE · SECTION · 22 px', [
        _Cell('shown / hidden + action', 296, 22, _fixed(296, Row(children: [Text('54 shown', style: T.label(N.g91)), Text(' · ', style: T.label(N.g76)), Text('9 hidden', style: T.label(N.g76)), const Spacer(), const _Chip('Show all')]))),
        _Cell('action on', 296, 22, _fixed(296, Row(children: [Text('54 shown', style: T.label(N.g91)), Text(' · ', style: T.label(N.g76)), Text('9 hidden', style: T.label(N.g76)), const Spacer(), const _Chip('Hide misfits', on: true)]))),
        _Cell('fit / do not (dim case)', 296, 22, _fixed(296, _t('54 fit · 9 do not', T.label(N.g76)))),
        _Cell('section header', 296, 22, _fixed(296, const _Sec('Blur & Sharpen', n: '7'))),
        _Cell('nothing selected: no line is drawn', 296, 22, const SizedBox.shrink()),
      ]),
    ], absent: 'a pressed look on tiles (a click applies at once), a focus ring (keys act on the panel), and disabled tiles (the dim tile is the "does not fit" state). The error case is the reason chip on the dim tile.');

// ================================================================ the set

_Sel _selKnob(BuildContext c, _Sel init) => c.knobs.object.dropdown<_Sel>(label: 'Selection', options: _Sel.values, initialOption: init, labelBuilder: (s) => _selName(s));

/// Panel drafts of group B (Browser), released as nine small components in the recommended order. Each starts with its Parts @200% sheet(s), then the panels.
/// Every panel has a one-line story outside the frame and the two knobs Panel width and Panel height.
List<WidgetbookComponent> panelBrowserBSets() => [
      WidgetbookComponent(name: 'Browser B7 Click', useCases: [
        _partsTiles(),
        _uc('B7-a Click selects, double-click uses: Effects', 'B7-a Grammar', _Hab.habit, '単クリックは選ぶだけ、ダブルクリックか Enter で選択対象に使う。全棚で同じ。', (c, w) => _GrammarLab(w: w, mode: 0)),
        _uc('B7-a Click selects, double-click uses: Media', 'B7-a Grammar', _Hab.habit, 'The same rule on the Media shelf: double-click adds the file to the comp as a layer.', (c, w) => _GrammarLab(w: w, mode: 0, startTab: 1)),
        _uc('B7-b Two shelves, two meanings', 'B7-b Split meaning', _Hab.counter, '反例: Effects は単クリックで使い、Media は単クリックで選ぶ。契約が 1 行で言えない。', (c, w) => _GrammarLab(w: w, mode: 1)),
        _uc('B7-c Single click uses, undo 1 step', 'B7-c Click uses', _Hab.departs, '単クリックで選択対象に使い、undo は 1 手。対象が無ければ何も書かない。', (c, w) => _GrammarLab(w: w, mode: 2, target: _selKnob(c, _Sel.text))),
        _uc('B7-c Single click uses: no target', 'B7-c Click uses', _Hab.departs, 'Nothing is selected, so a click asks what to apply it to and writes nothing.', (c, w) => _GrammarLab(w: w, mode: 2, target: _Sel.none)),
        _uc('B7-d Browser shows its target', 'B7-d Target line', _Hab.addition, '対象は 1 行で見え、tile の click と Enter はその対象にだけ効く。', (c, w) => _GrammarLab(w: w, mode: 3, target: _selKnob(c, _Sel.text))),
        _uc('B7-d Browser shows its target: several layers', 'B7-d Target line', _Hab.addition, 'Five layers selected: the line says so before you act, and one click is one undo for all five.', (c, w) => _GrammarLab(w: w, mode: 3, target: _Sel.many)),
      ]),
      WidgetbookComponent(name: 'Browser B9 Swap', useCases: [
        _partsSwapChips(),
        _partsSwapRows(),
        _uc('B9-a Hot-swap: link mode (Q), at rest', 'B9-a Hot-swap', _Hab.habit, 'Q ties the Browser to the selected effect; ↑ ↓ swaps it in place; Esc goes back before Enter.', (c, w) => _HotSwap(w: w)),
        _uc('B9-a Hot-swap: linked, trying candidates', 'B9-a Hot-swap', _Hab.habit, 'While linked the card shows BEFORE and NOW, and history stays untouched until Enter.', (c, w) => _HotSwap(w: w, startLinked: true)),
        _uc('B9-a Hot-swap: layer has no effect yet', 'B9-a Hot-swap', _Hab.habit, 'The layer has no effect yet: Link is disabled, the card says why, and a double-click on a row adds one.', (c, w) => _HotSwap(w: w, mode: 1)),
        _uc('B9-a Hot-swap: layer locked', 'B9-a Hot-swap', _Hab.habit, 'The layer is locked: Link is disabled and the card says why in words.', (c, w) => _HotSwap(w: w, mode: 2)),
      ]),
      WidgetbookComponent(name: 'Browser B12 Library', useCases: [
        _partsCells(),
        _partsLibStates(),
        _uc('B12-a Thumbnails, many items', 'B12-a Grid ↔ list', _Hab.habit, 'List and thumbnails are one thing with two faces: the selection and the count stay. Knob: Items.', (c, w) {
          final n = c.knobs.object.dropdown<int>(label: 'Items', options: const [5000, 200, 5], initialOption: 5000, labelBuilder: (v) => _fmt(v));
          return _Library(key: ValueKey(n), w: w, n: n);
        }),
        _uc('B12-a List, 200 items', 'B12-a Grid ↔ list', _Hab.habit, 'The list face of the same panel: long Japanese and English names truncate, never wrap.', (c, w) => _Library(w: w, n: 200, view: 1)),
        _uc('B12-a Loading', 'B12-a Grid ↔ list', _Hab.habit, 'Skeleton tiles keep the layout, so nothing jumps when the thumbnails arrive.', (c, w) => _Library(w: w, n: 5000, state: 1)),
        _uc('B12-a No results', 'B12-a Grid ↔ list', _Hab.habit, 'Nothing matches: the card says what was searched and gives a one-click way out.', (c, w) => _Library(w: w, n: 5000, query: 'hypno')),
        _uc('B12-a Folder unreadable', 'B12-a Grid ↔ list', _Hab.habit, 'The folder cannot be read: the reason, the path and two next steps.', (c, w) => _Library(w: w, n: 5000, state: 2)),
      ]),
      WidgetbookComponent(name: 'Browser B13 Context', useCases: [
        _partsContext(),
        _uc('B13-a Misfits are not shown: Text layer', 'B13-a Hidden', _Hab.addition, '選択に合わない物は見せない。選択を変えると一覧が変わり、隠した件数を 1 行で言う。', (c, w) => _Context(w: w, mode: 0, sel: _selKnob(c, _Sel.text))),
        _uc('B13-a Misfits are not shown: Key', 'B13-a Hidden', _Hab.addition, 'With keys selected only the curve presets remain; Show all brings the rest back in one click.', (c, w) => _Context(w: w, mode: 0, sel: _selKnob(c, _Sel.key))),
        _uc('B13-a Misfits are not shown: nothing selected', 'B13-a Hidden', _Hab.addition, 'Nothing selected means everything is shown, so there is no count line.', (c, w) => _Context(w: w, mode: 0, sel: _selKnob(c, _Sel.none))),
        _uc('B13-b Dim, with a reason word', 'B13-b Dim + reason', _Hab.addition, '合わない tile は淡く残し、理由を tile の上に書く。押しても何も書かない。', (c, w) => _Context(w: w, mode: 1, sel: _selKnob(c, _Sel.shape))),
        _uc('B13-c Press a misfit, then pick a layer', 'B13-c Pick later', _Hab.addition, '合わない tile を押すと layer を選ぶ手順になり、選ぶと適用、Esc で何も書かない。', (c, w) => _Context(w: w, mode: 2, sel: _selKnob(c, _Sel.shape))),
      ]),
      WidgetbookComponent(name: 'Browser B10 Favourites', useCases: [
        _partsFav(),
        _uc('B10-a Numbered collections (1-9)', 'B10-a Collections', _Hab.habit, '選んで 1-9 で付け、もう一度で外し、0 で全部外す。集合は名前で絞り、Recent は自動。', (c, w) => _Collections(w: w)),
        _uc('B10-a Numbered collections: first use (empty)', 'B10-a Collections', _Hab.habit, 'First use: nine named, empty collections; the first number key press teaches itself.', (c, w) => _Collections(w: w, empty: true)),
        _uc('B10-b One star and automatic Recent / Frequent', 'B10-b Star + auto', _Hab.habit, '星は 1 種(F キー)。Recent と Frequent は使った分から自動で、手では触らない。', (c, w) => _StarAuto(w: w)),
        _uc('B10-b One star: Frequent view', 'B10-b Star + auto', _Hab.habit, 'Frequent is ordered by use count: double-click an effect in All and watch it climb.', (c, w) => _StarAuto(w: w, start: 3)),
        _uc('B10-b One star: nothing yet (fresh install)', 'B10-b Star + auto', _Hab.habit, 'Nothing yet (fresh install): each empty view says how it fills itself.', (c, w) => _StarAuto(w: w, start: 1, fresh: true)),
        _uc('B10-c A pile of what was used', 'B10-c Pile', _Hab.addition, '使った物は先頭に積まれ、同じ物は前へ動き、上限を超えると古い物が落ちる。Knob: Cap.', (c, w) {
          final cap = c.knobs.int.slider(label: 'Cap', initialValue: 12, min: 4, max: 12);
          return _Pile(key: ValueKey(cap), w: w, cap: cap);
        }),
      ]),
      WidgetbookComponent(name: 'Browser B9 Cards A-B', useCases: [
        _partsCards(),
        _uc('B9-b Drop on an effect card replaces (Alt adds)', 'B9-b Card drop', _Hab.habit, 'カードへ drop で置換(同名の値は持ち越し)、Alt で新規、Esc で取り消し。tray が Browser の代役。', (c, w) => _DropCards(w: w, altSim: c.knobs.boolean(label: 'Alt held (simulate)', initialValue: false)), w: 282),
        _uc('B9-c Card ↑↓ walks the family', 'B9-c Card step', _Hab.addition, '選んだカードで ↑↓ かホイール = 同 family の次/前に置換。1 回 = 1 undo。', (c, w) => _CardStep(w: w), w: 282),
        _uc('B9-d A/B two frames, open', 'B9-d A/B', _Hab.addition, '候補を選ぶと今の絵と 2 枠で並び、Keep で 1 回書き、Close で何も書かず閉じる。', (c, w) => _Ab(w: w)),
        _uc('B9-d A/B two frames, closed', 'B9-d A/B', _Hab.addition, 'Closed: the compare region is one quiet line until a candidate is picked.', (c, w) => _Ab(w: w, startOpen: false)),
      ]),
      WidgetbookComponent(name: 'Browser B8 Drop', useCases: [
        _partsDrop(),
        _uc('B8-a Add by default, Alt replaces', 'B8-a Drop on layer', _Hab.habit, 'layer の上でも drop は追加、Alt で置換(bar と key は残る)。hover で結果が先に見える。', (c, w) => _DropLab(w: w, mode: 0, altSim: c.knobs.boolean(label: 'Alt held (simulate)', initialValue: false))),
        _uc('B8-b Replace by default, Alt adds', 'B8-b Drop on layer', _Hab.departs, 'layer への drop が絵の差し替え、Alt で追加、空き地は新規。D-2 は採らなかった(上書き事故)。', (c, w) => _DropLab(w: w, mode: 1, altSim: c.knobs.boolean(label: 'Alt held (simulate)', initialValue: false))),
        _uc('B8-c Layers refuse; Source field replaces', 'B8-c Refuse', _Hab.departs, 'layer は drop を受けず、Inspector の Source 欄が唯一の置換口。空き地は新規。', (c, w) => _DropLab(w: w, mode: 2)),
        _uc('B8-d Hold still: Replace | Add faces', 'B8-d Two faces', _Hab.addition, 'hover が止まると行の下に 2 面が開き、面の上で離した方を実行。面の外で離すと取り消し。', (c, w) => _DropLab(w: w, mode: 3, dwell: c.knobs.object.dropdown<int>(label: 'Dwell (ms)', options: const [1000, 500, 250], initialOption: 1000, labelBuilder: (v) => '$v'))),
      ]),
      WidgetbookComponent(name: 'Browser B11 Mine', useCases: [
        _partsMine(),
        _uc('B11-a Carry onto Mine to save', 'B11-a Save by drop', _Hab.habit, 'drop = 保存(元は変わらない)。名前欄がその場で開き、保存先は 1 語で選ぶ。', (c, w) => _SaveToShelf(w: w)),
        _uc('B11-a Carry onto Mine: empty shelf', 'B11-a Save by drop', _Hab.habit, 'Mine is empty: the card says what goes here and how to put it there.', (c, w) => _SaveToShelf(w: w, empty: true)),
        _uc('B11-b Editing a bundled preset makes a copy', 'B11-b Copy on edit', _Hab.addition, '同梱 preset は書き換わらず、値に触れた瞬間に複製が Mine に増える。', (c, w) => _CopyOnEdit(w: w)),
        _uc('B11-c History snapshots; name to keep', 'B11-c Snapshots', _Hab.addition, '履歴は上限付きで自動に取り、名前を付けた物だけが残る。layer は右の箱から動かす。', (c, w) => _Snaps(w: w)),
      ]),
      WidgetbookComponent(name: 'Browser B12 Similar Axis Live', useCases: [
        _partsLive(),
        _uc('B12-b Find similar: ready', 'B12-b Similar', _Hab.addition, '似た物を探すと chip になり、外すと全件に戻る。Right-click a tile or press S.', (c, w) => _Similar(w: w)),
        _uc('B12-b Find similar: ranked', 'B12-b Similar', _Hab.addition, 'Ranked by a made-up 8-number description: the source stays first and each tile shows a closeness %.', (c, w) => _Similar(w: w, start: 6)),
        _uc('B12-c One-axis band: colour', 'B12-c Axis strip', _Hab.departs, '軸を 1 つ選び、帯を drag して範囲で絞る。400 件。', (c, w) => _AxisStrip(w: w)),
        _uc('B12-c One-axis band: duration', 'B12-c Axis strip', _Hab.departs, 'The same band on a log duration axis: stills pile up on the left.', (c, w) => _AxisStrip(w: w, axis: 1)),
        _uc('B12-d A folder as a live source', 'B12-d Live folder', _Hab.addition, 'ファイルを置くと一覧に出て、消すと Missing の印で残る。ディスクは右の箱から動かす。', (c, w) => _LiveFolder(w: w)),
      ]),
    ];
