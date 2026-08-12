# SetPositionKeyTime — position keyの時刻移動の閉じた契約

- 日付: 2026-08-12
- 状態: 決定(実装同時着地)
- 系譜: U4b-0(Add Position Key)・U4b-0V(SetPositionKeyValue)のCAS族に第三の成員を足す。Timeline設計決定(2026-08-08)「Timelineは時間の操作へ集約する」の直接の帰結であり、キーの水平dragはこの決定が予定した操作である(禁止されているのは縦ドラッグの値編集のみ)
- 先例確認: Rerun time panelは時間navigation/密度のみでkeyframe編集を持たない(2026-07-20 Rerun先例調査 §「recordingの編集系全般」)。編集意味の先例は自Repo内のSetPositionKeyValue/Interp対とする(既知実装優先: 新形の発明ではなく既存CAS族の鏡映)

## 1. コマンド

```rust
Command::SetPositionKeyTime {
    target: LayerId,
    key: KeyframeId,
    old: RationalTime,
    new: RationalTime,
}
```

- forward: `target` の `envelope.transform.position`(`DocParam::Keyframes`)から `key` を `get_by_id` し、`t == old` を検査(CAS)。`t` を `new` へ移す(remove_by_id→同id/同value/同interpでinsert。昇順不変条件はinsertが維持)
- inverse: `old`/`new` を入れ替えた同型コマンド
- journal: SetPositionKeyValueと同じ層・同じ符号化規約で1 entry。replayはforwardの再適用

## 2. 拒否(typed、暗黙修正なし)

- `target` 不在、position が `Keyframes` でない(空`Keyframes`はadd族の先例どおり`SourceUnsupported`)、`key` 不在
- CAS不一致(`t != old`)
- `new` に**既存の別keyが存在**(同時刻二重keyの禁止はDocKeyframeTrackの型不変条件。clampや自動押し退けをしない)
- `new < RationalTime::ZERO`
- `old == new` はno-op扱いで受理(revision不変)

## 3. wire intent(additive、version 1)

`{"kind":"set_position_key_time","host_handle":..,"target":"<layer u64>","key_id":"<u64>","time":{num,den}}`
`time` = 移動先。`old` はhostが現Documentから読む(呼び手はCAS値を運ばない — SetPositionKeyValueの既存dispatch形と同じ)。

## 4. 入口(通常製品route)

native TimelineのkeyDrag(real行)のrelease時に1回dispatch。drag中はlocal preview、cancelはdown時状態へ復元(既存gesture文法)。値・interp・他key・ID・countは不変。

## 5. 非目標

汎用key API、複数key一括移動、clip範囲外へのclamp意味の新設(移動先はclip範囲内にUI側でclampして送る。範囲外を意味として許すかは別粒)、Curve Editor、BPM snap。
