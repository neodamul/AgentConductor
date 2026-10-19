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
      "quota_refresh_minutes": 5,
      "quota_rules": [
        {
          "event_type": "usage",
          "bucket_pointer": "/payload/account",
          "label_pointer": "/payload/period",
          "used_percent_pointer": "/payload/used_percent",
          "resets_at_pointer": "/payload/resets_at",
          "window_minutes_pointer": "/payload/window_minutes"
        }
      ]
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
existing profile validator so permission, event mapping, session, quota, and
fallback rules stay consistent. A quota rule maps one custom `usage` JSONL event
with JSON Pointers into the shared quota-window model. Pointers and event types
are bounded, event types are unique, and reported usage must be 0–100 percent.
These values come only from the agent's owned stdout stream. Registered profiles appear by ID in the native agent
profile selector; invalid registry files leave built-in profiles available and
show a localized warning.

This registry is data-only: it never executes a script or calls an
OpenAI-compatible endpoint. CLI credentials continue to come from the CLI's
existing login/configuration. The application currently reads but does not edit
the registry or issue a custom quota-query command. Custom quota snapshots are
available only when a matching `usage` event is observed during an agent session;
an on-demand refresh control remains unimplemented.
