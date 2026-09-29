// TEMPORARY — the Skin Swap Proof for the Stage. A deliberately different skin over the same StageSession for the
// Camera view: no cage handles, selected layers drawn as yellow outlines, a text toolbar; a press picks or starts a
// marquee, a drag moves, the wheel zooms. If this works with no change to StageSession, the Stage's meaning lives
// there. Delete after the proof.
import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show mono;
import '../../hf/neutral.dart';
import '../../panels/stage_session.dart';
import '../../session/editor_session.dart';

final altStageSkin = ValueNotifier(false);

class AltStage extends StatefulWidget {
  const AltStage({super.key, required this.c});
  final EditorSession c;
  @override
  State<AltStage> createState() => _AltStageState();
}

class _AltStageState extends State<AltStage> {
  late final s = StageSession.of(widget.c, 'Camera');
  Size tab = Size.zero;
  Offset? _down;
  bool _moved = false;

  @override
  void initState() {
    super.initState();
    widget.c.attachView('Camera');
  }

  StMods get _mods {
    final k = HardwareKeyboard.instance;
    return StMods(add: k.isShiftPressed || k.isMetaPressed, shift: k.isShiftPressed, alt: k.isAltPressed, cmd: k.isMetaPressed);
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g00,
        child: Column(children: [
          SizedBox(
            height: 22,
            child: ListenableBuilder(
              listenable: s,
              builder: (context, _) => Row(children: [
                GestureDetector(key: const ValueKey('alt-fit'), onTap: s.fit, child: Text(' [fit] ', style: mono(10, c: N.g95))),
                GestureDetector(key: const ValueKey('alt-actual'), onTap: s.actual, child: Text(' [1:1] ', style: mono(10, c: N.g95))),
                Text('  ${(s.scaleIn(tab) * 100).round()}%', style: mono(10, c: N.g63)),
              ]),
            ),
          ),
          Expanded(
            child: LayoutBuilder(builder: (context, box) {
              tab = box.biggest;
              return Listener(
                behavior: HitTestBehavior.opaque,
                onPointerSignal: (e) {
                  if (e is PointerScrollEvent) s.zoomAt(s.scaleIn(tab) * math.exp(-e.scrollDelta.dy * .0015), e.localPosition, tab);
                },
                onPointerDown: (e) {
                  if (e.buttons != kPrimaryMouseButton || s.busy) return;
                  _down = e.localPosition;
                  _moved = false;
                  final comp = s.toComp(e.localPosition, tab);
                  final layer = s.layerAt(comp);
                  s.press(layer == null ? const StEmpty() : StLayer((layer['id'] as num).toInt()), comp, _mods);
                },
                onPointerMove: (e) {
                  if (_down == null) return;
                  _moved = _moved || (e.localPosition - _down!).distance > 3;
                  s.drag(s.toComp(e.localPosition, tab), beyondSlop: _moved, mods: _mods, viewScale: s.scaleIn(tab));
                },
                onPointerUp: (e) {
                  if (_down == null) return;
                  _down = null;
                  s.release(s.toComp(e.localPosition, tab), _mods, s.scaleIn(tab));
                },
                child: ListenableBuilder(
                  listenable: Listenable.merge([s, widget.c.textureIds, widget.c.rendered]),
                  builder: (context, _) {
                    final origin = s.originIn(tab), scale = s.scaleIn(tab);
                    final id = widget.c.textureIds.value['Camera'];
                    return Stack(children: [
                      if (id != null) Positioned(left: origin.dx, top: origin.dy, width: s.width * scale, height: s.height * scale, child: Texture(textureId: id)),
                      Positioned.fill(child: IgnorePointer(child: CustomPaint(painter: _AltPainter(s, tab, widget.c.selectedIds.toSet())))),
                    ]);
                  },
                ),
              );
            }),
          ),
        ]),
      );
}

class _AltPainter extends CustomPainter {
  _AltPainter(this.s, this.tab, this.picked);
  final StageSession s;
  final Size tab;
  final Set<int> picked;
  @override
  void paint(Canvas cv, Size size) {
    final ink = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5
      ..color = const Color(0xFFFFE14D);
    for (final l in s.visible) {
      if (!picked.contains(l['id'])) continue;
      final pts = s.corners(l).map((p) => s.toScreen(p, tab)).toList();
      if (pts.length > 2) cv.drawPath(Path()..addPolygon(pts, true), ink);
    }
    if (s.marquee case final m?) cv.drawRect(Rect.fromPoints(s.toScreen(m.topLeft, tab), s.toScreen(m.bottomRight, tab)), ink);
  }

  @override
  bool shouldRepaint(_AltPainter o) => true;
}
