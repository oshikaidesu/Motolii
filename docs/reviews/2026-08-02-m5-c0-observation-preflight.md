# M5-C0 Observation preflight

状態: **WAIT / SPECIFY前のdocs-only preflight**（2026-08-02）

## 1. 目的と結論

M5-A0Sと既知実装検証後に、Planar互換cameraからSpatial／Perspectiveへ進む前の
Observation Contract閉鎖点を、現行正本と実在code targetで固定する。本書は新しい公開型、
Document field、provider ID、serde、runtime実装を決定しない。

結論は、**製品M5-C0実装をまだ開始しない**である。M5-C0は次の意味decision／schema
specificationへ送れるまで準備済みだが、現行codeにその接続seamは存在しない。M4 K1aの
Host resource owner／hard budgetも未実装なので、M5 rendererの製品接続は同時に開始しない。

## 2. authorityと現行code fact

| 境界 | 現行authority／実在target | 成立している事実 | 未成立のgap |
|---|---|---|---|
| Camera意味 | [`Camera Object / Provider決定`](2026-07-24-camera-object-provider-decision.md) | Hostが観測の席、単一active camera、単一world、Planar互換維持 | Observationの公開型、初期capability閉集合、provider identity／version pin、parameter mappingは未決 |
| Document | `crates/motolii-doc/src/schema.rs`、`crates/motolii-doc/src/camera_eval.rs` | `CompCameraDoc::PlanarOrthographic`をruntime `CompCamera`へ評価 | Camera Object／active binding／Spatial providerのschemaは未実装 |
| Core camera | `crates/motolii-core/src/camera.rs` | `CompCamera`がPlanarのworld→NDC→pixel、FrameDesc整合、typed拒否を所有 | `CompCamera`はObservation Contractではなく、Spatial／Perspective providerではない |
| Render route | `crates/motolii-render/src/lib.rs`、`crates/motolii-plugin/src/lib.rs` | RenderRequest／Graph／LayerSourceContextが具体`CompCamera`を受け取る | representation非依存Observationを消費するroute、capability preflight、provider欠落診断がない |
| Oracle | `crates/motolii-core/src/camera.rs`、`crates/motolii-render/tests/cam_g0_planar_identity.rs` | Planar pixel identityと既存FrameDesc拒否試験がある | 二つの独立provider、Spatial golden、provider version mismatch、換装Undo、Preview／Export同一Observationはない |
| Resource gate | [`M4仕様`](../specs/M4-cache-and-analysis.md) K1a、実装台帳M4行 | Host ResourceLedger／hard budgetの意味は決定済み | ResourceLedger、全owner accounting、admission、M4 runtimeは未実装 |

## 3. M5仕様との突合

M5仕様P3はorientation補間、handedness／axis、projection／clip、target constraint特異点、
Planar切替、provider identity／version、bounds／picking参加、typed refusal、Preview／Export
同一評価を要求する。Camera決定はmatrix単独への縮約と、未実証のray／differential／shutter
fieldの先行恒久化を禁じる。

従って、次の問いに正本・公開境界・拒否fixtureを割り当てるまで、既存`CompCamera`の引数を
一般化したり、renderer側へ新しいmatrix／JSON／opaque IDを追加したりしない。

1. initial Observation capabilityの閉集合と、各consumerが要求するcapability。
2. Camera Object、active binding、provider parameter、version pinのDocument／wire形。
3. provider換装の全体preflight、parameter mapping、1 Undo／失敗時変更ゼロ。
4. provider欠落・version不一致・capability不足のtyped diagnosticとDocument不変。
5. bounds／picking／view-space depthのHost参加境界と、同期GPU readbackなしのoracle。
6. Planar providerとSpatial providerの3 OS／Preview／Export／pixel identity fixture。
7. Observationのcache key／invalidationと、M4 K1a resource ownerへの接続点。

## 4. 次の一手と停止線

次はM5-C0の**意味decision／schema仕様粒**を1契約境界で閉じる。その仕様化後にのみ、
private fixture（少なくともPlanar保持、独立provider 2種、capability拒否、provider mismatch、
換装Undo）を作り、公開型変更とruntime接続を別粒へ分ける。

以下はこのpreflightからは実行しない。

- `CompCamera`へSpatial／Perspective variantを追加する。
- `Observation`、Camera Object、provider registry、version pinの型・JSON・serdeを発明する。
- `RenderRequest`／plugin facadeをmatrixや具体provider IDへ変更する。
- M4 K1a未実装のままGPU resource owner、budget、cache、compiled assetを製品経路へ接続する。
- Planar以外を未対応時にPlanarへ黙ってfallbackする。

M5-A2までのprivate検証は独立して完了しているため、このWAITは既知実装検証全体の停止ではなく、
共有公開境界とM4 resource gateに依存するM5-C0／製品runtimeだけの局所停止である。
