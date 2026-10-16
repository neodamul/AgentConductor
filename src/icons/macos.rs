use super::Provider;
use std::ffi::{CString, c_char, c_void};
use std::mem::transmute;

type Id = *mut c_void;
#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Id;
    fn sel_registerName(name: *const c_char) -> Id;
    fn objc_msgSend();
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}
fn class(name: &str) -> Id {
    let name = CString::new(name).unwrap();
    unsafe { objc_getClass(name.as_ptr()) }
}
fn sel(name: &str) -> Id {
    let name = CString::new(name).unwrap();
    unsafe { sel_registerName(name.as_ptr()) }
}
fn send0(object: Id, name: &str) -> Id {
    unsafe {
        transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id) -> Id>(objc_msgSend)(
            object,
            sel(name),
        )
    }
}
fn send1(object: Id, name: &str, arg: Id) -> Id {
    unsafe {
        transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, Id) -> Id>(objc_msgSend)(
            object,
            sel(name),
            arg,
        )
    }
}
fn string(text: &str) -> Id {
    let text = CString::new(text).unwrap();
    unsafe {
        transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, *const c_char) -> Id>(
            objc_msgSend,
        )(
            class("NSString"),
            sel("stringWithUTF8String:"),
            text.as_ptr(),
        )
    }
}

pub fn icon(provider: Provider) -> Option<String> {
    let bundle = match provider {
        Provider::Claude => "com.anthropic.claudefordesktop",
        Provider::Codex => "com.openai.codex",
        Provider::OpenCode => "ai.opencode.desktop",
        Provider::Custom => return None,
    };
    let pool = unsafe { objc_autoreleasePoolPush() };
    let result = render(bundle);
    unsafe { objc_autoreleasePoolPop(pool) };
    result
}

fn render(bundle: &str) -> Option<String> {
    let workspace = send0(class("NSWorkspace"), "sharedWorkspace");
    let url = send1(
        workspace,
        "URLForApplicationWithBundleIdentifier:",
        string(bundle),
    );
    if url.is_null() {
        return None;
    }
    let image = send1(workspace, "iconForFile:", send0(url, "path"));
    if image.is_null() {
        return None;
    }
    let side = 64isize;
    let rep = unsafe {
        transmute::<
            unsafe extern "C" fn(),
            extern "C" fn(
                Id,
                Id,
                Id,
                isize,
                isize,
                isize,
                isize,
                bool,
                bool,
                Id,
                isize,
                isize,
            ) -> Id,
        >(objc_msgSend)(
            send0(class("NSBitmapImageRep"), "alloc"),
            sel(
                "initWithBitmapDataPlanes:pixelsWide:pixelsHigh:bitsPerSample:samplesPerPixel:hasAlpha:isPlanar:colorSpaceName:bytesPerRow:bitsPerPixel:",
            ),
            std::ptr::null_mut(),
            side,
            side,
            8,
            4,
            true,
            false,
            string("NSDeviceRGBColorSpace"),
            side * 4,
            32,
        )
    };
    if rep.is_null() {
        return None;
    }
    let context = send1(
        class("NSGraphicsContext"),
        "graphicsContextWithBitmapImageRep:",
        rep,
    );
    let graphics = class("NSGraphicsContext");
    send0(graphics, "saveGraphicsState");
    send1(graphics, "setCurrentContext:", context);
    let target = Rect {
        x: 0.0,
        y: 0.0,
        width: side as f64,
        height: side as f64,
    };
    let zero = Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
    unsafe {
        transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, Rect, Rect, usize, f64)>(
            objc_msgSend,
        )(
            image,
            sel("drawInRect:fromRect:operation:fraction:"),
            target,
            zero,
            2, // NSCompositingOperationSourceOver
            1.0,
        );
    }
    send0(context, "flushGraphics");
    send0(graphics, "restoreGraphicsState");
    // NSBitmapImageFileTypePNG = 4
    let data = unsafe {
        transmute::<unsafe extern "C" fn(), extern "C" fn(Id, Id, usize, Id) -> Id>(objc_msgSend)(
            rep,
            sel("representationUsingType:properties:"),
            4,
            send0(class("NSDictionary"), "dictionary"),
        )
    };
    let result = if data.is_null() {
        None
    } else {
        let length = send0(data, "length") as usize;
        let bytes = send0(data, "bytes") as *const u8;
        (!bytes.is_null())
            .then(|| unsafe { std::slice::from_raw_parts(bytes, length) }.to_vec())
            .and_then(|png| super::data_url(&png).ok())
    };
    send0(rep, "release");
    result
}
