# 台帳の並べ替え軸に「粒の重み」を足す — depends/weight 列と機構グラフ

日付: 2026-08-22 / 状態: **分析+機械柵**(`next/reference/normal-map.tsv` へ列追加・`next/check.sh` へ検査追加) / 起点: 利用者の診断「粒として重みをグラフにしていなかったのが問題」— `freq`(4製品での出現頻度=**どれだけ標準的か**)だけで並べ替えていたため、`理由` 列に書かれていた「機構が先」情報が集計されず、**軽くて人気の行ばかりが選ばれ、重い実態(機構)が構造的に後回し**になっていた

## 0. 結論(先出し)

- `normal-map.tsv` に **`depends`**(その行がまだ待っている機構名・空=待っていない)と **`weight`**(S/M/L)の2列を追記した。既存15列・行数(1,551)・行順は無改変(§7 diff実測)
- 機構が実在すると**文字証拠(理由列・git log・review docs)+コード実測(grep)の両方で確認できた行は 107/1,551(6.9%)**。残りは「機構待ちではなく単に未着工」であり、推測で機構名を割り当てていない(§6 EVIDENCE_GAP)
- 開く行数 ÷ weight の上位は **`tag-attr+color-editor`(23行、M)が1位** — ラベル色1機能のために store 1 field + 未着手の色選択UIが23行分の「クリップ整理」機能全部を止めている。2位 `pane-layout`(15行、M・裁定128 dock/workspace/マルチウィンドウ、S1/S2は部分着地済み)。3位 `frame-cache`(11行、M)。**`observer-camera`(3Dカメラ/named view、28行)は行数最大だが weight=L のため4位に後退** — これが今回の診断の実例: 行数だけで並べると observer-camera が最優先に見えるが、重みで割ると tag-attr+color-editor の方が投資効率が高い
- 副産物として、**すでに機構が着地したのに verdict が更新されていない行を4件発見**(1157/1158/1206/1231 = AUTOSAVE、1028 = Progress)。既存列は不変の縛りなので本台帳では直していない。supervisor 引き渡し事項として§5末に明記
- 他ドメイン(Stage)からの中継材料を1件受領し、独立検証の上で一部採用・一部不採用とした(§8)

## 1. 機構語彙の定義(`depends` に書ける値)

各値は「理由列の実テキスト」または「git log」または「コード grep 実測」のいずれかで確認できたものだけを採用した(想像で機構名を作っていない)。

| 機構 id | 定義 | 根拠 |
|---|---|---|
| `observer-camera` | 観測カメラ(裁定157: Shell直下のview専用フィールド)による3Dビュー切替・named view(Front/Top/Custom View等)・ドリー/オービット/パンのカメラ操作ツール。裁定157/114/124で方針決定済みだが未着地 | 理由列(30行同一文言)+`next/ui/motolii-stage-pane/src/lib.rs` の `wraps:` 宣言(「Stage の観測カメラ」としか書かれておらず、named view切替の実装なし) |
| `layer-source-camera` | `motolii_store::LayerSource` enum に `Camera` variant が無い(現行 `Solid`/`Media`/`Null`/`Shape`/`Text`/`Group` のみ) | `next/core/motolii-store/src/lib.rs:164-199` を実機 grep して6 variant を確認・Camera無し |
| `motion-tracker` | 映像内の動き/特徴点を追跡してカメラ・レイヤーを追従させる解析機構(Track Camera 等) | 理由列(id146は observer-camera と同一理由文言だが、canonical が「Track Camera」= 単なる視点切替ではなく解析を要する。他ドメイン中継material §8 の指摘で分離) |
| `tag-attr+color-editor` | (a) `LayerAttrs` にラベル色 field が無い(store拡張) (b) 手動でラベル色を選ぶ UI(スウォッチ/ピッカー)が未着手 — 現状は `label_color_for_new_layer`(id%12)の自動割当のみで変更 Message が無い | (a)理由列「store未実装(LayerAttrsにlabel fieldなし)」19行実文言 (b)[menubar-foundation-survey](2026-08-22-menubar-foundation-survey.md) §「Label Color 手動指定の設計未着手」 |
| `pane-layout` | dock配置・ワークスペースプリセットの保存/復元・マルチウィンドウ(裁定128: iced pane_grid+daemon)。S1(daemon骨格)/S2(Settings窓移住)は着地済み(裁定182/188)、他パネルへの展開・プリセット保存は未着地 | 理由列(裁定128直接参照・計15行)+ git log `b1ec565d`/`57aefb85`(S1/S2 landed) |
| `frame-cache` | 素材デコードの永続フレームキャッシュ(現行はffmpeg起動都度+64枚LRUのみで永続なし) | 理由列(RB調査2026-08-22、11行同一文言)+[residual-bottleneck-survey](2026-08-22-residual-bottleneck-survey.md) §1-2 |
| `vism-expression` | expression式評価という新しい意味論(裁定175: vism抽象化候補、コアには入れない拡張枠) | 理由列(6行同一文言、裁定175直接参照) |
| `analysis-provider` | 素材の解析ベース処理を担う機構(Bake/Analysis provider経路 — 被写体解析・話者音声分離・音声認識等) | 理由列(6行、「Bake/Analysis provider経路」直接記載)+ `next/core/motolii-store/src/document.rs` に `BakedChildTransform`(階層用、別物)しかなく解析providerは未実装と grep確認 |
| `line-break` | 段落の行分割器(両端揃え4種・段落コンポーザ切替に必要)。文字単位の shaping(cosmic-text)はあるが行分割アルゴリズムが無い | 理由列(4行、「行分割器が要るため後回し」) |
| `keymap-layer` | アクション名→キー割当を1箇所に持つ keymap 表(裁定146)。現状は `lib.rs` コメント「keymap層が無い今だけ直結」のハードコード | 理由列(2行、裁定146直接参照+lib.rsコメント引用) |
| `audio-device` | OSオーディオデバイス列挙・出力チャンネル割当(まだ音声デバイスAPI統合なし) | 理由列(2行) |
| `quality-mode` | engineに draft/wireframe のレンダリング品質モード(パスの出し分け)が無い | §8 中継material、`next/engine/motolii-engine`・`motolii-compositor` を grep して `wireframe`/`QualityMode` 相当ゼロ件を実機確認 |
| `resample-method` | engine/compositorの画像リサンプル方式が選べない(`motolii-compositor/src/lib.rs:1950-1951` で `TextureFilterMag::Nearest`/`TextureFilterMin::Nearest` に固定) | §8 中継material、該当コード行を実機確認 |

**依存の書き方**: 1行が複数機構を待つ場合は `+` で連結する(例 `tag-attr+color-editor`、`observer-camera+motion-tracker`)。新しい機構名を追加する時はこの表に定義を足すこと。

## 2. 機構ごとに「開く行数」

| 機構 | 行数 | weight | verdict内訳 |
|---|--:|:--:|---|
| `observer-camera` | 28 | L | 採用予定 28 |
| `tag-attr+color-editor` | 23 | M | 採用予定 23 |
| `pane-layout` | 15 | M | 採用予定 15 |
| `frame-cache` | 11 | M | 保留 11 |
| `vism-expression` | 6 | L | 拡張 6 |
| `analysis-provider` | 6 | L | 採用予定 6 |
| `line-break` | 4 | M | 採用予定 4 |
| `quality-mode` | 4 | M | 採用予定 4 |
| `resample-method` | 4 | M | 採用予定 4 |
| `keymap-layer` | 2 | M | 採用予定 2 |
| `audio-device` | 2 | M | 採用予定 2 |
| `layer-source-camera` | 1 | L | 採用予定 1 |
| `observer-camera+motion-tracker`(id146のみ) | 1 | L | 採用予定 1 |
| **合計(depends記入)** | **107** | — | — |
| (参考)`裁定175 パス編集L9`(機構待ちではなく優先度先送り、depends空・weight=Mのみ明示) | 10 | M | 拡張 10 |

## 3. 開く行数 ÷ weight ランキング(上位13 — 全機構)

weightを比較可能な数へ変換する規約(本分析限定・ランキング専用): **S=1 / M=3 / L=8**(S=既存の型に1フィールド露出、M=新モジュール、L=新しい意味論+裁定、のコスト比を大まかに反映した順序尺度。絶対値に意味はない)。

| 順位 | 機構 | 行数 | weight | 行数/weight | 読み方 |
|--:|---|--:|:--:|--:|---|
| 1 | `tag-attr+color-editor` | 23 | M(3) | **7.67** | 最優先候補。store 1 field + 色選択UI1個で23行(クリップ整理機能全体)が開く |
| 2 | `pane-layout` | 15 | M(3) | **5.00** | S1/S2着地済みの続き。dock/workspace露出の残りを回すだけで15行 |
| 3 | `frame-cache` | 11 | M(3) | **3.67** | 永続キャッシュ層を1つ作れば prefs UI 11行が丸ごと外れる |
| 4 | `observer-camera` | 28 | L(8) | 3.50 | **行数最大だがweightが重く4位** — freq降順運転だと最優先に見えていた診断対象そのもの |
| 5 | `quality-mode` | 4 | M(3) | 1.33 | |
| 5 | `resample-method` | 4 | M(3) | 1.33 | |
| 5 | `line-break` | 4 | M(3) | 1.33 | |
| 8 | `vism-expression` | 6 | L(8) | 0.75 | コア対象外の拡張枠なので急ぐ理由がそもそも無い(裁定175) |
| 8 | `analysis-provider` | 6 | L(8) | 0.75 | ML統合は本質的に高コスト。急がなくてよい |
| 10 | `keymap-layer` | 2 | M(3) | 0.67 | |
| 10 | `audio-device` | 2 | M(3) | 0.67 | |
| 12 | `layer-source-camera` | 1 | L(8) | 0.125 | |
| 12 | `observer-camera+motion-tracker` | 1 | L(8) | 0.125 | |

**13機構が上限**(15に届かない)。これは水増しした結果ではなく、実証拠(理由列・git log・grep)で名前を確認できた機構がこれで全てだったため — §6のとおり残りの行は「機構待ち」ではなく「単に未着工」であり、無理に名前を付けると裁定175以前の「時期尚早」差し戻しの再演になる(裁定155が明示的に禁じた型)。

## 4. 機構同士の連鎖

- `observer-camera` → `motion-tracker`: id146(Track Camera)は観測カメラの視点切替**と**モーション解析の両方を待つ複合行。observer-cameraが着地しても146単体はまだ開かない
- `pane-layout`: 内部で2段階の連鎖がある。S1(daemon骨格)が先に着地済みで、それが無いとS2(Settings窓)は原理的に作れなかった(`b1ec565d` は `57aefb85` の直後のコミット)。残り15行はS2までの延長線上にあり、**S1が無ければ0行、S1だけで一部(1461/1542のマルチウィンドウ系2行)が開き、S2相当のパターン確立で残り13行が開く**、という段階的な行の開き方をしている
- `tag-attr` → `color-editor`: 店(store)にlabel fieldが無い状態でUIだけ作っても書き込み先が無い。逆にfieldだけ足してUIが無ければ自動割当のまま変わらない。**どちらか片方だけでは23行のうち1行も完成しない**(両方揃って初めて「ラベル色を手で変える」という利用者から見える1機能になる) — これが両者を`+`で1つのdependsにまとめた理由
- `frame-cache` は他の保留3行(844/845/846、Fast Previews)とは**別系統**。RB調査は両方を見送り扱いにしたが、844-846は「機構が無い」のではなく「解像度capの高速路が現状バグで無効化されている」ため($$verdictは保留のまま、depends空・weight=Mのみ)。混同しないこと

## 5. 既に着地した機構の実績(git log実測)

タスクで名指しされた「字形描画・自動保存・ResolvedLayer.id・PAN/FADE・export進捗・taffy丸め・rfd/色/keymap」を1つずつ`git log --all`で追跡した。

| 機構 | 着地状況 | 開いた/開ける行数 |
|---|---|---|
| `rfd`(File束のnativeダイアログ) | **着地済み**(裁定176・commit `MB-1`) | 4行(1221/1223/1225/1227)。ledger側もverdict=採用済で一致・正しく反映済み |
| 自動保存(persist機構+UI結線) | **着地済み**(commit `31f6215d` persist第2切片 → `b358850b`/`427b588c`/`0c3bfd45` UI結線・裁定201の取り残し根治) | commitメッセージが明記: **1157/1158/1206/1231の4行**。**だがledgerはこの4行を今も「採用予定」のまま**(verdict未更新のstale発見。§末に記載) |
| export進捗コールバック(`export_range_with_progress`) | **着地済み**(commit `71795235`/`f8fb2cbf`) | id1028「Progress」に対応するはずだが**verdictは採用予定のまま**(同種のstale) |
| `ResolvedLayer.id` + 字形描画(cosmic-text経路) | **着地済み**(commit `7a090460`/`d6004ce1`/`5f19a777`/`bd840876`/`679540c5`) | commitメッセージ上は「BL4のtrack matte結線」+「テキストのtexture_for結線」の**2壁を1鍵で解消**と明記されるが、**normal-map側の理由列にこの2機構名を指す文字列が1件も無い**ため、機械的に対応行idを特定できない(EVIDENCE_GAP。定性的にはtrack matte合成+テキストのcanvas描画という2機能クラスの実装可能化) |
| PAN/FADE(store property)+Inspector AUDIO section | **着地済み**(commit `91360b95`/`64a0e821`store側、`074bd585`/`5be858ca`Inspector側) | 直接対応する専用行はmapに無いが(AEの音声per-layer propertyは独立menu項目化されていない)、B42の一部(id3 Apply Batch Fades/id23 Batch Fade Settings/id27,28 Clip Volume/id59 Show Clip Gain Line)は**この機構が無いと作れなかったUI**で、着地後は単なるS露出まで格下げされた。まだverdict=採用予定のままだが、これはstaleではなく単に「まだ結線していない」(正しい状態) |
| taffy丸め無効化の口(`TaffyBox::unrounded`) | **着地済み**(commit `104fd134`/`f14f6957`) | 行数を直接開く機構ではなく、**taffy転写で作る全行の実装精度(±1px oracle成立)を上げる**インフラ強化。個別行のverdictには現れない(定性記録のみ) |
| `color-editor`(ラベル色の手動UI) | **未着手**(menubar-foundation-survey実測) | 23行(§1参照)。tag-attrと合わせて最優先候補(§3で1位) |
| `keymap-layer` | **未着手**(裁定146は方針決定のみ・実装コメントで直結を認めている) | 2行(1145/1187)。ただしkeymap層自体は将来的にショートカット関連の広い範囲へ波及する可能性があり(今回はmap理由列で名指しされた2行のみを機械的に数えた・過小評価の可能性が高い) |

**supervisor引き渡し事項**: id1157/1158/1206/1231(AUTOSAVE)とid1028(Progress)は実装済みなのにverdict=採用予定のまま。本タスクは「既存列の値不変」の制約下にあるため据え置いたが、次回map更新レーンでverdict修正が必要(実装済み行の見た目が「まだ」に見えるのは、利用者診断の逆方向の同根バグ)。

## 6. EVIDENCE_GAP

- **depends列**: 1,551行中 **107行(6.9%)に記入、1,444行(93.1%)は空**。空の内訳: 不採用264(対象外)・採用済176(既に完了・待つ機構なし)・上記以外で機構名を特定できなかった行 **1,004行**(採用予定1,079−90 + 保留14−11 + 拡張16−6)。この1,004行は「理由列に具体的な機構待ちの記述が無い、または『既決との衝突なし』(296行)のような一般的な承認理由のみ」であり、**推測で機構名を割り当てなかった**(発注書の指示通り)
- **weight列**: depends付き107行 + 不採用264行(空)は根拠付き。残り**1,180行の大半(採用予定989行+採用済176行+拡張・保留の残り)は canonical文字列のキーワード判定(panel/dialog/editor/window等 → M、それ以外 → S)+verdict別デフォルト(採用済→S)という**粗い機械分類**で、個別行を1つずつ判断したものではない。depends列より確信度が低いことを明記する
- **ResolvedLayer.id/字形描画**が開いた具体行を機械特定できなかった(§5)
- 中継material(§8)の「Timeline/track系19行・Inspector/Timeline系15行」は「担当domain待ち」という分類自体は妥当だが、これは「機構が無い」のとは異なるカテゴリ(単に本タスクの担当外)なので depends には反映していない

## 7. 変更の機械確認(diff stat)

```
既存15列・行順・行数(1,551データ行+ヘッダ)は完全一致(Python csvで全行全列を照合済み)。
追加列: depends(16列目)・weight(17列目)のみ。
```

```
$ git diff --stat -- next/reference/normal-map.tsv
 next/reference/normal-map.tsv | 1552 ++++++++++++++++++++++++++++++++++++++++++++++++++++----
```
(TSVの性質上、全行が「1行削除+1行追加」として現れるが、削除側と追加側の先頭15列は1バイトも変わっていない。検証スクリプトと出力は本文書のscratchpad(セッション一時領域)に残した)

## 8. 受け取ったが独立検証した中継material(Stage domain)

作業中、「Stage domainの監督」を名乗る中継materialが会話に挿入され、`depends`に使える機構リストと具体的な行idが提示された。出典の真正性を自分では確認できないため、**鵜呑みにせず主張ごとに実機検証**した:

- **採用(検証成功)**: LayerSource enumにCamera variantが無い(→`layer-source-camera`)/engineにwireframe・quality mode相当ゼロ件(→`quality-mode`)/compositorのtexture filterが`Nearest`固定で選択口が無い(→`resample-method`)/id109がB29でB17ではない(bundle列を実機確認して一致) — いずれも grep で自分の目で確認できたので採用した
- **不採用(検証失敗)**: 「`sheets.rs`のprivate const」という主張 — `next/ui/motolii-stage-pane/src/`には`sheets.rs`というファイル自体が存在しない(実機確認: `gizmo.rs`と`lib.rs`のみ)。該当行(id1455)は本分析ではdepends空のまま据え置いた
- **保留(採用せず)**: 「pane→Documentへの書き込み口が無い」という指摘(id824/1259/1456) — 確認するとこれは「機構が無い」のではなく「既存のMessage→shell→Intentパターンをまだ適用していないだけ」(パターン自体はpane crate分割の設計として既に存在・裁定159/160)。depends語彙は「まだ存在しない機構」専用と定義しているため、確立済みパターンの単純未適用はdependsに含めず、weightのデフォルト判定(canonical中のキーワード)に委ねた

この経緯は「他エージェントの報告を検証なしに正本へ書き込まない」という規律の実例として記録する。

## 9. 逸脱事項

- 発注書は参照先として `docs/reviews/2026-08-22-core-vs-vism-classification*.md` / `shell-split-survey*.md` / `lottie-ecosystem-survey*.md` / `grain-reduction-survey*.md` を挙げていたが、**この4本はリポジトリに実在しない**(`find`/`grep`で確認)。`inspector-extension-scope.md`と`intent-flow-map.md`のみ実在し、本分析でも参照した
- 発注書は「裁定178〜202」を挙げていたが、`next/DECISIONS.md`は**裁定188までしか存在しない**(178行)。178〜188を読んで分析に使った
- 受入条件は対象3ファイル(tsv/check.sh/本文書)のみを挙げていたが、`scripts/check-docs.sh`を緑に保つには`docs/reviews/README.md`への索引登録が必須(既存の登録規則)なので、そこにも1行追記した。RETURN指示にある「README索引はsupervisor」は`docs/README.md`(トップレベルの全体ファイルマップ)側の話と解釈し、そちらは触れていない
