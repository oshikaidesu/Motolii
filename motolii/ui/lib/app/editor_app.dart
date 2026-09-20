import 'package:flutter/gestures.dart';
import 'package:flutter/widgets.dart';

import '../foundation/leaves.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';
import 'editor_window.dart';

class EditorApp extends StatefulWidget {
  const EditorApp({super.key, this.controller, this.initialize = true});
  final EditorSession? controller;
  final bool initialize;

  /// Hovering a control shows nothing: every [EditorTooltip] below is off.
  static Widget noHover(BuildContext context, Widget? child) =>
      EditorLook(tooltips: false, child: child ?? const SizedBox());
  @override
  State<EditorApp> createState() => _EditorAppState();
}

class _EditorAppState extends State<EditorApp> {
  final scale = ValueNotifier(1.0);
  late final appearance = ValueNotifier(EditorTheme.chromatic);
  @override
  void dispose() {
    scale.dispose();
    appearance.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => EditorAppearance(
    notifier: appearance,
    child: ValueListenableBuilder(
      valueListenable: appearance,
      builder: (context, theme, _) => theme.wrap(
        Builder(
          builder: (context) => ScrollConfiguration(
            behavior: const EditorScrollBehavior(),
            child: WidgetsApp(
              debugShowCheckedModeBanner: false,
              color: EditorTheme.of(context).app,
              textStyle: EditorTheme.of(context).text,
              pageRouteBuilder:
                  <T>(RouteSettings settings, WidgetBuilder builder) =>
                      PageRouteBuilder<T>(
                        settings: settings,
                        pageBuilder: (context, _, _) => builder(context),
                      ),
              builder: (context, child) => IconTheme(
                data: EditorTheme.of(context).icon,
                child: DefaultSelectionStyle(
                  cursorColor: EditorTheme.of(context).caret,
                  selectionColor: EditorTheme.of(context).selection,
                  child: EditorScale(
                    notifier: scale,
                    child: ValueListenableBuilder(
                      valueListenable: scale,
                      builder: (context, s, _) => EditorScaledViewport(
                        scale: s,
                        child: EditorApp.noHover(context, child),
                      ),
                    ),
                  ),
                ),
              ),
              home: EditorWindow(
                controller: widget.controller,
                initialize: widget.initialize,
              ),
            ),
          ),
        ),
      ),
    ),
  );
}

/// Desktop scrolling: every pointer kind drags, lists stop at their ends
/// (no bounce, no glow), and a vertical list wears the editor's scrollbar.
/// The framework's default is the phone: touch-only drag, overscroll glow,
/// bouncing on iOS-style platforms.
class EditorScrollBehavior extends ScrollBehavior {
  const EditorScrollBehavior();
  @override
  Set<PointerDeviceKind> get dragDevices => PointerDeviceKind.values.toSet();
  @override
  ScrollPhysics getScrollPhysics(BuildContext context) =>
      const ClampingScrollPhysics();
  @override
  Widget buildOverscrollIndicator(
    BuildContext context,
    Widget child,
    ScrollableDetails details,
  ) => child;
  @override
  Widget buildScrollbar(
    BuildContext context,
    Widget child,
    ScrollableDetails details,
  ) => switch ((details.direction, getPlatform(context))) {
    (
      AxisDirection.up || AxisDirection.down,
      TargetPlatform.macOS || TargetPlatform.linux || TargetPlatform.windows,
    ) =>
      EditorScrollbar(controller: details.controller, child: child),
    _ => child,
  };
}
