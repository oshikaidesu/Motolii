import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../foundation/theme.dart';
import '../../hf/bp/common.dart' show sans, mono;
import '../../hf/glyphs.dart';
import '../../hf/neutral.dart';
import '../../hf/shell/menu.dart' show showHfMenu;
import '../../hf/shell/place.dart' show H, Fam;
import '../../hf/shell/sheet.dart' show HfAction;
import '../../input/viewport_motion.dart';
import '../../session/editor_session.dart';
import '../../timeline_core/layout.dart' show TrackRow;
import '../../timeline_core/semantics.dart';
import '../../timeline_core/session.dart';

/// The Timeline's tools at its seat strip's right end (the Dock asks the front panel for them): Split and Marker.
class LiveTimelineTools extends StatelessWidget {
  const LiveTimelineTools({super.key, required this.c});
  final EditorSession c;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: c.document,
        builder: (context, _) => Row(mainAxisSize: MainAxisSize.min, children: [
          HfAction('Split', onTap: c.supports('split') ? () => c.command('split') : null),
          HfAction('Marker', onTap: c.supports('addMarker') ? () => c.command('addMarker') : null),
        ]),
      );
}

/// A layer's colour, by its id so it stays with the layer wherever it scrolls.
final _families = <Fam>[H.scatter, H.stagger, H.along, H.face, H.attach, H.follow];
Fam _family(int id) => _families[id % _families.length];

/// The live Timeline: a skin over [TimelineSession]. It lays rows out and draws them, decides what is under the
/// pointer with its own geometry, and tells the session in rows and frames. Nothing of the Timeline's meaning or state
/// lives here beyond the pointer's own gesture (slop, hover, momentum).
class LiveTimeline extends StatefulWidget {
  const LiveTimeline({super.key, required this.c});
  final EditorSession c;
  @override
  State<LiveTimeline> createState() => _LiveTimelineState();
}

class _LiveTimelineState extends State<LiveTimeline> {
  // the skin's geometry: its own, told to nobody
  static const rowH = 22.0, rulerH = 24.0, keySlop = 6.0, edgeSlop = 5.0, dragSlop = 4.0;

  /// The name column: a share of the seat, so a narrow window keeps its time.
  double labelW = 236;

  EditorSession get c => widget.c;
  late final TimelineSession s = TimelineSession.of(c);
  final focus = FocusNode();
  late final motion = ViewportMotion((scale, x, y) => s.setView(pixelsPerFrame: scale, startFrame: x / scale, firstRow: y / rowH));
  int? hover;
  MouseCursor _cursor = MouseCursor.defer;
  Offset? _down;
  bool _dragging = false, _navigating = false;
  Offset _navOrigin = Offset.zero;

  @override
  void dispose() {
    motion.dispose();
    focus.dispose();
    super.dispose();
  }

  // ---- the skin's transforms -------------------------------------------------------------------------------------
  double xOf(num frame) => labelW + (frame - s.startFrame) * s.pixelsPerFrame;
  double frameAt(double x) => s.startFrame + (x - labelW) / s.pixelsPerFrame;
  double yOf(int row) => rulerH + (row - s.firstRow) * rowH;
  double rowAt(double y) => s.firstRow + (y - rulerH) / rowH;

  /// A bar's interaction geometry, the one place that says what a press takes (hit-testing and the cursor both ask):
  /// [dx] from the bar's drawn left edge, for a bar [w] wide on screen. A thin bar is at least [minHit] to the pointer
  /// (visual size is not hit size); its ends take at most a third each, so there is always a middle to carry it by;
  /// an end is also taken from just outside the bar.
  static const minHit = 12.0;
  TlBarPart? barGrab(double dx, double w) {
    final pad = math.max(0.0, (minHit - w) / 2);
    if (dx < -pad - edgeSlop || dx > w + pad + edgeSlop) return null;
    if (dx < -pad) return TlBarPart.start;
    if (dx > w + pad) return TlBarPart.end;
    final edge = math.min(edgeSlop, (w + 2 * pad) / 3);
    if (dx + pad < edge) return TlBarPart.start;
    if (w + pad - dx < edge) return TlBarPart.end;
    return TlBarPart.body;
  }

  TlMods get _mods {
    final k = HardwareKeyboard.instance;
    return TlMods(primary: k.isMetaPressed || k.isControlPressed, shift: k.isShiftPressed, alt: k.isAltPressed);
  }

  /// What is under [p] on the lanes, as the session's terms.
  TlTarget _hit(Offset p) {
    final i = rowAt(p.dy).floor();
    if (p.dy < rulerH || i < 0 || i >= s.rows.length) return const TlEmpty();
    final r = s.rows[i];
    if (r.property != null) {
      final frames = [for (final k in r.keys) (k['frame'] as num).toInt()]..sort();
      for (final f in frames) {
        if ((xOf(f) - p.dx).abs() <= keySlop) return TlLaneKey(i, f);
      }
      for (var n = 0; n + 1 < frames.length; n++) {
        if (p.dx > xOf(frames[n]) + keySlop && p.dx < xOf(frames[n + 1]) - keySlop) return TlSpan(i, frames[n], frames[n + 1]);
      }
      return const TlEmpty();
    }
    final start = (r.layer['start'] as num? ?? 0).toDouble(), end = start + (r.layer['duration'] as num? ?? 0).toDouble();
    final part = barGrab(p.dx - xOf(start), xOf(end) - xOf(start));
    // keys win inside a bar with room for them; a bar too short to hold its keys apart from its middle is carried,
    // and its keys are picked on its opened lanes (a bar always has a middle to carry it by)
    final roomy = xOf(end) - xOf(start) >= minHit * 2;
    if (!r.lanesOpen && (roomy || part == null) && (part == TlBarPart.body || part == null || (p.dx > xOf(start) && p.dx < xOf(end)))) {
      for (final f in r.summaryFrames) {
        if ((xOf(f) - p.dx).abs() <= keySlop) return TlLayerKey(i, f);
      }
    }
    if (part != null) return TlBar(i, part);
    return const TlEmpty();
  }

  // ---- pointer -----------------------------------------------------------------------------------------------------
  /// [p] in the panel's coordinates.
  void _press(PointerDownEvent e, TlTarget target, Offset p) {
    focus.requestFocus();
    motion.stop();
    if (e.buttons != kPrimaryMouseButton) return;
    _down = p;
    _dragging = false;
    s.press(target, frame: frameAt(p.dx), row: rowAt(p.dy), mods: _mods);
  }

  void _move(Offset p) {
    if (_down == null) return;
    if (!_dragging && (p - _down!).distance < dragSlop) return;
    _dragging = true;
    s.drag(frame: frameAt(p.dx), row: rowAt(p.dy));
  }

  void _up(PointerUpEvent e) {
    if (_down == null) return;
    _down = null;
    s.release();
  }

  void _cancel() {
    _down = null;
    s.cancel();
  }

  void _wheel(PointerSignalEvent event) {
    if (event is! PointerScrollEvent || _navigating) return;
    GestureBinding.instance.pointerSignalResolver.register(event, (_) {
      motion.stop();
      final p = event.localPosition, d = event.scrollDelta;
      final k = HardwareKeyboard.instance;
      if (k.isMetaPressed || k.isControlPressed || (p.dy < rulerH && p.dx >= labelW)) {
        s.zoomAt(math.exp(-d.dy * .002), frameAt(math.max(labelW, p.dx)));
      } else if (k.isShiftPressed) {
        s.setView(startFrame: s.startFrame + (d.dy + d.dx) / s.pixelsPerFrame);
      } else {
        s.setView(startFrame: s.startFrame + d.dx / s.pixelsPerFrame, firstRow: s.firstRow + d.dy / rowH);
      }
    });
  }

  Future<void> _menu(Offset p, Offset global) async {
    final i = rowAt(p.dy).floor();
    final row = p.dy >= rulerH && i >= 0 && i < s.rows.length ? i : null;
    final lines = await s.menu(row);
    if (!mounted) return;
    final chosen = await showHfMenu<String>(
      context,
      Rect.fromLTWH(global.dx, global.dy, 220, 0),
      [for (final l in lines) (l.value, l.label)],
      disabled: {for (final l in lines) if (!l.enabled) l.value},
      shortcuts: {for (final l in lines) if (l.shortcut.isNotEmpty) l.value: l.shortcut},
      dividers: {for (final l in lines) if (l.groupEnd) l.value},
    );
    if (chosen != null) s.runMenu(row, chosen);
  }

  Future<void> _markerMenu(String id, Offset global) async {
    if (!c.supports('deleteMarker')) return;
    final chosen = await showHfMenu<String>(context, Rect.fromLTWH(global.dx, global.dy, 180, 0), const [('delete', 'Delete marker')]);
    if (chosen == 'delete') s.deleteMarker(id);
  }

  KeyEventResult _key(FocusNode node, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final k = e.logicalKey;
    final handled = s.key(
      left: k == LogicalKeyboardKey.arrowLeft && !HardwareKeyboard.instance.isAltPressed,
      right: k == LogicalKeyboardKey.arrowRight && !HardwareKeyboard.instance.isAltPressed,
      escape: k == LogicalKeyboardKey.escape,
      shift: HardwareKeyboard.instance.isShiftPressed,
    );
    return handled ? KeyEventResult.handled : KeyEventResult.ignored;
  }

  // ---- build -------------------------------------------------------------------------------------------------------
  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, box) {
        labelW = (box.maxWidth * .26).clamp(168.0, 236.0);
        final frames = math.max(1.0, (box.maxWidth - labelW) / s.pixelsPerFrame), rows = math.max(1.0, (box.maxHeight - rulerH) / rowH);
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) s.viewport(frames: frames, rows: rows);
        });
        final theme = EditorTheme.of(context);
        return DragTarget<Map<String, dynamic>>(
          onWillAcceptWithDetails: (d) => d.data['asset'] != null && c.supports('placeAsset'),
          onMove: (d) {
            final p = (context.findRenderObject() as RenderBox).globalToLocal(d.offset);
            s.aimAsset(row: rowAt(p.dy), frame: p.dx >= labelW ? frameAt(p.dx) : null);
          },
          onLeave: (_) => s.leaveAsset(),
          onAcceptWithDetails: (d) => s.acceptAsset(d.data['asset']),
          builder: (context, _, __) => Focus(
            focusNode: focus,
            onKeyEvent: _key,
            child: GestureDetector(
              supportedDevices: const {PointerDeviceKind.trackpad},
              onScaleStart: (e) {
                motion.stop();
                _navigating = true;
                _navOrigin = e.localFocalPoint;
                final anchor = math.max(0.0, e.localFocalPoint.dx - labelW);
                motion.begin(
                  mode: e.localFocalPoint.dy < rulerH && e.localFocalPoint.dx >= labelW ? ViewportGestureMode.scrubZoom : ViewportGestureMode.pan,
                  scale: s.pixelsPerFrame,
                  frame: s.startFrame + anchor / s.pixelsPerFrame,
                  anchor: anchor,
                  y: s.firstRow * rowH,
                );
              },
              onScaleUpdate: (e) => motion.update(e.localFocalPoint - _navOrigin, e.scale, e.sourceTimeStamp),
              onScaleEnd: (_) {
                _navigating = false;
                motion.end();
              },
              child: Listener(
                onPointerSignal: _wheel,
                child: MouseRegion(
                  cursor: _cursor,
                  onHover: (e) {
                    final c = e.localPosition.dx < labelW ? MouseCursor.defer : switch (_hit(e.localPosition)) {
                      TlBar(part: TlBarPart.start || TlBarPart.end) => SystemMouseCursors.resizeLeftRight,
                      TlBar() || TlLayerKey() || TlLaneKey() => SystemMouseCursors.click,
                      _ => MouseCursor.defer,
                    };
                    if (c != _cursor) setState(() => _cursor = c);
                    final i = rowAt(e.localPosition.dy).floor();
                    final next = e.localPosition.dy < rulerH || i >= s.rows.length ? null : i;
                    if (next != hover) setState(() => hover = next);
                  },
                  onExit: (_) => setState(() => hover = null),
                  child: ClipRect(
                    child: Stack(children: [
                      // rows, names, lanes: redrawn when the Timeline changes (not every frame of playback)
                      Positioned.fill(
                        child: ListenableBuilder(
                          listenable: s,
                          builder: (context, _) => Stack(children: [
                            Positioned.fill(child: CustomPaint(painter: _RowsPainter(this, theme.accent))),
                            ..._outline(),
                            // the lanes: every press is hit-tested here and told to the session
                            Positioned(
                              left: labelW,
                              top: rulerH,
                              right: 0,
                              bottom: 0,
                              child: Listener(
                                behavior: HitTestBehavior.opaque,
                                onPointerDown: (e) => _press(e, _hit(e.localPosition + Offset(labelW, rulerH)), e.localPosition + Offset(labelW, rulerH)),
                                onPointerMove: (e) => _move(e.localPosition + Offset(labelW, rulerH)),
                                onPointerUp: _up,
                                onPointerCancel: (_) => _cancel(),
                                child: GestureDetector(
                                  behavior: HitTestBehavior.opaque,
                                  onSecondaryTapDown: (e) => _menu(e.localPosition + Offset(labelW, rulerH), e.globalPosition),
                                  child: const SizedBox.expand(),
                                ),
                              ),
                            ),
                            // the ruler: press or drag to seek
                            Positioned(
                              left: labelW,
                              top: 0,
                              right: 0,
                              height: rulerH,
                              child: GestureDetector(
                                behavior: HitTestBehavior.opaque,
                                onTapDown: (e) => s.seek(frameAt(labelW + e.localPosition.dx).round()),
                                onHorizontalDragUpdate: (e) => s.seek(frameAt(labelW + e.localPosition.dx).round()),
                                child: const SizedBox.expand(),
                              ),
                            ),
                            for (final m in EditorSession.maps(c.state['markers'])) _marker(m),
                          ]),
                        ),
                      ),
                      // the playhead: its own layer, the only thing a frame of playback repaints
                      Positioned.fill(
                        child: IgnorePointer(
                          child: RepaintBoundary(child: CustomPaint(painter: _HeadPainter(this))),
                        ),
                      ),
                    ]),
                  ),
                ),
              ),
            ),
          ),
        );
      });

  Widget _marker(Map<String, dynamic> m) {
    final id = '${m['id']}';
    final carried = s.markerDrag?.$1 == id ? s.markerDrag!.$2 : (m['frame'] as num).toDouble();
    final x = xOf(carried);
    if (x < labelW - 6) return const SizedBox.shrink();
    return Positioned(
      left: x - 6,
      top: 0,
      width: 12,
      height: rulerH,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onSecondaryTapDown: (e) => _markerMenu(id, e.globalPosition),
        onHorizontalDragUpdate: c.supports('setMarker') ? (e) => s.dragMarker(id, (s.markerDrag?.$1 == id ? s.markerDrag!.$2 : carried) + e.delta.dx / s.pixelsPerFrame) : null,
        onHorizontalDragEnd: c.supports('setMarker') ? (_) => s.dropMarker() : null,
        child: const SizedBox.expand(),
      ),
    );
  }

  /// The name column: per row, switches and twirls that call the session directly, then the name, which picks and
  /// carries the row.
  List<Widget> _outline() {
    final out = <Widget>[];
    final first = s.firstRow.floor();
    for (var i = first; i < s.rows.length; i++) {
      final y = yOf(i);
      if (y > 4000) break;
      final r = s.rows[i];
      final lane = r.property != null, group = r.isGroup && !lane;
      final indent = r.depth * 12.0;
      Widget mark(Widget child, VoidCallback? onTap, {double w = 16}) => SizedBox(
            width: w,
            height: rowH,
            child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: onTap, child: Center(child: child)),
          );
      Widget glyph(HG g, Color color, {bool on = false, double size = 11}) => Container(
            width: 15,
            height: 15,
            decoration: on ? BoxDecoration(color: H.toggleOn, borderRadius: BorderRadius.circular(3)) : null,
            alignment: Alignment.center,
            child: SizedBox.square(dimension: size, child: CustomPaint(painter: HgPainter(g, on ? N.g100 : color, on ? H.toggleOn : N.g10))),
          );
      final quiet = hover == i || c.selectedIds.contains(r.id);
      final name = Expanded(
        child: Listener(
          behavior: HitTestBehavior.opaque,
          onPointerDown: (e) => _press(e, TlRowName(i), Offset(labelW / 2, y + e.localPosition.dy)),
          onPointerMove: (e) => _move(Offset(labelW / 2, y + e.localPosition.dy)),
          onPointerUp: _up,
          onPointerCancel: (_) => _cancel(),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onSecondaryTapDown: (e) => _menu(Offset(0, y + e.localPosition.dy), e.globalPosition),
            child: Align(
              alignment: Alignment.centerLeft,
              child: Text(
                lane ? '${r.property!['label'] ?? r.property!['id']}' : '${r.layer['name'] ?? r.layer['kind']}',
                softWrap: false,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: lane
                    ? sans(11, c: N.g63)
                    : sans(12, c: c.selectedIds.contains(r.id) ? N.g100 : N.g82, w: group || c.selectedIds.contains(r.id) ? FontWeight.w600 : FontWeight.w400),
              ),
            ),
          ),
        ),
      );
      out.add(Positioned(
        left: 0,
        top: y,
        width: labelW,
        height: rowH,
        child: Row(children: [
          const SizedBox(width: 6),
          if (lane) ...[
            SizedBox(width: 48 + 6 + indent + 18),
            name,
            mark(_Diamond(keyed: r.keys.any((k) => (k['frame'] as num).round() == c.frame.value)), () => s.toggleKeyHere(i), w: 24),
          ] else ...[
            mark(glyph(r.layer['hidden'] == true ? HG.eyeOff : HG.eye, r.layer['hidden'] == true ? N.g38 : N.g63, size: 12), () => s.toggleSwitch(i, 'hidden')),
            mark(r.layer['solo'] == true || quiet ? glyph(HG.solo, N.g38, on: r.layer['solo'] == true) : const SizedBox(), () => s.toggleSwitch(i, 'solo')),
            mark(r.layer['locked'] == true || quiet ? glyph(HG.lock, N.g38, on: r.layer['locked'] == true) : const SizedBox(), () => s.toggleSwitch(i, 'locked')),
            SizedBox(width: 6 + indent),
            mark(_Twirl(open: group ? r.groupOpen : r.lanesOpen, strong: group), () => group ? s.toggleFold(i) : s.toggleLanes(i), w: 14),
            SizedBox(width: 18, child: Center(child: _Chip(r))),
            const SizedBox(width: 4),
            name,
            if (group) mark(_Twirl(open: r.lanesOpen, strong: false), () => s.toggleLanes(i), w: 14),
            mark(r.layer['clipToBelow'] == true || quiet ? glyph(HG.crop, N.g38, on: r.layer['clipToBelow'] == true) : const SizedBox(), () => s.toggleSwitch(i, 'clipToBelow'), w: 22),
          ],
          const SizedBox(width: 2),
        ]),
      ));
    }
    return out;
  }
}

class _Twirl extends StatelessWidget {
  const _Twirl({required this.open, required this.strong});
  final bool open, strong;
  @override
  Widget build(BuildContext context) => SizedBox(width: 9, height: 9, child: CustomPaint(painter: _TriPainter(open, strong ? N.g69 : N.g51)));
}

class _TriPainter extends CustomPainter {
  _TriPainter(this.open, this.color);
  final bool open;
  final Color color;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height;
    final p = open ? (Path()..moveTo(0, h * .2)..lineTo(w, h * .2)..lineTo(w / 2, h * .85)..close()) : (Path()..moveTo(w * .2, 0)..lineTo(w * .85, h / 2)..lineTo(w * .2, h)..close());
    cv.drawPath(p, Paint()..color = color);
  }

  @override
  bool shouldRepaint(_TriPainter o) => o.open != open || o.color != color;
}

class _Chip extends StatelessWidget {
  const _Chip(this.r);
  final TrackRow r;
  @override
  Widget build(BuildContext context) {
    final kind = r.layer['kind'];
    if (kind == 'Camera' || r.isGroup) {
      return SizedBox.square(dimension: 12, child: CustomPaint(painter: HgPainter(kind == 'Camera' ? HG.camera : HG.list, N.g69, N.g10)));
    }
    return Container(width: 10, height: 10, decoration: BoxDecoration(color: r.layer['hidden'] == true ? N.g33 : _family(r.id).n, borderRadius: BorderRadius.circular(2.5)));
  }
}

class _Diamond extends StatelessWidget {
  const _Diamond({required this.keyed});
  final bool keyed;
  @override
  Widget build(BuildContext context) => SizedBox.square(dimension: 9, child: CustomPaint(painter: _DiamondPainter(keyed ? N.g95 : null, N.g56)));
}

class _DiamondPainter extends CustomPainter {
  _DiamondPainter(this.fill, this.edge);
  final Color? fill, edge;
  @override
  void paint(Canvas cv, Size s) => _diamond(cv, s.center(Offset.zero), s.width / 2, fill, edge);
  @override
  bool shouldRepaint(_DiamondPainter o) => o.fill != fill || o.edge != edge;
}

void _diamond(Canvas cv, Offset c, double r, Color? fill, Color? edge) {
  final p = Path()..moveTo(c.dx, c.dy - r)..lineTo(c.dx + r, c.dy)..lineTo(c.dx, c.dy + r)..lineTo(c.dx - r, c.dy)..close();
  if (fill != null) cv.drawPath(p, Paint()..color = fill);
  if (edge != null) cv.drawPath(p, Paint()..style = PaintingStyle.stroke..strokeWidth = 1..color = edge);
}

/// Everything but the playhead: ruler, row grounds, bars, keys, spans, ghosts, waveforms, markers, marquee, drop guide.
class _RowsPainter extends CustomPainter {
  _RowsPainter(this.t, this.accent);
  final _LiveTimelineState t;
  final Color accent;

  @override
  void paint(Canvas cv, Size size) {
    final s = t.s, c = t.c;
    final L = t.labelW;
    const top = _LiveTimelineState.rulerH, rh = _LiveTimelineState.rowH;
    final fill = Paint();
    cv.drawRect(Offset.zero & size, fill..color = N.g10);
    // the ruler: the finest step whose labels keep ~70 px apart, minor ticks between; frames when close
    cv.drawRect(Rect.fromLTWH(0, 0, size.width, top), fill..color = N.g07);
    final rate = s.fps;
    final steps = {1, 2, 5, 10, rate ~/ 2, rate, 2 * rate, 5 * rate, 10 * rate, 30 * rate, 60 * rate, 300 * rate}.where((f) => f > 0).toList()..sort();
    final major = steps.firstWhere((f) => f * s.pixelsPerFrame >= 70, orElse: () => steps.last);
    final minor = steps.lastWhere((f) => f < major && major % f == 0 && f * s.pixelsPerFrame >= 8, orElse: () => major);
    final last = t.frameAt(size.width).ceil();
    for (var f = math.max(0, s.startFrame.floor() ~/ minor * minor); f <= last; f += minor) {
      final x = t.xOf(f);
      if (x < L) continue;
      final isMajor = f % major == 0;
      cv.drawRect(Rect.fromLTWH(x.roundToDouble(), isMajor ? top - 8 : top - 4, 1, isMajor ? 8 : 4), fill..color = isMajor ? N.g44 : N.g26);
      if (isMajor) {
        final m = f ~/ (rate * 60), sec = (f ~/ rate) % 60, ff = f % rate;
        final label = major < rate ? '${sec.toString().padLeft(2, '0')}:${ff.toString().padLeft(2, '0')}' : '${m.toString().padLeft(2, '0')}:${sec.toString().padLeft(2, '0')}';
        final tp = TextPainter(text: TextSpan(text: label, style: mono(10, c: N.g63)), textDirection: TextDirection.ltr)..layout();
        tp.paint(cv, Offset(x + 4, 4));
        tp.dispose();
      }
    }
    final picked = s.selectedKeys;
    final held = s.heldTimings;
    final moving = s.gesture == TlGesture.keys ? s.initialKeys : s.settlingKeys;
    final shift = s.gesture == TlGesture.keys ? s.deltaFrames : s.settlingDelta;
    final waves = {for (final w in EditorSession.maps(c.state['waveforms'])) w['layer']: EditorSession.maps(w['columns'])};
    for (var i = s.firstRow.floor(); i < s.rows.length; i++) {
      final y = t.yOf(i);
      if (y > size.height) break;
      final r = s.rows[i];
      final cy = y + rh / 2;
      final selected = c.selectedIds.contains(r.id) && r.property == null;
      cv.drawRect(Rect.fromLTWH(0, y, size.width, rh), fill..color = selected ? N.g15 : (t.hover == i ? N.g13 : (r.property != null ? N.g07 : N.g10)));
      cv.drawRect(Rect.fromLTWH(0, y + rh - 1, size.width, 1), fill..color = N.g13);
      cv.save();
      cv.clipRect(Rect.fromLTWH(L, y, size.width - L, rh));
      double keyX(Map<String, dynamic> k) {
        final f = (k['frame'] as num).toInt();
        final carried = moving.any((m) => tlSameKey(m, k) || (m['layer'] == k['layer'] && m['frame'] == k['frame'] && r.property == null));
        return t.xOf(carried ? f + shift : f);
      }

      bool isPicked(Map<String, dynamic> k) => picked.any((p) => tlSameKey(p, k));
      if (r.property == null) {
        final timing = held[r.id] ?? r.layer;
        final start = (timing['start'] as num? ?? 0).toDouble(), end = start + (timing['duration'] as num? ?? 0).toDouble();
        // quiet until chosen: a wall of full-colour bars out-shouts the work; the chosen layer takes its full colour
        final tone = r.isGroup ? N.g38 : _family(r.id).t;
        final colour = r.layer['hidden'] == true ? N.g20 : (selected ? tone : Color.lerp(N.g10, tone, t.hover == i ? .62 : .5)!);
        final ghost = r.layer['ghost'];
        if (ghost is num && ghost != 0) {
          final a = ghost > 0 ? end : math.max(0.0, start + ghost), b = ghost > 0 ? end + ghost : start;
          cv.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(t.xOf(a), cy - 6, t.xOf(b), cy + 6), const Radius.circular(3)), fill..color = colour.withValues(alpha: .28));
        }
        if (r.layer['kind'] != 'Camera') {
          // however short in time, a bar is drawn at least 3 px, so it can be seen where it is grabbed
          final bar = Rect.fromLTRB(t.xOf(start), cy - 6, math.max(t.xOf(end), t.xOf(start) + 3), cy + 6);
          cv.drawRRect(RRect.fromRectAndRadius(bar, const Radius.circular(3)), fill..color = colour);
        }
        final wave = waves[r.id];
        if (wave != null && wave.isNotEmpty) {
          final p = Path();
          for (final col in wave) {
            final x = t.xOf((col['frame'] as num).toDouble());
            if (x < L || x > size.width) continue;
            final h = math.min(rh / 2 - 2, ((col['max'] as num) - (col['min'] as num)).toDouble() / 2 * 9);
            if (h > .3) p.addRect(Rect.fromLTRB(x, cy - h, x + math.max(1.0, s.pixelsPerFrame * .7), cy + h));
          }
          cv.drawPath(p, fill..color = Color.lerp(H.wave, N.g100, .38)!);
        }
        if (!r.lanesOpen) {
          final byFrame = <int, Map<String, dynamic>>{for (final k in r.allKeys) (k['frame'] as num).toInt(): k};
          for (final k in byFrame.values) {
            final on = r.allKeys.where((a) => a['frame'] == k['frame']).any(isPicked);
            final onBar = r.layer['kind'] != 'Camera';
            _diamond(cv, Offset(keyX(k), cy), on ? 4.6 : 3.6, on ? accent : (onBar && selected ? N.g10 : N.g86), on ? N.g100 : null);
          }
        }
      } else {
        final keys = [...r.keys]..sort((a, b) => (a['frame'] as num).compareTo(b['frame'] as num));
        for (var n = 0; n + 1 < keys.length; n++) {
          final a = keyX(tlKeyOf(r, keys[n])) + 5, b = keyX(tlKeyOf(r, keys[n + 1])) - 5;
          if (b <= a) continue;
          final chosen = isPicked(tlKeyOf(r, keys[n])) && isPicked(tlKeyOf(r, keys[n + 1]));
          final linear = EditorSession.map(keys[n]['interp'])['kind'] == 'Linear';
          final line = Paint()..strokeWidth = chosen ? 1.6 : 1..color = chosen ? accent : N.g51;
          if (linear) {
            for (var x = a; x < b; x += 6) {
              cv.drawLine(Offset(x, cy), Offset(math.min(x + 3, b), cy), line);
            }
          } else {
            cv.drawLine(Offset(a, cy), Offset(b, cy), line);
          }
        }
        for (final k in keys) {
          final key = tlKeyOf(r, k), on = isPicked(key);
          _diamond(cv, Offset(keyX(key), cy), on ? 4.6 : 4, on ? accent : N.g86, on ? N.g100 : null);
        }
      }
      cv.restore();
    }
    // markers
    for (final m in EditorSession.maps(c.state['markers'])) {
      final id = '${m['id']}';
      final x = t.xOf(s.markerDrag?.$1 == id ? s.markerDrag!.$2 : (m['frame'] as num).toDouble());
      if (x < L - 6) continue;
      cv.drawRect(Rect.fromLTWH(x - .5, top, 1, size.height - top), fill..color = H.record.withValues(alpha: .45));
      cv.drawPath(Path()..moveTo(x - 4, top - 10)..lineTo(x + 4, top - 10)..lineTo(x + 4, top - 4)..lineTo(x, top)..lineTo(x - 4, top - 4)..close(), fill..color = H.record);
    }
    cv.drawRect(Rect.fromLTWH(L - 1, 0, 1, size.height), fill..color = N.g15);
    if (s.marquee case final m?) {
      final rect = Rect.fromPoints(Offset(t.xOf(m.f0), top + (m.r0 - s.firstRow) * rh), Offset(t.xOf(m.f1), top + (m.r1 - s.firstRow) * rh));
      cv.drawRect(rect, fill..color = accent.withValues(alpha: .12));
      cv.drawRect(rect, Paint()..style = PaintingStyle.stroke..color = accent);
    }
    if (s.drop case final d?) {
      final y = top + (d.at - s.firstRow) * rh;
      final guide = Paint()..color = H.guide..strokeWidth = 2..style = PaintingStyle.stroke;
      if (d.inside) {
        cv.drawRect(Rect.fromLTWH(1, y + 1, size.width - 2, rh - 2), guide);
      } else {
        cv.drawLine(Offset(d.depth * 12.0 + 60, y), Offset(size.width, y), guide);
      }
    }
  }

  @override
  bool shouldRepaint(_RowsPainter o) => true;
}

/// The playhead alone, repainted by the frame (and a scrub) without rebuilding anything.
class _HeadPainter extends CustomPainter {
  _HeadPainter(this.t) : super(repaint: Listenable.merge([t.c.frame, t.s.scrub, t.s]));
  final _LiveTimelineState t;
  @override
  void paint(Canvas cv, Size size) {
    final x = t.xOf(t.s.scrub.value ?? t.c.frame.value);
    if (x < t.labelW - 1) return;
    final fill = Paint()..color = H.playhead;
    cv.drawRect(Rect.fromLTWH(x - .5, 0, 1, size.height), fill);
    cv.drawPath(Path()..addRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x - 5, 2, 10, 11), const Radius.circular(2)))..moveTo(x - 5, 12)..lineTo(x, 17)..lineTo(x + 5, 12), fill);
  }

  @override
  bool shouldRepaint(_HeadPainter o) => false;
}
