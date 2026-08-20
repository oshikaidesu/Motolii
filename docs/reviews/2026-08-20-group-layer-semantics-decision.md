# グループレイヤーの意味 — プリコンポの3つの仕事を3つの明示機構へ

日付: 2026-08-20
状態: **起草**(supervisor 起草。DECISIONS への採番はレーン合流後。実装束の発注はこの文書を仕様源にする)
経緯: 利用者「グループレイヤーは motolii の意味論を出す唯一の特効薬かもしれない」。先人調査(Godot / Blender、一次資料)で3分解を検証済み。

## 原則

**暗黙を許さない。** AE のプリコンポは「変換の集約 / 畳んで見る / 集合への効果」という3つの別の仕事を、
1つの重い機構(常時レンダーターゲット化)に暗黙に抱き合わせた。Lottie の shape group は修飾子の効き先を
暗黙(配列の手前の兄弟)にした。両者の病は同じで、**Motolii は3つの仕事を3つの明示機構に分ける**。

先駆者の裏付け: Godot も Blender も、この3つを**別の生成物**として持つ。
- 変換: Godot = Node2D 継承(opacity と物理的に独立)/ Blender = parenting(Collection は変換を一切持たない、と設計文書が明言)
- 整理: Godot = エディタ専用メタデータ(実行時影響ゼロ)/ Blender = Collection(多対多メンバーシップ)+ Grease Pencil Layer Group(**公式が謳う効能は visibility / lock / onion skinning のみ — opacity/blend は非対象**)
- 集合への効果: Godot = `CanvasGroup`(専用ノード、バックバッファへ都度合成)/ Blender = View Layer → Compositor(レンダーパス化)

## 決定

### 1. 変換の集約 = `parent`(実装済み)

層の一般属性 `parent`(layer-meta 束で採用済、循環拒否済み)。変換だけが親子を流れる。
opacity・効果・可視性は**変換の木を流れない**。

### 2. 整理と畳み = グループ(Document)+ fold 状態(Session)

- **グループの所属は Document**(構造であり undo 対象)。Timeline の行の入れ子と検査系トグルの単位
- **畳んでいるか(fold 開閉)は Session**(選択・playhead と同じ家。undo で開閉が戻ってはいけない)
- グループが伝播させてよいのは**検査系トグルのみ**: hidden / solo / lock(m/s/l。層側は `LayerAttrs`)。
  実効可視性は Godot の `visible` と同じ**祖先チェーンの AND 導出**(状態のコピーを持たない)
- **グループは合成コストを持たない**。グループ化しても絵は1画素も変わらない(検査系トグルを除く)

### 3. 集合への不透明度・効果 = 隔離グループ(明示)

- グループの属性 `isolate: bool`(既定 false)。**true にした時だけ**、その部分木を中間テクスチャへ
  隔離合成してから、群としての opacity / blend / effect を掛ける(Godot `CanvasGroup` の重量クラス:
  フレーム内の部分合成。別レンダーパス方式=Blender View Layer は採らない — ノードグラフ UI は非目標)
- **継承 opacity(modulate 相当)を Motolii は持たない**。層の opacity は常に自分だけ(self_modulate 相当のみ)。
  群を薄くしたければ隔離グループにする、の一択。これで Godot proposal #7293(継承 modulate が
  CanvasGroup 境界を素通りして「重なりが濃くなる」を再発させる。2023年提起・未解決放置)の罠クラスが
  **プロパティが存在しないことによって**丸ごと消える
- **マスク(clip)と隔離(bake)は同一グループに同時適用不可**。Godot が `clip_children` × `CanvasGroup` を
  明示的に動作不能と文書化した形(沈黙のバグより明示の排他)を踏襲
- 重ね順あわせのために隔離グループを使うのはアンチパターン(Godot コミュニティで既知)。
  順序は depth_offset / z の仕事

### 4. ベイク(資産への焼き込み)は隔離とは別の明示操作

「群を1本の素材に確定させる」(プリコンポの残りの用途: time remap・重い群の固定)は、
隔離グループと別の**明示のベイク操作**(群 → メディア資産を生成し、群を置き換える)。
可逆性は Document の履歴が担う。実装は後日の束(export 経路の再利用)。

## 実装束への含意(発注時に写す)

- store: `Group` entity(member 列 + `isolate` + 検査系トグル)。層は複数グループに属さない(第1弾は木)
- m/s/l: `LayerAttrs` に `solo` / `locked` を追加(hidden は既にある)。シェイプ個別の表示/ロックは
  `Layer:shapes` 側の属性(シェイプ UI の日)
- compositor: `isolate` の部分木を ViewBuilder の別ターゲットへ → TexturedRect として親フレームへ
- 可視性の伝播はテスト行列を先に書く(Blender T73692: 成熟ソフトでも取りこぼした領域)
- Timeline UI: グループ行 = fold の口 + 検査系トグルの口。隔離は見た目で区別(暗黙に見えない隔離を作らない)

## 未決のまま残す物

- 隔離グループの入れ子の可否(Godot は資源競合で壊れた実績あり。第1弾は入れ子禁止から始めて実測で開ける)
- 隔離中間テクスチャの解像度の正本(裁定105 のラスタライズ解像度問題と同根)
- ベイク束の詳細(export 経路の再利用範囲)
