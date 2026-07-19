# Lyric Identity/Reconcile fixture

状態: **非永続prototype**。製品仕様、Document schema、公開API、M3実装ではない。

目的は「夜を走る」から「夜道を走る」へ文字数を変えた時、新しい`道`を親SequenceのAuto/Random Inへ参加させながら、既存の`走`へ付けた配置・大きさ・Timingを別文字へ誤接続せず保持できるかを観察すること。

## 起動

```sh
python3 -m http.server 4179 --directory docs/spikes/lyric-identity-reconcile
```

ブラウザで`http://127.0.0.1:4179/`を開く。

## 自動検査

```sh
node --test docs/spikes/lyric-identity-reconcile/reconcile.test.mjs
```

## このfixtureが保存しないもの

- Motolii Document field
- stable IDの公開型
- override store
- Timeline lane
- physics state

ここで使う`g1`等はprototype内の仮IDである。文字数変更後のidentity挙動を比較するためだけに存在し、採用schema名ではない。

## 物理衝突との関係

文字同士の衝突、落下、押し合いは次段候補。先に「衝突した結果をどの文字へ返すか」が安定していなければ、本文編集後に物理介入が別文字へ移る。したがって本fixtureではRandom Inの初期poseまでを扱い、衝突solver、StateTrack、Bakeは追加しない。
