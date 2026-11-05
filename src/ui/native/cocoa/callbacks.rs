use super::*;

#[repr(C)]
struct Block {
    isa: Id,
    flags: i32,
    reserved: i32,
    invoke: Id,
    descriptor: Id,
}
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
            close::close_if_confirmed(BRIDGE.get().unwrap());
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
pub(super) fn bridge_delegate() -> Result<Id, String> {
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
