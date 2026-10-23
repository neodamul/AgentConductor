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
    // SAFETY: the temporary CString lives to the end of this expression, past the call.
    unsafe { objc_getClass(CString::new(name).unwrap().as_ptr()) }
}
fn sel(name: &str) -> Id {
    // SAFETY: the temporary CString lives to the end of this expression, past the call.
    unsafe { sel_registerName(CString::new(name).unwrap().as_ptr()) }
}
fn send(object: Id, selector: &str) -> Id {
    // SAFETY: objc_msgSend is cast to `fn(Id, Id) -> Id`; `object` is a live object or nil.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id) -> Id>(objc_msgSend)(
            object,
            sel(selector),
        )
    }
}
fn flag(object: Id, selector: &str, value: bool) {
    // SAFETY: objc_msgSend is cast to `fn(Id, Id, bool)`, the signature of the BOOL setters sent here.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id, bool)>(objc_msgSend)(
            object,
            sel(selector),
            value,
        )
    }
}
fn object(object: Id, selector: &str, value: Id) {
    // SAFETY: objc_msgSend is cast to `fn(Id, Id, Id)`, the signature of the object setters sent here.
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
    executable: bool,
    reply: std::sync::mpsc::Sender<Result<Option<PathBuf>, String>>,
}
extern "C" fn show(context: Id) {
    // SAFETY: `context` is the `Box::into_raw` pointer built in `pick`, passed to exactly one call of `show`.
    let request = unsafe { Box::from_raw(context.cast::<Request>()) };
    let result = std::panic::catch_unwind(|| {
        let pool = send(send(class("NSAutoreleasePool"), "alloc"), "init");
        let app = send(class("NSApplication"), "sharedApplication");
        flag(app, "activateIgnoringOtherApps:", true);
        let panel = send(class("NSOpenPanel"), "openPanel");
        flag(panel, "setCanChooseDirectories:", request.folder);
        flag(panel, "setCanChooseFiles:", !request.folder);
        flag(panel, "setAllowsMultipleSelection:", false);
        let title = if request.folder { crate::ui::messages::text("native.choose_a_project_folder") } else if request.executable { crate::ui::messages::text("native.choose_an_agent_cli_executable") } else { crate::ui::messages::text("native.choose_a_plan_document") };
        let text = CString::new(title).unwrap();
// SAFETY: objc_msgSend is cast to `stringWithUTF8String:`; `text` is a NUL-terminated CString alive for the call.
        let title = unsafe { transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id, *const c_char) -> Id>(objc_msgSend)(class("NSString"), sel("stringWithUTF8String:"), text.as_ptr()) };
        object(panel, "setTitle:", title);
        object(panel, "makeKeyAndOrderFront:", std::ptr::null_mut());
// SAFETY: objc_msgSend is cast to `runModal`'s `fn(Id, Id) -> isize`; `panel` is live and this runs on the main thread.
        let response = unsafe { transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Id) -> isize>(objc_msgSend)(panel, sel("runModal")) };
        let path = if response == 1 {
            let path = send(send(panel, "URL"), "path");
            let bytes = send(path, "UTF8String").cast::<c_char>();
            if bytes.is_null() { Err(crate::ui::messages::text("native.picker_returned_no_path").into()) }
// SAFETY: `bytes` is non-null (checked above) and NUL-terminated, the `UTF8String` of a live NSString.
            else { unsafe { CStr::from_ptr(bytes) }.to_str().map(|s| Some(PathBuf::from(s))).map_err(|_| crate::ui::messages::text("native.non_utf_8_selected_path").into()) }
        } else { Ok(None) };
        send(pool, "drain");
        path
    }).unwrap_or_else(|_| Err(crate::ui::messages::text("native.native_picker_failed").into()));
    let _ = request.reply.send(result);
}
pub fn pick(folder: bool, executable: bool) -> Result<Option<PathBuf>, String> {
    let (reply, receiver) = std::sync::mpsc::channel();
    let context = Box::into_raw(Box::new(Request {
        folder,
        executable,
        reply,
    }))
    .cast();
    // SAFETY: pthread_main_np takes no arguments; `context` is the Box<Request> pointer, consumed by exactly one `show` call, and the main queue lives for the process.
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
