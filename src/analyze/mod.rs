use rustdoc_denormalize::Item;

// Most items in the stdlib can be const probably. It's mainly not things which
// touch host APIs, globals, or directly allocate on the heap. Though the heap
// ones we can probably overcome eventually, so for now we're counting them.
pub fn count_const_items(items: &[Item]) -> (usize, usize) {
    let exclude_paths = &["std::os", "std::fs", "std::net", "std::process"];
    let should_exclude = |item: &&Item| false;
    let count_current = |item: &&Item| item.is_const;
    count_items(items, exclude_paths, should_exclude, count_current)
}

// Async items are a bit trickier. We probably don't want async ops. But we
// do want to count every single generic param. But also make sure we include
// all of net, fs, and most traits + trait impls.
pub fn count_async_items(items: &[Item]) -> (usize, usize) {
    let exclude_paths = &[
        "core::ops",
        "std::thread",
        "core::any",
        "core::borrow",
        "core::marker",
        "core::panic",
        "core::clone",
        "core::default",
        "core::hash::Hash",
        "core::convert::AsRef",
        "core::convert::AsMut",
        "core::cmp",
    ];
    let should_exclude = |item: &&Item| false;
    let count_current = |item: &&Item| item.is_async;
    count_items(items, exclude_paths, should_exclude, count_current)
}

fn count_items(
    items: &[Item],
    exclude_paths: &[&str],
    mut should_exclude: impl FnMut(&&Item) -> bool,
    count_current: impl FnMut(&&Item) -> bool,
) -> (usize, usize) {
    let mut excluded = 0;
    let count = items
        .iter()
        .filter(|item| item.stability.is_stable())
        .filter(|item| {
            if should_exclude_path(dbg!(&item.path), exclude_paths) {
                excluded += 1;
                false
            } else if should_exclude_path(&item.target_trait, exclude_paths) {
                excluded += 1;
                false
            } else if should_exclude(item) {
                excluded += 1;
                false
            } else {
                true
            }
        })
        .filter(count_current)
        .count();
    (count, excluded)
}

fn should_exclude_path(target: &str, exclude_paths: &[&str]) -> bool {
    // println!("\n\n");
    let out = exclude_paths.iter().fold(false, |should_exclude, path| {
        // println!("{}\t starts with \t {}", target, path);
        if should_exclude {
            true
        } else {
            match target.starts_with(path) {
                true => true,
                false => false,
            }
        }
    });
    // println!("{out}");
    out
}
