//! キー選択の確定・補間切替・全選択系・Time-Reverse・キー追加削除・
//! コピー/ペースト(SP-2 分割、`write.rs` 1234-1341行・1354-1650行を移設)。
//! **中身は無改変**。

use super::*;
use super::key_drag::commit_key_frames;
use super::misc::composition;

impl PaneState {
    // ---- クリップボード(第4切片 keys2、map 507 の土台) ----

    /// コピー(507 の土台)。選択キーの位置は [`keys2::copy_keys`]、実際の値は
    /// ここが Document から読み直して [`KeyClipboardState`] へ保存する
    /// (`keys2::ClipboardKey` は layer/property/offset だけしか運ばない —
    /// `keys2` モジュール doc 参照、値は結線側の責任)。stale なキー(track
    /// から既に消えている)は黙って除外する(`select_all_keys_of_property`
    /// の空無視と同格) — 除外後に1本も残らなければクリップボードは
    /// 更新しない(古いコピーを黙って壊さない)。空選択は理由つき拒否(M13)。
    pub(crate) fn copy_selected_keys(&mut self, doc: &Document, session: &Session) -> Option<String> {
        if session.selected_keys.is_empty() {
            return Some("キーが選ばれていないのでコピーできない — 菱形を選んでから".into());
        }
        let Some(composition) = composition(doc) else {
            return None;
        };
        let fps = composition.fps;
        let store = doc.view();
        let mut present = Vec::new();
        let mut values = Vec::new();
        for key in &session.selected_keys {
            let Ok(Some(track)) = store.track(key.layer, &key.property) else {
                continue;
            };
            let Some(found) = track.keys().iter().find(|k| k.t.try_to_frame_round(fps) == Ok(key.frame)) else {
                continue;
            };
            present.push(key.clone());
            values.push(found.clone());
        }
        drop(store);
        if present.is_empty() {
            return None; // 選択が stale(track から消えている等)— 黙って無視。
        }
        self.key_clipboard = KeyClipboardState { clip: keys2::copy_keys(&present), values };
        None
    }

    /// 貼り付け(507 本体 + 通常貼り付けの土台)。`reversed` で
    /// [`keys2::paste_keys`]/[`keys2::paste_keys_reversed`] を切り替える —
    /// どちらも新しい anchor は playhead(`session.playhead`)。貼り先 layer が
    /// どれかロックなら理由つき拒否(M13、書く前に検分 — 一部だけ書いて
    /// 途中で止まる状態を作らない)。書き込みは (layer,property) ごとに
    /// track を読み直し、コピー時の値([`motolii_store::Keyframe`]、時刻だけ
    /// 貼り先へ差し替え)を `insert`(同時刻キーは自動上書き —
    /// `KeyframeTrack::insert` の doc 参照、`commit_key_frames` のように
    /// 手で削ってから足し直す必要が無い)してから1回の `apply_all` で確定
    /// する(**1操作 = 1 undo**)。貼り付けた集合を新しい選択にする(貼り付け
    /// 直後に続けて動かせる — 通常のペーストの慣習)。クリップボードが空
    /// なら理由つき拒否(M13)。
    pub(crate) fn paste_keys(&self, doc: &mut Document, session: &mut Session, reversed: bool) -> Option<String> {
        if self.key_clipboard.clip.keys.is_empty() {
            return Some("コピー済みのキーが無いので貼り付けられない — 先に Copy".into());
        }
        let target = session.playhead;
        let targets = if reversed {
            keys2::paste_keys_reversed(&self.key_clipboard.clip, target)
        } else {
            keys2::paste_keys(&self.key_clipboard.clip, target)
        };

        let store = doc.view();
        let layers: HashSet<LayerId> = targets.iter().map(|key| key.layer).collect();
        for &layer in &layers {
            let locked = store.attrs(layer).ok().flatten().unwrap_or_default().locked;
            if locked {
                drop(store);
                return Some(format!("layer {} はロックされているので貼り付けられない", layer.0));
            }
        }
        drop(store);

        let Some(composition) = composition(doc) else {
            return None;
        };
        let fps = composition.fps;
        let mut groups: BTreeMap<(LayerId, PropertyId), Vec<(i64, motolii_store::Keyframe)>> = BTreeMap::new();
        for (selector, value) in targets.iter().zip(self.key_clipboard.values.iter()) {
            groups.entry((selector.layer, selector.property.clone())).or_default().push((selector.frame, value.clone()));
        }

        let store = doc.view();
        let mut intents = Vec::new();
        for ((layer, property), inserts) in groups {
            let mut track = store.track(layer, &property).ok().flatten().unwrap_or_default();
            for (frame, value) in inserts {
                let Ok(t) = RationalTime::try_from_frame(frame, fps) else {
                    continue;
                };
                let mut key = value;
                key.t = t;
                track.insert(key);
            }
            intents.push(Intent::SetTrack { layer, property, track });
        }
        drop(store);

        if intents.is_empty() {
            return None;
        }
        if let Err(error) = doc.apply_all(intents) {
            return Some(format!("貼り付けを書けない: {error}"));
        }
        session.key_anchor = targets.first().cloned();
        session.selected_keys = targets;
        None
    }
}

/// キー選択の確定(第2波 T3・裁定148/151)。`key_rows::update` は「どのキーを・
/// どの操作で」までしか判定しない(canvas 側は Document/Session を直接書けない)
/// ので、`Session::selected_keys`/`key_anchor` の実際の読み書きはここで行う。
pub(crate) fn apply_key_selection(session: &mut Session, doc: &mut Document, op: KeySelectionOp) {
    match op {
        KeySelectionOp::Single(key) => {
            session.selected_keys = vec![key.clone()];
            session.key_anchor = Some(key);
        }
        KeySelectionOp::Toggle(key) => {
            if let Some(pos) = session.selected_keys.iter().position(|k| *k == key) {
                session.selected_keys.remove(pos);
            } else {
                session.selected_keys.push(key.clone());
            }
            session.key_anchor = Some(key);
        }
        KeySelectionOp::Range(key) => {
            let Some(anchor) = session.key_anchor.clone() else {
                // 基点が無ければ単独選択と同じ扱いへ安全側で倒す。
                session.selected_keys = vec![key.clone()];
                session.key_anchor = Some(key);
                return;
            };
            let fps = composition(doc).map(|c| c.fps);
            let property_rows = property_rows(&doc.view(), session, fps);
            let order = key_order(&property_rows);
            let anchor_pos = order.iter().position(|k| *k == anchor);
            let clicked_pos = order.iter().position(|k| *k == key);
            match (anchor_pos, clicked_pos) {
                (Some(a), Some(c)) => {
                    let (lo, hi) = if a <= c { (a, c) } else { (c, a) };
                    session.selected_keys = order[lo..=hi].to_vec();
                }
                _ => {
                    // anchor/clicked のどちらかが今の property_rows に無い —
                    // 単独選択へ安全側で倒す(M16)。
                    session.selected_keys = vec![key];
                }
            }
            // anchor は不変 — 同じ基点から Shift 連打で範囲を伸縮できる。
        }
    }
}

/// 選択中のキーを消す(正典 §3「Delete はキー選択が層選択より優先」)。
/// property ごとにまとめて読み直し、選択されたフレームだけを落とした
/// `KeyframeTrack` を1回の `apply_all` で書き戻す(**1操作 = 1 undo**)。
/// 選択が空なら no-op。
pub(crate) fn delete_selected_keys(doc: &mut Document, session: &mut Session) -> Option<String> {
    if session.selected_keys.is_empty() {
        return None;
    }
    let keys = std::mem::take(&mut session.selected_keys);
    session.key_anchor = None;
    let Some(composition) = composition(doc) else {
        return None;
    };
    let fps = composition.fps;

    let mut groups: BTreeMap<(LayerId, PropertyId), Vec<i64>> = BTreeMap::new();
    for key in keys {
        groups.entry((key.layer, key.property)).or_default().push(key.frame);
    }

    let store = doc.view();
    let mut intents = Vec::new();
    for ((layer, property), frames) in groups {
        let Ok(Some(track)) = store.track(layer, &property) else {
            continue;
        };
        let mut new_track = KeyframeTrack::new();
        for existing in track.keys() {
            let Ok(frame) = existing.t.try_to_frame_round(fps) else {
                continue;
            };
            if frames.contains(&frame) {
                continue; // 選択されたキーは書き戻さない = 削除。
            }
            new_track.insert(existing.clone());
        }
        intents.push(Intent::SetTrack { layer, property, track: new_track });
    }
    drop(store);

    if !intents.is_empty() {
        if let Err(error) = doc.apply_all(intents) {
            return Some(format!("キーを消せない: {error}"));
        }
    }
    None
}

/// 選択キーの補間切替(第3切片 B15、map 495/512/513/514 + プリセット 485〜490)。
/// property ごとに track を読み直し、**選択されたキーの `interp` だけ**を
/// 差し替えた `KeyframeTrack` を1回の `apply_all` で書き戻す(**1操作 = 1 undo**、
/// `delete_selected_keys` と同じ形)。時刻・値・spatial は不変 — 選択も動かない
/// (frame が変わらないので selector はそのまま生きている)。
///
/// 空選択は理由つき拒否(M13: メニュー/キー入口の動詞が黙って何もしないのは
/// 無反応系違反 — Delete キーの空 no-op とは入口の性格が違う)。ロック層も
/// 理由つき拒否(`start_key_drag` と同じ柵)。
pub(crate) fn set_key_interp(doc: &mut Document, session: &Session, interp: Interp) -> Option<String> {
    if session.selected_keys.is_empty() {
        return Some("キーが選ばれていないので補間を変えられない — 菱形を選んでから".into());
    }
    let Some(composition) = composition(doc) else {
        return None;
    };
    let fps = composition.fps;

    let mut groups: BTreeMap<(LayerId, PropertyId), Vec<i64>> = BTreeMap::new();
    for key in &session.selected_keys {
        groups
            .entry((key.layer, key.property.clone()))
            .or_default()
            .push(key.frame);
    }

    let store = doc.view();
    for &(layer, _) in groups.keys() {
        let locked = store.attrs(layer).ok().flatten().unwrap_or_default().locked;
        if locked {
            drop(store);
            return Some(format!("layer {} はロックされているので補間を変えられない", layer.0));
        }
    }
    let mut intents = Vec::new();
    for ((layer, property), frames) in groups {
        let Ok(Some(track)) = store.track(layer, &property) else {
            continue;
        };
        let mut new_track = KeyframeTrack::new();
        for existing in track.keys() {
            let mut key = existing.clone();
            if let Ok(frame) = existing.t.try_to_frame_round(fps) {
                if frames.contains(&frame) {
                    key.interp = interp;
                }
            }
            new_track.insert(key);
        }
        intents.push(Intent::SetTrack { layer, property, track: new_track });
    }
    drop(store);

    if intents.is_empty() {
        return None; // 選択が stale(track が消えている等)— 黙って無視。
    }
    if let Err(error) = doc.apply_all(intents) {
        return Some(format!("補間を書けない: {error}"));
    }
    None
}

/// property の全キー選択(map 509、正典 §8.1 SelectAllKeysOfProperty)。
/// track を読み、全キーを選択集合へ差し替える(足し引きではない — AE の
/// 「property 名クリック」と同じ全置換)。anchor は先頭キー(以後の Shift 範囲の
/// 基点)。track が無い/キーが無い(stale なクリック)は黙って無視。
pub(crate) fn select_all_keys_of_property(
    doc: &mut Document,
    session: &mut Session,
    layer: LayerId,
    property: PropertyId,
) -> Option<String> {
    let Some(composition) = composition(doc) else {
        return None;
    };
    let fps = composition.fps;
    let store = doc.view();
    let Ok(Some(track)) = store.track(layer, &property) else {
        return None;
    };
    let keys: Vec<KeySelector> = track
        .keys()
        .iter()
        .filter_map(|key| key.t.try_to_frame_round(fps).ok())
        .map(|frame| KeySelector { layer, property: property.clone(), frame })
        .collect();
    drop(store);
    if keys.is_empty() {
        return None;
    }
    session.key_anchor = Some(keys[0].clone());
    session.selected_keys = keys;
    None
}

/// 表示中の全キー選択(map 510)。「表示中」= `projection::property_rows` が
/// 今出している行(選択 layer のキー持ち property)の全キー — 見えている
/// とおりに採れる(正典 §4 Cmd+A の思想)。並びは `key_order`(行順→時刻順)
/// そのもの。何も見えていなければ黙って無視(Cmd+A の空 no-op と同格)。
pub(crate) fn select_all_visible_keys(doc: &mut Document, session: &mut Session) -> Option<String> {
    let fps = composition(doc).map(|c| c.fps);
    let rows = property_rows(&doc.view(), session, fps);
    let order = key_order(&rows);
    if order.is_empty() {
        return None;
    }
    session.key_anchor = Some(order[0].clone());
    session.selected_keys = order;
    None
}

/// Time-Reverse Keyframes(map 518)。選択キーを `(layer, property)` ごとに
/// まとめ、それぞれ独立に [`keys2::reversed_key_group`] で鏡映する——複数
/// property/複数 layer をまたぐ選択は、各グループが**それぞれ自分の
/// `[min, max]`** の中で鏡映する(全選択をまたいだ1つの時間軸で鏡映するの
/// ではない、[`keys2::reversed_key_group`] のモジュール doc「集合自身の
/// 範囲」参照)。鏡映後の全グループ分をまとめて1回の
/// [`commit_key_frames`](move/retime/nudge と共通の書き口、内部で
/// property ごとに `Intent::SetTrack` を束ねる)へ渡す——**1操作 = 1 undo**
/// (グループごとに別々の undo にはしない)。値・`interp` は動かさない
/// (frame の並びだけが入れ替わる)。
///
/// 空選択・ロック層は理由つき拒否(M13、`set_key_interp` と同じ形)。
pub(crate) fn reverse_selected_keys(doc: &mut Document, session: &mut Session) -> Option<String> {
    if session.selected_keys.is_empty() {
        return Some("キーが選ばれていないので時間反転できない — 菱形を選んでから".into());
    }
    let mut groups: BTreeMap<(LayerId, PropertyId), Vec<KeySelector>> = BTreeMap::new();
    for key in &session.selected_keys {
        groups.entry((key.layer, key.property.clone())).or_default().push(key.clone());
    }

    let store = doc.view();
    for &(layer, _) in groups.keys() {
        let locked = store.attrs(layer).ok().flatten().unwrap_or_default().locked;
        if locked {
            drop(store);
            return Some(format!("layer {} はロックされているので時間反転できない", layer.0));
        }
    }
    drop(store);

    let mut origins: Vec<KeySelector> = Vec::new();
    let mut news: Vec<i64> = Vec::new();
    for group in groups.into_values() {
        let origin_frames: Vec<i64> = group.iter().map(|k| k.frame).collect();
        let new_frames = keys2::reversed_key_group(&origin_frames);
        origins.extend(group);
        news.extend(new_frames);
    }
    commit_key_frames(doc, session, &origins, &news, 0)
}

/// キーの追加/削除(map 472/473/474/476/477/478、[`keys2::toggle_keyframe_at`]
/// の結線)。既存キーの frame 集合から対象を判定し、追加なら現在時刻の値を
/// `KeyframeTrack::eval` で焼き付け、削除ならその frame のキーだけ落とした
/// track を1回の `Intent::SetTrack` で書く(**1操作 = 1 undo**)。ロック層は
/// 理由つき拒否(M13)。comp が無ければ理由つき拒否(frame⇄時刻の変換が
/// できない)。
pub(crate) fn toggle_keyframe_at_playhead(
    doc: &mut Document,
    session: &Session,
    layer: LayerId,
    property: PropertyId,
) -> Option<String> {
    let locked = doc.view().attrs(layer).ok().flatten().unwrap_or_default().locked;
    if locked {
        return Some(format!("layer {} はロックされているのでキーを追加/削除できない", layer.0));
    }
    let Some(composition) = composition(doc) else {
        return Some("comp が無いのでキーを追加/削除できない".into());
    };
    let fps = composition.fps;
    let Ok(t) = RationalTime::try_from_frame(session.playhead, fps) else {
        return Some("この時刻にキーを置けない".into());
    };

    let mut track = doc.view().track(layer, &property).ok().flatten().unwrap_or_default();
    let existing_frames: Vec<i64> =
        track.keys().iter().filter_map(|k| k.t.try_to_frame_round(fps).ok()).collect();

    match keys2::toggle_keyframe_at(&existing_frames, session.playhead) {
        keys2::KeyframeToggle::Add => {
            let value = track.eval(t);
            track.insert(motolii_store::Keyframe { t, value, interp: Interp::Linear, spatial: None });
        }
        keys2::KeyframeToggle::Remove(_) => {
            let mut updated = KeyframeTrack::new();
            for existing in track.keys() {
                if existing.t.try_to_frame_round(fps) == Ok(session.playhead) {
                    continue; // ちょうどこの frame のキーだけ落とす = 削除。
                }
                updated.insert(existing.clone());
            }
            track = updated;
        }
    }

    if let Err(error) = doc.apply(Intent::SetTrack { layer, property, track }) {
        return Some(format!("キーを書けない: {error}"));
    }
    None
}
