// The window's top bar: who and what (wordmark, project, comp), the mode, and the one loud action (Render).
import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import '../forms/forms.dart';
import '../parts/skin_parts.dart';
import 'zoom.dart';

import '../theme.dart';
import '../tokens.dart';
import 'stage_art.dart' show WsArt;
import 'ws.dart';
import 'two_fingers.dart';
part 'top_bar_2.dart';

class WsTopBar extends StatelessWidget {
  const WsTopBar({super.key});

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context, {WsAspect.animating, WsAspect.document, WsAspect.picked});
    return Container(
      color: WsT.body,
      padding: EdgeInsets.symmetric(horizontal: Shape.s(10)),
      child: Row(
        children: [
          const _Wordmark(),
          SizedBox(width: Shape.s(14)),
          const _UndoRedo(),
          SizedBox(width: Shape.s(14)),
          Flexible(child: Text(ws.document ?? '', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.name(Grey.g95).copyWith(fontWeight: FontWeight.w600))), // (a long name gives way, not the bar)
          SizedBox(width: Shape.s(6)),
          Text('/', style: T.name(Grey.g44)),
          SizedBox(width: Shape.s(6)),
          Text(ws.layer?.name ?? 'Stage', style: T.name(Grey.g76)), // (where the hand is: the picked thing, or the stage itself)
          SizedBox(width: Shape.s(10)),
          const _SizeChip(),
          SizedBox(width: Shape.s(4)),
          _Chip('${Ws.fps} fps'),
          SizedBox(width: Shape.s(4)),
          _Chip(ws.timecode(Ws.duration)),
          const Spacer(),
          const WsZoomReadout(),
          const _ShadeSwitch(),
          SizedBox(width: Shape.s(10)),
          const _Waited(),
          const _Saved(),
          SizedBox(width: Shape.s(12)),
          const _Render(),
        ],
      ),
    );
  }
}

/// A square lime block with the wordmark in it: the poster's own manner, used once.
class _Wordmark extends StatelessWidget {
  const _Wordmark();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Container(
        width: Shape.z(22),
        height: Shape.z(22),
        alignment: Alignment.center,
        color: WsT.accent,
        child: Text('M', style: T.title(WsT.onAccent).copyWith(fontSize: T.px(14), fontWeight: FontWeight.w900, height: 1)),
      ),
      SizedBox(width: Shape.s(8)),
      Text('Motolii', style: T.title(Grey.g95).copyWith(fontSize: T.px(14), fontWeight: FontWeight.w800, letterSpacing: -.2)),
    ],
  );
}

class _Chip extends StatelessWidget {
  const _Chip(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Container(
    height: Shape.z(20),
    padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
    child: Text(text, style: T.value(Grey.g76).copyWith(fontSize: T.px(10))),
  );
}

class _Hover extends StatefulWidget {
  const _Hover({super.key, required this.builder, this.onTap});
  final Widget Function(bool hot) builder;
  final VoidCallback? onTap;
  @override
  State<_Hover> createState() => _HoverState();
}

class _HoverState extends State<_Hover> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    cursor: SystemMouseCursors.click,
    onEnter: (_) => setState(() => _h = true),
    onExit: (_) => setState(() => _h = false),
    child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: widget.onTap, child: widget.builder(_h)),
  );
}

class _UndoRedo extends StatelessWidget {
  const _UndoRedo();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      for (final flip in [false, true])
        _Hover(
          // (a step of the document back, or forward again)
          onTap: () {
            final ws = WsScope.read(context), m = ws.inside;
            if (m == null) return;
            ws.after(m.later(flip ? 'redo' : 'undo')); // (the window does not wait for the step)
          },
          builder: (h) => Container(
            width: Shape.z(24),
            height: Shape.z(24),
            alignment: Alignment.center,
            decoration: BoxDecoration(color: h ? WsT.raised : null, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
            child: Transform.flip(
              flipX: flip,
              child: CustomPaint(size: const Size(12, 10), painter: _UndoPaint(h ? Grey.g95 : Grey.g63)),
            ),
          ),
        ),
    ],
  );
}

class _UndoPaint extends CustomPainter {
  const _UndoPaint(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    if (SkinParts.draw(canvas, s, 'undo', c)) return; // (the skin's own picture, if it brings one)
    final p = Paint()
      ..color = c
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.4
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    canvas
      ..drawPath(
        Path()
          ..moveTo(1, 4)
          ..lineTo(8, 4)
          ..arcToPoint(Offset(8, s.height - 1), radius: Shape.r(3.2))
          ..lineTo(4, s.height - 1),
        p,
      )
      ..drawPath(
        Path()
          ..moveTo(4, 1)
          ..lineTo(1, 4)
          ..lineTo(4, 7),
        p,
      );
  }

  @override
  bool shouldRepaint(_UndoPaint o) => o.c != c;
}

/// Dark / light: flips [Grey.shade] and rebuilds the whole window in place (reassemble keeps every State, so the edit survives).
class _ShadeSwitch extends StatelessWidget {
  const _ShadeSwitch();
  static void flip() {
    Grey.shade.value = Grey.light ? Shade.dark : Shade.light;
    WidgetsBinding.instance.reassembleApplication();
  }

  @override
  Widget build(BuildContext context) => _Hover(
    key: const ValueKey('ws-shade'),
    onTap: flip,
    builder: (h) => Container(
      width: Shape.z(24),
      height: Shape.z(24),
      alignment: Alignment.center,
      decoration: BoxDecoration(color: h ? WsT.raised : null, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
      child: CustomPaint(size: const Size(12, 12), painter: _ShadePaint(h ? Grey.g95 : Grey.g63)),
    ),
  );
}

/// A ring with its left half filled: the usual sign for "light or dark".
class _ShadePaint extends CustomPainter {
  const _ShadePaint(this.c);
  final Color c;
  @override
  void paint(Canvas canvas, Size s) {
    if (SkinParts.draw(canvas, s, 'shade', c)) return; // (the skin's own picture, if it brings one)
    final r = Rect.fromLTWH(.7, .7, s.width - 1.4, s.height - 1.4);
    canvas
      ..drawOval(
        r,
        Paint()
          ..color = c
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.4,
      )
      ..drawArc(r, 1.5708, 3.14159, true, Paint()..color = c);
  }

  @override
  bool shouldRepaint(_ShadePaint o) => o.c != c;
}

/// When the document's file was last written (a change is written as it is made).
String _savedAt(Ws ws) {
  final f = File('${ws.documentFolder}/${ws.document}');
  if (ws.document == null || !f.existsSync()) return 'Saved';
  final t = f.lastModifiedSync();
  return 'Saved ${t.hour.toString().padLeft(2, '0')}:${t.minute.toString().padLeft(2, '0')}';
}

class _Saved extends StatelessWidget {
  const _Saved();
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Container(
        width: Shape.z(6),
        height: Shape.z(6),
        decoration: BoxDecoration(color: WsT.accent, shape: BoxShape.circle),
      ),
      SizedBox(width: Shape.s(6)),
      ConstrainedBox(
        constraints: BoxConstraints(maxWidth: Shape.z(260)),
        child: Text(WsScope.of(context, {WsAspect.document, WsAspect.notice}).notice ?? _savedAt(WsScope.of(context, {WsAspect.document, WsAspect.notice})), maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g63)),
      ),
    ],
  );
}

/// The one loud action: a flat lime pill with dark ink.
class _Render extends StatefulWidget {
  const _Render();
  @override
  State<_Render> createState() => _RenderState();
}

/// Cmd+R and the Render button both ask for the settings window (the film's name, its range and size, its rate) before a film is written.
final wsAskRender = ValueNotifier<int>(0);

class _RenderState extends State<_Render> {
  final _open = OverlayPortalController(), _name = TextEditingController(), _focus = FocusNode();
  @override
  void initState() {
    super.initState();
    wsAskRender.addListener(_ask);
  }

  @override
  void dispose() {
    wsAskRender.removeListener(_ask);
    _name.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _ask() {
    if (!mounted || _open.isShowing) return;
    final ws = WsScope.read(context), doc = ws.document ?? 'film.motolii';
    _name.text = '${doc.replaceFirst(RegExp(r'\.motolii$'), '')}.mp4';
    _open.show();
    WidgetsBinding.instance.addPostFrameCallback((_) => mounted ? _focus.requestFocus() : null);
  }

  void _go(Ws ws) {
    var name = _name.text.trim();
    if (name.isEmpty) return;
    if (!name.toLowerCase().endsWith('.mp4')) name = '$name.mp4';
    ws.renderTo = name.startsWith('/') ? name : '${ws.documentFolder ?? '.'}/$name';
    _open.hide();
    ws.onRender?.call();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context, {WsAspect.document});
    return OverlayPortal(
      controller: _open,
      overlayChildBuilder: (_) {
        final (from, to) = ws.work;
        final frames = ((to - from) * Ws.fps).round();
        String t(double s) => '${s.toStringAsFixed(2)} s';
        Widget row(String k, Widget v) => Padding(
          padding: EdgeInsets.only(bottom: Shape.s(8)),
          child: Row(crossAxisAlignment: CrossAxisAlignment.center, children: [SizedBox(width: Shape.z(70), child: Text(k, style: T.label(Grey.g56))), Expanded(child: v)]),
        );
        Text fact(String v) => Text(v, style: T.value(Grey.g91));
        return Stack(
          children: [
            Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: _open.hide, child: const ColoredBox(color: Color(0x88000000)))),
            Center(
              child: Container(
                width: Shape.z(380),
                padding: EdgeInsets.all(Shape.s(16)),
                decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.all(Shape.r(8)), border: Border.all(color: WsT.line)),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Padding(
                      padding: EdgeInsets.only(bottom: Shape.s(12)),
                      child: Text('Render', style: T.title(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
                    ),
                    row(
                      'File',
                      Container(
                        padding: EdgeInsets.symmetric(horizontal: Shape.s(8), vertical: Shape.s(5)),
                        decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(4))),
                        child: EditableText(
                          controller: _name,
                          focusNode: _focus,
                          style: T.value(Grey.g95),
                          cursorColor: WsT.accent,
                          backgroundCursorColor: Grey.g38,
                          maxLines: 1,
                          onSubmitted: (_) => _go(ws),
                        ),
                      ),
                    ),
                    row('Folder', Text(ws.documentFolder ?? '', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g76))),
                    row('Range', fact('Work area  ${t(from)} – ${t(to)}  ·  $frames frames')),
                    row('Size', fact('${WsArt.comp.width.round()} × ${WsArt.comp.height.round()}')),
                    row('Rate', fact('${Ws.fps} frames a second')),
                    row('Format', fact("H.264 .mp4, with the films' sound")),
                    SizedBox(height: Shape.s(4)),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        GestureDetector(
                          onTap: _open.hide,
                          child: Padding(padding: EdgeInsets.symmetric(horizontal: Shape.s(12), vertical: Shape.s(6)), child: Text('Cancel', style: T.name(Grey.g76))),
                        ),
                        SizedBox(width: Shape.s(6)),
                        Forms.worn.action(context, ActionSpec('Render', () => _go(ws))),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ],
        );
      },
      child: Forms.worn.action(context, ActionSpec('Render', _ask)), // (as the skin's forms draw it)
    );
  }
}

/// The document's size, beside the others: pressed twice it is written over (a width alone keeps the shape; width and
/// height, as 1080x1920, set both), scrubbed sideways it moves the width 1% every 3 px, the shape kept, and
/// the document is opened again at it when the hand lets go.
class _SizeChip extends StatefulWidget {
  const _SizeChip();
  @override
  State<_SizeChip> createState() => _SizeChipState();
}

class _SizeChipState extends State<_SizeChip> {
  final _text = TextEditingController(), _focus = FocusNode();
  bool _editing = false;
  double? _scrub;   // (the width being scrubbed to)
  double _dx = 0;

  @override
  void dispose() {
    _text.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _commit(Ws ws) {
    final nums = RegExp(r'\d+').allMatches(_text.text).map((m) => int.parse(m.group(0)!)).toList();
    setState(() => _editing = false);
    if (nums.isEmpty || ws.onSize == null) return;
    final w0 = WsArt.comp.width, h0 = WsArt.comp.height;
    final w = nums[0], h = nums.length > 1 ? nums[1] : (nums[0] * h0 / w0).round();
    if (w >= 16 && h >= 16 && w <= 16384 && h <= 16384 && (w != w0.round() || h != h0.round())) ws.onSize!(w, h);
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.of(context, {WsAspect.document}), w0 = WsArt.comp.width, h0 = WsArt.comp.height;
    final shownW = (_scrub ?? w0).round(), shownH = (shownW * h0 / w0).round();
    if (_editing) {
      return Container(
        height: Shape.z(20),
        width: Shape.z(92),
        padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
        alignment: Alignment.centerLeft,
        decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius)), border: Border.all(color: WsT.accent)),
        child: EditableText(
          controller: _text,
          focusNode: _focus,
          style: T.value(Grey.g95).copyWith(fontSize: T.px(10)),
          cursorColor: WsT.accent,
          backgroundCursorColor: Grey.g38,
          maxLines: 1,
          onSubmitted: (_) => _commit(ws),
          onTapOutside: (_) => _commit(ws),
        ),
      );
    }
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        supportedDevices: wsPressing,
        key: const ValueKey('ws-top-size'),
        behavior: HitTestBehavior.opaque,
        onDoubleTap: ws.onSize == null
            ? null
            : () {
                _text.text = '${w0.round()}x${h0.round()}';
                _text.selection = TextSelection(baseOffset: 0, extentOffset: _text.text.length);
                setState(() => _editing = true);
                WidgetsBinding.instance.addPostFrameCallback((_) => _focus.requestFocus());
              },
        onHorizontalDragStart: (_) => _dx = 0,
        onHorizontalDragUpdate: (e) => setState(() => _scrub = math.max(16, w0 + w0 * ((_dx += e.delta.dx) / 3).round() / 100)),
        onHorizontalDragEnd: (_) {
          final w = _scrub?.round();
          setState(() => _scrub = null);
          if (w != null && w != w0.round() && ws.onSize != null) ws.onSize!(w, (w * h0 / w0).round());
        },
        child: _Chip('$shownW × $shownH'),
      ),
    );
  }
}

/// How many times the window waited on the inside (the rule: never), a red dot and the count only when it did; a tap
/// lists each, named and timed.
class _Waited extends StatefulWidget {
  const _Waited();
  @override
  State<_Waited> createState() => _WaitedState();
}
