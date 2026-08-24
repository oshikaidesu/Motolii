import subprocess
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("check_responsibility.py")


class ResponsibilityTest(unittest.TestCase):
    def test_current_wire_boundary_has_no_forbidden_write(self):
        result = subprocess.run(
            ["python3", str(SCRIPT), str(Path(__file__).parents[1])],
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("違反 0", result.stdout)


if __name__ == "__main__":
    unittest.main()
