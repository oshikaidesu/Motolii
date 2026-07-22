# UIの音楽メタファー撤回（2026-07-22）

状態: **撤回**

対象: [UIコンセプト](../ui-concept.md)、[時間面UI構成モデル](../ui-score-model.md)、[concept.md](../concept.md)

## 撤回するもの

Motolii全体を「1曲を演奏する譜面台」、Timelineを「譜面」、初回体験を「First Beat」、製品の時間構造を「楽曲が背骨」と呼ぶ仮説を撤回する。

これは音声機能の削除ではない。MVを初期の完成条件とし、Soundtrack、BPM、拍grid、音声同期を提供する決定は維持する。ただし、それらは作品によって使う具体機能であって、Motolii、Vism、plugin、UI全体を定義する存在論ではない。Soundtrackが未設定でも、制作・preview・時間編集が自然に成立しなければならない。

## 理由

- 「演奏」「譜面」は説明用の比喩だったが、`concept.md`、UIの北極星、Timeline構成名へ広がり、実際の製品意味に見える状態になった
- 現行のMotoliiは、素材、生成、合成、keyframe、plugin、3D、previewを扱うコンポジット／モーショングラフィック環境であり、音楽は必須の操作主体ではない
- creatorを開発者として迎え、p5.jsやBlenderのように表現世界そのものを拡張する長期像は、音楽メタファーより「小さな実行可能表現を作り、組み、共有する」という構造で説明する方が正確である
- 「映像制作におけるVST」はHostと拡張単位の分離を説明する**アーキテクチャ上の類比**に限定できる。DAWのUIや音楽中心の制作順まで採る根拠にはしない

## 現行の中立語彙

| 旧語 | 現行語 | 意味 |
|---|---|---|
| 演奏する | 実行する／編集する／組み合わせる | 文脈に応じて具体動詞を使う |
| 譜面・譜面台 | 時間面・制作面 | Documentの時間投影または製品UI全体 |
| First Beat | 最初の結果 | 初回操作から意図した画がpreviewへ現れるまで |
| 楽曲が背骨 | 同じ時間と結果が見える | Soundtrackの有無に依存しないUI原則 |

過去の先例調査や履歴文書に現れる語は、当時の仮説を示す記録として残してよい。現行仕様、入口文書、製品UI名、発注書では上表の語を使う。

## 維持する判断

- MVを初期の完成条件とする
- Soundtrackは作品の音声基準、BPM／拍gridは任意の時間guideとして使える
- Timelineは固定Track／LaneをDocument所有者にせず、一枚の時間面へ投影する
- 高密度、既知の外殻、可視の因果、意図語彙、軽さをUI原則とする
- Vism、Kit、plugin authoringによって、creatorがHost全体をforkせず新しい表現を追加できる方向を維持する

## 拒否する逆流

- Soundtrack未設定時も空の楽曲領域を常設し、制作順を音楽起点へ強制する
- `Score`、`Beat`、`Performance`等を新しいDocument型、公開API、plugin capabilityの総称へする
- VST類比を理由にDAWのTrack、Mixer、instrument ownershipをMotoliiへ持ち込む
- この用語整理を理由にSoundtrack/BPMの既存意味、永続形式、task完了条件を黙って変更する

本撤回は文書語彙と体験仮説の訂正であり、コード、Document schema、公開API、既存の音声機能を変更しない。
