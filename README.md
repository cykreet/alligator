# alligator

discord webhook proxy server - works to replace discord webhook endpoints in your application. alligator will then merge sequential requests within a configurable time frame and make a single request to discord.

if a request batch collectively contains `10` embeds, the batch is delivered immediately. apart from the fairly obvious "mergeable" webhook request properties (like `embeds`), alligator uses other miscellaneous properties (like `username`) from the first request in the batch only - this currently also includes: `avatar_url`, `allowed_mentions` and `tts`.

## deploy

this repository is set up to automatically push to the [docker registry](https://hub.docker.com/r/cykreet/alligator) with each new version, so that would be the easiest way to get started.

```bash
docker run -d -p 8080:8080 --name alligator cykreet/alligator:latest
```

## kinda neat

once a batch has been delivered, alligator returns a few handy headers along with discord's response:

| header            | description                                                         |
| ----------------- | ------------------------------------------------------------------- |
| `x-batch-created` | iso string specifying when the first request in the batch was made. |
| `x-batch-id`      | the batch id, formatted as `webhook_id-webhook_token`.              |
| `x-batch-size`    | the number of requests contained in the request batch.              |
