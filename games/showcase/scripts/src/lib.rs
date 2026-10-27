//! Scripts are registered automatically: put one script per file anywhere under `src/`
//! (for example `src/movement/spinner.rs`) and it appears in the editor. No lists to maintain.
//! The type needs `pub struct Name` + `impl Script for Name`; see docs/SCRIPTING.md.

include!(concat!(env!("OUT_DIR"), "/kvas_scripts.rs"));
