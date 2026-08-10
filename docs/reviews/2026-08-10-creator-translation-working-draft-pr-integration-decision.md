# クリエイター翻訳機構・叩き台PR統合決定

日付: 2026-08-10
状態: **決定**
判断者: 利用者(owner)

## 決定

Motoliiは、個々の描画・編集技術を第一原理から作り直す製品ではなく、**クリエイターの表現意図を、既知の実装が実行できる型付き意味へ翻訳し、Stage上の結果へ戻すHost**である。

M3〜M5の開発では、一発で最終正答を作ることを施工条件にしない。まず一つの制作意図について、操作、型付き意味、既知実装、画または自動oracleまでの理論を実際に通す**叩き台**を作り、製品の現在地と既知の限界を明示したままmainへ現像する。main上の実物を次の修正入力にする。

```text
creator intent
  -> Path / Filter / Instance / Time等の型付き意味
  -> Skia / Rerun / wgpu / FFmpeg等の採択済み実装
  -> Stageの知覚可能なfeedback
  -> M4の連続再生・Preview / Export共通評価
```

## 良い塊をPRで運ぶ

- Rerunのように、責任と内部整合性を既に持つsubsystemは、細片へ再発明せず**良い塊**として採択・移植する。
- 一つのPRは、一つの利用者成果と一つの意味ownerへ閉じる。その成果を通すためなら、Rust、React Native、shader、fixture、test、docsを横断してよい。file数や施工step数を契約境界と取り違えない。
- PRはdiff、根拠、既知の限界、事後観測をまとめる**landing envelope**である。承認待ちの官僚的gateや、一発で完成を証明する容器にはしない。直接mainへ入れる既存許可も維持する。
- probe、製品source、main統合、通常製品route、製品完成の状態は引き続き分ける。叩き台であることは状態の繰り上げを許さない。

## parallelとconflict

共有seatを先に完全分割してから並列化するのではなく、Stage、Timeline、Browser、Inspector、Vism表現等の**利用者に見える縦slice**を並列に進め、統合担当が順にmainへ着地させる。

- Gitの機械的conflict、module登録、同じ索引・台帳・app rootへの追記は、失敗ではなく通常のintegration workとして統合担当が解消する。
- stable identity、Document意味、D2 single writer、単一GPU owner、公開／永続contractの競合はsemantic conflictであり、当該統合を止めてownerを一つに戻す。
- 同じ共有seatを複数laneが独立実装しない。共有seat自体は直列、そこへ接続する製品sliceは並列とする。
- 寸分違わない事前分割図、独自queue、broker、監督framework、placeholder APIを、conflict回避のために新設しない。

## M3・M4・M5への適用

- **M3**は完成し続ける制作面であり、Stageを主役に、Timelineと各panelから制作意図を画へ現像する。
- **M5**はStageが受け取る表現能力を供給する。Rerun Spatial Viewerと、Path、Filter、Instance等への翻訳を既知実装から採択する。
- **M4**はM3とM5の間で同じ評価を時間上に連続させる。独立した主役UIを増やすためではない。
- Skia Timeline、Rerun Stage、Path morph、clipping filter、particle、glow、feedback、datamosh等のprobeは、個別機能一覧ではなく、この翻訳routeが成立するかを確かめる入力として扱う。

## 非目標

- 複数の意味ownerを持つ巨大PRを許可しない。
- test、review、実機gate、状態区分を省略またはgreenと偽装しない。これらはmain統合後も事実として観測する。
- probeのprivate表現を、検証なしに公開plugin契約やDocument schemaへ昇格させない。
- すべての変更へPRを義務化せず、PR approvalをmainのマージ条件へ戻さない。
- RerunやSkiaの製品意味をそのままMotoliiの製品意味として保存しない。Motoliiが所有するのは薄い翻訳、admission、作品意味、oracleである。

## 施工時の問い

新しいM3〜M5施工は、細かな作業列より先に次を答える。

1. 制作者は何を行い、Stageで何を知覚するか。
2. その意図をどの既存の型付き意味へ翻訳するか。
3. どの既知実装を良い塊として採択し、Motolii固有の薄い残余は何か。
4. 一つの意味ownerは誰か。機械的conflictを解消するintegration ownerは誰か。
5. 今回mainへ入る叩き台が通す理論、既知の限界、製品状態、次に観測できるedgeは何か。

この問いが閉じれば施工できる。将来の全panel、全Vism、全conflictを先に閉じることは開始条件にしない。
