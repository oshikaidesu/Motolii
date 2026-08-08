# 完成条件の鎖 — 3〜5分・音楽同期のMVを1本書き出す（仮コード）

日付: 2026-08-08
状態: **観察 / 器具（非compile・非authority）/ 鎖のgate未通過**

## 0. 扱い

[仮コード器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)に従う。

- **compileしない。`crates/`へ置かない。authorityにしない**
- **closed orderのAUTHORITY欄へ引かない。** 仕様・schema・公開APIの根拠にしない
- `AGENTS.md`「findingは権限ではない」に従い、**報告と分類だけ**を行う
- 起草は Anthropic。**鎖のgate（§6.45）は別familyで未通過。** 実名・行番号・seam挿入可否は未検証である

base worktree: `/private/tmp/motolii-r2-spine-20260807`（`68546b8d`、背骨4粒統合後）

## 1. なぜ書いたか

`concept.md`の完成条件は sort key として全判定に使われているが、**その鎖自体が一度も書かれていない。**

> **MVを1本書き出せる**: 3〜5分・音楽同期の最終書き出し(音声mux込み)が完成条件

既存7区間のうち、outcome B は映像素材、outcome D は再生、outcome E は書き出し本体を扱ったが、
いずれも「1回置く」「1回再生する」「1回exportする」の**最短形**で書かれており、
**3〜5分・音楽同期・音声mux というスケールを一度も通していない。**

## 2. 呼び出し側

### 段1 — projectを開く

b1の「保存して開き直す」と同一。新規作成経路（`???_create_project_at`）は未接続のまま。

継ぎ目#2（[継ぎ目9件](2026-08-08-call-site-sketch-seams-and-stage-m5-verdict.md)）がここで効く。
`prepare_project_export(&project_path)` は保存済みpathを要求するため、
**既存fileをOpenした場合にしか書き出しへ到達できない。**

### 段2 — 映像素材を入れる

b1の「mediaを入れる」と同一。`???_pick_media_file` / `???_thumbnail` / `???_insertion_index` が残る。

### 段3 — 楽曲を入れる 【本鎖の中心。既存7区間のどこにも無い】

```rust
// asset登録までは映像と同じ経路が使える
let asset_id = doc.assets.allocate(name, "audio/mpeg", content_hash)?;
//   crates/motolii-doc/src/asset.rs:170

// PCM decode / 正準化は実在。48 kHz stereo f32
let pcm = motolii_audio::decode_file(&path)?;   // crates/motolii-audio/src/decode.rs:23
let pcm = motolii_audio::to_canonical(&pcm)?;   // crates/motolii-audio/src/convert.rs:22

// Soundtrack 値型は実在し、validation も持つ
let st = motolii_doc::Soundtrack::try_new(asset_id, start_offset, master_gain)?;
//   crates/motolii-doc/src/schema.rs:163-170（gain範囲・offset符号を検査）

// !! ここで止まる。Document.soundtrack へ書く Command が存在しない
doc.soundtrack = Some(st);          // ← これを行うのは test fixture だけである
???_set_soundtrack_command(st);     // 楽曲を作品へ据える編集操作
???_replace_soundtrack;             // 差し替え
???_set_soundtrack_offset;          // 頭出し（start_offset の編集）
???_set_soundtrack_gain;            // master_gain の編集
```

**2026-08-08 Codex再照合による訂正**: 「音声側で欠けているのはCommand 1点」は狭すぎる。
既存AssetをSoundtrackへ差し替えるだけならsetter系Commandで表せるが、実際の「楽曲を入れる」は
**Asset登録とSoundtrack設定を同じ製品編集として成立させる境界**を必要とする。

```rust
// 以下は望ましいAPIの提案ではなく、現行呼び出し側で埋まらない席の表示である。
let imported = ???_admit_soundtrack_asset(path, probe, content_hash);
// AssetTable::allocateは&mut Documentを直接要し、DocumentWriterにasset版prepare/reserveは無い。

???_queue_soundtrack_edit(imported, old_soundtrack, new_soundtrack);
// DocumentEditQueueのgeneric Applyはcommands.len() != 1を拒否する。
// specialized actionにもSoundtrack/Assetは無い。

let published = runtime.process_next(&mut queue, primary, generation)?;
// commit_commandは durable journal 1 Command → live apply_macro(vec![command]) の順。
// Asset登録をこの1 Commandへ含めるか、別lifecycleとするかは未決である。

runtime.undo()?;
// old Soundtrackの復元に加え、新規Assetを除去するかorphan keepするかも未決。
```

したがって仮コードが確定できるexact gapは、特定のCommand variant名ではなく次の4席である。

1. Asset identityのsingle-writer内prepare／reservation
2. Asset登録とSoundtrack設定のatomic durable edit
3. Undo時のAsset lifecycle（removeかorphan keepか）の既決policy
4. `DocumentEditQueue`／journal／history projectionへの製品route

単一のcomposite Commandで閉じられる可能性はあるが、本書で方式を決めない。

| 面 | 状態 |
|---|---|
| 値型・validation（`Soundtrack::try_new`） | 実在 |
| 永続化（`impl Deserialize for Soundtrack`, `schema.rs:153`） | 実在。project fileに書いてあれば復元される |
| 評価（`AudioProgram`, `program.rs:39` が `doc.soundtrack` を読む） | 実在 |
| mix（`mix_audio`, `crates/motolii-audio/src/mix.rs:60`） | 実在 |
| mux（`mux_soundtrack` / `mux_mixed_pcm`, `crates/motolii-media/src/mux.rs:123`） | 実在 |
| **既存AssetへのSoundtrack編集操作（Command）** | **無い** |
| **新規Asset登録を含むatomic製品編集** | **無い／lifecycle policy未決** |

clip に付随する音声は経路がある（`SetAudioComponentEnabled` / `SetAudioComponentGain`、
`command.rs:374,381`）。**無いのは MV の主役である楽曲bedを据える操作だけ**である。

### 段4 — 音楽に合わせて置く 【完成条件外。2026-08-08 訂正】

> **訂正**: 起草時、本段を完成条件「音楽同期」の実現手段として書いた。**これは誤りである。**
> `concept.md`「3〜5分書き出しが設計に課すこと」は完成条件の含意を自分で列挙しており、
> そこにあるのは**書き出しスループット / 音声mux必須 / DataTrack長（M4 cache）の3つだけ**である。
>
> > **書き出しには音声muxが必須**(映像だけのmp4はMVとして成立しない)
>
> **完成条件の「音楽同期」は段6の音声muxで満たされる。** Beat Grid・snap・BPMは要件に含まれない。
> 本段を完成条件の依存として扱うことは、[M3価値観更新](2026-08-07-m3-integration-zone-value-update.md)§8が
> 非目標とする「完成条件を『一般的な動画編集ソフトに必要な機能』へ読み替えること」に該当する。
>
> 以下は**完成条件を塞がない**観察として残す。

```rust
let bpm: motolii_doc::Bpm = doc.bpm;         // crates/motolii-doc/src/lib.rs:132
let beat = bpm.try_beat_duration()?;         // crates/motolii-doc/src/bpm.rs:66

???_set_bpm_command(new_bpm);                // Soundtrack と同じく Command が無い
???_beat_grid_projection(bpm, viewport);     // Timeline の Beat Grid
???_snap_to_beat(item, beat);                // 拍への吸着

let peaks = motolii_audio::waveform_peaks(&pcm, buckets)?;
//   crates/motolii-audio/src/waveform.rs:8 — 実在
???_timeline_waveform_row(peaks);            // 表示先が無い
```

**ここは「欠落」ではなく「所有者が決まっていて未実装」である。**
[BPM Rhythm Vism決定](../decision-index.md)（2026-07-24）が
「BPM／拍リズムの製品所有者は BPM Rhythm Vism。Coreは時刻・型・接続だけを持ち、
Beat Grid／snapは typed rhythm data の Host projection」と定めている。
`Document.bpm` は「旧Project互換の pre-Vism 入力として当面保持」であり、
**Commandが無いこと自体は決定と矛盾しない。**

未決なのは「v1の完成条件が要求する音楽同期を、Vism未実装のまま何で満たすか」である。

### 段5 — 3〜5分を通して確認する

```rust
let session = motolii_transport::PlaybackSession::open_default(
    program, start_frame, fps, motolii_core::Quality::DRAFT, Some(&gpu),
)?;   // crates/motolii-transport/src/playback.rs:26

// !! 製品 PlaybackSession は単一PCMのままで mixed AudioProgram を受けない = GAP-28 停止線
???_mixed_audio_program_in_product;

// 3〜5分 × 40 layer の preview 圧
???_preview_pressure;   // 統合地図 §5.8 の唯一の明示edge
                        // R3-PREVIEW-PRESSURE requires=[M4-PROVIDER-TARGET]
```

**M4がoutcome側から要求される唯一の地点がここである。** 3〜5分という尺を鎖に入れて初めて現れた。
ただし着地先は Group Bake鎖と同じく `STOP`（`M4-P02` が GAP-3 待ち）。

### 段6 — 書き出す 【通っている】

```rust
let prepared = motolii_cli::prepare_project_export(&project_path)?;  // cli/project.rs:379
let report = prepared.export(&gpu)?;                                  // cli/project.rs:430
//   → export_document_video (crates/motolii-export/src/lib.rs:221)
//     → resolve_audio_export (同 :406)
//        clip audio か非identity retime があれば → AudioExportPlan::MixedPcm
//        無くて doc.soundtrack があれば         → SoundtrackFast（stream-copy）
//        どちらも無ければ                        → None
//     → mux_soundtrack / mux_mixed_pcm (crates/motolii-media/src/mux.rs:123)
//       AudioProgram::mix_audio の正準PCMを AAC encode して mux
```

**音声muxは実装済み**（D6/AG-4、`lib.rs:1,6-7`）。
[v1 delivery決定](../decision-index.md)（2026-07-23、「v1の製品出力は既存`ExportJob`による
音声mux込み完成映像だけに限定する」）どおりの形で成立している。

## 3. `???` 一覧

| # | `???` | 種別 | 根拠・探した場所 |
|---|---|---|---|
| 1 | `set_soundtrack_command` / `replace` / `offset` / `gain` | **`ABSENT`（外部確認済み）** | `Command`列挙に Soundtrack variant 無し。`PropertyId`(`command.rs:55-68`)にも無し。`doc.soundtrack =` の代入は test 5件（`mix_program.rs:98,356`／`u0e2_reference_fixture.rs:89`／`d1a_schema.rs:133`／`d1b_validate.rs:146`）と `new_current`／`limits.rs:538` の `None` 初期化のみ。`ui/` 全体で grep 0件。**`~/Documents/Codex/` 配下も 0件**。ただしこれは既存Assetの編集席だけを示し、新規import全体を1 Commandで閉じる根拠ではない |
| 1a | Asset登録＋Soundtrack設定のatomic編集境界 | **`ABSENT / POLICY_GAP`** | `AssetTable::allocate`は生Document上、`DocumentWriter`にasset prepare/reserve無し。generic `DocumentEditAction::Apply`はmulti-commandを拒否し、specialized actionにもSoundtrack/Asset無し。`commit_command`はjournal 1 Command→live applyのため、AssetのUndo/remove-or-orphan policyが閉じるまでCommand形を確定できない |
| 2 | `set_bpm_command` | `ABSENT` だが**決定と整合・完成条件外** | 所有者は BPM Rhythm Vism（2026-07-24）。`Document.bpm` は pre-Vism 互換入力 |
| 3 | `beat_grid_projection` / `snap_to_beat` | `UNDECIDED`・**完成条件外** | typed rhythm data の Host projection と決定済み。供給する Vism が未実装。[vism-kit-model §9](../vism-kit-model.md#9-現行bpmからbpm-rhythm-vismへどう移るか)の未決点1〜5 |
| 4 | `timeline_waveform_row` | `BUILT_UNWIRED` 側・**完成条件外** | `waveform_peaks`(`waveform.rs:8`)は実在。決定-index 258 が「native Timeline waveform／row操作は未実装」と明記。採否は baseline 手続き（採用item 0）の領分 |
| 5 | `mixed_audio_program_in_product` | `ABSENT`（停止線） | GAP-28。決定-index 258 の停止線 |
| 6 | `preview_pressure` | `ABSENT`（上流STOP） | `M4-PROVIDER-TARGET` 待ち。M4-P02 は GAP-3 で `STOP` |
| 7 | `create_project_at` / `save_as` | `UNDECIDED` | b1と同一。継ぎ目#2 |
| 8 | `pick_media_file` / `thumbnail` / `insertion_index` | b1と同一 | 外部照合**未実施**（rfd probe が P06-C1 にある） |

**`SKETCHED: 6段 / RESOLVED: 14 / UNKNOWN: 9 / M4_CALLED: 1（着地先STOP）`**

## 4. 最大の発見 — 区間の隙間に落ちていた

> **「音声mux込み」は完成条件の文言そのものであり、mux側は完成しているのに、
> 楽曲を作品へ入れる編集操作が存在しない。**

既存7区間はこれを一度も検出していない。outcome B は映像資産を、outcome D は再生を、
outcome E は export 本体を見ており、**Soundtrack は3区間の隙間に落ちていた。**
[継ぎ目検査](2026-08-08-call-site-sketch-seams-and-stage-m5-verdict.md)も
Use→Tune→…の連続体を辿ったため、この隙間を通らなかった。

実害の形（**未検証の疑い。断定しない**）:
`resolve_audio_export` は `doc.soundtrack` が `None` で clip audio も無ければ
`AudioExportPlan::None` を返す。**製品routeで作った作品を書き出すと無音になるが、
error にならない。** 完成条件の中心が silent failure になりうる。

## 4.5 完成条件の読み方を確定した（起草時の誤りの訂正）

鎖を書く過程で、**起草者自身が完成条件を読み替えていた**ことが判明した。記録しておく。

| | 起草時の読み | `concept.md`の実際 |
|---|---|---|
| 「音楽同期」 | 拍に合わせて編集できること（Beat Grid / snap） | **音声muxが付いていること** |
| 依存 | BPM Rhythm Vism（未実装）に依存する | **依存しない。muxは実装済み** |

`concept.md`「3〜5分書き出しが設計に課すこと」が挙げる含意は3つだけである — 書き出しスループット、
**音声mux必須**、DataTrack長（M4 cache対象）。BPM・拍・grid はどこにも現れない。

**器具はこの誤りを自力で検出しなかった。** 段4を書いている間、`???`は正しく出ていたが、
それが「完成条件を塞ぐ`???`」なのか「完成条件外の`???`」なのかを器具は区別しない。
仕分けは sort key を正本で読み直して初めてできた。

> **`???`の数は塞いでいる量ではない。** 器具境界決定§6.45の検査4点に、
> 「その`???`は完成条件を塞ぐか」は含まれていない。**鎖のgateでもこの誤りは出ない。**

## 5. 相互検証（規約§4の必須手続き）

| 本鎖の `???` | survey / 地図側 | 判定 |
|---|---|---|
| #1 Soundtrack 編集操作 | **地図に node として存在しない** | **不一致 → 最重要。** `N-OVERLAY` と同型（地図が持っていないのに、呼び出し側を書くと必ず現れる） |
| #5 mixed AudioProgram | outcome D `ABSENT`（GAP-28） | 一致 |
| #6 preview pressure | 地図§5.8 の明示edge | 一致 |
| #4 waveform row | 決定-index 258 の未実装記述 | 一致 |
| #2 #3 BPM / Beat Grid | 地図に node 無し | 不一致だが**決定側で所有者が確定済み**（Vism）。地図の欠落であって未決ではない |

**地図に無い node が2系統（Soundtrack編集、BPM projection）出た。**
`N-OVERLAY` に続き、**「地図は M3 の接続粒を列挙したが、完成条件そのものの鎖では列挙し切れていない」**
という同じ機序が再現している。

### 5.1 media投入との収束（2026-08-08 Codex再照合）

本鎖のSoundtrack importと、b1の映像media投入は、異なるUIや値型から始まるが同じ地点で止まる。

```text
映像file → Asset登録 → AddTrackItem
楽曲file → Asset登録 → Soundtrack設定
                 ^
                 └─ DocumentWriter / Command / journal / Undoに正規route無し
```

これはGAP-3のcontent fingerprint形式とは別である。現在の任意`content_hash`を仮に受理しても、
AssetTable mutationをsingle writerとdurable historyへどう載せるかは残る。
既に[仮コードfinding返却](2026-08-07-call-site-sketch-findings-return.md)がM2 Document／journal／Undo ownerへ
返した「Asset登録がUndoで戻らない疑い」と同じ根であり、Soundtrack専用ownerが独自に決めてよい境界ではない。

したがって次の2つを分ける。

- **既存AssetのSoundtrack差し替え／offset／gain**: Soundtrack編集の狭い欠落
- **fileから新規楽曲を入れる**: mediaと共有するAsset admission/lifecycleが先に必要

本書は後者を`N-SOUNDTRACK-WRITE`一粒へ畳まず、M2 ownerの返却が閉じるまで`POLICY_GAP`として保つ。

## 6. 新規node候補（発注ではない）

### `N-SOUNDTRACK-WRITE` — 楽曲bedを作品へ据える編集操作

Soundtrack setterは`ABSENT`（repo内外とも確認済み）。値型・validation・評価・mix・mux・永続化は
実在するが、実際のimportにはAsset登録のsingle-writer／journal／Undo境界も必要である。
したがって`BUILT_UNWIRED`の単純接続ではない一方、**Command 1系統だけ**ともまだ確定できない。
既存Assetの差し替えと新規Asset導入を分けるか、atomic lifecycleへ畳むかはowner裁定前である。

完成条件の sort key で見ると、これは**映像素材投入と同格かそれ以上**に前段にある
（MV制作は楽曲が先にあり、映像を音に合わせるため）。

**着手前に既知実装 preflight を通すこと。** 本書は preflight を実施していない。

## 7. 非目標

- 本書を根拠に `N-SOUNDTRACK-WRITE` を発注・実装すること
- Command 列挙・`PropertyId`・Document schema の変更を提案すること
- BPM Rhythm Vism の前倒し、`Document.bpm` の意味変更
- GAP-28／GAP-3 の裁定、休止契約の解除
- 「無音書き出し」を確定した欠陥として外向きに扱うこと（未検証の疑いである）
- 本鎖を gate 未通過のまま capsule／発注の根拠にすること
