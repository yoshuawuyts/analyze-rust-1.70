use rustdoc_types::ItemEnum;

/// Internal rustdoc database structure with various query methods on it.
pub(crate) struct Database {
    inner: rustdoc_types::Crate,
}

impl Database {
    /// Create a new instance of database
    pub(crate) fn new(inner: rustdoc_types::Crate) -> Self {
        Self { inner }
    }

    /// Find a rustdoc `Item` by id
    pub(crate) fn find_item(&self, id: &rustdoc_types::Id) -> Option<rustdoc_types::Item> {
        let item = self.inner.index.get(id)?;
        Some(item.clone())
    }

    /// Find a rustdoc path by id.
    pub(crate) fn find_path(&self, id: &rustdoc_types::Id) -> Option<String> {
        let summary = self.inner.paths.get(id)?;
        Some(summary.path.join("::"))
    }

    /// Get a list of all modules
    pub(crate) fn modules(&self) -> Vec<(String, rustdoc_types::Module)> {
        let mut out: Vec<_> = self
            .inner
            .index
            .iter()
            .filter_map(|(id, item)| match &item.inner {
                ItemEnum::Module(module) => {
                    if module.is_stripped {
                        return None;
                    }
                    let path = self.find_path(id)?;
                    Some((path, module.clone()))
                }
                _ => None,
            })
            .collect();
        out.sort_by(|(left, _), (right, _)| left.cmp(right));
        out
    }

    /// Given a list of IDs, find all traits. A rustdoc module only
    /// provides a `Vec<Id>` for all items in it, so we have to do a filter-find
    /// to narrow it down to just traits, etc.
    pub(crate) fn find_traits(
        &self,
        ids: &[rustdoc_types::Id],
    ) -> Vec<(rustdoc_types::Item, rustdoc_types::Trait)> {
        ids.into_iter()
            .filter_map(|id| {
                self.find_item(id)
                    .and_then(|item| match item.clone().inner {
                        ItemEnum::Trait(adt) => Some((item, adt)),
                        _ => None,
                    })
            })
            .collect()
    }

    pub(crate) fn find_functions(
        &self,
        ids: &[rustdoc_types::Id],
    ) -> Vec<(rustdoc_types::Item, rustdoc_types::Function)> {
        ids.into_iter()
            .filter_map(|id| {
                self.find_item(id)
                    .and_then(|item| match item.clone().inner {
                        ItemEnum::Function(fn_) => Some((item, fn_)),
                        _ => None,
                    })
            })
            .collect()
    }
}
