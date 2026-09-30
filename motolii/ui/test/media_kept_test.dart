// Favorites and Recents are kept by the asset's catalog id in the user library, and come back when the app is opened again.
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:motolii_stage5/live_hf/adapters/browser_item.dart';
import 'package:motolii_stage5/live_hf/adapters/browser_user.dart';
import 'package:motolii_stage5/session/editor_session.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, (call) async => <String, dynamic>{});
  });
  tearDown(() => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(EditorSession.channel, null));

  test('a favourite is the asset id; keeping and letting go are one step each; a new session reads them back', () async {
    final c = EditorSession();
    final user = LiveBrowserUser(c, 'catalog');
    await user.collect(['uid-1', 'uid-2'], 1);
    expect(user.views.favorites, {'uid-1', 'uid-2'});
    await user.collect(['uid-1'], 0);
    expect(user.views.favorites, {'uid-2'});
    // "the app opened again": the saved desk work is what a new user store reads
    final reopened = LiveBrowserUser(c, 'catalog');
    expect(reopened.views.favorites, {'uid-2'});
    // the work's own library shelves are a different key: nothing leaks between them
    expect(LiveBrowserUser(c, 'media').views.favorites, isEmpty);
    user.dispose();
    reopened.dispose();
    c.dispose();
  });

  test('recents keep the latest first, each asset once, and a short tail', () async {
    final c = EditorSession();
    final user = LiveBrowserUser(c, 'catalog');
    for (final id in ['a', 'b', 'c', 'a']) {
      await user.used(id);
    }
    expect(user.views.recent, ['a', 'c', 'b']);
    for (var i = 0; i < 20; i++) {
      await user.used('x$i');
    }
    expect(user.views.recent.length, LiveBrowserUser.recentLimit);
    expect(user.views.recent.first, 'x19');
    user.dispose();
    c.dispose();
  });

  test('marking an item changes what is shown about it, never which asset it is', () {
    const plain = BrowserItem(id: 'uid-1', name: 'sky.png', path: '/r/sky.png', kind: 'image', mime: 'image/png', fingerprint: 'fp');
    final marked = plain.marked(used: true, favorite: true);
    expect((marked.id, marked.path, marked.fingerprint), (plain.id, plain.path, plain.fingerprint));
    expect((marked.used, marked.favorite), (true, true));
    expect(identical(plain.marked(used: false, favorite: false), plain), isTrue);
  });
}
