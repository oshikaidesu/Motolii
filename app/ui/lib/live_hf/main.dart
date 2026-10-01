// The hf GUI as its own client of Motolii Live: proto_hf's faces over the session, nothing from the older shells.
//   MOTOLII_NATIVE_LIBRARY=.../libmotolii_ui.dylib flutter run -d macos -t lib/live_hf/main.dart \
//     --dart-define=MOTOLII_DOCUMENT=/path.rrd [--dart-define=MOTOLII_SHOT=/path.png]
import 'package:flutter/widgets.dart';

import '../hf/shell/place.dart' show H;
import 'shell.dart';

void main() => runApp(WidgetsApp(
      color: H.window,
      debugShowCheckedModeBanner: false,
      textStyle: H.s(12),
      pageRouteBuilder: <T>(RouteSettings settings, WidgetBuilder builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (context, _, __) => builder(context)),
      home: const LiveShell(),
    ));
