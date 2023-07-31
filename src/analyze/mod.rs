use rustdoc_denormalize::Item;

pub fn count_const_items(items: &[Item]) -> (usize, usize) {
    let exclude_paths = &["std::os", "std::fs", "std::net", "std::process"];
    let should_exclude = |_item: &&Item| false;
    let count_current = |item: &&Item| item.is_const;
    count_items(items, exclude_paths, should_exclude, count_current)
}

pub fn count_async_items(items: &[Item]) -> (usize, usize) {
    let should_exclude = |_item: &&Item| false;
    let exclude_paths = &["std::ops", "std::thread"];
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
            if should_exclude_path(&item.path, exclude_paths) {
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
    exclude_paths.iter().fold(false, |should_exclude, pat| {
        if should_exclude {
            true
        } else {
            match target.contains(pat) {
                true => true,
                false => false,
            }
        }
    })
}
