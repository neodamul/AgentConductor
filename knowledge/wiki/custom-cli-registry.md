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

The registry is read once when a workspace controller opens from
`~/.agentconductor/cli-profiles.json` (`%USERPROFILE%/.agentconductor` on
Windows); snapshots reuse the cached entries rather than polling the file. The
loader denies unknown fields, duplicate IDs, built-in IDs, per-profile
environment values, shell wrappers, symlinks, and non-regular files. It opens
without following symlinks on supported platforms, checks the size before
allocation, and rejects a file that changes while being read. It reuses the
existing profile validator so permission, event mapping, session, and fallback
rules stay consistent. Registered profiles appear by ID in the native agent
profile selector; invalid registry files leave built-in profiles available and
show a localized warning.

This registry is data-only: it never executes a script or calls an
OpenAI-compatible endpoint. CLI credentials continue to come from the CLI's
existing login/configuration. The application currently reads but does not edit
the registry; declarative custom quota definitions are not yet available.
