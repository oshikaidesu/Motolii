// The Camera Instrument: the view ray as one face, with the exact numbers, the lens, and the way to Depth beside it.
// Face: target point, eye on its orbit, ray with a distance handle, roll ring, all live at once (no modes).
// Zoom is a different tool from Distance and is shown as one: they meet only in the magnification at the target plane.
import 'package:flutter/widgets.dart';
import '../bp/common.dart';
import '../desk/common.dart' show kYellow, kInk;
import 'camera_face.dart';
import 'camera_model.dart';
import 'rows.dart';
import 'toys.dart';
import '../neutral.dart';

class CameraInstrument extends StatelessWidget {
  const CameraInstrument(this.store, {super.key, this.title = 'Camera', this.animating, this.onAnimate});
  final CameraStore store;
  final String title;

  /// Animate (auto-key), as the Transform header shows it; absent when the host has no such mode.
  final bool? animating;
  final VoidCallback? onAnimate;
  CameraStore get s => store;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: s,
        builder: (context, _) => LayoutBuilder(builder: (context, box) {
          final narrow = box.maxWidth < 230;
          final pad = narrow ? 8.0 : 12.0;
          final w = box.maxWidth - pad * 2;
          Widget val(String id, Color tone, {String? tag, int? axis, bool gated = true, bool units = true, int? decimals}) => Expanded(
                child: Opacity(opacity: gated ? 1 : .5, child: IgnorePointer(ignoring: !gated, child: ValueToy(Slot(s, id, axis), tag: tag, tone: tone, showUnit: units && !narrow, decimals: decimals))),
              );
          const gap = SizedBox(width: 3);
          final free = !s.targetLocked;
          return SingleChildScrollView(
            key: const ValueKey('camera-scroll'),
            padding: EdgeInsets.fromLTRB(pad, 10, pad, 16),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _header(),
              const SizedBox(height: 8),
              CameraFace(s, size: Size(w, narrow ? 148 : 176)),
              const SizedBox(height: 10),
              // the target: one point, however many rows declare it; a Target layer overrides X, Y and Z
              _label('TARGET', targetColor, s.targetLocked ? 'set by the layer' : null, 'target', ['camera.center', 'camera.target.z']),
              narrow
                  ? Column(children: [Row(children: [val('camera.center', targetColor, tag: 'X', axis: 0, gated: free, decimals: 0), gap, val('camera.center', targetColor, tag: 'Y', axis: 1, gated: free, decimals: 0)]), const SizedBox(height: 3), Row(children: [val('camera.target.z', targetColor, tag: 'Z', gated: free, decimals: 0)])])
                  : Row(children: [val('camera.center', targetColor, tag: 'X', axis: 0, gated: free, decimals: 0), gap, val('camera.center', targetColor, tag: 'Y', axis: 1, gated: free, decimals: 0), gap, val('camera.target.z', targetColor, tag: 'Z', gated: free, decimals: 0)]),
              const SizedBox(height: 3),
              ChoiceToy(s, 'camera.target', tone: targetColor),
              _label('ORBIT', orbitColor, null, 'orbit', ['camera.orbit']),
              Row(children: [val('camera.orbit', orbitColor, tag: narrow ? 'P' : 'Pitch', axis: 0, decimals: 0), gap, val('camera.orbit', orbitColor, tag: narrow ? 'Y' : 'Yaw', axis: 1, decimals: 0)]),
              _label('FRAMING', distanceColor, 'one magnification, two tools', 'framing', ['camera.distance', 'camera.zoom']),
              // Distance moves the eye (perspective changes); Zoom changes the lens (it does not)
              Row(children: [val('camera.distance', distanceColor, tag: narrow ? 'D' : 'Distance', units: false), gap, val('camera.zoom', kYellow, tag: narrow ? 'Z' : 'Zoom', units: false)]),
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text('Frames the target at ×${s.magnification.toStringAsFixed(2)}', key: const ValueKey('magnification'), style: sans(9.5, c: kMuted)),
              ),
              _label('ROLL', rollColor, null, 'roll', ['camera.roll']),
              Row(children: [val('camera.roll', rollColor, decimals: null)]),
            ]),
          );
        }),
      );

  Widget _header() => SizedBox(
        height: 24,
        child: Row(children: [
          Container(width: 3, height: 14, margin: const EdgeInsets.only(right: 7), decoration: BoxDecoration(color: orbitColor, borderRadius: BorderRadius.circular(1.5))),
          Expanded(child: Text(title, key: const ValueKey('camera-title'), softWrap: false, overflow: TextOverflow.clip, style: sans(13, c: kInk, w: FontWeight.w600))),
          if (s.frozen) Padding(padding: const EdgeInsets.only(right: 8), child: Text('Locked', style: sans(9.5, c: kMuted, w: FontWeight.w600))),
          if (animating != null)
            GestureDetector(
              key: const ValueKey('camera-animate'),
              behavior: HitTestBehavior.opaque,
              onTap: onAnimate,
              child: Padding(
                padding: const EdgeInsets.only(right: 10),
                child: Text('Animate', style: sans(10.5, c: animating! ? kYellow : kMuted, w: FontWeight.w600)),
              ),
            ),
          GestureDetector(
            key: const ValueKey('route-depth'),
            behavior: HitTestBehavior.opaque,
            onTap: () => s.route('Depth', 'camera'),
            child: Container(height: 22, padding: const EdgeInsets.symmetric(horizontal: 9), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: orbitColor.withValues(alpha: .7)), borderRadius: BorderRadius.circular(11)), child: Text('Depth →', style: sans(9.5, c: orbitColor, w: FontWeight.w700))),
          ),
        ]),
      );

  bool _modified(List<String> ids) => ids.any((i) => modified(s.row(i)));

  // each group carries its key lamp and reset, as Classic's wells do
  Widget _label(String t, Color tone, String? note, String name, List<String> ids) {
    final keyed = ids.any((i) => s.keyedNow.contains(i)), animated = ids.any((i) => s.animated.contains(i));
    return Padding(
      padding: const EdgeInsets.only(top: 12, bottom: 5),
      child: Row(children: [
        Container(width: 3, height: 9, margin: const EdgeInsets.only(right: 6), decoration: BoxDecoration(color: tone, borderRadius: BorderRadius.circular(1.5))),
        Text(t, style: sans(9, c: N.g51, w: FontWeight.w600, ls: 1.3)),
        Expanded(child: note == null ? const SizedBox.shrink() : Padding(padding: const EdgeInsets.only(left: 8), child: Text(note, softWrap: false, overflow: TextOverflow.clip, style: sans(9, c: N.g38)))),
        GestureDetector(key: ValueKey('key-$name'), behavior: HitTestBehavior.opaque, onTap: () => s.toggleKeys(ids), child: Padding(padding: const EdgeInsets.symmetric(horizontal: 5), child: SizedBox(width: 9, height: 9, child: CustomPaint(painter: _DiamondP(keyed, keyed || animated ? tone : N.g26))))),
        if (_modified(ids) && !s.frozen) GestureDetector(key: ValueKey('reset-$name'), behavior: HitTestBehavior.opaque, onTap: () => s.resetMany(ids), child: Padding(padding: const EdgeInsets.only(left: 3), child: Text('↺', style: sans(11, c: N.g38)))) else const SizedBox(width: 14),
      ]),
    );
  }
}

class _DiamondP extends CustomPainter {
  _DiamondP(this.filled, this.col);
  final bool filled;
  final Color col;
  @override
  void paint(Canvas c, Size s) {
    final p = Path()..moveTo(s.width / 2, 0)..lineTo(s.width, s.height / 2)..lineTo(s.width / 2, s.height)..lineTo(0, s.height / 2)..close();
    c.drawPath(p, Paint()..color = col..style = filled ? PaintingStyle.fill : PaintingStyle.stroke..strokeWidth = 1.3);
  }
  @override
  bool shouldRepaint(_DiamondP o) => o.filled != filled || o.col != col;
}
