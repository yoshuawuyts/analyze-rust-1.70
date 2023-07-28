use rustdoc_denormalize::Crate;

const EXCLUDE_PATHS: &[&str] = &["std::os", "std::fs", "std::net", "std::process"];

pub fn count_const_functions(krate: &Crate) -> usize {
    krate
        .functions
        .iter()
        .filter(|fn_| fn_.stability.is_stable())
        .filter(|fn_| fn_.decl.contains("const"))
        .filter(|fn_| !should_exclude(&fn_.path))
        .count()
}

fn should_exclude(target: &str) -> bool {
    EXCLUDE_PATHS.iter().fold(false, |state, pat| {
        if state {
            return true;
        } else {
            // dbg!(&target, &pat);
            match target.contains(pat) {
                true => true,
                false => false,
            }
        }
    })
}
