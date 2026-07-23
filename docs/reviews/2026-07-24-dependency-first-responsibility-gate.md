# 依存優先・責任最小化ゲート（2026-07-24）

状態: **決定**

## 1. 決定

スクラッチ実装は、目の前の問題を解く最短経路になり得る。しかし、そのコードを所有した瞬間に、
OS差、権限、互換性、更新追従、障害解析、試験、配布、削除時の移行までMotoliiの責任になる。
したがって実装手段は、初期差分の小ささではなく、**目的達成までと、その後にMotoliiが持ち続ける
総責任が最小になる経路**で選ぶ。

Motoliiが直接所有するのは、作品の意味、純関数評価、Document、single writer、Undo、正準座標、
Preview / Export共通経路、VRAMと色変換の境界など、製品固有の意味と性能保証である。file dialog、
clipboard、AccessibilityのOS bridge、一般的なlayout数学、入力注入、外部E2E runner、依存監査等の
一般問題は、既存実装へ委ねられない理由を確認してから自作する。

これは依存を増やす決定ではない。依存もversion、供給網、license、build、型漏出、撤去という責任を
持ち込む。採用、薄いwrap、外部runner、自作、棄却を同じ表で比較する。

## 2. 着手前の短い動線

新しい汎用機構、OS統合、基盤、test harnessを含む粒は、closed orderを作る前に次を順に通す。
既存の製品固有ロジックを数行直すだけの粒へ、広域調査を形式的に要求しない。

1. **AUTHORITY**: 対象spec、絶対規律、既決の責任所有者を確認する
2. **IN-REPO**: 現行code、helper、testkit、既存依存に同等経路がないか検索する
3. **DECISION**: `decision-index.md`と`references.md`で採否済み候補を確認する
4. **ECOSYSTEM**: 一般問題が残る場合だけ、公式docs、公式repository、registryから候補を探す
5. **RESPONSIBILITY**: 減る責任と増える責任を対で書く。人気やdownload数を実利用者数とみなさない
6. **DISPOSITION**: `REUSE / ADOPT / WRAP / EXTERNAL / BUILD / REJECT`の一つを選ぶ
7. **EXIT**: Motolii fixtureを依存非依存に保ち、交換・削除時に触る境界を限定する

closed orderには次の短票だけを置く。調査報告を毎回作り直さない。

```text
RESPONSIBILITY DISPOSITION: REUSE | ADOPT | WRAP | EXTERNAL | BUILD
EXISTING ROUTE: repo内経路、既決候補、または該当なし
OWNED RESIDUE: Motoliiに残す固有意味・性能境界
IMPORTED RESPONSIBILITY: version、license、build、権限、OS差、供給網
EXIT: adapter、fixture、交換時の限定範囲
```

`BUILD`は悪い判定ではない。ただし、既存候補が契約を満たさない事実、Motolii固有として残る部分、
公開型やDocumentへOS／vendor型を漏らさない境界を示す。調査のための大きな抽象化を先に作らない。

## 3. STOP条件

次のどれかに当たれば実装を開始せず、責任処分へ戻る。

- 一般問題なのにrepo内再利用と既存候補を確認せず、新しいframework、manager、runner、codec、
  scheduler、cache、layout、OS bridgeを作ろうとしている
- libraryの内部型、thread model、OS handleを公開API、Document、serde面へ出さないと採用できない
- 目先の実装行数は減るが、Motoliiの正本、single writer、純関数、VRAM、色変換を依存側へ移す
- download数、star数、採用企業だけで機能適合、保守継続、利用者数を証明したことにする
- 実証済みの既存実装を、障害・保守費・移植費の証拠なしにlibraryへ全面置換する
- test-only harnessを製品基盤へ昇格する、またはsynthetic試験を実IME／実機／人間審判へ読み替える

## 4. 2026-07-24 Fable広域調査の処分

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

## 5. 既完了粒と残粒への適用

既完了粒を一括で書き直さない。変更量、OS固有code、独自framework、保守事故の順に
`KEEP / WRAP / TEST-ONLY / DELETE-LATER / REDUNDANT`を付け、後続製品経路が同じ証明を持つ時だけ
縮退する。fixtureが製品意味を持たず、削除可能に閉じているなら、存在自体を失敗扱いしない。

未着手粒は、依存関係を保ったまま着手時に短票を追加する。短票の結果、既存libraryや外部runnerで
完了条件を満たせるなら、粒IDを消さず実装量を減らす。粒は成果と審判の単位であり、自作code量の
割当ではない。

## 6. 非目標

- この決定だけで新しいcrateやtoolをCargo workspaceへ追加しない
- Fable候補を一括採用しない
- Motolii固有の意味を「既存libraryがある」だけで外部化しない
- 既完了fixtureを証拠なく削除、置換、製品昇格しない
- license許可だけで供給網、保守、platform適合まで合格としない
