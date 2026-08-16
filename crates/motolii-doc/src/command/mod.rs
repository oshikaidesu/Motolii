//! D2: コマンドシステム(apply/revert)。#103⑨: atomic command=property単位。
//!
//! **コマンドは決定済みの値を記録する**(実装ガード5)。「ドラッグ中」等の意図やデルタは
//! 持たず、apply/revertはold_value/new_valueの単純な書き込みで成立する(対称設計)。
//! 選択・hover・IME中間状態はこのenumに入れない(#103⑨、UI状態のまま)。
//!
//! **スコープ外**: `VectorContent` / `PathOp` 配下の DocParam 編集。複製時のID再写像は
//! `duplicate`が担当する。`ClipSource::Plugin.params` は `ScalarPropertyId::SourceParam`
//! で既存キーへ任意`DocParam`を書く(vism param の型天井にしない)。
//!
//! なぜ: 単一fileだと責任クラスタが混線する。公開pathはここが維持する。

mod apply;
mod asset;
mod clip;
mod effect;
mod effect_instance;
mod envelope;
mod error;
mod ids;
mod inverse;
mod locate;
mod meta;
mod position_key;
mod reservation;
mod split;
mod track_item;
mod variant;

pub use clip::{
    prepare_reparent_clip, prepare_set_clip_start, prepare_set_item_solo, prepare_set_item_visible,
    prepare_trim_clip_in, prepare_trim_clip_out,
};
pub use split::prepare_split_clip;
pub use error::CommandError;
pub use ids::{CommandKind, GestureId, MergeKey, ParentLocator, PropertyId, ScalarPropertyId};
pub use locate::{collect_layer_ids, find_envelope, find_item_location, layer_names_for_item};
pub use variant::{Command, PreparedAssetAdmission};

pub(crate) use asset::{prepare_admit_asset, prepare_remove_asset};
pub(crate) use effect::{
    find_use_location, guard_effect_lifecycle_document, introduced_ids_copy_local,
    introduced_ids_create, introduced_ids_link,
};
pub(crate) use locate::{
    envelope_of, envelope_of_mut, find_audio_component_mut, find_envelope_mut, find_items_vec,
    find_items_vec_mut,
};
pub(crate) use position_key::{
    prepare_set_position_key_interp, prepare_set_position_key_time, prepare_set_position_key_value,
    transform_param_key_time_for_command,
};
pub(crate) use reservation::validate_reservation_closure;
