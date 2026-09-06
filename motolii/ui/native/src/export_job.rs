use std::sync::{Arc,Mutex};
use std::path::PathBuf;
use crate::doc::store::Document;
use crate::render::{engine::Engine,export::{Cancel,ExportJob,ExportError,export_range_with_progress}};
use serde_json::{json,Value};

struct State {phase:&'static str,done:i64,total:i64,path:Option<String>,error:Option<String>}
pub(crate) struct ExportController {state:Arc<Mutex<State>>,cancel:Option<Cancel>}
impl Default for ExportController{
    fn default()->Self{Self{state:Arc::new(Mutex::new(State{phase:"idle",done:0,total:0,path:None,error:None})),cancel:None}}
}
impl ExportController{
    pub(crate) fn status(&self)->Value{let s=self.state.lock().unwrap_or_else(|e|e.into_inner());json!({"phase":s.phase,"done":s.done,"total":s.total,"path":s.path,"error":s.error})}
    pub(crate) fn cancel(&self){if let Some(c)=&self.cancel{c.cancel();let mut s=self.state.lock().unwrap_or_else(|e|e.into_inner());if s.phase=="running"{s.phase="cancelling"}}}
    pub(crate) fn start(&mut self,document:&Document,path:PathBuf,start:i64,end:i64)->Result<(),String>{
        {let s=self.state.lock().map_err(|e|e.to_string())?;if matches!(s.phase,"running"|"cancelling"){return Err("An export is already running".into())}}
        let comp=document.view().composition().map_err(|e|e.to_string())?.ok_or("No composition")?;
        if start<0||end<=start||end>comp.duration_frames{return Err("Export range must be inside the composition and nonempty".into())}
        let snapshot=document.flattened().map_err(|e|e.to_string())?;
        let cancel=Cancel::new();self.cancel=Some(cancel.clone());
        {let mut s=self.state.lock().map_err(|e|e.to_string())?;*s=State{phase:"running",done:0,total:end-start,path:Some(path.to_string_lossy().into_owned()),error:None};}
        let state=self.state.clone();
        let spawn=std::thread::Builder::new().name("motolii-port-export".into()).spawn(move||{
            let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||->Result<(),ExportError>{
                let mut engine=Engine::new()?;let job=ExportJob{out_path:path,qp0:false};
                export_range_with_progress(&mut engine,&snapshot.view(),&job,start..end,&cancel,|progress|{
                    let mut s=state.lock().unwrap_or_else(|e|e.into_inner());s.done=progress.frames_done;s.total=progress.frames_total;
                })?;Ok(())
            }));
            let mut s=state.lock().unwrap_or_else(|e|e.into_inner());
            match result{Ok(Ok(()))=>s.phase="complete",Ok(Err(ExportError::Cancelled))=>s.phase="cancelled",Ok(Err(error))=>{s.phase="failed";s.error=Some(error.to_string())},Err(_)=>{s.phase="failed";s.error=Some("Export worker panicked".into())}}
        });
        if let Err(error)=spawn{let mut s=self.state.lock().map_err(|e|e.to_string())?;s.phase="failed";s.error=Some(error.to_string());return Err(error.to_string())}
        Ok(())
    }
}
impl Drop for ExportController{fn drop(&mut self){self.cancel();}}
