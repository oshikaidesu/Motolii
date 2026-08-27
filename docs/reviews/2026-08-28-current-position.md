# 現在地(2026-08-28) — 引き継ぎ台帳

**この文書は現在地であって歴史ではない。** 進んだら書き換える。次のセッションはここから読む。

## 第一目標

**GUI だけで、普通の編集の輪が回ること**(2026-08-27 利用者裁定)。

> 素材の配置 → エフェクトの適用 → プレビューカメラの分離 → xyz などパラメータの駆動 →
> 区間イージング GUI。**一通り普通に使えるようになるまで。**

書き出しも配布も、その後。まず**作れる状態**になること。目的は
[裁定273](../decision-index.md)(自分の MV を作る)であり、優先順位の根は
**「自分が1本作るのに要る物が先」**。

## 夜のあいだに何が変わったか(2026-08-28 未明)

**タイムラインのクリップが本当に掴めるようになった。本体 main に入っており、いま起動すれば触れる。**
端を掴んで伸ばす・丸ごとずらす・行をクリックして選ぶ。錠のかかった行は**掴む前に**カーソルが
「不可」に変わるので、伸ばしてから黙って戻る嘘が出ない。1ジェスチャ = 1 undo。

| 第一目標 | 判定 | 状態 |
|---|---|---|
| (土台)トリム・移動・選択 | **PASS** | **本体 main に統合済み・今すぐ触れる** |
| エフェクトの適用 | **PASS** | **統合待ち**。実在する GLOW を積む/切る/振るが store と絵に届くのをヘッドレスで実測済み |
| 素材の配置 | **PARTIAL** | 中身は出来ている。`IMPORT_WIRED = false` と main.rs の受け口10行が未着地で**窓から到達できない** |
| xyz の駆動 | **STOPPED** | Inspector 側の口は出来た。store へ届く半分が main.rs 側 |
| 区間イージング | **STOPPED** | GUI の型(◆ と LINEAR/EASE/HOLD)は出来ている |
| カメラ分離 | **STOPPED** | **実装ゼロ**。engine に camera を差し込む口が無く front だけでは原理的に組めない |

**FX STACK の `TURBULENT DISPLACE` は engine が一つも知らない名前だった**(6欄がドラッグできて
どこにも届いていなかった)。撤去して実在する GLOW に置き換えてある。

**カメラの STOPPED は失敗ではない。** 担当は canon(`stage-semantics.html` v5)を読んで
「Stage の絵を画面上で拡大縮小するビューア倍率」という近道が **canon 違反**だと突き止め、
実装せずに返した。write-set が front 3ファイルに切られている限り必ず手が伸びる場所なので、
先に潰したのは大きい。

## ★ 次の一手 — engine に front 向けの口を立てる

**3レーンが独立に同じ場所で止まった。** ここを1本開ければ、止まっている物のうち2つが動く。

| 誰が | 何が要るか |
|---|---|
| トリム | `Engine::media_frames(&self, path) -> Option<i64>` — Media の尺の壁(裁定272) |
| 素材の配置 | 同じ probe キャッシュ |
| カメラ | `Engine::render_frame_into_with_view_camera(view, t, target, &ObservationCamera, include_background)` — **約15行、既存 `render_frame_into` の複製**。camera は `layers_from_resolved` と `compositor.render_into` の**両方**へ通す(片方だけだと絵が壊れる) |
| エフェクト | 「どの plugin_id が描けるか」と param 名・既定値。いまは `translate.rs` に private で front は**写し**を持っている |

**engine の write-set を握る単独レーンを1本立てるのが正解。** front レーンの再発注はその後。

## 統合の手順(順序が決まっている)

4つの worktree は基底 `151d9da1` で、土台の main.rs 変更を持っていない。全部が
`App::handle_actions` を触るので**当てる順序を固定する**:

```
本体 main(土台=済) → effect → material → keyframe
```

- **実衝突が1件**: `inspector_surface.rs` で **effect が丸ごと削除する行**(`fx_stack` /
  `advanced` / `FxPower`)に **keyframe が5行書き足している**。
  **解決は「effect の削除が勝つ」で決まり** — `turbulent_displace` は engine に存在しない
  名前なので、そこへ prop を付ける意味が無い。`keyed.prop: "fill"` も写像先が無いので落とす
- `ScrubValue` の Rust 実装と TRANSFORM/APPEARANCE の行宣言は**両レーンとも触っていない = 安全**
- **`crate::fx_stack::script_mod(vm)` は inspector の直後**でなければ `ScrubValue` を引けない
  (機構の制約であって好みではない)。FX ブロックは timeline の早期 return より前
- material だけが `motolii-media` と `rfd = "0.15"` を足す(`app/Cargo.lock` も動く)

## 裁定待ち(あなたの判断が要る)

1. **engine の口**(上記★)— これが最重要
2. **Group の timing は独立した値か、子からの導出か。** 裁定272 は「中身に従う」と言うが
   その規則の実装がモデルのどこにも無い。front で書くと同じ規則の家が2つになる
3. **音声の取り込み。** `motolii_media::probe` は先頭 video stream を要求するので
   **audio-only は必ず失敗する**。一方 `motolii-audio` は `LayerSource::Media` を soundtrack の
   候補として読んでいる。「front から音を持ち込む時、尺を何から取るか」「front は
   `motolii-audio` を引くか」。**hero が MV である以上、必ず要る**
4. **区間イージングの対象は誰か。** 二代目 UI は**選択されたキー**、今回のレーンは
   **playhead の直前のキー**。裁定274 で慣習(**選択されたキー** — AE の F9・Premiere・Resolve)
   を採ったが、これは Lottie が黙っている所なので利用者は覆せる。

   **※ 中身は既に決まっている**(2026-07-10 決定、`concept.md:188-192`)。
   エージェントが「LINEAR / EASE / HOLD」で止まったのは**この決定に到達していなかった**から:

   - **データモデルは既に一致** — `motolii-eval` の `Interp::Bezier{x1,y1,x2,y2}` は
     「区間の正規化位置 u∈[0,1) に対する連続イージング」= CSS `cubic-bezier()` = Flow = AM と
     **同一表現**。**fps・解像度に非依存**。UI はこの4値を編集する**薄いポップアップで足り、
     スキーマ変更は不要**
   - **AM 式の高度イージング型を採用** — 動きの"性格"(バウンス・バネ・段階移動)は式や
     ParamDriver ではなく**区間の補間タイプ**として持つ。Cubic Bezier に加え
     **Bounce / Elastic / Steps / Elastic Steps**、オーバーシュートはトグル。
     実装は `Interp` への variant 追加(`Elastic{amplitude,period}` /
     `Bounce{bounces,decay}` / `Steps{count,..}`)で、どれも「u∈[0,1]→値」の純関数
     = **評価器の約束を崩さない追加的スキーマ変更**。
     **AE では `valueAtTime` の物理シミュ式が必須だった領域を GUI の選択肢へ畳み込む**のが狙い
   - **混同禁止**: Graph View は時間方向の値グラフエディタ、Interval Easing Editor は
     1区間の正規化 time remap、空間モーションパスは位置の 2D 経路。
     **三者を同じ curve state・座標・操作面へ統合しない**
5. **`rotation.x`/`rotation.y` と `scale.x/y/z` に store の property が無い。**
   Inspector は3欄ずつ見せているが配線先が存在しない。欄を消すのは簡単だが、それは
   **「3D 回転を諦める」という意味の決定**になる
6. **User View の初期倍率**(等倍 / fit / 現行62%)

## いま何が在って、何が無いか

**背骨は端から端まで通っている** — 作り物のドキュメント → `motolii-engine` →
`motolii-compositor`(re_renderer)→ 共有 Surface → Stage。絵は出る。

**入口と出口が無い** — `app/` の実行可能バイナリは front 1つだけ。`motolii-export` を
呼ぶ者はテスト以外に居ない。front が読むのは `motolii_fixture::build()` の作り物で、
**実素材が1つも入っていない**。

| 第一目標の5つ | store(意味) | front(操作) |
|---|---|---|
| 素材の配置 | `AddLayer`/`SetSource`/`SetTiming`/`SetOrder` | **ファイルを開く口が無い**。Browser は8枚の作り物 |
| エフェクトの適用 | `EffectId`/`EffectInstance`/`SetEffects` | FX STACK は**見た目だけ** |
| カメラ分離 | `ResolvedCamera`(compositor が使う) | **タブのラベルだけ**(`"User View"`) |
| xyz の駆動 | プロパティ track・`SetTrack`・`value_at` | `ScrubValue` は入ったが store へ届かない |
| 区間イージング | 補間は track の中 | **`easing` も `keyframe` も1行も無い** |

### 5つが共有している、たった1本の欠落

**front から store へ書く経路(`Intent`)。**

`Intent` は既に27種ある(`app/core/motolii-store/src/document.rs:32`)。
第一目標に必要な物はほぼ全部揃っている。**意味の側は待っている** — front が呼んでいないだけ。

だから最初の仕事は**この経路を1本ちゃんと通すこと**。1本通れば残りは同じ型の反復で、
今日のように5レーンが別々に「store へ届かない」と報告する事態も消える。

**front 側の先例**: `main.rs` の `toggle_lane_flag_from_timeline` と
`restack_from_timeline` は実データへ書いている。新しい経路を発明せず、この形に倣う。

## 発注の順序

1. **front→store の経路を1本**(`SetTiming` = トリム)— 型を作る。**ここだけ設計が要る**
2. **素材の配置** — ファイルを開く・probe・`AddLayer`+`SetSource`。**ここから実物が入る**
3. **xyz の駆動**(`SetTrack`)→ **区間イージング** — 同じ track の上なので続き
4. **カメラ分離** — 空間で作業するなら要る。store を触らないので独立
5. **エフェクトの適用**(`SetEffects`)— ここから Vism が効き始める。LLM が並列で書ける場所

1 が終われば 2〜5 は**並列に出せる**(write-set が互いに素)。

## 履歴の注意 — `c766bd7b` の題は中身を説明していない

**21件の UX 修正(11ファイル・2,149行)は `c766bd7b` に入っている。** その commit の題は
`docs: rule 273 -- the purpose is one video, and Vism is a road to it` で、**実装のことを
一言も言っていない**。

原因: 統合担当が作業ツリーへ束ねている最中に、監督(supervisor)が `git add -A` で docs を
commit し、担当の作業を巻き込んだ。押し済みなので履歴は書き換えず、ここに記録して直す。

- **コードは正しい**。ビルド green、実窓検収も全項目 PASS(下記)
- **「トリムが効くようになったのはいつか」を commit の題から探すと見つからない**。
  `git log -S` か、この行を頼りにすること
- 監督側の再発防止: **リポ根で `git add -A` を使わない。触ったパスを明示する**

## 直前の状態(2026-08-28 未明 — 統合完了)

- **21件の UX 欠陥**: 5レーン実装 → 本体へ統合済み → **実窓で検収済み**。
  `cargo build` 0 error、テスト green、`/log` に式評価エラー無し
- **実窓で PASS を確認した物**: A1 の帯が消えている / A5 がクリックで色を変えない /
  B1 行高が 26px 固定(ペインを縮めても文字が縮まない)/ C1 検索で `8 items → 3 items` /
  **F2 検索欄にフォーカス中の Space で再生が始まらない** / D1 の Position X が
  ドラッグで増えて Esc で戻る(`0.179 → 0.224 → 0.179`)/ D2 の折りたたみと ON/OFF /
  F1 パネル開閉 / F3 ズーム % の一本化 / E4 ロックが離しても青のまま

- **「消える虚報」への裁定(a)を適用済み** — A1(トリム掴み代)・A5(選択ハイライト)・
  E5(色見本の受け口)は**触れるように見せない**状態にした。実装は温存されており、
  `TRIM_HANDLE_WIRED` / `SELECTION_WIRED` を `true` にするだけで復活する。
  **上記「発注の順序」の 1(front→store の経路)が通った時点で本物として戻る**

- **未確認のまま残っている物**: A3 タイムラインの縦スクロールの**符号**(実機裁定待ち)。
  Chrome gallery は `chrome_body: height: Fit` が固定高ペインを溢れるため
  **COLOR 以降(SEARCH/NAV/MENU/TRANSPORT/FEEDBACK)が通常状態で一度もレイアウトされない**
  — E2/E5 は節を一時的に動かして確認し、元へ戻してある(構造的ギャップ、未修正)。
  `chrome/gallery.rs:76` の `ChromeStepper{value.text: "24"}` は「24」と「0」が並んで見える
  (1行で直る、未修正)

## 素材の配置 — 「どこを参照するか」に当たり前の解がある

**道具が推測しない。利用者が足す。** 分からないのが当然であって、既定を発明する所ではない
(2026-08-28 利用者指摘)。

**先例は全部同じ形**: Ableton の `Places`(`Add Folder...` で足す)、Premiere の Media Browser、
Resolve の Media Storage、Blender のブックマーク。どれも **(a) 利用者が登録した場所** と
**(b) このプロジェクトに実際に入っている物** の2本立てで、**どこを見るかをソフトが決めない**。

そして **Motolii の Browser は既にこの構造を持っている**(`browser_surface.rs:299-316`):

```
Collections   Favorite / Brand              ← タグ束(予約地)
Library       All media / Video / Images / Audio / Project / Recent
Places        Starter Media / Project assets / Motion assets / Add Folder...
```

`Add Folder...` と `Project assets` は `RailRowReserved` — **意味だけ先に置いて機能は保留**、
と明示されている。**構造は正しく、配線されていないだけ。** 新しい概念を作らず、この行に
機能を付ける。

一発で開く経路(メニュー/ボタン → native file dialog)は `Places` とは別に要る。
両方あるのが普通(Ableton も「ドラッグして足す」と「ブラウズ」の両方を持つ)。

## 決まっている事(読む順)

1. [CANON](../CANON.md) — 憲法5条・何のために作るか・hero・自作の範囲
2. [裁定271-273](../decision-index.md) — 意図論 / Lottie が審判 / 目的は自分の MV
3. [注意の失敗と世界の分断](2026-08-27-attention-failures-and-the-partition.md) — 裁定270
4. [レイヤー UI 欠陥台帳](2026-08-27-layer-ui-parity-defects.md) — 21件の一覧
5. [AGENTS.md](../../AGENTS.md) — 運転(製品は `app/` に居る・ホットリロード運転)

## 効いている柵

- **AE parity の裁定は利用者に聞かず Lottie に聞く**(裁定272)。審判は
  `app/reference/lottie.schema.json` と `lottie-coverage.tsv`。利用者が要るのは
  Lottie が黙っている所と、意図的に AE から離れる所だけ
- **UX の分岐は仕様でなく意図論**(裁定271)。「利用者は何を求めてこの操作をするのか」。
  **操作は意図が名指した物だけを変える**
- **繋げるだけを自作に膨らませない** — 新しい型・trait を定義する瞬間に
  「これを既にやっている物は何か」を答える。答えられないなら探していない。
  **詰まったら `re_renderer` の中を読む** — 手元に在る:
  `~/.cargo/git/checkouts/rerun-bdb1f1ac6277bf7e/7cca401/crates/viewer/re_renderer`
- **Inspector に renderer のパラメータを出さない**(2026-08-28 利用者指示)。
  モデルが Lottie なので、利用者が触るのは**意味**(位置・不透明度・trim path・エフェクト)
  であって**機構**(MSAA・alpha channel usage・render config)ではない。
  判定は機械的: **Lottie で表現できない物は Inspector に出さない** — 出しても保存で消えるので
  利用者から見れば嘘になる。迷ったら `app/reference/lottie.schema.json` を引く
- **テストは9ファイル + Lottie 計器3本だけ**。見える物は窓で見る(裁定270)。
  足りなければホットリロードで足す
