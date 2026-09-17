// 証明の 1 枚: Web の人が語彙を全部知っている(Flex・重力・stagger)のに、見た事が無い画。
// 歌詞の 1 行の語が箱(Display: Flex)に並び、箱に立った場(Field: 重力 6 割 + 底の真ん中へ寄る 4 割)で行から落ち、底に積もって眠る。
// 順番の札(Stagger)が語ごとに落ちる時刻をずらす。宣言は 3 つ、鍵は 0。
// (Split: Words の単位は今は物にならないので、語は層で書く — 束 1 の画廊に理由)
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#101014" });
const words = "きみの こえが よるを こえて ひかる".split(" ").map((w) => text(w, { name: w }).fill("#F2EDE4").font("Hiragino Sans").set("Size", 78));
const g = ellipse({ name: "Gravity" }).fill("#10101400").set("Scale", [2 / 270, 2 / 270]);
const room = group(...words, g).name("Room");
room.set("Display", "Flex").set("Gap", 40).set("Padding", [60, 80]).set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Width", 1820).set("Height", 880)
  .set("Stagger", 1.0).set("Background", "#17171D").set("Border Radius", 24).set("Position", [50, 100]);
words.forEach((w) => w.set("Position", [0, 0]));
g.set("Position Type", "Absolute").set("Position", [910, 800]).effect("Field", { "Spread": 0.6, "Angle": 90, "Strength": 1500 });
