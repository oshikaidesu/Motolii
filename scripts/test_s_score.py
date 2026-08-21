#!/usr/bin/env python3
"""Contract tests for the S 空間スコア器具第一波(`scripts/s-score.py`)."""

from __future__ import annotations

import importlib.util
import math
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/s-score.py"

SPEC = importlib.util.spec_from_file_location("s_score", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
# `dataclasses` は自分のモジュールを `sys.modules` から引くので、実行前に
# 登録しておかないと `EntranceRow` 等の dataclass 定義自体がクラッシュする。
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


ENTRANCE_ATLAS_FIXTURE = """# fixture

## 入口台帳(`|` 区切り)

列: `操作名 | 種別(a-d,自信度) | 現在の入口 | Message/Intent | map行id | S0期待入口(m:s:p:pref) | 差(S0>a-d 辞書式)`

```
Undo | c(高) | headerボタン | Message::Undo | 10(1:3:0:0) | shortcut優勢 | 適合
CopyLayer | b(高) | **入口なし** | CopyLayer | 20(4:2:0:0) | menu+shortcut | 群0
AddLayer | b(中) | headerボタン | Message::AddLayer | ― | ― | 適合
Mystery | a(低) | どこか | Mystery | ― | ― | ―
```
"""

NORMAL_MAP_FIXTURE = (
    "id\tcategory\tcanonical\t意味\tae\tpr\tdr\tcc\tfreq\t"
    "entries(menu:shortcut:panel:pref)\tquality\tscope\tverdict\t理由\n"
    "10\tedit_basic\tUndo\t元に戻す\t1\t1\t1\t0\t3\t1:3:0:0\t\t\t採用済\tx\n"
    "20\tedit_basic\tCopy\tコピー\t1\t1\t1\t1\t4\t4:2:0:0\t\t\t採用済\ty\n"
)

ATLAS_TSV_FIXTURE = (
    "id\tx\ty\tw\th\tcontent\n"
    "\t26.0\t19.4\t26.7\t14.3\tUndo\n"
)


class ParsingTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.entrance_path = self.root / "entrance.md"
        self.normal_map_path = self.root / "normal-map.tsv"
        self.atlas_path = self.root / "atlas.tsv"
        self.entrance_path.write_text(ENTRANCE_ATLAS_FIXTURE, encoding="utf-8")
        self.normal_map_path.write_text(NORMAL_MAP_FIXTURE, encoding="utf-8")
        self.atlas_path.write_text(ATLAS_TSV_FIXTURE, encoding="utf-8")

    def test_extract_map_ids_strips_parenthetical_entries_tuples(self) -> None:
        self.assertEqual(MODULE._extract_map_ids("437(3:3:0:0)/466"), [437, 466])
        self.assertEqual(MODULE._extract_map_ids("1345/1355/1339(各1:0:0:0)"), [1345, 1355, 1339])
        self.assertEqual(MODULE._extract_map_ids("169(3:2:0:0)"), [169])
        self.assertEqual(MODULE._extract_map_ids("―"), [])
        self.assertEqual(MODULE._extract_map_ids("未特定"), [])

    def test_parse_entrance_atlas_reads_all_four_rows(self) -> None:
        rows = MODULE.parse_entrance_atlas(self.entrance_path)
        self.assertEqual([r.op for r in rows], ["Undo", "CopyLayer", "AddLayer", "Mystery"])
        undo = rows[0]
        self.assertEqual(undo.map_ids, [10])
        self.assertEqual(undo.entrance, "headerボタン")
        self.assertEqual(undo.s0_expected, "shortcut優勢")
        copy_layer = rows[1]
        self.assertIn("入口なし", copy_layer.entrance)
        mystery = rows[3]
        self.assertEqual(mystery.map_ids, [])

    def test_parse_normal_map_reads_entries_and_freq(self) -> None:
        rows = MODULE.parse_normal_map(self.normal_map_path)
        self.assertEqual(rows[10].freq, 3)
        self.assertEqual((rows[10].menu, rows[10].shortcut, rows[10].panel, rows[10].pref), (1, 3, 0, 0))
        self.assertEqual(rows[20].freq, 4)

    def test_parse_atlas_reads_bounds_and_content(self) -> None:
        rows = MODULE.parse_atlas(self.atlas_path)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0].content, "Undo")
        self.assertEqual(rows[0].w, 26.7)


class DominantEntryTypeTest(unittest.TestCase):
    def test_shortcut_dominates_when_it_has_the_most_entries(self) -> None:
        self.assertEqual(MODULE.dominant_entry_type(1, 3, 0, 0), "shortcut")

    def test_menu_wins_ties_by_dictionary_order(self) -> None:
        # menu:shortcut:panel:pref が全て同数なら S0 の辞書式優先で menu が勝つ。
        self.assertEqual(MODULE.dominant_entry_type(2, 2, 2, 2), "menu")

    def test_all_zero_is_no_evidence(self) -> None:
        self.assertEqual(MODULE.dominant_entry_type(0, 0, 0, 0), "―")


class S0TableTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        entrance_path = self.root / "entrance.md"
        normal_map_path = self.root / "normal-map.tsv"
        entrance_path.write_text(ENTRANCE_ATLAS_FIXTURE, encoding="utf-8")
        normal_map_path.write_text(NORMAL_MAP_FIXTURE, encoding="utf-8")
        self.entrance_rows = MODULE.parse_entrance_atlas(entrance_path)
        self.normal_map = MODULE.parse_normal_map(normal_map_path)

    def test_undo_dominant_matches_atlas_expectation(self) -> None:
        table = MODULE.build_s0_table(self.entrance_rows, self.normal_map)
        undo = next(r for r in table if r["op"] == "Undo")
        self.assertEqual(undo["computed_dominant"], "shortcut")
        self.assertTrue(undo["consistent"])
        self.assertFalse(undo["low_confidence"])

    def test_copy_layer_dominant_is_menu_and_consistent(self) -> None:
        table = MODULE.build_s0_table(self.entrance_rows, self.normal_map)
        copy_layer = next(r for r in table if r["op"] == "CopyLayer")
        self.assertEqual(copy_layer["computed_dominant"], "menu")
        self.assertTrue(copy_layer["consistent"])

    def test_row_without_map_id_is_low_confidence_with_no_dominant(self) -> None:
        table = MODULE.build_s0_table(self.entrance_rows, self.normal_map)
        mystery = next(r for r in table if r["op"] == "Mystery")
        self.assertEqual(mystery["computed_dominant"], "―")
        self.assertIsNone(mystery["consistent"])
        self.assertTrue(mystery["low_confidence"])


class S1RankingTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        entrance_path = self.root / "entrance.md"
        normal_map_path = self.root / "normal-map.tsv"
        atlas_path = self.root / "atlas.tsv"
        entrance_path.write_text(ENTRANCE_ATLAS_FIXTURE, encoding="utf-8")
        normal_map_path.write_text(NORMAL_MAP_FIXTURE, encoding="utf-8")
        atlas_path.write_text(ATLAS_TSV_FIXTURE, encoding="utf-8")
        self.entrance_rows = MODULE.parse_entrance_atlas(entrance_path)
        self.normal_map = MODULE.parse_normal_map(normal_map_path)
        self.atlas_rows = MODULE.parse_atlas(atlas_path)

    def test_missing_entrance_ranks_first_with_infinite_cost(self) -> None:
        ranked, _unranked = MODULE.build_s1_ranking(self.entrance_rows, self.normal_map, self.atlas_rows)
        self.assertEqual(ranked[0]["op"], "CopyLayer")
        self.assertEqual(ranked[0]["cost"], math.inf)
        self.assertEqual(ranked[0]["freq"], 4)

    def test_undo_gets_a_finite_geometric_cost_from_the_atlas_row(self) -> None:
        ranked, _unranked = MODULE.build_s1_ranking(self.entrance_rows, self.normal_map, self.atlas_rows)
        undo = next(e for e in ranked if e["op"] == "Undo")
        atlas_row = self.atlas_rows[0]
        cx, cy = atlas_row.center
        rx, ry = MODULE.REFERENCE_POINT
        expected_distance = math.hypot(cx - rx, cy - ry)
        expected_cost = MODULE.fitts_cost(expected_distance, atlas_row.min_dim)
        self.assertAlmostEqual(undo["cost"], expected_cost, places=6)
        self.assertAlmostEqual(undo["score"], 3 * expected_cost, places=6)

    def test_op_with_no_atlas_widget_mapping_is_left_unranked_not_fabricated(self) -> None:
        ranked, unranked = MODULE.build_s1_ranking(self.entrance_rows, self.normal_map, self.atlas_rows)
        self.assertFalse(any(e["op"] == "Mystery" for e in ranked))
        self.assertTrue(any(e["op"] == "Mystery" for e in unranked))

    def test_missing_atlas_leaves_addlayer_unranked_with_a_clear_reason(self) -> None:
        ranked, unranked = MODULE.build_s1_ranking(self.entrance_rows, self.normal_map, None)
        self.assertFalse(any(e["op"] == "AddLayer" for e in ranked))
        entry = next(e for e in unranked if e["op"] == "AddLayer")
        self.assertIn("atlas TSV 未指定", entry["reason"])


class S2KlmTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        entrance_path = self.root / "entrance.md"
        entrance_path.write_text(ENTRANCE_ATLAS_FIXTURE, encoding="utf-8")
        self.entrance_rows = MODULE.parse_entrance_atlas(entrance_path)

    def test_direct_click_operation_costs_p_plus_m_only(self) -> None:
        row = next(r for r in self.entrance_rows if r.op == "AddLayer")
        ops = MODULE.klm_operators_for(row)
        self.assertEqual(ops, {"P": 1, "K": 0, "H": 0, "M": 1})
        self.assertAlmostEqual(MODULE.klm_seconds(ops), 1 * MODULE.KLM_P + 1 * MODULE.KLM_M, places=6)

    def test_missing_entrance_adds_recall_and_hand_switch_cost(self) -> None:
        row = next(r for r in self.entrance_rows if r.op == "CopyLayer")
        ops = MODULE.klm_operators_for(row)
        self.assertEqual(ops, {"P": 1, "K": 1, "H": 1, "M": 2})
        expected = 1 * MODULE.KLM_P + 1 * MODULE.KLM_K + 1 * MODULE.KLM_H + 2 * MODULE.KLM_M
        self.assertAlmostEqual(MODULE.klm_seconds(ops), expected, places=6)

    def test_build_s2_table_sums_only_the_steps_it_could_compute(self) -> None:
        table, total = MODULE.build_s2_table(self.entrance_rows)
        by_step = {entry["step"]: entry for entry in table}
        # 「配置」= AddLayer は入口台帳にあるので計算できる。
        self.assertIsNotNone(by_step["配置"]["seconds"])
        self.assertAlmostEqual(by_step["配置"]["seconds"], 1 * MODULE.KLM_P + 1 * MODULE.KLM_M, places=6)
        # 「書き出し」はそもそも正準ワークフローの op=None(入口台帳未収録) — FINDING として ops=None。
        self.assertIsNone(by_step["書き出し"]["ops"])
        # 「トリム」「キー」「素材ドロップ」「再生確認」はこの fixture の入口台帳に対応行が無い。
        for step in ["素材ドロップ", "トリム", "キー", "再生確認"]:
            self.assertIsNone(by_step[step]["ops"], f"{step} は fixture に対応行が無いはず")
        self.assertAlmostEqual(total, 1 * MODULE.KLM_P + 1 * MODULE.KLM_M, places=6)


class CliEndToEndTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.entrance_path = self.root / "entrance.md"
        self.normal_map_path = self.root / "normal-map.tsv"
        self.atlas_path = self.root / "atlas.tsv"
        self.entrance_path.write_text(ENTRANCE_ATLAS_FIXTURE, encoding="utf-8")
        self.normal_map_path.write_text(NORMAL_MAP_FIXTURE, encoding="utf-8")
        self.atlas_path.write_text(ATLAS_TSV_FIXTURE, encoding="utf-8")

    def invoke(self, *extra: str) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--entrance-atlas",
                str(self.entrance_path),
                "--normal-map",
                str(self.normal_map_path),
                *extra,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    def test_prints_all_three_section_headers_to_stdout(self) -> None:
        result = self.invoke("--atlas", str(self.atlas_path))
        self.assertEqual(result.returncode, 0, result.stderr)
        out = result.stdout.decode("utf-8")
        self.assertIn("## S0 適合表", out)
        self.assertIn("## S1 到達コストランキング", out)
        self.assertIn("## S2 工程動線", out)
        self.assertIn("CopyLayer", out)

    def test_writes_report_to_out_file_when_given(self) -> None:
        out_path = self.root / "report.md"
        result = self.invoke("--out", str(out_path))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(out_path.exists())
        self.assertIn("## S0 適合表", out_path.read_text(encoding="utf-8"))

    def test_malformed_entrance_atlas_fails_loudly_not_silently(self) -> None:
        bad_path = self.root / "bad.md"
        bad_path.write_text(
            "## 入口台帳(`|` 区切り)\n\n"
            "列: `操作名 | 種別(a-d,自信度) | 現在の入口 | Message/Intent | map行id | "
            "S0期待入口(m:s:p:pref) | 差(S0>a-d 辞書式)`\n\n```\nOnlyTwoFields | oops\n```\n",
            encoding="utf-8",
        )
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--entrance-atlas",
                str(bad_path),
                "--normal-map",
                str(self.normal_map_path),
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"7", result.stderr)


class RealRepositoryFilesSmokeTest(unittest.TestCase):
    """κ調査・normal-map の実ファイルを壊さず読めることの smoke test
    (内容更新はしない — parse できることだけを見る、NON-GOALS 遵守)。"""

    def test_real_entrance_atlas_and_normal_map_parse_without_crashing(self) -> None:
        entrance_path = ROOT / "docs/reviews/2026-08-21-ui-entrance-atlas-survey.md"
        normal_map_path = ROOT / "next/reference/normal-map.tsv"
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "--entrance-atlas",
                str(entrance_path),
                "--normal-map",
                str(normal_map_path),
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        out = result.stdout.decode("utf-8")
        self.assertIn("Undo", out)
        self.assertIn("## S2 工程動線", out)


if __name__ == "__main__":
    unittest.main()
