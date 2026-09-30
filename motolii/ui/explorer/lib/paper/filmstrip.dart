// A filmstrip of the fluid change: the same Result Set on the board, switched from List to Thumbnail to Explore, with a
// picture of the board taken part-way through each change. It shows that the same faces move (identity and selection
// stay); it is not a product widget.
import 'dart:async';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'package:motolii_ui/theme/surface.dart';
import 'package:motolii_ui/theme/neutral.dart';
import 'package:motolii_ui/browser/media/catalog_session.dart';
import 'package:motolii_ui/browser/media/fluid.dart';
import 'package:motolii_ui/session/editor_session.dart';

import 'explore_view.dart';

class FluidFilmstrip extends StatefulWidget {
  const FluidFilmstrip(this.c, {super.key});
  final EditorSession c;
  @override
  State<FluidFilmstrip> createState() => _FluidFilmstripState();
}

class _FluidFilmstripState extends State<FluidFilmstrip> {
  late final CatalogSession session = CatalogSession(widget.c);
  final boundary = GlobalKey();
  String view = 'list';
  String? selected;
  final frames = <(String, ui.Image)>[];
  static const size = Size(230, 540);

  @override
  void initState() {
    super.initState();
    session.load().then((_) async {
      await Future<void>.delayed(const Duration(seconds: 3)); // faces arrive
      final it = session.items.firstWhere((i) => i.name == 'afterglow.jpg', orElse: () => session.items.first);
      selected = it.id;
      setState(() {});
      await Future<void>.delayed(const Duration(milliseconds: 600));
      await _shoot('list');
      for (final next in ['thumbnail', 'explore']) {
        setState(() => view = next);
        for (final ms in [110, 110, 220]) {
          await Future<void>.delayed(Duration(milliseconds: ms));
          await _shoot('$next +${frames.where((f) => f.$1.startsWith(next)).length * 110}ms');
        }
        await Future<void>.delayed(const Duration(milliseconds: 500));
        await _shoot('$next settled');
      }
    });
  }

  Future<void> _shoot(String label) async {
    final box = boundary.currentContext?.findRenderObject() as RenderRepaintBoundary?;
    if (box == null) return;
    final image = await box.toImage(pixelRatio: 1);
    if (mounted) setState(() => frames.add((label, image)));
  }

  @override
  void dispose() {
    session.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: N.g00,
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Padding(
            padding: const EdgeInsets.all(8),
            child: RepaintBoundary(
              key: boundary,
              child: SizedBox.fromSize(
                size: size,
                child: ColoredBox(color: N.g10, child: ListenableBuilder(listenable: Listenable.merge([session, exploreChanged]), builder: (_, __) => FluidBoard(items: session.items, selected: selected, view: view, onTap: (_) {}, explore: exploreLayout))),
              ),
            ),
          ),
          Expanded(
            child: Wrap(spacing: 6, runSpacing: 6, children: [
              for (final (label, image) in frames)
                Padding(
                  padding: const EdgeInsets.only(left: 8),
                  child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                    Text(label, style: Dn.label(N.g69)),
                    const SizedBox(height: 2),
                    SizedBox(width: 150, height: 150 * size.height / size.width, child: RawImage(image: image, fit: BoxFit.fill)),
                  ]),
                ),
            ]),
          ),
        ]),
      );
}
