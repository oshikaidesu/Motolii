import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/widgets.dart';

import '../panel_chrome.dart' show GlyphBox;
import '../../theme/glyphs.dart';
import '../../theme/metrics.dart';
import '../../theme/neutral.dart';
import '../item.dart';
import 'library.dart' show materialFace;
import 'model_face.dart';

/// What the preview may ask the owner for: a clip's frame at a time and a still at a larger size.
abstract class FaceService {
  Future<String?> frameAt(BrowserItem item, double seconds, {int edge = 480});
  Future<String?> pictureOf(BrowserItem item);
}

/// Selection Preview: the chosen asset opened where it stands (no window), as a live face and the facts the catalog has.
/// The face is an instrument for what it is: an image zooms and pans, an environment is dragged round, a model is turned,
/// a clip is scrubbed, a sound shows a position. Nothing here decodes media itself: frames and pictures come from the owner.
class MediaPreview extends StatelessWidget {
  const MediaPreview({super.key, required this.item, this.faces, required this.onClose, this.onReveal, this.onPlace});
  final BrowserItem item;
  final FaceService? faces;
  final VoidCallback onClose;
  final VoidCallback? onReveal;
  final VoidCallback? onPlace;

  @override
  Widget build(BuildContext context) => Container(
        decoration: const BoxDecoration(color: N.g10, border: Border(top: BorderSide(color: N.g20))),
        child: LayoutBuilder(builder: (context, box) {
          final side = box.maxWidth >= 520;
          final face = ClipRect(child: _LiveFace(key: ValueKey(item.id), item: item, faces: faces));
          final info = _Info(item: item, onClose: onClose, onReveal: onReveal, onPlace: onPlace);
          return side
              ? Row(children: [Expanded(flex: 3, child: face), SizedBox(width: 220, child: info)])
              : Column(children: [Expanded(child: face), SizedBox(height: 118, child: info)]);
        }),
      );
}

class _Info extends StatelessWidget {
  const _Info({required this.item, required this.onClose, this.onReveal, this.onPlace});
  final BrowserItem item;
  final VoidCallback onClose;
  final VoidCallback? onReveal;
  final VoidCallback? onPlace;
  @override
  Widget build(BuildContext context) {
    Widget chip(String text) => Container(margin: const EdgeInsets.only(right: 4, bottom: 3), padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2.5), decoration: BoxDecoration(color: N.g15, borderRadius: BorderRadius.circular(Surface.controlRadius)), child: Text(text, softWrap: false, style: Dn.label(N.g91, FontWeight.w500)));
    Widget fact(String label, String? value) => value == null || value.isEmpty
        ? const SizedBox.shrink()
        : Padding(padding: const EdgeInsets.only(bottom: 3), child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [SizedBox(width: 48, child: Text(label, style: Dn.label(N.g56))), Expanded(child: Text(value, maxLines: 2, overflow: TextOverflow.ellipsis, style: Dn.value(N.g86)))]));
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 8, 6, 6),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          Expanded(child: Text(item.name, maxLines: 1, overflow: TextOverflow.ellipsis, style: Dn.name(N.g100, FontWeight.w600).copyWith(fontSize: 13.5))),
          GestureDetector(key: const ValueKey('preview-close'), behavior: HitTestBehavior.opaque, onTap: onClose, child: const SizedBox(width: 28, height: 24, child: Center(child: GlyphBox(HG.cross, size: 11, color: N.g76)))),
        ]),
        const SizedBox(height: 4),
        Wrap(children: [
          chip('${item.typeWord} · ${item.mime.split('/').last.toUpperCase()}'),
          if (item.size != null) chip(sizeText(item.size)),
          if (item.seconds != null) chip(clockText(item.seconds)),
          if (item.width != null && item.height != null) chip('${item.width} × ${item.height}'),
        ]),
        SizedBox(height: Surface.inlineGap),
        Expanded(
          child: SingleChildScrollView(
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
              fact('Audio', item.sampleRate == null ? null : '${(item.sampleRate! / 1000).toStringAsFixed(item.sampleRate! % 1000 == 0 ? 0 : 1)} kHz · ${item.channels == 1 ? 'mono' : 'stereo'}'),
              fact('Source', item.source),
              fact('Path', item.rel.isEmpty ? item.path : item.rel),
              fact('Date', dateText(item.mtimeNs)),
              if (onPlace != null)
                GestureDetector(behavior: HitTestBehavior.opaque, onTap: onPlace, child: Padding(padding: const EdgeInsets.only(top: 6), child: Text('Place in project', style: Dn.label(N.g95, FontWeight.w600).copyWith(decoration: TextDecoration.underline)))),
              if (onReveal != null)
                GestureDetector(behavior: HitTestBehavior.opaque, onTap: onReveal, child: Padding(padding: const EdgeInsets.only(top: 4), child: Text('Reveal in Finder', style: Dn.label(N.g82).copyWith(decoration: TextDecoration.underline)))),
            ]),
          ),
        ),
      ]),
    );
  }
}

Uint8List? _bytes(String? dataUri) {
  if (dataUri == null || !dataUri.startsWith('data:')) return null;
  final comma = dataUri.indexOf(',');
  try {
    return comma < 0 ? null : base64Decode(dataUri.substring(comma + 1));
  } catch (_) {
    return null;
  }
}

class _LiveFace extends StatefulWidget {
  const _LiveFace({super.key, required this.item, this.faces});
  final BrowserItem item;
  final FaceService? faces;
  @override
  State<_LiveFace> createState() => _LiveFaceState();
}

class _LiveFaceState extends State<_LiveFace> {
  String? picture; // a larger still (image, environment) or the frame being scrubbed to (clip)
  double at = 0; // the scrub / audition position, 0..1
  bool _busy = false;
  double? _wanted;

  BrowserItem get item => widget.item;

  @override
  void initState() {
    super.initState();
    if (item.kind == 'environment') {
      widget.faces?.pictureOf(item).then((p) {
        if (mounted && p != null) setState(() => picture = p);
      });
    } else if (item.kind == 'video') {
      _scrub(0);
    }
  }

  /// Ask for the frame at a position; while one is being made only the latest wish is kept.
  void _scrub(double f) {
    setState(() => at = f.clamp(0.0, 1.0));
    final faces = widget.faces, seconds = item.seconds;
    if (faces == null || seconds == null) return;
    _wanted = at * seconds;
    if (_busy) return;
    _busy = true;
    () async {
      while (_wanted != null && mounted) {
        final t = _wanted!;
        _wanted = null;
        final frame = await faces.frameAt(item, t);
        if (mounted && frame != null) setState(() => picture = frame);
      }
      _busy = false;
    }();
  }

  @override
  Widget build(BuildContext context) => switch (item.kind) {
        'image' => _zoomable(Image.file(File(item.path), fit: BoxFit.contain, cacheWidth: 1800, gaplessPlayback: true, errorBuilder: (_, __, ___) => materialFace(item.shelf))),
        'environment' => _panorama(),
        'model' => ModelFace(path: item.path, turnable: true, fallback: materialFace(item.shelf)),
        'video' => _clip(),
        'audio' => _sound(),
        _ => materialFace(item.shelf),
      };

  Widget _zoomable(Widget child) => MouseRegion(cursor: SystemMouseCursors.zoomIn, child: InteractiveViewer(minScale: 1, maxScale: 8, child: Center(child: child)));

  /// An environment: the whole panorama, wider than the seat, dragged round.
  Widget _panorama() => LayoutBuilder(builder: (context, box) {
        final bytes = _bytes(picture) ?? _bytes(item.thumbnail);
        final h = box.maxHeight;
        return MouseRegion(
          cursor: SystemMouseCursors.grab,
          child: InteractiveViewer(
            constrained: false,
            minScale: .5,
            maxScale: 3,
            child: SizedBox(width: h * 2, height: h, child: bytes == null ? materialFace(item.shelf) : Image.memory(bytes, fit: BoxFit.fill, gaplessPlayback: true)),
          ),
        );
      });

  Widget _clip() {
    final bytes = _bytes(picture) ?? _bytes(item.thumbnail);
    return Column(children: [
      Expanded(child: Center(child: bytes == null ? materialFace(item.shelf) : Image.memory(bytes, fit: BoxFit.contain, gaplessPlayback: true))),
      _Scrubber(at: at, label: '${clockText(item.seconds == null ? null : at * item.seconds!)} / ${clockText(item.seconds)}', enabled: widget.faces != null && item.seconds != null, onChanged: _scrub),
    ]);
  }

  Widget _sound() => Column(children: [
        Expanded(child: Stack(fit: StackFit.expand, children: [materialFace(item.shelf), Positioned(left: 0, right: 0, top: 0, bottom: 0, child: CustomPaint(painter: _Position(at)))])),
        _Scrubber(at: at, label: '${clockText(item.seconds == null ? null : at * item.seconds!)} / ${clockText(item.seconds)}', enabled: true, onChanged: (f) => setState(() => at = f)),
      ]);
}

class _Position extends CustomPainter {
  const _Position(this.at);
  final double at;
  @override
  void paint(Canvas c, Size s) => c.drawRect(Rect.fromLTWH(s.width * at - .5, 0, 1.5, s.height), Paint()..color = N.g100);
  @override
  bool shouldRepaint(_Position o) => o.at != at;
}

/// A thin position bar under a clip or a sound: drag to move (a position, not a play head: nothing plays).
class _Scrubber extends StatelessWidget {
  const _Scrubber({required this.at, required this.label, required this.enabled, required this.onChanged});
  final double at;
  final String label;
  final bool enabled;
  final ValueChanged<double> onChanged;
  @override
  Widget build(BuildContext context) => Container(
        height: 22,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        child: Row(children: [
          Expanded(
            child: LayoutBuilder(builder: (context, box) {
              void set(Offset p) => enabled ? onChanged((p.dx / box.maxWidth).clamp(0.0, 1.0)) : null;
              return GestureDetector(
                key: const ValueKey('scrubber'),
                behavior: HitTestBehavior.opaque,
                onPanDown: (d) => set(d.localPosition),
                onPanUpdate: (d) => set(d.localPosition),
                child: CustomPaint(size: Size(box.maxWidth, 22), painter: _Bar(at, enabled)),
              );
            }),
          ),
          const SizedBox(width: 8),
          Text(label, style: Dn.value(N.g69).copyWith(fontSize: Dn.labelSize)),
        ]),
      );
}

class _Bar extends CustomPainter {
  const _Bar(this.at, this.enabled);
  final double at;
  final bool enabled;
  @override
  void paint(Canvas c, Size s) {
    final y = s.height / 2;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(0, y - 1.5, s.width, 3), const Radius.circular(1.5)), Paint()..color = N.g20);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(0, y - 1.5, s.width * at, 3), const Radius.circular(1.5)), Paint()..color = enabled ? N.g76 : N.g38);
    c.drawCircle(Offset(s.width * at, y), 4.5, Paint()..color = enabled ? N.g95 : N.g44);
  }

  @override
  bool shouldRepaint(_Bar o) => o.at != at || o.enabled != enabled;
}
