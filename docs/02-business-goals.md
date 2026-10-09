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
