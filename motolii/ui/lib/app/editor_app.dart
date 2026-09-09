import 'package:flutter/material.dart';

import '../foundation/theme.dart';
import 'editor_window.dart';

class EditorApp extends StatelessWidget {
  const EditorApp({super.key});
  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: EditorTheme.data,
    builder: (context, child) =>
        TooltipVisibility(visible: false, child: child!),
    home: const EditorWindow(),
  );
}
