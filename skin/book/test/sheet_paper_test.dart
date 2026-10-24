import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lab_book/sets/sheet_page.dart';

void main() {
  testWidgets('the sheet page draws its shapes', (t) async {
    await t.pumpWidget(WidgetsApp(color: const Color(0xFF000000), builder: (c, _) => const SheetPaper()));
    await t.pump();
    expect(find.text('Fit'), findsOneWidget);
    expect(find.text('Snap'), findsOneWidget);
    expect(find.text('Linear'), findsOneWidget);
  });
}
