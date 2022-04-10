# alligator

discord webhook proxy server - works to replace discord webhook endpoints in your application. alligator will then merge sequential requests within a configurable time frame and make a single request to discord.

if a request batch collectively contains `10` embeds, the batch is delivered immediately. apart from the fairly obvious "mergeable" webhook request properties (like `embeds`), alligator uses the first occurrence of other miscellaneous properties (like `username`) - this currently also includes: `avatar_url`, `allowed_mentions` and `tts`.

## deploy

this repository is set up to automatically push to the [docker registry](https://hub.docker.com/r/cykreet/alligator) with each new version, so that would be the easiest way to get started.

```bash
docker run -d -p 8080:8080 --name alligator cykreet/alligator:latest
```

## config

alligator is configurable using a few different environment variables. here they are, for your convenience
| variable                            | default                          | description                                                                                          |
|-------------------------------------|----------------------------------|------------------------------------------------------------------------------------------------------|
| LISTEN_PORT                         | `8080`                             | the port that alligator should listen to for incoming requests                                       |
| EXECUTION_TIMEOUT_MS                | `2000`                             | how long alligator should wait for subsequent incoming requests before sending                       |
| DISCORD_WEBHOOK_MESSAGE_EMBED_LIMIT | `10`                               | the number of embeds alligator should join in a single message                                       |
| DISCORD_WEBHOOK_ENDPOINT            | `https://discord.com/api/webhooks` | the endpoint that your webhooks should be sent to. in almost all cases you won't need to change this |

## kinda neat

once a batch has been delivered, alligator returns a few handy headers along with discord's response:

| header            | description                                                         |
| ----------------- | ------------------------------------------------------------------- |
| `x-batch-created` | iso string specifying when the first request in the batch was made. |
| `x-batch-id`      | the batch id, identical to the webhook id.                          |
| `x-batch-size`    | the number of requests contained in the request batch.              |
