#[cfg(test)]
mod tests {
    use super::super::*;

    fn frames(rt: &crate::EditorRuntime) -> Vec<(i64, String)> {
        let fps = rt.doc.view().composition().unwrap().unwrap().fps;
        rt.doc
            .view()
            .markers()
            .unwrap()
            .iter()
            .map(|m| (m.time.try_to_frame_floor(fps).unwrap(), m.name.clone()))
            .collect()
    }

    fn id(rt: &crate::EditorRuntime, index: usize) -> String {
        let m = &rt.doc.view().markers().unwrap()[index];
        format!("{}/{}", m.time.num(), m.time.den())
    }

    /// A marker moves to another frame by `setMarker{frame}` (the Timeline drags it there), keeps its name, and
    /// refuses a frame another marker already holds.
    #[test]
    fn a_marker_moves_to_another_frame_and_never_onto_another() {
        let mut rt = crate::EditorRuntime::open("").unwrap();
        rt.request(json!({"op": "seek", "frame": 10})).unwrap();
        rt.request(json!({"op": "addMarker"})).unwrap();
        rt.request(json!({"op": "seek", "frame": 40})).unwrap();
        rt.request(json!({"op": "addMarker"})).unwrap();
        assert_eq!(frames(&rt), vec![(10, "10".into()), (40, "40".into())]);

        let first = id(&rt, 0);
        rt.request(json!({"op": "setMarker", "id": first, "frame": 25})).unwrap();
        assert_eq!(frames(&rt), vec![(25, "10".into()), (40, "40".into())]);

        let moved = id(&rt, 0);
        assert!(rt.request(json!({"op": "setMarker", "id": moved, "frame": 40})).is_err());
        assert_eq!(frames(&rt), vec![(25, "10".into()), (40, "40".into())]);

        rt.request(json!({"op": "setMarker", "id": moved, "name": "Drop"})).unwrap();
        assert_eq!(frames(&rt)[0], (25, "Drop".into()));
    }
}
