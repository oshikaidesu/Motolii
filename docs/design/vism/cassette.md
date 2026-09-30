# Vism Cassette — 新しい mapping 系は要らなかった

作成日: 2026-09-26

状態: **調査結果の固定／schema・container・runtime・製品UI未決**。Cassette を実装する文書ではない。probe で分かったこと、Motolii が既に持つもの、足りないもの、意図して先送りするものを、参加していない人が読めるように残す。

関連: [Vism / Kitモデル](kit-model.md)、[Vismコンセプト](package-concept.md)、[スクリプトの口](../rationale/script-mouth.md)、[UI rebaseline brief](../../product/ui-rebaseline/brief.md)、Draft PR #533「docs: draft Vism cassette distribution and Host capability API」(branch `draft/vism-cassette-ts-host-api`。この repo の main には無い)。

## 1. 結論

```text
Cassette instance
├ 通常の Motolii composition          (層・効果・キー。既存)
├ 公開パラメータの宣言                (小さく足りない意味)
└ PropertyLink                        (既存。新しい mapping 系は要らない)
```

- **Vism Cassette に新しい retained mapping 機構は要らない。** Motolii は `PropertySource` / `PropertyLink` として既に持っている。
- **JavaScript / TypeScript は再生中に生きている必要がない。** JS/TS は構造を作る(または取り込む)時だけ走る。Document の正本は通常の Motolii の意味であり、再生・保存・Undo・書き出しは Host だけで成り立つ。
- Cassette は「Host を再実装する物」ではなく「Host の意味を組み合わせ、小さな操作面を付けた物」である。

**Compose Host semantics. Do not reimplement the Host.**

この結論は「作ろうとしたら既に Host にあった」という経緯で得た。足したのではなく削った。半年後に「Cassette 用の mapping / expression / runtime を足そう」となったら、まずこの節に戻ること。

## 2. どう辿り着いたか

1. **Inspector rebaseline**: 未知の値には Toy、既知の概念には意味を持つ振る舞い、既知の関係には Instrument。「gesture は共有し、関係は専用にする」。
2. **Camera 調査**: 現行 Camera は Aim + Orbit rig で、撮影技法(handheld、orbit、dolly)を Host の意味へ昇格する必要はなかった。**創造的な技法は自動的に Host semantic に値しない。**
3. **既存 Script API で作れた**: Camera 技法は既存 row とキーだけの one-shot generator になった(keys 37 本/row、同じ seed で同じ shot、保存・読込・手編集・失敗時 rollback を確認)。Shatter Repeater 風の構成も既存の Repeater だけで組めた。
4. **Shatter probe が本当の問題を出した**: Script が焼いた値は、上位の公開パラメータ(Amount、Spread)という identity を失う。Pieces = Repeater の Count のように既存 property をそのまま出す物は、焼いても live のまま残る。
5. **仮説**: 「1つの公開パラメータが複数の既存 property を駆動し、後から編集できる」層が要る。
6. **read-only audit**: その層は既にある。`PropertySource` の modulators として。
7. **2 つの probe で確認**: Shatter Repeater と Title Treatment(§5、§6)。

## 3. 既存の Host 機構(コードから)

正本は `crates/motolii-doc/src/store/slot.rs`、評価は `store/view.rs` の `value_at_path_resolving_links`。

- **`PropertySource`** = `base`(Track / Slot / Constant のいずれか、または無し) + `modulators: Vec<PropertyLink>`。
- **`PropertyLink`** = 読み元の layer と property、`time_offset`、translator の `plugin_id`、`params`。
- **評価**: 値 = base に、各 link を translator で変換した値を足した物。base が無ければ寄与の和。型が合わない、または未知の translator は近似せず寄与ゼロ。
- **translator は 3 つ**:
  - `motolii.link.identity`
  - `motolii.link.linear`(scale、offset。数と 2 成分の点に効く。2 成分には同じ scale/offset が両方に掛かる)
  - `motolii.link.remap`(in_min、in_max、out_min、out_max、clamp。数だけ。in の幅 0 は寄与ゼロ)
- **link の鎖**: 読み元も PropertySource なので鎖になる。書き込み時に循環を拒否し(`validate_no_link_cycle`)、評価側にも深さの上限がある。
- **番地**: 読み元も書き先も `(LayerId, PropertyId)`。effect の param(`effect.0.param.count`)、text の style 行(`text_style.0.size`)、layout 行(`layout.stagger`)、通常の layer 行のどれも同じ形で指せる。
- **Intent**: `SetPropertyLink`、`SetPropertyModulators`(base は保つ)、`SetCameraPropertyModulators`。
- **Copy / Paste / Split**: コピー集合の内側にある読み元は複製側へ付け替える。外側なら元を指したまま。
- **窓の側の guard**: 駆動されている行への手編集は「Edit the driver before changing a driven value」で断る(`motolii/crates/motolii-edit/src/document/edit.rs`)。
- **無い物**: 現行の窓と Script API に link を**作る**口が見当たらない(Intent としては在る)。式、2 つ以上の source を掛ける translator、スカラーから 2 成分への配り、曲線は無い。

## 4. 公開パラメータは既に property として置ける

制御用の層(Null)に、名前を宣言していない property を置くと、store は受け付け、Inspector はそれを行として並べる(probe で Punch、Looseness、Delay を確認)。つまり「値を置く場所」と「その値を Inspector から動かす経路」は既にある。足りないのは意味の宣言である(§8)。

## 5. Shatter Repeater probe

```text
Pieces  ─────────────→ Repeater Count
Seed    ─────────────→ Repeater Seed
Amount  ──remap──────→ Rotation Random   (0..1 → 0..180)
        └─linear─────→ Scale Random      (× -0.6)
Spread  ─────────────→ Position Random   (2 成分どうし)
```

- **Pieces を後で変える**と、実際の Repeater の複製数が変わる(16 → 5 を確認)。
- **Amount を 1 回キーする**だけで、以前は 5 本の焼いたトラックだった動きが 1 本の公開コントロールになる。
- 結果は通常の Motolii composition のまま。隠れた helper 層は無い。

限界(今回は解かない):

- **スカラー → 2 成分は配られない。** 読み出した値がスカラーのままで、2 成分の行として不正になる。Spread は 2 成分の値として公開した。
- **Spread × Amount のような積は作れない。** link は足すだけで、2 つの source を掛け合わせる translator が無い。
- **本物の破片にならない。** Repeater は元の絵を丸ごと複製するので、「1 つの絵 → 独立に指せる破片」という Host 能力が別に要る。これは **Piece Selection / Fragmentation** という別の Host 能力候補で、Cassette で偽装しない。

## 6. 2 つ目の probe: Title Treatment

```text
Punch      ──remap───→ Size        (48..160)
Looseness  ──linear──→ Tracking    (× 40)
Delay      ──────────→ Stagger
```

Repeater を使わず、text と layout の通常の番地だけで同じ機構が成り立った。Punch を後から変えると Size が追従する。**retained parameter mapping は Shatter 専用ではなく Host の一般的な原始である**ことの証拠。

## 7. 生きている間の振る舞い(試した範囲)

| 観点 | 結果 |
|---|---|
| 保存 / 読込 | mapping と値が保たれる |
| Undo | 1 回の transaction で組めば、Undo も 1 回で公開パラメータと mapping が一緒に戻る |
| 制御層 + 駆動層を一緒に複製 | 複製側の link は複製した制御層を指す |
| 駆動層だけ複製 | 元の制御層を指したまま |
| Detach | 評価値を通常の定数へ焼き、link を外し、制御層を消しても、Repeater・row・複製数は残る。Cassette 固有の実行状態は要らなかった |

**Nested Cassette は試していない**。link の鎖と循環拒否から成り立つはずだ、という推測にとどまる。

**Detach の試験は 1 時刻の値を定数へ焼いただけ**である。駆動側 source にキーがある場合は、トラックとして焼く手順が別に要る。

## 8. 所有と直接編集

2 つの層を分けて読む。

- **窓**: 駆動されている行は手編集を断り、driver を直すよう言う。
- **store**: 通常の property 書き込みは `PropertySource` ごと置き換えるので、link が消える。`SetPropertyModulators` だけが base を保ったまま modulators を置き換える。

したがって「base をずらして局所調整を足す」は、現行コードは持っていない。この文書は**窓の現行挙動をそのまま保守的な Cassette の挙動とする**。局所調整を足すかどうかは新しい製品判断であり、ここでは決めない。

## 9. 足りない意味(公開パラメータの宣言)

表現されていない物:

- Cassette instance の identity(どの層が 1 つの Cassette か)
- 公開パラメータの宣言: label、意味の種類、範囲、既定値
- 駆動されている行の提示情報(row の JSON に driven の印も driver も出ない)

保存表現は決めない。観察として、free-form の component 名(`Layer:<name>`)が往復する事実があるので、宣言の運び手の**慣習の候補**にはなり得る。これは決定した schema ではない。

**型**: 文書の値は number と 2 成分のままでよく、Cassette のために count、seed、angle、toggle、choice といった保存型を足す必要はない。意味の宣言を Inspector が Toy に解釈する。Authors declare meaning; the Host supplies the interaction.

## 10. 作者の入口は 1 つの構造に収束する

```text
GUI の Make Cassette ─┐
TypeScript SDK ───────┼→ 同じ構造: 通常の層・効果、定数・キー、公開パラメータの宣言、PropertyLink
AI 生成 ──────────────┘
```

再生時の closure は作らない。Vism の意味は JavaScript の上にある。Draft PR #533 の「JS bundle が最初の実行形」は、**構築時・取り込み時に走る**と読み替える。

## 11. Unpack の原則

Cassette は可能な限り通常の Motolii の意味へ開けるまま保つ。Detach は外国の runtime からの変換ではなく、すでに通常の composition である物の周りの、小さな retained 構造(制御層、link、宣言)を焼く/外す操作になる。

## 12. Kit との関係(不整合の記録)

この repo の語は **Kit** である。`vism-kit-model.md` は Kit を「接続済みの用途セット、初期値と公開 control を持つ Rack 型の作者成果」とし、v1 は Project へ **1 回 materialize** して、以後は Kit runtime が無くても通常の Project の意味が残る、と決めている。「Cassette」はこの repo の Vism docs に無い語で、PR #533 と今回の調査が使った作業名である。

- probe の結果は Kit の不変条件と衝突しない。材料は通常の Object・Effect・接続で、runtime も無い。
- 追加で分かったのは、**公開 control は materialize 後も、制御層の property と PropertyLink として通常の Project に残せる**こと。Kit の文書は展開後の公開 control の行き先を書いていない。
- 「Kit の更新で既存 Project を自動変更しない」は、この形と両立する。
- **Cassette と Kit を同じ物として統合するか、別名を残すかは未決**。ここでは統合も新設もしない。

## 13. Draft PR #533 との関係

PR #533 の draft(`docs/reviews/2026-09-26-vism-cassette-ts-host-api-draft.md`)は、この調査の前の仮説である。履歴は書き換えず、次を読み替える。

- 「small live package」「retained cassette semantics が要る」: 保つべきは**構造の retained 性**(PropertyLink と公開パラメータの宣言)で、JS の live 実行ではない。
- 「main.js が最初の実行形」: 構築時にだけ走る。再生時に常駐させる根拠は、今回の probe では出なかった。
- 「Piece Selection を含む Shatter」: 別の Host 能力で、Cassette の課題ではない。

## 14. 意図して確立しない物

plugin system、reactive な JavaScript runtime、property ごとの式、node graph、最終的な `.vism` package 形式、公開 SDK、Piece Selection の実装、Camera の再設計、製品の Cassette UI、最終的な Cassette instance の schema。

## 15. 残る判断

**最初の Cassette UI の前に要る**

1. 公開パラメータの宣言をどこに持つか(§9)。
2. Cassette instance の identity をどう表すか。
3. group 層が instance の自然な境界になるか。
4. 駆動されている行を Inspector でどう見せるか。
5. 窓に link を作る口が要るか(現状は Intent だけ)。

**具体的な構成が要求するまで先送りできる**

6. スカラー → 2 成分の配り。
7. 積 / 2 source の translator。
8. Nested Cassette の意味(試験も含む)。
9. Kit と Cassette の語の統合(§12)。

## 16. UI rebaseline との接続

```text
Toy         値を編集する
Instrument  既知の Host の関係を編集する
Cassette    通常の Host の意味の組み合わせの上に、小さな操作面を出す
```

これは 3 段の plugin ではなく、Host が意味の構造を増やしながら見せる 3 つの姿である。調査はここで十分なので、Inspector / UI rebaseline へ戻る。

## 証拠

- 試験と probe: `motolii/ui/native/src/editor/script/probes.rs`、`motolii/crates/motolii-script/probes/`(`camera.js`、`shatter.js`)。
- ローカル commit(push していない): `51bbad184`(Camera の共通 math と evaluator の差の固定)、`0bbd2c43a`(Script probe と native build の修理)、`4eb1dd014`(retained mapping の probe)。
- 別件として記録: Script の時間 budget は host の時間も数える。dev build の shader hot-reload watcher が最初の mesh 生成を 70〜90 秒止める(Script API の問題ではない)。
