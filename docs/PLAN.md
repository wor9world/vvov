# VVOV — แผนลงมือ (ฉบับเข้าใจก่อนทำ)

> เขียนไว้ก่อนแตะโค้ดจริง · 2026-11-21 · ประกอบกับ [ARCHITECTURE.md](./ARCHITECTURE.md)
> สถานะ: **Phase 1–4 ผ่านแล้ว** — วิธีใช้จริง/คำสั่ง/ troubleshooting อยู่ใน [USAGE.md](./USAGE.md)
>
> | Phase | สถานะ | หลักฐาน |
> |---|---|---|
> | 1 พิสูจน์ vertical slice | ✅ | ส่งข้อความ → stream → refresh แล้วประวัติอยู่, tool call ถูกบันทึก |
> | 2 build จริง | ✅ | `bun run build` ผ่าน (adapter-bun) |
> | 3 UI | ✅ | ใช้ shadcn + AI elements (message/response/reasoning/loader) |
> | 4 agent/project | ✅ | agent = persona, project = instructions + กลุ่มแชท |
> | 5 ทางเลือก | ยังไม่ทำ | search, attachments, model selector, Rust tool UI |

---

## 1. สรุปสิ่งที่มีอยู่ตอนนี้ (facts จากโค้ด ไม่ใช่การเดา)

| ชั้น | สถานะ |
|---|---|
| Frontend | SvelteKit 3.4 + Svelte 5.59 (runes) + Tailwind 4 + adapter-bun |
| AI | `ai@7` + `@ai-sdk/svelte@5` (`Chat`) + `ai-sdk-ollama@4.6` (Ollama local: qwen3:0.6b) |
| Backend | **ไม่มีแยกแล้ว** — ใช้ SvelteKit server (Bun) เป็น fullstack |
| DB | SQLite ผ่าน `bun:sqlite` ตรง ๆ (ไม่มี ORM) — เขียนแล้ว: schema + repos + migration runner |
| Routes | `/` → `/chat`, `/chat/[id]` แชทจริง, `/agent/[id]`, `/project/[id]` ใช้ chat id แบบ namespaced |
| Rust | `server/` (axum + math engine) — **ยังไม่อยู่ในเส้นทางของ v1** |
| Rust scaffolder | `E:\vvov\main.rs` — tooling ไม่เกี่ยวกับ runtime |

**สิ่งที่เขียนไปแล้ว (draft):** `#lib/server/db/{index,chats,messages}.ts`, `#lib/server/ai/models.ts`,
`/api/chat` (stream + persist), `+layout.server.ts`, `ChatView.svelte`, หน้า chat/agent/project
→ `bun run check` ผ่าน 0 error; **ยังไม่ผ่านการรันจริง** (นี่คือ Phase 1)

---

## 2. สิ่งที่อ่านจาก package แล้ว — API ที่ควรใช้ vs ที่เลิกใช้

อ่านจาก type definitions + README ที่ติดตั้งจริงใน `node_modules` (ไม่ใช่จากความจำ)

### SvelteKit 3.4

| เรื่อง | ใช้แบบนี้ | ห้าม/เลิกใช้ |
|---|---|---|
| JSON response | `Response.json(data, { status })` | `json()` จาก `@sveltejs/kit` (deprecated) |
| Text response | `new Response(text)` | `text()` (deprecated) |
| URL/route ปัจจุบัน | `import { page } from '$app/state'` | `$app/stores` |
| Page props | `PageProps = { params, data, form }` | อย่าคาดว่า `url` อยู่ใน props |
| Layout props | `{ params, data, children }` | — |
| invalidate | `refreshAll()` | `invalidateAll()` (deprecated) |
| error | `error(status, message, props?)` | ส่ง body เป็น arg 2 (deprecated) |
| redirect | `redirect(303\|307, url)` | — |
| form action fail | `fail(status, data)` | — |
| remote functions | `query` / `command` / `form` จาก `$app/server` (เปิดใช้แล้วใน `vite.config`) | ยังไม่จำเป็นกับ streaming |
| alias | `#lib/*` (package.json imports) + tsconfig paths ที่เพิ่มให้ `#lib`/`$lib` | `$lib` เปล่า ๆ ไม่ถูก generate ให้ |

**Load/route:** `+page.server.ts` load เป็น sync ได้; `+server.ts` ใช้ `RequestHandler` จาก `./$types`

### adapter-bun 1.0.2

- build แล้วรันด้วย `bun ./build` (ไม่ใช่ `node`)
- options: `out` (default `build`), `precompress`, `envPrefix` (`MY_`), `serverOptions` (host/port/idleTimeout…), `buildOptions.compile` → **single executable**
- env: ตั้ง `HOST`/`PORT` ได้ตอนรัน
- ข้อควรระวัง: `data/` และ path ของ migration ต้องสัมพันธ์กับ cwd ตอนรัน

### AI SDK 7 (`ai`)

| API | ใช้ทำอะไร | หมายเหตุ |
|---|---|---|
| `streamText()` | สร้าง stream จากโมเดล | คืน `result.stream` |
| `toUIMessageStream({ stream, originalMessages, generateMessageId, onEnd, onError })` | แปลงเป็น UIMessage stream + hook persistence | **`onEnd` คือจุดเก็บ DB** |
| `createUIMessageStreamResponse({ stream })` | ห่อเป็น Response | `toUIMessageStreamResponse()` แบบเก่า deprecated |
| `validateUIMessages({ messages })` | validate body จาก client | คืน `UIMessage[]` |
| `convertToModelMessages(uiMessages)` | แปลงเป็น model messages (async) | — |
| `onEnd` payload | `{ messages, responseMessage, outcome, isContinuation, isAborted, isCancelled, finishReason }` | `outcome.status`: completed/failed/aborted/unknown |
| `Chat` ฝั่ง client | `new Chat({ id, messages, onFinish, onError })`, `sendMessage`, `regenerate`, `stop`, getters `messages/status/error` | body ที่ส่งมา: `{ id, messages, trigger, messageId }` |
| `DefaultChatTransport` | default `api: '/api/chat'` | เปลี่ยน endpoint/body ได้ผ่าน options |

### ai-sdk-ollama 4.6

- `ollama(modelId, { options: { num_ctx, repeat_penalty, temperature, keep_alive }, think })`
- `createOllama({ baseURL, apiKey, headers })` สำหรับ cloud/remote
- มี wrapper `streamText`/`generateText` ของตัวเองที่เพิ่มความทนทาน tool-call + JSON repair + web search/fetch
- cloud model ต้องมี `OLLAMA_API_KEY`; local ใช้ฟรี

### bun:sqlite

- `new Database(path, { create })`, `db.exec`, `db.query<Row, Params>(sql).get/all/run`, `db.transaction(fn)` (sync)
- migration ด้วย `PRAGMA user_version` — DDL ฝังในโค้ด ไม่ต้อง copy ไฟล์ตอน build
- WAL + `busy_timeout` ตั้งแล้ว; เขียนที่ process เดียว

### Svelte 5.59 / Tailwind 4 / lucide

- runes: `$props`, `$state`, `$derived`, `$effect`; `{#key}` เพื่อรีเซ็ต state ต่อ chat
- Tailwind 4 = CSS-first (`@import 'tailwindcss'`) ไม่มี `tailwind.config.js`
- icon: `import { Bot } from '@lucide/svelte/icons'`, รับ `size`/`strokeWidth`

---

## 3. เข้าใจโจทย์ยังไง (inference — ยืนยันก่อน)

- แอปนี้คือ **AI workspace ส่วนตัวบนเครื่องเดียว**: sidebar ซ้าย = agent / project / chat, ขวา = feed/tasks/tools/library (ยังเป็น icon เปล่า)
- คุณค่าหลัก = คุยกับโมเดล local แล้ว **เก็บประวัติได้จริง**, จัดกลุ่มตาม project/agent
- math engine ของ Rust เป็นของที่แยกมา **ไม่ใช่แกนของ v1** (คุณบอกเองว่า SvelteKit น่าจะจบได้หมด)
- ยังไม่ต้องมี auth, multi-user, cloud — local-first ก่อน แล้วค่อยย้าย

---

## 4. กลยุทธ์ (ทำไมเรียงแบบนี้)

1. **พิสูจน์ vertical slice ก่อนเพิ่มของ** — 1 chat → 1 reply → refresh แล้วยังอยู่ → นี่คือคุณค่าทั้งหมดของงานนี้ ถ้ายังไม่ผ่าน อย่าไปแตะ agent/project
2. **DB เป็น source of truth, client เป็น cache** — เก็บ user message ก่อน stream, assistant ตอน `onEnd`; ไม่เขียนทุก chunk
3. **อย่าเดา semantics ของ agent/project** — ใช้ chat id แบบ `agent:<id>` / `project:<id>` ไปก่อน แล้วค่อยเพิ่มตารางเมื่อคำถาม §9 ข้อ 1–2 มีคำตอบ
4. **ทดสอบจุดที่พังเงียบ** — migration, seq, idempotent save, regenerate, HMR double-open
5. **ทุก phase ต้องมีเกณฑ์ตรวจรับที่รันได้** ไม่ใช่ "รู้สึกว่าเสร็จ"

---

## 5. แผนเป็นระยะ + เกณฑ์ตรวจรับ

### Phase 0 — ฐานราก (เขียนแล้ว, รอตรวจ)
- [x] schema chats/messages/settings + migration runner (`user_version`)
- [x] repos: create/list/get/ensure/touch/delete, save idempotent, seq, title
- [x] `/api/chat` stream + persist (CRUD ใช้ load + form action — ไม่มี `/api/chats`)
- [x] `ChatView` + หน้า chat/agent/project + sidebar list
- [x] `bun run check` = 0 error
- [ ] **ยังไม่ verify runtime**

### Phase 1 — พิสูจน์ vertical slice (งานแรกที่จะทำจริง)
ตรวจรับ:
1. `GET /chat` → redirect ไป `/chat/<id>` และมีแถวใน `chats`
2. พิมพ์ข้อความ → stream ขึ้นทีละ token → จบแล้วมี 2 แถวใน `messages` (user/assistant) พร้อม `parts`
3. refresh `/chat/<id>` → เห็นข้อความเดิมครบ, `title` ถูกตั้งจากข้อความแรก
4. ส่งข้อความที่ 2 → `seq` ต่อเนื่อง 3,4 ไม่ชน
5. กด Stop กลางทาง → แถว assistant เป็น `interrupted` (หรือไม่มี) และ user message ยังอยู่
6. เปิด 2 แท็บแชทเดียวกัน → ไม่มี unique constraint พัง
7. ไฟล์ `data/app.db` + `app.db-wal` เกิดขึ้น, `PRAGMA user_version = 1`
เครื่องมือ: dev server + curl + `bun:sqlite` query ผ่าน script + ตามด้วย `bun test`
**ถ้า Phase 1 ไม่ผ่าน ห้ามไป Phase 2**

### Phase 2 — build จริง (ไม่ใช่ dev)
- `bun run build` → `bun ./build` → ยิงซ้ำข้อ 1–4 บน production build
- ตรวจ `DATABASE_PATH`, cwd, การสร้าง `data/` อัตโนมัติ, migration ตอน boot
- ตรวจว่า `bun:sqlite` ถูก external ถูกต้องใน bundle

### Phase 3 — เก็บ UI ให้พอใช้
- `ChatView` เดียวใช้ทั้ง 4 หน้า (ทำแล้ว), empty state, error state, stop, auto-scroll, ปุ่ม New chat
- sidebar: ชื่อแชทตัดบรรทัด, active state, จำนวนรายการ, ลบแชท (พร้อม confirm)
- ยังไม่แตะ design system

### Phase 4 — ตัดสิน agent/project (ต้องตอบคำถามก่อน)
- ถ้า agent = persona → ตาราง `agents` + `chats.agent_id`
- ถ้า agent = process → ต้องมี `runs/steps` (คนละงานใหญ่)
- project = กลุ่มแชท + instructions หรือมีไฟล์ → ต่างกันมาก
**ห้ามเดา** — รอคำตอบ §9 ของ ARCHITECTURE.md

### Phase 5 (ทางเลือก, หลัง v1 ใช้ได้)
- model routing (ollama cloud / Groq / DeepSeek), settings ใน DB
- search (FTS5 จาก `messages.text`), attachments (`data/files/`)
- ต่อ Rust math engine เป็น tool — เมื่อมี use case จริง

---

## 6. สิ่งที่ "ไม่ทำ" ใน v1 (non-goals)

auth / multi-user / Postgres / realtime หลายแท็บ / branching UI / attachments / FTS5 / deploy cloud / Rust integration / design polish

---

## 7. ความเสี่ยงและจุดที่จะพัง

| ความเสี่ยง | ผล | มาตรการ |
|---|---|---|
| `Chat` ใน Svelte 5 ไม่อ่าน `messages` เริ่มต้นถูก | refresh แล้วแชทว่าง | Phase 1 ข้อ 3 เป็นตัวตัดสิน |
| SvelteKit dev HMR เปิด DB ซ้ำ | ไฟล์ lock / migration ซ้ำ | singleton `globalThis` + `user_version` กันซ้ำ |
| `onEnd` ไม่ถูกเรียกเมื่อ client ตัด | คำตอบหาย | เก็บ user ก่อน stream; เพิ่ม heartbeat/telemetry ทีหลัง |
| `seq` ชนตอน regenerate | unique index error | คำนวณ seq ใน transaction เดียว + ลบก่อนบันทึก |
| adapter-bun cwd ต่างจาก dev | DB ไปอยู่ผิดที่ | ใช้ `DATABASE_PATH` ชัด ๆ และ log path ตอน boot |
| โมเดล 0.6b ตอบช้า/มั่ว | ดูเหมือนแอปพัง | เก็บ `status/error` ใน DB และแสดงใน UI |
| API key ใน `.env` เป็นของจริง | รั่วถ้า commit | gitignore แล้ว; ไม่เขียนลง DB/log; rotate ถ้าเคย push |

---

## 8. คำถามที่ต้อง answered ก่อน Phase 4 (ไม่บล็อก Phase 1–3)

1. Agent = persona หรือ process?
2. Project = กลุ่มแชท + instructions หรือมีไฟล์?
3. ต้องมี branching (แก้ข้อความ/ตอบใหม่) ใน 6 เดือนไหม?
4. UI 4 หน้าใช้ component เดียวได้ไหม (ตอนนี้ทำแล้ว)

---

## 9. ลำดับงานที่จะทำถัดไป (สั้น ๆ)

1. รัน dev server + ยิง `/api/chat` จริง (สร้างแชทผ่าน `/chat?/default`) → ตรวจ DB ตาม Phase 1
2. เขียน `bun test` ครอบ db layer (migration, save idempotent, seq, title, regenerate)
3. แก้ bug ที่เจอ → ตรวจ Phase 1 ครบ
4. `bun run build` + `bun ./build` → ตรวจ Phase 2
5. เก็บ UI Phase 3
6. หยุด รอคำตอบ Phase 4

**Definition of done ของรอบนี้:** เปิดแอป → คุยกับ qwen3:0.6b → ปิด/refresh → ประวัติยังอยู่ → build production แล้วยังทำงานเหมือน dev
