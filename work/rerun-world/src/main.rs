//! Rerun holds the world over time; this asks it what is true at each frame and prints numbers.
//! Documents stay what they are: the video is referenced by timestamp (VideoFrameReference), never converted.
//! usage: rerun-world <frames> <fps> <video_at_seconds> [fx] > params.csv   (with `fx`: the effect parameters instead)

use std::sync::Arc;

use re_chunk::{Chunk, RowId};
use re_chunk_store::LatestAtQuery;
use re_entity_db::EntityDb;
use re_log_types::{EntityPath, StoreId, StoreKind, TimePoint, Timeline};
use re_sdk_types::archetypes::{Scalars, Transform3D, VideoFrameReference};
use re_sdk_types::components::{Scalar, Translation3D, VideoTimestamp};
use re_types_core::AsComponents;

fn timeline() -> Timeline {
    Timeline::new_sequence("frame")
}

fn log(db: &mut EntityDb, path: &str, frame: i64, what: &dyn AsComponents) {
    let chunk = Chunk::builder(EntityPath::from(path))
        .with_archetype(RowId::new(), TimePoint::default().with(timeline(), frame), what)
        .build()
        .expect("chunk");
    db.add_chunk(&Arc::new(chunk)).expect("add_chunk");
}

fn main() {
    let mut args = std::env::args().skip(1);
    let frames: i64 = args.next().and_then(|v| v.parse().ok()).expect("frames");
    let fps: f64 = args.next().and_then(|v| v.parse().ok()).expect("fps");
    let video_at: f64 = args.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let fx = args.next().as_deref() == Some("fx");

    let mut db = EntityDb::new(StoreId::random(StoreKind::Recording, "motolii-world"));

    // What the Skin would write as the user edits: where the camera is, what it looks at, which video time shows.
    for f in 0..frames {
        let t = f as f32 / fps as f32;
        let eye = [-0.5 + 1.4 * (t * 0.9).sin(), 1.6 + 0.35 * (t * 0.7).sin(), -11.5 + 0.8 * (t * 0.6).cos()];
        log(&mut db, "/world/camera", f, &Transform3D::from_translation(eye));
        log(&mut db, "/world/focus", f, &Transform3D::from_translation([0.0, 0.2, 0.0]));
        log(&mut db, "/world/video", f, &VideoFrameReference::new(VideoTimestamp::from_secs(video_at + f as f64 / fps)));
        // the glow effect's parameters over time (names are the effect.wgsl uniform members)
        log(&mut db, "/world/fx/glow/threshold", f, &Scalars::single(0.92));
        log(&mut db, "/world/fx/glow/intensity", f, &Scalars::single(0.55 + 0.35 * (t * 2.2).sin() as f64));
        log(&mut db, "/world/fx/glow/radius", f, &Scalars::single(28.0));
    }

    if fx {
        println!("frame,threshold,intensity,radius");
        for f in 0..frames {
            let q = LatestAtQuery::new(*timeline().name(), f);
            let get = |name: &str| -> f64 {
                db.latest_at_component_quiet::<Scalar>(&EntityPath::from(format!("/world/fx/glow/{name}")), &q, Scalars::descriptor_scalars().component)
                    .map(|(_, v)| v.0 .0)
                    .unwrap_or_else(|| panic!("no {name} at {f}"))
            };
            println!("{f},{:.4},{:.4},{:.3}", get("threshold"), get("intensity"), get("radius"));
        }
        return;
    }
    println!("frame,ex,ey,ez,ax,ay,az,video_sec");
    for f in 0..frames {
        let q = LatestAtQuery::new(*timeline().name(), f);
        let at = |path: &str| -> [f32; 3] {
            let (_, v) = db
                .latest_at_component_quiet::<Translation3D>(&EntityPath::from(path), &q, Transform3D::descriptor_translation().component)
                .unwrap_or_else(|| panic!("no transform for {path} at {f}"));
            [v.x(), v.y(), v.z()]
        };
        let eye = at("/world/camera");
        let focus = at("/world/focus");
        let (_, ts) = db
            .latest_at_component_quiet::<VideoTimestamp>(&EntityPath::from("/world/video"), &q, VideoFrameReference::descriptor_timestamp().component)
            .expect("video timestamp");
        println!("{f},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.6}", eye[0], eye[1], eye[2], focus[0], focus[1], focus[2], ts.0.as_secs());
    }
}
