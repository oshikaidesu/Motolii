# 引き継ぎ — node登録・実装なぞり・RN probeのSkia Timeline(2026-08-10 後半)

状態: **引き継ぎ記録**
このセッションでmainに入った範囲: `dac04b2f`〜（[前半の引き継ぎ](2026-08-10-session-handoff-friction-removal-and-recovery.md)の続き）。
以後の施工はCodexへ渡す。次の作業者は本書→[統合地図§5.10](../outcome-driven-integration-map.md)→
[implementation-ledger](../implementation-ledger.md)の順に読む。

## 1. 方針の変更（以後の前提）

**仮コードで全体を起こしてから進む進め方を止める。** 普通の製品開発と同じく、
**1つずつ繋いで、繋がったものを見る**へ切り替える（利用者裁定、2026-08-10）。

- 仮コード6区間の`NEEDS_REVISION`は**未解消のまま置く**。修正して通過させる作業を新たに起こさない
- 器具境界決定は生きている。仮コードは非authorityで、closed orderの`AUTHORITY`欄へ引かない
- 施工の担い手はCodex。監督側は現物確認と回収に寄る

## 2. 完成条件を塞ぐ8件をnode化した（`dac04b2f`）

[鎖のgate](2026-08-09-chain-gate-results-and-audio-path.md)が確定した8件は
`docs/reviews/README.md`に索引されながら**台帳・統合地図のnodeへ翻訳されていなかった**。
[統合地図§5.10](../outcome-driven-integration-map.md)と[台帳](../implementation-ledger.md)へ登録した。

**この欠陥は索引漏れではない。索引から発注順へ翻訳する工程が台帳側に無い。**
今も無い。gate結果・observation文書が増えたら、誰かが手でnodeへ写す必要がある。

## 3. 登録直後に実装をなぞり、自分の登録3行を訂正した（本セッション）

利用者の指示で5 nodeの実装を呼び出し側からなぞったところ、**3行が誤り**だった。

| node | 当初 | 訂正後 | 誤りの中身 |
|---|---|---|---|
| `N-IMPORT-AUDIO` | `BUILT_UNWIRED` | **`ABSENT`** | `asset_video_only`の8箇所は全て`document_edit_runtime.rs:1135`以降の`#[cfg(test)]`内。「製品importは常に音声を落とす」は誤りで、**製品import経路が無い** |
| `N-MEDIA-PLACE` | `PARTIAL` | **`BUILT_UNWIRED`** | `prepare_admit_asset`の呼び出しはtestのみ、`motolii-ui`に`AdmitAsset`が0件。admission自体が製品から一度も呼ばれていない |
| `N-PROJECT-NEW` | `PARTIAL` | **`N-PROJECT-ENTRY` `ABSENT`** | Newだけでなく**Openにも入口が無い**。path供給は`--motolii-project`かenv `MOTOLII_PROJECT_PATH`のみ（`AppDelegate.mm:23`） |

**誤りの型は3件とも同一** — 型・Command・関数が実在することを「製品にある」と読んだ。
`#[cfg(test)]`の内側か、呼び出し元が0件かを数えていない。
**発注前に必ず`#[cfg(test)]`境界と呼び出し元数を数えること。**

### 正味の現在地

media鎖は**端から端まで製品コードが0**である。
素材を選ぶ → Assetを受け入れる → Timelineへ置く → 音声を含める、のどこにも製品呼び出しが無い。
実在するのは`Command`型層とtest fixtureだけ。

そして`N-MEDIA-PICK` / `N-PROJECT-ENTRY`(New/Open) / Save Asは
**「native file dialogの席が無い」1件へ収束する**。別nodeとして数えると4件、実体は1件。
`rfd`は`git log --all -S"rfd"`で0件、リポ外にも製品資産としては無い
（rfd probeは製品転記禁止で意図的に非コミット）。**個別発注しない。**

完成条件に対して非対称になっている。**書き出し側は繋がった**（`97830975`で`export-document`
subcommandが音声muxへ到達）が、**取り込み側は空**である。

## 4. RN probeのTimelineをSkiaへ差し替えた（`6fc1456c`）

利用者の「見ている物の確かめ」のため、`spikes/motolii-rn-probe`のnative timelineを
Skia描画に替えた。**参照が出来れば以後はCodexが引き継ぐ**という位置づけである。

- 差し替え前は`timeline_vertices`が20×25の色quadを頂点で並べるだけで、
  [Timeline設計決定](2026-08-08-timeline-design-decisions-and-skia-fixtures.md)の絵と無関係だった
- `spikes/skia-timeline-probe/src/bin/motolii_full.rs`を
  `native-renderer/src/timeline_skia.rs`へ移植。**新しい仕組みは足していない** —
  Stage overlayが既に使っている経路（CPU raster → `write_texture` → 全画面blit）をそのまま使う
- blit textureは`Rgba8UnormSrgb`。SkiaはsRGB byteを書きsurfaceもsRGBなので往復が恒等になる。
  `Rgba8Unorm`だと変換が1回余計にかかり全体が白茶ける
- `playhead`と`selection`だけ状態で動く。fixtureは静止画と同一で**Documentは読まない**
- `timeline_hit_test`を実レイアウトへ写した（Inbox/rail/rulerは`None`、空帯は`-1`）
- test 4件。`MOTOLII_WRITE_PREVIEW=1 cargo test -- --nocapture`でpanel実寸PNGが出る
  （`native-renderer/timeline-rn-probe-preview.png`）

**visual referenceの所在**（Codexはここを読む）:

| 物 | 位置 |
|---|---|
| 移植元の静止画bin | `spikes/skia-timeline-probe/src/bin/motolii_full.rs`（+ `motolii_tl.rs` / `motolii_kf.rs`） |
| 出力PNG | `spikes/skia-timeline-probe/motolii-full.png` / `motolii-tl.png` / `motolii-kf.png` / `timeline-skia-probe.png` |
| renderer経由の実出力 | `spikes/motolii-rn-probe/native-renderer/timeline-rn-probe-preview.png` |
| 意味の正本 | [Timeline設計決定](2026-08-08-timeline-design-decisions-and-skia-fixtures.md) |

**probe境界は動いていない。** 製品コードではなく、`N-OVERLAY`は統合地図で`PROBE_ONLY`のまま。
製品側の`crates/motolii-ui/src/timeline_skia_raster.rs`（`pub(crate)`、`project_timeline`投影を読む）
とは別物であり、**本probeのコードをそのまま製品へimportしない**。

### 既知の限界

- panel下半分が余る。幅1240の論理座標にscaleを合わせており帯6本ぶんしか高さを使わない。
  可変行高は「音声だけ別扱いで足りる、本決定では開けない」と決定済みなので触っていない
- RNアプリ内で動いている状態は**未確認**。`yarn install` + Pods + Xcodeビルドが要る

## 5. 未完・次の一手

1. **RN probeを実際に起動して見る** — `yarn install` → Pods → Xcode。今回は未実施
2. **file dialogの席** — `N-MEDIA-PICK` + `N-PROJECT-ENTRY`を1件として既知実装調査から
3. **`N-MEDIA-PLACE`** — admission接続と配置intentを1粒に束ねない。どちらが先かを先に決める
4. **`N-SOUNDTRACK-WRITE`** — `command.rs`にvariantが0件。M2 Document ownerへ返す粒
5. [前半引き継ぎ](2026-08-10-session-handoff-friction-removal-and-recovery.md)§4の6項目は生きている
   （M2-ASSET-1Cのcapsule先行、RecipeKeyV1 runtime helper、RN probe残りの製品移管、N-OVERLAY移管ほか）

## 6. 保留

**PR追跡の慣習化は保留**。直近60コミット中PR経由は14件で、Codex発注分は全てPR、
**監督側の作業16コミットは全て直接push**という偏りがある。
ただし今日の登録誤り3件はPRにしていても捕まらなかった（捕まえたのは実装なぞり）。
[段差撤廃決定](2026-08-10-main-merge-friction-removal-decision.md)により
**PRをマージ条件にはしない**。慣習化するならその一文を同時に書くこと。

## 7. 追補 — RN Stage B001（2026-08-11）

§4の未確認事項を実機で閉じ、同じRN window内で次を同時表示した。

- 既存RN Browser／Inspector／shell
- 固定commit `954bf95a`の`re_renderer`が描くRect／Circle
- 既存rust-skia Stage overlay
- 既存rust-skia Timeline（`NATIVE` mode）

RerunはRN Stageが既に持つ`wgpu::Device / Queue`から`RenderContext`を作り、offscreen textureへ描く。
Motoliiの既存compositeが同じtop-level surfaceへSkia overlayと合成するため、第二device／queue／surfaceは作らない。
これは`spikes/motolii-rn-probe`の接続probeであり、固定fixtureだけではDocument投影や製品route完成を意味しない。ただし2026-08-11再訂正により、このhost artifact自体は別targetへ移植せず、Document入力接続後に`PRODUCT_SOURCE`へその場で繰り上げる。

### Build ID（用語凍結）

このprobeの連番は **Build ID**、表記は`Bnnn`とする。他の呼称を作らない。
Build IDは依存と描画構成を固定した実機確認単位で、同じ構成の再compileでは変えず、構成を変えた時に進める。
画面にはBuild IDとRN／Rerun／Skiaの実値を併記する。

**B001 = RN `0.81.2` / Rerun `954bf95a` / Skia `0.99.0`**。

初回の白浮きは二つの境界を分けて解消した。Rerun出力textureは`Rgba8Unorm`のままrender targetとし、
Motolii compositeでは許可した`Rgba8UnormSrgb` viewからsampleする。またXcodeは
`native-renderer/target/release/libmotolii_native_renderer.a`をlinkするため、Rust変更後は
`cargo build --release`を先に実行する。debugの`cargo check / test`だけでは古いarchiveが残り、実画面判定にならない。

B001でrelease build、Xcode Debug build、Rust test、Jest、ESLint、`plutil`、実画面の暗色Stageを確認した。
M5-PATH2D-S1のhost seatは`DONE`で、同じartifactの製品source昇格を進める。Path2D固定fixtureは`PROBE ONLY`、Document入力接続は未成立のまま分けて記録する。

## 8. 追補 — RN Stage B002 chroma key（2026-08-11）

B001の既存RN Stageへ、2秒／60 frameの固定MP4 fixtureを追加した。Rerun `re_renderer`のvideo textureを
同じpreview textureへWGSLでalpha合成し、緑背景を抜く。第二device／queue／surface、CPU色変換、再生UIは追加しない。

実画面では動画前景がマゼンタ矩形（左）からシアン矩形（右）へ進み、両状態で緑背景は透過し、
背面のRerun Rect／Circleと既存Skia overlayが維持された。同じ1.25秒を二度選ぶtestでは同一sample indexを返した。
Rust test 6件、Jest 1件、ESLint、release build、Xcode Debug build、`git diff --check`を通過した。

これはchroma key合成能力の`PROBE ONLY`証拠である。直接依存した`re_video`はfixture decodeのpattern利用に限り、
Motolii製品decoder、media owner、Document意味、Preview／Export routeの採択や完成を示さない。
