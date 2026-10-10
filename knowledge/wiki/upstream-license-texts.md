# Omitted crate license texts

Trace: R-24 → BG-10 → SF-20 → UC-17 → QAS-18 → DDR-03/DDR-08 → DD-27.

Native Windows CI 38040929721 built the release executable but refused packaging
because webview2-com 0.39.1 omitted a license file. Inspection of the actual locked
Windows dependency closure also found omissions in webview2-com-sys 0.39.1 and
webview2-com-macros 0.8.1. SPDX declaration inventory alone had not established
that the distributable contained every required original notice.

Official upstream MIT text (Bill Avery, 2021) was retrieved on 2026-10-10 from
the exact commits recorded in each published crate's .cargo_vcs_info.json:
- [webview2-com and sys source](https://github.com/wravery/webview2-rs/blob/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE)
- [macros source](https://github.com/wravery/webview2-rs/blob/dffa41a8a46d3f5565eefbff2de57d38d399f158/LICENSE)

Retained copies and sources.json record package/version/SPDX, VCS commit, URL,
retrieval date and SHA-256. Packaging uses them offline only when the crate omits
its own text; unknown versions, mismatched VCS commits, missing copies and altered
bytes refuse packaging. The original license and provenance are copied into each
affected dependency's package notice directory. The MIT notice preservation
requirement is retained; no new public root-project license is granted.

TDD Red reproduced the missing-text error; Green and refusal cases passed. Seven
Python packaging/smoke tests passed, including altered bytes and version/revision
mismatches before output. Actual offline package review resolved 42 macOS and 108
Windows dependencies, with three reviewed upstream copies on Windows. Native
Windows package execution remains pending after this fix. No new Rust dependency
or GUI process was introduced. Revalidate on any dependency upgrade or changed
license/provenance digest.

Sources:
- [scripts/release_package.py](../../scripts/release_package.py): SHA-256 `11212db2a9faca007d2ab2abc5a45b43b38eb7ce7c91127ad798a4c39a27a321`.
- [tests/test_release_package.py](../../tests/test_release_package.py): SHA-256 `956ed98ac8f2b4542c85bfc32f614ee8ef009c147f7263c2ed87cde8124ad8bf`.
- [licenses/upstream/sources.json](../../licenses/upstream/sources.json): SHA-256 `682c6f1679f354d0e99faa029b893969cfd75d124219d49165f0bde432653978`.
