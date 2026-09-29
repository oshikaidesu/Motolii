//! Depth desk の平面図を掴む手: 図の上の動き(注視点を原点にした world の x・z)を、層の位置かカメラの orbit・距離へ写す。
use crate::edit::{Animate, Document, Intent};
use crate::doc::store::*;

pub(crate) enum DepthGrip {
    /// 層: 掴んだ時の position と z、world の X・Z を親の局所へ戻す軸。
    Layer{layer:LayerId,local:[f64;3],inverse_x:[f64;3],inverse_z:[f64;3]},
    /// カメラ: 注視点から見た eye の位置が向き(yaw)と距離。pitch は掴んだ時のまま。
    Camera{layer:LayerId,pitch:f64,base:f64},
}
pub(crate) struct DepthDrag {pub(crate) grip:DepthGrip,pub(crate) at:RationalTime,pub(crate) revision:Revision}
impl DepthDrag {
    /// `point`: 層は掴んだ所からの world のずれ、カメラは注視点から見た eye の world 位置(どちらも図の x, z)。
    pub(crate) fn edits(&self,doc:&Document,point:[f64;2],animate:Animate)->Result<Vec<Intent>,String>{
        if doc.revision()!=self.revision{return Err("Gesture canceled because document changed".into())}
        let [wx,wz]=point;
        let values=match self.grip{
            DepthGrip::Layer{layer,local,inverse_x:x,inverse_z:z}=>{
                let next=[0,1,2].map(|i|local[i]+x[i]*wx+z[i]*wz);
                vec![(layer,property::POSITION,Value::Vec2([next[0],next[1]])),(layer,property::POSITION_Z,Value::F64(next[2]))]
            }
            DepthGrip::Camera{layer,pitch,base}=>{
                // eye は注視点のまわりを回る: 向きが yaw、平面の距離を cos(pitch) で割ったものが距離。
                let yaw=(-wx).atan2(-wz).to_degrees();
                let cos=pitch.to_radians().cos().abs().max(0.05);
                vec![(layer,property::CAMERA_ORBIT,Value::Vec2([pitch,yaw])),(layer,property::CAMERA_DISTANCE,Value::F64((wx.hypot(wz)/cos/base).clamp(0.01,100.0)))]
            }
        };
        let mut out=Vec::new();
        for (layer,name,value) in values{
            if let Some(edit)=doc.place_checked(layer,&PropertyId::new(name).map_err(|e|e.to_string())?,value,self.at,animate).map_err(|e|e.to_string())?{out.push(edit)}
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::EditorRuntime;
    use crate::doc::store::*;
    use serde_json::json;
    fn at(rt:&EditorRuntime,layer:LayerId,name:&str)->Option<Value>{rt.doc.view().value_at(layer,&PropertyId::new(name).unwrap(),rt.time().unwrap()).unwrap()}
    /// Depth の図で掴んだ層とカメラは、Stage と同じ stageGesture で運ぶ: 図の動きを値にするのは native。離して 1 手。
    #[test]
    fn the_depth_plan_carries_a_layer_and_the_camera(){
        let mut rt=EditorRuntime::open("").unwrap();
        rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
        let shape=rt.viewer.selected().unwrap();
        rt.request(json!({"op":"setAttrs","layers":[shape.0],"patch":{"projection":"3D"}})).unwrap();
        let Some(Value::Vec2(p0))=at(&rt,shape,property::POSITION) else{panic!("position")};
        let z0=match at(&rt,shape,property::POSITION_Z){Some(Value::F64(z))=>z,_=>0.0};
        let go=|rt:&mut EditorRuntime,phase:&str,extra:serde_json::Value|{let mut j=json!({"op":"stageGesture","phase":phase,"mode":"depth"});for(k,v)in extra.as_object().unwrap(){j[k]=v.clone();}rt.request(j).unwrap();};
        go(&mut rt,"begin",json!({"ids":[shape.0],"start":[0.0,0.0],"point":[0.0,0.0]}));
        go(&mut rt,"update",json!({"point":[120.0,-40.0]}));
        go(&mut rt,"commit",json!({"point":[120.0,-40.0]}));
        let Some(Value::Vec2(p1))=at(&rt,shape,property::POSITION) else{panic!()};
        let z1=match at(&rt,shape,property::POSITION_Z){Some(Value::F64(z))=>z,_=>0.0};
        assert!((p1[0]-p0[0]-120.0).abs()<1e-3&&(p1[1]-p0[1]).abs()<1e-3&&(z1-z0+40.0).abs()<1e-3,"{p0:?},{z0} -> {p1:?},{z1}");
        rt.request(json!({"op":"undo"})).unwrap();
        assert_eq!(at(&rt,shape,property::POSITION),Some(Value::Vec2(p0)),"one release, one undo");

        rt.request(json!({"op":"create","kind":"camera"})).unwrap();
        let camera=rt.viewer.selected().unwrap();
        let comp=rt.doc.view().composition().unwrap().unwrap().spec();
        let base=crate::doc::core::distance_from_camera(comp,0.0) as f64;
        go(&mut rt,"begin",json!({"handle":"camera","start":[0.0,0.0],"point":[0.0,0.0]}));
        // eye を注視点の右、base の倍の遠さへ: yaw -90、距離 2
        go(&mut rt,"update",json!({"point":[base*2.0,0.0]}));
        go(&mut rt,"commit",json!({"point":[base*2.0,0.0]}));
        let Some(Value::Vec2(orbit))=at(&rt,camera,property::CAMERA_ORBIT) else{panic!("orbit")};
        let Some(Value::F64(distance))=at(&rt,camera,property::CAMERA_DISTANCE) else{panic!("distance")};
        assert!((orbit[1]+90.0).abs()<1e-3&&(distance-2.0).abs()<1e-3,"{orbit:?} {distance}");
    }
}
