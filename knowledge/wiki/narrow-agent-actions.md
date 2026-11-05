# Narrow agent action strip

Source: #424, R-210 / UC-202 / QAS-209 / DD-222. At 840px the natural 167px action strip wrapped from 22px to 48px in both app and approved mock. Reducing non-delete padding to 4px was sufficient for Korean but not for English: `Models` made the row approximately 177px wide.

At viewport widths up to 900px both app and reference use 2px gaps, 2px horizontal padding on edit/model, 4px on reorder and unchanged delete spacing. The labels stay visible and the reorder click width stays unchanged. Wider viewports keep their previous spacing.

Actual Chrome checks cover Korean light/dark at 840 and 1280, English light/dark at 840, icons, focus, accessible names, pointer cursors, overflow and app/reference dimensions. Full-suite and Windows receipts remain separate.
