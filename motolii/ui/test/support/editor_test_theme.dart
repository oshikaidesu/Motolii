import 'package:flutter/material.dart';
import 'package:motolii_stage5/foundation/metrics.dart';
import 'package:motolii_stage5/foundation/theme.dart';

/// The Material theme the tests mount panels under, now that the window
/// itself runs on `flutter/widgets` alone: the same body text, icon size and
/// density the app's ThemeData once carried, so a test's layout stays what it
/// measured before.
final editorTestTheme = ThemeData.dark(useMaterial3: true).copyWith(
  scaffoldBackgroundColor: EditorTheme.app,
  canvasColor: EditorTheme.panel,
  colorScheme: const ColorScheme.dark(
    primary: EditorTheme.accent,
    secondary: EditorTheme.accent,
    surface: EditorTheme.panel,
  ),
  textTheme: const TextTheme(
    bodyLarge: TextStyle(fontSize: EditorMetrics.font, color: EditorTheme.ink),
    bodyMedium: TextStyle(fontSize: EditorMetrics.font, color: EditorTheme.ink),
    bodySmall: TextStyle(
      fontSize: EditorMetrics.dense,
      color: EditorTheme.muted,
    ),
    titleMedium: TextStyle(
      fontSize: EditorMetrics.font,
      color: EditorTheme.ink,
    ),
    labelLarge: TextStyle(fontSize: EditorMetrics.font, color: EditorTheme.ink),
  ),
  visualDensity: VisualDensity.compact,
  typography: Typography.material2014(platform: TargetPlatform.macOS),
  materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
  iconTheme: EditorTheme.icon,
);
