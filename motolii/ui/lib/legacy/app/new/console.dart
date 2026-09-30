import 'package:flutter/widgets.dart';

import '../../../theme/editor_metrics.dart';
import '../../foundation/shell_tokens.dart';
import '../../../theme/editor_theme.dart';
import '../../../session/console_log.dart';

export '../../../session/console_log.dart';
import 'primitives.dart';

/// The Console panel: the log, newest first, filtered by the search field, with Clear.
class NewConsole extends StatefulWidget {
  const NewConsole({super.key, required this.log});
  final ConsoleLog log;
  @override
  State<NewConsole> createState() => _NewConsoleState();
}

class _NewConsoleState extends State<NewConsole> {
  final search = TextEditingController();

  @override
  void dispose() {
    search.dispose();
    super.dispose();
  }

  static String _clock(DateTime t) =>
      '${t.hour.toString().padLeft(2, '0')}:${t.minute.toString().padLeft(2, '0')}:${t.second.toString().padLeft(2, '0')}';

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: ShellTokens.surface,
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: ShellTokens.gutter),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: NewSearch(
                  controller: search,
                  hint: 'Search console',
                  onChanged: (_) => setState(() {}),
                ),
              ),
              EditorButton('Clear', widget.log.clear, tooltip: 'Clear the console'),
            ],
          ),
          Expanded(
            child: ListenableBuilder(
              listenable: widget.log,
              builder: (context, _) {
                final query = search.text.trim().toLowerCase();
                final shown = [
                  for (final e in widget.log.entries.reversed)
                    if (query.isEmpty || e.text.toLowerCase().contains(query)) e,
                ];
                if (shown.isEmpty) {
                  return Padding(
                    padding: const EdgeInsets.all(ShellTokens.gutter),
                    child: Text(
                      widget.log.entries.isEmpty ? 'No messages' : 'No matches',
                      style: ShellTokens.kickerStyle(ShellTokens.inkFaint),
                    ),
                  );
                }
                return ListView.builder(
                  primary: false,
                  itemCount: shown.length,
                  itemBuilder: (context, i) {
                    final e = shown[i];
                    final color = e.level == ConsoleLevel.error ? ShellTokens.pink : ShellTokens.peach;
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: EditorMetrics.s4),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(_clock(e.at), style: ShellTokens.readoutStyle(ShellTokens.inkFaint)),
                          const SizedBox(width: EditorMetrics.s8),
                          Container(
                            width: EditorMetrics.s6,
                            height: EditorMetrics.s6,
                            margin: const EdgeInsets.only(top: EditorMetrics.s4),
                            color: color,
                          ),
                          const SizedBox(width: EditorMetrics.s8),
                          Expanded(
                            child: Text(e.text, style: EditorTheme.of(context).text.copyWith(color: ShellTokens.ink)),
                          ),
                        ],
                      ),
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    ),
  );
}
