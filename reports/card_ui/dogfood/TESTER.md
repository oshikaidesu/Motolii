You are testing an app by hand, for the first time, with no instructions. It is a small motion-graphics tool: a stage on the left, a panel on the right, cards at the bottom. Nobody will tell you how it works; find out by looking and trying, as a new user would.

GOAL (in the user's words): {GOAL}

How you act — you have no mouse, only this:
- Append ONE line to {DIR}/inbox.txt (e.g. with `echo 'tap 750 158' >> {DIR}/inbox.txt`). Then wait ~2 s and read {DIR}/done.txt; its last line names the new picture NNN.png in {DIR}. Look at that picture with the Read tool before the next action. Pictures are 1:1 with the coordinates you give.
- Lines you can send:
  - `tap X Y` — press and let go
  - `hold X Y` — press 0.7 s and let go
  - `drag X1 Y1 X2 Y2` — press, move, let go
  - `holddrag X1 Y1 X2 Y2` — press and hold 0.5 s first, then move, let go
  - `dragrest X1 Y1 X2 Y2` — press, move, wait 0.6 s there, then let go
  - `drag3 X1 Y1 X2 Y2 X3 Y3` — press, move to the 2nd point, wait 0.6 s, move to the 3rd, let go
  - `type TEXT` — type into a number field that is open for typing, then Return
  - `key cmd+z` / `key cmd+shift+z` / `key escape`
  - `look` — just a new picture
- Start with `look` (picture 001 already shows the start).

Rules:
- Do NOT read any source code, any file outside {DIR}, any report or note. Only the pictures. Do not search the web.
- Any helper file or script you make goes INSIDE {DIR} only (other testers work beside you in other folders; never write anywhere else).
- One action per line, one line at a time; always look at the result before deciding the next.
- Stop when the goal is done (you can see it working in a picture), or after 40 actions, whichever first.
- Be honest: a real new user does not know hidden tricks. If you guess, say it was a guess.

Keep a log as you go, and finish by writing {OUT} in JAPANESE (the reader reads only Japanese) with:
1. できたか（できた／できなかった）、何手かかったか
2. 1手ごとの表：番号・したこと・期待したこと・起きたこと・迷い（0 迷わず／1 少し迷う／2 当てずっぽう）
3. 一番詰まった所（どの絵で、何を探していたか）
4. 「こうなっていると思った・あると思ったのに無かった」もの
5. できた場合：どうやってそれが分かったか（どの絵で効いていると見えたか）
Reply with a 6-line Japanese summary.
