//! M3実装ガード1 / Issue #56: 日本語IME受け入れスパイク骨格。
//!
//! Slint 組み込み `TextInput` でチェックリスト4項目を人手審判する。合否は `docs/spikes/ime-acceptance.md` に記録。
//!
//! 実行 (開発主機・GUI必須):
//!   `cargo run`
//!   `IME_ACCEPTANCE_MANIFEST=../../docs/spikes/ime-acceptance-evidence/manifest.json cargo run`
//!
//! 静的検査のみ (ヘッドレス可):
//!   `cargo test`

use std::path::PathBuf;

use ime_acceptance::{AcceptanceManifest, ChecklistId};

slint::slint! {
    import { VerticalBox, HorizontalBox, ScrollView } from "std-widgets.slint";

    export component ImeSpikeWindow inherits Window {
        title: "IME受け入れスパイク (M3ガード1 / #56)";
        preferred-width: 720px;
        preferred-height: 640px;

        in-out property <string> lyric-text;
        in-out property <string> shortcut-log;

        VerticalBox {
            padding: 12px;
            spacing: 8px;

            Text {
                text: "チェックリスト駆動 — 各項目を人手で確認し docs/spikes/ime-acceptance.md に記録";
                wrap: word-wrap;
                font-size: 13px;
            }

            Text {
                text: "① preedit下線  ② 候補追従  ③ Enter未食い  ④ 長文歌詞";
                font-size: 12px;
                color: #888;
            }

            Text { text: "長文歌詞 / 連続入力 (TextInput):"; font-size: 13px; }

            Rectangle {
                min-height: 160px;
                border-width: 1px;
                border-color: #404048;
                border-radius: 4px;

                lyric-input := TextInput {
                    text <=> root.lyric-text;
                    single-line: false;
                    wrap: word-wrap;
                    font-size: 14px;
                    width: parent.width;
                    height: parent.height;
                    key-pressed(event) => {
                        if (event.text == "\n") {
                            root.shortcut-log = root.shortcut-log + "[Enter shortcut]\n";
                            return accept;
                        }
                        return reject;
                    }
                }
            }

            Text { text: "ショートカット発火ログ (③: 変換中Enterで増えたら不合格):"; font-size: 13px; }
            Rectangle {
                min-height: 48px;
                background: #1a1a22;
                border-radius: 4px;
                Text {
                    text: root.shortcut-log;
                    color: #c0c0c8;
                    font-size: 12px;
                    wrap: word-wrap;
                }
            }

            ScrollView {
                min-height: 180px;
                VerticalBox {
                    spacing: 6px;
                    for item in [
                        "① preedit下線: TextInputでローマ字→変換前の未確定表示",
                        "② 候補追従: カーソル移動で候補ウィンドウが追従するか",
                        "③ Enter未食い: 未確定のままEnter — 上ログに出なければOK",
                        "④ 長文歌詞: 下欄へ長文を貼付/連続入力 — 欠落・化けなし",
                    ]: Text {
                        text: item;
                        font-size: 12px;
                        wrap: word-wrap;
                    }
                }
            }

            HorizontalBox {
                Text {
                    text: "対象: Win MS-IME / macOS / Linux(fcftx5+Wayland, ibus+X11)";
                    font-size: 11px;
                    color: #666;
                }
            }
        }
    }
}

/// 長文歌詞サンプル — ④の手動試験用 (仕様どおり歌詞想定)
pub const LONG_LYRIC_SAMPLE: &str = "\
夜明けの空に浮かぶ雲の隙間から\n\
金色の光が差し込む朝の静けさ\n\
窓を叩く風の音 遠くで鳴く電車\n\
目を覚ます街の鼓動 今日も始まる\n\
\n\
記憶の断片を繋ぎ合わせながら\n\
綴った言葉はまだ未完成のまま\n\
変換を重ねても消えない想い\n\
この歌詞が届くまで走り続ける\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = ImeSpikeWindow::new()?;
    app.set_lyric_text(LONG_LYRIC_SAMPLE.into());
    app.set_shortcut_log("".into());

    if let Ok(path) = std::env::var("IME_ACCEPTANCE_MANIFEST") {
        write_skeleton_manifest(PathBuf::from(path))?;
    }

    for id in ChecklistId::ALL {
        eprintln!("checklist {}: {}", format!("{id:?}"), id.manual_steps());
    }

    app.run()?;
    Ok(())
}

fn write_skeleton_manifest(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let manifest = AcceptanceManifest::skeleton_template();
    std::fs::write(&path, serde_json::to_string_pretty(&manifest)?)?;
    eprintln!("wrote skeleton manifest: {}", path.display());
    Ok(())
}
