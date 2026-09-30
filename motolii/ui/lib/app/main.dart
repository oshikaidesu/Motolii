// The hf GUI as its own client of Motolii Live: the hf faces (lib/hf) over the session. From the older shells it takes
// only what live_hf_contract_test allows: the preserved production Stage (panels/stage) and the foundation it needs.
//   MOTOLII_NATIVE_LIBRARY=.../libmotolii_ui.dylib flutter run -d macos -t lib/app/main.dart \
//     --dart-define=MOTOLII_DOCUMENT=/path.rrd [--dart-define=MOTOLII_SHOT=/path.png]
import 'package:flutter/widgets.dart';

import '../controls/panel/scale.dart';
import '../theme/tokens.dart' show H;
import '../theme/live_palette.dart';
import 'window.dart';
import 'ui_scale.dart';

void main() => runApp(WidgetsApp(
      color: H.window,
      debugShowCheckedModeBanner: false,
      textStyle: H.s(12),
      pageRouteBuilder: <T>(RouteSettings settings, WidgetBuilder builder) => PageRouteBuilder<T>(settings: settings, pageBuilder: (context, _, __) => builder(context)),
      // the UI's size scales the whole root — the overlay with its menus and sheets included — and nothing of the work
      // the foundation's widgets (the Stage's chrome) take the live app's colours
      builder: (context, child) => liveEditorTheme.wrap(EditorScale(
        notifier: LiveUiScale.instance.factor,
        child: ValueListenableBuilder<double>(
          valueListenable: LiveUiScale.instance.factor,
          builder: (context, s, _) => Stack(children: [
            Positioned.fill(child: EditorScaledViewport(scale: s, child: child!)),
            const Positioned.fill(child: UiScaleReadout()),
          ]),
        ),
      )),
      home: const LiveShell(),
    ));
