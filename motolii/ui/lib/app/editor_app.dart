import 'package:flutter/material.dart';

import '../foundation/theme.dart';
import 'editor_window.dart';

class EditorApp extends StatelessWidget {
  const EditorApp({super.key});

  /// Hovering a control shows nothing: every [Tooltip] below is off.
  static Widget noHover(BuildContext context, Widget? child) =>
      TooltipVisibility(visible: false, child: child ?? const SizedBox());
  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: EditorTheme.data,
    builder: noHover,
    home: const EditorWindow(),
  );
}
