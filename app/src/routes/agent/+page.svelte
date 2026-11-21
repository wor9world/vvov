<script lang="ts">
	import { Plus, Sparkles, Trash2 } from '@lucide/svelte/icons';
	import { Button, buttonVariants } from '#lib/components/ui/button';
	import * as Card from '#lib/components/ui/card';
	import * as Empty from '#lib/components/ui/empty';
	import * as Field from '#lib/components/ui/field';
	import { Input } from '#lib/components/ui/input';
	import * as Select from '#lib/components/ui/select';
	import { Textarea } from '#lib/components/ui/textarea';

	let { data } = $props();

	let name = $state('');
	let avatar = $state('');
	let description = $state('');
	let systemPrompt = $state('');
	let model = $state('default');

	let draftPrompt = $state('');
	let drafting = $state(false);
	let draftError = $state('');

	async function draftFromPrompt() {
		const prompt = draftPrompt.trim();
		if (!prompt || drafting) return;

		drafting = true;
		draftError = '';

		try {
			const response = await fetch('/api/draft', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ kind: 'agent', prompt })
			});
			const result = await response.json();

			if (!response.ok) throw new Error(result.error ?? response.statusText);

			name = result.name ?? '';
			avatar = result.avatar ?? '';
			description = result.description ?? '';
			systemPrompt = result.systemPrompt ?? '';
		} catch (cause) {
			draftError = cause instanceof Error ? cause.message : String(cause);
		} finally {
			drafting = false;
		}
	}
</script>

<div class="mx-auto flex h-full max-w-4xl flex-col gap-6 overflow-y-auto p-6">
	<header class="flex items-start justify-between gap-4">
		<div class="flex flex-col gap-1">
			<h1 class="text-xl font-semibold tracking-tight">Agents</h1>
			<p class="text-sm text-muted-foreground">
				{data.agents.length} persona — expert ที่มีบุคลิกและ system prompt ของตัวเอง
			</p>
		</div>

		<details class="relative">
			<summary class={buttonVariants({ size: 'sm' })}>
				<Plus data-icon="inline-start" />
				สร้าง agent
			</summary>

			<Card.Root class="absolute right-0 z-10 mt-2 w-96 shadow-lg">
				<Card.Content class="pt-4">
					<form method="POST" action="?/create">
						<Field.Group>
							<Field.Field>
								<Field.Label for="agent-draft">ให้ AI ช่วยร่าง</Field.Label>
								<Textarea
									id="agent-draft"
									bind:value={draftPrompt}
									rows={2}
									placeholder="เช่น ผู้เชี่ยวชาญ Rust สอนมือใหม่ ตอบเป็นกันเอง"
								/>
								<Button
									type="button"
									variant="secondary"
									size="sm"
									disabled={drafting || !draftPrompt.trim()}
									onclick={draftFromPrompt}
								>
									<Sparkles data-icon="inline-start" />
									{drafting ? 'กำลังร่าง…' : 'ร่างให้หน่อย'}
								</Button>
								{#if draftError}
									<p class="text-xs text-destructive">{draftError}</p>
								{/if}
							</Field.Field>

							<Field.Field>
								<Field.Label for="agent-name">ชื่อ</Field.Label>
								<Input id="agent-name" name="name" bind:value={name} required placeholder="เช่น Rust Expert" />
							</Field.Field>

							<Field.Field>
								<Field.Label for="agent-avatar">Emoji</Field.Label>
								<Input id="agent-avatar" name="avatar" bind:value={avatar} maxlength={4} placeholder="🦀" />
							</Field.Field>

							<Field.Field>
								<Field.Label for="agent-description">คำอธิบาย</Field.Label>
								<Textarea
									id="agent-description"
									name="description"
									bind:value={description}
									rows={2}
									placeholder="สั้น ๆ ว่าเก่งอะไร"
								/>
							</Field.Field>

							<Field.Field>
								<Field.Label for="agent-prompt">System prompt</Field.Label>
								<Textarea
									id="agent-prompt"
									name="system_prompt"
									bind:value={systemPrompt}
									rows={5}
									placeholder="บุคลิกและกติกาของ agent"
								/>
							</Field.Field>

							<Field.Field>
								<Field.Label>โมเดล</Field.Label>
								<Select.Root type="single" name="model_id" bind:value={model}>
									<Select.Trigger class="w-full">{model === 'default' ? 'ค่าเริ่มต้น' : model}</Select.Trigger>
									<Select.Content>
										<Select.Item value="default">ค่าเริ่มต้นของระบบ</Select.Item>
										{#each data.models as item (item.id)}
											<Select.Item value={item.id}>{item.label}</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							</Field.Field>

							<button type="submit" class={buttonVariants()}>สร้าง agent</button>
						</Field.Group>
					</form>
				</Card.Content>
			</Card.Root>
		</details>
	</header>

	{#if data.agents.length === 0}
		<Empty.Root class="mt-16">
			<Empty.Header>
				<Empty.Title>ยังไม่มี agent</Empty.Title>
				<Empty.Description>สร้าง persona แรกของคุณ เช่น "Rust Expert" หรือ "Math Tutor"</Empty.Description>
			</Empty.Header>
		</Empty.Root>
	{:else}
		<div class="grid gap-3 sm:grid-cols-2">
			{#each data.agents as agent (agent.id)}
				<Card.Root>
					<Card.Content class="flex items-start gap-3 pt-4">
						<div class="flex size-10 shrink-0 items-center justify-center rounded-full bg-muted text-lg">
							{agent.avatar || '🤖'}
						</div>

						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<a href="/agent/{agent.id}" class="truncate font-medium hover:underline">{agent.name}</a>
							<p class="line-clamp-2 text-xs text-muted-foreground">
								{agent.description || 'ไม่มีคำอธิบาย'}
							</p>
							<p class="text-xs text-muted-foreground">
								{agent.chat_count} แชท{agent.model_id ? ` · ${agent.model_id}` : ''}
							</p>
						</div>

						<form method="POST" action="?/remove">
							<input type="hidden" name="id" value={agent.id} />
							<button
								type="submit"
								class={buttonVariants({ variant: 'ghost', size: 'icon' })}
								aria-label="ลบ agent"
							>
								<Trash2 />
							</button>
						</form>
					</Card.Content>
				</Card.Root>
			{/each}
		</div>
	{/if}
</div>
