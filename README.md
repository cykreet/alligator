# alligator

discord webhook proxy server - works to replace discord webhook endpoints in your application. alligator will then merge concurrent requests within a configurable time frame and make a single request to discord.

if a request batch collectively contains `10` embeds, the batch is delivered immediately. apart from the fairly obvious "mergeable" webhook request properties like `embeds`, alligator uses other miscellaneous properties (like `username`) from the first request in the batch only - this currently also includes: `avatar_url`, `allowed_mentions` and `tts`.

this repository is set up to automatically push to docker hub with each new version, so that would be the easiest way to get started:

```bash
docker run -d -p 8080:8080 --name alligator cykreet/alligator:latest
```
