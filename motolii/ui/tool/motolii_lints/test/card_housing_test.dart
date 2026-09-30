import 'package:analyzer_testing/analysis_rule/analysis_rule.dart';
import 'package:motolii_lints/src/card_housing.dart';
import 'package:test_reflective_loader/test_reflective_loader.dart';

void main() {
  defineReflectiveSuite(() {
    defineReflectiveTests(CardHousingTest);
  });
}

@reflectiveTest
class CardHousingTest extends AnalysisRuleTest {
  @override
  void setUp() {
    rule = CardHousing();
    super.setUp();
  }

  void test_card_is_reported() async {
    await assertDiagnostics(
      r'''
class Card { const Card({Object? child}); }
void f() {
  const Card();
}
''',
      [lint(57, 12)],
    );
  }

  void test_rounded_outlined_box_is_reported() async {
    await assertDiagnostics(
      r'''
class BoxDecoration { const BoxDecoration({Object? border, Object? borderRadius, Object? color}); }
void f() {
  const BoxDecoration(border: 1, borderRadius: 2);
}
''',
      [lint(113, 47)],
    );
  }

  void test_a_level_or_a_single_edge_passes() async {
    await assertNoDiagnostics(r'''
class BoxDecoration { const BoxDecoration({Object? border, Object? borderRadius, Object? color}); }
void f() {
  const BoxDecoration(color: 1);
  const BoxDecoration(border: 1);
  const BoxDecoration(borderRadius: 2);
}
''');
  }

  void test_a_reason_excuses_it() async {
    await assertNoDiagnostics(r'''
class BoxDecoration { const BoxDecoration({Object? border, Object? borderRadius, Object? color}); }
void f() {
  // surface: a badge is a control with its own outline, not a section
  const BoxDecoration(border: 1, borderRadius: 2);
}
''');
  }

  void test_a_bare_excuse_excuses_nothing() async {
    await assertDiagnostics(
      r'''
class BoxDecoration { const BoxDecoration({Object? border, Object? borderRadius, Object? color}); }
void f() {
  // surface:
  const BoxDecoration(border: 1, borderRadius: 2);
}
''',
      [lint(127, 47)],
    );
  }
}
