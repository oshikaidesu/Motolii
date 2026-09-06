Source: https://github.com/DioxusLabs/blitz/tree/64eb27853aa2672486b7edf825fb044be78c9db3/packages/blitz-dom

License: MIT OR Apache-2.0 (included).

Only Rust change: clear the old stacking context when promoting paint children to a parent. Exact diff: ../../reference/blitz-stale-stacking-context.patch.

Cargo.toml expands upstream workspace inheritance. In-repo dependencies retain the same Git revision; all other dependency versions/features are inherited unchanged. [workspace] prevents accidental membership in the product workspace.
