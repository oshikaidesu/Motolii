import 'package:analyzer_testing/analysis_rule/analysis_rule.dart';
import 'package:motolii_lints/src/material_import.dart';
import 'package:test_reflective_loader/test_reflective_loader.dart';

void main() {
  defineReflectiveSuite(() {
    defineReflectiveTests(MaterialImportTest);
  });
}

@reflectiveTest
class MaterialImportTest extends AnalysisRuleTest {
  @override
  void setUp() {
    rule = MaterialImport();
    super.setUp();
    // A stand-in flutter package, so the imports resolve and only the rule
    // speaks.
    newPackage('flutter');
    for (final lib in ['material', 'cupertino', 'widgets', 'services']) {
      newFile(
        '/package/flutter/lib/$lib.dart',
        'class ${lib.substring(0, 1).toUpperCase()}${lib.substring(1)} {}',
      );
    }
    writeTestPackageConfig2();
  }

  void test_material_and_cupertino_are_reported() async {
    await assertDiagnostics(
      r'''
import 'package:flutter/material.dart';
import 'package:flutter/cupertino.dart';
Material? a;
Cupertino? b;
''',
      [lint(0, 39), lint(40, 40)],
    );
  }

  void test_widgets_passes() async {
    await assertNoDiagnostics(r'''
import 'package:flutter/widgets.dart';
import 'package:flutter/services.dart';
Widgets? a;
Services? b;
''');
  }

  void test_theme_types_only_pass() async {
    newFile(
      '/package/flutter/lib/material.dart',
      'class Theme {} class ThemeData {} class ThemeExtension {} class ColorScheme {}',
    );
    await assertNoDiagnostics(r'''
import 'package:flutter/material.dart' show Theme, ThemeData, ThemeExtension, ColorScheme;
Theme? a;
ThemeData? b;
ThemeExtension? c;
ColorScheme? d;
''');
  }

  void test_theme_does_not_allow_controls() async {
    newFile(
      '/package/flutter/lib/material.dart',
      'class Theme {} class Button {}',
    );
    const source =
        "import 'package:flutter/material.dart' show Theme, Button;\nTheme? a; Button? b;";
    await assertDiagnostics(source, [lint(0, source.indexOf(';') + 1)]);
  }
}
