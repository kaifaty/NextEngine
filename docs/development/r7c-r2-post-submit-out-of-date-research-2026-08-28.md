# R7c R2 post-submit out-of-date research — 2026-08-28

## Problem and stopping rule

Two coherent active-kernel collection attempts stopped with
`R2_ALPHA_RENDER_WORKLOAD_INVALID: desktop-finish`. Attempt 1 stopped in its
first report; attempt 2 published two complete clean reports and stopped in
report 3. Neither stop published a performance verdict for the failing member.
The repeated symptom triggered the repository two-cycle research escalation;
blind recollection stopped.

## Competing hypotheses

| Hypothesis | Evidence for | Evidence against | Conclusion |
| --- | --- | --- | --- |
| External CPU/GPU pressure invalidated the workload | Earlier unrelated tasks had occupied the host | Both failing reports entered through ready preflight; complete neighbouring reports were clean | Rejected as the direct cause |
| Repeated SDL/libdecor initialization broke later windows | GTK/libdecor warnings begin after the first SDL lifetime | The same warnings occur on complete six-window PASS controls | Rejected as the counter divergence cause; warning remains non-authoritative noise |
| Wayland configure/fullscreen produced a post-submit swapchain transition | Failures occur after one or more valid windows; SDL/Wayland window state is asynchronous | Needed exact internal counters | Supported after instrumentation |
| The bounded loop submitted one replacement frame after a post-submit out-of-date result | Generic validation could hide `rendered_frames` versus GPU/cache/UI divergence | Needed a failing instrumented run | Confirmed |

## Exact local evidence

Temporary stderr-only instrumentation was built from dirty `9f239928`; it is
diagnostic, not release evidence. The reproduced failure was the second
`primary-1080p` window (`combat`):

- public `rendered_frames=4200`, expected `4200`;
- `frame_timings=4200`, but Vulkan timestamps `8402` and `dropped=1`;
- frame-plan `cache_hits=4200`, `cache_misses=1`;
- `ui_frames=4201`.

`GraphicsContext::render` submits the command buffer, records profiling and UI
counters, then calls `queue_present`. When presentation returns
`VK_ERROR_OUT_OF_DATE_KHR`, `defer_out_of_date` yields `None`. The old branch
recreated the swapchain and returned `Ok(None)`, so the caller did not increment
`rendered_frames` and submitted a replacement iteration. The already-complete
submission could not be rolled back, producing exactly the observed extra
timestamp/cache/UI frame and one dropped timing sample.

## External primary evidence

- SDL documents video/event initialization as main-thread and reference-counted:
  <https://wiki.libsdl.org/SDL3/SDL_Init>.
- SDL's Wayland guidance states that windows/configures are asynchronous and
  require a continuously serviced event loop:
  <https://github.com/libsdl-org/SDL/blob/main/docs/README-wayland.md>.
- SDL exposes libdecor control before initialization, but disabling it is not a
  fix for engine-owned post-submit accounting:
  <https://wiki.libsdl.org/SDL3/SDL_HINT_VIDEO_WAYLAND_ALLOW_LIBDECOR>.
- SDL's current Wayland backend sends fullscreen/window state from compositor
  events, consistent with a legitimate late out-of-date presentation result:
  <https://github.com/libsdl-org/SDL/blob/main/src/video/wayland/SDL_waylandwindow.c>.

These sources support asynchronous presentation as a valid boundary event;
the conclusion that Next Engine double-counted its replacement is an inference
from the exact local counters and source order.

## Decision and falsifier

After a command buffer and profiling queries have been submitted, an
out-of-date `queue_present` result recreates the swapchain for the next frame
but returns the already-submitted frame to the bounded loop. Acquisition-time
out-of-date still returns `None` because no submission occurred. No budget,
scenario, fingerprint or preflight rule changes.

The first post-fix six-window control completed with, for every window,
`rendered_frames=4200`, `timings=4200`, `timestamps=8400`, `dropped=0`, cache
`4199/1` and exact UI-frame count. Reconsider the fix if a post-fix run shows
counter divergence, a recoverable presentation transition without a completed
submission, or a focused platform regression. The smallest next action is to
remove diagnostic prints, run focused desktop/performance checks, commit the
one-line semantic fix plus this evidence record, rebuild from the clean commit
and start a wholly fresh R2–R5 campaign. Attempts 1 and 2 remain immutable
negative/incomplete evidence.
