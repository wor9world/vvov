/**
 * Minimal shiki wrapper used by `#lib/hooks/use-highlighted-code.svelte.ts` + `code-block`.
 * โหลด grammar/theme แบบ lazy ต่อภาษา เพื่อไม่ให้ bundle โตทั้งชุด
 */
import { createHighlighterCore } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import type { GrammarState } from 'shiki/core';

export type HighlightedToken = { content: string; style?: string };
export type HighlightedLine = string | HighlightedToken[];

export interface Highlighter {
	highlight(
		code: string,
		state?: GrammarState
	): { lines: HighlightedLine[]; state?: GrammarState };
}

const THEME = 'github-light';

const LANG_LOADERS: Record<string, () => Promise<unknown>> = {
	ts: () => import('shiki/langs/typescript.mjs'),
	typescript: () => import('shiki/langs/typescript.mjs'),
	js: () => import('shiki/langs/javascript.mjs'),
	javascript: () => import('shiki/langs/javascript.mjs'),
	json: () => import('shiki/langs/json.mjs'),
	bash: () => import('shiki/langs/bash.mjs'),
	sh: () => import('shiki/langs/bash.mjs'),
	shell: () => import('shiki/langs/bash.mjs'),
	python: () => import('shiki/langs/python.mjs'),
	py: () => import('shiki/langs/python.mjs'),
	rust: () => import('shiki/langs/rust.mjs'),
	rs: () => import('shiki/langs/rust.mjs'),
	svelte: () => import('shiki/langs/svelte.mjs'),
	html: () => import('shiki/langs/html.mjs'),
	css: () => import('shiki/langs/css.mjs'),
	sql: () => import('shiki/langs/sql.mjs'),
	markdown: () => import('shiki/langs/markdown.mjs'),
	md: () => import('shiki/langs/markdown.mjs'),
	diff: () => import('shiki/langs/diff.mjs'),
	yaml: () => import('shiki/langs/yaml.mjs'),
	yml: () => import('shiki/langs/yaml.mjs')
};

const cache = new Map<string, Promise<Highlighter | null>>();

export function loadHighlighter(lang: string): Promise<Highlighter | null> {
	const key = lang.toLowerCase();
	const existing = cache.get(key);
	if (existing) return existing;

	const promise = create(key).catch((cause) => {
		console.warn(`[highlighter] ${key} unavailable`, cause);
		return null;
	});
	cache.set(key, promise);
	return promise;
}

async function create(lang: string): Promise<Highlighter | null> {
	const loadLang = LANG_LOADERS[lang];
	if (!loadLang) return null;

	const [grammar, theme] = await Promise.all([
		loadLang() as Promise<{ default: unknown }>,
		import('shiki/themes/github-light.mjs') as Promise<{ default: unknown }>
	]);

	const core = await createHighlighterCore({
		themes: [theme.default as never],
		langs: [grammar.default as never],
		engine: createJavaScriptRegexEngine()
	});

	return {
		highlight(code) {
			const { tokens } = core.codeToTokens(code, { lang, theme: THEME });
			return {
				lines: tokens.map((line) =>
					line.map((token) => ({
						content: token.content,
						style: token.color ? `color:${token.color}` : undefined
					}))
				)
			};
		}
	};
}
