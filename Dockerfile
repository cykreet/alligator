FROM denoland/deno:alpine

WORKDIR /app

USER deno

COPY deps.ts .
RUN deno cache deps.ts

ADD . .
RUN deno cache src/main.ts

CMD ["run", "--allow-env", "--allow-net", "src/main.ts"]