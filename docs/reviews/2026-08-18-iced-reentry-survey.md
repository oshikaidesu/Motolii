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
