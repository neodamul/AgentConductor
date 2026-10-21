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
pub fn pick(folder: bool, dialog_title: &'static str) -> Result<Option<PathBuf>, String> {
    let owner = unsafe { GetForegroundWindow() };
    let result = unsafe { dialog(folder, dialog_title, owner) }.map_err(|e| {
        crate::ui::messages::render("native.folder_dialog_failed", &[("value1", &e.to_string())])
    })?;
    result
        .map(|path| path_from_utf16(&path))
        .transpose()
        .map(Option::flatten)
}
unsafe fn dialog(
    folder: bool,
    dialog_title: &'static str,
    owner: HWND,
) -> ::windows::core::Result<Option<Vec<u16>>> {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()? };
    let _apartment = Apartment;
    let dialog: IFileOpenDialog =
        unsafe { CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)? };
    let title = wide(dialog_title);
    unsafe { dialog.SetTitle(PCWSTR(title.as_ptr()))? };
    let flags = unsafe { dialog.GetOptions()? } | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
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
        unsafe { dialog.SetFileTypes(&filters)? };
    }
    if let Err(error) = unsafe { dialog.Show(Some(owner)) } {
        if error.code() == HRESULT(0x800704c7u32 as i32) {
            return Ok(None);
        }
        return Err(error);
    }
    let item = unsafe { dialog.GetResult()? };
    let path = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
    let mut len = 0;
    while unsafe { *path.0.add(len) } != 0 {
        len += 1;
    }
    let result = unsafe { std::slice::from_raw_parts(path.0, len) }.to_vec();
    unsafe { CoTaskMemFree(Some(path.0.cast())) };
    Ok(Some(result))
}
