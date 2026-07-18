# Vism — 持ち運べる映像表現

作成日: 2026-07-17

状態: **長期コンセプト・名称・拡張子決定／ファイル形式未決**。Motoliiから持ち運べる映像表現を**Vism（ヴィズム）**、拡張子を**`.vism`**とする。一方、container、manifest field、MIME、署名、動的ロード、marketplace、複数capabilityの同梱規則は未決であり、本書はv1のloader実装許可ではない。

関連正本: [コンセプト](concept.md)、[Vism / Kitモデル](vism-kit-model.md)、[小さなコアと探索可能な拡張](extensible-core-model.md)、[プラグイン作者向け規約](plugin-authoring.md)、[ジェネラティブユーザー境界](generative-user-boundary.md)

実装順と停止ゲート: [Vism実装計画](reviews/2026-07-17-vism-implementation-plan.md)

## 1. 一文で

> **Vismは、時間に沿って働く一つの映像表現を、作品やHostから切り離して保存・共有・再利用するための配布単位である。**

MotoliiはVismを扱う最初のリファレンスHostを目指す。VismはMotolii projectの別名でも、Motolii専用plugin binaryの呼称でもない。憲法上の可搬先はMotoliiの公開契約を引き継ぐ互換Host／fork群とする。他製品adapterは可能性として残すが、全映像ソフト共通規格やVismの完成条件にはしない。

```text
                 one Vism
          時間・入力・parameter
                    ↓
              映像／制御値
                    ↓
       ┌────────────┼────────────┐
       ↓            ↓            ↓
    Motolii      互換fork     任意のadapter
```

すべてのVismが全forkで無条件に動くことを保証するのではない。fork固有能力を要求するVismはあり得る。重要なのは、要求する型と能力、非互換理由を宣言し、Host名、OS、GPU vendor、UI実装への暗黙依存として隠さないことである。具体的な別Vismをconsumerから直接参照せず、型付きinputを宣言し、provider選択はKitへ置く。

## 2. なぜpluginと別の言葉が要るか

`Filter`、`Composite`、`LayerSource`、`ParamDriver`等は、Hostが評価と責任を割り当てる**内部の実行分類**である。制作者が探すものは実装分類ではなく、「Glow」「Lyrics」「Beat Pulse」「Particle」のような一つの表現である。

| 層 | 問うこと | 語彙 |
|---|---|---|
| 制作者 | 何を作品へ追加できるか | **Vism** |
| 構成作者 | どのVismをどう接続して用途にするか | **Kit** |
| 配布・管理 | 誰が作り、何が必要で、どの版か | package identity / manifest |
| Host | どの入口で評価し、何を所有するか | plugin capability / kind |
| 実装 | どう計算するか | Rust、WGSL、将来のWASM等 |

一つのVismが一つのplugin kindと一致するとは限らない。ただし、独立して更新・差替えできるproviderとconsumerを一packageへ詰めず、まず小さなVismとKitの接続で表せるかを審判する。複数entryは同一lifecycle／compatibility責任から分離できない場合の候補であり、万能bundleの既定にはしない。

> **Vismはユーザーが持ち運ぶ表現の単位、pluginはHostが実行責任を分ける境界である。**

## 3. Project、Preset、Asset、Cacheと混ぜない

Vismは次の成果物と責任が異なる。

| 成果物 | 正本 | Vismとの関係 |
|---|---|---|
| **Project Document** | Timeline、Object、参照、plugin instance、parameter | Vismの安定ID、version、instance payloadを参照する |
| **Preset / Recipe** | 既存能力の設定・接続 | Vismを要求できるが、実装codeそのものではない |
| **Vism** | 再利用可能な表現とその要求能力 | Projectから独立して配布・更新される |
| **Kit** | Vism要求、型付き接続、初期値、素材要求 | HostがpreflightしProjectへmaterializeする |
| **Asset** | 画像、動画、音声、font、SVG、glTF等 | Vismが宣言的に要求できる |
| **Bake / Cache** | 再生成可能な評価結果 | Hostが所有し、Vismの正本へ入れない |
| **Workspace / User settings** | UI配置、最近使った値、端末固有選択 | VismにもProjectにも入れない |

VismへProject全体、ユーザーのTimeline、Hostのcache、UI window配置を詰め込まない。複数Vismの用途構成はKitへ置く。逆にProjectへVismの実装を秘密裏に埋め込み、自動実行させない。

## 4. 三つの層を分離する

### 4.1 Expression contract

Hostを越えて持ち運びたい意味である。

- 安定した表現identityとversion。
- 型付き入力、出力、parameter、default。
- 時刻、seed、Quality、必要なscope、型付きinput／output。
- 要求capabilityと、対応しない時の診断。
- PreviewとExportで共有する評価意味。
- migrationまたは互換性判定に必要な宣言。

この層へMotoliiのSlint型、Timeline row、window座標、内部Document layout、CUDA/Metal/DX等を出さない。consumer Vismはprovider VismのIDでなく必要な型を宣言する。

### 4.2 Package

表現を検査、導入、更新、再現するための配布面である。長期的な候補には次があるが、fieldとcontainerは未決とする。

- 作者、由来、license、version、互換範囲。
- capability宣言、型付きport、外部asset、実装上のpackage依存。
- 実装source、WGSL、将来のWASM、またはbuild recipe。
- icon、短い説明、preview、使用例、検証fixture。
- migration情報、conformance結果、署名・trust情報。

Vismは必ず実行codeを含むとは限らない。既存capabilityを型付きに合成した宣言的表現、WGSLだけの表現、Host buildが必要なsourceを含む表現等を、同じ名前へ無理に畳まず比較する。Presetとの境界は「設定差分」か「独立した表現identityと互換責任を持つか」で審判する。

「ネイティブに評価すること」と「OS別binaryを配ること」は別問題である。source配布＋Host側buildは比較候補の一つだが、WGSL、WASM、native artifact等とtrust、再現性、build時間、作者DXをv2の配布設計で比較する前に既定へしない。

### 4.3 Host integration

Motoliiが引き受ける投影である。

- 発見、install/update/remove、依存・trust診断。
- NodeDesc等からの標準UI生成。
- Timeline、Inspector、Stage、parameter source等の文脈別入口。
- Undo、single writer、Document保存、欠落時保持。
- GPU resource、cache、StateTrack、Preview/Export。
- error、Cancel、復旧、accessibility。

同じVismを互換forkが扱う時、この層はそのHostが実装する。Motolii固有のUIをpackageの唯一の操作面にしない。互換性の反証には、別製品ではなくMotoliiのUI／Document実装を使わないheadless compatible runnerを用いる。

## 5. Vismが発明できるものと、Hostへ残すもの

Vismは未知の**名詞**と**動詞**を発明できる。

- 名詞: 粒、文字glyph、追跡点、解析領域、未知の評価domain。
- 動詞: Glow、Fold、Kick、Pin、Exclude、未知の表現操作。
- 計算: Filter、Generator、Driver、Simulation、Authoring Tool等。
- parameter、入力、出力、表現固有の型付きpayload。

Hostへ残すもの:

- identityの保持と参照解決の外殻。
- 時間、評価順、循環拒否、依存。
- 保存、Undo、version、欠落、診断、再導入時復元。
- Preview / Commit / Cancelの操作文法。
- resource、cache、Quality、Preview/Export。
- installとproject openの分離、trustと権限。

自由なscript panel、任意Document mutation、名前検索、独自Undo、隠れ状態を「Vismだから」と許可しない。[小さなコアと探索可能な拡張 §8](extensible-core-model.md#8-表現の種類をコアへ列挙しない)の憲法を配布面へ延長したものがVismである。

### 5.1 Vism間接続はKitへ置く

Vismは別Vismの実装identityを直接要求しない。`BeatEvents`、texture、DataTrack等の型付きinputを要求し、Kitが具体的なprovider、consumer、接続、初期値、assetを選ぶ。

v1のKitは宣言をProjectへ1 macroでmaterializeし、展開後は通常のVism instanceと接続として編集する。Kit runtimeを常駐させず、Kit更新で既存Projectを自動変更しない。詳しい責任とBPM例は[Vism / Kitモデル](vism-kit-model.md)を正本とする。

## 6. Projectから見たVism

ProjectはVism実装の内部構造ではなく、少なくとも概念上次を参照する。

```text
vism identity
+ compatible version requirement
+ selected capability / entry
+ typed instance payload
+ declared asset references
```

具体fieldは未決である。固定するのはlifecycleである。

1. Vismが存在すれば、Hostは互換性を判定して評価する。
2. 欠落・非互換でも、未知payloadを削らずProjectを開く。
3. 無関係なDocument領域は編集できる。
4. 最終結果に必要なVismを評価できなければ、似た絵へ黙ってfallbackせずexportを型付きで拒否する。
5. 互換Vismの再導入後、保持したpayloadから復元する。

Projectを開く操作は、Vismをinstallしたり同梱codeを実行したりしない。導入は由来、要求能力、権限、build結果を確認できる別の操作にする。

## 7. First-partyもVism境界の上に置く

標準搭載のGlow、Lyrics、Particle等は、配布上はMotoliiに同梱されても、実装上の特権を持たない。

- 公開plugin contractだけで書く。
- 第三者と同じparameter、resource、diagnostic、missing/version規則を通す。
- 独自UIだけに保存値を隠さない。
- Host内部APIが必要なら、表現専用の裏口ではなく欠けた共通能力として審判する。
- 参照実装、scaffold、testkit、conformance fixtureとして公開する。

v1の静的リンクされたfirst-party pluginは、Vism package実装そのものではない。将来のpackage境界を反証する**pre-Vism reference**である。

## 8. 先行文化との関係

VismはVSTの映像版binary互換ではない。VSTから借りるのは、Hostと専門実装、parameter、automation、保存互換を分けて作者と制作資産の生態系を成立させた構造である。

- [OpenFX](https://github.com/AcademySoftwareFoundation/openfx): Host横断の画像処理plugin契約。VismはFilter以外の生成、解析、制御、作品内lifecycleまで問題にする。
- [ISF](https://github.com/mrRay/ISF_Spec): shader＋metadataによる自己記述型表現、自動UI、複数Hostという最も近い先例。Vismはplugin所有のframe永続bufferを標準状態境界として採らず、逐次状態はHost管理のSimulation / Bake境界へ置く。
- frei0r / FreeFrameGL: 小さな映像effectを複数Hostへ持ち運んだ先例。VismはGPU vendor非依存、型付き時間・parameter、欠落・versionを強くする。
- AviUtl文化: 小さな作者がHostの想定外の表現を作り、共有した実証。Vismはその発明速度を残し、導入、依存、version、診断、復元を属人的手順からHost契約へ移す。

設計時は「ISFの再発明ではない理由」「ISF import／adapter／非採用のどれを選ぶか」を明記する。先例の名前やcontainerを写すのではなく、成功した責任分離と失敗した自由を比較する。

## 9. 名前と拡張子

ユーザー向け名称は**Vism（ヴィズム）**、拡張子は**`.vism`**とする。`Visual Module`等の正式なバクロニムを与えず、一つの映像表現・作風を持ち運ぶ固有名詞として扱う。

想定する利用者語彙:

- Vismを追加する。
- 歌詞Vismを探す。
- このProjectで使っているVismを確認する。
- Vismが欠けている／更新できる。

```text
glow.vism
lyrics.vism
beat-pulse.vism
particle.vism
```

拡張子の確定は、内部形式の確定を意味しない。container、MIME、署名、OS登録、source/binary分離、schema、配布方式は別の審判を通す。`.vism`という名前からZIP、JSON、単一binary等の実装を逆算しない。

## 10. 現在地

| 項目 | 状態 |
|---|---|
| 持ち運べる映像表現をProjectとHostから分離する | **長期コンセプト決定** |
| Motoliiを最初のリファレンスHostとする | **長期コンセプト決定** |
| Vismと内部plugin kindを分離する | **設計原則** |
| Vismは型を要求し、具体provider選択をKitへ置く | **設計原則** |
| v1 Kitはmaterialize型 | **設計方向／schema未決** |
| 可搬先をMotolii互換Host／fork群とする | **長期コンセプト決定** |
| Project openとVism installを分離する | **安全原則** |
| first-partyへ特権を与えない | **設計原則** |
| Vismという名称 | **決定** |
| `.vism`拡張子 | **決定** |
| manifest / container / MIME / signing | **未決** |
| 1 package内のcapability数 | **未決** |
| Kit container／拡張子／linked update | **未決** |
| source / native binary / WGSL / WASMの同梱方式 | **未決** |
| marketplace / registry / trust policy | **v2・未決** |

## 11. 実装へ進む前の停止線

v1では既存の静的plugin境界とfirst-party参照実装を育てる。次を満たす前にVism loader、`.vism` parser、registry、marketplaceを実装しない。

1. Filter、ParamDriver、Generator、Simulation等、複数の実pluginで公開境界をコード実証する。
2. first-party実装が内部APIなしで成立する。
3. Vismが別VismのIDでなく型付きinputを要求し、Kitがproviderを選ぶfixtureを作る。
4. package、entry、Kit、Project instance、artifact identityの違いをfixture化する。
5. 欠落、非互換、未知payload、migration、再導入をsave/reloadで反証する。
6. source配布、Host build、WGSL、WASM、native binaryのtrust・可搬性・DXを比較する。
7. Motolii内部UI／Documentを使わないheadless compatible runnerで契約漏れを反証する。
8. ISF/OpenFX等とのimport、adapter、非互換範囲を決める。
9. 反対側レビューとv2の配布ゲートを通す。

この段階までは、Vismは機能追加の口実ではなく、現在のplugin境界を将来閉じないための審判である。
