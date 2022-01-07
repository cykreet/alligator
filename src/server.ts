import { DISCORD_WEBHOOK_ENDPOINT, DISCORD_WEBHOOK_MESSAGE_EMBED_LIMIT, EXECUTION_TIMEOUT_MS } from "./constants.ts";
import { serve } from "https://deno.land/std@0.119.0/http/server.ts";
import { Status } from "https://deno.land/std@0.119.0/http/mod.ts";
import { validateRequestPath } from "./helpers/validate-path.ts";
import { mergeRequestBody } from "./helpers/merge-request-body.ts";
import { RequestBatch, Payload } from "./types.ts";

const webhookMessageMap = new Map<string, RequestBatch>();
const timeoutMap = new Map<string, number>();

async function handleRequest(request: Request): Promise<Response> {
	const validatedRequest = validateRequestPath(request.url);
	if (!validatedRequest.valid) {
		console.error(validatedRequest.message);
		const errorMessage = { error: "Not Found", code: 0 };
		return new Response(JSON.stringify(errorMessage), { status: Status.NotFound });
	}

	// hacky as shit, but i haven't seen a "proper" way to do this.
	// using a promise allows us to reply to all requests in the batch
	// at the same time by resolving the returned promise when needed
	let reply: (response: Response) => void = () => {};
	const responsePromise = new Promise<Response>((resolve, _reject) => {
		reply = resolve;
	});

	const webhookId = validatedRequest.webhookId!;
	const webhookToken = validatedRequest.webhookToken!;
	const batchId = `${webhookId}-${webhookToken}`;
	let webhookMessageBatch = webhookMessageMap.get(batchId);
	const requestBody: Payload = await request.json();
	if (webhookMessageBatch == null) {
		webhookMessageBatch = {
			payloads: [requestBody],
			created: new Date(),
			webhookToken,
			webhookId,
			batchId,
			reply,
		};

		webhookMessageMap.set(batchId, webhookMessageBatch);
	} else {
		webhookMessageBatch.payloads.push(requestBody);
	}

	const batchEmbeds = webhookMessageBatch.payloads.map((payload) => payload.embeds);
	if (batchEmbeds.length >= DISCORD_WEBHOOK_MESSAGE_EMBED_LIMIT) {
		webhookMessageMap.delete(batchId);
		removeTimeout(batchId);
		// responds to requests in the current batch, so the request is then added
		// to a new batch with handleRequest()
		executeBatch(webhookMessageBatch);
		return await handleRequest(request);
	}

	if (timeoutMap.has(batchId)) removeTimeout(batchId);
	const executeTimeout = setTimeout(async () => {
		webhookMessageMap.delete(batchId);
		await executeBatch(webhookMessageBatch!);
	}, EXECUTION_TIMEOUT_MS);

	timeoutMap.set(batchId, executeTimeout);
	return await responsePromise;
}

function removeTimeout(batchId: string) {
	const timeoutId = timeoutMap.get(batchId);
	clearTimeout(timeoutId);
	timeoutMap.delete(batchId);
}

async function executeBatch(batch: RequestBatch): Promise<void> {
	const requestBody = mergeRequestBody(batch.payloads);
	const requestOptions: RequestInit = {
		method: "POST",
		body: JSON.stringify(requestBody),
		headers: {
			"Content-Type": "application/json",
		},
	};

	const endpoint = `${DISCORD_WEBHOOK_ENDPOINT}/${batch.webhookId}/${batch.webhookToken}`;
	const discordResponse = await fetch(endpoint, requestOptions);
	const headers = new Headers(discordResponse.headers);
	headers.append("X-Batch-Id", batch.batchId);
	headers.append("X-Batch-Size", batch.payloads.length.toString());
	headers.append("X-Batch-Created", batch.created.toISOString());

	const response = new Response(discordResponse.body, {
		status: discordResponse.status,
		statusText: discordResponse.statusText,
		headers: headers,
	});

	batch.reply(response);
}

serve(handleRequest, { port: 8080 });
