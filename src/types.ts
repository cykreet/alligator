import { WebhookPayload } from "../deps.ts";

export interface ValidatedRequest {
	valid: boolean;
	message?: string;
	webhookId?: string;
	webhookToken?: string;
}

export interface RequestBatch {
	reply: (response: Response) => void;
	batchId: string;
	payloads: WebhookPayload[];
	webhookToken: string;
	webhookId: string;
	created: Date;
}
