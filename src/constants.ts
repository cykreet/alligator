export const EXECUTION_TIMEOUT_MS = +(Deno.env.get("EXECUTION_TIMEOUT_MS") ?? 2000);
export const DISCORD_WEBHOOK_MESSAGE_EMBED_LIMIT = Deno.env.get("DISCORD_WEBHOOK_MESSAGE_EMBED_LIMIT") ?? 10;
export const DISCORD_WEBHOOK_ENDPOINT = Deno.env.get("DISCORD_WEBHOOK_ENDPOINT") ?? "https://discord.com/api/webhooks";
export const REQUEST_URL_REGEX =
	/(http?s?:\/\/)?[A-z0-9.:\-_]{1,253}\/api\/(v[0-9]{1,3}\/)?webhooks\/(?<webhook_id>[0-9]\w+)\/(?<webhook_token>[A-z0-9-]{1,100})/;
