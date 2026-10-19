FROM node:24-alpine AS build
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-workspace.yaml ./
COPY server/package.json server/package.json
RUN pnpm install --filter @arkham/server... --no-frozen-lockfile
COPY tsconfig.base.json ./
COPY server ./server
RUN pnpm --filter @arkham/server build

FROM node:24-alpine AS runtime
WORKDIR /app
ENV NODE_ENV=production
ENV PORT=8787
RUN corepack enable
COPY server/package.json ./package.json
RUN pnpm install --prod --no-frozen-lockfile
COPY --from=build /app/server/dist ./dist
EXPOSE 8787
USER node
CMD ["node", "dist/index.js"]
