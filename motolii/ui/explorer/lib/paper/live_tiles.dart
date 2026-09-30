// The live tiles as they look while a pointer is over them: a clip at the frame under the pointer with a progress line, a
// sound with a position, a model turned, a panorama panned in, a still zoomed. Each tile is shown twice, at rest and held.
import 'package:flutter/widgets.dart';
import 'package:motolii_ui/theme/surface.dart';
import 'package:motolii_ui/theme/neutral.dart';
import 'package:motolii_ui/browser/item.dart';
import 'package:motolii_ui/browser/media/catalog_session.dart';
import 'package:motolii_ui/browser/media/fluid.dart';
import 'package:motolii_ui/browser/media/preview.dart';
import 'package:motolii_ui/session/editor_session.dart';

class _Faces implements FaceService {
  _Faces(this.s);
  final CatalogSession s;
  @override
  Future<String?> frameAt(BrowserItem item, double seconds, {int edge = 480}) => s.frameAt(item.id, seconds, edge: edge);
  @override
  Future<String?> pictureOf(BrowserItem item) => s.pictureOf(item.id);
}

class LiveTiles extends StatefulWidget {
  const LiveTiles(this.c, {super.key});
  final EditorSession c;
  @override
  State<LiveTiles> createState() => _LiveTilesState();
}

class _LiveTilesState extends State<LiveTiles> {
  late final CatalogSession session = CatalogSession(widget.c);
  static const wanted = {'coast-drift.mp4': Offset(.62, .5), 'drum-loop.wav': Offset(.4, .5), 'Camera_01.glb': Offset(.85, .5), 'env-meadow.hdr': Offset(.3, .5), 'afterglow.jpg': Offset(.5, .3)};

  @override
  void initState() {
    super.initState();
    session.load();
  }

  @override
  void dispose() {
    session.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: session,
        builder: (context, _) {
          final chosen = [for (final i in session.items) if (wanted.containsKey(i.name)) i];
          Widget board(bool held) => SizedBox(
                width: 340,
                height: 190,
                child: ColoredBox(
                  color: N.g10,
                  child: FluidBoard(items: chosen, selected: null, view: 'thumbnail', onTap: (_) {}, faces: _Faces(session), holding: held ? {for (final i in chosen) i.id: wanted[i.name]!} : const {}),
                ),
              );
          return ColoredBox(
            color: N.g00,
            child: Padding(
              padding: const EdgeInsets.all(10),
              child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                Text('At rest', style: Dn.label(N.g69)),
                const SizedBox(height: 4),
                board(false),
                const SizedBox(height: 10),
                Text('A pointer over each (clip: frame at the pointer + progress; sound: position; model: turned; panorama: panned in; still: zoomed)', style: Dn.label(N.g69)),
                const SizedBox(height: 4),
                board(true),
              ]),
            ),
          );
        },
      );
}
