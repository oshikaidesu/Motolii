// TEMPORARY — the Skin Swap Proof for the Browser. A deliberately different skin over the same BrowserSession: plain
// text lists (every thing that does something, every swatch, every font), a click uses it. If this makes layers,
// applies effects, colours and fonts with no change outside it, the Browser's meaning lives in the session. Delete
// after the proof.
import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show mono;
import '../../hf/neutral.dart';
import '../../session/editor_session.dart';
import 'browser_session.dart';

final altBrowserSkin = ValueNotifier(false);

class AltBrowser extends StatelessWidget {
  const AltBrowser({super.key, required this.c, required this.shelf});
  final EditorSession c;
  final String shelf;
  @override
  Widget build(BuildContext context) {
    final s = BrowserSession.of(c);
    return ListenableBuilder(
      listenable: s,
      builder: (context, _) {
        Widget line(String key, String text, VoidCallback onTap) => GestureDetector(
              key: ValueKey('alt-$key'),
              behavior: HitTestBehavior.opaque,
              onTap: onTap,
              child: Padding(padding: const EdgeInsets.symmetric(vertical: 3), child: Text(text, style: mono(11, c: N.g91))),
            );
        final names = {for (final (_, t) in s.catalog.files) '${t['id']}': '${t['name']}'};
        final lines = switch (shelf) {
          'Colors' => [for (final sw in s.colors) line(sw.$1, '■ ${sw.$1}  ${sw.$3}', () => s.applyColor(sw))],
          'Fonts' => [for (final f in s.fonts) line(f.family, 'Aa ${f.family}', () => s.applyFont(f, create: s.dressing == null))],
          _ => [
              for (final e in s.bindings.entries)
                if ((shelf == 'Effects') == (e.value.$1 == 'applyEffect')) line(e.key, '> ${names[e.key] ?? e.key}', () => s.use(shelf, e.key)),
            ],
        };
        return ColoredBox(color: N.g00, child: ListView(padding: const EdgeInsets.all(10), children: [Text(shelf.toUpperCase(), style: mono(10, c: N.g56)), ...lines]));
      },
    );
  }
}
