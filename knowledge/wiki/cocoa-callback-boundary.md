# Cocoa callback boundary

Source: #472, R-35 / DD-41. The Cocoa facade keeps the `open` entry point, bridge ownership, Objective-C helpers, native menu and close modules. `cocoa/callbacks.rs` holds WebKit message, reply, navigation and delegate callbacks.

The move preserves callback ABIs and bodies, trusted main-frame and `about:blank` checks, the eight-request limit, reply block ownership and delivery to the main queue. Parent-private visibility is sufficient; no runtime dependency is added.

The Cocoa layout test fails against the original 494-line file and passes with the 292-line facade and 206-line callback module. Nine focused local tests and all-target Clippy pass. The full local all-target suite on `3df36abb` exits 0 with 844 passed / 0 failed / 15 ignored across 272 suites. Live provider, actual window interaction and Windows checks need separate evidence.
