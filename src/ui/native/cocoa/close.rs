//! R-118 / DD-127: the window asks before closing while a run works. Keeping the run going is
//! the default; asking again within 10 s, or a page that cannot answer, closes the window.
use super::{BRIDGE, Bridge, Id, Sel, class_addMethod, objc_msgSend, sel, send0, send1, string};
use std::{
    ffi::CString,
    mem::transmute,
    sync::Mutex,
    time::{Duration, Instant},
};

static ASKED: Mutex<Option<Instant>> = Mutex::new(None);

extern "C" fn should_close(_: Id, _: Sel, _: Id) -> bool {
    std::panic::catch_unwind(|| {
        let Some(bridge) = BRIDGE.get() else {
            return true;
        };
        let mut asked = ASKED.lock().unwrap_or_else(|e| e.into_inner());
        if bridge.room.closing()
            || asked.is_some_and(|at| at.elapsed() < Duration::from_secs(10))
            || bridge.room.active_runs().is_empty()
        {
            return true;
        }
        *asked = Some(Instant::now());
        let Ok(script) = string(super::super::CLOSE_REQUESTED) else {
            return true;
        };
        // SAFETY: objc_msgSend is cast to `evaluateJavaScript:completionHandler:`'s signature; the view is live and a nil handler is allowed.
        unsafe {
            transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, Id, Id)>(
                objc_msgSend,
            )(
                bridge.view as Id,
                sel("evaluateJavaScript:completionHandler:"),
                script,
                std::ptr::null_mut(),
            );
        }
        false
    })
    .unwrap_or(true)
}

/// Adds `windowShouldClose:` to the window delegate class before it is registered.
pub(super) fn register(delegate_class: Id) {
    // BOOL is `bool` on Apple silicon and `signed char` on Intel.
    let encoding = CString::new(if cfg!(target_arch = "aarch64") {
        "B@:@"
    } else {
        "c@:@"
    })
    .unwrap();
    // SAFETY: the class is not yet registered; the selector, encoding and `should_close` signature agree.
    unsafe {
        class_addMethod(
            delegate_class,
            sel("windowShouldClose:"),
            transmute::<extern "C" fn(Id, Sel, Id) -> bool, unsafe extern "C" fn()>(should_close),
            encoding.as_ptr(),
        );
    }
}

/// After a reply on the main thread: the operator confirmed closing.
pub(super) fn close_if_confirmed(bridge: &Bridge) {
    if bridge.room.closing() {
        send1(
            send0(bridge.view as Id, "window"),
            "performClose:",
            std::ptr::null_mut(),
        );
    }
}
