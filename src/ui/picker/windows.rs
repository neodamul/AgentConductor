//! DD-151: Windows Common Item Dialog directly, without shell/.NET startup.
use ::windows::{
    Win32::{
        Foundation::HWND,
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
        },
        UI::{
            Shell::{
                Common::COMDLG_FILTERSPEC, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM,
                FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, FileOpenDialog, IFileOpenDialog,
                SIGDN_FILESYSPATH,
            },
            WindowsAndMessaging::GetForegroundWindow,
        },
    },
    core::{HRESULT, PCWSTR},
};
use std::path::PathBuf;
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: one CoUninitialize pairs the successful CoInitializeEx made by `dialog` on this thread, whose `Apartment` guard this is.
        unsafe { CoUninitialize() }
    }
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
pub(crate) fn path_from_utf16(path: &[u16]) -> Result<Option<PathBuf>, String> {
    let text = String::from_utf16(path)
        .map_err(|_| crate::ui::messages::text("native.selected_path_encoding_invalid"))?;
    super::picked(text.into_bytes())
}
pub fn pick(folder: bool, executable: bool) -> Result<Option<PathBuf>, String> {
    // SAFETY: GetForegroundWindow takes no arguments; the result is only an owner hint and may be null.
    let owner = unsafe { GetForegroundWindow() };
    // SAFETY: this is a fresh call on the picker thread, which `dialog` initializes as an apartment; `owner` is a handle or null.
    let result = unsafe { dialog(folder, executable, owner) }.map_err(|e| {
        crate::ui::messages::render("native.folder_dialog_failed", &[("value1", &e.to_string())])
    })?;
    result
        .map(|path| path_from_utf16(&path))
        .transpose()
        .map(Option::flatten)
}
/// # Safety
/// Call from a thread that may enter a COM apartment; `owner` is a window handle or null.
unsafe fn dialog(
    folder: bool,
    executable: bool,
    owner: HWND,
) -> ::windows::core::Result<Option<Vec<u16>>> {
    // SAFETY: no reserved pointer is passed; on success the `Apartment` guard below pairs it with CoUninitialize.
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()? };
    let _apartment = Apartment;
    // SAFETY: the class id is the system file-open dialog, created in-process after COM was initialized above.
    let dialog: IFileOpenDialog =
        unsafe { CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)? };
    let title = crate::ui::messages::text(if folder {
        "native.choose_folder_for_agentconductor"
    } else if executable {
        "native.choose_an_agent_cli_executable"
    } else {
        "native.choose_plan_document_markdown"
    });
    let title = wide(title);
    // SAFETY: `title` is a NUL-terminated UTF-16 buffer that outlives the call, and `dialog` is a live COM object.
    unsafe { dialog.SetTitle(PCWSTR(title.as_ptr()))? };
    // SAFETY: `dialog` is a live COM object created above.
    let flags = unsafe { dialog.GetOptions()? } | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
    // SAFETY: `dialog` is a live COM object created above.
    unsafe {
        dialog.SetOptions(
            flags
                | if folder {
                    FOS_PICKFOLDERS
                } else {
                    FOS_FILEMUSTEXIST
                },
        )?
    };
    if !folder {
        let filter = crate::ui::messages::text(if executable {
            "native.executable_filter"
        } else {
            "native.document_filter"
        });
        let parts: Vec<_> = filter.split('|').map(wide).collect();
        let filters: Vec<_> = parts
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| COMDLG_FILTERSPEC {
                pszName: PCWSTR(p[0].as_ptr()),
                pszSpec: PCWSTR(p[1].as_ptr()),
            })
            .collect();
        // SAFETY: `filters` points into `parts`, which outlives the call and holds NUL-terminated UTF-16 strings.
        unsafe { dialog.SetFileTypes(&filters)? };
    }
    // SAFETY: `dialog` is live and `owner` is a window handle or null; the call blocks until the dialog closes.
    if let Err(error) = unsafe { dialog.Show(Some(owner)) } {
        if error.code() == HRESULT(0x800704c7u32 as i32) {
            return Ok(None);
        }
        return Err(error);
    }
    // SAFETY: the dialog returned success, so a result item exists.
    let item = unsafe { dialog.GetResult()? };
    // SAFETY: `item` is a live COM object; the returned string is owned by us until CoTaskMemFree.
    let path = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
    let mut len = 0;
    // SAFETY: `path` is a NUL-terminated string, so reading up to the terminator stays inside its allocation.
    while unsafe { *path.0.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: `path` points to `len` initialized u16 values (the terminator excluded) and is copied before it is freed.
    let result = unsafe { std::slice::from_raw_parts(path.0, len) }.to_vec();
    // SAFETY: `path` came from GetDisplayName, which allocates with CoTaskMemAlloc, and is freed once.
    unsafe { CoTaskMemFree(Some(path.0.cast())) };
    Ok(Some(result))
}
