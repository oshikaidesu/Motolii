# CU-106 selection consumer分割決定

- 日付: 2026-07-27
- 状態: **決定**
- CU-106S: **DONE**
- CU-106: **SPLIT**
- CU-106P / CU-106F: **WAIT**

## 1. 再確認した事実

CU-106Sは、U2h-1P selection producerと同じ差分で成立する実在production callerを探し、
primary selectionとessential focusを分離するためのdocs-only再確認である。

- `DocumentEditAction`はApply / Undo / Redoだけで、selection-only actionは未実装である。
- `DocumentEditQueue`のnon-test push siteはU2b-1 document smoke内だけであり、通常起動では
  queueへactionを入れる利用者入力がない。
- `NormalizedInput`にpointer / hit入力はなく、`DomainIntent`へselection variantを加えると
  公開APIとadapter kind契約を変更する。
- `project_timeline` / `TimelineHit`のproduction callerは0件である。
- egui側にTimeline製品面はなく、新規製品面をeguiへ実装することは禁止済みである。
- U3a-2はwindowed native Timelineを所有するが、依存の`G0-9`がG0-9LとG0-9Dの
  どちらまでを要求するかは未分割である。

したがって現時点でU2h-1Pを施工すると、lint抑制、dummy / `#[cfg(test)]` caller、
smoke期待列への混入、公開intent追加、またはU3a-2の先取りが必要になる。
producer-only実装へ戻さず、consumer surface成立まで待つ。

## 2. CU-106の分割

| ID | 責任 | 状態 | 入場条件 |
|---|---|---|---|
| CU-106P | primary selection consumer。U2h-1P P5を内包し、producerと実在callerを同じ差分で成立させる | `WAIT` | U3a-2入場範囲が決定済みで、`TimelineHit`を呼ぶnon-test consumerとpointer相当のproduction入力が存在する |
| CU-106F | essential focus。primary selection、hover、surface focusを混同せず、focus ownerと移譲を閉じる | `WAIT` | 実consumer surfaceとproduction入力が成立し、U3a-2 / Host coordinatorのfocus責任を先取りせず記述できる |

CU-106親は`SPLIT`とし、親名でclosed orderを作らない。三surface接続、hidden件数、
additive / range / marquee / bounded AXはCU-106P/Fへ束ねず、U2h-2または後続粒に残す。
playhead / range ownerもこの決定では定めない。

## 3. CU-106Pの到達性ゲート

CU-106Pを`DO`へ移す前に、次をコード事実として全て確認する。

1. `TimelineHit`または同じ既存typed hit結果を使うnon-test callerがある。
2. pointer相当のproduction入力がHostの既存閉じた境界からcallerへ届く。
3. selection producerとcallerを同じ差分で実装でき、未使用private APIを残さない。
4. lint抑制、dummy参照、`#[cfg(test)]`到達性、env-gated smokeを製品callerとして数えない。
5. 公開`DomainIntent` / adapter kind、Document、serde、journal、Undo、plugin契約を変えない。

一つでも満たさなければ`WAIT`を維持する。

## 4. 後続実装の必須負例

CU-106P実装時は、U2h-1PRで決定済みの順序と既存U2h-1I経路を再利用する。

1. `ReplacePrimary`は`find_envelope`による存在拒否をsame-id no-opより先に行う。
2. same-idと`ClearPrimary(None)`はpublish 0で、generation / revision / Document /
   history / primaryを変えない。
3. 拒否とno-opもactionを1回消費し、自動retryしない。
4. accepted changeは`u64::MAX`枯渇preflightをmutation前に通る。
5. 第2 selection store、第2 generation counter、第2 reconcile経路を作らない。
6. non-test callerが消えた時に到達性を検知できる試験またはlintを置き、
   test-only参照をproduction到達性として合格させない。

## 5. 次の判断

`U3a-2S0`で発注依存証跡を閉じた後のdocs-only `U3a-2S`は`DONE`であり、
U3a-2の`G0-9`依存を、G0-9Lの固定Mac prerequisite evidenceで入場できる範囲と、
G0-9DのWindows / 追加hardware / Distribution Readyまで待つ範囲へ分けた。
現行の次のPRODUCT-ASSET判断はdocs-only `U3a-2R`の`DO`とする。
windowed native Timelineを今どこまで施工できるかを決めるまで、CU-106P/Fを起動しない。

## 6. 非目標

- Rust / JS / fixture / guard / golden / threshold変更。
- U2h-1P、CU-106P/F、U3a-2、Host transport、製品入力adapterの実装。
- semantic zoom段階、playhead / range owner、production pointer eventの新しい意味。
- 公開API、Document、serde、journal、Undo/history、ProjectSession、plugin契約の変更。
- egui製品Timeline、surface別selection store、dummy callerの追加。

## 7. STOP

1. CU-106P/Fを実consumer surface成立前にproducer-onlyで実装したくなる。
2. 到達性のためにlint抑制、dummy / test-only caller、smoke混入が必要に見える。
3. 公開intent / keymap / transport契約または永続意味を変える必要がある。
4. CU-106Pへfocus、三surface接続、hidden件数、additive / range / marquee / AXを束ねたくなる。
5. U3a-2S前にG0-9L / G0-9Dの依存範囲を推測してwindowed製品面へ着手したくなる。
