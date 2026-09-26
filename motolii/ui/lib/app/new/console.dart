import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../foundation/metrics.dart';
import '../../foundation/shell_tokens.dart';
import '../../foundation/theme.dart';
import '../../session/editor_session.dart';
import 'primitives.dart';

enum ConsoleLevel { error, notice }

class ConsoleEntry {
  const ConsoleEntry(this.at, this.level, this.text);
  final DateTime at;
  final ConsoleLevel level;
  final String text;
}

/// What the status line says, kept: every operation error and every document notice the session already
/// surfaces, oldest first. It adds no message of its own; the status line still shows the latest one.
/// It lives with the shell, not with the panel, so closing and reopening the panel loses nothing.
class ConsoleLog extends ChangeNotifier {
  ConsoleLog(this._c, this._notice, this._noticeText) {
    _c.error.addListener(_error);
    _notice.addListener(_noticed);
  }

  final EditorSession _c;
  final ValueListenable<Map<String, dynamic>> _notice;
  final String? Function(Map<String, dynamic> document) _noticeText;
  final entries = <ConsoleEntry>[];
  String? _lastNotice;

  void _error() {
    final text = _c.error.value;
    if (text != null && text.isNotEmpty) _add(ConsoleLevel.error, text);
  }

  void _noticed() {
    final text = _noticeText(_notice.value);
    if (text == _lastNotice) return;
    _lastNotice = text;
    if (text != null && text.isNotEmpty) _add(ConsoleLevel.notice, text);
  }

  void _add(ConsoleLevel level, String text) {
    entries.add(ConsoleEntry(DateTime.now(), level, text));
    notifyListeners();
  }

  void clear() {
    entries.clear();
    notifyListeners();
  }

  @override
  void dispose() {
    _c.error.removeListener(_error);
    _notice.removeListener(_noticed);
    super.dispose();
  }
}

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
