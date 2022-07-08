# alligator

Discord webhook proxy, built to replace webhook endpoints in your application. alligator will then merge sequential requests made within a configurable time frame and make a single request to Discord.

```
https://discord.com/api/webhooks/985948921572499487/KD9kzKaXuLvcc6lfDMQsUd4h8DCyow45JIF2scb8IOdwQsx9lVJXoIoBwghF38lAQP8w
```

to

```
http://127.0.0.1:8080/api/webhooks/985948921572499487/KD9kzKaXuLvcc6lfDMQsUd4h8DCyow45JIF2scb8IOdwQsx9lVJXoIoBwghF38lAQP8w
```

## Deploy

This repository is set up to automatically push to the [Docker registry](https://hub.docker.com/r/cykreet/alligator) with each new version, so that would be the easiest way to get started.

```bash
docker run -d -p 8080:8080 --name alligator cykreet/alligator:latest
```

## Function

Apart from the fairly obvious "mergeable" webhook request properties (like `embeds`), alligator uses the first occurrence of other miscellaneous properties (like `username`) - this currently also includes: `avatar_url`, `allowed_mentions`, `thread_name` and `tts`.

Responses are returned as soon as possible, i.e once the request has been parsed and added to the request batch, so Discord API errors can't be directly reported, but are logged to the std err.

alligator also returns a few potentially useful response headers:

| header            | description                                                                  |
| ----------------- | ---------------------------------------------------------------------------- |
| `x-batch-id`      | The batch id, formatted as `webhook_id-webhook_token`.                       |
| `x-batch-size`    | The number of requests contained in the request batch.                       |
| `x-batch-created` | Milliseconds since UNIX epoch when the first request was added to the batch. |

## Environment Variables

| variable                   | description                                                                                          | default                             |
| -------------------------- | ---------------------------------------------------------------------------------------------------- | ----------------------------------- |
| `LISTEN_PORT`              | The port alligator should bind to.                                                                   | `8080`                              |
| `DELIVER_MS`               | How long alligator should wait for subsequent incoming requests before sending.                      | `7000`                              |
| `DISCORD_WEBHOOK_ENDPOINT` | The endpoint that your webhooks should be sent to. in almost all cases you won't need to change this | `https://discord.com/api/webhooks/` |
