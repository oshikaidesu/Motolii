# M3完成像 — 並列発注時に全ownerが共有する1枚

日付: 2026-08-09
状態: **参照**。決定でもacceptanceでもない。

## 0. なぜこの文書があるか

**完成像が無いと、実装担当は「何のためか」を自分で埋める。**
埋めた像は人ごとに違うので、findingから自己発注し、メタ考慮が再帰し、
並列数が増えるほど発散する。2026-08-09の総監督sessionで観測された
「監督装置が成果物になる」ドリフトの一因はこれである。

**本書は新しい決定を1つも作らない。** 既に決まっているものを1枚へ集めるだけである。
`concept.md`、[M3 RN runtime実行地図](m3-rn-runtime-execution-map.md)、
UI設計decision、[M5採択地図](m5-known-implementation-adoption-map.md)からの導出であり、
**本書と現行正本が食い違ったら現行正本が勝つ。その時は本書を直す。**

発注時は本書をorderのREAD SETへ入れてよい。ただし
**本書をacceptance条件やoracleにしてはならない。**

### この像は既に仮コードで実証されている

本書は構想ではない。**完成像は2026-08-07/08に仮コードで一度実行され、
繋がる形と繋がらない箇所が具体名で出ている。**

| 実証物 | 何を出したか |
|---|---|
| [完成条件の鎖](reviews/2026-08-08-completion-condition-call-site-sketch.md) | 完成条件（3〜5分・音楽同期・音声mux）そのものの鎖を初めて1本書いた。**音声muxは実装済みなのに楽曲bedを作品へ据える編集操作が無い**ことを検出（`N-SOUNDTRACK-WRITE`） |
| [M4/M5仮接続](reviews/2026-08-08-m4-m5-call-site-connection-sketch.md) | **M4とM5はM3の後へ直列に積む別phaseではなく、PreviewとExportが共有する同じ背骨へ合流する**ことを呼び出し側から示した |
| [合成失敗14件](reviews/2026-08-07-call-site-sketch-composition-failures.md) | 決定どうしが同時に成立しない箇所 |
| [継ぎ目9件とStage×M5判定](reviews/2026-08-08-call-site-sketch-seams-and-stage-m5-verdict.md) | 区間をまたぐ合成失敗 |
| [仮コード成果物の保全](reviews/2026-08-08-call-site-sketch-artifacts.md) | 成果物そのもの |

背骨は1枚に収まる。

```text
Document snapshot
  -> build_document_frame_graph
  -> [M4: Host cache / resource admission]
  -> render_graph_cached
       -> [M5: LayerSourcePlugin / RenderStep::Plugin]
       -> Composite
  -> Preview / Export
```

- **M4**は背骨の**外側**で、同一recipeの成果物を再利用し、miss時は既存評価へ透明に戻る
- **M5**は背骨の**内側**で、既存 `LayerSourcePlugin` 席からpremultiplied RGBAを返し既存Compositeへ入る
- **M4 K1a resource ownerはM5にも共有される。**
  M4完了後にM5を始めるのではなく、同じHost所有境界を一度だけ閉じる関係である

**注意**: 仮コードは[器具境界決定](reviews/2026-08-07-provisional-call-site-sketch-instrument-decision.md)により
**非compile・非authority**であり、closed orderのAUTHORITY欄へ引いてはならない。
本書が引用するのは**接続形状の観察**であって、API名・schema・実装許可ではない。
`???` は希望API名ではなく、現行repoで実名を置けなかった契約境界である。

## 1. 完成条件（動かせない線）

`concept.md`:

> **MVを1本書き出せる**: 3〜5分・音楽同期の最終書き出し（音声mux込み）が完成条件

一人の制作者が、3〜5分のMVを最後まで完成できること。それだけである。
「音楽同期」は含意列挙では**音声mux**であり、拍同期編集ではない
（[完成条件の鎖](reviews/2026-08-08-completion-condition-call-site-sketch.md)で
起草者自身の読み替えを訂正済み）。

### 残工程に「開発」は無い

**M3もM4もM5も、新規開発ではない。** 残っているのは接続と採択だけである。
これは希望的観測ではなく、3つの正本を実測した結果である。

| | 中身 | 実測 |
|---|---|---|
| **M3** | 旧route（direct-wgpu/Vello + `ProductApp`）で**受入済みの意味資産** — Place、Undo／Redo、Timeline move／trim、Position key追加・値編集、easing、playback spine | 2026-08-07の再基線で**製品runtimeを失った**だけ。R1〜R4は繋ぎ直す工程 |
| **M4** | [採択地図](m4-known-implementation-adoption-map.md)は全classが `REUSE` / `PATTERN` / `REMAP` — `sha2`、`fs4`、`tempfile`、`priority-queue`、`vello_svg`、現行FFmpeg sidecar、既存 `RgbaDownloader`／`LatestWorker` | 第一原理から作る機構は1つも無い |
| **M5** | [採択地図](m5-known-implementation-adoption-map.md)が閉じ、private fixtureとreceiptが `DONE / KEEP` | 製品依存・接続が未了なだけ |

M3は2026-08-07に「UIを作る工程」から
**「先に作った資産を接続し製品として成立させる統合ゾーン」**へ読み直されている
（[M3統合ゾーン価値観更新](reviews/2026-08-07-m3-integration-zone-value-update.md)）。
**同じ読み方がM4とM5にも当てはまる。**

#### 発注への含意（これが本節の実利）

- **orderの形が「Xを実装せよ」なら疑う。** 正しい形は
  **「既にあるXを、既にあるYへ、既にあるoracleの下で繋げ」**である
- 繋ぎ先が実在しない場合、それは実装課題ではなく
  **探索の失敗か、記録と実態のdrift**である。先に測る
- したがって既定推定は `BUILT_UNWIRED`（作ってあるが繋がっていない）であり、
  `ABSENT`（本当に無い）は外部確認を経てから宣言する
- **「無い」と判断する前に、リポジトリ外も見る。**
  skiaのTimeline／Depth Rail fixtureはrepo外に実在し、repo検索では出なかった

`docs/README.md` の開発原則が同じことを定めている —
**既知実装優先、新設前に探索・採択。** 一般機構を第一原理から発明しない。

## 2. 触れる順序 = wave

[実行地図](m3-rn-runtime-execution-map.md)の利用者outcomeがそのまま操作列になる。

| wave | 触れるようになること | 状態 |
|---|---|---|
| R0 | projectを開いて、安全に同じ状態が見える | **DONE** |
| R1 | 図形を置く → Stage・Timeline・Inspectorが**同じものを指す** → Undoで消える | `READY-RECHECK` |
| R2 | Stageで掴んで動かす → その位置にkeyが打たれる → Curve／Easingで詰める。Timelineでseek・選択・move・trim・lane・snap | `OPEN / KNOWN GAPS` |
| R3 | mediaを入れる → 保存・再open → **再生** → **書き出し** | `OPEN / MIXED` |
| R4 | macOS／Windowsで人が普通に使い、**配布物として成立する** | `EXTERNAL_GATE_PENDING` |

## 3. M3完成時の一日

上の積み上げが具体的に何になるか。

1. Motoliiを開き、projectを作る
2. 楽曲と映像素材を入れる
3. Timelineに並べ、`move` / `trim` / `snap` で尺を合わせる
4. 図形・テキストを乗せる
5. Stageで**掴んで動かす**。動かした位置にPosition keyが打たれる
6. Curve／Easingで動きを詰める
7. **再生して音と合わせる**
8. **音声mux込みのmp4へ書き出す**

これで完成条件に到達する。

**重要**: この操作列は新規開発ではない。Rectangle Place、Undo／Redo、
Timeline move／trim、Position key追加・値編集、easing、playback spineは
**旧route（direct-wgpu/Vello + `ProductApp`）で既に受入済みの意味資産**である。
2026-08-07の再基線でそれらが**製品runtimeを失った**のが現在地であり、
R1〜R4は資産を新runtimeへ**繋ぎ直す**工程である。

**つまりMotoliiは「機能が足りない」のではなく「成立済みの意味が接続されていない」。**
これが完成像を明確に描ける理由であり、同時に
「既に動いているものを再実装させない」が最重要規律である理由でもある。

## 4. 触り心地

完成像には見た目も含む。以下はすべて既決である。

- **Ableton風の配色、初回既定はDark。** 派手さではなく密度で読ませる
- **逸脱時のみ表示。** 既定値は沈み、いじった所だけが立ち上がる。
  AEの常時全部見えている密度とは逆向き
- **Timelineは時間の操作へ集約する。** 行高は固定・最小（縦が情報を持たないため）、
  object barは読み取り専用（誤爆コストが非対称なため）、畳み＝射影、glyphは形で示す
  （[Timeline設計決定](reviews/2026-08-08-timeline-design-decisions-and-skia-fixtures.md)）
- **値・M/S・エフェクト・ブレンド・クリッピングはInspectorが受ける。**
  Timelineが送った責務の受け皿であり、載るかは未検証
- **Depth Rail**: z=0の既定群は灰色に統合され、**個別化されているものが逸脱として目立つ**
  （[Depth Rail決定](reviews/2026-08-08-depth-rail-selection-focus-decision.md)）
- **gizmoはdrag中write 0、release 1 Undo**
- **CJKフォント同梱は製品要求**（skia既定フォントがCJKを解決せず日本語が豆腐になる）
- Direct tools instead of setup rituals — 設定儀式ではなく直接の道具

### Alight Motionは独立した判定軸ではない（誤読を防ぐ）

docsに「AMを北極星とする」記述があるため、**実装担当がAMを調べに行く**誤読が起きうる。
利用者の整理（2026-08-09）:

> AM体験自体はAEに近い。いや、むしろAM自体の実装は無い。
> あるのはAEが回収できていなかった部分。
> AMの特性として区間イージングとエフェクトサムネイル程度で、それはもう回収できている。

**AMから採るべきものは `concept.md` へ畳み込み済みである（すべて2026-07-10前後の決定）。**

| デルタ | 正本 |
|---|---|
| プリコンポを作らず**グループレイヤー**（再帰可）+ 仮出力（ベイク）で置換 | `concept.md:153` / `:198` |
| グループはクリップと同じ項目エンベロープを持ち、**子を合成したフラット1枚**へエフェクトを適用 | `concept.md:199`。AMの意味論を明示採用。AviUtlのper-child適用は理由つきで不採用 |
| **クリッピングマスク**（AEのマスクパス方式ではなく下レイヤーへクリップ、モード選択式） | `concept.md:152` / `:197` |
| **区間イージング**（Flow/AM式。AE式グラフビューを作らずポップアップへ置換。Bounce/Elastic/Stepsも `Interp` variant） | `concept.md:178` |
| エフェクトサムネイル | `concept.md:174`。**契約の口は必須メタデータとして予約済み。ブラウザGUIはM3以降のフィニッシュ領域** |

したがって操作の既定は **「AEを含む既存ソフトの最適解 + 上表の凍結済みデルタ」** であり、
**AMを調査対象にしない。** UI判断で迷ったら `concept.md` と本書§4を読む。
そこに無ければ未決として返し、AMの挙動から推測しない。

## 5. M5と3Dについて（誤読を防ぐ）

**3Dは未解決の研究課題ではない。** 既知実装調査が閉じ、採択地図があり、
private fixtureとreceiptまで通っている（`M5-A1` / `M5-A2` / `M5-R0` / `M5-T0` /
`M5-P0` が `DONE / KEEP`）。

| class | 裁定 |
|---|---|
| glTF／OBJ import | `ADOPT`: `gltf` / `tobj` / `mikktspace` private leaf |
| camera数学 | `ADOPT`: `glam` private leaf（Document／serde／公開APIへ出さない） |
| spatial renderer | `REUSE`: **wgpu／現行RenderSession** |
| Duplicator seed | `ADOPT`: `rand_pcg` |
| text | `REUSE/WRAP`: 現行Fontique＋HarfRust＋Vello |

**境界を正確に**: Rerunは採択地図の**全行で `PATTERN`（参照）限定**である。
`re_renderer` の製品依存、Rerun store、Bevy ECS、renderling ownerの輸入は禁止、
`rend3` は `REJECT`。

> 「M5はほぼRerunの流用」は、**Rerunが解いた一般機構をMotoliiが再発明しない**
> という意味では正しく、**Rerunのコードが入る**という意味では正本が明確に否定している。

したがって3Dが解決済みなのは「委託先と裁定が確定していて、残りが薄い接続だから」である。
そしてM5は[休止契約](reviews/2026-08-02-m5-pause-until-m3-semantic-release.md)により
**M3の意味開放まで動かない**。M3が閉じないと接続先の製品routeが存在しないためである。

## 6. その先 — 上限が無い側

完成条件はここで閉じるが、北極星はその先にある。

- **Vism（`.vism`）** — 映像表現を特定project内の手順ではなく、
  時刻と型付きparameterから結果を返す**持ち運べる配布単位**にする
- **plugin契約が小さい** — 型付きparameterとGPU textureのin/outだけ。
  公開契約からplugin を足場作りでき、Hostが標準の編集UIを生成する
- **開発者は編集アプリ全体を作らず、ひとつの表現へ集中できる**

**M3が閉じるまでVismを渡す相手は存在しない。** 閉じた瞬間に外部の作者が入れる。
公開・共同制作フェーズの開始点はM3完成である。

## 7. 完成条件に含まないもの

北極星は**迷った時に境界を削るための判断基準**であり、完成条件を膨らませる口実ではない
（`concept.md`）。次はv1完成条件に**入れない**。

- 動的配布marketplace、第三者SDK、独自plugin UI、VST互換
- 解析駆動ジェネレーティブ合成（映像解析→DataTrack→パラメータ駆動）は**最終フェーズ**。
  DataTrack／ParamDriverの評価機構と口だけ凍結ゲートで予約する（2026-07-09決定）
- 物理への忠実度、全環境でのbit一致

## 8. 迷ったときの戻り先

- **「これは何のためか」** → §1〜§3
- **「この見た目でよいか」** → §4。無ければUI決定docsを読み、それでも無ければ未決として返す
- **「3Dはどうするのか」** → §5。**Rerunのコードを入れようとしていたら止まる**
- **「これは完成条件か」** → §7に載っていたら完成条件ではない

本書に書かれていないことは、本書が答えではない。現行正本と current code へ戻る。
