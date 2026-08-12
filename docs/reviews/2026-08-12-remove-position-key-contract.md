# RemovePositionKey — position keyの削除の閉じた契約

- 日付: 2026-08-12
- 状態: 決定(実装同時着地)
- 系譜: `AddPositionKey`/`UndoAddPositionKey`対(U4b-0)の鏡映。全curve old/new CAS + `stable_id_reservation` の同機構を逆向きに使う。SetPositionKeyTime契約(2026-08-12)に続くU4b族第四の成員

## 1. コマンド対

```rust
Command::RemovePositionKey {
    target: LayerId,
    old_value: DocParam,          // 削除前curve(CAS)
    new_value: DocParam,          // 削除後curve
    removed_key_id: KeyframeId,
    stable_id_reservation: StableIdReservation, // undo再追加で同一KeyframeIdを保証
}
Command::UndoRemovePositionKey { /* 同fieldの逆向き */ }
```

- forward: `old_value` CAS一致を検査し `new_value` へ置換(削除後curve)。残key・ID・interp不変
- inverse: `UndoRemovePositionKey` — 同一 `KeyframeId`/時刻/値/interpで復元(reservation使用)
- journal: Add対と同じ層・同じ符号化規約

## 2. 削除後curveの意味

- key 2個以上: 対象keyを除去するだけ(他keyの時刻・値・interp不変。curve形状は自然に変わる — 通常編集ソフトの削除と同じで、curve保存変換はしない)
- **最後の1個の削除**: `Const(そのkeyの値)` へ収束(AddのConst→1 key昇格の逆向き対称)
- 拒否(typed): target不在、positionがKeyframesでない、key_id不在、CAS不一致

## 3. wire intent(additive)

`{"kind":"remove_position_key","host_handle":..,"target":"<layer u64>","key_id":"<u64>"}`
old/newはhostが現Documentから構成。

## 4. 入口

Timeline viewのDelete/Backspace: real行で**keyが選択中**ならそのkeyを削除、選択keyなしなら従来どおりlayer削除。Undoで完全復元(同一ID)。

## 5. 非目標

複数key一括削除、範囲削除、他property keyへの一般化、Curve Editor。
