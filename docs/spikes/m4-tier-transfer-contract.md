# M4 hierarchy transfer contract

状態: **VALIDATION PASS / K1b・K1c未着手**

## 目的

ResourceLedgerのhard capを使い、VRAM→RAM→disk階層を実装する前に、転送失敗、
同時要求、cancel、旧generationで失ってはいけない性質をtest-only modelで固定する。

## 固定した負例

1. destination admission拒否時はsource residentとその会計を維持する
2. copy中はsourceとdestinationを両方計上し、成功commit後だけsourceを解放する
3. copy失敗/abortはdestinationだけを解放してsourceへ戻る
4. UMA共有capは転送中のVRAM+RAM二重常駐も合算して拒否する
5. LRUはin-useとpinnedを飛ばし、候補がなければ型付き全pin拒否へ進む
6. 同じkey/generationの需要は一つのjobと一つのreservationだけを持つ
7. generation更新後に完了した旧jobは結果を登録せずreservationを解放する
8. cancelはin-flight reservationを解放し、結果を生成しない

`crates/motolii-testkit/tests/m4_tier_transfer_contract.rs`は製品へimportしない契約modelである。
転送本体、copy completion、disk IO、cache key、LRU storeを実装した証拠にはしない。

## 設計への帰結

- 降格はsourceを先にDropして空きを作る操作ではない。destinationを確保できなければ元を維持する
- dGPUでもcopy中は二層分、UMAでは共有物理memory上の二重常駐としてhard capへ入れる
- cache lookupのsingle-flight lockとResourceLedgerの会計Mutexを同一lockへしない
- stale generation判定は画素登録前に行い、古い結果を一瞬でも表示/cache登録しない
- cancellationはdeadline制御であり、hard budget超過をframe dropだけで解決しない

K1bの参照handle・generation・single-flight storeが成立するまでK1c製品実装へ進まない。
