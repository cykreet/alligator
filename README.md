# alligator

discord webhook proxy, built to replace webhook endpoints in your application. alligator will then merge sequential requests made within a configurable time frame and make a single request to discord.

## error handling

since responses are returned as early as possible, discord api errors are not returned, but logged to std out.

## deploy

wip
