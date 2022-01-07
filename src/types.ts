import { RESTPostAPIWebhookWithTokenJSONBody } from "https://deno.land/x/discord_api_types/v9.ts";

export interface ValidatedRequest {
	valid: boolean;
	message?: string;
	webhookId?: string;
	webhookToken?: string;
}

export type Payload = RESTPostAPIWebhookWithTokenJSONBody;

export interface RequestBatch {
	reply: (response: Response) => void;
	batchId: string;
	payloads: Payload[];
	webhookToken: string;
	webhookId: string;
	created: Date;
}
