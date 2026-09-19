import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/foundation/theme.dart';
import '../lib/workspace/layout.dart';
import '../lib/workspace/workspace_view.dart';

void main() {
  testWidgets('dock tabs and drag feedback need no Material ancestor', (
    tester,
  ) async {
    final layout = WorkspaceLayout();
    await tester.pumpWidget(
      WidgetsApp(
        color: EditorTheme.app,
        pageRouteBuilder: <T>(settings, builder) => PageRouteBuilder<T>(
          settings: settings,
          pageBuilder: (context, _, __) => builder(context),
        ),
        home: DefaultTextStyle(
          style: const TextStyle(color: EditorTheme.ink),
          child: WorkspaceView(
            layout: layout.root,
            panelBuilder: (name) => Text('body:$name'),
            onMove: layout.move,
            onClose: layout.close,
            onDetach: (_) {},
            onLayoutChanged: () {},
          ),
        ),
      ),
    );
    expect(tester.takeException(), isNull);
    await tester.tap(find.text('Media'));
    await tester.pump();
    expect(find.text('body:Media'), findsOneWidget);
    expect(find.text('body:Create'), findsNothing);
    final drag = await tester.startGesture(
      tester.getCenter(find.text('Media')),
    );
    await drag.moveBy(const Offset(80, 45));
    await tester.pump();
    expect(tester.takeException(), isNull);
    await drag.up();
    await tester.pumpWidget(const SizedBox());
    layout.dispose();
  });
}
