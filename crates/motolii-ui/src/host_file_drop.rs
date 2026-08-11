//! Finder file dropをAppKitからproduct event loopへ薄く橋渡しする。

use std::path::PathBuf;

use winit::event_loop::EventLoopProxy;

use crate::product_runtime::ProductEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostFileDropTerminal {
    Perform,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HostFileDropEvent {
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) position: [f64; 2],
    pub(crate) terminal: HostFileDropTerminal,
}

#[cfg(target_os = "macos")]
mod platform {
    use std::collections::HashMap;
    use std::ffi::CStr;
    use std::sync::{Mutex, OnceLock};

    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel};
    use objc2::{msg_send, sel};
    use objc2_app_kit::NSView;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    use super::*;

    static PROXIES: OnceLock<Mutex<HashMap<usize, EventLoopProxy<ProductEvent>>>> = OnceLock::new();

    pub(crate) struct PlatformFileDrop {
        view: Retained<NSView>,
        original_class: &'static AnyClass,
        key: usize,
    }

    impl PlatformFileDrop {
        pub(crate) fn new(
            window: &winit::window::Window,
            proxy: EventLoopProxy<ProductEvent>,
        ) -> Result<Self, PlatformFileDropError> {
            let handle = window
                .window_handle()
                .map_err(|_| PlatformFileDropError::WindowHandle)?;
            let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
                return Err(PlatformFileDropError::WrongPlatform);
            };
            // SAFETY: WindowHandleのlifetime中はns_viewが有効であり、retainして所有を得る。
            let view: Retained<NSView> =
                unsafe { Retained::retain(handle.ns_view.as_ptr().cast()) }
                    .ok_or(PlatformFileDropError::MissingView)?;
            let key = Retained::as_ptr(&view) as usize;
            let original_class = view.class();
            let subclass = drop_subclass(original_class)?;
            PROXIES
                .get_or_init(Default::default)
                .lock()
                .map_err(|_| PlatformFileDropError::ProxyMapPoisoned)?
                .insert(key, proxy);
            // SAFETY: subclassは元class直下でivarを追加せず、同じsignatureのdrag methodだけをoverrideする。
            let replaced = unsafe { AnyObject::set_class(view.as_ref(), subclass) };
            if replaced != original_class {
                if let Ok(mut proxies) = PROXIES.get_or_init(Default::default).lock() {
                    proxies.remove(&key);
                }
                return Err(PlatformFileDropError::ViewClassChanged);
            }
            if let Err(error) = register_file_urls(&view) {
                // SAFETY: 上で同じinstanceへ設定したclassを直ちに元へ戻す。
                unsafe { AnyObject::set_class(view.as_ref(), original_class) };
                if let Ok(mut proxies) = PROXIES.get_or_init(Default::default).lock() {
                    proxies.remove(&key);
                }
                return Err(error);
            }
            Ok(Self {
                view,
                original_class,
                key,
            })
        }
    }

    impl Drop for PlatformFileDrop {
        fn drop(&mut self) {
            if let Some(proxies) = PROXIES.get() {
                if let Ok(mut proxies) = proxies.lock() {
                    proxies.remove(&self.key);
                }
            }
            // SAFETY: ownerの生存中に差し替えた同じviewを元classへ戻す。
            unsafe { AnyObject::set_class(self.view.as_ref(), self.original_class) };
        }
    }

    fn drop_subclass(
        original_class: &'static AnyClass,
    ) -> Result<&'static AnyClass, PlatformFileDropError> {
        let name = c"MotoliiFileDropContentView";
        if let Some(class) = AnyClass::get(name) {
            return (class.superclass() == Some(original_class))
                .then_some(class)
                .ok_or(PlatformFileDropError::SubclassConflict);
        }
        let mut builder = ClassBuilder::new(name, original_class)
            .ok_or(PlatformFileDropError::SubclassConflict)?;
        let dragging_copy: unsafe extern "C-unwind" fn(
            *mut AnyObject,
            Sel,
            *mut AnyObject,
        ) -> usize = dragging_copy;
        let perform_drop: unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject) -> Bool =
            perform_drop;
        // SAFETY: NSDraggingDestinationのObjective-C method encodingと各function signatureは一致する。
        unsafe {
            builder.add_method::<AnyObject, _>(sel!(draggingEntered:), dragging_copy);
            builder.add_method::<AnyObject, _>(sel!(draggingUpdated:), dragging_copy);
            builder.add_method::<AnyObject, _>(sel!(performDragOperation:), perform_drop);
        }
        Ok(builder.register())
    }

    unsafe extern "C-unwind" fn dragging_copy(
        _view: *mut AnyObject,
        _cmd: Sel,
        _sender: *mut AnyObject,
    ) -> usize {
        1 // NSDragOperationCopy
    }

    unsafe extern "C-unwind" fn perform_drop(
        view: *mut AnyObject,
        _cmd: Sel,
        sender: *mut AnyObject,
    ) -> Bool {
        let Some(sender) = (unsafe { sender.as_ref() }) else {
            return Bool::NO;
        };
        // SAFETY: callback receiverはNSViewのinstance-local subclassである。
        let Some(view) = (unsafe { view.cast::<NSView>().as_ref() }) else {
            return Bool::NO;
        };
        let paths = match unsafe { file_paths(sender) } {
            Ok(paths) => paths,
            Err(()) => return Bool::NO,
        };
        // 型推論でdraggingLocationのNSPoint ABIをNSView変換へ接続する。
        let point =
            view.convertPoint_fromView(unsafe { msg_send![sender, draggingLocation] }, None);
        let height = view.bounds().size.height;
        let position = if view.isFlipped() {
            [point.x, point.y]
        } else {
            [point.x, height - point.y]
        };
        let key = view as *const NSView as usize;
        let proxy = PROXIES
            .get()
            .and_then(|proxies| proxies.lock().ok())
            .and_then(|proxies| proxies.get(&key).cloned());
        Bool::new(proxy.is_some_and(|proxy| {
            proxy
                .send_event(ProductEvent::FileDrop(HostFileDropEvent {
                    paths,
                    position,
                    terminal: HostFileDropTerminal::Perform,
                }))
                .is_ok()
        }))
    }

    unsafe fn file_paths(sender: &AnyObject) -> Result<Vec<PathBuf>, ()> {
        let file_type = unsafe { object_string(c"public.file-url") }?;
        let pasteboard: Retained<AnyObject> = unsafe { msg_send![sender, draggingPasteboard] };
        let items: Option<Retained<AnyObject>> =
            unsafe { msg_send![&*pasteboard, pasteboardItems] };
        let Some(items) = items else {
            return Ok(Vec::new());
        };
        let count: usize = unsafe { msg_send![&*items, count] };
        let url_class = AnyClass::get(c"NSURL").ok_or(())?;
        let mut paths = Vec::new();
        for index in 0..count {
            let item: Retained<AnyObject> = unsafe { msg_send![&*items, objectAtIndex: index] };
            let value: Option<Retained<AnyObject>> =
                unsafe { msg_send![&*item, stringForType: &*file_type] };
            let Some(value) = value else {
                continue;
            };
            let url: Option<Retained<AnyObject>> =
                unsafe { msg_send![url_class, URLWithString: &*value] };
            let Some(url) = url else {
                continue;
            };
            let is_file: bool = unsafe { msg_send![&*url, isFileURL] };
            if !is_file {
                continue;
            }
            let path: Option<Retained<AnyObject>> = unsafe { msg_send![&*url, path] };
            let Some(path) = path else {
                continue;
            };
            let bytes: *const std::ffi::c_char = unsafe { msg_send![&*path, UTF8String] };
            if bytes.is_null() {
                continue;
            }
            // SAFETY: NSString UTF8StringはNUL終端でpathの生存中有効。
            let path = unsafe { CStr::from_ptr(bytes) }.to_str().map_err(|_| ())?;
            paths.push(PathBuf::from(path).components().collect());
        }
        Ok(paths)
    }

    fn register_file_urls(view: &NSView) -> Result<(), PlatformFileDropError> {
        let file_type = unsafe { object_string(c"public.file-url") }
            .map_err(|_| PlatformFileDropError::FoundationClass)?;
        let array_class =
            AnyClass::get(c"NSArray").ok_or(PlatformFileDropError::FoundationClass)?;
        let types: Retained<AnyObject> =
            unsafe { msg_send![array_class, arrayWithObject: &*file_type] };
        let _: () = unsafe { msg_send![view, registerForDraggedTypes: &*types] };
        Ok(())
    }

    unsafe fn object_string(value: &CStr) -> Result<Retained<AnyObject>, ()> {
        let class = AnyClass::get(c"NSString").ok_or(())?;
        Ok(unsafe { msg_send![class, stringWithUTF8String: value.as_ptr()] })
    }

    #[derive(Debug, thiserror::Error)]
    pub(crate) enum PlatformFileDropError {
        #[error("native window handle is unavailable")]
        WindowHandle,
        #[error("native window handle is not AppKit")]
        WrongPlatform,
        #[error("native content view is unavailable")]
        MissingView,
        #[error("native file-drop proxy map lock is poisoned")]
        ProxyMapPoisoned,
        #[error("native content view class changed during file-drop registration")]
        ViewClassChanged,
        #[error("native file-drop subclass name conflicts with another superclass")]
        SubclassConflict,
        #[error("required Foundation class is unavailable")]
        FoundationClass,
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct PlatformFileDrop;

    impl PlatformFileDrop {
        pub(crate) fn new(
            _window: &winit::window::Window,
            _proxy: EventLoopProxy<ProductEvent>,
        ) -> Result<Self, PlatformFileDropError> {
            Ok(Self)
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("native file drop is unavailable on this platform")]
    pub(crate) struct PlatformFileDropError;
}

pub(crate) use platform::{PlatformFileDrop, PlatformFileDropError};
