# Planning dialog close ordering

Source: #484, R-210 / UC-202 / QAS-209 / DD-222.

## Failure mechanism

HTML dialog closure updates the open state before the queued close event is delivered. Reopening a reused dialog replaces its onclose callback. The previous cancellation can then resolve the new confirmation with a false decision, silently skipping save/baseline/read_plan even though the new dialog later closes with a confirm return value. Waiting for !dialog.open is not proof that the previous decision settled.

## Resolution

The shared planning confirmation records each native form submission or Escape cancellation in its own invocation. A delayed close observed while the dialog is open cannot settle a new review. Native dialog behavior, coordinator/model evidence and revision/context guards remain unchanged.

## Evidence

The VM test forces the order and fails before the fix. The assembled Chrome test withholds the first native close event until the second dialog opens, reproducing the exact CI diagnostic (enabled controls, confirm return value, no new save/baseline/read_plan). After the fix, both tests and the four light/dark × 840/1280 browser cases pass. Ten consecutive runs with two owned CPU-load workers pass (40 cases); workers terminate in the runner finally block. A full-suite receipt remains required before completion.
