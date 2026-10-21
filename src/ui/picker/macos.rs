//! App-owned NSOpenPanel, executed on AppKit's main thread.
use std::{
    ffi::{CStr, CString, c_char, c_void},
    mem::transmute,
    path::PathBuf,
};
type Id = *mut c_void;
#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Id;
    fn objc_msgSend();
}
#[link(name = "System")]
unsafe extern "C" {
    static mut _dispatch_main_q: c_void;
    fn dispatch_async_f(queue: Id, context: Id, work: extern "C" fn(Id));
    fn pthread_main_np() -> i32;
}
fn class(name: &str) -> Id {
    unsafe { objc_getClass(CString::new(name).unwrap().as_ptr()) }
}
fn sel(name: &str) -> Id {
    unsafe { sel_registerName(CString::new(name).unwrap().as_ptr()) }
}
fn send(object: Id, selector: &str) -> Id {
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id) -> Id>(objc_msgSend)(
            object,
            sel(selector),
        )
    }
}
fn flag(object: Id, selector: &str, value: bool) {
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id, bool)>(objc_msgSend)(
            object,
            sel(selector),
            value,
        )
    }
}
fn object(object: Id, selector: &str, value: Id) {
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id, Id)>(objc_msgSend)(
            object,
            sel(selector),
            value,
        )
    }
}
struct Request {
    folder: bool,
    title: &'static str,
    reply: std::sync::mpsc::Sender<Result<Option<PathBuf>, String>>,
}
extern "C" fn show(context: Id) {
    let request = unsafe { Box::from_raw(context.cast::<Request>()) };
    let result = std::panic::catch_unwind(|| {
        let pool = send(send(class("NSAutoreleasePool"), "alloc"), "init");
        let app = send(class("NSApplication"), "sharedApplication");
        flag(app, "activateIgnoringOtherApps:", true);
        let panel = send(class("NSOpenPanel"), "openPanel");
        flag(panel, "setCanChooseDirectories:", request.folder);
        flag(panel, "setCanChooseFiles:", !request.folder);
        flag(panel, "setAllowsMultipleSelection:", false);
        let text = CString::new(request.title).unwrap();
        let title = unsafe { transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id, *const c_char) -> Id>(objc_msgSend)(class("NSString"), sel("stringWithUTF8String:"), text.as_ptr()) };
        object(panel, "setTitle:", title);
        object(panel, "makeKeyAndOrderFront:", std::ptr::null_mut());
        let response = unsafe { transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id) -> isize>(objc_msgSend)(panel, sel("runModal")) };
        let path = if response == 1 {
            let path = send(send(panel, "URL"), "path");
            let bytes = send(path, "UTF8String").cast::<c_char>();
            if bytes.is_null() { Err(crate::ui::messages::text("native.picker_returned_no_path").into()) }
            else { unsafe { CStr::from_ptr(bytes) }.to_str().map(|s| Some(PathBuf::from(s))).map_err(|_| crate::ui::messages::text("native.non_utf_8_selected_path").into()) }
        } else { Ok(None) };
        send(pool, "drain");
        path
    }).unwrap_or_else(|_| Err(crate::ui::messages::text("native.native_picker_failed").into()));
    let _ = request.reply.send(result);
}
pub fn pick(folder: bool, dialog_title: &'static str) -> Result<Option<PathBuf>, String> {
    let (reply, receiver) = std::sync::mpsc::channel();
    let context = Box::into_raw(Box::new(Request {
        folder,
        title: dialog_title,
        reply,
    }))
    .cast();
    unsafe {
        if pthread_main_np() != 0 {
            show(context);
        } else {
            dispatch_async_f(
                std::ptr::addr_of_mut!(_dispatch_main_q).cast(),
                context,
                show,
            );
        }
    }
    receiver
        .recv()
        .map_err(|_| crate::ui::messages::text("native.native_picker_closed").to_owned())?
}
