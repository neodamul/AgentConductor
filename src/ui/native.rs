#[cfg(target_os = "macos")]
mod cocoa {
    use std::{
        ffi::{CString, c_char, c_void},
        mem::transmute,
    };
    #[used]
    #[unsafe(link_section = "__TEXT,__info_plist")]
    static INFO_PLIST: [u8; include_bytes!("Info.plist").len()] = *include_bytes!("Info.plist");

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
            transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id) -> Id>(
                objc_msgSend,
            )(object, sel(name), arg)
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
    fn menu_item(title: &str, key: &str, action: &str, target: Id) -> Result<Id, String> {
        let item = send0(class("NSMenuItem"), "new");
        send1(item, "setTitle:", string(title)?);
        send1(item, "setKeyEquivalent:", string(key)?);
        send1(item, "setAction:", sel(action));
        send1(item, "setTarget:", target);
        Ok(item)
    }
    fn menu(app: Id, window: Id) -> Result<(), String> {
        let main = send0(class("NSMenu"), "new");
        for (title, items) in [
            (
                "AgentConductor",
                vec![("Quit AgentConductor", "q", "performClose:", window)],
            ),
            (
                "Edit",
                vec![
                    ("Undo", "z", "undo:", std::ptr::null_mut()),
                    ("Cut", "x", "cut:", std::ptr::null_mut()),
                    ("Copy", "c", "copy:", std::ptr::null_mut()),
                    ("Paste", "v", "paste:", std::ptr::null_mut()),
                    ("Select All", "a", "selectAll:", std::ptr::null_mut()),
                ],
            ),
            (
                "Window",
                vec![("Close Window", "w", "performClose:", window)],
            ),
        ] {
            let parent = send0(class("NSMenuItem"), "new");
            send1(parent, "setTitle:", string(title)?);
            let submenu = send0(class("NSMenu"), "new");
            send1(submenu, "setTitle:", string(title)?);
            for (title, key, action, target) in items {
                let item = menu_item(title, key, action, target)?;
                send1(submenu, "addItem:", item);
                send0(item, "release");
            }
            send1(parent, "setSubmenu:", submenu);
            send1(main, "addItem:", parent);
            send0(submenu, "release");
            send0(parent, "release");
        }
        send1(app, "setMainMenu:", main);
        send0(main, "release");
        Ok(())
    }
    extern "C" fn close_window(_: Id, _: Sel, _: Id) {
        send1(
            send0(class("NSApplication"), "sharedApplication"),
            "stop:",
            std::ptr::null_mut(),
        );
    }
    pub fn open(url: &str) -> Result<(), String> {
        std::hint::black_box(&INFO_PLIST);
        let pool = send0(send0(class("NSAutoreleasePool"), "alloc"), "init");
        let app = send0(class("NSApplication"), "sharedApplication");
        unsafe {
            transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, isize) -> bool>(
                objc_msgSend,
            )(app, sel("setActivationPolicy:"), 0);
        }
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
            return Err("native window initialization failed".into());
        }
        send1(window, "setTitle:", string("AgentConductor · Agent Room")?);
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
            return Err("system WebKit initialization failed".into());
        }
        send1(window, "setContentView:", view);
        let request = send1(
            class("NSURLRequest"),
            "requestWithURL:",
            send1(class("NSURL"), "URLWithString:", string(url)?),
        );
        send1(view, "loadRequest:", request);
        let name = CString::new("AgentConductorWindowDelegate").unwrap();
        let delegate_class = unsafe { objc_allocateClassPair(class("NSObject"), name.as_ptr(), 0) };
        if delegate_class.is_null() {
            return Err("native window delegate initialization failed".into());
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
        menu(app, window)?;
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
        send0(view, "release");
        send0(configuration, "release");
        send0(window, "release");
        send0(pool, "drain");
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub fn open(url: &str) -> Result<(), String> {
    cocoa::open(url)
}
#[cfg(not(target_os = "macos"))]
pub fn open(url: &str) -> Result<(), String> {
    std::process::Command::new(if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    })
    .arg(url)
    .spawn()
    .map_err(|e| e.to_string())?;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
