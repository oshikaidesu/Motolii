# 「普通の動画ソフト」の合否条件

**この表が「完成」の定義**。既存台帳(`../docs/`)から組み立てたもので、ここで新しく発明していない。
出所欄は `../docs/` 配下。状態欄は **新 workspace(`next/`)** の実装状況。

旧 workspace で実装済みでも、新側に無ければ「未」と書く。旧側の実装は移植元であって成果ではない。

## 必須 — これが無いと動画ソフトと呼べない

| # | 条件(観測可能な形) | 出所 | `next/` |
|---|---|---|---|
| M1 | 起動して project を新規/既存で開ける。無ければスタート画面 | ux-check-first-ten-minutes | 未 |
| M2 | Finder からドロップで素材が入る。開けない物は**理由つきで skip**(黙って消えない) | ux-check P3/P5、first-touch観察 | 未 |
| M3 | 置いた clip が Timeline に立ち、**Stage に絵が出る**。待たされない | ui-inherited-grammar-gap | 未 |
| M4 | clip の尺は min(source, comp残り)。source 終端の先はフリーズせず背景 | first-real-run 欠陥(1) | 未 |
| M5 | drag=移動 / 端drag=trim / release=確定 / Esc=復元。snap は clip端・key・playhead・loop端・0・終端 | normal-timeline-prior-art | 未 |
| M6 | split(Cmd+K)・Delete・複製(Cmd+D)・複数選択(Shift/Cmd/marquee/Cmd+A) | 同上 | 未 |
| M7 | **Copy / Cut / Paste が効く** — 旧 egui は menu に項目があるのに何も起きない(Q0違反の現物) | 同上、egui能力台帳§2 | 未 |
| M8 | Space で再生。**音が鳴り**、playhead が音に同期。scrub で Stage 追従 | ui-inherited-grammar-gap Tier0 | 未 |
| M9 | Export → mp4。**音声mux込み**。報告フレーム数=現物、cancel で残骸なし | concept、first-real-run 欠陥(2) | **部分**(報告=現物・cancel は済。音声mux は未結線) |
| M10 | Document を変える操作は**1回の Undo で戻る**。1 gesture = 1 Undo | ui-quality-bar Q2 | **済**(R0-2 / store の時間旅行) |
| M11 | Cmd+S・未保存●・閉じる確認・**再起動で続きが開く** | ux-check P2/P5、外部診断F-01 | 未 |
| M12 | **触れそうな物は全部機能する**。未実装の chrome を置かない(disabled も不可=撤去) | ui-quality-bar **Q0**(利用者裁定) | 未 |
| M13 | **無反応ゼロ**。拒否は理由がその場で分かる。旧 iced は拒否を `let _ =` で捨てていた | ui-quality-bar Q3、能力台帳§5-2 | 未 |
| M14 | 選択・時刻・幾何の正本は1つ。全面が同じ真実を映す | ui-quality-bar Q5 | **済**(StoreView 投影で構造的に) |
| M15 | **Preview = Export**。同じ評価関数を通り byte 一致 | concept 絶対規律、DECISIONS #15 | **済**。可逆書き出しした**現物を decode し直して** preview と突き合わせる試験まで通した(旧は最後まで未検証) |
| M16 | どの入力でも panic/クラッシュ/喪失なし。render 失敗でも画面を空にしない | ui-quality-bar Q6 | 未 |
| M17 | 空 project は空として表示。空でも place/scrub/keymap が効く | ui-quality-bar Q7 | 未 |
| M18 | Zoom(カーソル下の時刻を保つ)と Fit | prior-art 必須12件 | 未 |
| M19 | keyframe の追加/削除/移動が property 単位で効く | 同上 | **部分**(store/eval/書き出しまで済、UI が無い) |
| M20 | undo/redo/delete がどの面からでも。TextInput 中はテキスト優先。IME を壊さない | ui-quality-bar Q9 | 未 |

## 標準 — 普通は持っている

context menu / カーソル言語(trim端=resize、clip=grab、marquee=crosshair)/ 矢印1フレーム送り・Home/End /
行の rename・label色・lock・mute-solo / fold と**グループ化**(プリコンポの代替)/ M キーでマーカー /
soundtrack の波形帯 / ループ区間再生 / 再生中の playhead 追従 / **区間イージングの切替** /
Time Remap / 親子(型付き Follow/LookAt)/ Effect の追加削除 UI /
**Inspector に Anchor・Scale・Rotation 行** / Stage の bounding box と scale/rotate ハンドル /
Browser から **drag で配置** / Export 設定 UI と割合進捗 /
時間予算 B1〜B7(定常 p99 ≤ 8ms、gesture 中に 16.7ms 超を連続2枚出さない)/
トンマナ不変の機械検収 / 日本語・スペース入りファイル名 / soundtrack の差し替え・gain・clip音声mix /
テキストレイヤー(モデルは凍結前)

## 差別化 — Motolii がそれである理由

| # | 条件 | `next/` |
|---|---|---|
| D1 | **Preview = Export を機械で示す**(byte 一致試験が常設) | **済** |
| D2 | **Undo が壊れない・深さで落ちない**(AE の痛点Aの逆) | **済**(R0)。GC 方針は空席 |
| D3 | ネイティブな区間イージング(Bounce/Elastic/Steps、オーバーシュート可) | 部分(Bezier まで) |
| D4 | プリコンポ地獄が無い(グループ+fold+ベイク) | 未 |
| D5 | **文字列式が要らない**(wiggle/loopOut/ピックウィップを型付きの口で全数カバー) | 未・カバレッジ表に穴 |
| D6 | **拡張の口が trait 1本**。first/third-party が同じ口 | 意図的に未着手(DECISIONS #13) |
| D7 | 3〜5分(5,400〜9,000フレーム)を実用スループットで書き出せる | 未計測 |
| D8 | ビート検出・拍グリッド吸着(MV では編集の起点) | 空席 |
| D9 | 起動〜最初の結果が数秒 | 数値バーが空席 |
| D10 | first-party パーティクル(音楽同期) | 未 |

## 要らないもの — 欠落ではなく設計上の除外

**これらを「足りない」と数えない。**

- **trim family 一式**(ripple / roll / slip / slide / insert / overwrite / lift / extract / sync lock)
  — 自由配置土台の裁定(2026-08-19)。gapless packing 前提なので既存 gesture と機構的に衝突する
- 「以降を押し出す」修飾キー drag(便利機能として先送り)
- **プリコンポ / Nest / Compound clip** — グループ化+ベイクへ置換済み
- **ノードグラフ UI** — ユーザーに見せない
- **JS 文字列式 / AE のグラフエディタ** — 型付きlink + 区間イージング + ParamDriver へ写像
- IK / キャラリグ / 状態を積む本物のシミュレーション / 120Hz 最適化
- 動的配布 marketplace / 第三者 SDK / 独自 plugin UI / VST 互換
- **第二 runtime・第二評価経路**(背骨2)
- rerun の viewer 層 / egui shell / `ui/motolii-rn`

空席のまま(禁止ではない): 3点編集、A/V link-unlink、マルチカム、J/K/L shuttle。

## 順序

1. ~~media を移植して compositor に**実素材**を流す~~ — **済**(2026-08-20)
2. ~~export で **鎖を閉じる**~~ — **済**(2026-08-20。音声 mux だけ残)。M15 をここで閉じた
3. iced shell の骨。**背骨1を型で作る**(`StoreView` と Intent の送り口しか渡らない)
4. **核の一周**を1本ずつ: ドロップ → clip が立つ → Stage に絵 → Space で音同期再生 → Export
5. 編集の必須12件(M5〜M7。**Copy-Paste の死に席を最初に潰す**)
6. 保存と復帰(M11)
7. **Q0 と Q3 を柵にする** — 「触れそうな物は全部機能する」「拒否は必ず理由が出る」を機械検査に。
   旧実装は同じ穴を2回開けているので、後から一括では潰せない
8. keyframe + 区間イージング + Inspector の Transform 全行(M19 / 標準)
9. 品質バー B1〜B7 を計測に乗せる
10. 実機で `ux-check-first-ten-minutes` の台本を通す —
    **旧実装の「良い」判定は全て機械検証止まりで、実機の手触りは人間未検証**

7 を 8 より前に置くのは、Q0/Q3 が「機能を足すたびに再発する型の穴」だから。
