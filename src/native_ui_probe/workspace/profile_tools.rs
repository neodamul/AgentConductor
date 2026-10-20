mod editor;
mod helpers;
mod view;

pub use editor::{Editor, Event};
pub use helpers::{
    merge_catalogs, metadata_request, metadata_running, profile_save_request, remove_profile,
    remove_references, rename_references, switch_profile,
};
pub use view::show;
