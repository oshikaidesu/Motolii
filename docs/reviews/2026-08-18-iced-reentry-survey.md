# iced 再評価の実態調査 — 「GUIとCLIの段差ゼロ」軸

日付: 2026-08-18
状態: **観察**(乗り換え判断はしない。トリガーは[運転席決定の追記](2026-08-18-cli-gui-driver-seat.md)が正本)

評価軸(利用者定義): **GUIとCLIの段差を構造的に無にできるか** — GUIの全操作が型付き
Message/intent 列で、renderer 無しに同じ列をCLI/テストから流せるか。
資産量(sunk cost)は論拠にしない。

## icedが軸上で本物である点(出典は調査ログ)

- **iced 0.14(2025-12)は replay を公式不変量にした**: `iced_test` の Simulator が
  「操作→Message列→update→assert」を GPU/窓なしで回す公式 headless テスト基盤
  (PR #2698)。さらに Time Travel Debugging(PR #2910)が「初期状態+Message列=
  任意時点の状態」を出荷機能として実装 — replay 可能性がドキュメントの主張でなく
  機能が依存する不変量になっている。**これは egui に構造的に無い性質**で、
  評価軸そのものと同型。

## いま乗り換えない実測理由(軸上の壁)

1. **Rerun=合成基盤裁定との衝突が具体化した**: iced 0.14 は wgpu 27(pop-os fork
   でも 28)、Motolii と Rerun fork は **wgpu 29**。shader widget での外部 texture
   埋め込みは「同一 device 共有」が前提で、版が揃わない限り成立しない。
   egui を iced 内に埋める先例は**ゼロ**(失敗報告すら無い=全リスク自前)。
   **→ 訂正(同日・利用者指摘)**: released 版しか見ていなかった。**iced master
   (0.15.0-dev)は wgpu = "29"**(workspace.dependencies を直接確認)で、版の壁は
   master 基準では消えている。同一 device 埋め込み(shader widget に Rerun offscreen
   合成を bind + 入力→Message→set_camera)の実測 probe を同日発注
   (`spikes/iced-rerun-embed-probe/`、隔離 workspace)。残る壁は「master は未 release
   の移動標的」という安定度の問題に変わる。
2. **検証基盤の空白**: AccessKit は upstream 未統合(issue #552 が2020年から
   open)。iced_test は出た直後で、COSMIC 実アプリ群が使っている証拠も未観測。
   日本語 IME は 0.14 が初実装(fcitx 追従等の未解決あり)。
   「主張と実践の間に空白がある」(調査原文)。
3. **先人ゼロ**: iced 製の NLE/タイムライン編集の先例は見つからず。直近の
   Rust 製ネイティブ編集ソフト(Gausian)は egui を選んでいる。
   1.0 前で minor ごとに全面破壊的変更、0.14.0 以後8ヶ月パッチ無し。

## 中間の道と現状マップ(繋がり3分類)

Elm 性の不変量(UI層=入力→intent の翻訳だけ、状態変化は replay 可能)は
**toolkit を替えずに採れる**。2026-08-18 時点の shell の分類:

| 面 | 分類 |
|---|---|
| Document 編集(D2 Command 全系) | 繋がっている(intent replay 可) |
| New/Open/Export/未保存 dialog | 繋がっている(ScriptedPrompts で台本 replay 可) |
| Stage カメラ | 繋がっている(`set_camera` API) |
| スタート画面・status 帯・ドロップ | 橋渡しのみ(kittest/AccessKit 経由) |
| Timeline 密描画面(clip/key 操作) | 橋渡しのみ(座標契約経由) |
| **繋がっていない** | **現時点で確認ゼロ** |

## 同一 device 埋め込み probe の実測(同日・2巡目)

`spikes/iced-rerun-embed-probe/`(隔離 workspace、iced master
`3de45144` pin + Rerun fork `483b8559`)。証拠は
`evidence/iced-rerun-embed-probe/`(results.md と PNG 群)。

- **(a) 同一 device: 条件つき成立** — wgpu 29.0.4 / egui 0.35 が iced・re_renderer・
  probe で各1本に解決し、device 1つで動いた。ただし **iced が `max_bind_groups: 2`
  をべた書き**しており上書きの口が無く、re_renderer の `LineRenderer`(3 groups)が
  作れない — 矩形・grid は出るが**線分・outline・gizmo が犠牲**。
- **(b) 絵の到達: 成立** — E0 の4象限 oracle 通過。窓の fnv1a が Rerun の
  offscreen texture の fnv1a と一致(窓に出ているのは frame そのもの)。
- **(c) 入力→Message: 成立** — click が型付き `Message::StageClicked` として
  `update` に届き、`set_camera` で絵が変わった(winit→widget tree の最終区間のみ
  自動走行外。interactive 実行の口は残してある)。
- **新しい壁2つ**: ① iced が `web-sys = "=0.3.85"` を完全一致で釘打ちしており
  re_renderer と **cargo グラフ上で同居できない**(vendor+1行 patch で回避 =
  実採用なら iced fork を持つことになる)。② 上記 bind groups 上限。どちらも
  上流に直せる浅い壁だが、**採用時は fork 2本体制**(rerun + iced)になる。
- 付帯: `Pipeline` の Send+Sync 要求で SpatialStage 束は thread_local 置き。
  依存 836 package・clean build 5分34秒(M4)・増分2〜4秒。0.14→0.15-dev の
  API 差では詰まらなかった(詰まりは wgpu 29 側)。

## egui との共存(入力ブリッジ)の実測(同日・3巡目)

「Rerun の egui ビューと iced は共存できないのか」への実測回答。ブリッジ
(`probe/src/bridge.rs`、iced event→`egui::RawInput` 翻訳、442行の半分は説明とテスト)
を書き、iced 窓の中の SpatialStage を **Rerun 自身の `EyeController` で** orbit させた
(`set_camera` 直呼びなし)。証拠は `evidence/iced-rerun-embed-probe/interactive-bridge-*`。

- **入力到達: 成立** — 合成 drag 12手で `last_eye()` が動き、wheel 6ノッチで軌道半径
  0.457 移動。翻訳の薄さは「原点補正と pixels_per_point の2点だけ」
- **ドラッグ中の絵: iced device では凍る** — orbit 目印が `LineRenderer`(3 bind
  groups)で描かれ、`max_bind_groups=2` により**コマンドバッファごと**蹴られる。
  同じ台本を re_renderer 記述の device で回した対照群は18フレーム全追随・検証エラー0
  (eye 数列は両者で完全一致=切れているのは絵だけ)。**壁は iced の定数1つ**
- **姿勢の保持: iced 無関係の既知 seam** — `EyeController::save_to_blueprint` の
  `AppendToStore` を SpatialStage が捨てるため補間で戻る。これは egui ホストの製品でも
  同じ(orbit 持続は S2 レーンの残件。[fork seam ledger](2026-08-18-rerun-fork-seam-ledger.md))
- **cursor: 写る**(egui 26種中 `VerticalText` 以外対応、構造的に1フレーム遅れ)
- **構造的な穴2つ**: iced の `shader::Program` は (1) egui の repaint_delay に答える
  口が無い(毎フレーム描画で回避=省電力性を失う)、(2) `request_input_method` を
  呼べず **島の中へ IME を届ける経路が無い**。島を Stage(view操作専用・text入力
  なし)に限る設計なら踏まない

**共存の結論**: 「できない」ではない。island 方式(アプリ=iced Message列、
Stage 1面=egui の島)は、iced fork の定数修正1つと既知の S2 残件を払えば成立する。
島に text 入力を置かない限り IME の穴も踏まない。

**ホスト選択の再構成(同日・利用者指摘「Rerunも埋め込み想定では」)**:
2026-08-16 の「ホストが egui なのは構造上の帰結」論は、本日の3 probe で**実測的に
弱まった**。Rerun は offscreen texture+入力ブリッジの後ろに置けば任意の wgpu
ホストへ埋め込める島であり、egui runtime は島の内側の実装詳細に縮む。
ホスト選択(egui/iced)は Rerun に拘束されず、DX・検証基盤・エコシステムの判断に
戻った。また UiIntent 背骨(ログと構造の強制)は iced の Message と同型のため、
将来ホストを移す場合も shell の意味層(intent/gateway/replay oracle)は持ち越せる —
egui 固有なのは view 層と kittest だけ。

## 判断

- 現時点では乗り換えの実測根拠なし(「繋がっていない」= 0 件)。
- トリガー(繋がっていない箇所の散見)は運転席決定に記録済み。以後の UX 欠陥は
  3分類つきで記録し、件数で判断する。
- probe により**構造的不成立は消えた**: iced 採択は「できない」ではなく
  「fork 2本体制+master 追随+検証基盤(AccessKit/iced_test 実績)の空白を払うか」
  という**コストの選択**になった。トリガーが積み上がった時の乗り換え先として
  実測済みの道が1本ある、が現在地。
- iced 側の再観測ポイント: 1.0 到達(wgpu 29 での release)/ AccessKit 統合 /
  `web-sys` 釘打ちと `max_bind_groups` の上流修正 / iced_test の実運用事例。
  どれかが動いたらこの表を更新する。
- **DX(if 沼)は慢性コスト枠として記帳**(2026-08-18 利用者所感: 「if まみれで
  開発体験が悪い」)。immediate mode では局所状態をその場で触るのが最も楽で、
  F-03 型のバグはその症状。緩和は「widget は型付き action を返すだけ」+
  gateway+フェンス(ログと構造の強制)。**フェンス下でも F-03 型の再発が続く**
  なら「柵の維持費が高すぎる」証拠として乗り換え判断の材料に数える。なお
  timeline 等の密な面は iced でも custom widget(painter 的コード)になるため、
  DX 差が最大なのは chrome 側・最小なのは密な面である。
  **→ 後段は同日の仮タイムライン spike で半分反証された**(下記)。

## 仮タイムライン spike の DX 実測(同日・4巡目。利用者依頼)

`spikes/iced-rerun-embed-probe/timeline/`(iced canvas、製品と同じ行高20px・
trim 8px・AE 同型の面割り当て。20テスト+500/5000 clips 計測。証拠は
`evidence/iced-timeline-probe/`)。同じ4ジェスチャ(移動/トリム/スクラブ/zoom)の比較:

- **行数**: egui 約1,415行(非テストの26%、**1,724行の単一 `show()` 関数内**)
  vs iced **342行**(model/update/view 分離)
- **if 沼度(数えた)**: egui は `show()` 内の永続状態書き換え **97文**(うち
  Document/undo 到達26)、draw パス内の input 読み14、最悪例は**行を描くループの
  中から undo 可能な Document コマンドを毎フレーム積む**箇所。iced は draw パス
  副作用 **0 — 規律でなく型**(`Program::draw(&self)` が書く道を持たない)。
  状態を書くのは `update()` 系の16文が全て
- **描画は速い**: 500 clips draw 0.27–0.37ms、5000 clips でも 1.57ms(vsync 張付き)
- **iced が明確に不利な点は1つ**: `WheelScrolled` が modifiers を運ばない
  (Cmd+ホイール= zoom に `ModifiersChanged` の別購読が要る)。他: canvas に
  フォーカス概念が無い(複数 canvas のキー調停は自前)
- **訂正**: 「密な面は DX 差最小」は**描画については真、対話については偽**。
  対話の状態機械は iced では型で守られ、egui では draw パスに滲む — 利用者の
  if 沼批判は密な面でこそ最大だった。egui 側の緩和(action 化+フェンス)で
  同じ規律は作れるが、型が禁じるのと柵が禁じるのの維持費差は残る
