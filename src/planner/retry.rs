pub(super) fn scope_error(error: &str) -> bool {
    error.contains("unsafe or overlapping scope") || error.contains(crate::plan::EMPTY_PATHS)
}
