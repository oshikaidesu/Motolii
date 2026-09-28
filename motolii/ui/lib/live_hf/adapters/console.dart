import 'package:flutter/widgets.dart';

import '../../hf/shell/place.dart' show H;
import '../../session/console_log.dart';

/// The Console panel in the Timeline seat: every operation error and document notice the session surfaced (the
/// log the New shell keeps, one owner), newest first, with Clear. It adds no message of its own.
class LiveConsole extends StatelessWidget {
  const LiveConsole({super.key, required this.log});
  final ConsoleLog log;

  static String _clock(DateTime t) =>
      '${t.hour.toString().padLeft(2, '0')}:${t.minute.toString().padLeft(2, '0')}:${t.second.toString().padLeft(2, '0')}';

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: H.window,
    child: ListenableBuilder(
      listenable: log,
      builder: (context, _) {
        final shown = log.entries.reversed.toList();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(
              height: 32,
              child: Row(
                children: [
                  const SizedBox(width: 14),
                  Expanded(
                    child: Text(
                      '${shown.length} ${shown.length == 1 ? 'message' : 'messages'}',
                      style: H.s(11, color: H.text2),
                    ),
                  ),
                  GestureDetector(
                    key: const ValueKey('console-clear'),
                    behavior: HitTestBehavior.opaque,
                    onTap: shown.isEmpty ? null : log.clear,
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                      child: Text('Clear', style: H.s(11, color: shown.isEmpty ? H.text3 : H.text2)),
                    ),
                  ),
                ],
              ),
            ),
            Container(height: 1, color: H.rule),
            Expanded(
              child: shown.isEmpty
                  ? Center(child: Text('No messages', style: H.s(12, color: H.text3)))
                  : ListView.builder(
                      primary: false,
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                      itemCount: shown.length,
                      itemBuilder: (context, i) {
                        final e = shown[i];
                        return Padding(
                          padding: const EdgeInsets.symmetric(vertical: 4),
                          child: Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(_clock(e.at), style: H.m(11, color: H.text3)),
                              const SizedBox(width: 10),
                              Container(
                                width: 6,
                                height: 6,
                                margin: const EdgeInsets.only(top: 5),
                                color: e.level == ConsoleLevel.error ? H.scatter.n : H.follow.b,
                              ),
                              const SizedBox(width: 10),
                              Expanded(child: Text(e.text, style: H.s(12.5, color: H.text))),
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
