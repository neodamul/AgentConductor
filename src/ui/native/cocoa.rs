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
    unsafe {
        transmute::<Id, extern "C" fn(Id, Id, Id)>((*(block as *const Block)).invoke)(
            block, value, error,
        );
    }
}
extern "C" fn deliver(context: Id) {
    let _ = std::panic::catch_unwind(|| {
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
        unsafe {
            _Block_release(result.block as Id);
        }
    });
}
fn boolean(object: Id, name: &str) -> bool {
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel) -> bool>(objc_msgSend)(
            object,
            sel(name),
        )
    }
}
fn text(object: Id) -> Result<String, String> {
    let is_string =
        unsafe {
            transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id) -> bool>(
                objc_msgSend,
            )(object, sel("isKindOfClass:"), class("NSString"))
        };
    if !is_string {
        return Err(crate::ui::messages::text("native.native_message_must_be_a_utf_8_json").into());
    }
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
                    unsafe {
                        _Block_release(copied as Id);
                    }
                } else {
                    let result = Box::into_raw(Box::new(Reply {
                        block: copied,
                        value,
                    }));
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
    unsafe {
        transmute::<Id, extern "C" fn(Id, isize)>((*(block as *const Block)).invoke)(
            block,
            if allowed { 1 } else { 0 },
        );
    }
}
fn bridge_delegate() -> Result<Id, String> {
    let name = CString::new("AgentConductorMessageDelegate").unwrap();
    let delegate = unsafe { objc_allocateClassPair(class("NSObject"), name.as_ptr(), 0) };
    if delegate.is_null() {
        return Err(crate::ui::messages::text(
            "native.native_message_delegate_initialization_failed",
        )
        .into());
    }
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
    unsafe { objc_getClass(name.as_ptr()) }
}
fn sel(name: &str) -> Sel {
    let name = CString::new(name).unwrap();
    unsafe { sel_registerName(name.as_ptr()) }
}
fn send0(object: Id, name: &str) -> Id {
    unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel) -> Id>(objc_msgSend)(
            object,
            sel(name),
        )
    }
}
fn send1(object: Id, name: &str, arg: Id) -> Id {
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
    let delegate_class = unsafe { objc_allocateClassPair(class("NSObject"), name.as_ptr(), 0) };
    if delegate_class.is_null() {
        return Err(crate::ui::messages::text(
            "native.native_window_delegate_initialization_failed",
        )
        .into());
    }
    let encoding = CString::new("v@:@").unwrap();
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
