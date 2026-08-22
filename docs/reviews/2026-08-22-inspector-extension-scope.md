# Inspector 拡張の射程 — 「レイヤーとして現れる物は Inspector」原則の適用(2026-08-22)

状態: **裁定184(原則)+適用表は案**(利用者の赤入れ待ち)
発端: 利用者裁定「テキストプロパティなど別窓は恐らくインスペクターで話ができる。主に**タイムラインにレイヤーとして現れる物はインスペクターで処理した方が動線がいい**」(動線図 §1.1 の家未決への回答)

## 1. 裁定184(原則)

**タイムラインにレイヤー(クリップ)として現れる物の調整は Inspector が家。** 根拠は動線の一点性 — 選択(Timeline/キャンバス)→調整(Inspector)の視線往復が1本に固定され、レイヤー種別ごとに別窓を開く分岐が消える(裁定177「1意図=1つの家」の空間版。AE の Character/Paragraph/Audio 別パネル散在への反論)。他の面(キャンバスハンドル・右クリック・メニュー)は**入口**であって家ではない(動線図の「shortcut は入口」と同じ整理)。

**判定式**: その意図の対象が「選択中のレイヤー」なら Inspector。対象がプロジェクト・出力・環境・画面そのものなら Inspector ではない。

## 2. 適用表(案) — 専用パネル勢と家未決12束の再判定

### 2.1 Inspector へ引っ越す束

| 束 | 旧 home | 判定 | Inspector での形 |
|---|---|---|---|
| B46 テキストプロパティ | Character/Paragraph 別窓 | **Inspector**(利用者名指し) | テキストレイヤー選択時に TEXT section が現れる(型別 section) |
| B42 音声内容整形 | Audio パネル | **Inspector** | 音声レイヤー選択時に AUDIO section(gain/pan/fade)。コンポ全体系(マスター)は対象がレイヤーでないため対象外 → §3 |
| B04 字幕 | Caption パネル | **Inspector**+Timeline | 字幕はレイヤーとして現れる前提なら通常のテキスト系 section。字幕トラック一括ナビだけ Timeline 側 |
| B16 解析→自動KF | Analysis/Tracker 別窓 | **Inspector** | 対象レイヤー選択時の ANALYZE section(実行ボタン+結果はキーフレームとして Timeline に落ちる) |
| B38 エフェクト適用 | Effects/Transitions パネル | **Inspector**(編集の家) | 適用済み stack の編集= Inspector EFFECTS section(現 ATTRS の延長)。**探す・掛けるの入口= Browser effects タブ**(実装済み)からの drag |
| B02 マスク/マット | 未決(Inspector/キャンバス) | **Inspector** | MASK section。キャンバスハンドル=直接操作の入口 |
| B03 ラベル色 | 未決(右クリック/Inspector) | **Inspector** | ident 帯の色チップ。右クリック=入口 |
| B07 プリセット | 未決(Inspector上部/別ブラウザ) | **Inspector** | section 上部の適用/保存。閲覧の入口= Browser |
| B37 リタイム・B44 変形・B01 ブレンド・B05 の色補正側 | Inspector(既定) | 変更なし | 既存 section のまま |

### 2.2 Inspector に入れない物(判定式の否定側)

| 束 | 理由(対象がレイヤーでない) | 家 |
|---|---|---|
| B09 書き出し | 対象=出力 | Export ダイアログ |
| B10 プロジェクト整理 | 対象=プロジェクト | Project Manager |
| B12 環境設定 | 対象=環境 | ポップアップ(裁定182) |
| B26 ワークスペース | 対象=画面配置 | メニュー+drag(pane_grid) |
| B25 パネル可視性 | 対象=画面 | パネルタブ+メニュー |
| B06 ヘルプ | 対象=文書 | メニューバー |
| B05 のチャンネル表示側 | 対象=Viewer の見え方(レイヤー属性ではない) | Viewer オーバーレイ |
| B23 解像度/画質・B22 ガイド・B17 カメラ視点 | 対象=Viewer | Viewer(状態帯) |

### 2.3 残る本当の未決(この原則でも解けない物)

| 束 | 論点 |
|---|---|
| B15 キーフレーム | 家は Timeline(キー行実在)で確定的だが、**Graph Editor を将来立てるか**だけが残る(現時点は不要 — 速度/補間は Inspector+キー行で足りる) |
| B19 マーカー | Timeline レーンで確定的(Markers 一覧パネルは入口に降格でよいか、だけ) |
| B08 取り込み | Browser drop が家・File>Import は入口(S6 併存)— 確認のみ |
| B09 書き出しの形 | ダイアログ1枚か Render Queue 常設か(freq 的にはダイアログで開始が軽い) |
| B40 ソース参照・マルチカム | マルチカム自体が拡張圏 — 保留のまま |
| B31 選択 / B34 グループ化 / B28 描画 | 家=キャンバス+Timeline(操作の場がそのまま家)。メニュー/右クリックは入口 — 確認のみ |

## 3. Inspector 構造への含意(実装の形)

1. **型別 section**: 現行の常設4 section(裁定: Inspector=A 案)に「**選択レイヤーの型で現れる section**」の層を足す — TEXT / AUDIO / MASK / EFFECTS / ANALYZE。mock v3.1 の section 文法(--section 高さ・fold)をそのまま増設する形で、新しい視覚文法は不要
2. **スクロール税**: section が増えると縦に伸びる → fold 既定(型 section は開・汎用 section は現状維持)+ section ジャンプ(将来)。**別窓化はしない** — 縦に長い1本の方が、どこにあるか分からない別窓より動線が短い(本裁定の趣旨そのもの)
3. **多選択**: 型が混在する複数選択では共通 section(TRANSFORM 等)のみ表示 — 既存の投影規則の延長
4. **コンポ選択**: レイヤー非選択時= Composition の Inspector(background 等、既存)。マスター音声などコンポ級はここに住める(B42 の残り)
5. 実装順(棚崩しの束単位): TEXT section(B46 — 字形描画の依存裁定が先行要件)→ EFFECTS section 整備(B38 — vism S1〜S5 済で土台あり)→ MASK section(B02 — MK2 済)→ AUDIO(B42)→ ANALYZE(B16)

## 4. 手続き

利用者の赤入れ後: intent-bundles.tsv の home 列を確定値へ更新(map の bundle 列は束 id 参照なので無変更)+ §2.3 の確認6件を裁定へ。
