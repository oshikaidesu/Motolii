// Prototype: Explore. There is no similarity backend, so this places the chosen asset in the middle and the rest around it
// by how near their own mean colour is (measured here from their pictures; sounds and models, which have none, stand in the
// outer ring). It is a projection of the same Result Set: choosing another asset re-centres the space on it. Nothing
// here is a capability of the product; it is the interaction, to be judged.
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/hf/metrics.dart';
import 'package:motolii_stage5/hf/neutral.dart';
import 'package:motolii_stage5/live_hf/adapters/browser_item.dart';
import 'package:motolii_stage5/live_hf/adapters/media_library.dart' show materialFace;

final _colours = <String, HSLColor?>{};
final _asked = <String>{};

Future<HSLColor?> _measure(BrowserItem it) async {
  Uint8List? bytes;
  final t = it.thumbnail;
  if (t != null && t.startsWith('data:')) {
    bytes = base64Decode(t.substring(t.indexOf(',') + 1));
  } else if (it.kind == 'image') {
    try {
      bytes = await File(it.path).readAsBytes();
    } catch (_) {}
  }
  if (bytes == null) return null;
  try {
    final codec = await ui.instantiateImageCodec(bytes, targetWidth: 16);
    final image = (await codec.getNextFrame()).image;
    final data = await image.toByteData(format: ui.ImageByteFormat.rawRgba);
    if (data == null) return null;
    var r = 0.0, g = 0.0, b = 0.0, n = 0;
    for (var i = 0; i + 3 < data.lengthInBytes; i += 4) {
      if (data.getUint8(i + 3) < 32) continue;
      r += data.getUint8(i);
      g += data.getUint8(i + 1);
      b += data.getUint8(i + 2);
      n++;
    }
    return n == 0 ? null : HSLColor.fromColor(Color.fromARGB(255, (r / n).round(), (g / n).round(), (b / n).round()));
  } catch (_) {
    return null;
  }
}

double _distance(HSLColor a, HSLColor b) {
  var dh = (a.hue - b.hue).abs();
  dh = math.min(dh, 360 - dh) / 180;
  final w = math.min(a.saturation, b.saturation); // a grey has no hue to speak of
  return math.sqrt(math.pow(dh * w * 1.6, 2) + math.pow(a.lightness - b.lightness, 2) + math.pow((a.saturation - b.saturation) * .6, 2));
}

Widget exploreView(BuildContext context, List<BrowserItem> items, String? selected, ValueChanged<String> onTap, ValueChanged<String>? onOpen) => _Explore(items: items, selected: selected, onTap: onTap, onOpen: onOpen);

class _Explore extends StatefulWidget {
  const _Explore({required this.items, required this.selected, required this.onTap, this.onOpen});
  final List<BrowserItem> items;
  final String? selected;
  final ValueChanged<String> onTap;
  final ValueChanged<String>? onOpen;
  @override
  State<_Explore> createState() => _ExploreState();
}

class _ExploreState extends State<_Explore> {
  @override
  void initState() {
    super.initState();
    _measureAll();
  }

  @override
  void didUpdateWidget(_Explore old) {
    super.didUpdateWidget(old);
    _measureAll();
  }

  void _measureAll() {
    for (final it in widget.items) {
      final key = '${it.id}:${it.thumbnail?.length}';
      if (_colours.containsKey(it.id) && _colours[it.id] != null) continue;
      if (!_asked.add(key)) continue;
      _measure(it).then((c) {
        if (c != null) {
          _colours[it.id] = c;
          if (mounted) setState(() {});
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final items = widget.items;
    final centre = items.firstWhere((i) => i.id == widget.selected, orElse: () => items.first);
    final around = [for (final i in items) if (i.id != centre.id) i];
    final c0 = _colours[centre.id];
    // near by colour first; what has no colour of its own goes last
    double rank(BrowserItem i) => (c0 == null || _colours[i.id] == null) ? 9 : _distance(c0, _colours[i.id]!);
    around.sort((a, b) => rank(a).compareTo(rank(b)));
    const node = 46.0, step = 62.0, first = 62.0;
    final placed = <String, Offset>{};
    var ring = 0, slot = 0, index = 0;
    List<BrowserItem> take(int n) {
      final out = around.skip(index).take(n).toList();
      index += out.length;
      return out;
    }

    while (index < around.length) {
      final capacity = 6 + ring * 6;
      final members = take(capacity)..sort((a, b) => ((_colours[a.id]?.hue ?? 0)).compareTo(_colours[b.id]?.hue ?? 0));
      final radius = first + ring * step;
      for (final (k, m) in members.indexed) {
        final angle = -math.pi / 2 + (k + (ring.isOdd ? .5 : 0)) * 2 * math.pi / members.length;
        placed[m.id] = Offset(math.cos(angle) * radius, math.sin(angle) * radius);
      }
      ring++;
      slot++;
    }
    final extent = (first + math.max(0, ring - 1) * step + node) * 2 + 24;
    return LayoutBuilder(builder: (context, box) {
      final size = math.max(extent, math.min(box.maxWidth, box.maxHeight));
      return InteractiveViewer(
        constrained: false,
        minScale: .4,
        maxScale: 2.5,
        boundaryMargin: const EdgeInsets.all(120),
        child: SizedBox(
          width: math.max(size, box.maxWidth),
          height: math.max(size, box.maxHeight),
          child: Stack(children: [
            for (final i in items)
              AnimatedPositioned(
                key: ValueKey('explore-${i.id}'),
                duration: const Duration(milliseconds: 380),
                curve: Curves.easeOutCubic,
                left: math.max(size, box.maxWidth) / 2 + (i.id == centre.id ? 0 : placed[i.id]!.dx) - _w(i, i.id == centre.id) / 2,
                top: math.max(size, box.maxHeight) / 2 + (i.id == centre.id ? 0 : placed[i.id]!.dy) - _h(i, i.id == centre.id) / 2,
                width: _w(i, i.id == centre.id),
                height: _h(i, i.id == centre.id),
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: () => widget.onTap(i.id),
                  onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(i.id),
                  child: Container(
                    decoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: i.id == centre.id ? N.g95 : N.glaze15, width: i.id == centre.id ? 2 : 1)),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(3),
                      child: Stack(fit: StackFit.expand, children: [
                        ColoredBox(color: N.g13, child: materialFace(i.shelf)),
                        if (i.mark.isNotEmpty) Positioned(left: 2, top: 2, child: Container(padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 1), decoration: BoxDecoration(color: N.veil, borderRadius: BorderRadius.circular(2)), child: Text(i.mark, style: Dn.micro(N.g95).copyWith(fontSize: 8)))),
                      ]),
                    ),
                  ),
                ),
              ),
          ]),
        ),
      );
    });
  }

  double _w(BrowserItem i, bool centre) {
    final base = centre ? 96.0 : 46.0;
    final a = i.aspect;
    return a >= 1 ? base : base * a;
  }

  double _h(BrowserItem i, bool centre) {
    final base = centre ? 96.0 : 46.0;
    final a = i.aspect;
    return a >= 1 ? base / a : base;
  }
}
