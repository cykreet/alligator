import { DISCORD_WEBHOOK_MESSAGE_EMBED_LIMIT, EXECUTION_TIMEOUT_MS, LISTEN_PORT } from "./constants.ts";
import { serve, HttpStatus, WebhookPayload } from "../deps.ts";
import { validateRequestPath } from "./helpers/validate-path.ts";
import { deliverBatch } from "./helpers/deliver-batch.ts";
import { RequestBatch } from "./types.ts";

const webhookMessageMap = new Map<string, RequestBatch>();
const timeoutMap = new Map<string, number>();

async function handleRequest(request: Request): Promise<Response> {
	const validatedRequest = validateRequestPath(request.url);
	if (!request.bodyUsed || !validatedRequest.valid) {
		console.error(validatedRequest.message);
		const errorMessage = { error: "Not Found", code: 0 };
		return new Response(JSON.stringify(errorMessage), { status: HttpStatus.NotFound });
	}

	// hacky as shit, but i haven't seen a "proper" way to do this.
	// using a promise allows us to reply to all requests in the batch
	// at the same time by resolving the returned promise when needed.
	let reply: (response: Response) => void = () => {};
	const responsePromise = new Promise<Response>((resolve, _reject) => {
		reply = resolve;
	});

	const webhookId = validatedRequest.webhookId!;
	const webhookToken = validatedRequest.webhookToken!;
	const batchId = `${webhookId}-${webhookToken}`;
	let webhookMessageBatch = webhookMessageMap.get(batchId);
	const requestBody: WebhookPayload = await request.json();
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
		deliverBatch(webhookMessageBatch);
		return await handleRequest(request);
	}

	if (timeoutMap.has(batchId)) removeTimeout(batchId);
	const executeTimeout = setTimeout(async () => {
		webhookMessageMap.delete(batchId);
		await deliverBatch(webhookMessageBatch!);
	}, EXECUTION_TIMEOUT_MS);

	timeoutMap.set(batchId, executeTimeout);
	return await responsePromise;
}

function removeTimeout(batchId: string) {
	const timeoutId = timeoutMap.get(batchId);
	clearTimeout(timeoutId);
	timeoutMap.delete(batchId);
}

serve(handleRequest, { port: LISTEN_PORT });
