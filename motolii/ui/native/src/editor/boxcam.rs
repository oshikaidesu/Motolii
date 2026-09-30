//! Stage に置いた箱(Boxcam)と作業範囲を掴む手: pointer をカメラと範囲の値へ写す。
use crate::edit::{Animate, Document, Intent};
use crate::doc::store::*;

/// Stage に置いた箱(Boxcam)の取っ手、またはその作業範囲の辺を掴んだ手。点は Stage の comp 画像 px。
pub(crate) enum BoxcamGrip {
    /// 箱の中心・拡大(角)・回転。`centroid` は掴んだ時の箱の 4 角の重心、値は掴んだ時の解決済みのカメラ。
    Camera{layer:LayerId,handle:BoxcamHandle,centroid:[f64;2],center:[f64;2],zoom:f64,roll:f64},
    /// 範囲の辺: 上 0・右 1・下 2・左 3。`margins` は [左, 上, 右, 下]。
    Extent{layer:LayerId,side:usize,margins:[f64;4]},
}
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub(crate) enum BoxcamHandle {Center,Zoom,Roll}
impl BoxcamHandle {
    pub(crate) fn parse(s:&str)->Result<Self,String>{match s{"center"=>Ok(Self::Center),"zoom"=>Ok(Self::Zoom),"roll"=>Ok(Self::Roll),_=>Err("Unknown camera handle".into())}}
}
/// 箱と範囲の手: pointer の動きをカメラと範囲の値へ写す唯一の所。`scale` は観測者の距離倍率(画像 px → comp px)。
pub(crate) struct BoxcamDrag {pub(crate) grip:BoxcamGrip,pub(crate) start:[f64;2],pub(crate) scale:f64,pub(crate) at:RationalTime,pub(crate) revision:Revision}
impl BoxcamDrag {
    pub(crate) fn edits(&self,doc:&Document,point:[f64;2],animate:Animate)->Result<Vec<Intent>,String>{
        if doc.revision()!=self.revision{return Err("Gesture canceled because document changed".into())}
        let (s,p)=(self.start,point);
        let (dx,dy)=((p[0]-s[0])*self.scale,(p[1]-s[1])*self.scale);
        let (layer,name,value)=match self.grip{
            BoxcamGrip::Extent{layer,side,margins}=>{
                let (index,outward)=match side{0=>(1,-dy),1=>(2,dx),2=>(3,dy),_=>(0,-dx)};
                (layer,property::STAGE_MARGINS[index],Value::F64((margins[index]+outward).max(0.0)))
            }
            BoxcamGrip::Camera{layer,handle,centroid:c,center,zoom,roll}=>match handle{
                BoxcamHandle::Center=>(layer,property::CAMERA_CENTER,Value::Vec2([center[0]+dx,center[1]+dy])),
                BoxcamHandle::Roll=>{
                    let turned=((p[1]-c[1]).atan2(p[0]-c[0])-(s[1]-c[1]).atan2(s[0]-c[0])).to_degrees();
                    (layer,property::CAMERA_ROLL,Value::F64(roll-turned))
                }
                BoxcamHandle::Zoom=>{
                    let ratio=(p[0]-c[0]).hypot(p[1]-c[1])/(s[0]-c[0]).hypot(s[1]-c[1]).max(1e-6);
                    (layer,property::CAMERA_ZOOM,Value::F64(zoom/ratio.max(0.01)))
                }
            },
        };
        doc.place_checked(layer,&PropertyId::new(name).map_err(|e|e.to_string())?,value,self.at,animate).map(|v|v.into_iter().collect()).map_err(|e|e.to_string())
    }
}
