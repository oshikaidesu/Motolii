import 'package:flutter/foundation.dart';

import 'editor_session.dart';

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
