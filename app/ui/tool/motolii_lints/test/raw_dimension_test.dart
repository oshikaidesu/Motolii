import 'package:analyzer_testing/analysis_rule/analysis_rule.dart';
import 'package:motolii_lints/src/raw_dimension.dart';
import 'package:test_reflective_loader/test_reflective_loader.dart';

void main() {
  defineReflectiveSuite(() {
    defineReflectiveTests(RawDimensionTest);
  });
}

@reflectiveTest
class RawDimensionTest extends AnalysisRuleTest {
  @override
  void setUp() {
    rule = RawDimension();
    super.setUp();
  }

  void test_named_measurement_is_reported() async {
    await assertDiagnostics(
      r'''
void box({double? width, double? radius}) {}
void f() {
  box(width: 300);
  box(radius: -3.5);
}
''',
      [lint(69, 3), lint(89, 4)],
    );
  }

  void test_positional_geometry_is_reported() async {
    await assertDiagnostics(
      r'''
class EdgeInsets { const EdgeInsets.all(double v); }
void f() {
  const EdgeInsets.all(8);
}
''',
      [lint(87, 1)],
    );
  }

  void test_arithmetic_and_branches_inherit_the_position() async {
    await assertDiagnostics(
      r'''
void box({double? width, double? height}) {}
void f(bool wide, double rows) {
  box(width: wide ? 420 : 320, height: rows * 2 + 4);
}
''',
      [lint(98, 3), lint(104, 3), lint(128, 1)],
    );
  }

  void test_zero_and_hairline_pass() async {
    await assertNoDiagnostics(r'''
void box({double? width, double? height}) {}
void f() {
  box(width: 0, height: 1);
}
''');
  }

  void test_other_numbers_pass() async {
    await assertNoDiagnostics(r'''
class TextStyle { const TextStyle({double? height}); }
void wait({int? milliseconds, double? opacity}) {}
void f() {
  wait(milliseconds: 300, opacity: .4);
  const TextStyle(height: 1.2);
}
''');
  }

  void test_scale_files_may_hold_numbers() async {
    final path = '$testPackageLibPath/foundation/metrics.dart';
    newFile(path, r'''
void box({double? width}) {}
void f() => box(width: 300);
''');
    await assertNoDiagnosticsInFile(path);
  }
}
