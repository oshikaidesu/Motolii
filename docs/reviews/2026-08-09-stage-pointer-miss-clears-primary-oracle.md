# Stage pointer miss の選択解除を明示oracleへ昇格

日付: 2026-08-09
状態: **決定 / oracle実装済み**

## 0. この文書の扱い

`StageHit::Miss` 時の primary selection の扱いだけを明示oracleへ固定する。
Document schema、公開API、pointer 転送契約、hit test の幾何判定は変更しない。

## 1. 経緯 — 正しい理由で通っていなかったtest

R2 stage geometry 鎖の独立検収(Grok 4.5, ACCEPT / P0=0 / P1=2)のP1-1に対応して、
layer 単位の特異 transform を projection 全体の `Err` から当該 layer の
`StageGeometryUnavailable::SingularTransform` へ変更した。可視 rect が2つあり一方が
`scale=[0,1]` のとき、健全な layer への pointer down が拒否されていたためである。

この変更で `stage_pointer_down_geometry_error_keeps_primary` が落ちた。
原因を測ると、このtestは **miss 経路に到達していなかった**。projection が早期に
`Err` を返すため、`rn_product_host.rs` の

```rust
match hit { ... StageHit::Miss => queue.push_clear_primary() }
```

が動いていなかった。つまり「geometry error だから primary が保たれた」のであって、
「miss で primary が保たれる」ことを示すtestではなかった。

`push_clear_primary()` は鎖が明示的に書いた実装であり、miss を直接扱うtestは
1本も存在しなかった。実装済みだが未固定の挙動だった。

## 2. 決定

**Stage の空き領域への pointer down は primary selection を解除する。**

一般的な編集ソフトの標準挙動であり、鎖の実装意図と一致する。これを2本のtestで固定する。

1. `stage_pointer_down_on_singular_layer_clears_primary`
   — 旧 `..._geometry_error_keeps_primary` を実態へ向け直した。layer 単位の特異は
   `Unavailable` になり hit は Miss へ落ちるため、primary は解除される
2. `stage_pointer_down_miss_clears_primary`(新規)
   — 健全な rect の外側を押すと primary が解除される

`camera_view` 自体が特異な場合は従来どおり `Err` を返し、pointer は拒否される。
この経路は通常の camera 値では到達しないため、testを新設していない。

## 3. UX上の位置づけ

この挙動は利用者に見える。違和感が出た場合は本決定を改訂対象とし、
`StageHit::Miss` の処分を `push_clear_primary()` から no-op へ変える一契約として扱う。
その場合も projection の layer 単位不在化(P1-1対応)は維持する。両者は独立している。

## 4. 非目標

- multi-select、marquee、modifier 併用時の miss の扱い(R2-SELECTION-AUTHORITY 本体)
- Timeline 側の空き領域 click
- `camera_view` 特異時の `LayerId::from_raw(0)` 誤帰属の修正(別 finding)
- 空き領域 click の Document write(write は発生しない。clear は published projection のみ)
