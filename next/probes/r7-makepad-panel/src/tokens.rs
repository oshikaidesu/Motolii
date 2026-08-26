use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*

    // 面の境の目盛り。太さは1つしか無い — 「1px」と書き散らすと、あとで
    // 太さを変えたい時に全箇所を grep する羽目になる(Tailwind の scale と同じ考え)。
    // 色は「何の境か」で分ける。線は操作の持ち主か座標系が変わる所にだけ引く。
    let rules = {
        size: 1.0
        // 持ち主が変わる境(同じ面の中の群 — Collections / Library / Places)
        owner: #x434343
        // 面と面の継ぎ目
        seam: #x2d2d2d
        // pane の境(rail とリストのように座標系が変わる)
        pane: #x343434
        // 窓の大きな面の境
        surface: #x1d1d1d
    }
    mod.tokens = { rules: rules }
}
