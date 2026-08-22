# ペルソナ調査 第2周: TouchDesigner 上がり — 空の戸に何を付けるか

日付: 2026-08-22 / 状態: **調査・設計提案**(read-only・コード変更なし・cargo 不使用・第1手 `git merge main` 実施 = fast-forward、差分ゼロで up to date) / 対象リポ: `next/`(2026-08-20 リセット後の正本)。前提は
[第1周](2026-08-22-persona-touchdesigner.md)。

---

## 0. 第1周の結論と、第2周の宿題

第1周は「最初に見限られるのは A2(パラメータリンク)」「vism(拡張圏)は空の戸(`pub trait` ゼロ件)」の2点を実測で固定した。本書は**その2つの穴に対する具体的な設計案**を出す——ただし発注書の制約どおり「作らない」。両方とも「型・保存・undo・決定論」または「trait 擬似コード・2人目問題」まで詰めた**提案**であり、`next/DECISIONS.md` への採番は supervisor に委ねる。

---

## 1. 型付き link の設計案

### 1.1 先例を一次資料で当たる

| 製品 | 機構 | 一次資料 | Motolii へ写せる形 |
|---|---|---|---|
| TouchDesigner | **CHOP Export** — 数値の時系列を持つ CHOP チャンネルを、任意 operator の任意パラメータへ**名前で片方向束縛**する。`Export Flag` で on/off、生成側は消費先を知らない | [Export](https://derivative.ca/UserGuide/Export)・[CHOP Export](https://docs.derivative.ca/CHOP_Export) | 「ソースは自分の消費先を知らない、宛先だけが参照を持つ」という**非対称な片方向束縛**の形はそのまま使える。CHOP自体は式処理系を持たない値の器 |
| Blender | **Driver** — 被駆動 property が `DriverVariable`(Single Property / Transform Channel / Rotational Difference / Distance)を経由し、`Averaged Value`/`Sum`/`Min`/`Max`/`Scripted Expression` の**関数**で合成する。**循環は実行時検出**——依存グラフが「ループを検出すると警告を出し、ランダムな点で切って」続行する(runtime 検出・事後警告、grep実測: Blender Projects issue #64793 ほか) | [Drivers Panel](https://docs.blender.org/manual/en/latest/animation/drivers/drivers_panel.html)・[DriverVariable API](https://docs.blender.org/api/current/bpy.types.DriverVariable.html) | 「被駆動 property が参照側」という主従の向き、「変換は閉じた関数の集合(Scripted Expression 以外)」という区分は写せる。**循環検出の実行時・事後警告という方式は写さない**——Motolii は書き込み時に拒否する既存の `validate_no_parent_cycle`(§1.5)の方が強い保証で、これは Blender の弱点(#64793 のジャンプ・チラつき)の裏返し |
| After Effects | **pick whip → `effect("Slider")("Slider")`**(文字列式、名前で他 property を参照)。`wiggle(frequency, amount)` は式だが**tと seed だけから閉じた式**で書ける([expression basics](https://helpx.adobe.com/after-effects/using/expression-basics.html)) | pick whip の「参照先を名前で探す」体験は保つが、文字列式そのものは Lottie 地図が不採用済み(`lottie-coverage.tsv:325`) | **wiggle は Link の問題ではない**——`docs/plugin-authoring.md` §4.5 のレベル0(`t`の閉形式純関数、バネ/バウンス/ウィグルを名指しで例示、264-277行)が既に解を持っている。Link が解くべきは「A の値を読んで B へ渡す」部分だけ |

**結論**: Link が本当に必要とするのは「他 property の値を読む」経路だけであり、乱数・ノイズ生成(wiggle)は別の(既に解のある)問題。スコープをここで絞る。

### 1.2 v1 のスコープ(閉じた集合)

- **単一ソース→単一ターゲット、片方向**(TD CHOP export と同型)。A の回転で B の不透明度を動かす(発注書 A2)、AE の Follow/LookAt 相当のうち **Follow**(位置を写す)はこの形に収まる。
- **LookAt(2点間の角度、`atan2` — 2入力)は v1 の外**——`next/GOALS.md` の「標準」節が既に「親子(型付き Follow/LookAt)」(38行目)を Link とは別に名指ししている。LookAt は「2つの値から1つを作る」形で、本設計の「1対1・閉じた写像」を超える。**v1 で解くのは Follow 型(1対1)のみ**とし、LookAt は Link の型が伸びる先(将来 `sources: Vec<...>`)として位置だけ示す。
- **決定論的な写像だけを許す集合**(裁定不明の JS を持たない):`identity`(そのまま渡す)/ `linear`(scale・offset の1次式、F64・Vec2)/ `remap`(区間→区間、clamp可否)/ `time_offset`(評価時刻をずらす。経路C「Temporal Window」の最小形と共有)。

### 1.3 型:既存の `PropertySource` を第三の枝へ拡張する

**発明ではなく、既に稼働している機構の横展開**。`next/core/motolii-store/src/slot.rs` の `PropertySource` は既に **untagged 2択**(`Track`/`Slot`)で、「同じ場所に置く・第二の差し替え機構を作らない」という規律のもとで作られている(module doc 1-22行、および `next/reference/lottie-coverage.tsv:324` の採用理由「`PropertySource::{Track,Slot}` の2択」)。この2択を3択にするだけで新しい仕組みを増やさずに済む。

```rust
// next/core/motolii-store/src/slot.rs へ追記するイメージ(擬似コード)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertySource {
    Track(motolii_eval::KeyframeTrack),
    Slot(SlotId),
    Link(PropertyLink),              // ← 追加する第三の枝
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyLink {
    pub source_layer: LayerId,
    pub source_property: PropertyId,
    /// 評価時刻をずらす量。**Host が構造として知る必要がある**(scrub・cache key・
    /// 依存解決に効くため、`effect.plugin_id` のような不透明文字列に隠さない —
    /// 経路C「Temporal Window」が要求する「必要なoffsetの静的宣言」と同じ理由)。
    pub time_offset: RationalTime,
    /// 値変換の閉じた集合を指す id。**EffectInstance.plugin_id と同じ扱い**
    /// (裁定70: Document は具体的な変換の中身を型で持たない)。
    pub plugin_id: String,           // "motolii.link.identity" / "motolii.link.linear" / "motolii.link.remap"
    /// 変換の係数。**全て animatable**(裁定72 と同じ扱い — 新機構ゼロ)。
    pub params: Vec<(String, motolii_eval::Value)>,
}
```

**なぜ `plugin_id` + `params` で「動く仕組み」自体を型にしないのか**——これが裁定191 との緊張を解く核心(§1.9 で詳述)。`EffectInstance`(`next/core/motolii-store/src/effect.rs:28-52`)が既に同じ形をしていて、`next/reference/lottie-coverage.tsv` の `effect-values/*/ty` 系5行(93,96,99,102,111行)は揃って「**不採用**: Document は effect を型で持たない(裁定70)」と書きながら、同じ行の `v`(Value 本体)は**採用済**にしている。つまり地図自身が「具体的な変換の中身(ty)」と「変換の入れ物・係数(v)」を明確に別扱いにする先例を既に持っている。Link の `plugin_id`(=ty 相当)と `params`(=v 相当)はこの先例をそのまま踏襲する。

### 1.4 評価:`value_at_path` への挿入(既存関数の延長)

```rust
// next/core/motolii-store/src/view.rs の value_at_path (337-357行) を拡張するイメージ
match self.source_at_path(path, property)? {
    Some(PropertySource::Track(track)) => Ok(Some(track.eval(t))),
    Some(PropertySource::Slot(slot_id)) => {
        Ok(self.slot_track(&slot_id)?.map(|track| track.eval(t)))
    }
    Some(PropertySource::Link(link)) => {
        let source_t = t + link.time_offset;              // 経路Cと同じ「静的offset」
        let source_value = self.value_at(link.source_layer, &link.source_property, source_t)?;
        // ↑ 再帰呼び出し。source 自身が Track/Slot/Link のどれでも同じ口を通る。
        Ok(source_value.and_then(|v| translate_link(&link.plugin_id, &link.params, v)))
        // translate_link は motolii-eval 側の closed match。型不一致は None
        // (裁定153 の EXACT TARGET #2 と同じ規約 — 黙って近似しない)。
    }
    None => Ok(None),
}
```

`translate_link` の形は `next/engine/motolii-engine/src/lib.rs:1281-1297` の `translate_glow_params` と同型(`find(name, default) -> Option<f64>` で named param を読み、型不一致は `None`)。**新しい評価機構を作らない**——既存の `StoreView::value_at` 1本がそのまま再帰する。

### 1.5 決定論と循環拒否:既存の `validate_no_parent_cycle` をそのまま転用

`next/core/motolii-store/src/document.rs:1375-1403` の `validate_no_parent_cycle`(layer の `parent` 循環を書き込み時に拒む、`HashSet` で防御的に保険をかけた while ループ)を **`(LayerId, PropertyId)` ペア版**として複製するだけで足りる——新しい検査手法の発明ではない。

```rust
// document.rs の validate_no_parent_cycle と同型(擬似コード)
fn validate_no_link_cycle(
    view: &StoreView,
    layer: LayerId,
    property: &PropertyId,
    new_link: &PropertyLink,
) -> Result<(), StoreError> {
    let start = (layer, property.clone());
    let mut current = Some((new_link.source_layer, new_link.source_property.clone()));
    let mut seen = HashSet::new();
    while let Some(candidate) = current {
        if candidate == start {
            return Err(StoreError::Property("link が循環する".into()));
        }
        if !seen.insert(candidate.clone()) {
            break; // 既に壊れた鎖(バグ由来)を無限に辿らない防御
        }
        current = view.source_at_path(&candidate.0.entity_path(), &candidate.1)?
            .and_then(|src| match src {
                PropertySource::Link(l) => Some((l.source_layer, l.source_property)),
                _ => None, // Track/Slot は鎖の終端
            });
    }
    Ok(())
}
```

**Blender との重要な差**: Blender の依存グラフは循環を**実行時**に検出し「ランダムな点で切って」評価を続ける(§1.1、issue #64793 が報告する「ジャンプ・チラつき」の原因そのもの)。Motolii は `Intent` の**書き込み時**に拒否する——`validate_no_parent_cycle` の先例と同じ「循環参照は絶対に作れない」規律(document.rs:1378)を踏襲すれば、Blender より強い保証になる。実行時に「たまに壊れる」状態を許すと、`motolii-eval` の純関数契約(「時刻t→値」`next/core/motolii-eval/src/lib.rs:15`)そのものが崩れ、Preview=Export(裁定15/M15)の保証が失われるため、書き込み時拒否は妥協ではなく必須。

**決定論そのもの**は Link が読むのは (a) `t + time_offset`(Document 由来の静的値)(b) 参照先 property の**同じ `value_at` 経路**で解決された値(c) `params`(Document 由来の animatable 値)だけであり、壁時計・ライブ音声・乱数・OS入力を一切読まない。`motolii-eval` の「時刻t→値の純関数」契約(`next/core/motolii-eval/src/lib.rs:15`)にそのまま収まる。

### 1.6 保存:untagged serde の第三の枝は移行コストゼロ

`slot.rs` 自身が「移行コストゼロ」を試験で固定している(`property_source_track_serializes_identically_to_a_bare_keyframe_track`、slot.rs:105-114)——`Track` は裸の `KeyframeTrack` と、`Slot` は裸の文字列とビット単位で同じ JSON になる。`Link`(オブジェクト、フィールド名 `source_layer`/`source_property`/`time_offset`/`plugin_id`/`params`)は `Track` の `{"keys":[...]}` とも `Slot` の裸文字列とも構造的に衝突しない第三の形なので、**同じ untagged trick がそのまま使える**。旧 Document(Track/Slot のみ)は無改造で読める——保存形式(`.rrd`)を壊さない、という発注書の制約をこの一点だけで満たす。

### 1.7 undo:`Document::apply_all` に乗るだけ

`Intent::SetPropertyLink { layer, property, link: PropertyLink }` を `Intent::SetPropertySlot`(document.rs:277-281)と同型で追加すれば、既存の `Document::apply_all`(裁定48・M10)がそのまま1 gesture = 1 undo を保証する。新しい undo 機構は不要——これは第1周が判定表(A2)で既に指摘した「新機構ゼロ」という Motolii の一貫した設計様式に沿う。

### 1.8 A1(音反応)をこの枠へ落とす

第1周 A1 の判定は「`AudioMeter` は Document へ流す口が無く、`waveform_peaks` は表示専用の一括解析」だった。**この枠に落とすと Link の2番目の設計判断がクリアになる**:

- `AudioMeter`(`next/engine/motolii-audio/src/meter.rs`)は module doc が明言する通り「Documentへ永続化しない」——これは Link の source として**構造的に使えない**。Link は `value_at` の再帰で解決されるが、`AudioMeter` はセッション中の再生状態にしか存在せず、export 時(再生していない)に同じ値を再現できない。**これは実装の遅れではなく、Preview=Export(裁定15)を守るための必然的排除**。
- 正しい経路は「先にベイクする」(裁定150・GOALS.md B15「Convert Audio to Keyframes」、`next/reference/normal-map.tsv:480` 採用予定)——`waveform_peaks` から作った解析結果を**普通の `KeyframeTrack`**として保存し、それを Link の `source_property` にする。この時点で **A1 は A2 の特殊形に潰れる**——音声解析専用の新しい参照機構は要らない。
- 結論: Link を先に作れば、A1 は「ベイクした track を Link の source にする」だけで済み、第1周 D 節が示した「まず Link → 次に A1」という順序がそのまま設計の必然になる。

### 1.9 裁定191 との緊張——正直な判定

**ここが本設計の一番弱い点であり、supervisor/利用者裁定を要する**。`next/reference/lottie-coverage.tsv` を全文grepした結果、Lottie の property-to-property 参照機構は `x`(Expression、325/575行)`ix`(Property Index、295行)の2つしかなく、**両方とも不採用**——理由は「JS文字列式は要らないもの」「処理系を1本抱えることになり軸4に反する」。つまり `PropertySource::Link` は、Slot(`sid`、行324・採用済)のように**地図の行を直接引き継いだ拡張ではない**——Lottie の唯一の対応物(`x`)は不採用のまま。

しかし §1.3 で示した通り、`effect-values/*/ty`(93,96,99,102,111行)は「具体的な変換の中身は Document が型で持たない」という**分類自体**を既に地図へ書き込んでいる。この分類に従えば、Link は「**描画語彙(パス・シェイプ・mask・変形)ではなく、値の計算という付随的な振る舞い(behavior)**」であり、裁定191 が縛る対象(「データ意味の正本」)ではなく、effect(裁定70/72)と同じ「vism圏の入れ物」に属する——というのが本書の判定。ただしこれは**裁定70 からの類推であって、既存裁定がこの分類を明言したことは一度もない**。第1周 B2 が見つけた「Lottie は Document を縛るが vism圏は縛らない」という二層構造の実例は今のところ effect だけであり、Link をこの側へ分類してよいかは利用者裁定が必要な**新規の線引き**だと明記する。

代替案(§1.9 の判定が通らない場合の逃げ道): `PropertySource::Link` を Document/`.rrd` に一切持たず、Motolii 独自の非 Lottie 互換な**vism 側サイドテーブル**として持つ(Blender の driver が glTF/FBX に存在せず、export 時に F-curve へベイクされるのと同じ扱い)。この場合「常時追従する生きた link」は Motolii 独自ファイルの中でのみ有効で、Lottie 互換書き出しをする日には Link を辿って track へベイクする変換が要る。**Motolii の保存正本は上流 `.rrd` そのまま**(裁定55、第1周 B3)であって Lottie JSON そのものではないので、この代替案はどちらにしても致命的ではないが、「Lottie地図を汚さない」という発注の字面には代替案の方がより忠実。

---

## 2. vism trait の設計案(effect)

### 2.1 現状(コード実測、裁定153着地後)

第1周執筆時点の `effect-seam-survey.md`(調査時点、未着手)から進み、**現在は実装済み**——`next/engine/motolii-compositor/src/effects/mod.rs` の `EffectPass`(`Identity`/`Glow{threshold,intensity,radius}`、closed enum、`#[derive(Clone, Copy, Debug, PartialEq)]`)、`next/engine/motolii-engine/src/lib.rs:1252` の `translate_effect_passes`(`match effect.plugin_id.as_str() { "motolii.glow" => ..., _ => None }`)。第1周が確認した「`pub trait` ゼロ件」は現在も真——`EffectPass` は enum、`translate_effect_passes` は自由関数。

### 2.2 Tier 1(達成可能・今すぐ):engine 内の翻訳 trait

**裁定13に抵触しない**——同一 binary・closed 集合のまま、`translate_effect_passes` の match 腕を trait 実装の集合へ**内部リファクタ**するだけ。外部から見た「third-party が別クレートとして書く」体験は**まだ生まれない**が、GLow 以降の2本目・3本目の効果(halftone・feedback、裁定153が既に温存指名)を追加する時の write-set を engine 側1箇所の巨大 match から分散できる。

```rust
// next/engine/motolii-engine 内(擬似コード)。まだ pub trait を「外部公開」はしない
// — crate 内の私的 trait object 集合として飼う(裁定13「trait はまだ作らない」の
// 精神を保ちつつ、内部の見通しだけ良くする)。
trait EffectTranslator {
    fn plugin_ids(&self) -> &'static [&'static str];
    fn translate(
        &self,
        plugin_id: &str,
        params: &[(String, motolii_store::Value)],
    ) -> Option<motolii_compositor::EffectPass>;
}

struct GlowTranslator;
impl EffectTranslator for GlowTranslator {
    fn plugin_ids(&self) -> &'static [&'static str] { &["motolii.glow"] }
    fn translate(&self, plugin_id: &str, params: &[(String, motolii_store::Value)])
        -> Option<motolii_compositor::EffectPass> {
        // 中身は現行 translate_glow_params (lib.rs:1281-1297) をそのまま移すだけ
        todo!()
    }
}

fn translate_effect_passes(
    translators: &[&dyn EffectTranslator],
    effects: &[motolii_store::ResolvedEffect],
) -> Vec<motolii_compositor::EffectPass> {
    effects.iter().filter_map(|effect| {
        translators.iter()
            .find(|t| t.plugin_ids().contains(&effect.plugin_id.as_str()))
            .and_then(|t| t.translate(&effect.plugin_id, &effect.params))
    }).collect()
}
```

これは**まだ `EffectPass` 自体を開かない**——新しい shader を足すには依然として `next/engine/motolii-compositor/src/effects/mod.rs` の enum へ variant を足す必要がある。つまり Tier 1 だけでは「外部作者が別クレートとして」効果を書けるようにはならない。**内部整理としてのみ**有用。

### 2.3 Tier 2(凍結ゲート・今は作らない):compositor を開く trait

`docs/plugin-authoring.md` §8「まだ凍結していない口」が既に同じ形の口を予約している——「Vismのsource／WGSL／WASM／native payload、動的load」(306行)。Tier 2 はこの予約口の具体化案として、擬似コードだけ示す(**実装しない**):

```rust
// next/engine/motolii-compositor(擬似コード・未実装・凍結ゲート待ち)
// EffectPass の closed enum を置き換えるのではなく、"Identity"/"Glow" の隣に
// 3つ目の選択肢として Dynamic(Box<dyn EffectPassKind>) を足す形が最小差分。
pub trait EffectPassKind: Send + Sync {
    /// 現行 EffectPass::padding() と同じ契約(次項参照)。
    fn padding(&self, params: &EffectParams) -> u32;
    /// 初期化時に1回だけ呼ばれる(F-10「毎フレーム新規生成しない」§3-3禁止6と同じ規律)。
    /// 返した handle は EffectScratch と同じ寿命で Host(compositor)が所有する
    /// ——プラグインは device/RenderContext への生アクセスを持たない
    /// (plugin-authoring.md §3禁止3「隠れた可変状態を持たない」の GPU 版)。
    fn build_pipeline(&self, device: &wgpu::Device) -> PipelineHandle;
    /// 純関数契約: 同じ params + 同じ入力 texture → 同じ出力(purity試験の対象、
    /// plugin-authoring.md §7チェックリスト「同じtに2回呼んでも同一出力」のGPU版)。
    fn encode(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &EffectRenderCtx,   // source texture view / scratch pool の型付き貸与のみ
        params: &EffectParams,
    );
}
```

**なぜ今作らないか(裁定13 そのものの理由を実測で補強)**:

1. `next/reference/KNOWN.md:44`「effect の複数 pass は連鎖しない(最後勝ち)」がまだ未修正——1個の内製効果でさえスタックが未完成の段階で外部向け trait を凍結すると、「壊れた内部を前提に口を決める」ことになる(裁定13 が警戒する「口を先に決めると中身が歪む」の教科書的な例)。
2. `EffectScratch`(mod.rs 内の texture プール)は現状 Glow 専用の再利用パターンで検証済みだが、**任意 pipeline を汎用に持ち回れることはまだ実証されていない**——feedback(フレーム間状態)・halftone(解像度非依存の別軸)という異なる形の効果が実際に着地して初めて、`EffectRenderCtx` が本当に必要十分な貸与量かが分かる。
3. `Compositor::ctx`(`RenderContext`)は private field(`next/engine/motolii-compositor/src/lib.rs:161`)——Tier 2 は「device への直接アクセスを与えずに shader を書かせる」という新しい capability 境界を発明する必要があり、これは `docs/extensible-core-model.md` §5 の capability 表(196-211行、"Evaluate": 「宣言入力から結果を返す」/ Host が「scheduling、cache、resource、Preview/Export」を持つ、という既存分類の GPU 版)を新規に具体化する作業であって、片手間でできる分量ではない。

### 2.4 「2人目の利用者」問題への見解

**このペルソナは2人目の利用者に該当しない、というのが本書の見解。** 裁定13 が待っている「2人目の利用者」は、TD 出身者が抱く「別プロセスとして書きたい」という**願望**ではなく、**実際に実装を試みて壁にぶつかった実績**——ペルソナの思考実験はその代用にならない。

**本当の2人目・3人目は既に名指しされている**——`docs/reviews/2026-08-21-effect-seam-survey.md` §5 が halftone・feedback を「vism第2号以降」として明示的に温存し(259-265行)、裁定153 の末尾も「feedback系はフレーム間状態を要するため vism 第2号以降へ温存」と同じ結論に達している。この2つが実際に compositor 内へ着地すれば:

- halftone(解像度非依存という Glow と異なる軸)が **同じ closed enum の中でどれだけ形が変わるか**を実測できる。
- feedback(フレーム間状態、`spikes/m5-known-implementation/M5-R0/src/feedback.rs` の ping-pong texture)が **`EffectScratch` の「毎フレーム再生成しない」モデルをどこまで再利用できるか**を実測できる。

この2つの実測結果が揃って初めて、Tier 2 の trait 形状(`EffectRenderCtx` が何を貸すべきか、`build_pipeline` の寿命管理をどこまで Host が肩代わりすべきか)が**推測ではなく証拠**に基づいて決まる。**推奨する順序**: Glow(済)→ halftone → feedback(いずれも内製・closed enum のまま)→ その3例が出揃った時点で Tier 2 trait 設計を再着手。TouchDesigner ペルソナのような外部作者志望者は、この3例が揃った後の「口」が実際に開いた時に初めて迎え入れられる対象になる。

---

## 3. コアと vism の線(この2案に即して)

`docs/reviews/2026-08-22-core-vs-vism-classification.md` の判定式(「動画編集ソフトとして成立するために必須か」)をこの2つの提案物へ適用する。

| 要素 | 判定 | 理由 |
|---|---|---|
| `PropertySource` の3択ディスパッチ機構(Track/Slot/Link を同じ口で解決する仕組みそのもの) | **コア** | これが無いと Link は原理的に存在できない土台(Slot と同じ扱い)。M19(keyframe操作)・M14(正本1つ)の背骨と同じ層 |
| Link の書き込み時循環拒否・undo・save(§1.5-1.7) | **コア** | Document の整合性を守る Host 専権事項(`docs/extensible-core-model.md` §3「Host が小さくても手放さない責任」の直系) |
| `plugin_id` が指す**具体的な変換の中身**(identity/linear/remap の実装、将来増える変換の種類) | **vism** | effect の `ty` が不採用のまま v(Value)だけ採用済みなのと同型(§1.3)。新しい変換を1つ足すのに Document 側の型は増えない |
| `EffectPass` closed enum・`translate_effect_passes` の配線(現行 Glow) | **コア寄りfirst-party**(裁定70 の言い方: 型にしないが同梱はする) | Document は effect を型で持たないが、compositor/engine の実装自体は現状 Motolii 自身のソースの一部——ここまでは「拡張の戸」ではなくまだ内製 |
| Tier 2 の `EffectPassKind` trait(未実装) | **vism(将来)** | まさにこれが「口」——実装されて初めて第三者が compositor のソースを fork せずに効果を足せる。GOALS.md D6「拡張の口が trait 1本」が指す対象そのもの |

線の引き方の要約: **コアは「土台となるディスパッチ機構と、その整合性を守る規律(循環・undo・save)」まで。vismは「土台の上に載る具体的な中身(変換の種類・shader の実装)」から先**。Link も effect も、この線の位置は同じ形をしている——第1周 B2 が見つけた「Lottieは意味を縛るがvism圏は縛らない」の二層構造と、コア/vismの線は**別の軸**であることに注意(裁定191 は描画語彙の軸、コア/vism は「動画編集ソフトとして必須か」の軸)。

---

## 4. 先例引用まとめ

| 先例 | 一次資料 | 写した/写さなかった部分 |
|---|---|---|
| TouchDesigner CHOP Export | [derivative.ca/UserGuide/Export](https://derivative.ca/UserGuide/Export)、[docs.derivative.ca/CHOP_Export](https://docs.derivative.ca/CHOP_Export) | 写した: 片方向・名前束縛・宛先だけが参照を持つ非対称性 |
| Blender Driver / DriverVariable | [docs.blender.org/manual...drivers_panel](https://docs.blender.org/manual/en/latest/animation/drivers/drivers_panel.html)、[docs.blender.org/api/current/bpy.types.DriverVariable](https://docs.blender.org/api/current/bpy.types.DriverVariable.html)、循環実測: Blender Projects issue [#64793](https://projects.blender.org/blender/blender/issues/64793) | 写した: 閉じた関数集合という区分。**写さなかった**: 循環の実行時検出・事後警告(Motolii は書き込み時拒否で上回る) |
| After Effects pick whip / wiggle | [helpx.adobe.com expression-basics](https://helpx.adobe.com/after-effects/using/expression-basics.html) | 写さなかった: 文字列式そのもの(Lottie地図で不採用済み)。**発見**: wiggle は Link の問題ですらない(既存のレベル0純関数で解決済み) |
| `docs/plugin-authoring.md`(旧世界・歴史文書) | 本リポジトリ、1-318行 | Tier 2 trait の purity 契約(§3禁止3)・「まだ凍結していない口」という提示形式(§8)をそのまま模倣 |
| `docs/generative-user-boundary.md` | 本リポジトリ、経路B「Pure Live f(t)」table(96-99行付近) | 「型付きLink」という語彙は2026-07-15時点で既にこの文書の2章に存在していた(先取りされていた概念)ことを確認 |
| `docs/extensible-core-model.md` | 本リポジトリ、§4.2 Behavior定義(180-186行)・§5 capability表(196-211行) | Connect capability の「型検査・循環拒否はHost、公開paramの宣言はplugin」という分担をLink/vism双方の線引きへ転用 |

---

## 逸脱

- 発注書は「先例を一次資料で当たる」と指示したが、Blender/TD/AE の公式ドキュメントの一部(`docs.blender.org` 本体)は 403 を返したため、検索結果のスニペット経由の間接引用と、代替可能な同系ミラー・APIリファレンスページで補った。挙げた URL は実際に到達確認済みのものだけを残した。
- §1.9(裁定191 との緊張)は発注書が「保存形式と Document の意味論を壊さない形で」提案することを求めていたが、**壊さない形が一意に決まらなかった**——本文中の主提案(`PropertySource::Link`)は 裁定70 からの類推で正当化しているが、既存裁定がこの類推を明言したことはない。これは調査の失敗ではなく、発注書自身が触れている「二層構造」(裁定191)の適用範囲が effect 以外に及ぶかどうかという**未決の論点**そのものである。代替案(vism側サイドテーブル、Lottie地図に一切触れない形)も併記し、判断を利用者/supervisor へ返す。
- vism trait の Tier 1/Tier 2 という2段階分けは発注書に明示された枠ではなく、「口を先に決めない」裁定13 と「口の形はいずれ要る」GOALS.md D6 を両立させるために本書が導入した区分。Tier 1 は今すぐ着手可能な範囲、Tier 2 は凍結ゲート待ちとして明確に分離した。
