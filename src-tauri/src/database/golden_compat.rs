//! Adapter for the frozen tests, whose large fixture includes the historical
//! truncation. The uncapped path has its own independent oracle in db-bench.

use super::types;

mod engine {
    use std::ops::Deref;

    use crate::database::{engine, types::{Database, View}};
    use crate::model::Item;

    pub(super) struct Ctx<'a>(engine::Ctx<'a>);

    impl<'a> Ctx<'a> {
        pub(super) fn new(items: impl Iterator<Item = &'a Item>, now: i64, tz: i64) -> Self {
            Self(engine::Ctx::new(items, now, tz))
        }

        pub(super) fn run_view(&self, db: &Database, view: &View) -> engine::ViewResult {
            self.0.run_view_with_old_limit(db, view)
        }
    }

    impl<'a> Deref for Ctx<'a> {
        type Target = engine::Ctx<'a>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[path = "golden/mod.rs"]
mod golden;
