// The app: Motolii's window as its own client of Motolii Live — the faces over the session.
//   MOTOLII_NATIVE_LIBRARY=.../libmotolii_ui.dylib flutter run -d macos \
//     --dart-define=MOTOLII_DOCUMENT=/path.rrd [--dart-define=MOTOLII_SHOT=/path.png]
import 'package:flutter/widgets.dart';

import 'app/ui_scale.dart';
import 'app/window.dart';
import 'theme/live_palette.dart';
import 'theme/identity.dart' show H;
import 'theme/metrics.dart';

void main() => runApp(WidgetsApp(
      color: Surface.base,
      debugShowCheckedModeBanner: false,
      textStyle: H.s(Surface.px(12)),
      pageRouteBuilder: <T>(RouteSettings settings, WidgetBuilder builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (context, _, __) => builder(context)),
      // the foundation's widgets (the Stage's chrome) take the live app's colours; the UI Scale is tokens read at build time, and the
      // root below builds the whole tree again when it changes (menus and sheets live in the overlay below it), keeping every State
      builder: (context, child) => liveEditorTheme.wrap(Stack(children: [
        Positioned.fill(child: UiScaleScope(child: child!)),
        const Positioned.fill(child: UiScaleReadout()),
      ])),
      home: const LiveShell(),
    ));
