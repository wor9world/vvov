<script lang="ts">
	import { Chat } from '@ai-sdk/svelte';
	import type { UIMessage } from 'ai';
	import { untrack } from 'svelte';
	import { replaceState } from '$app/navigation';
	import {
		ArrowUp,
		BookOpen,
		Check,
		Code,
		Copy,
		FileText,
		Mail,
		RefreshCw,
		Square
	} from '@lucide/svelte/icons';
	import { Response } from '#lib/components/ai-elements/response';
	import * as Reasoning from '#lib/components/ai-elements/reasoning';
	import { Loader } from '#lib/components/ai-elements/loader';
	import { Button } from '#lib/components/ui/button';
	import * as Select from '#lib/components/ui/select';
	import ToolCall from './ToolCall.svelte';
	import { isToolPart } from './tool-part';

	interface ModelOption {
		id: string;
		label: string;
	}

	interface Props {
		chatId: string;
		messages: UIMessage[];
		placeholder?: string;
		/** true เมื่ออยู่บน /chat/new — จะเปลี่ยน URL เป็น /chat/<id> หลังส่งข้อความแรก */
		adoptUrl?: boolean;
		models?: ModelOption[];
		/** โมเดลที่เลือกอยู่ — 'default' = ใช้ค่าเริ่มต้นของระบบ */
		modelId?: string;
		/** true เมื่อแชทมีแถวใน DB แล้ว — เปลี่ยนโมเดลจะ PATCH ไปเก็บทันที */
		persistModel?: boolean;
	}

	let {
		chatId,
		messages: initialMessages,
		placeholder = 'ถามอะไรก็ได้…',
		adoptUrl = false,
		models = [],
		modelId = 'default',
		persistModel = false
	}: Props = $props();

	const SUGGESTIONS = [
		{ label: 'สรุปข้อความนี้', icon: FileText },
		{ label: 'เขียนโค้ด Rust', icon: Code },
		{ label: 'อธิบายแนวคิดนี้', icon: BookOpen },
		{ label: 'ร่างอีเมล', icon: Mail }
	];

	// สร้างครั้งเดียวต่อ chat — หน้าห่อด้วย {#key} เพื่อรีเซ็ตเมื่อเปลี่ยน chat
	const chat = createChat();

	function createChat() {
		return new Chat({ id: chatId, messages: initialMessages });
	}

	let input = $state('');
	let scroller = $state<HTMLDivElement | null>(null);
	let composer = $state<HTMLTextAreaElement | null>(null);
	let selectedModel = $state(untrack(() => modelId));
	let copiedId = $state<string | null>(null);

	const isBusy = $derived(chat.status === 'submitted' || chat.status === 'streaming');
	const isEmpty = $derived(chat.messages.length === 0);
	const lastMessageId = $derived(chat.messages.at(-1)?.id ?? null);
	const lastAssistantId = $derived(
		chat.messages.filter((message) => message.role === 'assistant').at(-1)?.id ?? null
	);

	const selectedModelLabel = $derived(
		selectedModel === 'default'
			? 'โมเดลเริ่มต้น'
			: (models.find((item) => item.id === selectedModel)?.label ?? selectedModel)
	);

	$effect(() => {
		void chat.messages.length;
		void chat.status;
		if (scroller) scroller.scrollTop = scroller.scrollHeight;
	});

	function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		const text = input.trim();
		if (!text || isBusy) return;

		// แชทใหม่: ยังไม่สร้างแถวใน DB — แถวจะถูกสร้างตอนยิง /api/chat ครั้งแรก
		// เปลี่ยน URL ตรงนี้เลยโดยไม่ remount เพื่อให้ stream ไม่ขาด
		if (adoptUrl) replaceState(`/chat/${chatId}`, {});

		const modelToSend = selectedModel === 'default' ? undefined : selectedModel;
		chat.sendMessage({ text }, modelToSend ? { body: { model: modelToSend } } : undefined);
		input = '';
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			(event.currentTarget as HTMLTextAreaElement).form?.requestSubmit();
		}
	}

	function useSuggestion(text: string) {
		input = text;
		composer?.focus();
	}

	async function handleModelChange(next: string) {
		selectedModel = next;
		if (!persistModel) return;

		try {
			await fetch(`/api/chats/${chatId}`, {
				method: 'PATCH',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ modelId: next === 'default' ? null : next })
			});
		} catch {
			// เก็บค่าที่เลือกไว้ในหน้าจอ — การส่งครั้งถัดไปยังใช้ค่าเดิมจาก DB
		}
	}

	function textOf(message: UIMessage): string {
		return message.parts
			.filter((part) => part.type === 'text')
			.map((part) => (part.type === 'text' ? part.text : ''))
			.join('\n\n');
	}

	async function copyMessage(message: UIMessage) {
		try {
			await navigator.clipboard.writeText(textOf(message));
			copiedId = message.id;
			setTimeout(() => {
				if (copiedId === message.id) copiedId = null;
			}, 1500);
		} catch {
			// clipboard อาจถูก block — ไม่ต้องแจ้ง error ให้รบกวน
		}
	}
</script>

{#snippet composerForm()}
	<form onsubmit={handleSubmit} class="w-full">
		<div
			class="flex w-full items-end gap-1 rounded-3xl border border-border bg-card py-1.5 pr-1.5 pl-3 transition-colors focus-within:border-ring/60"
		>
			<textarea
				bind:this={composer}
				bind:value={input}
				{placeholder}
				rows={1}
				disabled={isBusy}
				onkeydown={handleKeydown}
				class="field-sizing-content max-h-40 min-h-9 w-full flex-1 resize-none bg-transparent py-2 pr-1 text-sm outline-none placeholder:text-muted-foreground disabled:opacity-50"
			></textarea>

			{#if models.length > 0}
				<Select.Root type="single" value={selectedModel} onValueChange={handleModelChange}>
					<Select.Trigger
						class="mb-0.5 h-8 max-w-32 shrink-0 border-transparent bg-transparent px-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground dark:bg-transparent dark:hover:bg-muted"
						aria-label="เลือกโมเดล"
					>
						{selectedModelLabel}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="default">โมเดลเริ่มต้น</Select.Item>
						{#each models as item (item.id)}
							<Select.Item value={item.id}>{item.label}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			{/if}

			<div class="mb-0.5 flex shrink-0 items-center">
				{#if isBusy}
					<Button
						type="button"
						variant="outline"
						size="icon"
						class="rounded-full"
						onclick={() => chat.stop()}
						aria-label="หยุดสร้างคำตอบ"
						title="หยุด"
					>
						<Square />
					</Button>
				{:else}
					<Button
						type="submit"
						size="icon"
						class="rounded-full"
						disabled={!input.trim()}
						aria-label="ส่งข้อความ"
						title="ส่ง"
					>
						<ArrowUp />
					</Button>
				{/if}
			</div>
		</div>
	</form>
{/snippet}

<div class="flex h-full min-h-0 flex-col">
	<div bind:this={scroller} class="min-h-0 flex-1 overflow-y-auto">
		{#if isEmpty}
			<div
				class="mx-auto flex h-full w-full max-w-3xl flex-col items-center justify-center gap-4 px-4 pb-12 text-center"
			>
				<div class="flex flex-col gap-1.5">
					<h2 class="text-2xl font-semibold tracking-tight sm:text-3xl">จะเริ่มจากอะไรดี?</h2>
					<p class="text-sm text-muted-foreground">
						ถามอะไรก็ได้ — หรือเลือกตัวอย่างด้านล่างเพื่อเริ่มเร็วขึ้น
					</p>
				</div>

				<div class="w-full max-w-xl">{@render composerForm()}</div>

				<div class="flex flex-wrap justify-center gap-2">
					{#each SUGGESTIONS as suggestion (suggestion.label)}
						<button
							type="button"
							onclick={() => useSuggestion(suggestion.label)}
							class="inline-flex items-center gap-1.5 rounded-full border border-border px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/30 focus-visible:outline-none"
						>
							<suggestion.icon size={13} />
							{suggestion.label}
						</button>
					{/each}
				</div>
			</div>
		{:else}
			<div class="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 pt-6 pb-10">
				{#each chat.messages as message (message.id)}
					{@const streaming = chat.status === 'streaming' && message.id === lastMessageId}
					<div class={message.role === 'user' ? 'flex justify-end' : 'flex justify-start'}>
						<div
							class={message.role === 'user'
								? 'max-w-[85%] rounded-2xl bg-secondary px-4 py-2.5 text-sm text-secondary-foreground'
								: 'w-full text-sm text-foreground'}
						>
							{#each message.parts as part, index (index)}
								{#if part.type === 'text'}
									{#if message.role === 'user'}
										<p class="whitespace-pre-wrap">{part.text}</p>
									{:else}
										<Response content={part.text} />
									{/if}
								{:else if part.type === 'reasoning'}
									<Reasoning.Root isStreaming={streaming} defaultOpen={streaming}>
										<Reasoning.Trigger />
										<Reasoning.Content content={part.text} />
									</Reasoning.Root>
								{:else if isToolPart(part)}
									<ToolCall {part} />
								{/if}
							{/each}

							{#if message.role === 'assistant' && !streaming && textOf(message)}
								<div class="mt-1 flex items-center gap-1">
									<button
										type="button"
										class="rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/30"
										onclick={() => copyMessage(message)}
										aria-label={copiedId === message.id ? 'คัดลอกแล้ว' : 'คัดลอกข้อความ'}
										title="คัดลอก"
									>
										{#if copiedId === message.id}
											<Check size={13} />
										{:else}
											<Copy size={13} />
										{/if}
									</button>

									{#if message.id === lastAssistantId}
										<button
											type="button"
											class="rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/30 disabled:opacity-50"
											onclick={() => chat.regenerate({ messageId: message.id })}
											disabled={isBusy}
											aria-label="สร้างคำตอบใหม่"
											title="สร้างคำตอบใหม่"
										>
											<RefreshCw size={13} />
										</button>
									{/if}
								</div>
							{/if}
						</div>
					</div>
				{/each}

				{#if isBusy}
					<div class="flex items-center gap-2 pl-1 text-xs text-muted-foreground" role="status">
						<Loader size={14} />
						กำลังคิด…
					</div>
				{/if}

				{#if chat.error}
					<p class="rounded-lg bg-destructive/10 px-3 py-2 text-sm text-destructive">
						{chat.error.message}
					</p>
				{/if}
			</div>
		{/if}
	</div>

	{#if !isEmpty}
		<div class="px-4 pt-2 pb-3">
			<div class="mx-auto w-full max-w-3xl">{@render composerForm()}</div>
		</div>
	{/if}
</div>
