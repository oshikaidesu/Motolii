import 'package:flutter/widgets.dart';

import '../../hf/shell/top_bar.dart';
import '../../hf/shell/top.dart';
import '../../session/editor_session.dart';
import '../../session/stage_actions.dart';
import 'document.dart';
import 'sheets.dart';

/// The top face over the real session. The face is the reference's, unchanged; this only hands it values
/// (fps, the document's length in frames, the playhead's clock, which of EDIT·PLAY·EXPORT is on) and operations.
class SessionTop extends StatefulWidget {
  const SessionTop({super.key, required this.c});
  final EditorSession c;
  @override
  State<SessionTop> createState() => _SessionTopState();
}

class _SessionTopState extends State<SessionTop> {
  static const _watched = ['fps', 'durationFrames', 'animate', 'capabilities'];
  EditorSession get c => widget.c;
  final readouts = ValueNotifier(const ['', '', '']);
  bool exporting = false;

  static String clock(int frame, num fps) {
    final rate = fps <= 0 ? 1 : fps;
    final seconds = frame / rate;
    final m = seconds ~/ 60;
    final sec = (seconds % 60).floor();
    final ff = (frame % rate.round().clamp(1, 1000)).toString().padLeft(2, '0');
    return '${m.toString().padLeft(2, '0')}:${sec.toString().padLeft(2, '0')}:$ff';
  }

  void _read() {
    final fps = (c.state['fps'] as num?) ?? 30;
    final duration = (c.state['durationFrames'] as num?)?.toInt() ?? 0;
    readouts.value = [fps.toStringAsFixed(2), '$duration', clock(c.frame.value, fps)];
  }

  @override
  void initState() {
    super.initState();
    c.frame.addListener(_read);
    c.slice('sessionTop', _watched).addListener(_read);
    _read();
  }

  @override
  void dispose() {
    c.frame.removeListener(_read);
    c.slice('sessionTop', _watched).removeListener(_read);
    readouts.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: Listenable.merge([c.playing, c.slice('sessionTop', _watched)]),
        builder: (context, _) {
          final playing = c.playing.value;
          Future<void> mode(TopMode to, [Rect? key]) async {
            if ((to == TopMode.edit && playing) || (to == TopMode.play && !playing)) c.togglePlayback();
            if (to == TopMode.export) {
              setState(() => exporting = true);
              await showExportSheet(context, c, anchor: key);
              if (mounted) setState(() => exporting = false);
            }
          }

          return TopBar(
            TopModel(
              readouts: readouts,
              mode: exporting ? TopMode.export : (playing ? TopMode.play : TopMode.edit),
              onPlay: c.togglePlayback,
              onStop: () {
                c.stopPlayback();
                c.seek(0);
              },
              onAnimate: c.supports('animate') ? () => c.toggleAnimate() : null,
              onMarker: c.supports('addMarker') ? () => c.command('addMarker') : null,
              onMode: mode,
              // Fit frames the Stage (the same operation as Cmd+0); Open replaces the document; Pin has no operation
              onFit: c.supports('stageView') ? () => stageView(c, 'Fit') : null,
              onOpen: () async {
                if (await mayReplace(context, c)) await c.chooseOpen();
              },
            ),
            onModeAt: mode,
          );
        },
      );
}
