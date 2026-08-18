//! fork seam 2(bind group 床)の**実効** oracle — M-2 outcome 5。
//!
//! [iced fork seam 台帳](../../../docs/reviews/2026-08-18-iced-fork-seam-ledger.md) §4 が
//! 「seam 2 の効きを見る oracle はまだ無い。この穴は M-2 の受け入れ条件に含める」と
//! 明記した、その受け入れ条件である。fork 内の単体テストは「seam が生きている」
//! までしか言わない — ここでは **iced が実際に建てた device** が、床を上げる前は
//! 上流既定の 2 を、上げた後は Rerun の要求以上を**要求・取得**することを測る。
//!
//! 床は process 全体で1つ(上がる一方)なので、この審判は**1つのテスト**の中で
//! 順序を持って行う。他のテストをこの binary に足すときは床に触らないこと。

mod common;

use motolii_shell_iced::stage_island::{
    self, StageIsland, RERUN_MIN_MAX_BIND_GROUPS,
};

fn snapshot_island() {
    let island = StageIsland {
        composition_aspect: None,
        grab_probe: None,
    };
    let mut ui: iced_test::Simulator<'_, motolii_shell_iced::Message> = iced_test::Simulator::with_size(
        iced_test::core::Settings::default(),
        iced::Size::new(160.0, 120.0),
        iced::Element::from(
            iced::widget::shader(island)
                .width(iced::Fill)
                .height(iced::Fill),
        ),
    );
    let _ = ui
        .snapshot(&iced::Theme::Dark)
        .expect("headless snapshot が撮れる");
}

/// 床を上げる前 = 上流既定の 2。上げて建て直す = Rerun の要求(4)以上。
///
/// `observed_max_bind_groups` は shader `Pipeline::new` が **iced のランタイム
/// device から読み戻した** `device.limits().max_bind_groups` の履歴である。
/// 「要求できた」ではなく「取得できた」を見るのがこの oracle の要。
#[test]
fn the_bind_group_floor_reaches_iceds_real_device() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };

    // (1) 床に触っていない process の headless renderer は、上流と同じ 2 を取得する。
    //     ここが 2 でないなら、床が別の誰かに上げられているか上流既定が変わった —
    //     どちらでも「床の効き」を測ったことにならないので、まず言う。
    snapshot_island();
    let observed = stage_island::observed_max_bind_groups();
    assert_eq!(
        observed.first().copied(),
        Some(2),
        "床を上げる前の iced device が上流既定の 2 ではない: {observed:?}"
    );

    // (2) 床を上げてから建て直した device は、Rerun の要求以上を取得する。
    stage_island::install_rerun_device_floor();
    snapshot_island();
    let observed = stage_island::observed_max_bind_groups();
    let raised = observed
        .last()
        .copied()
        .expect("2回目の snapshot でも Pipeline::new は走る");
    assert!(
        raised >= RERUN_MIN_MAX_BIND_GROUPS,
        "床を上げたのに iced の device が {raised} 個しか取得していない\
         (要求 {RERUN_MIN_MAX_BIND_GROUPS})。fork seam 2 が効いていない: {observed:?}"
    );
    // 実測値を残す(`--nocapture` / 証拠採取用)。
    println!(
        "observed max_bind_groups on iced's headless device: before floor = {:?}, after floor = {raised}",
        observed.first()
    );
}

/// [`RERUN_MIN_MAX_BIND_GROUPS`] が `re_renderer` の実要求を下回っていないこと。
///
/// 床の値は窓を建てる前(adapter が無い時点)に決めるので定数だが、定数が
/// Rerun の実要求からずれたら **rev bump でここが落ちて教える**。床には触らない
/// (触ると上のテストの (1) が壊れる)。
#[test]
fn the_floor_constant_covers_what_re_renderer_actually_asks_for() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    let instance = wgpu::Instance::new(re_renderer::device_caps::instance_descriptor(None));
    let adapters =
        iced::futures::executor::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    let adapter = re_renderer::device_caps::select_adapter(&adapters, wgpu::Backends::all(), None)
        .expect("gpu_or_skip を通っている");
    let descriptor = re_renderer::device_caps::DeviceCaps::from_adapter(&adapter)
        .expect("adapter が re_renderer の要求を満たす")
        .device_descriptor();
    assert!(
        descriptor.required_limits.max_bind_groups <= RERUN_MIN_MAX_BIND_GROUPS,
        "re_renderer は {} 個要求するが、床の定数は {} — 定数を上げること",
        descriptor.required_limits.max_bind_groups,
        RERUN_MIN_MAX_BIND_GROUPS
    );
}
