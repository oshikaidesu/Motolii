// The window's top bar: who and what (wordmark, project, comp), the mode, and the one loud action (Render).
import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/services.dart' show LogicalKeyboardKey;
import 'package:flutter/widgets.dart';
import '../forms/forms.dart';
import '../parts/skin_parts.dart';
import 'zoom.dart';

import '../theme.dart';
import '../tokens.dart';
import 'stage_art.dart' show WsArt;
import 'ws.dart';
import 'desk_kit.dart' show DeskChip;
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
          key: ValueKey(flip ? 'ws-redo' : 'ws-undo'),
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

  String _free = '';   // (the folder's room, read as the dialog opens)

  /// How much room the folder's disk has, as the Mac's df says it (its Available, in 1024-byte blocks).
  static String _room(String? folder) {
    try {
      final cols = (Process.runSync('df', ['-k', folder ?? '.']).stdout as String).trim().split('\n').last.split(RegExp(r'\s+'));
      return '${(int.parse(cols[3]) / (1024 * 1024)).toStringAsFixed(1)} GB free';
    } catch (_) {
      return '';
    }
  }

  void _ask() {
    if (!mounted || _open.isShowing) return;
    final ws = WsScope.read(context), doc = ws.document ?? 'film.motolii';
    _free = _room(ws.documentFolder);
    _name.text = '${doc.replaceFirst(RegExp(r'\.motolii$'), '')}.mp4';
    _open.show();
    WidgetsBinding.instance.addPostFrameCallback((_) => mounted ? _focus.requestFocus() : null);
  }

  // (the kinds the inside writes, by the file's ending: kit_draw.h; Alight Motion's own four)
  static const _formats = [('MP4', '.mp4', "H.264, with the films' sound"), ('HEVC', '.mov', "HEVC by the Mac's own encoder, with the films' sound"), ('GIF', '.gif', '15 frames a second, no sound'), ('PNG', '.png', 'a picture each, name_0001.png on'), ('JPEG', '.jpg', 'a JPEG each, name_0001.jpg on')];
  String get _ending => _formats.map((f) => f.$2).firstWhere((e) => _name.text.trim().toLowerCase().endsWith(e), orElse: () => '.mp4');
  void _as(String ending) => setState(() => _name.text = '${_name.text.trim().replaceFirst(RegExp(r'\.(mp4|mov|gif|png|jpg)$', caseSensitive: false), '')}$ending');

  void _go(Ws ws) {
    var name = _name.text.trim();
    if (name.isEmpty) return;
    if (!_formats.any((f) => name.toLowerCase().endsWith(f.$2))) name = '$name.mp4';
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
        // (Esc and ⌘. cancel, as a Mac dialog's Cancel does: Apple's; the keys reach here from the name field it holds)
        return CallbackShortcuts(
          bindings: {const SingleActivator(LogicalKeyboardKey.escape): _open.hide, const SingleActivator(LogicalKeyboardKey.period, meta: true): _open.hide},
          child: Stack(
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
                    row('Folder', Text('${ws.documentFolder ?? ''}${_free.isEmpty ? '' : '  ·  $_free'}', maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g76))),
                    row('Range', fact('Work area  ${t(from)} – ${t(to)}  ·  $frames frames')),
                    row('Size', fact('${WsArt.comp.width.round()} × ${WsArt.comp.height.round()}')),
                    // (how it is written, the document's own and kept with it: Alight Motion's export rows; 0 as the stage is)
                    for (final (label, name, choices) in [
                      ('Resolution', 'render_height', [(0, 'Full'), for (final p in sizeResolutions) if (p < math.min(WsArt.comp.width, WsArt.comp.height)) (p, '${p}p')]),
                      ('Rate', 'render_rate', [(0, '${Ws.fps}'), for (final r in const [30, 24, 15]) if (r < Ws.fps) (r, '$r')]),
                      ('Quality', 'render_mbps', [(0, 'Auto'), for (final b in const [4, 8, 16, 32]) (b, '$b Mbps')]),
                      ('Motion blur', 'render_blur', const [(0, 'Off'), (8, 'On')]),   // (After Effects' 180° shutter: eight moments laid on each picture)
                      ('Background', 'render_alpha', const [(0, 'Solid'), (1, 'Clear')]),   // (clear where nothing is drawn: kept by PNG, GIF and a .mov's HEVC, kit_draw.h film)
                    ])
                      row(label, StatefulBuilder(builder: (context, again) {
                        final m = ws.inside, now = (m?.value(0, name, 0) ?? 0).round();
                        return Wrap(spacing: Shape.s(6), runSpacing: Shape.s(4), crossAxisAlignment: WrapCrossAlignment.center, children: [
                          for (final (v, said) in choices)
                            DeskChip(key: ValueKey('render-$name-$v'), label: said, on: now == v, onTap: m == null ? null : () => ws.after(m.later('set', thing: 0, name: name, number: v.toDouble()), (_, _) => mounted ? again(() {}) : null)),
                          if (name == 'render_mbps' && now > 0) Text('about ${(now * (to - from) / 8).toStringAsFixed(1)} MB', key: const ValueKey('render-estimate'), style: T.label(Grey.g76)),   // (megabits a second over its seconds, as Alight Motion estimates it)
                        ]);
                      })),
                    row('Format', StatefulBuilder(builder: (context, again) => Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                      Wrap(spacing: Shape.s(6), runSpacing: Shape.s(4), children: [
                        for (final (label, ending, _) in _formats) DeskChip(key: ValueKey('render-$label'), label: label, on: _ending == ending, onTap: () => again(() => _as(ending))),
                      ]),
                      SizedBox(height: Shape.s(4)),
                      Text(_formats.firstWhere((f) => f.$2 == _ending).$3, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.label(Grey.g76)),   // (under them, one line: five chips leave no room beside)
                    ]))),
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
          ),
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

/// Alight Motion's new-project box, its shapes and its resolutions (each the shorter side, as its 2160p to 270p are):
/// a shape and a resolution pressed is the document's size, opened again at it.
const sizeShapes = [('16:9', 16, 9), ('9:16', 9, 16), ('4:5', 4, 5), ('1:1', 1, 1), ('4:3', 4, 3)];
const sizeResolutions = [2160, 1440, 1080, 720, 540, 360, 270];
(int, int) sizeOf((String, int, int) shape, int p) {
  final (_, a, b) = shape;
  int even(double v) => (v / 2).round() * 2;   // (an even number of pixels: H.264 asks it)
  return a >= b ? (even(p * a / b), p) : (p, even(p * b / a));
}

/// A new document asks for its size, as Alight Motion's new-project box comes first: the size chip's box opened once the
/// window shows the new one (true until it is).
final wsAskSize = ValueNotifier<bool>(false);

class _SizeChipState extends State<_SizeChip> {
  final _text = TextEditingController(), _focus = FocusNode(), _box = OverlayPortalController();
  bool _editing = false;
  double? _scrub;   // (the width being scrubbed to)
  double _dx = 0;

  @override
  void initState() {
    super.initState();
    wsAskSize.addListener(_asked);
    WidgetsBinding.instance.addPostFrameCallback((_) => _asked());   // (asked while the window was opening the new one)
  }

  void _asked() {
    if (!mounted || !wsAskSize.value || WsScope.read(context).onSize == null) return;
    wsAskSize.value = false;
    _box.show();
  }

  @override
  void dispose() {
    wsAskSize.removeListener(_asked);
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
    // (pressed once: the shapes and resolutions; the one it is now lit, a shape of its own said as it is)
    final short = math.min(w0, h0).round(), now = sizeShapes.where((s) => sizeOf(s, short) == (w0.round(), h0.round())).firstOrNull;
    Widget box(BuildContext _) => Stack(
      children: [
        Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: _box.hide, child: const ColoredBox(color: Color(0x88000000)))),
        Center(
          child: Container(
            key: const ValueKey('ws-size-box'),
            width: Shape.z(380),
            padding: EdgeInsets.all(Shape.s(16)),
            decoration: BoxDecoration(color: WsT.card, borderRadius: BorderRadius.all(Shape.r(8)), border: Border.all(color: WsT.line)),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Document', style: T.title(Grey.g95).copyWith(fontWeight: FontWeight.w700)),
                SizedBox(height: Shape.s(12)),
                Text('Shape', style: T.label(Grey.g56)),
                SizedBox(height: Shape.s(6)),
                Wrap(spacing: Shape.s(6), runSpacing: Shape.s(6), children: [
                  for (final s in sizeShapes) DeskChip(key: ValueKey('size-${s.$1}'), label: s.$1, on: s == now, onTap: () => _to(ws, sizeOf(s, short))),
                  if (now == null) DeskChip(label: '$shownW:$shownH', on: true, onTap: () {}),   // (its own: a size typed)
                ]),
                SizedBox(height: Shape.s(12)),
                Text('Resolution', style: T.label(Grey.g56)),
                SizedBox(height: Shape.s(6)),
                Wrap(spacing: Shape.s(6), runSpacing: Shape.s(6), children: [
                  for (final p in sizeResolutions)
                    DeskChip(key: ValueKey('size-${p}p'), label: '${p}p', on: p == short, onTap: () => _to(ws, now == null ? ((p * w0 / short).round(), (p * h0 / short).round()) : sizeOf(now, p))),
                ]),
                SizedBox(height: Shape.s(12)),
                Text('Frame rate', style: T.label(Grey.g56)),   // (Alight Motion's ten: the document seen only at its frames, written at them too)
                SizedBox(height: Shape.s(6)),
                Wrap(spacing: Shape.s(6), runSpacing: Shape.s(6), children: [
                  for (final r in const [8, 10, 12, 15, 24, 25, 30, 48, 50, 60])
                    DeskChip(key: ValueKey('size-${r}fps'), label: '$r', on: r == Ws.fps, onTap: ws.inside == null ? null : () => ws.after(ws.inside!.later('set', thing: 0, name: 'frame_rate', number: r == ws.inside!.rate ? 0 : r.toDouble()), (_, _) => ws.reread())),
                ]),
                SizedBox(height: Shape.s(12)),
                Text('$shownW × $shownH  ·  ${Ws.fps} frames a second', style: T.value(Grey.g76)),
              ],
            ),
          ),
        ),
      ],
    );
    return OverlayPortal(
      controller: _box,
      overlayChildBuilder: box,
      child: MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        supportedDevices: wsPressing,
        key: const ValueKey('ws-top-size'),
        behavior: HitTestBehavior.opaque,
        onTap: ws.onSize == null ? null : _box.show,
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
      ),
    );
  }

  void _to(Ws ws, (int, int) size) {
    _box.hide();
    if (size != (WsArt.comp.width.round(), WsArt.comp.height.round())) ws.onSize?.call(size.$1, size.$2);
  }
}

/// How many times the window waited on the inside (the rule: never), a red dot and the count only when it did; a tap
/// lists each, named and timed.
class _Waited extends StatefulWidget {
  const _Waited();
  @override
  State<_Waited> createState() => _WaitedState();
}
