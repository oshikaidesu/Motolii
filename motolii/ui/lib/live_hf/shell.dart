import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

import '../hf/bp/effects.dart' show EffectScene;
import '../hf/shell/place.dart' show H;
import '../hf/shell/shell_face.dart';
import '../session/editor_session.dart';
import 'adapters/browser.dart';
import 'adapters/document.dart';
import 'adapters/stage.dart';
import 'adapters/timeline.dart';
import 'adapters/top.dart';
import 'adapters/transform.dart';

/// A path: once the document is open, the 1536x1024 face is written there as a PNG, the same capture proto_hf's
/// `PROTO_SHOT` makes, so the two can be compared pixel for pixel.
const _shot = String.fromEnvironment('MOTOLII_SHOT');

/// The same [ShellFace] proto_hf builds, with Motolii Live in every seat.
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

  @override
  void initState() {
    super.initState();
    c.confirmClose = () => mayReplace(context, c);
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

  @override
  Widget build(BuildContext context) => ColoredBox(
        color: H.window,
        child: !ready
            ? const SizedBox.expand()
            : Center(
                child: FittedBox(
                  fit: BoxFit.scaleDown,
                  child: RepaintBoundary(
                    key: _face,
                    child: ShellFace(
                      top: SessionTop(c: c),
                      browser: LiveBrowser(c: c, scene: scene),
                      stage: LiveStage(c: c),
                      right: NewTransform(controller: c),
                      timeline: LiveTimeline(c: c),
                    ),
                  ),
                ),
              ),
      );
}
