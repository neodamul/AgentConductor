# Local company build version

Cargo stays at a numeric `x.y.z` version so release tags and update comparison remain valid. The empty tracked root file `company-suffix.txt` is a local build input: a developer may write a short ASCII label such as `company_name`, rebuild, and see `<Cargo version>.company_name` in the Workspace and Agent Room titlebars.

`update::current()` validates the label before displaying it; invalid or overlong contents fall back to the numeric package version. MCP client protocol metadata continues to advertise the numeric Cargo version.

The Rust release packager rejects both a non-empty source suffix file and a binary compiled with a suffix. The second check matters if the file was cleared after the local binary was built. Keep the checked-in file empty for public CI and release artifacts.

Verification: unit tests cover valid/invalid labels and both release rejection cases. A temporary local build with `company_name` passed the titlebar and numeric update-comparison tests; `company-suffix.txt` was restored empty afterward.
