# 引き継ぎ — 段差撤廃・歴史回収セッション(2026-08-10)

状態: **引き継ぎ記録**
このセッションでmainに入った範囲: `c8bfba1e`(PR #467)〜`957ea893`。
次の作業者は本書→[回収監査](2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)→[implementation-ledger](../implementation-ledger.md)の順に読めば、歴史調査なしで現在地に立てる。

## 1. 構造変更(以後の全作業の前提)

- **mainマージの段差は3層とも撤廃済み**([決定](2026-08-10-main-merge-friction-removal-decision.md))。
  GitHub ruleset「M2E-2 require code owner review」は削除済みで**mainへ直接push可**。事前gateなし。
  検証laneは事後観測に降格し、redはmain上でfix-forwardする(本セッションでUTF-8破損1件を実演修復)。
  虚偽green報告禁止とtest/golden意味保護は維持。**事前検証をマージ条件として再提案しない。**
- 成果は当日中にmainへ。ブランチ・リポ外workdir・未コミットworktreeに滞留する成果は完了と数えない。
- `scripts/check-stray-work.sh`が滞留5層(ローカル/リモートブランチ、worktree、リポ外workdir、
  docsのリポ外パス参照)を1コマンドで観測する。歴史調査を再発明しない。

## 2. このセッションで回収したもの(全てmain到達済み)

| 種別 | 内容 | commit |
|---|---|---|
| 決定文書 | 段差撤廃決定+回収監査 | `54c99388` ほか |
| 実装 | Skia Timeline 6コミット+2,437行(`timeline_skia_raster.rs`、`stage_overlay_gpu/raster.rs`、`MotoliiTimelineComponentView.mm`、Fabric spec) — ローカル専用branch `codex/m3-collaborative-bringup-20260810`から | `2354a5e7` |
| 実装 | r2 position key intents(+494行)、single-writerガード限定化、group bounds draft棄却記録2本 | `4e92241b`〜`986baeff` |
| 文書 | 未コミットworktreeから`2026-08-06-storage-to-gpu-direct-io-design-observation.md`+索引登録 | `cb8c5e5b`直前 |
| 文書 | ローカル専用branchから決定文書20本(旧local-alpha線のCU-206C/210P/210R/211/212等16本、Dock/native drag技術移管、WGSLホットリロード作者経路、全docs横断棚卸し、egui/taffy spike観察) | `cb8c5e5b`, `d80d0c0b` |
| 実装 | M4検証ハーネス~3,000行(`allocation.rs`、`m4_validation` module、CLI4本、inventory/yuv-plan test — K1a APIと無改変適合、10/10 pass) | `d80d0c0b` |
| 発注回収 | `m4_tier_transfer_contract`のK1a API適合(composer-2.5発注、6/6 pass・assertion 28本維持を検収) | `86d5b148` |
| 発注回収 | **M4-P02-CODEC**: `RecipeKeyV1`/`ArtifactDigest` canonical codec+mutation corpus 8 test(grok-4.5-high発注、検収済み) → ledger 7B `DONE` | `f731384c` |
| probe | `spikes/skia-timeline-probe/`(bin15本+depth-rail v4〜v14)、`spikes/motolii-rn-probe/`(App.tsx 660行) — リポ外`~/Documents/Codex/`から移管、provenance README付き | `7cc861d7` |

台帳・地図の実態同期も完了: ledger 6A(M2-ASSET-1A `DONE`訂正)/7A/7B、m4地図P02-C1/C2、
統合地図N-OVERLAY、RN実行地図R1-BROWSER/R1-HOST-EDIT `DONE`、ui-reference-map、backlog INF-6/GAP-3。

## 3. 歴史保全(削除ゼロの方針)

棄却・超越済みの作業は**歴史事実として残す**(利用者方針)。削除しない。

- `rescue/*` 5本(originへpush済み): `m2-asset-1a-draft-20260809`(「qualified diffなし」とされたセッションの実+723行別実装)、`r2-intents-draft-20260809`、`r0-host-remap/seat-impl-draft-20260807`、`storage-gpu-observation-20260806`
- UNIQUE code branch(originへpush済み): `codex/m3-p06-c1-mac`(**名前は偽り、実体はM3 local-alpha製品接続1,438行**)、`codex/m4-k1a-validation-20260729`、`codex/m3-browser-panel-spike`、`codex/m3-egui-rerun-mock`(4,700行)、`codex/m5-render-common-foundation`
- 全裁定結果(RECOVERED/SUPERSEDED/UNIQUE、根拠文書名付き)は[回収監査](2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md)§5と本セッション記録にある。特記: rfd probe実装は「disposable probe debt・製品転記禁止」で**意図的に非コミット**(消えたのではない)

## 4. 未完・次の一手(優先順)

1. **M2-ASSET-1C**(ledger 7A、`ISSUE / CONTRACT CLOSED`) — 依存充足済みだが、同系統wide orderが3回空振りした履歴がある。**audio/export/decodeのcall siteを跨ぐcapsuleを設計してから**発注する。次善はfresh full hash再照合の一枝だけを最初の粒に切ること
2. **RecipeKeyV1 runtime key helper** — render graphからの実値収集。codec(済)とは別ticket(m4地図P02-C1に明記)
3. **RN probe残り8割の製品移管** — R1/R2粒。[RN実行地図](../m3-rn-runtime-execution-map.md)R1節に「発注前に両probeを読め」注記済み
4. **N-OVERLAY移管・接続** — `spikes/skia-timeline-probe`→`motolii-ui`。skia-safe依存は着地済み、コードはmainに`timeline_skia_raster.rs`のみ
5. 残リモートbranch 18本のうちコード先行分の回収判断(renewal-mainは退役CI修理なので非回収と裁定済み)
6. local-alpha製品接続1,438行の現行runtime再統合(routeはRN再基線で変わったため、コード直接mergeでなくR2粒への翻訳)

## 5. 発注の実務メモ(今回実測)

- 手順は`~/.claude/skills/motolii-dispatch`のskillどおり。**warm target配布(APFS `cp -Rc`)は必須**、無いとsandboxでRust laneが回らない
- 「止まる許可」を書いた発注は両方とも正しく動いた(EVIDENCE_GAP none、逸脱ゼロ、FINDINGでオーダー側の記述ミスまで指摘してきた)
- 検収は: exit 0を信用しない→diff実在→RETURN 6欄→oracle自走再現→assertion数等の機械照合→supervisor commit
