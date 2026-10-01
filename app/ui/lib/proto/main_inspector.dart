// Inspector alone, at three widths. flutter run -t lib/proto/main_inspector.dart
import 'dart:io';
import 'dart:ui' as ui;
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';
import 'inspector_panel.dart';
import 'tokens.dart';

const _shot = String.fromEnvironment('PROTO_SHOT');
final _root = GlobalKey();

void main() {
  runApp(WidgetsApp(
    color: P.bg,
    debugShowCheckedModeBanner: false,
    builder: (_, __) => Align(
      alignment: Alignment.topLeft,
      child: OverflowBox(
        alignment: Alignment.topLeft,
        minWidth: 1240, maxWidth: 1240, minHeight: 720, maxHeight: 720,
        child: RepaintBoundary(
          key: _root,
          child: DefaultTextStyle(
            style: P.body,
            child: ColoredBox(
              color: const Color(0xFF0B0B0B),
              child: Padding(
                padding: const EdgeInsets.all(20),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    for (final (n, w) in [('WIDE 480', 480.0), ('NORMAL 360', 360.0), ('NARROW 240', 240.0)]) ...[
                      Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                        Padding(padding: const EdgeInsets.only(bottom: 8), child: Text(n, style: P.numMuted)),
                        Expanded(child: SizedBox(width: w, child: ColoredBox(color: P.bg, child: const InspectorPanel2()))),
                      ]),
                      const SizedBox(width: 20),
                    ],
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  ));
  if (_shot.isNotEmpty) {
    Future<void>.delayed(const Duration(seconds: 2), () async {
      final b = _root.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      final img = await b.toImage(pixelRatio: 2);
      File(_shot).writeAsBytesSync((await img.toByteData(format: ui.ImageByteFormat.png))!.buffer.asUint8List());
      exit(0);
    });
  }
}
