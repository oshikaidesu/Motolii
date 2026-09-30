import 'package:flutter/widgets.dart';

import 'rows.dart';
import '../hf/shell/menu.dart' show showHfMenu;

/// The key lines of every Inspector value's right-click (Classic IN-038): "Key this frame" where no key is at the
/// playhead, "Remove key" where one is. [extra] lines follow (a store's own, such as relations); the chosen extra is
/// returned, the key line is done here through the store.
Future<String?> keyMenu(
  BuildContext context,
  ParamStore store,
  String id,
  Offset at, {
  List<(String, String)> extra = const [],
  Set<String> disabled = const {},
}) async {
  final keyed = store.row(id)['keyedNow'] == true;
  final chosen = await showHfMenu<String>(
    context,
    Rect.fromLTWH(at.dx, at.dy, 180, 0),
    [(keyed ? 'unkey' : 'key', keyed ? 'Remove key' : 'Key this frame'), ...extra],
    disabled: {if (!store.keyable || store.frozen) ...{'key', 'unkey'}, ...disabled},
    dividers: {if (extra.isNotEmpty) ...{'key', 'unkey'}},
  );
  if (chosen == 'key' || chosen == 'unkey') {
    store.toggleKey(id);
    return null;
  }
  return chosen;
}
