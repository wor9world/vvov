# VVOV — คู่มือใช้งานจริง (อัปเดต 2026-11-21)

> เขียนจากของที่ **รันและทดสอบจริง** ไม่ใช่ของที่ตั้งใจไว้
> งานออกแบบ/เหตุผลอยู่ที่ [ARCHITECTURE.md](./ARCHITECTURE.md) · แผน/เกณฑ์ตรวจรับอยู่ที่ [PLAN.md](./PLAN.md)

---

## 1. ตอนนี้มีอะไรใช้ได้จริง

| ฟีเจอร์ | สถานะ | หลักฐานที่ตรวจ |
|---|---|---|
| แชทกับ Ollama (local) + เก็บประวัติลง SQLite | ✅ | refresh แล้วประวัติอยู่, `messages` มี seq ต่อเนื่อง |
| Home / รวมแชท / รวม agent / รวม project | ✅ | `GET /`, `/chat`, `/agent`, `/project` = 200 |
| Agent = persona (system prompt + avatar + description) | ✅ | สร้างผ่านฟอร์มแล้วมีแถวใน `agents` |
| Project = กลุ่มแชท + instructions | ✅ | `projects.instructions` ถูก inject เข้า system prompt |
| Tool calling (math ผ่าน Rust engine, clock) | ✅ | `messages.parts` มี `tool-math` พร้อม input/output จริง |
| Markdown + code block + reasoning UI | ✅ | หน้า `/chat/[id]` SSR แสดงคำตอบและ tool block |
| `bun run check` / `bun run build` | ✅ | 0 errors / build ผ่าน (adapter-bun) |

---

## 2. สิ่งที่ต้องมีในเครื่อง

| ของ | เวอร์ชันที่ใช้จริง | หมายเหตุ |
|---|---|---|
| Bun | 1.4.4 | ใช้ทั้ง runtime และ package manager |
| Ollama | รันที่ `127.0.0.1:11434` | ต้องมีโมเดล `qwen3:0.6b` (หรือเปลี่ยน `DEFAULT_MODEL`) |
| Rust engine (ทางเลือก) | `server/target/debug/server.exe` | ใช้เฉพาะ tool `math` — ไม่รันก็ใช้แชทได้ |

```bash
ollama list                 # ต้องเห็น qwen3:0.6b
ollama pull qwen3:0.6b      # ถ้ายังไม่มี
```

---

## 3. คำสั่งรัน

```bash
cd E:\vvov\app

bun install                 # ครั้งแรก / หลัง pull
bun run dev                 # dev server (default 5173) — ระบุพอร์ตได้: bun run dev --port 5180
bun run check               # svelte-check + sync (ต้องได้ 0 errors)
bun run build               # build production ลง .svelte-kit/ + build/
bun ./build                 # รัน production (adapter-bun)
```

- production ตั้ง host/port ได้: `HOST=127.0.0.1 PORT=3000 bun ./build`
- ถ้าเจอ `$lib` ใน error → ผิดแล้ว SvelteKit 3 ลบ `$lib` ออก ใช้ `#lib` เท่านั้น (ดู §9)

---

## 4. Environment variables

| ตัวแปร | ค่าเริ่มต้น | ใช้ทำอะไร |
|---|---|---|
| `DATABASE_PATH` | `data/app.db` (relative กับ cwd) | ที่เก็บไฟล์ SQLite |
| `DEFAULT_MODEL` | `qwen3:0.6b` | โมเดล Ollama ที่ใช้เมื่อ chat ไม่ได้ระบุ |
| `WOR_ENGINE_URL` | `http://127.0.0.1:8080` | ปลายทางของ Rust math engine |
| `OLLAMA_HOST` | `http://127.0.0.1:11434` | ใช้โดย ollama client |
| `OLLAMA_API_KEY` | — | เฉพาะโมเดล cloud ของ Ollama |
| `GROQ_API_KEY` | — | เปิดใช้ provider `groq/...` |
| `GEMINI_API_KEY` | — | เปิดใช้ provider `gemini/...` |
| `DEEPSEEK_API_KEY` | — | เปิดใช้ provider `deepseek/...` |

`app/.env` ถูก gitignore อยู่แล้ว — **ห้าม commit** (ข้างในมี key ของ Groq/Gemini/DeepSeek)

---

## 5. ข้อมูลเก็บที่ไหน / รีเซ็ตยังไง

```
app/data/app.db        ← SQLite (WAL): chats, messages, agents, projects, settings
app/data/app.db-wal    ← WAL (ห้ามลบตอนเซิร์ฟเวอร์รัน)
```

- ตาราง: `chats`, `messages`, `agents`, `projects`, `settings`
- migrations ฝังในโค้ดที่ `app/src/lib/server/db/index.ts` (ใช้ `PRAGMA user_version`) — เพิ่ม migration ใหม่ = เติม entry ใน `MIGRATIONS` แล้วรีสตาร์ท
- รีเซ็ตข้อมูลทั้งหมด: หยุดเซิร์ฟเวอร์ → ลบ `data/app.db*` → เริ่มใหม่ (schema จะถูกสร้างให้อัตโนมัติ)

ตรวจแบบเร็ว:

```bash
cd E:\vvov\app
bun -e "import {Database} from 'bun:sqlite'; const db=new Database('data/app.db'); console.log(db.query('pragma user_version').get()); console.log(db.query('select id,title,updated_at from chats order by updated_at desc limit 5').all());"
```

---

## 6. วิธีใช้หน้าจอ

| หน้า | ทำอะไรได้ |
|---|---|
| `/` | ภาพรวม: จำนวนแชท/agent/project, แชทล่าสุด, ทางเข้าแต่ละ section |
| `/chat` | รวมแชท: เปลี่ยนชื่อ, ลบ (สร้างใหม่ผ่าน `/chat/new`) |
| `/chat/new` | **หน้าแชทใหม่** — ยังไม่สร้างแถวใน DB จนกว่าจะส่งข้อความแรก; URL จะเปลี่ยนเป็น `/chat/<id>` หลังส่ง |
| `/chat/[id]` | แชท: ประวัติ + ช่องพิมพ์ + tool block + reasoning + markdown |
| `/agent` | รวม persona: สร้าง (ชื่อ/emoji/คำอธิบาย/system prompt), ลบ |
| `/agent/[id]` | แก้ persona, เปิดแชทใหม่กับ agent นี้, ดูแชทของ agent |
| `/project` | รวม project: สร้าง (ชื่อ/คำอธิบาย/instructions), ลบ |
| `/project/[id]` | แก้ project, เปิดแชทใหม่ใน project, ดูแชทใน project |

**ลำดับการทำงานของ persona:** `system prompt = project.instructions + "\n\n" + agent.system_prompt` (ถ้าไม่มีทั้งคู่ ใช้ `SYSTEM_PROMPT` default)
โค้ดอยู่ที่ `app/src/routes/api/chat/+server.ts`

---

## 7. Tools

### ที่มีอยู่

| tool | ทำอะไร | ต้องพึ่งอะไร |
|---|---|---|
| `math` | ส่งโจทย์ไป Rust engine (`POST /api/v1/execute`) แล้วคืน `{ok,intent,output}` | `WOR_ENGINE_URL` + engine รันอยู่ |
| `clock` | วัน/เวลาปัจจุบันตาม timezone | ไม่พึ่งอะไร |

นิยามอยู่ที่ `app/src/lib/server/ai/tools.ts` — เปิดให้ทุกแชทอัตโนมัติ, จำกัด 4 steps ต่อการเรียก (`stopWhen`) เพื่อกัน loop

ผลลัพธ์ tool จะถูกเก็บลง `messages.parts` เป็น `tool-<name>` (มี `input`/`output`/`state`) — ดูได้ในหน้าแชทเป็นการ์ดพับได้

### เพิ่ม tool ใหม่

1. เพิ่มใน `app/src/lib/server/ai/tools.ts`:

```ts
export const echoTool = tool({
	description: 'อะไรที่ต้องการ',
	inputSchema: z.object({ text: z.string() }),
	execute: async ({ text }) => ({ ok: true, text })
});

export const tools = { math: mathTool, clock: clockTool, echo: echoTool };
```

2. **ห้าม throw** ออกมาจาก `execute` — คืน `{ ok:false, error }` เพื่อให้โมเดลอธิบายต่อได้
3. รัน `bun run check` แล้วทดสอบแชทด้วยคำสั่งที่บังคับให้เรียก tool

### รัน Rust engine (สำหรับ tool math)

```powershell
# วิธีที่ใช้ได้จริงบนเครื่องนี้: start แบบ detached ไม่ผูก stdin
Start-Process -FilePath "E:\vvov\server\target\debug\server.exe" -WindowStyle Hidden -PassThru
Invoke-RestMethod http://127.0.0.1:8080/health
```

> ⚠️ ห้ามรันแบบ background job ที่ stdin ปิดทันที — binary นี้มี REPL และจะ `exit` เมื่อ stdin EOF (เซิร์ฟเวอร์จะตายตาม)
> rebuild: `cd E:\vvov\server; cargo build`
> หยุด: `Get-Process server | Stop-Process`

ทดสอบเร็ว:

```powershell
Invoke-RestMethod -Uri http://127.0.0.1:8080/api/v1/execute -Method Post -ContentType 'application/json' -Body '{"query":"23x+100=491"}'
# → intent=SolveEquation, output="23x+100 = 491  ==>  x = 17"
```

---

## 8. ทดสอบ end-to-end (ที่ผมใช้จริง)

```powershell
# 1) หน้าเว็บ
Invoke-WebRequest http://127.0.0.1:5180/            # 200
Invoke-WebRequest http://127.0.0.1:5180/chat        # 200

# 2) สร้างแชท (form action) — ได้ redirect ไป /chat/<id>
Invoke-WebRequest -Uri 'http://127.0.0.1:5180/chat?/create' -Method Post -MaximumRedirection 0

# 3) ยิงแชท (SSE) — body ตาม AI SDK
$body = @{
  id = '<chat-id>'
  trigger = 'submit-message'
  messages = @(@{ id='u-1'; role='user'; parts=@(@{ type='text'; text='Use the math tool with query 23x+100=491' }) })
} | ConvertTo-Json -Depth 8
Invoke-WebRequest -Uri 'http://127.0.0.1:5180/api/chat' -Method Post -ContentType 'application/json' -Body $body
```

ตรวจว่า tool ถูกเรียกและบันทึกจริง:

```bash
cd E:\vvov\app
bun -e "import {Database} from 'bun:sqlite'; const db=new Database('data/app.db'); for (const r of db.query('select seq,role,parts,text from messages order by created_at desc limit 2').all()) { const p=JSON.parse(r.parts); console.log(r.seq,r.role,p.map(x=>x.type).join(',')); }"
```

ผลที่ควรเห็น: `user` มี `text`, `assistant` มี `step-start,reasoning,tool-math,...,text`

---

## 9. shadcn-svelte / AI Elements

### สถานะการติดตั้ง

- `components.json`: style `mira`, baseColor `neutral`, **aliases เป็น `#lib/...`** (ไม่ใช่ `$lib`)
- core ที่ติดตั้ง: button, input, textarea, badge, card, separator, empty, spinner, field, label, dialog, alert-dialog, collapsible
- AI Elements: `loader`, `response`, `reasoning` → อยู่ใต้ `src/lib/components/ai-elements/`
- `code-block` + `#lib/highlighter.ts` (shiki แบบ lazy) — ใช้กับ code ที่ AI ส่งมา

### เพิ่ม component

```bash
cd E:\vvov\app
bunx --bun shadcn-svelte@latest add card              # จาก registry หลัก
bunx --bun shadcn-svelte@latest add https://svelte-ai-elements.vercel.app/r/message.json -y
```

- ถ้ามีไฟล์เดิมจะถาม overwrite → ใส่ `--overwrite` (ระวังทับของที่แก้เอง)
- component จาก AI Elements ต้องมี `-y` และมักพ่วง dep (เช่น `streamdown-svelte`, `mode-watcher`)

### กติกาที่ต้องจำ

1. import จาก `#lib/components/...` เท่านั้น — `$lib` ถูก**ลบ**ใน SvelteKit 3 (build จะ error `module_removed_lib`)
2. ใช้ semantic token: `bg-background`, `text-muted-foreground`, `border-border` — เลี่ยงสีดิบ `bg-sky-600`
3. ฟอร์มใช้ `Field.Group` + `Field.Field`; ปุ่มใน `<summary>` ใช้ `buttonVariants()` ไม่ใช่ `<Button>` (button ซ้อน summary ไม่ valid)
4. `Textarea` รับข้อความเป็น **`value={...}`** ไม่ใช่ children
5. `rows`/`maxlength` ต้องเป็น number: `rows={4}` ไม่ใช่ `rows="4"`

---

## 10. ปัญหาที่เจอบ่อย

| อาการ | สาเหตุ | วิธีแก้ |
|---|---|---|
| `Cannot find module '#lib/...'` | tsconfig paths หาย | ตรวจ `tsconfig.json` ต้องมี `paths` สำหรับ `#lib` (มีอยู่แล้ว) |
| build error `module_removed_lib` | เขียน `$lib/...` | เปลี่ยนเป็น `#lib/...` |
| แชทตอบช้า/ค้าง | Ollama ไม่รันหรือโมเดลยังไม่โหลด | `ollama list`, `ollama serve`; ครั้งแรกจะช้าเพราะโหลดโมเดล |
| tool math คืน `engine ไม่พร้อมใช้งาน` | Rust engine ไม่รัน / พอร์ตไม่ตรง | start engine, ตรวจ `WOR_ENGINE_URL` |
| `database is locked` | มี process อื่นเปิด DB หรือ WAL ค้าง | หยุด dev/prod ให้เหลือตัวเดียว, ห้ามลบ `-wal` ตอนรัน |
| หน้าแรกโหลดช้าครั้งแรก | Vite optimize deps | ปกติ — ครั้งถัดไปเร็ว |
| ตาราง/คอลัมน์หายหลังแก้ schema | ยังไม่เพิ่ม migration | เพิ่ม entry ใน `MIGRATIONS` (`db/index.ts`) แล้วรีสตาร์ท (ห้ามแก้ DDL เดิม) |
| สร้าง agent/project แล้วไม่เห็น | ยังไม่รีเฟรชหน้า list | ฟอร์มเป็น full-page action — กลับไปหน้า list แล้วดูใหม่ |

---

## 11. ยังไม่ทำ (จงใจ)

- auth / multi-user / Postgres
- search (FTS5) — กันที่ไว้ด้วยคอลัมน์ `messages.text`
- branching UI (มี `messages.parent_id` เผื่อไว้)
- attachments / ไฟล์
- model selector หลาย provider (มี `agents.model_id` แล้ว แต่ยังไม่มี UI)
- tool UI แบบละเอียด (ตอนนี้เป็นการ์ด JSON พับได้)

---

## 13. Providers หลายตัว + AI ช่วยร่าง (เพิ่ม 2026-11-21)

### รูปแบบ model id

```
<provider>/<model>      เช่น  gemini/gemini-3.8-flash, groq/openai/gpt-oss-120b, deepseek/deepseek-chat
<model>                 ไม่มี prefix = Ollama เช่น qwen3:0.6b, lfm3:1.7b
```

- เลือกได้ 3 ที่:
  1. **`/chat/new`** — dropdown บนหัวหน้า เลือกก่อนส่งข้อความแรก (จะถูกบันทึกเป็น `chat.model_id` ตอนสร้างแถว)
  2. **`/chat/[id]`** — dropdown มุมขวาบน + ปุ่ม "ใช้" (มีผลกับข้อความถัดไป ไม่ย้อนหลัง)
  3. **หน้า agent** — ตอนสร้าง/แก้ persona (ผูกโมเดลกับ agent นั้น)
- priority ของโมเดล: `agent.model_id` → `chat.model_id` → `DEFAULT_MODEL`
- dropdown จะแสดงเฉพาะ provider ที่มี key ใน `.env` — Ollama ใช้รายการจาก `/api/tags` สด ๆ

### provider ที่รองรับ

| provider | โมเดลที่ตั้งไว้ | หมายเหตุ |
|---|---|---|
| `ollama` | ทุกตัวที่ `ollama list` (ยกเว้น embedding) | local, เชื่อถือได้สุด |
| `gemini` | `gemini-3.8-flash`, `gemini-flash-latest` | ฟรี tier มี 503 บ่อยช่วงคนใช้เยอะ |
| `groq` | `openai/gpt-oss-120b`, `openai/gpt-oss-20b`, `qwen/qwen3.8-27b` | เร็ว; ชื่อโมเดลเปลี่ยนบ่อย |
| `deepseek` | `deepseek-chat` | ใช้ schema แบบ compatibility mode |

> ชื่อโมเดลของผู้ให้บริการเปลี่ยนได้ตลอด — ถ้าเจอ `model_not_found` ให้เช็ก `GET /v1/models` ของ provider นั้นแล้วแก้ที่ `src/lib/server/ai/models.ts`

### AI ช่วยร่าง agent/project

- ปุ่ม **"ร่างให้หน่อย"** ในฟอร์มสร้าง agent/project → `POST /api/draft` `{ kind, prompt }`
- ลำดับโมเดล: gemini → deepseek → groq → ollama (ไล่จนสำเร็จ; ถ้าตัวแรก 503/404 จะข้ามให้)
- ผลลัพธ์ถูกเติมลงฟอร์มให้แก้ต่อก่อนกดสร้าง — ไม่ได้สร้างแถวอัตโนมัติ
- ไฟล์: `src/lib/server/ai/draft.ts` (schema อยู่ที่นี่), endpoint: `src/routes/api/draft/+server.ts`

---

## 14. Cheat sheet

```bash
# dev
cd E:\vvov\app && bun run dev --port 5180

# ตรวจคุณภาพ
bun run check && bun run build

# engine (terminal แยก หรือ start detached)
cd E:\vvov\server && cargo run

# DB
bun -e "import {Database} from 'bun:sqlite'; new Database('data/app.db').query('select count(*) n from messages').get()"
```
