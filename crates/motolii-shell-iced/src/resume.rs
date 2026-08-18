//! 起動時の座席決定 — `--project` と引数なし起動の「続きが開く」を1つに合流させる。
//!
//! 判断そのものは動かさない: 覚えている project の読み書きと3択(`Resume`)は
//! `motolii_ui::blitz_shell`(egui shell と**同じ関数**)をそのまま呼ぶ
//! (2026-08-19 M-2: `resume_last_project` / `ProjectSeat::open` を2つ目に
//! 実装しない。egui 側 wave D の `crate::last_project` を共用する)。
//! ここが持つのは「`--project` が来ていたらそちらを優先する」の合流点だけである。
//!
//! `--project` が開けない場合の帰結は egui 版の起動失敗
//! (`run_blitz_shell` の `Err` → プロセスがそもそも起動しない)とは**意図的に違う**:
//! iced はここで**窓を開いてから**理由を帯へ出す(`Resume::Explained`)。
//! 「開けなかった `--project` で窓ごと落ちる」を作らない — 覚えていた project が
//! 消えていた時と同じ言い方に揃えてある。

use std::path::Path;

use motolii_ui::blitz_shell::{resume_last_project, ProjectSeat, Resume};

use crate::launch::Launch;

/// 起動要求から座席の決め方を1つに合流させる。
///
/// - `launch.project`(`--project <path>`)が在れば、それを開こうとする。
///   開けたら [`Resume::Seated`]、開けなければ理由付きでスタート画面
///   ([`Resume::Explained`])。`resume_last_project` が「覚えていた project が
///   消えていた」時にやっている言い方と揃えてある(次にどうすればよいかまで言う)。
/// - `launch.project` が無ければ [`resume_last_project`] の3択(F-01。
///   引数なし起動の「続きが開く」)へそのまま委ねる。
///
/// `--fixture` 相当(この crate にはまだ無い)は、呼び手が `launch.project` を
/// 素通しにする前に fixture 分岐へ抜けるので、ここへは来ない。
pub fn decide_resume(launch: &Launch, last_project_store: Option<&Path>) -> Resume {
    match &launch.project {
        Some(path) => match ProjectSeat::open(path) {
            Ok(seat) => Resume::Seated(seat),
            // `ProjectSeat::open` の Err は既に path を名指ししている
            // (`resume_last_project` の `Explained` と同じ言い方に揃える)。
            Err(error) => Resume::Explained(format!("{error} — Cmd+N で作るか Cmd+O で開く")),
        },
        None => resume_last_project(last_project_store),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_ui::blitz_shell::create_project_file;

    /// `--project` が開けるなら、覚えていた project があってもそちらを優先する。
    #[test]
    fn the_project_flag_wins_over_the_remembered_project() {
        let dir = motolii_testkit::tmp_dir("resume_decide_priority");
        let store = dir.join("last-project.json");
        let remembered = dir.join("remembered.json");
        let explicit = dir.join("explicit.json");
        create_project_file(&remembered).expect("remembered project を作る");
        create_project_file(&explicit).expect("explicit project を作る");
        motolii_ui::remember_last_project(&store, &remembered).expect("覚える");

        let launch = Launch {
            project: Some(explicit.clone()),
            ..Launch::default()
        };
        match decide_resume(&launch, Some(&store)) {
            Resume::Seated(seat) => {
                assert_eq!(seat.path(), explicit.as_path());
            }
            _ => panic!("--project が開けるのに座っていない"),
        }
    }

    /// `--project` が開けなければ理由が付き、`Nothing` にも `Seated` にもならない。
    #[test]
    fn an_unopenable_project_flag_is_explained() {
        let dir = motolii_testkit::tmp_dir("resume_decide_unopenable");
        let missing = dir.join("missing.json");

        let launch = Launch {
            project: Some(missing.clone()),
            ..Launch::default()
        };
        match decide_resume(&launch, None) {
            Resume::Explained(reason) => {
                assert!(reason.contains("missing.json"), "{reason}");
            }
            _ => panic!("開けない --project は理由付きの Explained のはず"),
        }
    }
}
