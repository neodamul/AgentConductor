# Windows packaged chat test isolation

## Failure

Run `41356140816` first showed the three coordinator-chat journeys failing.
The follow-up exact-head run `41360959648` repeated all three failures
sequentially, which disproved concurrency as their root cause. The packaged
app's stderr said Windows denied process breakaway inside the GitHub Actions
host Job Object. The product then correctly refused to start background work
because survival after app closure was not guaranteed.

## Cause and correction

The coordinator-chat journeys control one interactive Windows desktop, so the
package test invocation keeps `--test-threads=1` to prevent competing UI
Automation instances. This is necessary desktop-test isolation, not the fix for
run `41360959648`.

The updated test harness launches the exact packaged executable through an
interactive Task Scheduler task, outside the Cargo test process tree, while
retaining serial UI Automation. See [interactive package launcher](windows-packaged-chat-launch-context.md).

## Verification boundary

The workflow-contract test checks for the interactive scheduled-task launcher,
and the local focused suite passes. Windows compilation and all three exact
package journeys must pass on a pushed head before packaged chat acceptance is
counted. Do not weaken the product's host-job safety check to make CI green.
