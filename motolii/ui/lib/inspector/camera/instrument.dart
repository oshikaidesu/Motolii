// The Camera Instrument: the view ray as one face, with the exact numbers, the lens, and the way to Depth beside it.
// Face: target point, eye on its orbit, ray with a distance handle, roll ring, all live at once (no modes).
// Zoom is a different tool from Distance and is shown as one: they meet only in the magnification at the target plane.
import 'package:flutter/widgets.dart';
import '../../browser/parts.dart';
import '../../desks/parts.dart' show kYellow;
import 'face.dart';
import 'model.dart';
import '../rows.dart';
import '../value_controls.dart';
import '../../hf/neutral.dart';
import '../../hf/metrics.dart' show Dn, Surface;

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
          final narrow = box.maxWidth < 172;
          final pad = narrow ? 6.0 : 9.0;
          final w = box.maxWidth - pad * 2;
          Widget val(String id, Color tone, {String? tag, int? axis, bool gated = true, bool units = true, int? decimals}) => Expanded(
                child: Opacity(opacity: gated ? 1 : .5, child: IgnorePointer(ignoring: !gated, child: ValueToy(Slot(s, id, axis), tag: tag, tone: tone, showUnit: units && !narrow, decimals: decimals))),
              );
          const gap = SizedBox(width: 2);
          final free = !s.targetLocked;
          return SingleChildScrollView(
            key: const ValueKey('camera-scroll'),
            padding: EdgeInsets.fromLTRB(pad, 7.5, pad, 12),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              _header(),
              const SizedBox(height: 6),
              CameraFace(s, size: Size(w, narrow ? 111 : 132)),
              const SizedBox(height: 7.5),
              // the target: one point, however many rows declare it; a Target layer overrides X, Y and Z
              _label('TARGET', targetColor, s.targetLocked ? 'set by the layer' : null, 'target', ['camera.center', 'camera.target.z']),
              narrow
                  ? Column(children: [Row(children: [val('camera.center', targetColor, tag: 'X', axis: 0, gated: free, decimals: 0), gap, val('camera.center', targetColor, tag: 'Y', axis: 1, gated: free, decimals: 0)]), const SizedBox(height: 2), Row(children: [val('camera.target.z', targetColor, tag: 'Z', gated: free, decimals: 0)])])
                  : Row(children: [val('camera.center', targetColor, tag: 'X', axis: 0, gated: free, decimals: 0), gap, val('camera.center', targetColor, tag: 'Y', axis: 1, gated: free, decimals: 0), gap, val('camera.target.z', targetColor, tag: 'Z', gated: free, decimals: 0)]),
              const SizedBox(height: 2),
              ChoiceToy(s, 'camera.target', tone: targetColor),
              _label('ORBIT', orbitColor, null, 'orbit', ['camera.orbit']),
              Row(children: [val('camera.orbit', orbitColor, tag: narrow ? 'P' : 'Pitch', axis: 0, decimals: 0), gap, val('camera.orbit', orbitColor, tag: narrow ? 'Y' : 'Yaw', axis: 1, decimals: 0)]),
              _label('FRAMING', distanceColor, 'one magnification, two tools', 'framing', ['camera.distance', 'camera.zoom']),
              // Distance moves the eye (perspective changes); Zoom changes the lens (it does not)
              Row(children: [val('camera.distance', distanceColor, tag: narrow ? 'D' : 'Distance', units: false), gap, val('camera.zoom', kYellow, tag: narrow ? 'Z' : 'Zoom', units: false)]),
              Padding(
                padding: const EdgeInsets.only(top: 3),
                child: Text('Frames the target at ×${s.magnification.toStringAsFixed(2)}', key: const ValueKey('magnification'), style: sans(Dn.microSize, c: Surface.muted)),
              ),
              _label('ROLL', rollColor, null, 'roll', ['camera.roll']),
              Row(children: [val('camera.roll', rollColor, decimals: null)]),
            ]),
          );
        }),
      );

  Widget _header() => SizedBox(
        height: 18,
        child: Row(children: [
          Container(width: 2, height: 10.5, margin: const EdgeInsets.only(right: 5), decoration: BoxDecoration(color: orbitColor, borderRadius: BorderRadius.circular(1.5))),
          Expanded(child: Text(title, key: const ValueKey('camera-title'), softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.nameSize, c: Surface.ink, w: FontWeight.w600))),
          if (s.frozen) Padding(padding: const EdgeInsets.only(right: 6), child: Text('Locked', style: sans(Dn.microSize, c: Surface.muted, w: FontWeight.w600))),
          if (animating != null)
            GestureDetector(
              key: const ValueKey('camera-animate'),
              behavior: HitTestBehavior.opaque,
              onTap: onAnimate,
              child: Padding(
                padding: const EdgeInsets.only(right: 7.5),
                child: Text('Animate', style: sans(Dn.labelSize, c: animating! ? kYellow : Surface.muted, w: FontWeight.w600)),
              ),
            ),
          GestureDetector(
            key: const ValueKey('route-depth'),
            behavior: HitTestBehavior.opaque,
            onTap: () => s.route('Depth', 'camera'),
            child: Container(height: 16.5, padding: const EdgeInsets.symmetric(horizontal: 7), alignment: Alignment.center, decoration: BoxDecoration(border: Border.all(color: orbitColor.withValues(alpha: .7)), borderRadius: BorderRadius.circular(8)), child: Text('Depth →', style: sans(Dn.microSize, c: orbitColor, w: FontWeight.w700))),
          ),
        ]),
      );

  bool _modified(List<String> ids) => ids.any((i) => modified(s.row(i)));

  // each group carries its key lamp and reset, as Classic's wells do
  Widget _label(String t, Color tone, String? note, String name, List<String> ids) {
    final keyed = ids.any((i) => s.keyedNow.contains(i)), animated = ids.any((i) => s.animated.contains(i));
    return Padding(
      padding: const EdgeInsets.only(top: 9, bottom: 4),
      child: Row(children: [
        Container(width: 2, height: 7, margin: const EdgeInsets.only(right: 4.5), decoration: BoxDecoration(color: tone, borderRadius: BorderRadius.circular(1.5))),
        Text(t, style: sans(Dn.microSize, c: N.g51, w: FontWeight.w600, ls: 1.3)),
        Expanded(child: note == null ? const SizedBox.shrink() : Padding(padding: const EdgeInsets.only(left: 6), child: Text(note, softWrap: false, overflow: TextOverflow.clip, style: sans(Dn.microSize, c: N.g38)))),
        GestureDetector(key: ValueKey('key-$name'), behavior: HitTestBehavior.opaque, onTap: () => s.toggleKeys(ids), child: Padding(padding: const EdgeInsets.symmetric(horizontal: 4), child: SizedBox(width: 7, height: 7, child: CustomPaint(painter: _DiamondP(keyed, keyed || animated ? tone : N.g26))))),
        if (_modified(ids) && !s.frozen && !ids.every(s.held)) GestureDetector(key: ValueKey('reset-$name'), behavior: HitTestBehavior.opaque, onTap: () => s.resetMany(ids), child: Padding(padding: const EdgeInsets.only(left: 2), child: Text('↺', style: sans(Dn.nameSize, c: N.g38)))) else const SizedBox(width: 10.5),
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
