//! 読み手の手控え — 巡って解く読み手が view の寿命の間だけ使う場所と、
//! 書類がコマをまたいで覚える場所。**中身を作るのは絵の側**で、コアは形だけ決めて運ぶ。
//! ここに置くのは、view がこれを持ち歩く以上、形が契約になるため。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::doc::core::RationalTime;
use crate::doc::store::LayerId;


/// その時刻に並べた結果。`slots` は並ぶ子、`sizes` は Display の Group の箱の大きさ(素材座標で [0, 0]..size)。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub slots: HashMap<LayerId, Slot>,
    pub sizes: HashMap<LayerId, [f32; 2]>,
    /// 容器の外で押し合った物の、親の空間でのずれ。
    pub nudges: HashMap<LayerId, [f32; 2]>,
    /// 押し合いの奥行きのずれ(両方が 2D でない物同士、Position Z に足す)。
    pub nudges_z: HashMap<LayerId, f32>,
    /// Display の Group の奥行きの範囲 [手前, 奥]。揃えなら面が奥で [-奥行き, 0]、奥へ積むなら [0, 積んだ厚み]。
    pub depths: HashMap<LayerId, [f32; 2]>,
    /// Grid の Group の升目(素材座標): 列の [始, 終] と行の [始, 終]。格子へ吸い付く子と、格子を描く線が読む。
    pub fields: HashMap<LayerId, (Vec<(f32, f32)>, Vec<(f32, f32)>)>,
}

/// 並ぶ子の変換の差し替え: 層の Position と Scale の代わりに使う値と、形の輪郭の伸び。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub stretch: [f32; 2],
    /// 横が Fill の文字: その幅で折り返す(素材座標の幅、Scale で割った値)。
    pub wrap: Option<f32>,
    /// 奥行きの揃えで足す z(Position Z に足す)。面が奥(z = 0)、物は camera 側(負)へ出る。
    pub z: f32,
    /// 奥行きに足す倍率(Scale Z に掛ける)。網・点群の奥行きは描く側が xy の拡縮から伸ばすので、今は 1。
    pub scale_z: f32,
    /// 並びに効く回転(Tilt X・Tilt Y・Rotation、度)。層の Rotation / Tilt に足す。
    pub rotation: [f32; 3],
    /// 拡縮・回転の中心(素材座標)。Anchor を書いていなければ箱の中心(CSS の transform-origin: 50% 50%)。
    pub anchor: [f32; 2],
}

/// 読み view 1 つが、巡る配置のために持つ手控え。中身は全部この view の物で、
/// 寿命も view と同じ(別の view と混ぜると、巡り止めが他人の巡りを止める)。
#[derive(Default)]
pub struct Scratch {
    /// 解いたコマの配置。view は値を変えないので、時刻ごとに 1 回で足りる。
    pub frames: HashMap<RationalTime, std::sync::Arc<Frame>>,
    /// 解いている入れ子の深さ。0 から入った物だけがコマをまたぐ覚えへ書く。
    pub depth: u32,
    /// 付き合いの輪と線の輪。掛けた物は必ず外す。
    pub anchoring: std::collections::HashSet<(u64, i64, i64)>,
    pub routing: std::collections::HashSet<(u64, i64, i64)>,
    /// 箱の子の順。版ごとに覚える。
    pub kids: HashMap<(u64, LayerId), std::sync::Arc<Vec<LayerId>>>,
}

impl Scratch {
    /// 一番外側から入ったか。出る時は必ず `leave`。
    pub fn enter(&mut self) -> bool {
        let outermost = self.depth == 0;
        self.depth += 1;
        outermost
    }

    pub fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }
}

/// 巡り止め: 掛かれば解いてよい、掛からなければ既に自分が巡っている。外すのは掛けた者の責任。
impl Scratch {
    pub fn begin_route(&mut self, key: (u64, i64, i64)) -> bool {
        self.routing.insert(key)
    }

    pub fn end_route(&mut self, key: (u64, i64, i64)) {
        self.routing.remove(&key);
    }
}

/// view の寿命の間、時刻ごとに 1 回だけ解く(view は値を変えない)。
pub type Memo = Rc<RefCell<Scratch>>;

/// 書類が持つ、コマをまたぐ配置の覚え。版が変われば丸ごと捨てる。移り方が 1 コマに過去の時刻の配置を何十回も解くので、
/// 次のコマで同じ時刻を解き直さない(天井の棚卸し 2026-09-15)。
#[derive(Default)]
pub struct LayoutCache {
    pub revision: Option<crate::doc::store::Revision>,
    pub frames: HashMap<RationalTime, std::sync::Arc<Frame>>,
    /// 版が変わらない限り同じ物。時刻では変わらないので、コマごとに作り直さない。
    /// (層の並び, 親子, Group 判定)。時刻で動く solo/effect enabled はここに置かない。
    pub structure: Option<std::sync::Arc<Structure>>,
}

/// 書類の形。時刻に依らない。
pub struct Structure {
    pub layers: Vec<crate::doc::store::LayerId>,
    pub present: std::collections::HashSet<crate::doc::store::LayerId>,
    /// 親子と積み順は書類の版でのみ変わる。再生の時刻では子が「居るか」
    /// だけが変わるので、Flow はここを再走査・再 sort しない。
    pub children: HashMap<crate::doc::store::LayerId, Vec<(i16, crate::doc::store::LayerId)>>,
    pub parents: HashMap<crate::doc::store::LayerId, Option<crate::doc::store::LayerId>>,
    pub groups: std::collections::HashSet<crate::doc::store::LayerId>,
}

impl LayoutCache {
    pub const LIMIT: usize = 4096;
}
