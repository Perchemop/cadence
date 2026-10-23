# Research — printing the Gantt view

Status: draft, not human-approved.

## Behaviour wanted

The Gantt view already draws bars, dependency arrows and a timeline to the
screen. Printing wants the same drawing as a page in a PDF, so a user can
attach the current plan to an email or print it for a site office.

## Acceptance criteria

1. A toolbar action captures the Gantt drawing (bars, arrows, axis, labels)
   as it is rendered on screen, at the current zoom.
2. The capture becomes a single landscape A4 page in a PDF via the existing
   `cadence-core/src/pdf.rs` image-page writer.
3. The destination is chosen with the same native save dialog as the report
   print.
4. The activity table is not required on the page — the report PDF already
   carries the tabular data.

## Implementation direction (original notes)

- egui exposes a screenshot request/consume pair on the context:
  request a capture, then read the produced `ColorImage` on a following frame
  (the capture is delivered after the next paint). API entry point:
  https://docs.rs/egui/latest/egui/struct.Context.html#method.request_screenshot
- `ColorImage` stores straight-alpha RGBA per pixel; the PDF writer's image
  pages want opaque RGB, so alpha is composited over the panel background
  colour before hand-off.
- The existing `Pdf::image_page` already accepts raw RGB and fits the page
  landscape; no PDF-side work is expected.
- Risk to check while implementing: virtualised rows mean only visible rows
  are painted; a capture is a capture of the viewport, not the whole plan.
  Either document that or scroll-and-stitch (later work).

## Sources

- egui `Context::request_screenshot`: https://docs.rs/egui/latest/egui/struct.Context.html#method.request_screenshot
- egui `ColorImage`: https://docs.rs/egui/latest/egui/struct.ColorImage.html
