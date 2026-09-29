import 'package:flutter/foundation.dart';

import '../../session/editor_session.dart';
import '../../session/media_actions.dart' show mediaFamily;
import 'browser_item.dart';

/// What this work already owns as assets, as one more Result Set for the Browser (next to the catalog's sources): the same
/// faces, the same views. Read from the document as it is (the owner's snapshot); nothing here keeps or edits a list.
/// An item's id is `project:<asset id>`, so a catalog asset and a work's asset are never taken for each other.
class ProjectSource extends ChangeNotifier implements ResultSource {
  ProjectSource(this.c, {required this.kinds, required this.text}) {
    c.document.addListener(notifyListeners);
  }
  final EditorSession c;
  /// The types and words the person chose (read when asked, so this can be kept for as long as the Browser is).
  final Set<String> Function() kinds;
  final String Function() text;

  static const prefix = 'project:';

  @override
  void dispose() {
    c.document.removeListener(notifyListeners);
    super.dispose();
  }

  static String kindOf(Map<String, dynamic> asset) => switch (mediaFamily(asset)) {
        'Video' => 'video',
        'Audio' => 'audio',
        '3D' => 'model',
        'HDR' => 'environment',
        _ => 'image',
      };

  @override
  List<BrowserItem> get items {
    num? n(Object? v) => v is num ? v : null;
    final chosen = kinds();
    final words = text().toLowerCase().split(RegExp(r'\s+')).where((w) => w.isNotEmpty).toList();
    final out = <BrowserItem>[];
    for (final a in EditorSession.maps(c.state['assets'])) {
      final kind = kindOf(a);
      if (chosen.isNotEmpty && !chosen.contains(kind)) continue;
      final name = '${a['name']}';
      final hay = '$name ${a['path'] ?? ''}'.toLowerCase();
      if (words.any((w) => !hay.contains(w))) continue;
      final facts = EditorSession.map(a['facts']);
      out.add(BrowserItem(
        id: '$prefix${a['id']}',
        name: name,
        path: '${a['path'] ?? ''}',
        kind: kind,
        mime: '${a['mime']}',
        source: 'This project',
        width: n(facts['width'])?.toInt(),
        height: n(facts['height'])?.toInt(),
        seconds: n(a['seconds'])?.toDouble() ?? n(facts['seconds'])?.toDouble(),
        sampleRate: n(facts['sampleRate'])?.toInt(),
        channels: n(facts['channels'])?.toInt(),
        faceKey: '$prefix${a['id']}',
        missing: a['missing'] == true,
        thumbnail: a['thumbnail'] as String?,
        peaks: a['peaks'],
      ));
    }
    out.sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
    return out;
  }
}
