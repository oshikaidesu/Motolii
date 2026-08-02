//! M4-P13-C1: vello_svgと現行Velloのsubset／unsupported診断を確認する。

#[test]
fn supported_path_group_fill_and_stroke_build_a_scene() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32">
          <g transform="translate(2 3)">
            <path d="M 0 0 L 20 0 L 20 20 Z" fill="#ff0000" stroke="#000000" stroke-width="2"/>
          </g>
        </svg>
    "##;
    let scene = vello_svg::render(svg).expect("supported path subset should parse");
    assert!(!scene.encoding().is_empty());
}

#[test]
fn unsupported_pattern_reports_a_node_without_custom_parser() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16">
          <defs><pattern id="p" width="4" height="4"><rect width="2" height="2" fill="#000000"/></pattern></defs>
          <rect width="16" height="16" fill="url(#p)"/>
        </svg>
    "##;
    let mut scene = vello_svg::vello::Scene::new();
    let mut unsupported = 0usize;
    vello_svg::append_with(&mut scene, svg, &mut |_scene, _node| unsupported += 1)
        .expect("unsupported subset should still parse");
    assert_eq!(unsupported, 1, "pattern must become an explicit diagnostic");
}

#[test]
fn malformed_svg_is_a_typed_error() {
    let result = vello_svg::render("<svg><path></svg>");
    assert!(matches!(result, Err(vello_svg::Error::Svg(_))));
}

#[test]
fn external_file_resource_is_not_rejected_by_the_library_boundary() {
    let svg = r##"
        <svg xmlns="http://www.w3.org/2000/svg" width="8" height="8">
          <image href="file:///definitely-not-a-motolii-resource.png" width="8" height="8"/>
        </svg>
    "##;
    let mut scene = vello_svg::vello::Scene::new();
    let mut unsupported = 0usize;
    let result = vello_svg::append_with(&mut scene, svg, &mut |_scene, _node| unsupported += 1);
    assert!(result.is_ok());
    assert_eq!(
        unsupported, 0,
        "usvg drops the missing external image before vello_svg sees it"
    );
    assert!(scene.encoding().is_empty());
}
