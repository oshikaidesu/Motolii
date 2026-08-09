# Group Bake 仮コード — プリコンポ解放が GAP-3 で止まっている

日付: 2026-08-08
状態: **観察 / 修理許可でも実装許可でもない**

## 0. 扱い

`AGENTS.md`「findingは権限ではない」に従う。報告と分類のみ。
本文書を根拠に発注・実装・優先度変更・GAP-3の裁定を行わない。

## 1. なぜ書いたか

2026-08-08に7 outcomeを仮コードで書いた際、**M4の呼び出しが一度も現れなかった**。
これを「M4は需要が立っていない」と読んだが、Group Bake（＝プリコンポの代替、Freeze）が
outcomeに入っていなかったためである可能性が高いと判明したため、
**Group Bakeを起点に鎖を書き直した。**

## 2. 結果: `M4_CALLED: 0`

鎖にはM4の8項目すべてが現れたが、**着地先が一つも実在しない。**

| M4項目 | 鎖に現れた | 実体 |
|---|---|---|
| K7a（成果物境界＋atomic commit） | ✅ | 無し（`STOP / RUNTIME ABSENT`） |
| K7b（区間無効化＋世代再利用） | ✅ | 無し（docsのみ） |
| K7c（bake再生置換＋再freeze） | ✅ | 無し（docsのみ） |
| P02（完全key） | ✅ | `STOP / GAP-3` |
| P03 RAM（`foyer-memory`） | ✅ | probe `VERIFIED`・**製品未接続** |
| P05 Disk（`tempfile`） | ✅ | probe `VERIFIED(V1)`・**製品未接続** |
| P06 区間（`rangemap`） | ✅ | probe `VERIFIED`・**製品未接続** |
| P07 schedule（`priority-queue`） | ✅ | probe `VERIFIED`・**製品未接続** |

`RESOLVED: 3 / UNKNOWN: 8 / M4_CALLED: 0`

## 3. 絶対規律6は `NG`（ただし「破れている」ではない）

現行の**非bake経路では規律6は実測で成立**している。
`render_frame`（`motolii-render/src/lib.rs:323`）と`render_graph_cached`（同`:399`）は単一関数へ収束し、
`motolii-export/tests/d3e_preview_export_same.rs`がpixelレベルで一致を検証している。

しかし bake成果物をsourceとして代替評価する分岐（K7c）が`render_graph_cached`に存在しないため、
**「bake有無でpreview/export pixel同一」を検査する対象コードが無い。**

> bake成果物に対する規律6は「保たれている」のではなく「**検査する経路が無いため判定不能**」である。

判定不能を`OK`と書かないため`NG`とした。

## 4. 阻害の連鎖 — 根は GAP-3

```
プリコンポからの解放（Timelineの独自性）
  ← Group Bake / Freeze
    ← K7a  bake成果物producer        ABSENT
      ← P02 完全key                   STOP
        ← GAP-3  同一性format未締結   ← 根
```

`GAP-3`（`docs/backlog.md:86`）:

> 現行`resolve_asset_path`は候補の実在だけを見て**内容指紋を照合せず**…
> `Asset`のhash欄も生文字列と任意欄で**同一性formatは未締結**
> M4 `source_id`とversion付き指紋を共有し、**algorithm、head/tail chunk長、encoding、size、
> 任意full hash、collision時照合を先に閉じる**。歴史案のXXH3/N MiB/hexを**未裁定defaultとして焼かない**

`M4-P02`停止線（decision-index）:

> `Asset.content_hash`は任意文字列で、version／algorithm／chunk／encoding／collision照合の
> **authorityがない**。P02-C1/C2はruntime encoder／fingerprintを**発明せずGAP-3決定まで停止**

## 5. GAP-3 が同時に止めているもの

`GAP-3`は`docs/backlog.md`で**優先度`P2`**、項目名は「メディア再リンク/オフライン素材」である。
Timelineの文脈では誰も見ない位置にある。

しかし本日の鎖により、次の**3つが同時にここで止まっている**ことが判明した。

1. **Timelineの独自性**（プリコンポ解放＝Group Bake）
2. **M4 cache全体**（P02が`STOP`のため採択済みprobe 4件が製品へ接続できない）
3. **GAP-7**（パッケージ化・可搬性。「GAP-3のversion付き指紋を再利用」と明記）

**優先度`P2`の再判定に足る材料である。ただし本文書は優先度を変更しない。**

## 6. Stage側との対比

| 面 | 席 | 規律 | 不足している一つ |
|---|---|---|---|
| **Stage / M5** | **実在**（`LayerSourcePlugin`→`RenderStep::Plugin`→`render_graph_cached`） | 2・6とも**成立** | **C0-Schema**（翻訳層1枚） |
| **Timeline / M4 Bake** | **無い**（K7a不在） | 6は**判定不能** | **GAP-3**（K7aより上流） |

**Stageは「席があって置くものが1つ足りない」、Bakeは「席そのものが無く、しかも上流が止まっている」。**
性質が異なるため、実装順を同列に扱わない。

## 7. UI/UXについて（記録）

利用者の観点では、Timelineの体系的UIは既存DAWに蓄積があり、Ableton参照は
`decision-index`が「**操作トポロジーと視覚言語の範囲に限定**」と既に定めている
（Track/Mixer/instrument ownershipの持ち込みは音楽メタファー撤回で非目標）。

したがってプリコンポ解放の残課題は**UI選択ではなく技術route**である。
「bakeがいつ無効化されるか」の正しさが利用者体験を決め、その正しさは**同一性format**に乗る。
GAP-3は利便性の項目ではなく、**Timeline体験の根**である。

## 8. 非目標

- 本文書を根拠にGAP-3を裁定・実装すること
- 優先度`P2`を変更すること
- K7a／K7b／K7cを発注すること
- 歴史案（XXH3／N MiB／hex）を既定として扱うこと
- bake未実装を「規律6違反」と報告すること
