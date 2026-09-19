import 'package:analyzer_testing/analysis_rule/analysis_rule.dart';
import 'package:motolii_lints/src/raw_color.dart';
import 'package:test_reflective_loader/test_reflective_loader.dart';

void main() {
  defineReflectiveSuite(() {
    defineReflectiveTests(RawColorTest);
  });
}

@reflectiveTest
class RawColorTest extends AnalysisRuleTest {
  @override
  void setUp() {
    rule = RawColor();
    super.setUp();
  }

  void test_hex_literal_is_reported() async {
    await assertDiagnostics(
      r'''
class Color { const Color(int v); }
void f() {
  const Color(0xff242424);
  Color(0xffaedce8);
}
''',
      [lint(49, 23), lint(76, 17)],
    );
  }

  void test_computed_and_named_pass() async {
    await assertNoDiagnostics(r'''
class Color {
  const Color(int v);
  const Color.fromARGB(int a, int r, int g, int b);
}
void f(int n) {
  Color(0xff000000 | n);
  const Color.fromARGB(255, 1, 2, 3);
}
''');
  }

  void test_theme_file_may_hold_colours() async {
    final path = '$testPackageLibPath/foundation/theme.dart';
    newFile(path, r'''
class Color { const Color(int v); }
const ink = Color(0xffdddddd);
''');
    await assertNoDiagnosticsInFile(path);
  }
}
