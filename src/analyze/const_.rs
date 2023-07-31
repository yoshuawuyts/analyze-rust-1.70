use rustdoc_denormalize::Item;

const EXCLUDE_PATHS: &[&str] = &["std::os", "std::fs", "std::net", "std::process"];

pub fn count_const_items(items: &[Item]) -> (usize, usize) {
    let mut excluded = 0;
    let count = items
        .iter()
        .filter(|item| item.stability.is_stable())
        .filter(|item| {
            if should_exclude(&item.path) {
                excluded += 1;
                false
            } else {
                true
            }
        })
        .filter(|item| item.is_const)
        .count();
    (count, excluded)
}

fn should_exclude(target: &str) -> bool {
    EXCLUDE_PATHS.iter().fold(false, |should_exclude, pat| {
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
