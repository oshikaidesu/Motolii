// The hf GUI as its own client of Motolii Live: the hf faces (lib/hf) over the session. From the older shells it takes
// only what live_hf_contract_test allows: the preserved production Stage (panels/stage) and the foundation it needs.
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
