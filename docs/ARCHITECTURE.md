# VVOV — Architecture & Database Decision Record (v1)

> สถานะ: **proposed** · 2026-11-21 · ยังไม่เขียนโค้ดจนกว่าจะเคาะ §9
> ภาษา: ไทยในคำอธิบาย, identifier/โค้ดเป็นอังกฤษ

---

## 0. ข้อเท็จจริงที่ตรวจจากโค้ด (ไม่ใช่การเดา)

| เรื่อง | สถานะจริง |
|---|---|
| Frontend | SvelteKit 3.4 + Svelte 5.59 (runes) + Tailwind 4 + adapter-bun 1.0 |
| AI layer | `ai@7` + `@ai-sdk/svelte` (`Chat`) + `ai-sdk-ollama` (default `qwen3:0.6b`) |
| Routes | `/` → `/chat`; `/chat/[id]` แชทจริง; `/agent/[id]`, `/project/[id]` — ใช้ `ChatView` ตัวเดียว |
| Backend เดิม | `server/` = axum 0.7, stateless, `/api/v1/execute` + math/SAT engine + tests |
| Database | **มีแล้ว (Phase 0)** — `data/app.db`, `bun:sqlite`, ตาราง chats/messages/settings |
| เบาะแสที่ตั้งใจไว้แล้ว | `app/vite.config.ts` มี `ssr.external: ["bun:sqlite"]` → คนเขียนเดิมเล็ง SQLite ผ่าน Bun ไว้แล้ว |
| Runtime เครื่องนี้ | bun 1.4.4, cargo 1.99.1, Windows, ไม่มี `sqlite3` CLI (ไม่จำเป็น) |
| Git | ยังไม่มี repo; `app/.gitignore` ignore `.env` แล้ว แต่ `.env` มี API key จริง (Groq/Gemini/DeepSeek) |
| Sidebars | ซ้าย: Agent / Project / Chat — ขวา: feed / tasks / tools / library (ยังเป็น icon เปล่า) |

**ปัญหาที่ต้องแก้:** ตอนนี้ทุกหน้าแชทไม่มี id, ไม่มี history, refresh แล้วหาย และทั้ง 4 route ทำงานเหมือนกันทุกอย่าง — ยังไม่มี "ที่เก็บความจริง" ของ agent/project/chat เลย

---

## 1. คำตัดสิน (สรุปสั้น)

| # | ตัดสินใจ | ทางเลือกที่ตัดทิ้ง | ต้นทุนถอยหลัง |
|---|---|---|---|
| D1 | **SQLite ผ่าน `bun:sqlite` + WAL** ไฟล์เดียว `data/app.db` | Postgres, MySQL, libsql/Turso | ต่ำ (schema เป็น portable SQL) |
| D2 | **SvelteKit (Bun) เป็นเจ้าของ DB แต่เพียงผู้เดียว** — process เดียว เขียนที่เดียว | ให้ Rust ถือ DB / ให้ browser ถือ state | ต่ำ |
| D3 | **`bun:sqlite` ตรง ๆ ไม่มี ORM** — typed row interfaces + prepared statements | Drizzle, Prisma, Kysely | ต่ำ (query รวมอยู่ใน repos ไม่กี่ไฟล์) |
| D4 | **เก็บ message เป็น UIMessage JSON ต่อแถว** ตามแบบ AI SDK (`id / role / parts`) + คอลัมน์สรุป | normalize parts/tool_calls, blob ทั้ง chat | ต่ำ |
| D5 | **id เป็น TEXT** (UUID) — ไม่ใช้ AUTOINCREMENT | integer PK | ต่ำ |
| D6 | **เวลาเป็น INTEGER epoch ms** | ISO text | ต่ำ |
| D7 | **Migrate ตอน boot** ด้วย DDL ฝังในโค้ด + `PRAGMA user_version` | ไฟล์ SQL แยก + runner, migrate มือ | ต่ำ |
| D8 | **Rust engine หลุดจาก critical path ของ v1** — เก็บโค้ดไว้เป็น sidecar/tool ทีหลัง | ผูก chat เข้ากับ axum ทันที, พอร์ต calculator เป็น TS ตอนนี้ | ต่ำ |
| D9 | **ไม่มี auth ใน v1** bind `127.0.0.1` เท่านั้น | ทำ user/login เผื่อไว้ | กลาง (ต้องเพิ่มตาราง + ทุก query) |
| D10 | **ต่อยอด branching ได้** ด้วย `messages.parent_id` (nullable, ไม่มี UI ใน v1) | เพิ่มทีหลัง | ต่ำ (เพิ่ม column ได้ แต่ backfill ยากถ้ามีข้อมูลแล้ว) |
| D11 | **Search ยังไม่ทำ** แต่กันที่ไว้ด้วยคอลัมน์ `text` ที่ strip จาก parts | FTS5 ทันที | ต่ำ |

> หัวใจ: **local-first, process เดียว, SQLite, AI SDK เป็นเจ้าของรูปทรงข้อความ**
> ถ้าวันหนึ่งต้องเป็น multi-user/cloud ค่อยเปลี่ยน D1+D9 พร้อมกันทีเดียว (ดู §8)

---

## 2. สถาปัตยกรรมเป้าหมาย

```
┌──────────────────────────────────────────────────────────────┐
│ Browser (Svelte 5, runes)                                    │
│  /chat/[id]  ← Chat จาก @ai-sdk/svelte (stream ผ่าน fetch)   │
└───────────────┬──────────────────────────────────────────────┘
                │  POST /api/chat  (UIMessage stream, SSE)
                │  GET  /chat/[id] (load: history จาก DB)
┌───────────────▼──────────────────────────────────────────────┐
│ SvelteKit server — Bun 1.4 (process เดียว, single writer)    │
│                                                              │
│  routes/api/chat/+server.ts   ← streaming + persist          │
│  routes/**/+page.server.ts    ← load (อ่าน DB)               │
│  lib/server/db/{schema,client,repos}                         │
│  lib/server/ai/{models,persist}                              │
└───────┬───────────────────────────────┬──────────────────────┘
        │                               │
┌───────▼────────┐            ┌─────────▼──────────┐
│ data/app.db    │            │ Ollama (localhost) │
│ SQLite + WAL   │            │ qwen3:0.6b         │
└────────────────┘            └────────────────────┘
                                        ▲
                     (อนาคต, ไม่ผูกใน v1) │ HTTP /api/v1/execute
                              ┌─────────┴──────────┐
                              │ server/ (axum)     │  stateless tool
                              │ math / SAT engine  │  ไม่แตะ DB
                              └────────────────────┘
```

**กฎที่ต้องยึด:**

1. มี process เดียวที่เปิดไฟล์ DB — ห้ามให้ Rust หรือ process อื่นเปิด `app.db`
2. component ห้าม import DB โดยตรง — ผ่าน `load` / `+server` / repository เท่านั้น
3. `lib/server/**` เท่านั้นที่แตะ DB (SvelteKit กัน client import ให้อยู่แล้ว)
4. DB client เป็น **singleton บน `globalThis`** เพื่อไม่ให้ HMR/dev reload เปิด connection ซ้ำ
5. Rust (ถ้าใช้) เป็น leaf: รับ input → คืนผล → ไม่มี state ไม่มี DB ไม่มี auth

### ทำไมไม่ให้ Rust เป็น backend หลัก

- ต้องเขียน auth, streaming (SSE), persistence, migration ใหม่ทั้งหมดใน Rust
- SvelteKit มี `load` + form actions/remote functions + AI SDK streaming อยู่แล้ว
- ภาษาเดียว = debug ง่าย, deploy ง่าย, dev loop เร็ว (bun --hot)
- math engine เป็น **tool** ไม่ใช่ application — เก็บไว้เป็น sidecar ได้ทุกเมื่อโดยไม่ต้องรื้อสถาปัตยกรรม

**เกณฑ์ตัดสิน Rust (เขียนไว้ล่วงหน้าเพื่อไม่ให้ค้าง):** ถ้าภายใน 2 สัปดาห์หลัง v1 ยังไม่มี route ไหนเรียก engine จริง → ย้ายเป็น `tools/math/` script หรือลบทิ้ง อย่าแบก process ที่ไม่มีคนเรียก

---

## 3. Data model v1

หลักคิด: **น้อยตารางที่สุดที่ยัง replay แชทได้ครบ + ต่อยอด agent/project ได้**
ยังไม่ทำ: users, teams, files/attachments, tasks, library, feed (รอ use case จริง)

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous  = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;

CREATE TABLE agents (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  system_prompt TEXT NOT NULL DEFAULT '',
  model_id      TEXT,                        -- NULL = ใช้ default
  tools_json    TEXT NOT NULL DEFAULT '[]',  -- allowlist ของ tool
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

CREATE TABLE projects (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  description  TEXT NOT NULL DEFAULT '',
  instructions TEXT NOT NULL DEFAULT '',     -- context ที่ inject ทุกแชทใน project
  created_at   INTEGER NOT NULL,
  updated_at   INTEGER NOT NULL
);

CREATE TABLE chats (
  id               TEXT PRIMARY KEY,
  project_id       TEXT REFERENCES projects(id) ON DELETE SET NULL,
  agent_id         TEXT REFERENCES agents(id)   ON DELETE SET NULL,
  title            TEXT NOT NULL DEFAULT 'New chat',
  model_id         TEXT,                     -- snapshot ตอนสร้าง (กันค่า default เปลี่ยนแล้วประวัติเพี้ยน)
  metadata         TEXT NOT NULL DEFAULT '{}',
  created_at       INTEGER NOT NULL,
  updated_at       INTEGER NOT NULL,
  last_message_at  INTEGER
);

CREATE TABLE messages (
  id          TEXT PRIMARY KEY,
  chat_id     TEXT NOT NULL REFERENCES chats(id) ON DELETE CASCADE,
  parent_id   TEXT REFERENCES messages(id) ON DELETE SET NULL,  -- เผื่อ edit/regenerate branching
  seq         INTEGER NOT NULL,              -- ลำดับในแชท, เริ่ม 1, ไม่มีช่องว่าง
  role        TEXT NOT NULL CHECK (role IN ('user','assistant','system')),
  parts       TEXT NOT NULL,                 -- JSON = UIMessage['parts'] ตรง ๆ
  text        TEXT NOT NULL DEFAULT '',      -- ข้อความล้วน สำหรับ preview/ค้นหา (strip จาก parts)
  status      TEXT NOT NULL DEFAULT 'complete'
              CHECK (status IN ('streaming','complete','error','interrupted')),
  model_id    TEXT,                          -- ผู้สร้างจริง (assistant)
  usage       TEXT,                          -- JSON: tokens/ต้นทุน ถ้ามี
  error       TEXT,
  created_at  INTEGER NOT NULL
);
CREATE UNIQUE INDEX messages_chat_seq ON messages(chat_id, seq);
CREATE INDEX messages_chat_created   ON messages(chat_id, created_at);

CREATE INDEX chats_updated  ON chats(updated_at DESC);
CREATE INDEX chats_project  ON chats(project_id, updated_at DESC);
CREATE INDEX chats_agent    ON chats(agent_id, updated_at DESC);

CREATE TABLE settings (
  key        TEXT PRIMARY KEY,
  value      TEXT NOT NULL,                  -- JSON
  updated_at INTEGER NOT NULL
);
```

> **v1 ที่ implement จริง:** `chats` (ยังไม่มี `project_id` / `agent_id`), `messages`, `settings`
> คอลัมน์ project/agent รอคำตอบ §9 ข้อ 1–2 ก่อน — เพิ่มทีหลังด้วย migration ใหม่ได้

### ทำไมออกแบบแบบนี้

- **`parts` เป็น JSON ไม่ normalize**: AI SDK 7 นิยามข้อความเป็น `UIMessage.parts` (text/reasoning/tool-*/file) — เก็บตามรูปเดิมทำให้ replay คือ `JSON.parse` ตรง ๆ ไม่ต้อง map กลับ และไม่ต้องแก้ schema เมื่อ AI SDK เพิ่ม part type ใหม่
- **`text` แยกคอลัมน์**: ได้ทั้ง title preview, `LIKE` search, และ FTS5 ทีหลัง โดยไม่ต้อง parse JSON ทุกครั้ง
- **`seq` + UNIQUE(chat_id, seq)**: ลำดับข้อความไม่พึ่ง timestamp (ซึ่งชนกันได้เวลา regenerate) และ pagination ทำ `WHERE seq < ? ORDER BY seq DESC LIMIT 50`
- **`parent_id`**: ต้นทุน 1 column วันนี้ ถูกกว่าการ backfill ทีหลังถ้าอยากได้ "แก้ข้อความแล้วตอบใหม่" / branch
- **`model_id` ทั้ง chats และ messages**: chat เก็บ snapshot นโยบาย, message เก็บความจริงว่าใครตอบ — เปลี่ยน default model แล้วประวัติเก่าไม่โกหก
- **`status`**: ทำให้รู้ว่าแถวไหนเป็นข้อความที่สตรีมไม่จบ (ตอน boot sweep เป็น `interrupted`)
- **ไม่มี `users`**: local single-user; ถ้าจะ multi-user ทีหลัง เพิ่ม `owner_id` + index (owner, updated_at) พร้อมกันทุกตาราง

### ขนาด/ประสิทธิภาพ

- 1 แชท 1k ข้อความ ≈ ไม่กี่ MB; SQLite รับได้ถึงระดับแสนแชทสบาย
- `bun:sqlite` เป็น **synchronous** → query ต้องมี index และเลือกคอลัมน์เท่าที่ใช้ (อย่า `SELECT *` ทั้งแชท)
- WAL ทำให้อ่านระหว่างเขียนได้; `synchronous=NORMAL` ยอมเสี่ยงเสีย transaction ล่าสุดตอนไฟดับ แต่ไม่ corrupt — เหมาะกับ local app

---

## 4. Flow: ส่งข้อความ → สตรีม → บันทึก

```
1. ผู้ใช้กดส่ง
   client: Chat.sendMessage({ text })
   POST /api/chat  { id: chatId, messages: UIMessage[], trigger, messageId }

2. server (+server.ts)
   a. validateUIMessages(body.messages) → UIMessage[]
   b. ensureChat(id) — chat ถูกสร้างจาก form action ตอนเปิดหน้า ไม่ใช่จาก body
   c. saveMessages(chatId, uiMessages)  // เก็บ user ก่อน stream (idempotent ตาม id)
   d. streamText({ model, system, messages: convertToModelMessages(...), abortSignal })
   e. toUIMessageStream({ stream, originalMessages, generateMessageId, onEnd })

3. onEnd (หลัง stream จบ — จุดเดียวที่เขียน assistant)
   - regenerate: deleteMessagesFrom(seq ของ messageId เดิม)
   - saveMessages([responseMessage], { status: complete | error | interrupted })
   - seq / last_message_at / updated_at / title ใน transaction เดียว

4. return createUIMessageStreamResponse({ stream }) → client อัปเดต UI; refresh แล้ว load จาก DB
```

**รายละเอียดที่พลาดบ่อย:**

- **Idempotency**: ข้อความจาก client มี id อยู่แล้ว → `ON CONFLICT(id) DO UPDATE` ทำให้ retry/network ซ้ำไม่สร้างแถวซ้ำ
- **chat id มาจาก server**: ปุ่ม "แชทใหม่" → form action `POST /chat?/default` → redirect `/chat/<id>` ก่อนพิมพ์ข้อความแรก — ไม่มี `/api/chats` แล้ว (CRUD ใช้ `load` + form action ของ SvelteKit)
- **สตรีมค้าง**: ถ้า process ตายกลาง stream แถว assistant อาจไม่มีเลย (ถ้าเขียนเฉพาะตอนจบ) — v1 ยอมรับได้; ทางอัปเกรดคือสร้างแถว `status='streaming'` ก่อนเริ่ม แล้ว update ตอนจบ + sweep เป็น `interrupted` ตอน boot (ทำเมื่อมีปัญหาจริง)
- **onEnd ต้องไม่ทำให้ response พัง**: wrap try/catch + log; เขียน DB ล้มไม่ควรทำให้ผู้ใช้ไม่ได้คำตอบ
- **Abort**: ผูก `request.signal`; เมื่อผู้ใช้กดหยุด ให้บันทึก parts เท่าที่ได้ + `status='interrupted'`

---

## 5. โครงสร้างไฟล์เป้าหมาย

```
E:\vvov\
  app\
    data\app.db                      # gitignore (WAL)
    src\lib\server\
      db\index.ts                    # เปิด DB + PRAGMA + migration (user_version)
      db\chats.ts                    # chats repo
      db\messages.ts                 # messages repo (parts JSON, seq, status)
      ai\models.ts                   # Ollama provider + DEFAULT_MODEL
    src\lib\components\chat\ChatView.svelte
    src\routes\
      +layout.server.ts              # sidebar: listChats
      +page.server.ts                # → /chat
      chat\+page.server.ts           # load → latest/new; action default = สร้างแชท
      chat\[id]\+page.server.ts      # load history
      chat\[id]\+page.svelte         # {#key} + ChatView
      agent\[id], project\[id]       # chat id แบบ namespaced (agent:<id>)
      api\chat\+server.ts            # streaming + persist (endpoint เดียว)
  server\                            # Rust — คงไว้, ไม่อยู่ใน v1 path
  docs\ARCHITECTURE.md               # ไฟล์นี้
  docs\PLAN.md                       # แผน + เกณฑ์ตรวจรับ
  main.rs                            # scaffolder — ย้ายไป tools\ ทีหลัง
```

**Layout ของ UI:** `/` = home/launcher, `/chat/[id]` = แชทเดี่ยว, `/project/[id]` = แชทที่ผูก project (system context + รายการแชทใน project), `/agent/[id]` = แชทที่ผูก agent (system prompt + tools)
→ ทั้ง 4 หน้าใช้ **component เดียว** (`<ChatView>`) ต่างกันแค่ loader/context — ลบ copy-paste 4 ชุดออก

---

## 6. ทางเลือกที่ปฏิเสธ (และเพราะอะไร)

| ทางเลือก | เหตุผลที่ปฏิเสธ |
|---|---|
| Postgres ทันที | ต้องมี server process + backup + connection config บนเครื่องเดียว; ยังไม่มี multi-user จึงได้ศูนย์ |
| ORM (Prisma/Drizzle) | เพิ่ม dependency + query layer ที่ยังไม่จำเป็น; `bun:sqlite` ตรง ๆ พอสำหรับ 3 ตาราง และคุม SQL ได้เต็ม |
| เก็บ blob ทั้ง chat | เขียนทับทั้งก้อนทุกครั้ง, เสี่ยง lost update, ค้นหา/แบ่งหน้าไม่ได้ |
| normalize `parts` เป็นตาราง | ต้อง map กลับเป็น `UIMessage` ทุกครั้งที่โหลด; AI SDK เพิ่ม part type แล้วต้อง migrate |
| localStorage/IndexedDB เป็นที่หลัก | หายเมื่อล้าง browser, ไม่มี backup, server อ่านไม่ได้, ขยายเป็น agent/project ยาก |
| ให้ Rust ถือ DB | สองภาษา สอง migration สอง connection writer; ไม่มีประโยชน์กับ local app |
| Auth/JWT ตั้งแต่แรก | ไม่มีผู้ใช้ที่สอง; ทำเมื่อมีคนที่สองจริง |
| FTS5 ทันที | ยังไม่มีข้อมูลให้ค้น; `text` column กันที่ไว้แล้ว เพิ่ม virtual table + trigger ได้ใน 1 migration |

---

## 7. ความเสี่ยงที่ต้องเฝ้า

1. **Dev/HMR เปิด DB ซ้ำ** → singleton บน `globalThis`, และปิด connection ตอน `import.meta.hot.dispose`
2. **SvelteKit build แล้ว path `db/migrations` หาย** → ตอน deploy ให้ copy migrations ติดไปกับ output หรือ embed SQL ไว้ใน bundle (ตัดสินตอน deploy จริง)
3. **Windows file lock / antivirus สแกน `app.db-wal`** → เก็บ DB ใน `data/` ใต้โปรเจกต์ (ไม่ใช่ OneDrive/temp) และปิด realtime scan เฉพาะโฟลเดอร์ถ้าพบปัญหา
4. **ข้อความยาว/มี reasoning + tool output** → `parts` JSON อาจใหญ่หลายร้อย KB; จำกัด `maxOutputTokens` และ paginate
5. **API key ใน `.env` เป็นของจริง** → อย่า commit, อย่าใส่ใน DB, อย่า log; ถ้าเคย push ที่ไหนให้ rotate
6. **Streaming + DB write race** → เขียนใน `onEnd` ครั้งเดียวต่อ turn, อย่าเขียนทุก chunk
7. **สองหน้าต่างเปิดแชทเดียวกัน** → ยอมรับ last-write-wins ใน v1; ถ้าต้องการ real-time ค่อยเพิ่ม polling/`BroadcastChannel`

---

## 8. ถ้าวันหนึ่งต้องโต (ทางออกที่เตรียมไว้)

- **Multi-user**: เพิ่ม `users` + `owner_id` ทุกตาราง + auth (Bun `Bun.password` หรือ better-auth) → ค่อยพิจารณาย้าย Postgres
- **SQLite → Postgres**: schema นี้ portable อยู่แล้ว (TEXT id, INTEGER ms, JSON เป็น TEXT) — เปลี่ยน driver/query layer; SQL ส่วนใหญ่ใช้ได้แทบเดิม
- **Sync ข้ามเครื่อง**: อย่าทำ hand-rolled; ใช้ Turso/libsql replication หรือ Postgres ตรง ๆ
- **Attachments**: `files(id, chat_id, name, mime, bytes)` + ไฟล์จริงใน `data/files/<id>` (อย่าเก็บ blob ใน DB)
- **Search**: `messages_fts` (FTS5) + trigger จาก `text` — ไม่ต้องแก้ตารางเดิม

---

## 9. ยังไม่ตัดสิน — ต้องตอบก่อนลงโค้ด

1. **Agent คืออะไรกันแน่?** ✅ **answered 2026-11-21: persona/expert** (แบบ grok bot) — มี `name`, `description`, `avatar`, `system_prompt`, `model_id`; หนึ่ง agent มีได้หลายแชท; system prompt มาจาก DB เท่านั้น (client ส่งมาไม่ได้)
   → ถ้าเป็น process ต้องมี `runs`, `steps`, สถานะ, queue (คนละเรื่องกับแชท) — ยังไม่ทำ
2. **Project คืออะไร?** แค่กลุ่มแชท + instructions หรือมีไฟล์/โฟลเดอร์ทำงาน (เขียนไฟล์ได้) ด้วย?
   → ถ้ามีไฟล์ ต้องตัดสินเรื่อง sandbox/permission ก่อนแตะ schema
3. **ต้องมี branching (แก้ข้อความ/ตอบใหม่) ใน 6 เดือนไหม?** → กำหนดว่าจะใช้ `parent_id` จริงหรือปล่อยทิ้ง
4. **UI 4 หน้า (/, /chat, /agent, /project) ใช้ component เดียวได้ไหม?** → ถ้าได้ จะลดโค้ดซ้ำ 4 เท่าก่อนทำ DB

## 10. ลำดับงานถัดไป

แผนเต็ม + เกณฑ์ตรวจรับอยู่ใน [PLAN.md](./PLAN.md) — ลำดับย่อ:

1. **Phase 1** — รันจริง: ส่งข้อความ → stream → refresh แล้วประวัติอยู่ + `bun test` ครอบ db layer
2. **Phase 2** — `bun run build` + `bun ./build` (dev ผ่านไม่เท่ากับ build ผ่าน)
3. **Phase 3** — เก็บ UI (ChatView ตัวเดียว, empty/error/stop, sidebar list/ลบ)
4. **Phase 4** — agent/project CRUD เมื่อ §9 ข้อ 1–2 มีคำตอบ
5. (ทางเลือก) model routing, search, attachments, Rust math tool
