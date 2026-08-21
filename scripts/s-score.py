#!/usr/bin/env python3
"""S 空間スコア器具第一波(`docs/ui-spatial-score.md` 「器具(実装計画)」2)。

atlas TSV(`next/shell/motolii-shell/tests/suite/entrance_atlas_dump.rs` が
吐く `id/x/y/w/h/content`)× 入口台帳(κ調査
`docs/reviews/2026-08-21-ui-entrance-atlas-survey.md` の「入口台帳」表)×
normal-map(`next/reference/normal-map.tsv`)を読み、S0 適合表・S1 到達コスト
ランキング・S2 工程動線(KLM秒数)を markdown で出す。

read-only な検査器具 — 入口台帳・normal-map の内容は書き換えない
(NON-GOALS)。柵化(CI で落とす判定)は次波。この波は「読める表を出すまで」。

## 各表の限界(隠さず明記 — capsule-gaps 原則)

- S0: 入口台帳の「S0期待入口」列は κ 調査時点の人間判定。本スクリプトは
  normal-map の entries(menu:shortcut:panel:pref)から機械的に dominant な
  入口種別を再計算し、その判定と横並びで**照合**するだけで、人間判定を
  上書きしない(consistent 列が一致/不一致を機械的に示す)。
- S1: Fitts のコストは atlas TSV に widget が実在する操作しか計算できない
  (`ATLAS_CONTENT_BY_OP` に無い操作 = Timeline の canvas 系など、
  `q0_fence.rs` の doc が明記する walker の構造的限界と同じ)。「距離 = 直前の
  操作の入口から」は本波では簡略化して既定シミュレータ窓の中心を起点に
  固定している(ワークフロー連鎖は S2 器具の次波で本実装)。
- S2: 各工程の KLM 演算子内訳(P/K/H/M)は、入口台帳の「入口なし」有無から
  導出する最小モデル(見えるボタン=P+M、入口なしのキー操作=+K+H+M の想起
  コスト、ui-spatial-score.md の「儀式の可視化」節と同じ発想)。演算子の
  精密なタスク分析は今後の改訂対象。
"""

from __future__ import annotations

import argparse
import csv
import math
import re
import sys
from dataclasses import dataclass
from pathlib import Path


# ---------------------------------------------------------------------------
# KLM 定数(docs/ui-spatial-score.md S2、裁定164)
# ---------------------------------------------------------------------------
KLM_P = 1.1  # Pointing(ペイン跨ぎ含む)
KLM_K = 0.2  # Keystroke/クリック
KLM_H = 0.4  # マウス⇄キーボード持ち替え
KLM_M = 1.35  # 思考準備

# Fitts ヒット寸下限(裁定164)— 本スクリプトの計算には使わないが、
# 器具境界の記録として残す(12x12px はタッチ規格でなくドメイン実測値)。
FITTS_MIN_HIT_PX = 12.0

# S1 の簡略化: 「直前の操作の入口」連鎖の代わりに使う固定基準点
# (既定シミュレータ窓 1024x768 の中心 — カーソル休止位置の代表値)。
REFERENCE_POINT = (512.0, 384.0)

# S1: 操作名 → atlas TSV の content 列で探す文字列群。iced widget として
# 実在が確認できる操作だけ機械的に対応づける(κ調査「器具化材料」の
# 「Target はスタイル情報を持たない→S1には十分」を前提に、bounds が
# 取れる操作だけを対象にする)。
ATLAS_CONTENT_BY_OP = {
    "Undo": ["Undo"],
    "Redo": ["Redo"],
    "AddLayer": ["+ Layer"],
    "ToggleSettingsPanel": ["Settings"],
    "TogglePlayback": ["Play", "Pause"],
}

# S2: 正準ワークフロー(ui-spatial-score.md S2 の初期セット)。
# op=None は「入口台帳に対応行がそもそも無い」ことを明示するための工程
# (書き出し = Export は κ調査時点で入口台帳に未収録 — FINDING として出す)。
CANONICAL_WORKFLOW: list[tuple[str, str | None]] = [
    ("素材ドロップ", "素材Import"),
    ("配置", "AddLayer"),
    ("トリム", "Clip move/trim"),
    ("キー", "Key select/削除"),
    ("再生確認", "TogglePlayback"),
    ("書き出し", None),
]

_ENTRY_PRIORITY = ["menu", "shortcut", "panel", "pref"]


# ---------------------------------------------------------------------------
# 入口台帳(κ)parse
# ---------------------------------------------------------------------------


@dataclass
class EntranceRow:
    op: str
    kind_raw: str  # 例 "c(高)"
    entrance: str  # 現在の入口
    message: str
    map_ids: list[int]
    map_field_raw: str
    s0_expected: str  # S0期待入口(m:s:p:pref)列の自由文
    diff: str  # 差(S0>a-d 辞書式)列


_ENTRANCE_HEADER_RE = re.compile(r"列:\s*`([^`]+)`")


def _extract_map_ids(field: str) -> list[int]:
    """`437(3:3:0:0)/466` のような列から map行id 群を取り出す。

    括弧内(entries タプル等、0-4 の一桁)を先に剥がしてから残りの数字列を
    拾う — そうしないと entries の桁を id と誤認する。
    """
    stripped = re.sub(r"\([^)]*\)", "", field)
    return [int(m) for m in re.findall(r"\d+", stripped)]


def parse_entrance_atlas(path: Path) -> list[EntranceRow]:
    text = path.read_text(encoding="utf-8")
    header_match = _ENTRANCE_HEADER_RE.search(text)
    if not header_match:
        raise ValueError(f"{path}: 「列: `...`」の列定義行が見つからない")
    columns = [c.strip() for c in header_match.group(1).split("|")]
    if len(columns) != 7:
        raise ValueError(f"{path}: 列定義が7列でない: {columns!r}")

    try:
        fence_start = text.index("```", header_match.end())
        fence_end = text.index("```", fence_start + 3)
    except ValueError as error:
        raise ValueError(f"{path}: 入口台帳のコードフェンスが見つからない") from error
    body = text[fence_start + 3 : fence_end]

    rows: list[EntranceRow] = []
    for line_no, line in enumerate(body.splitlines(), start=1):
        if not line.strip():
            continue
        fields = line.split(" | ")
        if len(fields) != 7:
            raise ValueError(f"{path}: 入口台帳の行が7列でない(line {line_no}): {line!r}")
        op, kind_raw, entrance, message, map_field, s0_expected, diff = fields
        rows.append(
            EntranceRow(
                op=op.strip(),
                kind_raw=kind_raw.strip(),
                entrance=entrance.strip(),
                message=message.strip(),
                map_ids=_extract_map_ids(map_field),
                map_field_raw=map_field.strip(),
                s0_expected=s0_expected.strip(),
                diff=diff.strip(),
            )
        )
    return rows


# ---------------------------------------------------------------------------
# normal-map.tsv parse
# ---------------------------------------------------------------------------


@dataclass
class NormalMapRow:
    id: int
    category: str
    canonical: str
    freq: int
    menu: int
    shortcut: int
    panel: int
    pref: int
    verdict: str


def parse_normal_map(path: Path) -> dict[int, NormalMapRow]:
    rows: dict[int, NormalMapRow] = {}
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        entries_key = "entries(menu:shortcut:panel:pref)"
        if reader.fieldnames is None or entries_key not in reader.fieldnames:
            raise ValueError(f"{path}: 期待した列 {entries_key!r} が無い: {reader.fieldnames!r}")
        for raw in reader:
            id_text = (raw.get("id") or "").strip()
            if not id_text.isdigit():
                continue
            entries = (raw.get(entries_key) or "").split(":")
            if len(entries) != 4:
                continue
            try:
                menu, shortcut, panel, pref = (int(part) for part in entries)
                freq = int((raw.get("freq") or "0").strip() or "0")
            except ValueError:
                continue
            row_id = int(id_text)
            rows[row_id] = NormalMapRow(
                id=row_id,
                category=raw.get("category", ""),
                canonical=raw.get("canonical", ""),
                freq=freq,
                menu=menu,
                shortcut=shortcut,
                panel=panel,
                pref=pref,
                verdict=raw.get("verdict", ""),
            )
    return rows


# ---------------------------------------------------------------------------
# atlas TSV parse(entrance_atlas_dump.rs の出力)
# ---------------------------------------------------------------------------


@dataclass
class AtlasRow:
    id: str
    x: float
    y: float
    w: float
    h: float
    content: str

    @property
    def center(self) -> tuple[float, float]:
        return (self.x + self.w / 2.0, self.y + self.h / 2.0)

    @property
    def min_dim(self) -> float:
        return min(self.w, self.h)


def parse_atlas(path: Path) -> list[AtlasRow]:
    expected = {"id", "x", "y", "w", "h", "content"}
    rows: list[AtlasRow] = []
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames is None or set(reader.fieldnames) != expected:
            raise ValueError(f"{path}: atlas TSV ヘッダが id/x/y/w/h/content でない: {reader.fieldnames!r}")
        for raw in reader:
            rows.append(
                AtlasRow(
                    id=raw["id"],
                    x=float(raw["x"]),
                    y=float(raw["y"]),
                    w=float(raw["w"]),
                    h=float(raw["h"]),
                    content=raw["content"],
                )
            )
    return rows


# ---------------------------------------------------------------------------
# S0. 慣習段差
# ---------------------------------------------------------------------------


def dominant_entry_type(menu: int, shortcut: int, panel: int, pref: int) -> str:
    counts = {"menu": menu, "shortcut": shortcut, "panel": panel, "pref": pref}
    top = max(counts.values())
    if top == 0:
        return "―"
    for name in _ENTRY_PRIORITY:  # 辞書式優先順位(menu>shortcut>panel>pref)
        if counts[name] == top:
            return name
    raise AssertionError("unreachable")  # pragma: no cover


def build_s0_table(entrance_rows: list[EntranceRow], normal_map: dict[int, NormalMapRow]) -> list[dict]:
    out = []
    for row in entrance_rows:
        matched = [normal_map[i] for i in row.map_ids if i in normal_map]
        if matched:
            freq = max(m.freq for m in matched)
            menu = sum(m.menu for m in matched)
            shortcut = sum(m.shortcut for m in matched)
            panel = sum(m.panel for m in matched)
            pref = sum(m.pref for m in matched)
            dominant = dominant_entry_type(menu, shortcut, panel, pref)
        else:
            freq = None
            dominant = "―"

        consistent = None
        if dominant != "―" and row.s0_expected not in ("", "―"):
            consistent = dominant in row.s0_expected

        low_confidence = (freq is not None and freq <= 1) or row.s0_expected in ("", "―")

        out.append(
            {
                "op": row.op,
                "kind": row.kind_raw,
                "entrance": row.entrance,
                "map_ids": row.map_ids,
                "freq": freq,
                "computed_dominant": dominant,
                "s0_expected_atlas": row.s0_expected,
                "diff_atlas": row.diff,
                "consistent": consistent,
                "low_confidence": low_confidence,
            }
        )
    return out


def render_s0_markdown(rows: list[dict]) -> str:
    lines = [
        "## S0 適合表(入口台帳 × normal-map 照合)",
        "",
        "| 操作 | 種別 | 現在の入口 | freq | 計算dominant | atlas期待 | 照合 | 自信度 | 差(atlas) |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for row in rows:
        consistent = {True: "○", False: "×不一致", None: "―"}[row["consistent"]]
        confidence = "低(要裁定)" if row["low_confidence"] else "―"
        freq = row["freq"] if row["freq"] is not None else "―"
        lines.append(
            "| {op} | {kind} | {entrance} | {freq} | {dominant} | {expected} | {consistent} | {confidence} | {diff} |".format(
                op=row["op"],
                kind=row["kind"],
                entrance=row["entrance"],
                freq=freq,
                dominant=row["computed_dominant"],
                expected=row["s0_expected_atlas"] or "―",
                consistent=consistent,
                confidence=confidence,
                diff=row["diff_atlas"],
            )
        )
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# S1. 到達コスト(Fitts)
# ---------------------------------------------------------------------------


def fitts_cost(distance: float, dim: float) -> float:
    dim = max(dim, 1.0)
    return math.log2(distance / dim + 1.0)


def find_atlas_row(atlas_rows: list[AtlasRow], contents: list[str]) -> AtlasRow | None:
    for row in atlas_rows:
        if row.content in contents:
            return row
    return None


def build_s1_ranking(
    entrance_rows: list[EntranceRow],
    normal_map: dict[int, NormalMapRow],
    atlas_rows: list[AtlasRow] | None,
) -> tuple[list[dict], list[dict]]:
    ranked: list[dict] = []
    unranked: list[dict] = []

    for row in entrance_rows:
        matched = [normal_map[i] for i in row.map_ids if i in normal_map]
        freq = max((m.freq for m in matched), default=0)
        no_entrance = "入口なし" in row.entrance

        if no_entrance:
            ranked.append(
                {
                    "op": row.op,
                    "freq": freq,
                    "cost": math.inf,
                    "cost_source": "入口なし(κ台帳)",
                    "score": math.inf,
                }
            )
            continue

        contents = ATLAS_CONTENT_BY_OP.get(row.op)
        atlas_row = find_atlas_row(atlas_rows, contents) if (contents and atlas_rows) else None
        if atlas_row is None:
            reason = "atlas TSV 未指定" if not atlas_rows else "atlas TSV に対応 widget 無し(canvas系 or 未対応)"
            unranked.append({"op": row.op, "freq": freq, "reason": reason})
            continue

        cx, cy = atlas_row.center
        rx, ry = REFERENCE_POINT
        distance = math.hypot(cx - rx, cy - ry)
        cost = fitts_cost(distance, atlas_row.min_dim)
        ranked.append(
            {
                "op": row.op,
                "freq": freq,
                "cost": cost,
                "cost_source": f"atlas:{atlas_row.content or atlas_row.id}",
                "score": freq * cost,
            }
        )

    def sort_key(entry: dict) -> tuple:
        score = entry["score"]
        return (0,) if score == math.inf else (1, -score)

    ranked.sort(key=sort_key)
    return ranked, unranked


def render_s1_markdown(ranked: list[dict], unranked: list[dict]) -> str:
    lines = [
        "## S1 到達コストランキング(freq × Fitts コスト、降順)",
        "",
        "| 順位 | 操作 | freq | cost | freq×cost | 出所 |",
        "|---|---|---|---|---|---|",
    ]
    for rank, entry in enumerate(ranked, start=1):
        cost = "∞" if entry["cost"] == math.inf else f"{entry['cost']:.2f}"
        score = "∞" if entry["score"] == math.inf else f"{entry['score']:.2f}"
        lines.append(f"| {rank} | {entry['op']} | {entry['freq']} | {cost} | {score} | {entry['cost_source']} |")

    if unranked:
        lines.extend(
            [
                "",
                "### 幾何コスト未計算(atlas に widget 対応なし)",
                "",
                "| 操作 | freq | 理由 |",
                "|---|---|---|",
            ]
        )
        for entry in unranked:
            lines.append(f"| {entry['op']} | {entry['freq']} | {entry['reason']} |")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# S2. 工程動線(KLM)
# ---------------------------------------------------------------------------


def klm_operators_for(row: EntranceRow) -> dict[str, int]:
    # ベース: 1回のポインティング(入口へ到達)+1回の判断(このまま進めるか確認)。
    ops = {"P": 1, "K": 0, "H": 0, "M": 1}
    if "入口なし" in row.entrance:
        # 「儀式の可視化」(ui-spatial-score.md S2): 表面に手がかりが無い分、
        # ショートカットの想起(M)・キー入力(K)・マウス→キーボードの
        # 持ち替え(H)が追加でかかる。
        ops["K"] += 1
        ops["H"] += 1
        ops["M"] += 1
    return ops


def klm_seconds(ops: dict[str, int]) -> float:
    return ops["P"] * KLM_P + ops["K"] * KLM_K + ops["H"] * KLM_H + ops["M"] * KLM_M


def build_s2_table(entrance_rows: list[EntranceRow]) -> tuple[list[dict], float]:
    by_op = {row.op: row for row in entrance_rows}
    out: list[dict] = []
    total = 0.0
    for step, op in CANONICAL_WORKFLOW:
        row = by_op.get(op) if op else None
        if row is None:
            out.append(
                {
                    "step": step,
                    "op": op or "―",
                    "ops": None,
                    "seconds": None,
                    "note": "κ入口台帳に対応行なし(FINDING: この工程の入口が調査対象外)",
                }
            )
            continue
        ops = klm_operators_for(row)
        seconds = klm_seconds(ops)
        total += seconds
        out.append({"step": step, "op": row.op, "ops": ops, "seconds": seconds, "note": row.entrance})
    return out, total


def render_s2_markdown(table: list[dict], total: float) -> str:
    lines = [
        "## S2 工程動線(KLM秒数、正準ワークフロー初期セット)",
        "",
        "| 工程 | 操作 | P | K | H | M | 秒 | 現在の入口/備考 |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for entry in table:
        if entry["ops"] is None:
            lines.append(f"| {entry['step']} | {entry['op']} | ― | ― | ― | ― | ― | {entry['note']} |")
            continue
        ops = entry["ops"]
        lines.append(
            "| {step} | {op} | {p} | {k} | {h} | {m} | {seconds:.2f} | {note} |".format(
                step=entry["step"],
                op=entry["op"],
                p=ops["P"],
                k=ops["K"],
                h=ops["H"],
                m=ops["M"],
                seconds=entry["seconds"],
                note=entry["note"],
            )
        )
    lines.extend(["", f"**合計(算出できた工程のみ): {total:.2f}秒**"])
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def render_report(
    entrance_rows: list[EntranceRow],
    normal_map: dict[int, NormalMapRow],
    atlas_rows: list[AtlasRow] | None,
) -> str:
    s0_rows = build_s0_table(entrance_rows, normal_map)
    ranked, unranked = build_s1_ranking(entrance_rows, normal_map, atlas_rows)
    s2_table, s2_total = build_s2_table(entrance_rows)

    parts = [
        "# S 空間スコア — 器具第一波の出力",
        "",
        "read-only 検査器具の出力。入口台帳・normal-map の内容はここでは書き換えていない"
        "(NON-GOALS)。柵化は次波 — この表は人が読んで裁定の材料にする。",
        "",
        render_s0_markdown(s0_rows),
        "",
        render_s1_markdown(ranked, unranked),
        "",
        render_s2_markdown(s2_table, s2_total),
        "",
    ]
    return "\n".join(parts)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--entrance-atlas", required=True, type=Path, help="κ調査 markdown の入口台帳")
    parser.add_argument("--normal-map", required=True, type=Path, help="next/reference/normal-map.tsv")
    parser.add_argument(
        "--atlas",
        type=Path,
        default=None,
        help="entrance_atlas_dump.rs が吐く atlas TSV(未指定なら S1 は幾何コスト計算をスキップ)",
    )
    parser.add_argument("--out", type=Path, default=None, help="出力先(未指定なら標準出力)")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        entrance_rows = parse_entrance_atlas(args.entrance_atlas)
        normal_map = parse_normal_map(args.normal_map)
        atlas_rows = parse_atlas(args.atlas) if args.atlas is not None else None
    except (ValueError, OSError) as error:
        print(f"s-score: {error}", file=sys.stderr)
        return 1

    report = render_report(entrance_rows, normal_map, atlas_rows)
    if args.out is not None:
        args.out.write_text(report, encoding="utf-8")
        print(f"OK: wrote {args.out}")
    else:
        print(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
