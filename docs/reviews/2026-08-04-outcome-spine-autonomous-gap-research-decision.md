# 利用者成果の背骨と調査不足粒の自律再接続決定

状態: **決定**

日付: 2026-08-04

## 1. 決めること

Motoliiの自律実装は、着手可能な粒を順に消化することを大目標にしない。先に、通常製品routeで
利用者が一続きに完走する**利用者成果の背骨**を固定し、その背骨を構成する一契約境界だけを粒として
実装する。

粒の調査が不足していても、未調査を即時の全体停止や独自実装の理由にしない。主担当Codexは
現行authorityとcodeから不足箇所を特定し、既知実装を再検索して`REUSE / REMAP / REDUCE / 再調査`の
いずれかで背骨へ戻す。実在targetを閉じられない粒だけを`WAIT_TARGET`へ残し、依存しない背骨上の
別edgeを継続する。

## 2. 二層の計画単位

| 層 | 所有するもの | 所有しないもの |
|---|---|---|
| 利用者成果の背骨 | 通常製品routeの操作列、同一identity、失敗回復、Undo/Redo、reopen、Preview/Export等の出口 | 一枚の巨大実装order、共有framework、全将来API |
| 実装粒 | 一つのowner、意味、write route、閉じた変更file、primary oracle | 背骨全体の完了宣言、隣接owner、未決の将来挙動 |

背骨は粒を大きくする許可ではない。複数の粒がどの利用者成果へ収束するかを固定し、局所最適な
task消化や遠い枝への脱線を防ぐための進捗軸である。実装、review、commitは引き続き
**一契約境界 / 1 ticket = 1 commit**で行う。

## 3. 背骨の固定形式

新規挙動または複数粒にまたがる制作体験は、実装前に次を短く固定する。

```text
USER OUTCOME: <通常製品routeで利用者が完走する結果>
OUTCOME SPINE: <操作 -> domain write -> projection -> feedback -> recovery>
STABLE IDENTITY: <全edgeで同一と判定するidentity>
SUCCESS EXIT: <E2Eで観測する完成状態>
FAILURE / CANCEL: <変更ゼロ、回復、typed failure>
AUTOMATED ORACLES: <grain oracleとspine E2E>
EXTERNAL GATES: <platform / hardware / provider / humanを分離>
NON-GOALS: <この背骨へ混ぜない意味>
```

task ID、crate、既存UI名を並べただけの列は背骨ではない。利用者操作から結果と回復までが連続し、
各edgeがどの既存ownerへ接続するかを辿れることを必要とする。

## 4. 調査不足粒の自律loop

### 4.1 粒の現行事実を閉じる

各粒を次の一行へ写す。

```text
AUTHORITY -> INTERNAL TARGET -> OWNER -> WRITE ROUTE -> GAP -> RESOLUTION ROUTE -> DISPOSITION
```

`GAP`は「まだ詳しく調べていない」「画面名が違う」ではなく、現行型、source、command、consumer、
layout slot、test、契約の不在または矛盾で示す。古い`next`文、task ID、過去branch、fixture greenを
現行targetの代わりにしない。

### 4.2 再検索の順序

不足を検出した主担当は、次の順で検索する。

1. 現行repoの同等owner、type、command、projection、fixture
2. `decision-index.md`と対象正本で採択済みのroute
3. `references.md`と製品先例
4. 必要な場合だけ一次資料

検索済みと報告するには、検索場所、候補、採否、不適合理由が必要である。単一keywordの0 hit、
古い文書の不在、外部LLMの「無い」という回答だけで検索完了にしない。

### 4.3 処分の優先順

1. **REUSE**: 実在するownerと意味が一致するならそのまま接続する
2. **REMAP**: target名や想定層が違うなら、同じ成果を所有する実在ownerへ写し直す
3. **REDUCE**: 広い親のうち、意味・owner・oracleが閉じた部分だけを背骨を保って先行する
4. **再調査**: 候補はあるがlicense、platform、failure、thread model等の比較が不足する場合だけ追加調査する
5. **WAIT_TARGET**: identity、command、consumer、owner、公開契約をなお閉じられない当該edgeだけを待機させる

`WAIT_TARGET`は親task全体や別laneを止める信号ではない。依存しないedgeが同じ背骨上にあれば継続し、
別edgeの成果から不足targetが実在した時に再入場する。利用者判断なしに意味を選ぶ必要がある場合だけ返す。

### 4.4 実装開始条件

[既知実装採択・置換開発モデル](../known-implementation-adoption-model.md)のpreflightを満たし、
`BUILD JUSTIFICATION: NONE / BUILD: FORBIDDEN`のまま薄い接続へ閉じる場合だけ実装する。
調査不足を一般helper、汎用gesture framework、第二state owner、仮UI、独自codecで埋めない。

実装とprimary oracleが閉じた後、主担当は背骨の未完edgeを現行codeから再計測する。古い粒数や
完了前の`next`を残量にせず、次の一契約境界を選び直す。

## 5. M3への適用

M3では[縦slice実行方針](2026-07-24-m3-vertical-slice-execution-decision.md)の縦sliceを
利用者成果の背骨として扱う。たとえばMotion Authoring Loopは、次の利用者成果を背骨に持つ。

```text
配置 -> Position選択 -> key追加 -> 別時刻の値変更 -> 区間選択
     -> Easing適用 -> Preview -> Undo/Redo -> save/reopen
```

これは`U4b-0`、active interval、outgoing Interp、Easing Host、Transportを一粒へ束ねる意味ではない。
各edgeを既存D2、single writer、Timeline/Inspector projection、native popup、Preview評価へ一つずつ接続する。

ユーザーによる目視確認は粒ごとに要求せず、技術実装と自動oracleを先に進め、**M3全体の最後のHUMAN
checklist**へ集約する。named platform、hardware、provider gateはHUMANと別に記録し、未観測をPASSへ
繰り上げない。人間目視の後回しは、粒の実装、machine observation、非LLM oracleを省略する意味ではない。

## 6. 完了報告

進捗は次を分離して報告する。

- 背骨上で新たに接続されたedge
- 実diffとowner / write route
- grain primary oracleとspine E2Eの現在地
- `REMAP / REDUCE / WAIT_TARGET`へ処分した不足
- external gateと最終HUMAN残件
- local main統合と次の一契約境界

一つの粒がgreenでも背骨を完成扱いせず、背骨の一部がblockedでも接続可能な別edgeを停止しない。

## 7. 既知実装採択サマリ

- MECHANISM CLASS: outcome-driven autonomous implementation control
- KNOWN IMPLEMENTATION SEARCH: 既存M3縦slice、既知実装採択モデル、既存契約接続票、Human Response Frontier
- CANDIDATES: 現行の縦slice二層分離、`REUSE / REMAP / REDUCE`局所STOP、rolling horizon
- ADOPTION ROUTE: `REUSE / REMAP`
- REJECTED CANDIDATES: 新runner、新state machine、全task共通pipeline、巨大slice order
- THIN MOTOLII SEAM: 背骨形式、調査不足loop、M3 HUMAN集約規則
- THIN MOTOLII RESIDUAL: authority照合、粒選定、oracle、最終採否
- RETIREMENT: 粒完了数と古い`next`を進捗軸として使う運用を停止
- BUILD JUSTIFICATION: NONE
- BUILD: FORBIDDEN

## 8. 非目標

- 全taskへ新しい必須schemaやreceiptを導入すること
- STOP、WAIT、未決契約を無視して走り続けること
- 背骨を一つの巨大PR、長期model session、固定外部LLM列へ変えること
- 未実装UIやownerを既知製品の見た目から推測すること
- external gateまたはHUMANを自動testで代替すること
