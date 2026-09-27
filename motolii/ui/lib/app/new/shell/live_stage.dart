import 'package:flutter/widgets.dart';

import '../../../foundation/theme.dart' show EditorMenuItem, showEditorMenu;
import '../../../hf/shell/place.dart';
import '../../../hf/shell/stage.dart';
import '../../../session/editor_session.dart';

/// The Stage face over the session: the host's rendered frame in the work area, the zoom (Fit or a fixed
/// magnification) and which view is drawn (the document camera, or the Stage observer).
class LiveStage extends StatefulWidget {
  const LiveStage({super.key, required this.c});
  final EditorSession c;
  @override
  State<LiveStage> createState() => _LiveStageState();
}

const _views = {'Camera': 'Camera View', 'User': 'Stage View'};

class _LiveStageState extends State<LiveStage> {
  EditorSession get c => widget.c;
  String view = 'Camera';

  Future<void> _menu(Rect at) async {
    final chosen = await showEditorMenu<String>(context, at.bottomLeft, [for (final e in _views.entries) EditorMenuItem(value: e.key, child: Text(e.value))]);
    if (chosen != null) setState(() => view = chosen);
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: c.slice('liveStage', const ['width', 'height', 'path']),
        builder: (context, _) {
          final w = (c.state['width'] as num? ?? 1920).toDouble(), h = (c.state['height'] as num? ?? 1080).toDouble();
          final shown = 777 / w < 537 / h ? 777 / w : 537 / h;
          final name = (c.state['path'] as String? ?? 'Untitled').split('/').last.replaceAll(RegExp(r'\.[^.]+$'), '');
          return RF(stage(StageModel(
            picture: _Picture(c: c, view: view),
            title: name,
            zoom: '${(shown * 100).round()}%',
            view: _views[view]!,
            onFit: view == 'User' && c.supports('stageView') ? () => c.command('stageView', {'fit': true}) : null,
            onView: _menu,
          )), ox: 344, oy: 62);
        },
      );
}

/// The host's picture of one view, the composition's shape, as large as the work area holds. The Stage observer is
/// drawn by the host into a window of that size showing the whole composition (`stageWindow`).
class _Picture extends StatefulWidget {
  const _Picture({required this.c, required this.view});
  final EditorSession c;
  final String view;
  @override
  State<_Picture> createState() => _PictureState();
}

class _PictureState extends State<_Picture> {
  EditorSession get c => widget.c;
  Map<String, dynamic>? _sent;

  @override
  void initState() {
    super.initState();
    c.attachView(widget.view);
  }

  @override
  void didUpdateWidget(_Picture old) {
    super.didUpdateWidget(old);
    if (old.view != widget.view) {
      c.detachView(old.view);
      if (old.view == 'User') _withdraw();
      c.attachView(widget.view);
    }
  }

  void _withdraw() {
    _sent = null;
    if (c.supports('stageWindow')) c.command('stageWindow', {'width': 0, 'height': 0});
  }

  @override
  void dispose() {
    c.detachView(widget.view);
    if (widget.view == 'User') _withdraw();
    super.dispose();
  }

  void _window(Size size, double w, double h) {
    if (widget.view != 'User' || !c.supports('stageWindow')) return;
    final ratio = MediaQuery.maybeDevicePixelRatioOf(context) ?? 1;
    final window = {'width': (size.width * ratio).round(), 'height': (size.height * ratio).round(), 'roi': [0.0, 0.0, w, h]};
    if (sameValue(window, _sent)) return;
    _sent = window;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) c.command('stageWindow', window);
    });
  }

  @override
  Widget build(BuildContext context) {
    final w = (c.state['width'] as num? ?? 1920).toDouble(), h = (c.state['height'] as num? ?? 1080).toDouble();
    return ColoredBox(
      color: H.window,
      child: Center(
        child: AspectRatio(
          aspectRatio: w / h,
          child: LayoutBuilder(builder: (context, box) {
            _window(box.biggest, w, h);
            return ValueListenableBuilder<Map<String, int>>(
              valueListenable: c.textureIds,
              builder: (_, ids, __) {
                final id = ids[widget.view];
                return id == null ? const SizedBox.expand() : RepaintBoundary(child: Texture(textureId: id, filterQuality: FilterQuality.low));
              },
            );
          }),
        ),
      ),
    );
  }
}
