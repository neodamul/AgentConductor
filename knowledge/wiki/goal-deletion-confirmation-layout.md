# Goal deletion confirmation layout

The permanent-delete dialog keeps the Goal name and exact folder path together in a bounded, keyboard-scrollable identity region. Long identity text can be read without pushing the irreversible warning or actions out of view.

The destructive action must never receive default focus; Cancel remains the initially focused action. Deletion is still performed only by the explicit confirmation action.

`tests/delete_goal_dialog_752.cjs` checks long Korean and English identities in both themes at 840 px and 1280 px, verifies the warning and actions remain visible at the minimum dialog height, and confirms Cancel closes the dialog without confirming deletion.
