//! 値の**型のある口**。`TrackJson`(文字列)に畳まず、Arrow の型のまま置く。
//!
//! 畳むと rerun からは文字列 1 個にしか見えず、型も索引も viewer も効かない。
//! 読む側は毎回 `serde_json` で解き直すことになり、それを償却するためだけの
//! 手控えが要る(2026-09-21 の標本で、書類の読みが再生の render thread の約 3 割)。
//!
//! **損失が無ければ rerun の型、あれば薄い自前の component。** どちらも Arrow の型で、
//! どちらも文字列ではない。rerun の `Vec2D` は f32、`Rgba32` は 8bit なので、
//! 座標と色はここで f64 のまま持つ(画面へは f32 で出るが、書類の値は丸めない)。
//!
//! 同じ component が編集ごとに型を変えられることは `type_probe.rs` で確かめてある。
//! だから古い書類は `TrackJson` のまま読め、新しい書き込みから型が付く。

use std::borrow::Cow;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, FixedSizeListArray, Float64Array, Int64Array, UInt64Array};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field};
use re_byte_size::SizeBytes;
use re_types_core::{
    Component, ComponentDescriptor, ComponentType, DeserializationResult, Loggable,
    SerializationResult, SerializedComponentBatch,
};

use crate::doc::eval::Value;
use crate::doc::store::{PropertyId, StoreError};

macro_rules! value_component {
    ($name:ident, $inner:ty, $arrow:ident, $array:ident, $id:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(pub $inner);

        impl SizeBytes for $name {
            #[inline]
            fn heap_size_bytes(&self) -> u64 { 0 }
        }

        impl<'a> From<$name> for Cow<'a, $name> {
            fn from(v: $name) -> Self { Self::Owned(v) }
        }

        impl<'a> From<&'a $name> for Cow<'a, $name> {
            fn from(v: &'a $name) -> Self { Self::Borrowed(v) }
        }

        impl Loggable for $name {
            fn arrow_datatype() -> DataType { DataType::$arrow }

            fn to_arrow_opt<'a>(
                data: impl IntoIterator<Item = Option<impl Into<Cow<'a, Self>>>>,
            ) -> SerializationResult<ArrayRef>
            where
                Self: 'a,
            {
                let values: Vec<Option<$inner>> =
                    data.into_iter().map(|v| v.map(|v| v.into().into_owned().0)).collect();
                Ok(Arc::new($array::from(values)))
            }

            fn from_arrow_opt(data: &dyn Array) -> DeserializationResult<Vec<Option<Self>>> {
                let array = data.as_any().downcast_ref::<$array>().ok_or_else(|| {
                    re_types_core::DeserializationError::datatype_mismatch(
                        DataType::$arrow,
                        data.data_type().clone(),
                    )
                })?;
                Ok(array.iter().map(|v| v.map(Self)).collect())
            }
        }

        impl Component for $name {
            fn name() -> ComponentType { $id.into() }
        }
    };
}

/// 数の組(座標・色)。Arrow の固定長 list の中身は f64 のまま — 書類の値を丸めない。
macro_rules! tuple_component {
    ($name:ident, $n:literal, $id:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(pub [f64; $n]);

        impl SizeBytes for $name {
            #[inline]
            fn heap_size_bytes(&self) -> u64 { 0 }
        }

        impl<'a> From<$name> for Cow<'a, $name> {
            fn from(v: $name) -> Self { Self::Owned(v) }
        }

        impl<'a> From<&'a $name> for Cow<'a, $name> {
            fn from(v: &'a $name) -> Self { Self::Borrowed(v) }
        }

        impl Loggable for $name {
            fn arrow_datatype() -> DataType {
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, false)), $n)
            }

            fn to_arrow_opt<'a>(
                data: impl IntoIterator<Item = Option<impl Into<Cow<'a, Self>>>>,
            ) -> SerializationResult<ArrayRef>
            where
                Self: 'a,
            {
                let rows: Vec<Option<[f64; $n]>> =
                    data.into_iter().map(|v| v.map(|v| v.into().into_owned().0)).collect();
                // 空の行にも桁は要る(固定長なので)。値は 0 で埋め、無いことは null の札で言う。
                let flat: Vec<f64> = rows.iter().flat_map(|r| r.unwrap_or([0.0; $n])).collect();
                let nulls = NullBuffer::from_iter(rows.iter().map(Option::is_some));
                Ok(Arc::new(FixedSizeListArray::new(
                    Arc::new(Field::new("item", DataType::Float64, false)),
                    $n,
                    Arc::new(Float64Array::from(flat)),
                    Some(nulls),
                )))
            }

            fn from_arrow_opt(data: &dyn Array) -> DeserializationResult<Vec<Option<Self>>> {
                let array = data.as_any().downcast_ref::<FixedSizeListArray>().ok_or_else(|| {
                    re_types_core::DeserializationError::datatype_mismatch(
                        Self::arrow_datatype(),
                        data.data_type().clone(),
                    )
                })?;
                // 長さを見ないと、同じ `FixedSizeList<Float64>` である色と座標が
                // 互いに読めてしまう(2 桁の口が 4 桁の値を先頭 2 つとして返す)。
                if array.value_length() != $n {
                    return Err(re_types_core::DeserializationError::datatype_mismatch(
                        Self::arrow_datatype(),
                        data.data_type().clone(),
                    ));
                }
                let values = array.values().as_any().downcast_ref::<Float64Array>().ok_or_else(|| {
                    re_types_core::DeserializationError::datatype_mismatch(
                        DataType::Float64,
                        array.values().data_type().clone(),
                    )
                })?;
                Ok((0..array.len())
                    .map(|row| {
                        array.is_valid(row).then(|| {
                            Self(std::array::from_fn(|i| values.value(row * $n + i)))
                        })
                    })
                    .collect())
            }
        }

        impl Component for $name {
            fn name() -> ComponentType { $id.into() }
        }
    };
}

value_component!(ValueF64, f64, Float64, Float64Array, "motolii.ValueF64");
value_component!(ValueBool, bool, Boolean, BooleanArray, "motolii.ValueBool");
value_component!(ValueEnum, i64, Int64, Int64Array, "motolii.ValueEnum");
value_component!(ValueLayer, u64, UInt64, UInt64Array, "motolii.ValueLayer");
tuple_component!(ValueVec2, 2, "motolii.ValueVec2");
tuple_component!(ValueColor, 4, "motolii.ValueColor");

fn descriptor<C: Component>(property: &PropertyId) -> ComponentDescriptor {
    ComponentDescriptor {
        archetype: Some(crate::doc::store::components::archetype_layer().into()),
        component: property.component(),
        component_type: Some(C::name()),
    }
}

/// 動かない値を型のまま置く。**ここに無い型は `None`** — 呼ぶ側が従来どおり文字列へ落とす。
/// 型を増やす時に触るのはこの関数と下の `constant_from` だけ。
pub fn constant_batch(
    property: &PropertyId,
    value: &Value,
) -> Option<Result<SerializedComponentBatch, StoreError>> {
    macro_rules! batch {
        ($kind:ident, $v:expr) => {
            Some(
                $kind::to_arrow([$kind($v)])
                    .map_err(|e| StoreError::Chunk(e.to_string()))
                    .map(|array| SerializedComponentBatch { descriptor: descriptor::<$kind>(property), array }),
            )
        };
    }
    match value {
        Value::F64(v) => batch!(ValueF64, *v),
        Value::Bool(v) => batch!(ValueBool, *v),
        Value::Enum(v) => batch!(ValueEnum, *v),
        Value::LayerId(v) => batch!(ValueLayer, *v),
        Value::Vec2(v) => batch!(ValueVec2, *v),
        Value::Color(v) => batch!(ValueColor, *v),
        // 道は数の組ではない(点・手・閉じ)。相当物が無いので文字列のまま。
        Value::Path(_) => None,
    }
}

/// 型のまま置かれた値を読む。置かれていなければ `None`(呼ぶ側が文字列を読む)。
pub fn constant_from(
    results: &re_query::LatestAtResults,
    component: re_types_core::ComponentIdentifier,
) -> Option<Value> {
    fn first<C: Component>(
        results: &re_query::LatestAtResults,
        component: re_types_core::ComponentIdentifier,
    ) -> Option<C> {
        results.component_batch::<C>(component)?.into_iter().next()
    }
    if let Some(v) = first::<ValueF64>(results, component) { return Some(Value::F64(v.0)) }
    if let Some(v) = first::<ValueBool>(results, component) { return Some(Value::Bool(v.0)) }
    if let Some(v) = first::<ValueEnum>(results, component) { return Some(Value::Enum(v.0)) }
    if let Some(v) = first::<ValueLayer>(results, component) { return Some(Value::LayerId(v.0)) }
    if let Some(v) = first::<ValueVec2>(results, component) { return Some(Value::Vec2(v.0)) }
    if let Some(v) = first::<ValueColor>(results, component) { return Some(Value::Color(v.0)) }
    None
}
