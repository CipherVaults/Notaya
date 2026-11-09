// The real engine files, included by path and not copied.
#[path = "../../../src-tauri/src/database/types.rs"]
pub mod types;
#[path = "../../../src-tauri/src/database/value.rs"]
pub mod value;
#[path = "../../../src-tauri/src/database/regex.rs"]
pub mod regex;
#[path = "../../../src-tauri/src/database/formula.rs"]
pub mod formula;
// The query benchmark supplies a fixed clock to Ctx; the OS clock module is unused.
#[path = "../../../src-tauri/src/database/engine.rs"]
pub mod engine;

#[cfg(test)]
#[path = "../../../src-tauri/src/database/golden/mod.rs"]
mod golden;
