import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

import '../foundation/metrics.dart';
import '../foundation/panel_controls.dart' show EditorScale;
import '../hf/shell/place.dart' show H;
import '../hf/shell/shell_face.dart';
import '../input/editor_shortcuts.dart';
import '../panels/composition_controls.dart';
import '../panels/export_controls.dart';
import '../panels/timeline.dart';
import '../session/editor_session.dart';
import 'editor_actions.dart';
import '../hf/bp/effects.dart' show EffectScene;
import 'new/shell/live_browser.dart';
import 'new/shell/live_stage.dart';
import 'new/shell/session_top.dart';
import 'new/inspector/new_transform.dart';
import 'new/shell_bar.dart' show NewSettings;

/// The proto_hf Shell face as the production window: the same [ShellFace] proto_hf's own `Shell` builds, with
/// Motolii Live in every seat. No dock yet — docking comes back later behind this face, not in front of it.
/// A path: once the document is open, the 1536x1024 face is written there as a PNG, the same capture
/// proto_hf's `PROTO_SHOT` makes, so the two can be compared pixel for pixel.
const _shot = String.fromEnvironment('MOTOLII_SHOT');

class ProductionProtoShell extends StatefulWidget {
  const ProductionProtoShell({super.key});
  @override
  State<ProductionProtoShell> createState() => _ProductionProtoShellState();
}

class _ProductionProtoShellState extends State<ProductionProtoShell> {
  final c = EditorSession();
  bool ready = false;
  late EffectScene scene;
  final _face = GlobalKey();
  String? sheet;
  late final uiScale = EditorScale.of(context) ?? ValueNotifier(1.0);
  late final shortcuts = EditorShortcuts(
    c,
    onMenu: menu,
    hasSheet: () => sheet != null,
    closeSheet: () => setState(() => sheet = null),
    showComposition: () => setState(() => sheet = 'Composition'),
    showInspector: () {},
  );

  @override
  void initState() {
    super.initState();
    c.confirmClose = () => confirmReplacement(context, c);
    c.filesDropped = (paths) {
      if (paths.isNotEmpty) c.importPaths(paths);
    };
    _initialize();
  }

  Future<void> _initialize() async {
    scene = await EffectScene.build();
    await c.initialize();
    if (!mounted) return;
    setState(() => ready = true);
    if (_shot.isNotEmpty) _capture();
  }

  Future<void> _capture() async {
    await Future<void>.delayed(const Duration(seconds: 3));
    for (var i = 0; i < 8; i++) {
      await WidgetsBinding.instance.endOfFrame;
    }
    final b = _face.currentContext!.findRenderObject()! as RenderRepaintBoundary;
    final img = await b.toImage(pixelRatio: 1.0);
    File(_shot).writeAsBytesSync((await img.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List());
    stdout.writeln('MOTOLII_SHOT ${img.width}x${img.height} $_shot');
  }

  @override
  void dispose() {
    c.dispose();
    super.dispose();
  }

  Future<void> menu(String action) async {
    setState(() => sheet = null);
    if (await documentAction(context, c, action)) return;
    await c.placePanel(action, 'show');
  }

  void toggleSheet(String name) => setState(() => sheet = sheet == name ? null : name);

  @override
  Widget build(BuildContext context) => Focus(
        autofocus: true,
        onKeyEvent: shortcuts.handle,
        child: ColoredBox(
          color: H.window,
          child: !ready
              ? const SizedBox.expand()
              : Center(
                  child: FittedBox(
                    fit: BoxFit.scaleDown,
                    child: RepaintBoundary(
                      key: _face,
                      child: SizedBox(
                      width: ShellFace.width,
                      height: ShellFace.height,
                      child: Stack(children: [
                        ShellFace(
                          top: SessionTop(c: c, sheet: sheet, onSheet: toggleSheet, onMenu: menu),
                          browser: LiveBrowser(c: c, scene: scene),
                          stage: LiveStage(c: c),
                          right: NewTransform(controller: c),
                          timeline: TimelinePanel(controller: c),
                        ),
                        if (sheet != null) ..._sheet(),
                      ]),
                    ),
                    ),
                  ),
                ),
        ),
      );

  List<Widget> _sheet() => [
        Positioned.fill(child: GestureDetector(behavior: HitTestBehavior.opaque, onTap: () => setState(() => sheet = null))),
        Positioned(
          left: 344,
          top: 62,
          child: Container(
            width: sheet == 'Settings' ? EditorMetrics.sheetWide : EditorMetrics.sheet,
            decoration: BoxDecoration(color: H.raised, border: Border.all(color: H.rule)),
            child: switch (sheet) {
              'Composition' => CompositionControls(controller: c),
              'Export' => ExportControls(controller: c),
              _ => NewSettings(c: c, scale: uiScale),
            },
          ),
        ),
      ];
}
