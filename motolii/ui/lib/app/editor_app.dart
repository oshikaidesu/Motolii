import 'package:flutter/material.dart';

import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import 'editor_window.dart';

class EditorApp extends StatefulWidget {
  const EditorApp({super.key});

  /// Hovering a control shows nothing: every [Tooltip] below is off.
  static Widget noHover(BuildContext context, Widget? child) =>
      TooltipVisibility(visible: false, child: child ?? const SizedBox());
  @override
  State<EditorApp> createState() => _EditorAppState();
}

class _EditorAppState extends State<EditorApp> {
  final scale = ValueNotifier(1.0);
  @override
  void dispose() {
    scale.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: EditorTheme.data,
    builder: (context, child) => EditorScale(
      notifier: scale,
      child: ValueListenableBuilder(
        valueListenable: scale,
        builder: (context, s, _) => EditorScaledViewport(
          scale: s,
          child: EditorApp.noHover(context, child),
        ),
      ),
    ),
    home: const EditorWindow(),
  );
}
