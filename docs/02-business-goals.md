# Business goals

| ID | Goal | Requirements | Success measure |
|---|---|---|---|
| BG-01 | Coordinate work using existing CLI subscriptions | R-01–03 | Zero additional LLM API key registrations |
| BG-02 | Reduce waiting across implementation, tests, documentation | R-04–06 | Three execution intervals overlap; compare latency to sequential execution |
| BG-03 | Preserve human control and trustworthy changes | R-07–09 | Zero unapproved protected-branch changes or completions without evidence |
| BG-04 | Reuse internal knowledge and lessons | R-10–11 | Decisions, lessons, sources retrievable on resume; unverified claims marked |
| BG-05 | Minimize distribution and maintenance cost | R-12–15 | One core executable; zero bundled components with unreviewed licenses |

Measure resource use and latency on the actual host and CLIs. Subscription usage and
estimated token costs are different; missing cost data must not appear as zero.

## Continuity goals

- BG-07 (R-18): Continue work across subscription reset windows without busy retry loops or lost evidence.
- BG-08 (R-19): Preserve delivery continuity by transferring a role to an available agent while keeping ownership explicit.
- BG-09 (R-20): Give people control over session models without inventing account capabilities or changing authentication.

### BG-09 — Start and review work in one local application

An operator can configure CLI teammates and start a goal without manually editing files or opening another application. Existing authentication remains authoritative. Trace: R-21.

### BG-10 Capacity visibility and portable delivery

Operators can plan paid-agent work using actual remaining subscription windows, and developers can continue the same product on macOS or Windows without losing reviewed decisions or requiring shared credentials. Verified binaries are distributed through the private repository.
