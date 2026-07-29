# M5 occlusion policy schema decision

作成日: 2026-07-29

状態: **決定／P2D-RCD2 DONE**

## 1. AuthorityとGR-PV判定

本decisionは[M5仕様](../specs/M5-3d-and-post.md)、
[Render Contribution typed seam](2026-07-29-m5-render-contribution-typed-seam-decision.md)、
[alpha意味decision](2026-07-29-m5-render-contribution-alpha-semantics-decision.md)、
[M2恒久焼き込み予防](2026-07-12-m2-permanence-prevention.md)を正本とする。

GR-PVの5条件を次で満たす。

1. 意味は既決のGroup visibility policyと明示`Depth Participant`だけ。
2. 恒久面はGroup policy key一つとItem participant一つだけ。
3. 現行fieldの再解釈をせず、次Document versionへの追加migrationとする。
4. M2-D1e完了後にdecisionを閉じ、schema実装は独立`P2D-RCD2I`へ分ける。
5. migration pixel不変、未知policy非fallback、Undo完全復元をsemantic oracleにする。

## 2. 永続ownerと最小面

- Groupは一つの`OcclusionPolicyKey`を所有する。
- 各`ItemEnvelope`は一つの明示`Depth Participant` booleanを所有する。
- participantは直接親Groupが`AE-style Bins`を選んだ時だけvisibility意味へ参加する。
- それ以外では値を**保持したままinert**とする。policy切替やreparentで黙って書き換えない。
- Advanced panelの開閉、導出bin、render order、diagnostic、runtime availabilityは保存しない。

policy切替はDocument子順、transform／Z、selection、participant値を変えない。
participant切替もpolicy、子順、transformを変えない。

## 3. 拡張可能なsemantic policy key

永続keyはprovider／package／registry identityでなく、Hostが所有するpolicy意味のidentityである。
wire上は次の閉じた構造だけを持つ。

長さはUTF-8 byte数で数えるが、許可文字はASCIIだけなので文字数と一致する。

| field | wire型／上限 | 完全文法 |
|---|---|---|
| `namespace` | string、3〜253 bytes、dot区切り2〜16 segment、各1〜63 bytes | 各segmentはlowercase letterで始め、中間はlowercase letter／digit／hyphen、2文字以上ならlowercase letterまたはdigitで終える。空segment、連続／先頭／末尾dotを拒否 |
| `name` | string、1〜63 bytes | lowercase letterで始め、lowercase letter／digitの語を単一underscoreで連結する。連続／末尾underscoreを拒否 |
| `major` | unsigned 16-bit integer | `1..=65535`。同じnamespace／nameで意味互換でない変更だけ増やす |

namespaceのsyntax合格はauthority、trust、package eligibilityを証明しない。それらはtyped seam §5の
前段責務であり、policy keyの保存可否や意味を変えない。

組み込みkeyは次で固定する。

| UI意味 | namespace | name | major |
|---|---|---|---:|
| `Layer Order` | `org.motolii.occlusion` | `layer_order` | 1 |
| `Group Depth` | `org.motolii.occlusion` | `group_depth` | 1 |
| `AE-style Bins` | `org.motolii.occlusion` | `ae_style_bins` | 1 |

このkeyは公開render phase、sort key、provider dispatch keyではなく、contribution APIへ渡さない。
runtime Host catalogがsemantic policy keyから利用可能な実装／能力を解決するが、そのcatalog形と
provider bindingは本decisionで決めずDocumentへ保存しない。

未知のwell-formed keyはload／save／Undoで原形を保持する。semantic admissionでは型付き
unsupported／unavailableとして拒否し、`Layer Order`や同名別majorへfallbackしない。
malformed keyはDocument validationで拒否する。

## 4. defaultとmigration

schema実装時の前置条件は`LATEST_DOCUMENT_VERSION == READER_VERSION == WRITER_VERSION == 5`である。
違う場合は実装を停止し、最新authorityへ再基線化する。前置条件が維持される場合、
`P2D-RCD2I`はversion 6を追加する。

v1〜v5の全Groupへ`Layer Order` key、全Itemへ`Depth Participant = false`をD1e migrationで
明示追加する。Rust／serdeの暗黙`Default`をmigrationの代わりにしない。

versionと`min_reader_version`を既存nested-schema helperで同時に上げる。旧readerは新fieldを
捨てて描画せず型付き拒否する。v1〜v5を名乗りながらpolicy／participant fieldを持つ
version spoofは拒否する。

## 5. D2 command／Undo／journal

既存property-scoped command形を再利用し、概念上次の二つを追加する。

- `SetOcclusionPolicy { target_group, old, new }`
- `SetDepthParticipant { target_item, old, new }`

両commandはtarget存在とold値一致をmutation前に検査する。`SetOcclusionPolicy`だけはGroup targetと
keyの**上記wire文法だけ**を追加検査し、`SetDepthParticipant`は任意Item targetのbooleanを扱う。
catalog membershipは検査しない。unknown well-formed keyもpolicy commandの`old`／`new`へ置け、
inverse／redoでbyte-equivalentに保持する。利用可能性はrender semantic admissionの責務である。
ON／OFFまたは一Groupのpolicy切替は1 command＝1 gesture＝1 Undoとする。複数participant編集は
既存`apply_macro`を使い、新しいtransaction APIを作らない。

inverseはDocument全体を完全復元し、redoは初回適用後と一致する。既存journal variant payloadや
v1 adapterを変更せず、新command variantだけを追加する。

## 6. migration／Undo oracle

- v1〜v5 corpusのmigrationが明示的かつ冪等。
- version／reader／writer／`min_reader_version`が同じ境界で上がる。
- migrated `Layer Order`が既存project pixelを不変にする。
- count、stable ID、子順、keyframe、dependency、`extra`、camera、effect、mask、transformが不変。
- unknown well-formed policyがsave→reopen→save、Undo／Redoで保持され、fallbackしない。
- version spoof、malformed key、missing target、non-Group target、stale old値が変更0で型付き拒否。
- policy switchで子順／participant値不変、participant switchでpolicy／子順不変。
- policyの異なるGroup間へreparentしてもparticipantをbit-for-bit保持する。
- unknown well-formed policyを持つGroupをduplicate／copyしてもkeyと全participantを保持する。
- apply→inverseがDocument全体を復元し、redoが初回結果と一致する。

semantic pixel oracle artifactと変更可能なharnessを分離し、oracleだけを
`classification.tsv`へ登録する。期待値変更でmigrationを通さない。

## 7. 保存しないもの

alpha class、cutout threshold、OIT mode、fallback policy、render phase／queue、sort key、
depth format、resource／budget、capability list、provider／package ID、private payload、
derived bin、diagnostic、UI開閉状態を保存しない。

`FrameDesc`、公開render trait、`Group.children: Vec<TrackItem>`の形を変更しない。
participantのためにchild-edge wrapperを新設しない。

unknown／unavailable policyを持つDocumentは保存可能だが、そのGroupを評価する要求は型付き拒否される。
拒否をGroupだけへ局所化するかframe全体へ伝播するか、製品表示が直前frameを保つかは
P2D runtime projection／M3 presentationの後続契約であり、RCD2／RCD2Iは決めない。
どの方式でも代替policyへのfallback、Document rewrite、replacement contributionは禁止する。

## 8. 必須負例

- effect、mask、object type、alpha、provider／package provenanceからpolicy／participantを推測する。
- policy変更でchildを並べ替える、Z／transformを書き換える、helper layerを生成する。
- participant値をscope外で削除または自動変更する。
- unknown／unavailable policyを`Layer Order`へfallbackまたは保存時に正規化する。
- raw JSON、単一opaque string、`Any`、provider-private payloadをpolicy keyにする。
- policy keyをcontribution APIの第二admission面またはprovider dispatch keyにする。
- alpha／OIT／copy／budget fieldをRCD2へ便乗して追加する。
- migration／golden期待値を書き換えて既存pixel不変を偽装する。

## 9. STOP

- structured semantic keyだけでは成立せず、provider／package契約やraw payloadが必要になる。
- participant追加に`Group.children`置換または既存意味の再解釈が必要になる。
- old projectを`Layer Order`でpixel不変に移行できない。
- alpha、OIT、fallback、phase、resourceを永続化しないとschemaが成立しない。
- 現行Document versionが5でなくなり、別のnested migrationと競合する。
- unknown policy保持とtyped refusalを両立できず、silent fallbackが必要になる。

## 10. 後続

`P2D-RCD2I`だけがschema、D1e migration、D2 command、journal、semantic oracleを実装する。
render projectionはtyped seam実装と`P2D-RCF1I`へ分ける。RCT1のalpha classはrequest-sideであり、
RCD2Iへfieldを追加しない。

## 11. 反対側監査

2026-07-29、`claude-fable-5`をread-onlyで呼び、恒久wire、GR-PV、現行Document codeへ再照合した。
初回はkey文法の文字集合／上限が曖昧なままsemantic oracle化されていたため`REJECT`
（P0=0／P1=1／P2=3）。完全regex／上限、well-formedとcatalog membershipの分離、
reparent／duplicate oracle、runtime refusal scopeの延期を補った。

再審査は`VERDICT: ACCEPT`、P0=0／P1=0／P2=1。残るP2のcommand別validation文言も本節追加前に
修正した。最終判断は主担当Codexが正本と現行codeへ再照合した。
