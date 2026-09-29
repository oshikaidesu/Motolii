//! Stage で掴む物の始まり: 層の籠(pointer と矢印キー)、箱(Boxcam)と作業範囲、Depth の図。どれも同じ DragSession で運ぶ。
use super::*;

impl EditorRuntime {
    pub(super) fn stage_drag_begin(&mut self,seen:crate::viewer::View,ids:&[LayerId],mode:&str,handle:&str,start:[f64;2])->Result<editor::stage::DragSession,String>{
        let at=self.time()?;
        let projection=ids.last().and_then(|id|self.doc.view().attrs(*id).ok().flatten()).map_or(LayerProjection::ThreeD,|a|a.projection);
        let observer=self.view_camera(seen)?;
        let projection_camera=self.projection_camera(seen,projection)?;
        editor::stage::DragSession::begin(&self.doc,&mut self.engine,ids,mode,handle,start,at,observer,projection_camera,self.viewer.stage_view_scale,self.viewer.stage_held.as_deref())
    }
    /// 矢印キーの nudge(AE の Alt+矢印): 選んだ層を、出力(Camera)の画面で `x`, `y` px 動かす。Stage で掴んで動かすのと同じ
    /// 移動の写像を 1 手で通す(Undo 一発)。
    pub(super) fn nudge(&mut self,j:&J)->Result<(),String>{
        let ids=self.viewer.selected_ids.clone();
        let layer=*ids.last().ok_or("Select a layer")?;
        // the cage is read from the scene of this revision (a key can come before any frame is drawn)
        let at=self.time()?;self.engine.frame_graph_editor_scene(&self.doc.view(),at).map_err(e)?;
        let bounds=self.bounds(layer).ok_or("Selected layer bounds are unavailable")?;
        let start:[f64;2]=serde_json::from_value(bounds["corners"][0].clone()).map_err(e)?;
        let drag=self.stage_drag_begin(crate::viewer::View::Camera,&ids,"move","body",start)?;
        let edits=drag.edits(&self.doc,[start[0]+num(j,"x")?,start[1]+num(j,"y")?],false,false,self.viewer.animate)?;
        self.apply(edits)
    }
    /// 箱(Boxcam)の取っ手か作業範囲の辺を掴む。camera: `ids` の 1 台と `handle`(center / zoom / roll)。extent: `handle` が辺(top / right / bottom / left)。
    /// 選択は変えない(箱の中心を掴んで選ぶのは skin の press)。
    pub(super) fn boxcam_drag(&self,j:&J,seen:crate::viewer::View)->Result<editor::boxcam::BoxcamDrag,String>{
        use editor::boxcam::{BoxcamGrip,BoxcamHandle};
        let start:[f64;2]=serde_json::from_value(j["start"].clone()).map_err(e)?;
        let at=self.time()?;
        let scale=if seen==crate::viewer::View::User{self.viewer.user_camera.distance_scale as f64}else{1.0};
        let grip=if j["mode"]=="extent"{
            let extent=motolii_render::picture::resolve::camera::resolve_stage_extent(&self.doc.view(),at).map_err(e)?;
            let layer=extent.layer.ok_or("No working area")?;
            let side=match string(j,"handle")?{"top"=>0,"right"=>1,"bottom"=>2,"left"=>3,_=>return Err("Unknown working-area side".into())};
            BoxcamGrip::Extent{layer,side,margins:extent.margins.map(f64::from)}
        }else{
            let layer=*ids(&j["ids"])?.last().ok_or("Missing camera")?;
            if let Some(reason)=editor::functions::lens::edit_rejection(&self.doc.view(),layer).map_err(e)?{return Err(reason.into())}
            let (camera,corners)=self.camera_box(layer)?;
            let corners=corners.ok_or("The camera box is not fully in view")?;
            let centroid=corners.iter().fold([0.0,0.0],|a,c|[a[0]+c[0] as f64/4.0,a[1]+c[1] as f64/4.0]);
            BoxcamGrip::Camera{layer,handle:BoxcamHandle::parse(string(j,"handle")?)?,centroid,center:camera.center.map(f64::from),zoom:camera.zoom as f64,roll:camera.roll_degrees as f64}
        };
        Ok(editor::boxcam::BoxcamDrag{grip,start,scale,at,revision:self.doc.revision()})
    }
    /// Depth desk の図で層(`ids` の 1 つ)かカメラ(`handle`: camera)を掴む。値の元は図に描いたのと同じ depthLayout。
    pub(super) fn depth_drag(&mut self,j:&J)->Result<editor::depth_drag::DepthDrag,String>{
        use editor::depth_drag::{DepthDrag,DepthGrip};
        let at=self.time()?;
        let scene=self.engine.frame_graph_editor_scene(&self.doc.view(),at).map_err(e)?;
        let layout=self.depth_layout(&scene)?;
        let f=|v:&J|v.as_f64().ok_or_else(||"Depth layout is incomplete".to_string());
        let three=|v:&J|->Result<[f64;3],String>{Ok([f(&v[0])?,f(&v[1])?,f(&v[2])?])};
        let grip=if j["handle"]=="camera"{
            let camera=&layout["camera"];
            let layer=LayerId(camera["layer"].as_u64().ok_or("The camera is not a layer")?);
            DepthGrip::Camera{layer,pitch:f(&camera["orbit"][0])?,base:f(&camera["baseDistance"])?}
        }else{
            let layer=*ids(&j["ids"])?.last().ok_or("Missing layer")?;
            let item=layout["items"].as_array().into_iter().flatten().find(|i|i["id"]==layer.0).ok_or("The layer is not on the Depth plan")?;
            if item["locked"]==true{return Err("Layer is locked".into())}
            DepthGrip::Layer{layer,local:three(&item["local"])?,inverse_x:three(&item["inverseX"])?,inverse_z:three(&item["inverseZ"])?}
        };
        Ok(DepthDrag{grip,at,revision:self.doc.revision()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    /// 矢印キーの nudge は 1 手: 出力の画面で 10 px 右へ、Undo 一発で戻る。
    #[test]
    fn a_nudge_is_one_step_on_the_output(){
        let mut rt=EditorRuntime::open("").unwrap();
        rt.request(json!({"op":"create","kind":"rectangle"})).unwrap();
        let shape=rt.viewer.selected().unwrap();
        let position=|rt:&EditorRuntime|rt.doc.view().value_at(shape,&PropertyId::new(property::POSITION).unwrap(),rt.time().unwrap()).unwrap();
        let before=position(&rt);
        rt.request(json!({"op":"nudge","x":10.0,"y":0.0})).unwrap();
        let (Some(Value::Vec2(a)),Some(Value::Vec2(b)))=(before.clone(),position(&rt)) else{panic!("position")};
        assert!((b[0]-a[0]-10.0).abs()<1e-3&&(b[1]-a[1]).abs()<1e-3,"{a:?} -> {b:?}");
        rt.request(json!({"op":"undo"})).unwrap();
        assert_eq!(position(&rt),before);
    }
}
