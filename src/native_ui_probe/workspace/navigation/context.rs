#[derive(Clone, Copy, Debug)]
pub struct NavigationContext<'a> {
    pub current_name: &'a str,
    pub current_path: &'a str,
    pub can_change: bool,
    pub dirty: bool,
    pub show_cache: bool,
    pub language: &'a str,
}
