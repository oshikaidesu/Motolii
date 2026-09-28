// Dock probe: the real proto_hf panels inside the off-the-shelf `docking` package. Not production.
//   flutter run -d macos -t lib/proto_hf/main_dock.dart      (live, drag things)
//   ... --dart-define=PROTO_SHOT=/path.png                   (1536x1024 capture)
import 'dart:io';
import 'dart:ui' as ui;
import 'package:docking/docking.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import '../hf/bp/catalog_io.dart';
import '../hf/bp/common.dart';
import '../hf/bp/effects.dart';
import 'dock/workspace.dart';
import '../hf/desk/blend.dart';
import '../hf/desk/ease.dart';
import 'hf.dart' as hf;
import '../hf/insp/panel.dart';
import 'fixtures.dart';
import '../hf/insp/rows.dart';
import '../hf/insp/transform.dart';
import '../hf/insp/transform_model.dart';
import 'main_browser.dart' as br;
import 'main_transform.dart' as tf;
import 'ref.dart' show H, RF;

const _shot = String.fromEnvironment('PROTO_SHOT');
const _thingsDir = String.fromEnvironment('PROTO_THINGS', defaultValue: 'lib/proto_hf/data/things');
final _root = GlobalKey();
const _w = int.fromEnvironment('PROTO_W', defaultValue: 1536), _h = int.fromEnvironment('PROTO_H', defaultValue: 1024);

Workspace buildWorkspace(EffectScene scene) {
  final layer = ParamStore(nativeLike());
  final transform = TransformStore(tf.layers());
  Widget docked(Widget body) => DockedPanel(child: body); // the tab names the panel; the panel keeps its own controls
  return Workspace(
    {
      'create': PanelDef('create', 'Create', () => docked(br.panel(0, scene)), minSize: 220),
      'effects': PanelDef('effects', 'Effects', () => docked(br.panel(1, scene)), minSize: 220),
      'stage': PanelDef('stage', 'Stage', () => const _Stage(), minSize: 260),
      'timeline': PanelDef('timeline', 'Timeline', () => const _Timeline(), minSize: 120),
      'transform': PanelDef('transform', 'Transform', () => TransformInstrument(transform), minSize: 240),
      'layer': PanelDef('layer', 'Layer', () => InspectorBody(store: layer, subject: 'Layer', thingId: 'Layer'), minSize: 240),
      'ease': PanelDef('ease', 'Ease', () => docked(const EaseDesk()), minSize: 240),
      'blend': PanelDef('blend', 'Blend', () => docked(const BlendDesk()), minSize: 240),
    },
    (item) => DockingRow([
      DockingTabs([item('create'), item('effects')], weight: .2),
      DockingColumn([
        DockingRow([item('stage', weight: .7), DockingTabs([item('transform'), item('layer'), item('ease'), item('blend')], weight: .3)]),
        item('timeline', weight: .3),
      ], weight: .8),
    ]),
  );
}

class _Stage extends StatelessWidget {
  const _Stage();
  @override
  Widget build(BuildContext context) => Image.file(File('lib/proto/stage_hf.png'), fit: BoxFit.cover, alignment: Alignment.center);
}

class _Timeline extends StatelessWidget {
  const _Timeline();
  @override
  Widget build(BuildContext context) => ClipRect(
        child: OverflowBox(
          // The reference draws a Timeline / Graph / Console mode strip in its top 38 px. The dock tab is the panel's name and
          // Graph does not exist, so the strip is cropped away.
          alignment: Alignment.topLeft, minWidth: 1178, maxWidth: 1178, minHeight: 253, maxHeight: 253,
          child: RF(hf.timeline(), ox: 344, oy: 741),
        ),
      );
}

/// Motolii chrome over the dock's tab and split parts.
TabbedViewThemeData motoliiTabs() {
  final t = TabbedViewThemeData.dark();
  t.tabsArea
    ..color = const Color(0xFF141414)
    ..border = const Border(bottom: BorderSide(color: kRule))
    ..initialGap = 0
    ..middleGap = 0
    ..gapBottomBorder = BorderSide.none;
  t.tab
    ..padding = const EdgeInsets.symmetric(horizontal: 12, vertical: 8)
    ..textStyle = sans(12.5, c: kMuted)
    ..decoration = const BoxDecoration(color: Color(0xFF141414))
    ..selectedStatus = (TabStatusThemeData()
      ..decoration = const BoxDecoration(color: kSel)
      ..fontColor = const Color(0xFFF5F5F5));
  t.contentArea
    ..decoration = const BoxDecoration(color: kGround)
    ..padding = EdgeInsets.zero;
  return t;
}

class DockProbe extends StatefulWidget {
  const DockProbe({super.key, required this.workspace});
  final Workspace workspace;
  @override
  State<DockProbe> createState() => _DockProbeState();
}

class _DockProbeState extends State<DockProbe> {
  @override
  Widget build(BuildContext context) => ColoredBox(
        color: H.window,
        // Dragging a tab needs an Overlay above the dock (the bare WidgetsApp builder has none).
        child: Overlay(initialEntries: [OverlayEntry(builder: (_) => TabbedViewTheme(
          data: motoliiTabs(),
          child: MultiSplitViewTheme(
            data: MultiSplitViewThemeData(dividerThickness: 4, dividerPainter: DividerPainters.background(color: const Color(0xFF0C0C0D), highlightedColor: const Color(0xFF3B3D42))),
            child: widget.workspace.view(),
          ),
        ))]),
      );
}

void main() async {
  final scene = await EffectScene.build();
  br.baseCatalog = loadCatalog(_thingsDir);
  br.grownCatalog = loadCatalog(_thingsDir, sets: const ['builtin', 'stress']);
  final ws = buildWorkspace(scene);
  runApp(WidgetsApp(
    color: kGround,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => _shot.isEmpty
        ? DefaultTextStyle(style: sans(12), child: DockProbe(workspace: ws))
        : Align(alignment: Alignment.topLeft, child: OverflowBox(alignment: Alignment.topLeft, minWidth: _w * 1.0, maxWidth: _w * 1.0, minHeight: _h * 1.0, maxHeight: _h * 1.0, child: RepaintBoundary(key: _root, child: DefaultTextStyle(style: sans(12), child: DockProbe(workspace: ws))))),
  ));
  if (_shot.isNotEmpty) {
    Future<void>.delayed(const Duration(seconds: 40), () => exit(2));
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      for (var i = 0; i < 8; i++) { await WidgetsBinding.instance.endOfFrame; }
      final img = await (b.debugLayer as OffsetLayer).toImage(Offset.zero & b.size, pixelRatio: 1.0);
      File(_shot).writeAsBytesSync((await img.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List());
      exit(0);
    });
  }
}
