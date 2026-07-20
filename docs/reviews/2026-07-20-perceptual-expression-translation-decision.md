# 知覚表現の翻訳 — Motolii Hostの役割（2026-07-20）

状態: **決定**。本書は既決のVism北極星、軽快さ、純関数評価、Preview / Export、Rerun転移方針を一つのHost命題へ統合する。新しい公開API、Document field、plugin kind、Rerun SDK依存を許可する文書ではない。

関連正本: [コンセプト](../concept.md)、[Vismコンセプト](../vism-package-concept.md)、[シミュレーションモデル](../simulation-model.md)、[性能モデル](../performance-model.md)、[Rerun先例調査](2026-07-20-rerun-prior-art-survey.md)、[Rerun学習・転移計画](2026-07-20-rerun-learning-transfer-plan.md)

## 1. 決定

Motoliiは、持ち運べる映像表現Vismを演奏・編集する最初のHostであり、クリエイターの感覚的な意図を、軽量で再現可能な画へ翻訳する。

このHostの働きを説明する内部概念として**知覚表現の翻訳**を採用する。「知覚表現コンパイラ」はその短い比喩として使用できるが、[concept.md](../concept.md)の一文定義、Vism、Kit、Project、plugin capabilityを置換する製品種別または新しい実装層ではない。

```text
クリエイターの意図
    ↓  制作語・直接操作
Vism / 型付き表現 / Project recipe
    ↓  Motolii Hostによる軽量な評価
Draft Preview ── Quality ── Final Preview / Export
```

## 2. 借りる厳密さ、借りない厳密さ

CAD、計測、科学可視化、Rerun等の工業・技術系先例から借りるのは、モデルとviewの分離、時間・空間に沿う観測、座標系、正本と派生物、resource lifecycle、再現手順、高密度UIを成立させる境界である。

Motoliiは物理量の測定器ではない。全環境でのbit一致、知覚不能な浮動小数点差、現実の物理への忠実度は完成条件にしない。一方、次は近似の対象にしない。

- 同じDocument、素材、plugin version、時刻、seed、Qualityから同じ意味の結果を再生成できること
- シーク順序や再生順序へ依存しないこと
- 色空間、premultiplied alpha、正準座標、TimeMap等、画の意味を支える契約
- PreviewとExportが同じ`render_frame(t, Quality)`を通り、別の評価経路を持たないこと
- Undo、再読込、欠落plugin診断、cache invalidationで作品意味を裏切らないこと

要約は**「意味は厳密に、計算は大胆に近似し、品質は目的に応じて落とし、最後は画で裁く」**とする。

## 3. 正直なシミュレーションより視覚的効能

Motoliiが通常提供するのは、物理現象そのものではなく「重そう」「漂う」「弾ける」「密度が増す」等の視覚的効能である。第一選択は閉形式、補間、決定的ノイズ、解析的な反射等、時刻`t`から直接評価できる安い`f(t)`とする。

これはSimulationPluginを廃止する決定ではない。正典の時間軸自由度5段はしごを維持し、本当に逐次履歴が必要な表現だけをHost管理のbake境界へ送る。bakeを中心思想や高度さの証にせず、隠れ状態を作らないための限定された正規ルートとして扱う。

Furikake型の価値は、汎用シミュレータを露出することではなく、少ない入力と安い評価から画面上の密度、偶然性、動勢を即時に得られることにある。

## 4. Draft / Finalの約束

Draft Previewは半解像度、fp16、色変換ショートカット、draftサンプル数等の知覚可能な品質差を許し、即時反応と試行回数を優先する。したがって「DraftとExportの知覚的一致」を最上位の公約にしない。

守るのは評価意味と経路の一致である。制作者は必要な時にFinal品質Previewへ切り替えられ、Exportだけが別の実装、別の状態、別の時間解釈を使って不意に異なる画を出してはならない。

## 5. Rerunの位置

Rerunは開発用観測器へ縮小しない。既決どおり、時間、view、selection、density、GPU scene、高密度viewer shellを、Motoliiのポップで直接操作可能な映像制作言語へ再翻訳する主要な**製品先例**である。

Rerunの公開ソースは、その責任分解、component、GPU接合、cache、試験方法を調べる実装地図として使う。Rerun SDKをMotoliiの開発telemetryへ組み込む案は未決であり、本決定から依存追加、recording store導入、Document field追加を導かない。

発注ではMotolii仕様と現行コードgapを先に固定し、Rerunは裁定済み先例として後からだけ入れる。Rerun assetを目的または仕様正本にした発注は通さない。強制動線、必須ラベル、STOP条件、負例は[Rerun学習・転移計画 §9](2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)を正本とする。

## 6. Vismとの接続

Vismはコンパイル後の最終画素や動画ではない。時間、入力、型付きparameterから働く一つの持ち運べる映像表現であり、Motolii HostがProject上の配置、変調、素材と合わせてPreview / Exportへ評価する。

したがって本決定はVism北極星を置き換えず、HostがVismを制作者へどう届けるかを補う。

- **圧縮**: 複雑な現象の視覚的効能を、少数の型付き意味と安い評価へ畳む
- **翻訳**: 内部の数式、GPU処理、依存関係を、制作意図と直接操作へ変える
- **即応**: 制作者が結果を見ながら探索できる速度で返す
- **可搬**: 得られた表現をProject内の手順へ閉じ込めず、Vismとして保存・共有・再利用可能にする

## 7. 非目標と未決

- `知覚表現コンパイラ`という新crate、trait、schema、pipeline stageを作らない
- 工業系先例を理由に物理忠実度、bit一致、汎用CAD / CAE機能を完成条件へ加えない
- すべてのparameter名を知覚語へ置換しない。制作語と正準意味の対応規範はNodeDesc / UI言語の別決定とする
- spring / inertia系を新しいdriver scopeへ戻さない。既決の補間型、型付きlink、Simulation境界を優先する
- RerunのEntity、Blueprint、store、時間意味、UI tokenをMotoliiのDocumentまたは公開契約へ流用しない
- 本書の標語を、対応する審判なしに性能・互換・再現性の外向き保証へ昇格しない

## 8. 実装への判定

新機能がこの軸に沿うかは、物理的に正しいかではなく次で判定する。

1. 制作者が欲しい視覚的効能を、実装方式ではなく制作意図として説明できるか。
2. より安い`f(t)`、既存補間型、型付きlinkで成立しないことを確認したか。
3. 近似による差が作品意味、色、座標、シーク、Preview / Export共通経路を壊さないか。
4. 操作から結果までの待ちと判断を減らし、一定時間内の試行回数を増やすか。
5. Host内部の厳密さを、内部数値や専門語の操作として制作者へ転嫁していないか。
6. Vismとして再利用できる表現を、特定Projectの手順またはMotolii内部実装へ閉じ込めていないか。
