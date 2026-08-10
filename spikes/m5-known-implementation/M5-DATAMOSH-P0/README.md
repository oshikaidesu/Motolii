# M5-DATAMOSH-P0 codec-domain proof

製品workspace外のprivate fixture。FFmpeg標準`noise` bitstream filterのpacket dropだけを使い、
固定MPEG-4 Part 2 / MP4入力の二つ目のkey packetを除去する。後続P-frameの参照欠落によるdecode結果の変化を確認し、
任意byte改変や独自codec parserは持たない。

```sh
./spikes/m5-known-implementation/M5-DATAMOSH-P0/probe.sh
```

oracleは入力不変、同一入力／recipe／toolでのbyte-identical出力、19 packetの再生可能MP4、PTS 1.1のdecode差を固定する。
`FFMPEG_BIN`または`FFPROBE_BIN`が無ければ`DATAMOSH_TOOL_MISSING`で型付きに停止する。
