import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

import '../hf/bp/effects.dart' show EffectScene;
import '../hf/shell/place.dart' show H;
import '../session/editor_session.dart';
import 'adapters/document.dart';
import 'adapters/top.dart';
import 'keys.dart';
import 'workspace.dart';

/// A path: once the document is open, the 1536x1024 default workspace is written there as a PNG for review.
const _shot = String.fromEnvironment('MOTOLII_SHOT');

/// The hf client root: reference faces and preserved production tools inside the live dock workspace.
class LiveShell extends StatefulWidget {
  const LiveShell({super.key});
  @override
  State<LiveShell> createState() => _LiveShellState();
}

class _LiveShellState extends State<LiveShell> {
  final c = EditorSession();
  final _face = GlobalKey();
  bool ready = false;
  late EffectScene scene;
  LiveWorkspace? workspace;
  Timer? _dockSave;
  late final keys = LiveKeys(c, () => context);

  @override
  void initState() {
    super.initState();
    c.confirmClose = () => mayReplace(context, c);
    c.panelPlacementRequested = (name, placement) async {
      if (placement == 'hide') {
        workspace?.dock.close(name);
      } else {
        workspace?.dock.activate(name, near: 'Stage');
      }
    };
    c.filesDropped = (paths) {
      if (paths.isNotEmpty) c.importPaths(paths);
    };
    _initialize();
  }

  Future<void> _initialize() async {
    scene = await EffectScene.build();
    await c.initialize();
    if (!mounted) return;
    workspace = LiveWorkspace(c: c, scene: scene);
    try {
      final settings = EditorSession.map(await c.native('readSettings'));
      workspace!.dock.restore(settings['hfWorkspace']);
    } catch (_) {}
    workspace!.dock.layout.addListener(_workspaceChanged);
    setState(() => ready = true);
    if (_shot.isNotEmpty) _capture();
  }

  Future<void> _capture() async {
    await Future<void>.delayed(const Duration(seconds: 3));
    for (var i = 0; i < 8; i++) {
      await WidgetsBinding.instance.endOfFrame;
    }
    final b =
        _face.currentContext!.findRenderObject()! as RenderRepaintBoundary;
    final img = await b.toImage(pixelRatio: 1.0);
    File(_shot).writeAsBytesSync(
      (await img.toByteData(format: ui.ImageByteFormat.png))!.buffer
          .asUint8List(),
    );
    stdout.writeln('MOTOLII_SHOT ${img.width}x${img.height} $_shot');
  }

  void _workspaceChanged() {
    _dockSave?.cancel();
    _dockSave = Timer(const Duration(milliseconds: 350), () {
      c.storeSetting('hfWorkspace', workspace!.dock.snapshot());
    });
  }

  @override
  void dispose() {
    _dockSave?.cancel();
    workspace?.dock.layout.removeListener(_workspaceChanged);
    c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Focus(
    autofocus: true,
    onKeyEvent: keys.handle,
    child: ColoredBox(
      color: H.window,
      child: !ready
          ? const SizedBox.expand()
          : RepaintBoundary(
              key: _face,
              child: Column(
                children: [
                  SizedBox(height: 62, child: SessionTop(c: c)),
                  Expanded(child: workspace!.build()),
                ],
              ),
            ),
    ),
  );
}
