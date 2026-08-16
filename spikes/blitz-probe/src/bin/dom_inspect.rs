//! HTMLを1回だけ解決して、指定classの要素の「親の連なり」と「確定した矩形」を出す。
//! 絶対配置が想定した包含ブロックへ効いているかを、絵ではなく数字で確かめるための道具。
//!
//! 使い方: dom_inspect <input.html> <class> [W] [H]

use blitz_dom::{DocumentConfig, StyleThreading};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: dom_inspect <input.html> <class> [W] [H]");
        std::process::exit(2);
    }
    let html = std::fs::read_to_string(&args[1]).expect("input html");
    let target = args[2].clone();
    let w: u32 = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(1280);
    let h: u32 = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(720);

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

    let mut hits = Vec::new();
    collect(&doc, doc.root_element().id, &target, &mut hits);
    println!("{} 個の .{target} を見つけた", hits.len());

    for id in hits {
        let node = doc.get_node(id).unwrap();
        let l = &node.final_layout;
        let pos = node
            .primary_styles()
            .map(|s| format!("{:?}", s.clone_position()))
            .unwrap_or_else(|| "?".into());

        // 親の連なりと、各親の position を出す
        let mut chain = Vec::new();
        let mut cur = node.parent;
        while let Some(pid) = cur {
            let p = doc.get_node(pid).unwrap();
            let cls = p
                .element_data()
                .and_then(|el| el.attr(blitz_dom::local_name!("class")))
                .unwrap_or("-");
            let ppos = p
                .primary_styles()
                .map(|s| format!("{:?}", s.clone_position()))
                .unwrap_or_else(|| "?".into());
            let pl = &p.final_layout;
            chain.push(format!(
                "{}[{}] @{:.0},{:.0} {:.0}x{:.0}",
                cls.split_whitespace().next().unwrap_or("-"),
                ppos,
                pl.location.x,
                pl.location.y,
                pl.size.width,
                pl.size.height
            ));
            cur = p.parent;
        }

        println!(
            "\nnode {id} position={pos} location={:.1},{:.1} size={:.1}x{:.1}",
            l.location.x, l.location.y, l.size.width, l.size.height
        );
        for (i, c) in chain.iter().enumerate() {
            println!("  親{}: {c}", i + 1);
        }
    }
}

fn collect(doc: &HtmlDocument, id: usize, class: &str, out: &mut Vec<usize>) {
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
            collect(doc, *c, class, out);
        }
    }
}
