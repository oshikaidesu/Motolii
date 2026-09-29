import 'package:docking/docking.dart' show DropPosition;
import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

import '../hf/bp/effects.dart' show EffectScene;
import '../hf/shell/place.dart' show H;
import '../session/console_log.dart';
import '../session/editor_session.dart';
import '../session/status_notice.dart';
import 'adapters/document.dart';
import 'adapters/top.dart';
import 'keys.dart';
import 'ui_scale.dart';
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
  final detached = <String, (String, String?)>{};
  late final keys = LiveKeys(c, () => context);

  /// Every operation error and document notice, for the Console (freeze progress and effect catalogue notices, as
  /// the Classic status line reads them).
  late final notice = c.slice('notice', const [], derived: () => freezeNotice(c.state) ?? effectsNotice(c.state));
  late final console = ConsoleLog(c, notice, (doc) => freezeNotice(doc) ?? effectsNotice(doc));

  @override
  void initState() {
    super.initState();
    c.confirmClose = () => mayReplace(context, c);
    console; // the log listens from the start: an error while the document opens is kept too
    c.panelPlacementRequested = (name, placement) async {
      if (placement == 'hide') {
        workspace?.dock.close(name);
      } else {
        // Contextual panels open beside the Inspector, where the desk sat before the Dock, as their own seat.
        workspace?.dock.activate(name, near: 'Inspector', side: DropPosition.bottom);
      }
    };
    c.windowClosed = _windowClosed;
    c.filesDropped = (paths) {
      if (paths.isNotEmpty) c.importPaths(paths);
    };
    _initialize();
  }

  Future<void> _initialize() async {
    scene = await EffectScene.build();
    await c.initialize();
    if (!mounted) return;
    workspace = LiveWorkspace(c: c, scene: scene, console: console, onDetach: _detach);
    LiveUiScale.instance.persist = (v) => c.storeSetting(LiveUiScale.settingsKey, v);
    try {
      final settings = EditorSession.map(await c.native('readSettings'));
      workspace!.dock.restore(settings['hfWorkspace']);
      LiveUiScale.instance.restore(settings[LiveUiScale.settingsKey]);
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

  Future<void> _detach(String name) async {
    if (c.windowInfo['main'] == false || workspace == null || !workspace!.dock.isOpen(name)) return;
    try {
      final info = EditorSession.map(await c.native('openPanelWindow', {'panels': [name]}));
      detached['${info['id']}'] = (name, workspace!.dock.neighbour(name));
      workspace!.dock.close(name);
    } catch (e) {
      c.error.value = '$e';
    }
  }

  void _windowClosed(Map<String, dynamic> info) {
    final back = detached.remove('${info['id']}');
    if (back != null) workspace?.dock.activate(back.$1, near: back.$2 ?? 'Stage');
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
    console.dispose();
    c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (ready && c.windowInfo['main'] == false) {
      final names = (c.windowInfo['panels'] as List? ?? const []).whereType<String>();
      final name = names.isEmpty ? null : names.first;
      final panel = name == null ? null : workspace?.dock.defs[name];
      return ColoredBox(
        color: H.window,
        child: panel?.build() ?? const SizedBox.expand(),
      );
    }
    return Focus(
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
                  SessionTop(c: c),
                  Expanded(child: workspace!.build()),
                ],
              ),
            ),
    ),
  );
  }
}
