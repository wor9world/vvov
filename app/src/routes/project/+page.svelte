<script lang="ts">
	import { Folder, Plus, Sparkles, Trash2 } from '@lucide/svelte/icons';
	import { Button, buttonVariants } from '#lib/components/ui/button';
	import * as Card from '#lib/components/ui/card';
	import * as Empty from '#lib/components/ui/empty';
	import * as Field from '#lib/components/ui/field';
	import { Input } from '#lib/components/ui/input';
	import { Textarea } from '#lib/components/ui/textarea';

	let { data } = $props();

	let name = $state('');
	let description = $state('');
	let instructions = $state('');

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
				body: JSON.stringify({ kind: 'project', prompt })
			});
			const result = await response.json();

			if (!response.ok) throw new Error(result.error ?? response.statusText);

			name = result.name ?? '';
			description = result.description ?? '';
			instructions = result.instructions ?? '';
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
			<h1 class="text-xl font-semibold tracking-tight">Projects</h1>
			<p class="text-sm text-muted-foreground">
				{data.projects.length} project — กลุ่มแชทที่ใช้ instructions ร่วมกัน
			</p>
		</div>

		<details class="relative">
			<summary class={buttonVariants({ size: 'sm' })}>
				<Plus data-icon="inline-start" />
				สร้าง project
			</summary>

			<Card.Root class="absolute right-0 z-10 mt-2 w-96 shadow-lg">
				<Card.Content class="pt-4">
					<form method="POST" action="?/create">
						<Field.Group>
							<Field.Field>
								<Field.Label for="project-draft">ให้ AI ช่วยร่าง</Field.Label>
								<Textarea
									id="project-draft"
									bind:value={draftPrompt}
									rows={2}
									placeholder="เช่น project ดูแลเว็บ vvov: frontend SvelteKit + Rust engine"
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
								<Field.Label for="project-name">ชื่อ</Field.Label>
								<Input id="project-name" name="name" bind:value={name} required placeholder="เช่น vvov core" />
							</Field.Field>

							<Field.Field>
								<Field.Label for="project-description">คำอธิบาย</Field.Label>
								<Textarea
									id="project-description"
									name="description"
									bind:value={description}
									rows={2}
									placeholder="project นี้ทำอะไร"
								/>
							</Field.Field>

							<Field.Field>
								<Field.Label for="project-instructions">Instructions</Field.Label>
								<Textarea
									id="project-instructions"
									name="instructions"
									bind:value={instructions}
									rows={5}
									placeholder="context ที่จะถูกส่งให้โมเดลทุกแชทใน project นี้"
								/>
							</Field.Field>

							<button type="submit" class={buttonVariants()}>สร้าง project</button>
						</Field.Group>
					</form>
				</Card.Content>
			</Card.Root>
		</details>
	</header>

	{#if data.projects.length === 0}
		<Empty.Root class="mt-16">
			<Empty.Header>
				<Empty.Title>ยังไม่มี project</Empty.Title>
				<Empty.Description>สร้าง project เพื่อรวมแชทที่ใช้ context เดียวกัน</Empty.Description>
			</Empty.Header>
		</Empty.Root>
	{:else}
		<div class="grid gap-3 sm:grid-cols-2">
			{#each data.projects as project (project.id)}
				<Card.Root>
					<Card.Content class="flex items-start gap-3 pt-4">
						<div class="flex size-10 shrink-0 items-center justify-center rounded-full bg-muted">
							<Folder class="text-muted-foreground" />
						</div>

						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<a href="/project/{project.id}" class="truncate font-medium hover:underline">{project.name}</a>
							<p class="line-clamp-2 text-xs text-muted-foreground">
								{project.description || 'ไม่มีคำอธิบาย'}
							</p>
							<p class="text-xs text-muted-foreground">{project.chat_count} แชท</p>
						</div>

						<form method="POST" action="?/remove">
							<input type="hidden" name="id" value={project.id} />
							<button
								type="submit"
								class={buttonVariants({ variant: 'ghost', size: 'icon' })}
								aria-label="ลบ project"
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
