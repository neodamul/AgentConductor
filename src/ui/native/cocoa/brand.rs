//! Project-owned artwork lives inside the executable, including direct CLI launches.
use super::*;
pub(super) fn install(app: Id) -> Result<(), String> {
    let bytes = include_bytes!("../../assets/ac.png");
    let data = unsafe {
        transmute::<unsafe extern "C" fn(), unsafe extern "C" fn(Id, Sel, *const u8, usize) -> Id>(
            objc_msgSend,
        )(
            class("NSData"),
            sel("dataWithBytes:length:"),
            bytes.as_ptr(),
            bytes.len(),
        )
    };
    let image = send1(send0(class("NSImage"), "alloc"), "initWithData:", data);
    if image.is_null() {
        return Err(
            crate::ui::messages::text("native.ac_application_icon_initialization_failed").into(),
        );
    }
    send1(app, "setApplicationIconImage:", image);
    send0(image, "release");
    Ok(())
}
