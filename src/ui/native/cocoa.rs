mod brand;
mod menu;
use std::{
    ffi::{CStr, CString, c_char, c_void},
    mem::transmute,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
#[used]
#[unsafe(link_section = "__TEXT,__info_plist")]
static INFO_PLIST: [u8; include_bytes!("../Info.plist").len()] = *include_bytes!("../Info.plist");

type Id = *mut c_void;
type Sel = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    origin: Point,
    size: Size,
}
#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}
#[link(name = "WebKit", kind = "framework")]
unsafe extern "C" {}
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_msgSend();
    fn objc_allocateClassPair(superclass: Id, name: *const c_char, extra: usize) -> Id;
    fn objc_registerClassPair(class: Id);
    fn class_addMethod(
        class: Id,
        selector: Sel,
        implementation: unsafe extern "C" fn(),
        encoding: *const c_char,
    ) -> bool;
}
#[link(name = "System")]
unsafe extern "C" {
    static mut _dispatch_main_q: c_void;
    fn dispatch_async_f(queue: Id, context: Id, work: extern "C" fn(Id));
    fn _Block_copy(block: Id) -> Id;
    fn _Block_release(block: Id);
}
#[repr(C)]
struct Block {
    isa: Id,
    flags: i32,
    reserved: i32,
    invoke: Id,
    descriptor: Id,
}
struct Bridge {
    room: Arc<super::super::Room>,
    view: usize,
    closed: AtomicBool,
    active: AtomicUsize,
}
static BRIDGE: OnceLock<Bridge> = OnceLock::new();
struct Reply {
    block: usize,
    value: Result<String, String>,
}
fn reply(block: Id, value: Result<String, String>) {
    let (value, error) = match value {
        Ok(value) => (
            string(&value).unwrap_or(std::ptr::null_mut()),
            std::ptr::null_mut(),
        ),
        Err(error) => (
            std::ptr::null_mut(),
            string(&error).unwrap_or(std::ptr::null_mut()),
        ),
    };
    // SAFETY: `block` is the live WebKit reply block its caller holds; `invoke` has the (block, value, error) signature.
    unsafe {
        transmute::<Id, extern "C" fn(Id, Id, Id)>((*(block as *const Block)).invoke)(
            block, value, error,
        );
    }
}
extern "C" fn deliver(context: Id) {
    let _ = std::panic::catch_unwind(|| {
        // SAFETY: `context` is the `Box::into_raw` pointer built in `message`, delivered to exactly one call of `deliver`.
        let result = unsafe { Box::from_raw(context as *mut Reply) };
        if BRIDGE
            .get()
            .is_some_and(|b| !b.closed.load(Ordering::Acquire))
        {
            let pool = send0(send0(class("NSAutoreleasePool"), "alloc"), "init");
            reply(result.block as Id, result.value);
            send0(pool, "drain");
        }
        if let Some(bridge) = BRIDGE.get() {
            bridge.active.fetch_sub(1, Ordering::AcqRel);
        }
        // SAFETY: `result.block` is the copy from `_Block_copy` in `message`; this is its only release.
        unsafe {
            _Block_release(result.block as Id);
        }
    });
}
fn boolean(object: Id, name: &str) -> bool {
    // SAFETY: objc_msgSend is cast to `fn(Id, Sel) -> bool`, the signature of the BOOL getters sent here.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel) -> bool>(objc_msgSend)(
            object,
            sel(name),
        )
    }
}
fn text(object: Id) -> Result<String, String> {
    // SAFETY: objc_msgSend is cast to `isKindOfClass:`'s `fn(Id, Sel, Id) -> bool`.
    let is_string =
        unsafe {
            transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id) -> bool>(
                objc_msgSend,
            )(object, sel("isKindOfClass:"), class("NSString"))
        };
    if !is_string {
        return Err(crate::ui::messages::text("native.native_message_must_be_a_utf_8_json").into());
    }
    // SAFETY: objc_msgSend is cast to `lengthOfBytesUsingEncoding:`'s `fn(Id, Sel, usize) -> usize`.
    let size = unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, usize) -> usize>(
            objc_msgSend,
        )(object, sel("lengthOfBytesUsingEncoding:"), 4)
    };
    if size > 64 * 1024 {
        return Err(crate::ui::messages::text("native.native_request_exceeds_64_kib").into());
    }
    let bytes = send0(object, "UTF8String") as *const c_char;
    if bytes.is_null() {
        return Err(crate::ui::messages::text("native.invalid_native_string").into());
    }
    // SAFETY: `bytes` is non-null (checked above) and NUL-terminated, being the `UTF8String` of a live NSString.
    let bytes = unsafe { CStr::from_ptr(bytes).to_bytes() };
    if bytes.len() != size {
        return Err(crate::ui::messages::text("native.embedded_nul_in_native_message").into());
    }
    String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
}
extern "C" fn message(_: Id, _: Sel, _: Id, message: Id, block: Id) {
    let outcome = std::panic::catch_unwind(|| {
        let bridge = BRIDGE.get().ok_or(crate::ui::messages::text(
            "native.native_bridge_unavailable",
        ))?;
        if bridge.closed.load(Ordering::Acquire)
            || send0(message, "webView") as usize != bridge.view
            || !boolean(send0(message, "frameInfo"), "isMainFrame")
        {
            return Err(crate::ui::messages::text("native.owned_main_frame_required").to_string());
        }
        let url = send0(send0(send0(message, "frameInfo"), "request"), "URL");
        if url.is_null() || text(send0(url, "absoluteString"))? != "about:blank" {
            return Err(crate::ui::messages::text("native.embedded_document_required").to_string());
        }
        let input = text(send0(message, "body"))?;
        if bridge.active.fetch_add(1, Ordering::AcqRel) >= 8 {
            bridge.active.fetch_sub(1, Ordering::AcqRel);
            return Err(crate::ui::messages::text("native.native_controller_is_busy").to_string());
        }
        // SAFETY: `block` is the live reply block WebKit passed to this call; the copy is released exactly once.
        let copied = unsafe { _Block_copy(block) } as usize;
        let spawned = std::thread::Builder::new()
            .name("room-request".into())
            .spawn(move || {
                let value = std::panic::catch_unwind(|| bridge.room.dispatch(input.as_bytes()))
                    .unwrap_or_else(|_| {
                        Err(crate::ui::messages::text("native.native_controller_failed").into())
                    })
                    .map(|value| value.to_string());
                if bridge.closed.load(Ordering::Acquire) {
                    bridge.active.fetch_sub(1, Ordering::AcqRel);
                    // SAFETY: `copied` is the copy made above and is released here instead of being delivered.
                    unsafe {
                        _Block_release(copied as Id);
                    }
                } else {
                    let result = Box::into_raw(Box::new(Reply {
                        block: copied,
                        value,
                    }));
                    // SAFETY: the main queue lives for the process; `result` is a leaked Box<Reply> that `deliver` takes back once.
                    unsafe {
                        dispatch_async_f(
                            std::ptr::addr_of_mut!(_dispatch_main_q),
                            result as Id,
                            deliver,
                        );
                    }
                }
            });
        if let Err(e) = spawned {
            bridge.active.fetch_sub(1, Ordering::AcqRel);
            // SAFETY: the thread never started, so this is the only release of `copied`.
            unsafe {
                _Block_release(copied as Id);
            }
            return Err(e.to_string());
        }
        Ok(())
    });
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(error)) => reply(block, Err(error)),
        Err(_) => reply(
            block,
            Err(crate::ui::messages::text("native.native_boundary_failed").into()),
        ),
    }
}
extern "C" fn navigation(_: Id, _: Sel, _: Id, action: Id, block: Id) {
    let allowed = std::panic::catch_unwind(|| {
        let url = send0(send0(action, "request"), "URL");
        !url.is_null() && text(send0(url, "absoluteString")).is_ok_and(|url| url == "about:blank")
    })
    .unwrap_or(false);
    // SAFETY: `block` is the live decision handler WebKit passed in; `invoke` takes (block, policy as isize).
    unsafe {
        transmute::<Id, extern "C" fn(Id, isize)>((*(block as *const Block)).invoke)(
            block,
            if allowed { 1 } else { 0 },
        );
    }
}
fn bridge_delegate() -> Result<Id, String> {
    let name = CString::new("AgentConductorMessageDelegate").unwrap();
    // SAFETY: `name` is a NUL-terminated CString alive for the call and the superclass is NSObject.
    let delegate = unsafe { objc_allocateClassPair(class("NSObject"), name.as_ptr(), 0) };
    if delegate.is_null() {
        return Err(crate::ui::messages::text(
            "native.native_message_delegate_initialization_failed",
        )
        .into());
    }
    // SAFETY: the class is not yet registered; each selector, encoding and handler signature agree.
    unsafe {
        class_addMethod(
            delegate,
            sel("userContentController:didReceiveScriptMessage:replyHandler:"),
            transmute::<extern "C" fn(Id, Sel, Id, Id, Id), unsafe extern "C" fn()>(message),
            CString::new("v@:@@@?").unwrap().as_ptr(),
        );
        class_addMethod(
            delegate,
            sel("webView:decidePolicyForNavigationAction:decisionHandler:"),
            transmute::<extern "C" fn(Id, Sel, Id, Id, Id), unsafe extern "C" fn()>(navigation),
            CString::new("v@:@@@?").unwrap().as_ptr(),
        );
        objc_registerClassPair(delegate);
    }
    Ok(send0(delegate, "new"))
}
fn class(name: &str) -> Id {
    let name = CString::new(name).unwrap();
    // SAFETY: `name` is a NUL-terminated CString alive for the call.
    unsafe { objc_getClass(name.as_ptr()) }
}
fn sel(name: &str) -> Sel {
    let name = CString::new(name).unwrap();
    // SAFETY: `name` is a NUL-terminated CString alive for the call.
    unsafe { sel_registerName(name.as_ptr()) }
}
fn send0(object: Id, name: &str) -> Id {
    // SAFETY: objc_msgSend is cast to `fn(Id, Sel) -> Id`; `object` is a live object or nil.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel) -> Id>(objc_msgSend)(
            object,
            sel(name),
        )
    }
}
fn send1(object: Id, name: &str, arg: Id) -> Id {
    // SAFETY: objc_msgSend is cast to `fn(Id, Sel, Id) -> Id`; `object` is a live object or nil.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id) -> Id>(objc_msgSend)(
            object,
            sel(name),
            arg,
        )
    }
}
fn string(text: &str) -> Result<Id, String> {
    let text = CString::new(text).map_err(|e| e.to_string())?;
    // SAFETY: objc_msgSend is cast to `stringWithUTF8String:`'s signature; `text` is a NUL-terminated CString alive for the call.
    Ok(unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, *const c_char) -> Id>(
            objc_msgSend,
        )(
            class("NSString"),
            sel("stringWithUTF8String:"),
            text.as_ptr(),
        )
    })
}
extern "C" fn close_window(_: Id, _: Sel, _: Id) {
    if let Some(bridge) = BRIDGE.get() {
        bridge.closed.store(true, Ordering::Release);
    }
    send1(
        send0(class("NSApplication"), "sharedApplication"),
        "stop:",
        std::ptr::null_mut(),
    );
}
pub fn open(room: Arc<super::super::Room>) -> Result<(), String> {
    std::hint::black_box(&INFO_PLIST);
    let pool = send0(send0(class("NSAutoreleasePool"), "alloc"), "init");
    let app = send0(class("NSApplication"), "sharedApplication");
    // SAFETY: objc_msgSend is cast to `setActivationPolicy:`'s `fn(Id, Sel, isize) -> bool`; `app` is the shared application.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, isize) -> bool>(
            objc_msgSend,
        )(app, sel("setActivationPolicy:"), 0);
    }
    brand::install(app)?;
    send0(app, "finishLaunching");
    let rect = Rect {
        origin: Point { x: 80., y: 80. },
        size: Size {
            width: 1280.,
            height: 860.,
        },
    };
    // SAFETY: objc_msgSend is cast to `initWithContentRect:styleMask:backing:defer:`'s signature on a freshly allocated NSWindow.
    let window = unsafe {
        transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(Id, Sel, Rect, usize, usize, bool) -> Id,
        >(objc_msgSend)(
            send0(class("NSWindow"), "alloc"),
            sel("initWithContentRect:styleMask:backing:defer:"),
            rect,
            15,
            2,
            false,
        )
    };
    if window.is_null() {
        return Err(crate::ui::messages::text("native.native_window_initialization_failed").into());
    }
    send1(
        window,
        "setTitle:",
        string(crate::ui::messages::text(
            "native.agentconductor_agent_room",
        ))?,
    );
    // SAFETY: objc_msgSend is cast to `setMinSize:` and `setReleasedWhenClosed:`'s signatures; `window` is live.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Size)>(objc_msgSend)(
            window,
            sel("setMinSize:"),
            Size {
                width: 960.,
                height: 640.,
            },
        );
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, bool)>(objc_msgSend)(
            window,
            sel("setReleasedWhenClosed:"),
            false,
        );
    }
    let configuration = send0(class("WKWebViewConfiguration"), "new");
    send1(
        configuration,
        "setWebsiteDataStore:",
        send0(class("WKWebsiteDataStore"), "nonPersistentDataStore"),
    );
    // SAFETY: objc_msgSend is cast to `initWithFrame:configuration:`'s signature on a freshly allocated WKWebView.
    let view = unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Rect, Id) -> Id>(
            objc_msgSend,
        )(
            send0(class("WKWebView"), "alloc"),
            sel("initWithFrame:configuration:"),
            rect,
            configuration,
        )
    };
    if view.is_null() {
        return Err(crate::ui::messages::text("native.system_webkit_initialization_failed").into());
    }
    send1(window, "setContentView:", view);
    let document = room.document()?;
    BRIDGE
        .set(Bridge {
            room,
            view: view as usize,
            closed: AtomicBool::new(false),
            active: AtomicUsize::new(0),
        })
        .map_err(|_| {
            crate::ui::messages::text("native.a_native_room_has_already_been_opened_in")
        })?;
    let message_delegate = bridge_delegate()?;
    let content = send0(configuration, "userContentController");
    // SAFETY: objc_msgSend is cast to `addScriptMessageHandlerWithReply:contentWorld:name:`'s signature; every argument is live.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id, Id, Id)>(
            objc_msgSend,
        )(
            content,
            sel("addScriptMessageHandlerWithReply:contentWorld:name:"),
            message_delegate,
            send0(class("WKContentWorld"), "pageWorld"),
            string("agentconductor")?,
        );
    }
    send1(view, "setNavigationDelegate:", message_delegate);
    // SAFETY: objc_msgSend is cast to `loadHTMLString:baseURL:`'s signature; `view` is live and the base URL is nil.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id, Id) -> Id>(
            objc_msgSend,
        )(
            view,
            sel("loadHTMLString:baseURL:"),
            string(&document)?,
            std::ptr::null_mut(),
        );
    }
    let name = CString::new("AgentConductorWindowDelegate").unwrap();
    // SAFETY: `name` is a NUL-terminated CString alive for the call and the superclass is NSObject.
    let delegate_class = unsafe { objc_allocateClassPair(class("NSObject"), name.as_ptr(), 0) };
    if delegate_class.is_null() {
        return Err(crate::ui::messages::text(
            "native.native_window_delegate_initialization_failed",
        )
        .into());
    }
    let encoding = CString::new("v@:@").unwrap();
    // SAFETY: the class is not yet registered; the selector, `v@:@` encoding and `close_window` signature agree.
    unsafe {
        class_addMethod(
            delegate_class,
            sel("windowWillClose:"),
            transmute::<extern "C" fn(Id, Sel, Id), unsafe extern "C" fn()>(close_window),
            encoding.as_ptr(),
        );
        objc_registerClassPair(delegate_class);
    }
    let delegate = send0(delegate_class, "new");
    send1(window, "setDelegate:", delegate);
    menu::install(app, window)?;
    send0(window, "center");
    send1(window, "makeKeyAndOrderFront:", std::ptr::null_mut());
    // SAFETY: objc_msgSend is cast to `activateIgnoringOtherApps:`'s `fn(Id, Sel, bool)`; `app` is live.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, bool)>(objc_msgSend)(
            app,
            sel("activateIgnoringOtherApps:"),
            true,
        );
    }
    send0(app, "run");
    send0(delegate, "release");
    send1(view, "setNavigationDelegate:", std::ptr::null_mut());
    // SAFETY: objc_msgSend is cast to `removeScriptMessageHandlerForName:contentWorld:`'s signature; `content` is still retained.
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id, Id)>(objc_msgSend)(
            content,
            sel("removeScriptMessageHandlerForName:contentWorld:"),
            string("agentconductor")?,
            send0(class("WKContentWorld"), "pageWorld"),
        );
    }
    send0(message_delegate, "release");
    send0(view, "release");
    send0(configuration, "release");
    send0(window, "release");
    send0(pool, "drain");
    Ok(())
}
