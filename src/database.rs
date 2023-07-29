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
        fn find_trait(
            db: &Database,
            id: &rustdoc_types::Id,
        ) -> Option<(rustdoc_types::Item, rustdoc_types::Trait)> {
            db.find_item(id).and_then(|item| match item.clone().inner {
                ItemEnum::Trait(ty) => Some((item, ty)),
                ItemEnum::Import(import) => find_trait(db, &import.id?),
                _ => None,
            })
        }
        ids.into_iter()
            .filter_map(|id| find_trait(self, id))
            .collect()
    }

    pub(crate) fn find_functions(
        &self,
        ids: &[rustdoc_types::Id],
    ) -> Vec<(rustdoc_types::Item, rustdoc_types::Function)> {
        fn find_function(
            db: &Database,
            id: &rustdoc_types::Id,
        ) -> Option<(rustdoc_types::Item, rustdoc_types::Function)> {
            db.find_item(id).and_then(|item| match item.clone().inner {
                ItemEnum::Function(ty) => Some((item, ty)),
                ItemEnum::Import(import) => find_function(db, &import.id?),
                _ => None,
            })
        }
        ids.into_iter()
            .filter_map(|id| find_function(self, id))
            .collect()
    }

    pub(crate) fn find_structs(
        &self,
        ids: &[rustdoc_types::Id],
    ) -> Vec<(rustdoc_types::Item, rustdoc_types::Struct)> {
        fn find_struct(
            db: &Database,
            id: &rustdoc_types::Id,
        ) -> Option<(rustdoc_types::Item, rustdoc_types::Struct)> {
            db.find_item(id).and_then(|item| match item.clone().inner {
                ItemEnum::Struct(strukt) => Some((item, strukt)),
                ItemEnum::Import(import) => find_struct(db, &import.id?),
                _ => None,
            })
        }
        ids.into_iter()
            .filter_map(|id| find_struct(self, id))
            .collect()
    }

    pub(crate) fn find_enums(
        &self,
        ids: &[rustdoc_types::Id],
    ) -> Vec<(rustdoc_types::Item, rustdoc_types::Enum)> {
        fn find_enum(
            db: &Database,
            id: &rustdoc_types::Id,
        ) -> Option<(rustdoc_types::Item, rustdoc_types::Enum)> {
            db.find_item(id).and_then(|item| match item.clone().inner {
                ItemEnum::Enum(enum_) => Some((item, enum_)),
                ItemEnum::Import(import) => find_enum(db, &import.id?),
                _ => None,
            })
        }
        ids.into_iter()
            .filter_map(|id| find_enum(self, id))
            .collect()
    }

    pub(crate) fn find_impls(
        &self,
        ids: &[rustdoc_types::Id],
    ) -> Vec<(rustdoc_types::Item, rustdoc_types::Impl)> {
        fn find_impl(
            db: &Database,
            id: &rustdoc_types::Id,
        ) -> Option<(rustdoc_types::Item, rustdoc_types::Impl)> {
            db.find_item(id).and_then(|item| match item.clone().inner {
                ItemEnum::Impl(impl_) => Some((item, impl_)),
                ItemEnum::Import(import) => find_impl(db, &import.id?),
                _ => None,
            })
        }
        ids.into_iter()
            .filter_map(|id| find_impl(self, id))
            .collect()
    }
}
