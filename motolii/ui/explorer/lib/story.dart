// One story: a real document (a fixture .rrd, or a host script run on a new document) opened in the real host, the
// production session attached to it, then the production panel drawn from that session at a chosen size and UI scale.
// A story may set inputs (choose layers, seek, open lanes) through the product's own operations; it never decides
// what anything means.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/foundation/panel_controls/scale.dart';
import 'package:motolii_stage5/hf/neutral.dart';
import 'package:motolii_stage5/hf/shell/place.dart' show H;
import 'package:motolii_stage5/live_hf/editor_theme.dart';
import 'package:motolii_stage5/live_hf/ui_scale.dart';
import 'package:motolii_stage5/session/editor_session.dart';
import 'package:widgetbook/widgetbook.dart';

import 'binding.dart';
import 'host.dart';

/// Where the fixtures are: MOTOLII_EXPLORER_FIXTURES (run.sh sets it), else found from the working directory.
String fixture(String name) {
  final root = Platform.environment['MOTOLII_EXPLORER_FIXTURES'] ?? '${Directory.current.path}/fixtures';
  return '$root/$name';
}

/// A document to open, and inputs to give the session once it is attached.
class Scene {
  const Scene(this.document, {this.inputs});
  final String document;
  final Future<void> Function(EditorSession c)? inputs;
}

/// A named state of a panel: the scene it opens, the panel it draws, and the size it is first seen at.
class Story {
  const Story(this.name, this.scene, this.panel, {this.width = 1080, this.height = 300, this.scale = 1.0});
  final String name;
  final Scene scene;
  final Widget Function(EditorSession c) panel;
  final double width, height, scale;
}

/// A story in Widgetbook: its size and UI scale are knobs.
WidgetbookUseCase useCase(Story s) => WidgetbookUseCase(
      name: s.name,
      builder: (context) {
        final w = context.knobs.double.slider(label: 'Width', initialValue: s.width, min: 240, max: 2000, divisions: 176, precision: 0);
        final h = context.knobs.double.slider(label: 'Height', initialValue: s.height, min: 120, max: 1200, divisions: 108, precision: 0);
        final scale = context.knobs.object.segmented<double>(label: 'UI scale', options: const [.70, .75, .80, .85, .90, 1.0, 1.15, 1.30], initialOption: s.scale, labelBuilder: (v) => '${(v * 100).round()}%');
        return ColoredBox(color: N.g00, child: Center(child: framed(s, w, h, scale)));
      },
    );

/// The panel at [width] x [height], at a UI scale, with the live app's theme.
Widget framed(Story s, double width, double height, double scale, {VoidCallback? onReady}) {
  LiveUiScale.instance.set(scale, keep: false);
  return SizedBox(
    width: width,
    height: height,
    // production's default text style (the live main sets it on its WidgetsApp)
    child: liveEditorTheme.wrap(DefaultTextStyle(style: H.s(12), child: EditorScale(
      notifier: LiveUiScale.instance.factor,
      child: ValueListenableBuilder<double>(
        valueListenable: LiveUiScale.instance.factor,
        builder: (context, f, _) => EditorScaledViewport(scale: f, child: StoryHost(key: ValueKey(s.scene), scene: s.scene, panel: s.panel, onReady: onReady)),
      ),
    ))),
  );
}

class StoryHost extends StatefulWidget {
  const StoryHost({super.key, required this.scene, required this.panel, this.onReady});
  final Scene scene;
  final Widget Function(EditorSession c) panel;

  /// Called once the session is attached and its inputs given (shots wait for it).
  final VoidCallback? onReady;
  @override
  State<StoryHost> createState() => _StoryHostState();
}

class _StoryHostState extends State<StoryHost> {
  EditorSession? c;
  Object? failed;

  @override
  void initState() {
    super.initState();
    _open();
  }

  Future<void> _open() async {
    try {
      final binding = ExplorerBinding.ensure();
      binding.host?.close();
      final doc = widget.scene.document;
      final host = doc.endsWith('.js') ? RealHost.open('') : RealHost.open(doc.isEmpty ? '' : fixture(doc));
      if (doc.endsWith('.js')) {
        final reply = host.request({'op': 'runScript', 'path': fixture(doc)});
        if (reply['error'] != null) throw StateError('${reply['error']}');
      }
      binding.host = host;
      final session = EditorSession();
      await session.initialize();
      await widget.scene.inputs?.call(session);
      if (!mounted) return session.dispose();
      setState(() => c = session);
      WidgetsBinding.instance.scheduleWarmUpFrame();
      widget.onReady?.call();
    } catch (e) {
      if (mounted) setState(() => failed = e);
      widget.onReady?.call();
    }
  }

  @override
  void dispose() {
    c?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (failed != null) return Center(child: Text('$failed', style: const TextStyle(color: N.g82, fontSize: 12)));
    final session = c;
    return session == null ? const ColoredBox(color: N.g10) : ColoredBox(color: N.g10, child: widget.panel(session));
  }
}
