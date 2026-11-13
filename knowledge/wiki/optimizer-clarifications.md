# Goal optimizer clarification review

Issues #935/#936 extend R-177 / BG-132 / SF-165 / UC-168 / FR-163 / QAS-174 / DD-188.

The existing read-only optimizer job accepts optional answers, bounded to 8 KiB before launch and again in the worker prompt builder. The dialog sends its current proposal plus the original confirmation list and operator answers. Clarifications belong to the Document section before Task and Format. They do not grant tool permissions or trigger plan execution.

One answer field covers the whole confirmation list. It disappears when the new result has no questions. While a request is running, duplicate requests and applying the old result are disabled. Errors preserve the visible proposal and answer text. Only Use this goal applies a result; saved workflow execution remains separate.

Result lists explicitly use 13px/1.6 secondary text instead of inheriting the 16px root font. Reference typography and input checks cover both themes and narrow/wide viewports. Tests cover browser behavior, request validation, read-only subprocess input/output and original settings preservation. No new dependency or polling worker is introduced.
