//! Vismを二つ目からdataだけで増やせることのfocused oracle。
//!
//! Effects BrowserとInspectorは`known_effects()`を読み、Engineも同じmanifest inventoryを
//! compileする。ここでは個別effect名のRust分岐を足さずに置かれた2本のWGSLが、
//! catalog metadataと同じeffect stackの両方へ届くことを見る。

mod testkit;

use motolii::doc::store::{
    Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, RationalTime,
};
use motolii::render::engine::{known_effects, Engine};

fn document_with_effects(path: &std::path::Path, plugins: &[&str]) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 24,
        height: 24,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 0.0],
    }))
    .unwrap();

    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: path.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 1),
            },
        },
        Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                name: Some("catalog source".to_owned()),
                ..Default::default()
            },
        },
    ])
    .unwrap();

    doc.apply(Intent::SetEffects {
        layer,
        effects: plugins
            .iter()
            .enumerate()
            .map(|(index, plugin_id)| EffectInstance {
                id: EffectId(index as u32),
                plugin_id: (*plugin_id).to_owned(),
            })
            .collect(),
    })
    .unwrap();
    doc
}

fn render(path: &std::path::Path, plugins: &[&str]) -> Vec<u8> {
    let doc = document_with_effects(path, plugins);
    Engine::new()
        .unwrap()
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap()
}

#[test]
fn two_manifest_only_visms_share_the_catalog_and_effect_stack() {
    let catalog = known_effects();
    let gradient = catalog
        .iter()
        .find(|effect| effect.plugin_id == "motolii.gradient")
        .expect("gradient.wgsl must appear in the generated Browser catalog");
    let tri_led = catalog
        .iter()
        .find(|effect| effect.plugin_id == "motolii.tri_led")
        .expect("tri_led.wgsl must appear in the generated Browser catalog");

    assert!(gradient.params.is_empty());
    assert_eq!(
        tri_led
            .params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<Vec<_>>(),
        ["glow"],
        "Inspector rows must come from the second Vism manifest"
    );

    let dir = testkit::tmp_dir("vism-catalog");
    let source = dir.join("source.png");
    let pixels = [32u8, 64, 96, 255]
        .into_iter()
        .cycle()
        .take(24 * 24 * 4)
        .collect::<Vec<_>>();
    image::save_buffer(&source, &pixels, 24, 24, image::ColorType::Rgba8).unwrap();

    let plain = render(&source, &[]);
    let gradient_only = render(&source, &["motolii.gradient"]);
    let tri_led_only = render(&source, &["motolii.tri_led"]);
    let gradient_then_tri_led = render(&source, &["motolii.gradient", "motolii.tri_led"]);
    let tri_led_then_gradient = render(&source, &["motolii.tri_led", "motolii.gradient"]);

    assert_ne!(
        plain, gradient_only,
        "first manifest-only Vism did not render"
    );
    assert_ne!(
        plain, tri_led_only,
        "second manifest-only Vism did not render"
    );
    assert_ne!(
        gradient_only, tri_led_only,
        "two Vism definitions collapsed to one result"
    );
    assert_eq!(
        gradient_then_tri_led, tri_led_only,
        "the second zero-input Vism must replace the first through the common ordered stack"
    );
    assert_eq!(
        tri_led_then_gradient, gradient_only,
        "reversing the data order must reverse the common stack result"
    );
}
