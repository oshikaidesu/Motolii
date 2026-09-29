// Shots: every story (or those whose name contains MOTOLII_EXPLORER_ONLY, '|'-separated) drawn at its own size and
// written to MOTOLII_EXPLORER_SHOTS/<name>.png, one after another; a hot restart shoots them again. For looking at
// many states at once without driving a window. Pictures only: nothing is compared or asserted.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'package:motolii_stage5/hf/neutral.dart';

import 'story.dart';

class ShotRunner extends StatefulWidget {
  const ShotRunner({super.key, required this.stories, required this.dir});
  final List<Story> stories;
  final String dir;
  @override
  State<ShotRunner> createState() => _ShotRunnerState();
}

class _ShotRunnerState extends State<ShotRunner> {
  final _frame = GlobalKey();
  late final List<Story> _todo = () {
    final only = Platform.environment['MOTOLII_EXPLORER_ONLY'];
    final words = only == null || only.isEmpty ? null : only.toLowerCase().split('|');
    return [for (final s in widget.stories) if (words == null || words.any((w) => s.name.toLowerCase().contains(w))) s];
  }();
  var _at = 0;

  static String slug(String name) => name.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]+'), '-').replaceAll(RegExp(r'^-|-$'), '');

  Future<void> _ready() async {
    await Future<void>.delayed(const Duration(milliseconds: 700));
    for (var i = 0; i < 6; i++) {
      await WidgetsBinding.instance.endOfFrame;
    }
    final box = _frame.currentContext?.findRenderObject() as RenderRepaintBoundary?;
    if (box != null) {
      final image = await box.toImage(pixelRatio: 2);
      final png = await image.toByteData(format: ui.ImageByteFormat.png);
      Directory(widget.dir).createSync(recursive: true);
      File('${widget.dir}/${slug(_todo[_at].name)}.png').writeAsBytesSync(png!.buffer.asUint8List());
    }
    if (!mounted) return;
    if (_at + 1 < _todo.length) {
      setState(() => _at++);
    } else {
      File('${widget.dir}/done.txt').writeAsStringSync('${_todo.length} ${DateTime.now()}');
      stdout.writeln('EXPLORER_SHOTS ${_todo.length} ${widget.dir}');
    }
  }

  @override
  Widget build(BuildContext context) {
    final s = _todo[_at];
    return ColoredBox(
      color: N.g00,
      child: UnconstrainedBox(
        alignment: Alignment.topLeft,
        child: RepaintBoundary(key: _frame, child: KeyedSubtree(key: ValueKey(_at), child: framed(s, s.width, s.height, 1.0, onReady: _ready))),
      ),
    );
  }
}
