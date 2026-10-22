// The bench for relation panels: the inside's own stage (Filament) on top, a bar to seek and play, and every panel to
// try side by side under it, all joined to one document: touch any, the stage answers; change a panel's code, hot
// reload, look again. Nothing else of the window.
//   flutter run -d macos --dart-define=MOTOLII_BENCH=true --dart-define=MOTOLII_DOC=<a document>
import 'package:flutter/widgets.dart';
import 'package:widgetbook/widgetbook.dart';

import '../motolii.dart';
import '../tokens.dart';
import '../workspace/desk_kit.dart';
import '../workspace/desk_relations.dart';
import '../workspace/panes.dart';
import '../workspace/stage.dart';
import '../workspace/ws.dart';
import 'group_rule_tries.dart';

const _doc = String.fromEnvironment('MOTOLII_DOC', defaultValue: '$motoliiDir/out/skin.motolii');

/// The panels on the bench, in order: the tab as it is first, then the tries.
List<WsPane> _panels() => [WsPane('Relations', null, () => const DeskRelations()), ...groupRuleTryPanes()];

class RelationBench extends StatefulWidget {
  const RelationBench({super.key, this.document = _doc});
  final String document;
  @override
  State<RelationBench> createState() => _RelationBenchState();
}

class _RelationBenchState extends State<RelationBench> {
  final _ws = Ws();
  Motolii? _inside;

  @override
  void initState() {
    super.initState();
    _join();
  }

  Future<void> _join() async {   // (as the window joins a document: open it, take its stage's picture, join)
    final inside = Motolii.open(widget.document);
    if (inside == null) return;
    _inside = inside;
    final id = await inside.texture();
    if (!mounted) return;
    if (id >= 0) _ws.stageTexture = id;
    _ws
      ..document = widget.document.split('/').last
      ..onSize = (wide, high) {
        if (inside.resize(wide, high)) _ws.reread();
      }
      ..join(inside);
    setState(() {});
  }

  @override
  void dispose() {
    _inside?.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => WsScope(
    ws: _ws,
    child: ColoredBox(
      color: Grey.g07,
      child: _ws.inside == null ? const SizedBox.expand() : Padding(   // (the stage drawn once it is joined)
        padding: const EdgeInsets.all(8),
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          const Expanded(flex: 5, child: WsStage()),
          const SizedBox(height: 6),
          const _Seek(),
          const SizedBox(height: 6),
          Expanded(
            flex: 4,
            child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              for (final (i, p) in _panels().indexed) ...[
                if (i > 0) const SizedBox(width: 6),
                Expanded(
                  child: ColoredBox(
                    color: WsT.card,
                    child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                      DeskStrip(left: [DeskLayerChip(p.name, WsT.toneGroup)]),
                      Expanded(child: p.build()),
                    ]),
                  ),
                ),
              ],
            ]),
          ),
        ]),
      ),
    ),
  );
}

/// Play and a bar to seek: the window's playhead, nothing more.
class _Seek extends StatelessWidget {
  const _Seek();
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: WsScope.read(context),
    builder: (context, _) {
      final ws = WsScope.read(context), u = ws.frame / Ws.duration;
      return Row(children: [
        DeskChip(label: ws.playing ? 'Pause' : 'Play', on: ws.playing, onTap: () => ws.playing = !ws.playing),
        const SizedBox(width: 8),
        Expanded(
          child: LayoutBuilder(builder: (context, box) {
            void seek(double x) {
              ws.playing = false;
              ws.frame = (x / box.maxWidth * Ws.duration).round();
            }

            return GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTapDown: (d) => seek(d.localPosition.dx),
              onHorizontalDragUpdate: (d) => seek(d.localPosition.dx),
              child: CustomPaint(size: Size(box.maxWidth, 22), painter: _Bar(u)),
            );
          }),
        ),
        const SizedBox(width: 8),
        Text('${(ws.frame / Ws.fps).toStringAsFixed(2)} s', style: T.value(Grey.g76)),
      ]);
    },
  );
}

class _Bar extends CustomPainter {
  const _Bar(this.u);
  final double u;
  @override
  void paint(Canvas c, Size s) {
    c.drawRRect(RRect.fromLTRBR(0, s.height / 2 - 2, s.width, s.height / 2 + 2, const Radius.circular(2)), Paint()..color = Grey.g26);
    c.drawRect(Rect.fromLTWH(u * s.width - 1, 2, 2, s.height - 4), Paint()..color = WsT.accent);
  }

  @override
  bool shouldRepaint(_Bar o) => o.u != u;
}

WidgetbookComponent relationBenchSet() => WidgetbookComponent(name: 'Bench', useCases: [WidgetbookUseCase(name: 'Panels', builder: (c) => const RelationBench())]);
