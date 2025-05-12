mod index;

pub(super) const fn pages() -> &'static [(&'static str, fn() -> uibeam::UI)] {
    &[
        ("/", index::page),
        ("/test", index::page),
        ("/test/test2", index::page),
    ]
}
