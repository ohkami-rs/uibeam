mod index;

pub(super) const fn pages() -> &'static [(&'static str, fn() -> uibeam::UI)] {
    &[
        ("/", index::page),
    ]
}
