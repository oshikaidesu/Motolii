//! P11: DOM経路の入力は本当に「タダで付いてくる」のか。
//!
//! P10 で密度の天井は出た。残った問いは入力側で、主張はこうだった —
//! 「DOMならhit-test・hover・イベント配送を自前で書かなくていい」。
//! それを、書き直したモックではなく **実物の timeline-library.html** の上で確かめる。
//!
//! 確かめること:
//!   1. clip の中心を突いて、その clip が hit target になるか
//!   2. CSSで7pxしか無い trim ハンドルを、座標計算なしに撃ち分けられるか
//!   3. hover の出入り(pointerleave/pointerenter)が自動で出るか
//!   4. down → move → up で pointerdown/pointerup/click が正しい相手へ届くか
//!   5. その move に合わせて DOM を書き換えると、clip が実際に動くか(結果まで見る)
//!
//! 窓は開けない。全部ヘッドレスで、同じ答えが何度でも出る形にする。
//!
//! 使い方: mock_input <input.html> [W] [H]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use blitz_dom::{Document, DocumentConfig, EventDriver, EventHandler, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, DomEvent, DomEventData, EventState, MouseEventButton,
    MouseEventButtons, Point, PointerCoords, PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use keyboard_types::Modifiers;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: mock_input <input.html> [W] [H]");
        std::process::exit(2);
    }
    let html = std::fs::read_to_string(&args[1]).expect("input html");
    let w: u32 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(1280);
    let h: u32 = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(500);

    let mut doc = HtmlDocument::from_html(
        &html,
        DocumentConfig {
            style_threading: StyleThreading::Sequential,
            ..Default::default()
        },
    );
    doc.set_viewport(Viewport {
        window_size: (w, h),
        hidpi_scale: 1.0,
        zoom: 1.0,
        color_scheme: ColorScheme::Dark,
    });
    doc.resolve(0.0);
    // hit() は stacking context を見る。1回目のresolveでは z-index を持つ
    // 絶対配置がまだ hoist されておらず、親が当たる。2回目で揃う。
    if std::env::var("P11_SINGLE_RESOLVE").is_err() {
        doc.resolve(0.0);
    }

    let bars = find_class(&doc, "objectBar");
    println!("== 面の中身 ==");
    println!("clip(.objectBar) {} 個", bars.len());

    // 見えている(幅のある)barだけを対象にする
    let mut visible: Vec<(usize, String, f32, f32, f32, f32)> = Vec::new();
    for id in &bars {
        let (x, y) = abs_pos(&doc, *id);
        let node = doc.get_node(*id).unwrap();
        let s = node.final_layout.size;
        if s.width < 1.0 || s.height < 1.0 {
            continue;
        }
        let owner = attr(&doc, *id, "data-owner").unwrap_or_else(|| "-".into());
        visible.push((*id, owner, x, y, s.width, s.height));
    }
    for (id, owner, x, y, cw, ch) in &visible {
        println!("  node {id:>4} owner={owner:<16} rect=({x:.0},{y:.0}) {cw:.0}x{ch:.0}");
    }
    assert!(!visible.is_empty(), "見えているclipが無い");

    // ---- 1. clip中心のhit ----
    println!("\n== 1. clip中心のhit-test ==");
    let mut ok = 0;
    for (id, owner, x, y, cw, ch) in &visible {
        let (px, py) = (x + cw / 2.0, y + ch / 2.0);
        match doc.hit(px, py) {
            Some(hit) => {
                let inside = chain_contains(&doc, hit.node_id, *id);
                if inside {
                    ok += 1;
                }
                println!(
                    "  ({px:.0},{py:.0}) → node {} [{}] {} owner={owner}",
                    hit.node_id,
                    class_of(&doc, hit.node_id),
                    if inside { "命中" } else { "★別物" },
                );
            }
            None => println!("  ({px:.0},{py:.0}) → hitなし ★ owner={owner}"),
        }
    }
    println!("  {ok}/{} が自分のclipに当たった", visible.len());

    // ---- 2. trimハンドル(CSSで7px)の撃ち分け ----
    println!("\n== 2. trimハンドルの撃ち分け(座標計算を書かずに) ==");
    let (bid, owner, bx, by, bw, bh) = visible.last().cloned().unwrap();
    let _ = bid;
    for (label, px) in [
        ("左端+3px", bx + 3.0),
        ("中央", bx + bw / 2.0),
        ("右端-3px", bx + bw - 3.0),
    ] {
        let py = by + bh / 2.0;
        match doc.hit(px, py) {
            Some(hit) => println!(
                "  {label:<10} ({px:.0},{py:.0}) → [{}]",
                class_of(&doc, hit.node_id)
            ),
            None => println!("  {label:<10} → hitなし"),
        }
    }
    println!("  (対象 owner={owner})");

    // ---- 3〜4. hoverの出入りと、down→move→upの配送 ----
    let log = Rc::new(RefCell::new(Vec::<String>::new()));

    println!("\n== 3. hoverの出入り ==");
    let (_, o_a, ax, ay, aw, ah) = visible[0].clone();
    let (_, o_b, bx2, by2, bw2, bh2) = visible[visible.len() - 1].clone();
    send(&mut doc, &log, UiEvent::PointerMove(pointer_at(ax + aw / 2.0, ay + ah / 2.0)));
    println!("  {o_a} の上へ: {}", drain(&log));
    send(&mut doc, &log, UiEvent::PointerMove(pointer_at(bx2 + bw2 / 2.0, by2 + bh2 / 2.0)));
    println!("  {o_b} へ移動: {}", drain(&log));

    println!("\n== 4〜5. 掴んで動かす(DOMも一緒に書き換える) ==");
    // 動かす対象は Background(左が0%から始まるので変化が読みやすい)
    let target = visible
        .iter()
        .find(|(_, o, ..)| o == "background")
        .cloned()
        .unwrap_or_else(|| visible[0].clone());
    let (tid, towner, tx, ty, tw, th) = target;
    let start_x = tx + tw / 2.0;
    let start_y = ty + th / 2.0;
    println!("  対象 owner={towner} 初期rect=({tx:.0},{ty:.0}) {tw:.0}x{th:.0}");

    send(&mut doc, &log, UiEvent::PointerDown(pointer_at(start_x, start_y)));
    println!("  down: {}", drain(&log));

    // 面の幅(=rowTrackの幅)を基準に%を作り直す。JSがやっていたことをRustでやる。
    let track_w = doc
        .get_node(doc.get_node(tid).unwrap().parent.unwrap())
        .unwrap()
        .final_layout
        .size
        .width as f64;
    // bar の left は rowTrack 基準。文書座標(tx)ではなく親の中での位置を使う。
    let start_left_pct =
        (doc.get_node(tid).unwrap().final_layout.location.x as f64 / track_w) * 100.0;

    let mut moved_to = start_x;
    for step in 1..=5 {
        let dx = 40.0 * step as f32;
        moved_to = start_x + dx;
        let pct = start_left_pct + (dx as f64 / track_w) * 100.0;
        {
            let mut m = doc.mutate();
            // P11_VAR=1 でモックと同じ custom property 経由に切り替える。
            // 既定は left/width を直接書く(JSがやっていた最終値をそのまま置く)。
            let style = if std::env::var("P11_VAR").is_ok() {
                format!("--left:{pct:.3}%;--width:{:.3}%", (tw as f64 / track_w) * 100.0)
            } else {
                format!(
                    "left:{pct:.3}%;width:{:.3}%",
                    (tw as f64 / track_w) * 100.0
                )
            };
            m.set_attribute(tid, blitz_dom::qual_name!("style"), &style);
        }
        // 時刻を進める。`transition` を持つプロパティは、同じ時刻で resolve し続けると
        // 開始値のまま止まる(見かけ上「書き換えが効かない」)。
        doc.resolve(step as f64 * 100.0);
        send(&mut doc, &log, UiEvent::PointerMove(pointer_at(moved_to, start_y)));
        log.borrow_mut().clear();
    }
    send(&mut doc, &log, UiEvent::PointerUp(pointer_at(moved_to, start_y)));
    println!("  up: {}", drain(&log));

    doc.resolve(0.0);
    let (nx, ny) = abs_pos(&doc, tid);
    let ns = doc.get_node(tid).unwrap().final_layout.size;
    println!(
        "  移動後rect=({nx:.0},{ny:.0}) {:.0}x{:.0}  Δx={:.0}px (指示は+200px)",
        ns.width,
        ns.height,
        nx - tx
    );

    // 動いた先で hit が付いてくるか = 掴み直しが効くか
    match doc.hit(nx + ns.width / 2.0, ny + ns.height / 2.0) {
        Some(hit) => println!(
            "  移動後の中心をhit: node {} [{}] {}",
            hit.node_id,
            class_of(&doc, hit.node_id),
            if chain_contains(&doc, hit.node_id, tid) {
                "命中"
            } else {
                "★別物"
            }
        ),
        None => println!("  移動後の中心: hitなし ★"),
    }
}

/// イベントを1つ流し、届いたDomEventを記録する
fn send(doc: &mut HtmlDocument, log: &Rc<RefCell<Vec<String>>>, event: UiEvent) {
    let names: Rc<RefCell<Vec<(String, usize)>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let mut driver = EventDriver::new(doc as &mut dyn Document, Recorder(names.clone()));
        driver.handle_ui_event(event);
    }
    let taken = names.borrow().clone();
    for (name, target) in taken {
        let cls = class_of(doc, target);
        log.borrow_mut().push(format!("{name}→[{cls}]"));
    }
}

fn drain(log: &Rc<RefCell<Vec<String>>>) -> String {
    let mut l = log.borrow_mut();
    if l.is_empty() {
        return "(イベントなし)".into();
    }
    let s = l.join("  ");
    l.clear();
    s
}

struct Recorder(Rc<RefCell<Vec<(String, usize)>>>);

impl EventHandler for Recorder {
    fn handle_event(
        &mut self,
        _chain: &[usize],
        event: &mut DomEvent,
        _doc: &mut dyn Document,
        _state: &mut EventState,
    ) {
        let name = match &event.data {
            DomEventData::PointerMove(_) => "pointermove",
            DomEventData::PointerDown(_) => "pointerdown",
            DomEventData::PointerUp(_) => "pointerup",
            DomEventData::PointerEnter(_) => "pointerenter",
            DomEventData::PointerLeave(_) => "pointerleave",
            DomEventData::PointerOver(_) => "pointerover",
            DomEventData::PointerOut(_) => "pointerout",
            DomEventData::MouseMove(_) => "mousemove",
            DomEventData::MouseDown(_) => "mousedown",
            DomEventData::MouseUp(_) => "mouseup",
            DomEventData::MouseEnter(_) => "mouseenter",
            DomEventData::MouseLeave(_) => "mouseleave",
            DomEventData::MouseOver(_) => "mouseover",
            DomEventData::MouseOut(_) => "mouseout",
            DomEventData::Click(_) => "click",
            other => {
                self.0
                    .borrow_mut()
                    .push((format!("{other:?}").split('(').next().unwrap_or("?").to_string(), event.target));
                return;
            }
        };
        // pointermove/mousemove は数が多いので落とす
        if name.ends_with("move") {
            return;
        }
        self.0.borrow_mut().push((name.to_string(), event.target));
    }
}

fn pointer_at(x: f32, y: f32) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons: MouseEventButtons::None,
        mods: Modifiers::default(),
        details: PointerDetails {
            pressure: 0.0,
            tangential_pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            twist: 0,
            altitude: 0.0,
            azimuth: 0.0,
        },
        element: Point { x, y },
        active_pointers: Arc::new(AtomicRefCell::new(Vec::new())),
    }
}

fn class_of(doc: &HtmlDocument, id: usize) -> String {
    doc.get_node(id)
        .and_then(|n| {
            n.element_data()
                .and_then(|el| el.attr(blitz_dom::local_name!("class")))
                .map(|c| c.to_string())
        })
        .unwrap_or_else(|| {
            // テキストノードなら親のclassを出す
            doc.get_node(id)
                .and_then(|n| n.parent)
                .map(|p| format!("text in {}", class_of(doc, p)))
                .unwrap_or_else(|| "-".into())
        })
}

fn attr(doc: &HtmlDocument, id: usize, name: &str) -> Option<String> {
    let node = doc.get_node(id)?;
    let el = node.element_data()?;
    el.attrs
        .iter()
        .find(|a| &*a.name.local == name)
        .map(|a| a.value.clone())
}

fn chain_contains(doc: &HtmlDocument, from: usize, ancestor: usize) -> bool {
    let mut cur = Some(from);
    while let Some(id) = cur {
        if id == ancestor {
            return true;
        }
        cur = doc.get_node(id).and_then(|n| n.parent);
    }
    false
}

/// 祖先のlocationを足して文書座標にする
fn abs_pos(doc: &HtmlDocument, id: usize) -> (f32, f32) {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut cur = Some(id);
    while let Some(nid) = cur {
        let node = doc.get_node(nid).unwrap();
        x += node.final_layout.location.x;
        y += node.final_layout.location.y;
        cur = node.parent;
    }
    (x, y)
}

fn find_class(doc: &HtmlDocument, class: &str) -> Vec<usize> {
    fn walk(doc: &HtmlDocument, id: usize, class: &str, out: &mut Vec<usize>) {
        if let Some(node) = doc.get_node(id) {
            if let Some(el) = node.element_data() {
                if el
                    .attr(blitz_dom::local_name!("class"))
                    .is_some_and(|c| c.split_whitespace().any(|c| c == class))
                {
                    out.push(id);
                }
            }
            for c in node.children.iter() {
                walk(doc, *c, class, out);
            }
        }
    }
    let mut v = Vec::new();
    walk(doc, doc.root_element().id, class, &mut v);
    v
}
