# Sidebar feature pass

## Brief

Left sidebar = navigation + chat history. Right sidebar = context for the current route.
No fake affordances: every control must do something real.

## Tasks

| # | Task | Files | Verify | Done when |
|---|------|-------|--------|-----------|
| 1 | Delete endpoint for chats | `src/routes/api/chats/[id]/+server.ts` | `bun run check` | DELETE returns 204, 404 on unknown id |
| 2 | Route context + counts + date buckets in layout load | `src/lib/server/context.ts`, `src/routes/+layout.server.ts` | `bun run check` | `data.context` and `data.counts` present on every route |
| 3 | Neutral accent token (active/hover no longer a blue brick) | `src/routes/layout.css` | visual | Select hover + active row are neutral |
| 4 | Rewrite left sidebar | `src/lib/components/layout/SidebarLeft.svelte` | `bun run check`, screenshot | Labels, search, groups, delete, theme, drawer, aria |
| 5 | Rewrite right sidebar as context panel | `src/lib/components/layout/SidebarRight.svelte` | `bun run check`, screenshot | No dead buttons; shows real route context |
| 6 | Shell: ModeWatcher, mobile bar, title/meta, lang | `src/routes/+layout.svelte`, `src/app.html` | Lighthouse | Dark mode reachable; a11y score >= 0.95 |
| 7 | Live verify at 1440 / 768 / 360, light + dark | browser | screenshots | No horizontal overflow at 360 |

## Status

All tasks implemented. `bun run check`: 0 errors, 0 warnings.
Verified live: theme toggle sets `html.dark` + persists; sidebar collapse → 56px;
right panel opens to 288px with real agent context; no console errors.

## Incident (unresolved)

During verification the live DB lost all chats/messages (1 agent kept).
Base `data/app.db` snapshot (10:30) still holds 7 chats / 15 messages; the
deletions live only in `app.db-wal` (10:37). No DELETE request came from the
browser tab. Cause not proven. Recovery = restore base + drop WAL (loses the
newest chat). Awaiting user decision; backup taken first.

## Pass 2 — remaining weak spots

| # | Task | Files | Verify | Done when |
|---|------|-------|--------|-----------|
| 8 | `/chat` list: edit-in-place, confirm before delete, actions on hover only | `src/routes/chat/+page.svelte` | `bun run check`, DOM | title appears once; 1 input max |
| 9 | Chat header responsive (no overflow at 360) | `chat/[id]/+page.svelte`, `chat/new/+page.svelte` | DOM/class | badges + breadcrumb collapse |
| 10 | Composer + empty state polish | `ChatView.svelte` | check | greeting empty state, larger send target |
| 11 | Thai reasoning labels | `reasoning-trigger.svelte` | grep | no English "Thinking" |
| 12 | Control sizing h-7 → h-8 | `button/button.svelte`, `input/input.svelte` | check | 32px targets |

## Pass 2 status

Done: `/chat` edit-in-place + confirm, responsive chat headers, composer/empty state,
Thai reasoning labels, 32px controls, friendly model labels.
Verified on a temp DB copy: 7 rows, 0 inline inputs, rename POST 200, send 36px,
no overflow, no console errors. `bun run check`: 0/0.

## Pass 3 status

Done: assistant copy + regenerate, model picker moved into the composer with PATCH
persistence, friendly labels.
Verified on a throwaway DB copy:
- copy handler wrote `ok`, label flipped to "คัดลอกแล้ว"
- `PATCH /api/chats/<id>` 200, invalid body 400 (server log confirms)
- reload shows `deepseek · chat` → persisted
- chip inside the composer form, not the header; no overflow; no console errors
`bun run check`: 0 errors, 0 warnings.

## Pass 4 status — context panel below 1280px

State lifted to `+layout.svelte` (`contextOpen` + localStorage). Panel is a fixed right
sheet with backdrop below `xl`, static at `xl`. Triggers: sidebar footer button
(`< xl`), mobile top bar (`< md`), rail toggle at `xl`. Closes on backdrop, close
button, Escape, and on internal link navigation below xl. Closed sheet is
`visibility:hidden` so it is not tabbable.

Verified in the real hydrated tab (1056px) and in same-origin iframes (real viewports):

| Width | Closed | Open | Backdrop | Overflow |
|---|---|---|---|---|
| 360 | fixed, hidden, off-screen (right 409) | fixed 289, right 360 | block | none |
| 1100 | fixed, hidden, off-screen (right 1149) | fixed 289, right 1100 | block | none |
| 1280 | static, visible, 49px rail | static 289, main 735 | display:none | none |

Real tab: footer click → open (289, right 1056, backdrop), Escape → closed.
360 iframe: top-bar click → open (289, right 360), backdrop click → closed.

## Pass 5 — visual design pass

- Chat empty state: 30px greeting centred at ~38% height, 4 suggestion chips that
  fill the composer and focus it.
- Composer: single card (1px border, 14px radius, white on white), borderless
  textarea, model chip as plain muted text, circular 36px send button.
- Sidebar "New" row: quiet secondary fill instead of a primary blue block.
- Home: 30px heading + primary "แชทใหม่" action.
- Model chip forced transparent in dark mode too (`dark:bg-transparent`).

Measured in light mode at 1056px: heading 38% of scroll area, card
`oklch(1 0 0)` + `oklch(0.922 0 0)` border, textarea border 0, send 36px
rounded-full, chip transparent in light and dark, no horizontal overflow.
`bun run check`: 0/0.

## Pass 6 — visual verification with real screenshots

Captured with Chrome over CDP (`Emulation.setDeviceMetricsOverride`), so the layout
viewport is exactly 360 / 768 / 1440 CSS px. Every size reported
`scrollWidth == clientWidth` (no horizontal overflow).

Changes driven by what the screenshots showed:
- Empty chat now matches the reference: greeting + subtitle + composer **together**
  in the middle, icon suggestion chips **below** it, composer is a rounded pill.
- Composer extracted to a snippet; rendered centred when empty, pinned bottom
  when a chat has messages.
- Home: stat cards become a compact 3-up row on phones (was three full-width
  stacked cards pushing recents off-screen); recent row now truncates the title
  and keeps the timestamp on one line.
- Verified: suggestion chip fills the composer, focuses it, and enables send.

## Data safety

- Base snapshot saved to `data/backup-base-2230/app.db` (7 chats / 15 messages).
- `data/app.db` itself untouched (mtime 22:30); the deletes live only in the WAL.
- Do NOT restore unless the user asks — they chose to keep the empty state.

## Out of scope (reported, not fixed)

- `/chat` list page inline rename inputs (noisy) — separate pass.
- Composer model chip outside the composer.
- Hardcoded English "Thinking..." in reasoning trigger.
