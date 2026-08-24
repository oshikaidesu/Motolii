import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("derive_components.py")
SPEC = importlib.util.spec_from_file_location("derive_components", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


CONTRACT = '''/* motolii-component
id = "edit.example"
kind = "semantic"
weight = "core_edit"
maps = [1]
entry = ["ExampleEntry"]
meaning = ["example_meaning"]
evaluation = ["example_evaluation"]
render = ["ExampleRender"]
observable = ["example_observable"]
*/
pub struct ExampleEntry;
fn example_meaning() {}
fn example_evaluation() {}
struct ExampleRender;
#[test]
fn example_observable() {}
'''


class DeriveComponentsTests(unittest.TestCase):
    def make_root(self, source: str) -> Path:
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / "next/reference/generated").mkdir(parents=True)
        (root / "next/reference/normal-map.tsv").write_text(
            "id\tcategory\tcanonical\t\t\t\t\t\t\t\t\t\tstatus\n"
            "1\tlocal\tExample\t\t\t\t\t\t\t\t\t\t採用済\n",
            encoding="utf-8",
        )
        (root / "next/core/example.rs").parent.mkdir(parents=True)
        (root / "next/core/example.rs").write_text(source, encoding="utf-8")
        return root

    def test_complete_contract_is_green(self):
        result = MODULE.derive(self.make_root(CONTRACT))
        self.assertEqual(result.red, ())
        self.assertEqual(len(result.rows), 5)

    def test_missing_atom_is_red_even_when_named_in_contract(self):
        source = CONTRACT.replace('meaning = ["example_meaning"]', 'meaning = ["missing_meaning"]')
        result = MODULE.derive(self.make_root(source))
        self.assertTrue(any("missing_meaning" in message for message in result.red))

    def test_generated_contract_is_not_evidence(self):
        source = CONTRACT.replace('render = ["ExampleRender"]', 'render = ["ExampleOnlyInContract"]')
        result = MODULE.derive(self.make_root(source))
        self.assertTrue(any("ExampleOnlyInContract" in message for message in result.red))

    def test_pending_map_is_red(self):
        root = self.make_root(CONTRACT)
        map_path = root / "next/reference/normal-map.tsv"
        map_path.write_text(
            "id\tcategory\tcanonical\t\t\t\t\t\t\t\t\t\tstatus\n"
            "1\tlocal\tExample\t\t\t\t\t\t\t\t\t\t採用予定\n",
            encoding="utf-8",
        )
        result = MODULE.derive(root)
        self.assertTrue(any("採用済でない" in message for message in result.red))


if __name__ == "__main__":
    unittest.main()
