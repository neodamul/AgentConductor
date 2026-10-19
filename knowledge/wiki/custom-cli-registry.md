# Custom CLI profile registry

The Rust migration branch accepts a versioned, local JSON registry through
`custom_profile_registry::CustomProfileRegistry`. Version 1 contains at most 16
profiles and the file is limited to 64 KiB. Entries must use the `custom`
protocol; built-in provider profiles remain owned by their Rust adapters.

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "aipro",
      "executable": "aipro",
      "args": ["run", "--jsonl"],
      "parser": "custom",
      "input_mode": "stdin",
      "permission_mode": "unverified",
      "max_sessions": 3,
      "quota_refresh_minutes": 5
    }
  ]
}
```

The loader denies unknown fields, duplicate IDs, shell wrappers, symlinks, and
non-regular files. It opens without following symlinks on supported platforms,
checks the size before allocation, and rejects a file that changes while being
read. It reuses the existing profile validator so permission, event mapping,
session, and fallback rules stay consistent.

This is the first loader slice only. The native profile editor does not yet read
or update the registry, and declarative custom quota definitions are not yet
available. The registry is data-only: it never executes a script or calls an
OpenAI-compatible endpoint.
