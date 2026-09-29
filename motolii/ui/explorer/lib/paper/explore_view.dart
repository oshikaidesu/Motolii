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
import 'package:motolii_stage5/live_hf/adapters/media_fluid.dart' show Frame;
import 'package:motolii_stage5/live_hf/adapters/media_library.dart' show materialFace;

final _colours = <String, HSLColor?>{};
final _asked = <String>{};

/// Bumped when a colour has been measured: the places change by themselves.
final exploreChanged = ValueNotifier<int>(0);

void _wantColours(List<BrowserItem> items) {
  for (final it in items) {
    if (_colours[it.id] != null) continue;
    if (!_asked.add('${it.id}:${it.thumbnail?.length}')) continue;
    _measure(it).then((c) {
      if (c != null) {
        _colours[it.id] = c;
        exploreChanged.value++;
      }
    });
  }
}

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

/// The chosen asset in the middle, the rest in rings round it, nearest colour first (what has no colour goes outermost).
/// The rings stay inside the seat's width (a face never falls off the side); what does not fit on a ring stands in rows
/// below the last one, so the space grows downward, where the seat scrolls.
Frame exploreLayout(List<BrowserItem> items, String? selected, Size viewport) {
  _wantColours(items);
  final centre = items.firstWhere((i) => i.id == selected, orElse: () => items.first);
  final around = [for (final i in items) if (i.id != centre.id) i];
  final c0 = _colours[centre.id];
  double rank(BrowserItem i) => (c0 == null || _colours[i.id] == null) ? 9 : _distance(c0, _colours[i.id]!);
  around.sort((a, b) => rank(a).compareTo(rank(b)));
  // faces as big as the seat lets them be: 52 px for a few dozen, less for many
  final halfW = viewport.width / 2;
  final big = (halfW * .6).clamp(72.0, 120.0).toDouble();
  const gap = 4.0, margin = 8.0;
  // every asset gets a place in the field: try the largest faces first, and smaller ones until all fit inside the seat's width
  final widest = math.min(around.length <= 40 ? 52.0 : (around.length <= 90 ? 40.0 : 30.0), math.max(30.0, halfW / 3.3));
  var node = widest;
  late List<List<BrowserItem>> rings;
  late List<double> radii;
  var index = 0;
  for (;; node -= 3) {
    rings = <List<BrowserItem>>[];
    radii = <double>[];
    index = 0;
    final r1 = big / 2 + node / 2 + 8 + 14; // 14: the chosen asset's name under it
    for (var k = 0;; k++) {
      final r = r1 + k * (node + gap + 4);
      if (r + node / 2 + margin > halfW || index >= around.length) break;
      final cap = math.max(4, (2 * math.pi * r / (node + gap)).floor());
      rings.add(around.skip(index).take(cap).toList());
      radii.add(r);
      index += rings.last.length;
    }
    if (index >= around.length || node - 3 < 30) break;
  }
  final reach = radii.isEmpty ? big / 2 : radii.last + node / 2;
  final rest0 = around.length - index;
  // with nothing left over, the rings are centred in the seat; else they sit at the top and the rest stands under them
  final cx = viewport.width / 2, cy = rest0 == 0 ? math.max(margin + reach, viewport.height / 2) : margin + reach;
  final faces = <String, Rect>{};
  Rect at(BrowserItem i, Offset c, double base) {
    final a = i.aspect;
    return Rect.fromCenter(center: c, width: a >= 1 ? base : base * a, height: a >= 1 ? base / a : base);
  }

  faces[centre.id] = at(centre, Offset(cx, cy), big);
  for (final (k, members) in rings.indexed) {
    final sorted = [...members]..sort((a, b) => (_colours[a.id]?.hue ?? 0).compareTo(_colours[b.id]?.hue ?? 0));
    for (final (n, m) in sorted.indexed) {
      final angle = -math.pi / 2 + (n + (k.isOdd ? .5 : 0)) * 2 * math.pi / sorted.length;
      faces[m.id] = at(m, Offset(cx + math.cos(angle) * radii[k], cy + math.sin(angle) * radii[k]), node);
    }
  }
  // the rest, in rows under the rings
  final rest = around.skip(index).toList();
  final cols = math.max(1, ((viewport.width - margin * 2 + gap) / (node + gap)).floor());
  final top = cy + reach + node / 2 + 14;
  for (final (n, m) in rest.indexed) {
    faces[m.id] = at(m, Offset(margin + (n % cols) * (node + gap) + node / 2, top + (n ~/ cols) * (node + gap) + node / 2), node);
  }
  final bottom = rest.isEmpty ? cy + reach + margin : top + ((rest.length - 1) ~/ cols + 1) * (node + gap) + margin;
  return (faces: faces, labels: {centre.id: Rect.fromLTWH(cx - big / 2, cy + big / 2 + 1, big, 14)}, links: _nearest(items, centre.id, 5), graph: null, content: Size(viewport.width, math.max(viewport.height, bottom)));
}

/// The few nearest by colour (only those that have a colour to compare), for the spokes drawn from the centre.
List<String> _nearest(List<BrowserItem> items, String? selected, int n) {
  final centre = items.firstWhere((i) => i.id == selected, orElse: () => items.first);
  final c0 = _colours[centre.id];
  if (c0 == null) return const [];
  final r = [for (final i in items) if (i.id != centre.id && _colours[i.id] != null) (i.id, _distance(c0, _colours[i.id]!))]..sort((a, b) => a.$2.compareTo(b.$2));
  return [for (final e in r.take(n)) e.$1];
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
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: exploreChanged,
        builder: (context, _) => LayoutBuilder(builder: (context, box) {
          final frame = exploreLayout(widget.items, widget.selected, Size(box.maxWidth, box.maxHeight));
          final chosen = widget.items.firstWhere((i) => i.id == widget.selected, orElse: () => widget.items.first).id;
          return SingleChildScrollView(
            child: SizedBox(
              width: frame.content.width,
              height: frame.content.height,
              child: Stack(children: [
                for (final i in widget.items)
                  AnimatedPositioned(
                    key: ValueKey('explore-${i.id}'),
                    duration: const Duration(milliseconds: 380),
                    curve: Curves.easeOutCubic,
                    left: frame.faces[i.id]!.left,
                    top: frame.faces[i.id]!.top,
                    width: frame.faces[i.id]!.width,
                    height: frame.faces[i.id]!.height,
                    child: GestureDetector(
                      behavior: HitTestBehavior.opaque,
                      onTap: () => widget.onTap(i.id),
                      onDoubleTap: widget.onOpen == null ? null : () => widget.onOpen!(i.id),
                      child: Container(
                        decoration: BoxDecoration(borderRadius: BorderRadius.circular(4), border: Border.all(color: i.id == chosen ? N.g95 : N.glaze15, width: i.id == chosen ? 2 : 1)),
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
        }),
      );
}
