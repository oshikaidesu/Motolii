# 副社長の査定 — 営業 41 社の売り込み(2026-09-03)

利用者(社長)の依頼: 3 波の売り込み([1](2026-09-03-vendor-pitches.md)・[2](2026-09-03-vendor-pitches-2.md)・[3](2026-09-03-vendor-pitches-3.md))を「割に合わない部分があるかもしれない」の目で査定する。副社長は会社(Motolii)の勘定で書く。読み取り専用、code は触っていない。

前提の裏取り: 台帳は ☑284 / ☐166。営業の目玉のうち CT1〜CT3・AK2 は 19:32 の commit で既に済(18:39 の締め後)、B1・B2 は 16:59、R9 は 17:08 に CPU 式で代替済。以下は「まだ買っていない物」の勘定。

## 7. 総評 — なぜ面白くなかったか

三つとも当たっているが、比重は違う。

1. **自前が既に少ない(最大の理由)**。`owned-budget.tsv` は 0/0/0/0/3/2/2 — 描画・shader・decode・store は全部委譲済で、残る自前は Intent・Session・Document の「編集の意味」だけ。憲法がそこは自作せよと言っているので、営業がカタログを開いても売り場が無い。41 社で本当に新しい能力を持ってきたのは 4 つ(OS の書体台帳、widget の字形、NSMainMenu、可変速再生)で、他は「既に lock に居る API の配線」。
2. **宿題の型が営業向きでない**。☐166 のうち営業が触れたのは ≒40、そのうち既決や上流 PR 待ちを除くと ≒25。残りは 縦書き・ルビ(誰も持っていない)、ST5/ST16/E2/E5/E6 のような操作の意味、CD/PD/UX の裁定 — カタログでは解けない。
3. **営業の形の限界**。第 1 波の「feature 2 語で 20 件」は幸運で、第 2 波以降は 1 社あたり数行の hygiene に収束(第 3 波 11 社の最大収穫は rfd の 8 行)。ただし収穫が別にあった: **誤診の訂正 5 件**(BU4・B8・Lottie・CV2 の根・WaveformBuilder)と、**既決の齟齬の露見**(下記 361 と 491、blend_preview.rs 129 行と裁定 495)。これは市場でなく監査だった。面白くないのは正しい結果で、次に同じ形を回す価値は無い。

## 3. 買う物 7 件(「触れて満足」に直結する順)

| 順 | 買う物 | 効く宿題 | Motolii 側の本当の手数 | 柱 | 実窓の合否 |
|---|---|---|---|---|---|
| 1 | **cosmic-text CT4+CT5+CT7**(weight・runs・wrap) | S6・S8・S10・L7・LD4 残 | 営業 ≒40 行 + Inspector の Text 節に family/weight/wrap の行 ≒40 行 + Lottie enums 1 枝。**≒90 行** | cosmic-text 0.19(既存、release) | W3→W6 で字が太る、枠を入れると折り返す、1 行の途中で色が変わる |
| 2 | **vello VL2〜VL5**(parley の `Type` 1 つ) | V8・ST10・S12 | 営業 ≒75 行 + `DocumentConfig.font_ctx` の配線(host.rs/gui.rs)。Cargo に parley 1 行(lock 済、compile 増無し)→ 裁定 E | parley 0.11(blitz が既に link) | 目盛に秒の数字、取っ手を掴むと脇に値、印の右に名前 |
| 3 | **rerun R5+R3+R8**(グリッド・縁取り・Size) | ST15 のグリッド分、ST5/ST16 の足場、ST11/F19 の再発止め | 営業 ≒65 行 + `TexturedRect.outline_mask` を層ごとに立てる配線。rectangles.rs:249 に口が在るのを確認 | re_renderer(fork、憲法の中) | 視点を回しても格子が世界に居る、選択の縁が α の形に沿う |
| 4 | **cpal AU1 + rtrb RT1(+AU2)** | A1 の土台、A2 の残余、AirPods 切替 | 営業 ≒40 行。**pause() が CoreAudio で通るかは実測が先**(営業自身の注記) | cpal 0.18 / rtrb 0.4(既存) | seek で音が途切れない・開き直しの空白が消える、出力先を替えても止まらない |
| 5 | **Apple AP1+AP2**(NSMainMenu + app delegate、objc2-app-kit 直) | H1〜H4・H12・H14・⌘Q の未保存素通り | 営業 ≒100 行 + Intent の match を closure から持ち上げる(MD5 の指摘、≒30 行)+ DOM menubar ≒335 行の撤去 → 裁定 A | objc2-app-kit 0.3.2(既存直接依存、feature 2 語) | ⌘Q で Save/Don't Save/Cancel が出る、別窓にも menu、Window menu に窓一覧 |
| 6 | **rfd RF1〜RF3** | H5・H6 の仕上げ | 8 行 | rfd 0.17(既存) | ⌘S の panel が窓に貼り付く、button が [Don't Save]…[Cancel][Save] |
| 7 | **tiny-skia TS5(条件付き)** | CV2 の残り(縁の黒ずみ) | 版上げ 1 語 + 2 行。**18:59 の α 修正の後も実窓で縁が暗い時だけ** | tiny-skia 0.11→0.12(release)→ 裁定 E | 縮小した白文字の縁が灰でなく白 |

補欠: ISF IS1(bool/color 4 成分、25 行、C12)。7 件の合計は Motolii 側 ≒500 行、新規 crate 0、直接依存の追加 1(parley)。

## 1. 割に合わない物(名指し)

- **dioxus-dnd DD1〜DD10(2 度目)**: 裁定 504(2026-09-02)が「完成 DnD component は `get_client_rect().await` が Blitz の poll と二重 borrow するので不採用、state machine だけ使う」と既決。営業は同じ `get_client_rect` 経路を「動く見込み、窓で実走はしていない」で ≒270 行の**新規**を売っている。(b)(c) に該当。
- **Lottie LT1〜LT4**: 裁定 L270(v1 の出力は MP4 のみ、Lottie は完成条件外)を覆す。(c)(f)。
- **rerun R1 の re_time_ruler 半分**: fork の `re_time_ruler/Cargo.toml` は egui・re_ui に依存 — Motolii の lock に egui を呼び戻す。裁定 L51 の「viewer 層を引かない」に反する。`re_format::next_grid_tick_magnitude_nanos` と `format_compact` の半分だけ可。(c)(d)。
- **rerun R2 PickingLayerProcessor**: 当たり判定を GPU readback に移す構造変更。owned-budget の「窓の経路で毎 frame 待たない」と隣接、ST20 の fit も VB 側へ動く。行数より配線と実窓が重い → 裁定 D。
- **vgpu VG1〜VG5**: 外部証拠に過ぎず、VG3(param を 1 struct)は vism.rs の束縛生成の作り直し ≒40 行 + 試験、VG4 は resolver が `reference/` を許すか未確認。SE6 系は「触れて満足」に効かない。(f)。
- **ffmpeg FF9(download)**: 署名済 .app を壊す・ffprobe が付かない・arm64 は固定 URL、と営業自身が 3 つ白状。SE1/SE2 は配布・法務で後。(f)。
- **wgpu WG2(GPU と ffmpeg の重ね)**: 裁定 L32「待つ方へ倒し、呼び手で待ち方を変えない」に触れる。preview==export の oracle が賭け金。(c)(e)。
- **Apache Arrow AR3/AR4**: 営業自身が「JSON のままでよい」と判定。AR1(版番号 25 行)だけは SE3 で後日。
- **Apple AP7**: crate が本機に無く未検証。来るのが早い。
- **muda MD1〜MD7**: AP1 と同じ穴で、依存 +1(推移 6)と試験の喪失が付く。→ §2。
- **色 CL3/CL7、kurbo KU1〜KU5**: doc crate への依存追加が要る(f32 化・裁定)。減るのは行数で、手触りは変わらない。(d)。

## 2. 同じ穴を複数社が塞ぐ件 — 決定

| 穴 | 決定 | 理由 |
|---|---|---|
| 書体 | **cosmic-text**(CT1 は 19:32 に済)。AP4 は不要、VL2 は別の穴(窓の字形)なので併存 | Core Text の列挙は fontdb の `load_system_fonts` が既に同じ結果。shaper を替えない |
| 実行時 .fs | **naga NG1+NG2**(後日、7 の外)。IS7 は同じ物、VG1 は断る | 診断の行番号は naga にしか無い。ISF の仕様側は IS1 だけ買う |
| 音の切替 | **cpal AU1 + rtrb RT1 は同じ層の 2 部品(競合でない)**。rubato RB1 は AU1 が実窓で通ってから。AP5 は買わない | AVAudio は 2 本目の音 stack、crate 未検証 |
| サムネイル | **どちらも今は買わない**。買う日が来たら R6(裁定 491 が decode を re_video に委譲済) | FF6 は sidecar の使い方の話で絵は変わらない。R6 は行数が嘘(§6) |
| 色 | **TS5 のみ(条件付き)**。CL1/CL2 は hygiene で後日、VL8/TS7 は買わない | 札の画素一致は誰も求めていない |
| menu | **Apple AP1(objc2 直)**。muda は断る | 依存 0 増、同じ試験喪失なら軽い方 |
| 設定の置き場 | **買わない**。UD1(NSUserDefaults 2 key)だけ Settings が残らない件で後日 | directories は 3 行→1 行の話 |
| Stage の形 | **rerun R3/R5/R8 = vello 営業の (a) と同じ物**。文字は (b)(VL2) | vello 自身が枠・取っ手を取り下げた |

## 4. 断る物(会社の勘定で 1 行ずつ)

R1(前半)egui が戻る / R2 裁定 D / R4 Fit ≒90 行の置換で手触り不変 / R6 行数不明 / R7 ✗ 済 / B3・B4・B9・B10・B11・BU1〜BU8 上流 PR 待ちか試験のみ、今日動かない / B5 R6 と同根 / B6 ComputedStyles 30 行は V25 だけ、CSS で済む所 / B7 AK1 が同じ口 / VL6 取り下げ済 / VL7 数字が名前になるだけ / VL8 裁定 495 と R9 の二重、後で束ねて裁定 / CT8 日本語に効かない / CT10 ST17 が決まってから / NG3〜NG5 実行時 .fs の後 / FF1〜FF5 format は X1 の設計が先、M5 の 2 field は後日 / FF6〜FF9 §2 / AU3 メーターは A7 の表示設計が先 / AU4〜AU8 長尺 stream 化は P6 の計測後 / AK1・AK3〜AK8 上流 3 件待ち、AK2 済 / DD 全部 裁定 504 / AP3 recents は済、OS 化は後 / AP4 済 / AP5〜AP8 §1 / LT 全部 L270 / IS2〜IS7 IS1 の後 / VG 全部 / AE1〜AE5 model の欄追加は正しいが**手触りが無い**、E12 を本気で始める日の初手として保留(AE3 印の持ち主だけ E14 と一緒に) / UP1〜UP5 買う側でなく出す側、社長の時間だけ要る / WN1・WN2 hygiene 後日 / WN3 回転 gesture は Stage の回転の意味が先 / WN4〜WN9 小物 / RB1〜RB6 AU1 の後 / RF4 D3 の設計後 / TS1 縁取りの手合成 12 行、精度は同じ / TS3・TS4 CT5 の後 / TS6・TS7 §2 / KU 全部 裁定 E / WF1・WF2 A8/A10 は表示の裁定が先 / WF3〜WF5 無し / RT2〜RT4 定数の話 / AR 全部 / RV1・RV7 GC4 を始める日に、RV2〜RV6 既決 / PT1 2 行で価値はあるが Z13 は閉じない、PT2〜PT5 試験の設計は社長の時間 / WG1・WG3 計測は P 波が実測で詰まってから、WG2 §1、WG4 無し、WG5・WG6 数行、後日 / MD 全部 §2 / CL 全部 §2 / RL1〜RL6 log の整理は「触れて満足」の外 / DR・UD §2。

## 5. 社長裁定に上げる物(5 件)

**A. DOM menubar を捨てて NSMainMenu へ(AP1)**。gui.rs の menu 試験 18 箇所(営業の勘定で 4〜5 本)は NSMenu を harness から押せないので消える。残せるのは MenuId→Intent の表の単体試験だけ。Composition/Export/Settings の popover は NSMenu に載らず DOM に残るので、semantic_menu.rs の半分は残る。買う価値(⌘Q・別窓・Window menu・加速鍵の 1 本化)と、試験の道 1 本の喪失を秤にかける裁定。副社長は買う側。

**B. Lottie 書き出し(LT1、SE5)**。裁定 L270(07-23)は v1 を MP4 に閉じた。営業 2 社(Lottie・AE)が別々に「呼ぶ側 0 件」を空手形として突いてきたが、それは既決の結果。覆すなら sheet に format 1 つ + 30 行だが、tsv の照合試験 `lottie_coverage.rs` が tree に無い(LT・RV7 が自白)ので、出す前に定規を先に立てる必要が在る。副社長は覆さない。

**C. dioxus-dnd の pointer 系(M8・C1・F20)**。裁定 504 が component を不採用にした根拠(二重 borrow)に営業は答えていない。M8「札を Stage/Timeline へ引く」は手触りに直結するので欲しいが、道は dnd でなく Motolii の winit pointer(dock.rs の TabDrag と同じ手)になる ≒60 行。「dnd の component を実走で試す 1 日」を許すか、自前 pointer で行くかの裁定。

**D. 当たり判定を PickingLayerProcessor へ(R2)**。ST10・ST5・ST16 の前提だが、GPU readback を窓の経路に足す(owned-budget の poll 天井は据え置き、非同期なら可)、fit を VB 側へ移す(ST20)、Stage の hit が 1 frame 遅れる、の 3 つを飲むか。R3/R8 を先に買って 1 週間実窓で見てから決めても遅くない。

**E. 依存の追加・版上げの束**: parley を直接依存に(買い 2 の前提、lock 済)、tiny-skia 0.12(買い 7)、kurbo を motolii-doc に(≒160 行減、断る側)、color を doc/render に(断る側)。憲法「足すなら消す」の勘定では parley は semantic_menu.rs の撤去(A)で相殺、tiny-skia は版上げなので無料。あわせて blend_preview.rs 129 行(17:08、R9 ☑「CPU の式で代替」)が裁定 495「式も自作しない」と食い違っている件 — 札の色に限るなら 495 の例外として記録するか、VL8 で借りるか。

## 6. 営業への差し戻し(既出 5 件の他)

- **ffmpeg FF7「decision-index:361 と thumbnail.rs が矛盾」**: 361 は 07-23、裁定 491(08-30)が encode/mux の委譲先に ffmpeg-sidecar を採択済で、Cargo.toml:37 は直接依存。矛盾しているのは code でなく index の 361 行(社長: 行の更新が要る)。
- **rerun R6「≒20 行減」**: `Video::frame_at` は `GpuTexture2D` を返す。DOM の札は PNG data URI なので readback → encode → data URI の道を数えていない。同期 spawn 1 本減の代わりに GPU 経路が 1 本増える。
- **Blitz B1「2 語で X1〜X20 が VoiceOver に出る」**: 出なかった。名前(VO1 の .a11y span)、押下(AK2 ≒15 行)、矩形(B13)、live(B14)が別途要った。第 2 波 AK が自ら暴いた。
- **muda「DOM ≒500 行減」**: app.rs の menubar は 1275〜1610 の ≒335 行、semantic_menu.rs は 308 行だが popover 分は残ると営業自身が言う。実質 ≒350。
- **dioxus-dnd「動く見込み」**: 裁定 504 を読んでいない。実走ゼロで ≒270 行の新規を「賄える」と言うのは営業の言葉でなく発注書。
- **proptest PT1「Z13 は設定値 1 つ」**: Z13 の本体は嵐が新しい状態を見ない事(PT2 の事後条件)で、`max_shrink_iters` では閉じない。
- **wgpu WG2**: 裁定 L32 に触れる提案を「既に払った代金」で出している。待ち方を変えれば経路が 2 本になる。
- **naga「naga 側 0 行」・cpal「0 行で切替追従」・AK「既に払った代金」**: 依存側を 0 行と言うのは全社共通の常套で、Motolii 側は毎回 10〜60 行 + panic を窓の文言にする配線 + 実窓が付く。次回は「Motolii 側の行数・feature の副作用・実窓で見る物」の 3 列を必須にする。
- **winit「H7・K7 は済んでいた可能性」**: 台帳で両方 ☑。営業が台帳を読んでいない。
