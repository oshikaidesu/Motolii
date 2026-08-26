#!/usr/bin/env python3
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("derive_design_values.py")
SPEC = importlib.util.spec_from_file_location("derive_design_values", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class DeriveDesignValuesTests(unittest.TestCase):
    def make_root(self) -> Path:
        temp = Path(tempfile.mkdtemp())
        (temp / "next/ui/motolii-tokens-rs/src").mkdir(parents=True)
        (temp / "next/ui/motolii-tokens-rs/src/dimensions.rs").write_text(
            "pub struct Dimensions {\n    pub row_height: f32,\n}\n",
            encoding="utf-8",
        )
        (temp / "next/ui/motolii-tokens-rs/src/colors.rs").write_text(
            "pub struct Colors {\n    pub text_primary: Color,\n}\n",
            encoding="utf-8",
        )
        (temp / "next/ui/demo/src").mkdir(parents=True)
        return temp

    def test_extracts_token_reference_and_raw_sink_with_source_pointer(self) -> None:
        root = self.make_root()
        source = root / "next/ui/demo/src/view.rs"
        source.write_text(
            """// .size(999) is documentation and must not count
fn view(dims: Dimensions, colors: Colors) {
    text("x").size(dims.row_height).color(colors.text_primary);
    container("x").height(Length::Fixed(180.0));
    let frames = 180;
}
""",
            encoding="utf-8",
        )
        findings = MODULE.extract(root)
        refs = {(item.owner, item.field_or_literal, item.verdict) for item in findings if item.kind == "token_ref"}
        self.assertIn(("dims", "row_height", "GREEN"), refs)
        self.assertIn(("colors", "text_primary", "GREEN"), refs)
        raw = [item for item in findings if item.kind == "raw_literal"]
        self.assertEqual(len(raw), 1)
        self.assertEqual(raw[0].value, "180.0")
        self.assertEqual(raw[0].line, 4)

    def test_unknown_token_is_red_and_write_is_tsv(self) -> None:
        root = self.make_root()
        source = root / "next/ui/demo/src/view.rs"
        source.write_text(
            'fn view(dims: Dimensions) { text("x").size(dims.missing); }\n',
            encoding="utf-8",
        )
        findings = MODULE.extract(root)
        self.assertEqual(findings[0].verdict, "RED_UNDEFINED_TOKEN")
        output = root / "next/reference/generated/design-values.tsv"
        MODULE.write_tsv(findings, output)
        rows = output.read_text(encoding="utf-8").splitlines()
        self.assertIn("kind\towner\tfield_or_literal", rows[0])
        self.assertIn("RED_UNDEFINED_TOKEN", rows[1])

    def test_theme_utility_is_checked_against_its_namespace(self) -> None:
        root = self.make_root()
        source = root / "next/ui/demo/src/view.rs"
        source.write_text(
            'fn view(dims: Dimensions) { text("x").size(dims.theme().text.body); }\n',
            encoding="utf-8",
        )
        findings = MODULE.extract(root)
        utility = [item for item in findings if item.kind == "utility_ref"]
        self.assertEqual(len(utility), 1)
        self.assertEqual(utility[0].field_or_literal, "theme.text.body")
        self.assertEqual(utility[0].verdict, "GREEN")


if __name__ == "__main__":
    unittest.main()
