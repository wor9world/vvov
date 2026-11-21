import { createGoogle } from '@ai-sdk/google';
import { createDeepSeek } from '@ai-sdk/deepseek';
import { createGroq } from '@ai-sdk/groq';
import { ollama } from 'ai-sdk-ollama';

/**
 * Model registry รองรับหลาย provider
 * รูปแบบ id: `provider/model` เช่น `groq/llama-3.3-70b-versatile`
 * ถ้าไม่มี prefix ที่รู้จัก = ถือว่าเป็นโมเดล Ollama (เช่น `qwen3:0.6b`)
 */

export type ProviderId = 'ollama' | 'groq' | 'gemini' | 'deepseek';

export const DEFAULT_MODEL = process.env.DEFAULT_MODEL ?? 'qwen3:0.6b';

export const SYSTEM_PROMPT = 'You are a helpful assistant.';

const OLLAMA_BASE_URL = process.env.OLLAMA_BASE_URL ?? 'http://127.0.0.1:11434';

const KNOWN_PROVIDERS: ProviderId[] = ['ollama', 'groq', 'gemini', 'deepseek'];

export function parseModelId(id: string): { provider: ProviderId; model: string } {
	const slash = id.indexOf('/');

	if (slash > 0) {
		const prefix = id.slice(0, slash) as ProviderId;
		const model = id.slice(slash + 1).trim();

		// เฉพาะ prefix ที่รู้จักเท่านั้น — ชื่อโมเดล Ollama บางตัวมี / อยู่ในชื่อ
		if (model && KNOWN_PROVIDERS.includes(prefix)) {
			return { provider: prefix, model };
		}
	}

	return { provider: 'ollama', model: id };
}

const groqProvider = createGroq({ apiKey: process.env.GROQ_API_KEY });
const googleProvider = createGoogle({
	apiKey: process.env.GOOGLE_GENERATIVE_AI_API_KEY ?? process.env.GEMINI_API_KEY
});
const deepseekProvider = createDeepSeek({ apiKey: process.env.DEEPSEEK_API_KEY });

export function languageModel(modelId?: string | null) {
	const { provider, model } = parseModelId(modelId?.trim() || DEFAULT_MODEL);

	switch (provider) {
		case 'groq':
			return groqProvider(model);
		case 'gemini':
			return googleProvider(model);
		case 'deepseek':
			return deepseekProvider(model);
		default:
			return ollama(model);
	}
}

export interface ModelInfo {
	id: string;
	label: string;
	provider: ProviderId;
}

interface ProviderConfig {
	keyEnv: string;
	models: Array<{ id: string; label: string }>;
}

const PROVIDERS: Record<Exclude<ProviderId, 'ollama'>, ProviderConfig> = {
	groq: {
		keyEnv: 'GROQ_API_KEY',
		models: [
			{ id: 'groq/openai/gpt-oss-120b', label: 'groq · gpt-oss-120b' },
			{ id: 'groq/openai/gpt-oss-20b', label: 'groq · gpt-oss-20b' },
			{ id: 'groq/qwen/qwen3.8-27b', label: 'groq · qwen3.8-27b' }
		]
	},
	gemini: {
		keyEnv: 'GEMINI_API_KEY',
		models: [
			{ id: 'gemini/gemini-3.8-flash', label: 'gemini · 3.8-flash' },
			{ id: 'gemini/gemini-flash-latest', label: 'gemini · flash-latest' }
		]
	},
	deepseek: {
		keyEnv: 'DEEPSEEK_API_KEY',
		models: [{ id: 'deepseek/deepseek-chat', label: 'deepseek · chat' }]
	}
};

function providerHasKey(provider: Exclude<ProviderId, 'ollama'>): boolean {
	if (provider === 'gemini') {
		return Boolean(process.env.GEMINI_API_KEY ?? process.env.GOOGLE_GENERATIVE_AI_API_KEY);
	}
	return Boolean(process.env[PROVIDERS[provider].keyEnv]);
}

const FALLBACK_OLLAMA: ModelInfo[] = [
	{ id: 'qwen3:0.6b', label: 'qwen3 0.6b (เร็ว)', provider: 'ollama' },
	{ id: 'lfm3:1.7b', label: 'lfm3 1.7b', provider: 'ollama' },
	{ id: 'qwen3:1.7b', label: 'qwen3 1.7b', provider: 'ollama' }
];

/** โมเดลที่ใช้แก้ embedding ไม่ควรโผล่ใน dropdown แชท */
const NOT_CHAT_MODEL = /embed|nomic/i;

export function modelLabel(id: string): string {
	for (const config of Object.values(PROVIDERS)) {
		const found = config.models.find((model) => model.id === id);
		if (found) return found.label;
	}

	return FALLBACK_OLLAMA.find((model) => model.id === id)?.label ?? id;
}

async function listOllamaModels(): Promise<ModelInfo[]> {
	try {
		const response = await fetch(`${OLLAMA_BASE_URL}/api/tags`, {
			signal: AbortSignal.timeout(2500)
		});
		if (!response.ok) throw new Error(`HTTP ${response.status}`);

		const data = (await response.json()) as { models?: Array<{ name: string }> };
		const ids = (data.models ?? [])
			.map((model) => model.name)
			.filter((name) => name && !NOT_CHAT_MODEL.test(name));

		if (ids.length > 0) {
			return ids.map((id) => ({ id, label: modelLabel(id), provider: 'ollama' as const }));
		}
	} catch (cause) {
		console.warn('[models] ใช้ fallback เพราะดึงจาก Ollama ไม่ได้:', String(cause));
	}

	return FALLBACK_OLLAMA;
}

/** รวมโมเดล Ollama + provider ที่มี API key อยู่ใน env */
export async function listModels(): Promise<ModelInfo[]> {
	const ollamaModels = await listOllamaModels();
	const remote: ModelInfo[] = [];

	for (const provider of ['gemini', 'groq', 'deepseek'] as const) {
		if (!providerHasKey(provider)) continue;
		for (const model of PROVIDERS[provider].models) {
			remote.push({ ...model, provider });
		}
	}

	return [...remote, ...ollamaModels];
}

/** เลือกโมเดลที่ "เก่งพอ" สำหรับงานร่างข้อความ — ใช้ cloud ถ้ามี key */
export function pickDraftModel(): string {
	if (providerHasKey('gemini')) return 'gemini/gemini-3.8-flash';
	if (providerHasKey('deepseek')) return 'deepseek/deepseek-chat';
	if (providerHasKey('groq')) return 'groq/openai/gpt-oss-120b';
	return DEFAULT_MODEL;
}

/** ลำดับโมเดลที่จะลองสำหรับงานร่าง (ตัวแรกไม่work จะไล่ต่อ) */
export function draftModelCandidates(): string[] {
	const candidates = [pickDraftModel()];

	if (providerHasKey('deepseek')) candidates.push('deepseek/deepseek-chat');
	if (providerHasKey('groq')) candidates.push('groq/openai/gpt-oss-120b');
	candidates.push(DEFAULT_MODEL);

	return [...new Set(candidates)];
}
