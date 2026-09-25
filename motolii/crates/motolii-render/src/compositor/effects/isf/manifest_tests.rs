use super::*;

fn manifest(header: &str) -> Result<IsfManifest, IsfError> {
    parse_isf_source(&format!("/*{{ \"INPUTS\": [{{\"NAME\":\"source\",\"TYPE\":\"image\"}}, {{\"NAME\":\"gain\",\"TYPE\":\"float\",\"DEFAULT\":1.0}}]{header} }}*/ fn f() {{}}")).map(|(m, _)| m)
}

/// 複数パスの器: 何段目か(PASSINDEX)と中間 buffer の名前が、shader から見えること。
/// どちらも host 側の束縛は在ったのに宣言が無く、PASSES を書いた効果が通らなかった
/// (2026-09-12、実写の bloom で発覚)。
#[test]
fn a_multi_pass_effect_can_see_its_index_and_its_buffers() {
    let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}], \"PASSES\": [{\"TARGET\":\"half\"}, {}] }*/\n\
                  void main() { gl_FragColor = PASSINDEX == 0 ? IMG_THIS_PIXEL(inputImage) : IMG_THIS_PIXEL(half); }";
    let (_manifest, _vertex, fragment) = compiled_stages(source).expect("PASSES が書ける");
    assert!(fragment.contains("fn main"), "{fragment}");
}

/// 上下の向き: ISF の座標は下端が 0、wgpu の texture は上端が 0。読む macro で裏返す。
/// 裏返さないと実写が上下逆になる(対称な効果では気づけない)。
#[test]
fn the_picture_is_read_right_side_up() {
    let (manifest, body) = parse_isf_source("/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}] }*/ void main() {}").unwrap();
    let (images, params) = crate::render::compositor::effects::vism::orders(&manifest);
    let glsl = wrap_fragment_source(&manifest, &images, &params, &body);
    assert!(glsl.contains("1.0 - isf_FragNormCoord.y"), "{glsl}");
}

/// TIME_OFFSET が名指す欄が無い(か float でない)なら、黙って 0 にせず名前を挙げて断る。
#[test]
fn a_time_offset_naming_a_missing_field_is_refused() {
    let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}, {\"NAME\":\"past\",\"TYPE\":\"image\",\"TIME_OFFSET\":\"nope\"}] }*/ void main() {}";
    let error = parse_isf_source(source).err().expect("断る").to_string();
    assert!(error.contains("nope") && error.contains("past"), "{error}");
}

/// 隣のコマは `TIME_OFFSET_FRAMES`(コマ数)で読む。時刻の読み方は 1 つの image に 1 つだけ。
#[test]
fn a_time_offset_in_frames_is_one_of_the_three_time_bases() {
    let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}, {\"NAME\":\"previous\",\"TYPE\":\"image\",\"TIME_OFFSET_FRAMES\":-1}] }*/ void main() {}";
    let previous = &parse_isf_source(source).unwrap().0.inputs[1];
    assert_eq!((previous.time_offset.clone(), previous.time_base), (Some(TimeOffset::Fixed(-1.0)), TimeBase::Frames));
    let both = source.replace("\"TIME_OFFSET_FRAMES\":-1", "\"TIME_OFFSET_FRAMES\":-1,\"TIME_AT\":0");
    let error = parse_isf_source(&both).err().expect("断る").to_string();
    assert!(error.contains("previous") && error.contains("1 つだけ"), "{error}");
}

/// 時計は ISF の綴り(TIME / TIMEDELTA / FRAMEINDEX / DATE)で、ホストの uniform として届く。
/// 読む効果だけ `uses_clock` が立つ(注釈の中の語や TIMER のような別名では立たない)。
#[test]
fn the_clock_is_declared_and_only_readers_are_marked() {
    let reader = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}] }*/\nvoid main() { gl_FragColor = vec4(sin(TIME), float(FRAMEINDEX), TIMEDELTA, 1.0) + DATE; }";
    let (manifest, _v, fragment) = compiled_stages(reader).expect("時計が読める");
    assert!(manifest.uses_clock);
    assert!(fragment.contains("fn main"), "{fragment}");
    let quiet = "/*{ \"INPUTS\": [] }*/\n// TIME is only mentioned here\nfloat TIMER = 1.0; void main() { gl_FragColor = vec4(TIMER); }";
    assert!(!parse_isf_source(quiet).unwrap().0.uses_clock);
}

/// point3D は 3 成分の欄。窓には x,y だけが出て z は既定のまま(取説に明記)。
#[test]
fn a_point3d_field_has_three_components() {
    let source = "/*{ \"INPUTS\": [{\"NAME\":\"p\",\"TYPE\":\"point3D\",\"DEFAULT\":[1.0,2.0,3.0]}] }*/ void main() { gl_FragColor = vec4(p, 1.0); }";
    let (manifest, _v, fragment) = compiled_stages(source).expect("point3D が通る");
    let p = &manifest.inputs[0];
    assert_eq!((p.ty, p.ty.component_count()), (IsfInputType::Point3D, 3));
    assert_eq!(&p.default[..3], &[1.0, 2.0, 3.0]);
    assert!(fragment.contains("fn main"), "{fragment}");
}

/// image の LAYER が名指す欄が無い(か layer でない)なら、名前を挙げて断る。
#[test]
fn an_image_naming_a_missing_layer_field_is_refused() {
    let source = "/*{ \"INPUTS\": [{\"NAME\":\"inputImage\",\"TYPE\":\"image\"}, {\"NAME\":\"matte\",\"TYPE\":\"image\",\"LAYER\":\"nope\"}] }*/ void main() {}";
    let error = parse_isf_source(source).err().expect("断る").to_string();
    assert!(error.contains("nope") && error.contains("matte"), "{error}");
}

/// VIEW_BLUR names the float input whose roughness bounds the Views' mips (BACKDROP_BLUR's
/// shape). A name that is not a float input of the surface is refused by name.
#[test]
fn a_view_blur_naming_a_missing_field_is_refused() {
    let surface = |extra: &str| parse_isf_source(&format!("/*{{ \"STAGE\": \"surface\", \"INPUTS\": [{{\"NAME\":\"roughness\",\"TYPE\":\"float\",\"DEFAULT\":0.1}}]{extra} }}*/ fn surface() {{}}"));
    assert_eq!(surface(", \"VIEW_BLUR\": \"roughness\"").unwrap().0.view_blur_input.as_deref(), Some("roughness"));
    assert_eq!(surface("").unwrap().0.view_blur_input, None);
    let error = surface(", \"VIEW_BLUR\": \"nope\"").err().expect("refused").to_string();
    assert!(error.contains("VIEW_BLUR") && error.contains("nope"), "{error}");
}

#[test]
fn constant_pass_dimensions_follow_isf_target_declarations() {
    let m = manifest(r#", "PASSES": [{"TARGET":"summary","WIDTH":"64","HEIGHT":"8","FLOAT":true},{}]"#).unwrap();
    assert_eq!((m.passes[0].width, m.passes[0].height), (Some(IsfDimension::Fixed(64)), Some(IsfDimension::Fixed(8))));
    assert_eq!((m.passes[1].width, m.passes[1].height), (None, None));
    let m = manifest(r#", "PASSES": [{"TARGET":"half","WIDTH":"$WIDTH/2","HEIGHT":"$HEIGHT/2"}]"#).unwrap();
    assert_eq!(m.passes[0].width.unwrap().resolve(97), 49);
    for value in ["0", "-1", "1.5", "16385", "\"$WIDTH/0\""] {
        assert!(manifest(&format!(", \"PASSES\": [{{\"TARGET\":\"summary\",\"WIDTH\":{value}}}]")).is_err());
    }
}
