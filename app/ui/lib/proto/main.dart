// Visual prototype of the New Shell. Not wired to any document, session or
// command. Run with:  flutter run -d macos -t lib/proto/main.dart
// Pass --dart-define=PROTO_SHOT=/path/out.png to write a screenshot 2 s after
// the first frame.
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'browser.dart';
import 'inspector.dart';
import 'stage.dart';
import 'timeline.dart';
import 'tokens.dart';
import 'top_bar.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
final _root = GlobalKey();

void main() {
  runApp(const ProtoApp());
  if (_shot.isNotEmpty) {
    Future<void>.delayed(const Duration(seconds: 2), () async {
      final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      final img = await b.toImage(pixelRatio: 2);
      final bytes = await img.toByteData(format: ui.ImageByteFormat.png);
      File(_shot).writeAsBytesSync(bytes!.buffer.asUint8List());
      exit(0);
    });
  }
}

class ProtoApp extends StatelessWidget {
  const ProtoApp({super.key});
  @override
  Widget build(BuildContext context) => WidgetsApp(
        color: P.bg,
        debugShowCheckedModeBanner: false,
        // Fixed to the concept art's canvas so the capture compares 1:1.
        builder: (_, __) => Align(
          alignment: Alignment.topLeft,
          child: OverflowBox(
            alignment: Alignment.topLeft,
            minWidth: 1536,
            maxWidth: 1536,
            minHeight: 1024,
            maxHeight: 1024,
            child: RepaintBoundary(key: _root, child: const ProtoShell()),
          ),
        ),
      );
}

class ProtoShell extends StatelessWidget {
  const ProtoShell({super.key});
  @override
  Widget build(BuildContext context) => DefaultTextStyle(
        style: P.body,
        child: ColoredBox(
          color: P.bg,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const TopBar(),
              const Expanded(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    BrowserPanel(),
                    // Timeline runs under both stage and inspector, as in the concept.
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Expanded(
                            child: Row(
                              crossAxisAlignment: CrossAxisAlignment.stretch,
                              children: [Expanded(child: StagePanel()), InspectorPanel()],
                            ),
                          ),
                          SizedBox(height: 270, child: TimelinePanel()),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      );
}
