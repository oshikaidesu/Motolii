# 依存優先・責任最小化ゲート（2026-07-24）

状態: **決定**

## 1. 決定

スクラッチ実装は、目の前の問題を解く最短経路になり得る。しかし、そのコードを所有した瞬間に、
OS差、権限、互換性、更新追従、障害解析、試験、配布、削除時の移行までMotoliiの責任になる。
したがって実装手段は、初期差分の小ささではなく、**目的達成までと、その後にMotoliiが持ち続ける
総責任が最小になる経路**で選ぶ。

Motoliiが直接所有するのは、作品の意味、製品policy、admission、acceptance oracle、絶対規律の
enforcement pointである。identity、Undo、layout、scheduler、codec等の機構classは解決済みの
OSS、標準、実装patternを供給源とし、Motolii固有の語彙や型名だけを理由に発明しない。必要な
first-party codeは、採択済み機構を既存contractへ写す薄いtranslation／admission adapter、製品policy、
fixtureに限定する。

これは依存を増やす決定ではない。依存もversion、供給網、license、build、型漏出、撤去という責任を
持ち込む。採用、薄いwrap、`PORT / PATTERN`、外部runner、棄却を同じ表で比較する。

## 2. 着手前の短い動線

新しい汎用機構、OS統合、基盤、test harnessを含む粒は、closed orderを作る前に次を順に通す。
既存の製品固有ロジックを数行直すだけの粒へ、広域調査を形式的に要求しない。

1. **AUTHORITY**: 対象spec、絶対規律、既決の責任所有者を確認する
2. **IN-REPO**: 現行code、helper、testkit、既存依存に同等経路がないか検索する
3. **DECISION**: `decision-index.md`と`references.md`で機構classの採択済みrouteを確認し、あれば継承する
4. **ECOSYSTEM**: 採択routeが無いか、後述の反証がある時だけ公式docs、公式repository、registryから候補を探し、結果を一度だけ正本と`decision-index.md`へ記録する
5. **RESPONSIBILITY**: 減る責任と増える責任を対で書く。人気やdownload数を実利用者数とみなさない
6. **DISPOSITION**: `REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL / REJECT`の一つを選ぶ
7. **EXIT**: Motolii fixtureを依存非依存に保ち、交換・削除時に触る境界を限定する

採択済みrouteがあるclosed orderは`ADOPTION ROUTE: <decision-indexの一意な行keyword>`だけを置く。
次の短票は機構classの初回裁定または具体的反証による再裁定時だけ一度作り、正本と台帳へ回収する。
粒ごとに短票や調査報告を作り直さない。

```text
RESPONSIBILITY DISPOSITION: REUSE | ADOPT | WRAP | PORT | PATTERN | EXTERNAL
EXISTING ROUTE: repo内経路、既決候補、または該当なし
OWNED RESIDUE: Motoliiに残す固有意味・性能境界
IMPORTED RESPONSIBILITY: version、license、build、権限、OS差、供給網
EXIT: adapter、fixture、交換時の限定範囲
RETIREMENT: 製品へ残す範囲、証拠確定後にfreeze/deleteする範囲
```

`PORT`／`PATTERN`は、解決済み機構をMotolii所有codeで実装・接続する処分であり、発明ではない。
新しい機構classの`BUILD`は通常処分に置かない。既知routeを列挙しても必須oracle、license、supported
platform、security、maintenanceのいずれかを満たせない具体的反証がある場合だけ、利用者例外へ返す
直前にFable 5へread-onlyで一回だけ先例の取りこぼしと既存routeへの再写像を照会する。Fableを含む
modelは新機構を認可・仕様化しない。採択済みcontractを正本へ記録することは発明禁止に含めない。

## 3. 製品外責任の予算

製品とは異なるacceptance harness、移行tool、調査fixture、failure injectorは、
長期保守する製品基盤ではなく**証拠カプセル**として扱う。存在を正当化できるのは、
Motolii固有の合否を既存経路へ載せる最小adapterに閉じる場合だけである。

証拠カプセルは次をすべて満たす。

1. product crate、通常runtime、公開API、Document、serde面、plugin契約へ型・依存・状態を追加しない
2. 一般機能はmaintained library、OSのsupported API / CLI、既存runnerを直接使い、
   manager、scheduler、process supervisor、capture library、retry frameworkへ一般化しない
3. `OWNED RESIDUE`の各項目はMotolii固有のoracle、epoch、fixture不変条件、manifest集約の
   いずれかであり、一般機能が1件でも残れば短票へ差し戻す
4. 外部process、window、fileを操作する場合は対象をread-onlyで完全解決し、不一致時は
   何も変更せず停止する。全対象、名前部分一致、未解決glob、private APIを使わない
5. 完了commit、対象OS/toolchain、再現command、raw evidenceを固定し、証拠確定後は
   `FROZEN / DELETE-LATER`とする。将来OSで壊れても通常製品の保守義務へ自動昇格させない
6. 後続製品codeは証拠カプセルをimportしない。同じ能力が製品に必要になった時は、
   その時点の公式routeと製品契約で改めて責任処分する

行数の少なさだけを予算にしない。重要なのは、長寿命のowner、公開面、状態正本、
background service、platform abstractionを増やさないことである。一般機能を薄いadapterへ
移しただけで複数粒から再利用し始めた場合は、証拠カプセルではなく新基盤なのでSTOPする。

後続粒は、正本と`decision-index.md`で一度裁定した機構classの採択routeを参照して継承する。
粒ごとのecosystem再調査・再裁定はしない。再裁定は、必須oracle、license、supported platform、security、
maintenanceの不適合、または薄いadapterが複数粒の共有基盤へ昇格する具体的反証が出た時だけ行う。
各粒では採択routeを成果へ接続できるかを次のいずれかで確定する。

- `PASS`: 既存routeとMotolii固有oracleだけで責任予算内
- `REDUCE`: 一般責任をlibrary / OS / external runnerへ戻してから再判定
- `STOP`: 責任予算を超えるため、手動審判、延期、粒の再分割、仕様縮小へ戻す

`PASS`以外では実装ループへ入らない。複数粒を一括で`PASS`にしない。

## 4. STOP条件

次のどれかに当たれば実装を開始せず、責任処分へ戻る。

- 一般問題なのにrepo内再利用と既存候補を確認せず、新しいframework、manager、runner、codec、
  scheduler、cache、layout、OS bridgeを作ろうとしている
- libraryの内部型、thread model、OS handleを公開API、Document、serde面へ出さないと採用できない
- 目先の実装行数は減るが、Motoliiの正本、single writer、純関数、VRAM、色変換を依存側へ移す
- download数、star数、採用企業だけで機能適合、保守継続、利用者数を証明したことにする
- 実証済みの既存実装を、障害・保守費・移植費の証拠なしにlibraryへ全面置換する
- test-only harnessを製品基盤へ昇格する、またはsynthetic試験を実IME／実機／人間審判へ読み替える
- 証拠確定後にも保守するplatform abstraction、process supervisor、capture/input frameworkを
  acceptance専用codeへ追加する
- `RETIREMENT`が空、または「後で判断」となっており、製品と証拠カプセルの寿命を分離できない
- 採択routeの適用範囲外まで前粒の`PASS`を広げる、または薄いadapterを共有基盤へ昇格するのに再裁定しない

## 5. 2026-07-24 Fable広域調査の処分

Fable 5へread-onlyで、workspace依存、CU粒度表、window/WebView、IME、Accessibility、layout、
desktop E2E、GPU/text、media、Document、cache、plugin sandbox、test toolingを横断調査させた。
候補の提案は採用根拠ではないため、Codexは次のように縮小して処分する。

| 処分 | 対象 | 現時点の意味 |
|---|---|---|
| `REUSE` | winit / wry、AccessKit、Taffy、egui_tiles、Vello、harfrust / fontique、Symphonia / cpal / rubato | 既存の採否と限定境界を維持する。別実装を重ねない |
| `EVALUATE` | rfd、arboard、muda、moka、loom、insta、cargo-deny、cargo-semver-checks、ffmpeg-sidecar | 対応する粒が実際に必要になった時だけ、小さいspikeまたは既存fixtureで比較する |
| `EXTERNAL` | Guidepup、XCUITest / Appium、Playwright | AX／native／DOM回帰の外側runner候補。実IME、VoiceOver追従、hardware審判を代替しない |
| `WRAP / DELETE-LATER` | OS固有の入力注入、spike限定文字経路、手書き依存監査、ffmpeg process管理 | 今すぐ書き直さない。製品E2E、3 OS移植、保守負担が同じ証明を持った時だけ縮退を比較する |
| `KEEP` | Document、D2 single writer、journal意味、CommandId / input意味、VRAM cache寿命、色変換、plugin純関数 | Motolii固有の正本。一般frameworkへ責任を移さない |
| `REJECT` | Tauri全面導入、別full UI framework、salsaによる設計反転、外部DB／WALへのjournal移管、`ui-events`二重導入 | 責任を減らさず、既決境界または正本を増やす |

主な未採択候補の一次資料は[参考ライブラリ一覧](../references.md#責任委譲候補2026-07-24未採択)へ
登録する。各候補のversion、maintenance、OS supportは採択時に再確認し、この調査日時の値を恒久契約へ
焼かない。

## 6. 既完了粒と残粒への適用

既完了であること、投入工数、差分量を維持理由にしない。独自機構の総責任が採択済みrouteより重いなら、
製品意味とoracleを保った置換を優先する。ただしbig-bang rewriteや旧新二重ownerにはせず、縦sliceごとに
新routeを同じ正例・負例へ通し、writer／consumerを一回だけ切り替え、旧routeを`FROZEN → RETIRE`する。
置換票には旧owner、採択route、parity oracle、切替点、削除条件を閉じ、移行途中を恒久互換層にしない。
fixtureが製品意味を持たず削除可能に閉じているなら、存在自体を失敗扱いしない。

未着手粒は、依存関係を保ったまま着手時に短票を追加する。短票の結果、既存libraryや外部runnerで
完了条件を満たせるなら、粒IDを消さず実装量を減らす。粒は成果と審判の単位であり、自作code量の
割当ではない。

`CU-0G04`のlifecycle harnessはcommit `021a16e7`と固定Mac構成へ閉じた
`WRAP / FROZEN / DELETE-LATER`証拠カプセルとする。wry callback、OS CLI、
custom protocolを製品契約へ昇格せず、PID探索、capture、failure進行を後続粒から
再利用しない。`CU-0G05L`でraw evidenceを限定確定した後は、同じcommitと構成の
再現以外にforward-maintenanceを約束しない。OS/toolchain変更で再実行不能になった場合は、
このadapterを通常保守せず、新しい公式routeを責任処分する。

## 7. 非目標

- この決定だけで新しいcrateやtoolをCargo workspaceへ追加しない
- Fable候補を一括採用しない
- Motolii固有の意味を「既存libraryがある」だけで外部化しない
- 既完了fixtureを証拠なく削除、置換、製品昇格しない
- license許可だけで供給網、保守、platform適合まで合格としない
