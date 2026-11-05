mod brand;
mod callbacks;
use callbacks::bridge_delegate;
mod close;
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
struct Bridge {
    room: Arc<super::super::Room>,
    view: usize,
    closed: AtomicBool,
    active: AtomicUsize,
}
static BRIDGE: OnceLock<Bridge> = OnceLock::new();
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
    }
    close::register(delegate_class);
    // SAFETY: every method of the delegate class was added above.
    unsafe { objc_registerClassPair(delegate_class) };
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
