# M2基盤再締結・独立追補実コードレビュー（2026-07-18）

ステータス: **ローカル統合候補の実コード審査完了／P0=0・P1=0**。これは
[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)のCに必要な
追補レビュー記録であり、単独ではゲート解除、main到達、remote CI成功、M3入場を意味しない。

## 役割と対象

- 実装担当: Cursor Composer 2.5 Fast（各隔離worktree）
- 独立反対側レビュー: Cursor版Grok 4.5（read-only）
- 証跡再実行・最終判定: Codex
- 対象ローカル統合候補: `codex/m2-reclosure-egui`、`e58f0a4`
- 固定面:
  1. schema / migration / validate
  2. command / single-writer / snapshot / journal / project session
  3. Document→render評価（D3 / D3e / D3f）
  4. unknown plugin保持とexport拒否
  5. semantic oracle更新禁止とworkspace回帰

レビューはテスト緑を十分条件にせず、期待値・oracleの書換え、fixture special-case、
生JSON走査によるtyped境界迂回、公開raw mutation、暗黙migration、partial mutation、
重複planner/helper、camera二重投影、非既定cameraのapproximate skipを反対側から確認した。

## 判定

| 優先度 | 件数 | 判定 |
|---|---:|---|
| P0 | 0 | 統合停止事項なし |
| P1 | 0 | 再締結を止める実コード事項なし |
| P2 | 6 | 下記。再締結ブロッカーにはしないが後続で追跡 |

初回レビューではD3fに次のP1を確認した。

1. 既定`CompCamera`を持つDocumentがCAM-G0 semantic oracle bytesを保持する審判がない
2. 非既定Document cameraを含むpreview/export同一審判がない

`e58f0a4`で次を追加し、同じ固定面を再レビューした結果、P1は解消した。

- `current_default_camera_document_gpu_matches_cam_g0_oracle`
- `migrated_default_camera_document_gpu_matches_cam_g0_oracle`
- `non_default_camera_document_gpu_does_not_match_cam_g0_oracle`
- `non_default_camera_preview_and_export_share_final_render_path`
- `non_default_camera_preview_differs_from_default_camera_preview`

CAM-G0照合は`build_document_frame_graph`が返す`built.camera`を正規GPU経路へ渡し、
既存oracleと`tol::EXACT`で比較する。preview/export審判は両方を
`Quality::FINAL`で通し、非既定cameraがdefaultと異なることを別の負例で固定する。
oracle、classification、生産コード、公開API、schema、migrationの期待値は変更していない。

## 面ごとの実コード証跡

| 面 | ローカルcommit | 主な審判 |
|---|---|---|
| Shared Effect schema / command / migration | `a23a4ad`、`74af37e`、`02192c2`（既存main記録）、統合候補上の追補 | `d1l_effect_definition`、`d1l_v2_lifecycle_commands`、`d1l_writer_prepare`、`d1l_journal_v1_compat` |
| Shared Effect評価 | `69dab03` | `d3e_shared_effect_eval`のP1〜P6・N1・N3、`d3e_preview_export_same::p7_preview_and_export_share_final_render_path` |
| Project sidecar / session | `5e909f0` | `d1m_sidecar_paths`、`d1m_session_lock`、`d1m_legacy_migration`、`d1m_public_api_closure` |
| CAM-G0 / schema | `5210f4e`、`1d768d3` | `cam_g0_planar_identity_matches_semantic_oracle`、`d1j_comp_camera` |
| runtime camera | `a325a85` | D1k runtime camera、必須camera入力、aspect mismatch、Draft縮退 |
| Document camera接続 | `c967849`、`e58f0a4` | D3f方程式、tiny非既定skip拒否、独立inverse-UV oracle、Document→CAM-G0、preview/export同一 |
| migration / validate | 統合候補履歴 | `d1e_migrate`、`d1h_validate`、`d1j_comp_camera`、OverrunMode拒否 |
| command / ownership | 統合候補履歴 | `d2_command`、`d8_ownership`、`mut_document_deny` |
| unknown / export | 統合候補履歴 | `d1f_unknown_plugin`、`d6_audio_mux`のdegraded/future-version/contract-only拒否 |
| journal / recovery | 統合候補履歴、`5e909f0` | `d1d_journal`、`d1m_session_lock`、`d1m_public_api_closure` |

## Codex再実行証跡

`e58f0a4`で次を再実行した。

```text
git diff --check
cargo fmt --all -- --check
MOTOLII_REQUIRE_GPU=1 cargo test -p motolii-doc --test d3f_comp_camera_eval -- --nocapture
  11 passed。current/migrated CAM-G0はいずれもmax byte diff=0
MOTOLII_REQUIRE_GPU=1 cargo test -p motolii-export --test d3f_preview_export_camera -- --nocapture
  2 passed。preview/exportはmax byte diff=0
MOTOLII_REQUIRE_GPU=1 cargo test -p motolii-render --test cam_g0_planar_identity -- --nocapture
  1 passed。max byte diff=0
MOTOLII_REQUIRE_GPU=1 cargo test -p motolii-export --test d3e_preview_export_same -- --nocapture
  1 passed。max byte diff=0
./scripts/check-golden-update-policy.sh d68e9bb
  OK（本チケット開始点からsemantic oracle変更0）
cargo test -p motolii-plugin
  全緑
cargo test -p motolii-testkit --test purity
  10 passed
MOTOLII_REQUIRE_GPU=1 cargo test --workspace
  Composer検証で全緑
```

`./scripts/check-golden-update-policy.sh`を`origin/main`既定で実行すると、統合候補へ
既に含まれる`d1i3_lookat_follow.rs`の`new_v1`→`new_current`変更を検出する。
本D3f追補差分によるoracle変更ではないため、本チケット開始点`d68e9bb`を比較基準に
ゼロ差分を確認した。再締結PRでは、remote mainの実際のbaseに対するpolicy結果を
CI URL付きで改めて記録する。

## P2追跡

1. migrated CAM-G0審判はmigration後にCAM-G0 sceneを載せるため、旧scene全体の
   migration pixel保持はD1e/D1jコーパスとの組合せで審判している
2. preview/exportの非既定cameraはcodec alpha差を避けるためcenter=0、roll=0、
   height=0.75のzoom-in。center/roll/height一般式はD3fの独立inverse-UV oracleで別に固定済み
3. 再締結ゲート本文の「現在地」はD3e/D1m/camera未着手時点の歴史記述
4. D1k解凍記録の冒頭にD3f WAITの歴史記述が残る
5. `DocumentWriter::edit`はprelude互換の公開口として残り、追加利用禁止をAST/ownership側で追跡する
6. stable IDの`from_raw` / `peek_next`は復元・test用途の公開面が残る

## 記録する実証跡（#217 / main）

前提コードのremote証跡:

- PR: `https://github.com/oshikaidesu/Motolii/pull/217`
- main merge SHA: `fa6850a3981c319973cf120e64976e6f8d79b969`
- PR CI: `https://github.com/oshikaidesu/Motolii/actions/runs/29646476618`
- push CI: `https://github.com/oshikaidesu/Motolii/actions/runs/29646451595`

上記は[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)のA〜C完了証跡表におけるコード到達・CI成功の根拠である。単独ではゲート解除、M3入場を意味しない。

## 記録する実証跡（#218 / main — 再締結宣言）

再締結解除宣言のremote証跡:

- PR: `https://github.com/oshikaidesu/Motolii/pull/218`
- main merge SHA: `cc87d8aa1d2cf2a2d24937d43e66c11df4aa769c`

上記は[M2基盤再締結ゲート](2026-07-15-m2-foundation-reclosure-gate.md)の再締結解除宣言がmain上で発効した根拠である。コード到達証跡（#217）とは別である。単独ではM3入場を意味しない。

## 未達（本追補レビューでは主張しない）

1. **別M3入場PR**（U0/U1依存の再翻訳と実装許可）

再締結解除宣言（PR [#218](https://github.com/oshikaidesu/Motolii/pull/218) / `cc87d8aa1d2cf2a2d24937d43e66c11df4aa769c`）はmain上で発効済みである。実コード固定面のP0/P1は0だが、**M3製品実装の着手許可は自動解禁されず**、別M3入場PRのみがU0/U1依存を最新mainへ再翻訳する。
