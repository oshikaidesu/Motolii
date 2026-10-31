// The Camera View tab: the picture the camera takes, beside the stage, whichever eye the stage has (the maker, 10/31:
// "ステージとは別にカメラのタブが必要"; After Effects' 2 Views with Active Camera). Nothing to hold here: a look, the hand is on the stage.
import 'package:flutter/widgets.dart';

import '../motolii.dart' show Motolii, MotoliiWrites;
import '../tokens.dart';
import 'stage_art.dart';
import 'ws.dart';

class WsCameraView extends StatefulWidget {
  const WsCameraView({super.key});
  @override
  State<WsCameraView> createState() => _WsCameraViewState();
}

class _WsCameraViewState extends State<WsCameraView> {
  (Motolii?, bool) _told = (null, false);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback(_look);
  }

  /// The inside told whether this picture is wanted: only while the tab is in front (tabbed_view keeps a tab behind built,
  /// Offstage), so a tab not looked at costs no second picture. Looked at after each frame the window draws.
  void _look(Duration _) {
    if (!mounted) return;
    final m = WsScope.read(context).inside, seen = context.findAncestorWidgetOfExactType<Offstage>()?.offstage != true;
    if (_told != (m, seen)) {   // (a document opened begins with the tab unsaid)
      _told = (m, seen);
      m?.later('camera_view', word: seen ? 'on' : '');
    }
    WidgetsBinding.instance.addPostFrameCallback(_look);
  }

  @override
  void dispose() {
    _told.$1?.later('camera_view');
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final id = WsScope.of(context, {WsAspect.document, WsAspect.stage}).stageTexture;
    return ColoredBox(
      color: Grey.g10,
      child: id == null ? null : Center(child: AspectRatio(aspectRatio: WsArt.comp.aspectRatio, child: Texture(textureId: id, filterQuality: FilterQuality.medium))),
    );
  }
}
