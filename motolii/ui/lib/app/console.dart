import 'package:flutter/widgets.dart';

import '../theme/identity.dart' show H;
import '../session/console_log.dart';
import '../theme/metrics.dart' show Dn, Surface;
import '../theme/neutral.dart';

/// The Console panel in the Timeline seat: every operation error and document notice the session surfaced (the
/// log the New shell keeps, one owner), newest first, with Clear. It adds no message of its own.
class LiveConsole extends StatelessWidget {
  const LiveConsole({super.key, required this.log});
  final ConsoleLog log;

  static String _clock(DateTime t) =>
      '${t.hour.toString().padLeft(2, '0')}:${t.minute.toString().padLeft(2, '0')}:${t.second.toString().padLeft(2, '0')}';

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: Surface.base,
    child: ListenableBuilder(
      listenable: log,
      builder: (context, _) {
        final shown = log.entries.reversed.toList();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(
              height: Surface.chromeRow,
              child: Row(
                children: [
                  SizedBox(width: Surface.px(10.5)),
                  Expanded(
                    child: Text(
                      '${shown.length} ${shown.length == 1 ? 'message' : 'messages'}',
                      style: H.s(Dn.nameSize, color: N.g82),
                    ),
                  ),
                  GestureDetector(
                    key: const ValueKey('console-clear'),
                    behavior: HitTestBehavior.opaque,
                    onTap: shown.isEmpty ? null : log.clear,
                    child: Padding(
                      padding: EdgeInsets.symmetric(horizontal: Surface.px(10.5), vertical: Surface.sectionGap),
                      child: Text('Clear', style: H.s(Dn.nameSize, color: shown.isEmpty ? N.g63 : N.g82)),
                    ),
                  ),
                ],
              ),
            ),
            Container(height: 1, color: Surface.divider),
            Expanded(
              child: shown.isEmpty
                  ? Center(child: Text('No messages', style: H.s(Dn.nameSize, color: N.g63)))
                  : ListView.builder(
                      primary: false,
                      padding: EdgeInsets.symmetric(horizontal: Surface.px(10.5), vertical: Surface.px(4.5)),
                      itemCount: shown.length,
                      itemBuilder: (context, i) {
                        final e = shown[i];
                        return Padding(
                          padding: EdgeInsets.symmetric(vertical: Surface.inlineGap),
                          child: Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(_clock(e.at), style: H.m(Dn.nameSize, color: N.g63)),
                              SizedBox(width: Surface.px(7.5)),
                              Container(
                                width: Surface.px(4.5),
                                height: Surface.px(4.5),
                                margin: EdgeInsets.only(top: Surface.px(4)),
                                color: e.level == ConsoleLevel.error ? H.scatter.n : H.follow.b,
                              ),
                              SizedBox(width: Surface.px(7.5)),
                              Expanded(child: Text(e.text, style: H.s(Dn.nameSize, color: Surface.ink))),
                            ],
                          ),
                        );
                      },
                    ),
            ),
          ],
        );
      },
    ),
  );
}
