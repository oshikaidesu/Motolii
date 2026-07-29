# M5 scene-color input contract decision

作成日: 2026-07-29

状態: **決定／P2D-RCR1 DONE**

## 1. Authorityと方式分離

本decisionはRender Contribution F4、[typed seam](2026-07-29-m5-render-contribution-typed-seam-decision.md)、
[scene-color semantics](2026-07-29-m5-scene-color-semantics-decision.md)、M4-K0のRoD／RoI契約を正本とし、
scene-color／refraction要求のsnapshot、range、order、failureを閉じる。

snapshotは意味入力であり、copied textureを意味しない。copy、subpass、input attachment、
resource lifetime、barrier、sampler／paddingは`P2D-RCP1`へ残す。

## 2. immutable pre-requester snapshot

scene-color inputは、**要求した一seatの一評価要求が持つ決定論的ordering pointの直前で凍結した、
immutable canonical scene-color result**である。

含むもの:

- Host orderingで要求より前に完全resolveされた全contribution。
- 選択policyで合成済みのcanonical linear-premultiplied scene-color。
- 要求と同じ`t`、Observation／world、`Quality`、`FrameDesc`評価。

含まないもの:

- requester自身のcolor／depth／partial output。
- requesterより後のcontribution。
- 未完了／部分admitされたupstream output。
- UI／Preview overlay、別Export経路の像。

snapshotはrequester評価中に不変であり、providerは同じ要求内で後から出すpixelを観測できない。
Preview／Exportは同じ評価関数とsnapshot pointを使う。

## 3. orderingと循環拒否

Hostはadmit済みtyped requirementから決定論的なtotal orderまたはacyclic dependency orderを作る。
requesterは宣言したupstream scene-color pointだけを読める。

次を型付き拒否する。

- self-read。
- 同じsemantic stepでlive canonical targetをread／writeするalias。
- requester間のdependency cycle。
- later／unresolved／final-scene resultの要求。
- orderingが一意に決まらない要求。
- 非対応snapshot point。

provider ID／provenanceでorderingを変えず、公開engine phase enum／sort keyを作らない。
複数refraction contributionはHost orderで明示的に前にresolveされたものだけを順次snapshotへ含める。

## 4. logical input range

要求は次から導くlogical scene-color RoIを宣言する。

1. contributionのrequested output region。
2. refraction／displacement mapping。
3. finite filter／sample footprint。

rangeはruntime-derivedでDocumentへ保存せず、backend pixel／texture boundsでなくK0のcanonical extentを使う。

- `Finite`: 算出したscene-color RoIを要求する。
- `Infinite`: active canonical Final／Stage demandとHost safety limitへclampする。
- `Unknown`: empty／tight boundsにせず、利用可能なupstream RoDまたはHost safe limit全域へ保守的fallbackする。
- 過小`Finite`がfull evaluationとpixel差を出した場合はconformance failure。
- GPU alpha readbackでrangeを導出しない。
- allocation pressureを理由にRoIを黙って縮めない。

## 5. out-of-domain意味

logical upstream domain外は、RCFP1Sのcanonical linear-premultiplied **transparent black**
（RGB=0、alpha=0）として観測する。undefined memory、wrap、edge clamp、別target、stale pixelを読まない。

この意味をcopy padding、border texture、sampler address mode等のどれで実現するかはRCP1へ残す。
必要なdomain外accessを選択方式が保証できなければwhole-request typed refusalとする。

## 6. self-read禁止と許可形

許可:

- immutable pre-requester snapshotを読む。
- Host所有の別outputへ書く。
- Hostが後でそのoutputをcanonical resultへ合成する。

禁止:

- requesterのlive outputへのfeedback。
- same-resource／same-step read-write。
- snapshot pointを変える隠れcopy。
- undeclared sub-contributionへ分割してpartial outputを読む。
- Preview-only／Export-only targetを読む。

将来feedbackが必要なら、refractionへ偽装せず時間／状態の別契約を作る。

## 7. whole-request preflight／failure

Hostはsnapshot point、acyclic order、capability、RCFP1意味／format compatibility、RoI、
off-domain support、resource need、budgetを一要求としてpreflightする。

一つでも失敗すればcanonical output mutation前に要求全体を拒否する。

- partial refraction outputなし。
- normal blend／opaque等のreplacementなし。
- range縮小／別snapshot pointへのfallbackなし。
- Document／他要求の既存画素変更なし。
- successful outputを称するcache entryなし。

diagnosticはself-read、cycle、unavailable snapshot、format／range／off-domain unsupported、
resource rejection等の意味原因を区別し、provider identityで分類しない。

## 8. cache completeness

具体cache keyは`P2D-RCBUD1`へ残すが、scene-color結果のidentityは少なくとも次へ依存する。

- exact upstream snapshot generation／content identity。
- semantic snapshot ordering pointとordered upstream dependency generation集合、または同等のHost fingerprint。
- `t`、`Quality`、`FrameDesc`。
- Host所有Observation／world／transform入力。
- RCFP1 color／alpha／format contract version。
- requested output regionとscene-color input RoI／extent。
- refraction parameterとcontribution capability／version。
- transparent-black off-domain意味。
- upstream canonical snapshotを変えるpolicy／admission結果。

GPU texture address／handle、copy／subpass選択、backend barrier／pass ID、semantic入力が同じ時の
provider provenance、UI overlayはkey入力にしない。

## 9. conformance oracle

- 二つのupstream色からresolved sceneを読み、直前provider一つだけを読んでいない。
- later layerがsnapshotへ現れない。
- upstream scene-colorだけの変更でrequesterが再計算される。
- requester order変更でsnapshotが決定論的に変わる。
- self-readとA↔B cycleがoutput mutationなしで拒否される。
- finite displacementが宣言量だけRoIを拡張する。
- Unknown fallbackとfull evaluationがpixel一致する。
- under-declared rangeをfull-evaluation比較で検出する。
- domain外がcanonical transparent blackで、wrap／clamp／stale readにならない。
- DRAFT／FINAL、Preview／Exportが同じsnapshot意味を使う。
- 将来copy／subpass両方式を残すならcanonical pixelが一致する。

## 10. STOP

- FP16等のconcrete format、texture lifetime、barrier、sampler、paddingを先取りする。
- K0と違うextent／RoI型を発明する。
- engine phase、pass ID、sort key、texture handle、backend APIを公開する。
- requesterのlive self-read、partial admission、pressure時range縮小を許す。
- provider／First Vism固有snapshot pointを作る。
- snapshot／range／order／cache derived stateをDocumentへ保存する。
- Preview／Exportで別scene-color sourceを使う。
