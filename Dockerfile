FROM node:lts-alpine AS build

WORKDIR /usr/src/dawn
COPY --chown=node:node ["package.json", "yarn.lock", "./"]
COPY apps/api/package.json ./apps/api/
COPY apps/client/package.json ./apps/client/

RUN yarn install --frozen-lockfile

COPY . .

RUN yarn workspace @dawn/api prisma generate

# RUN mkdir /memo && chown node /memo
# RUN yarn workspace @dawn/api prisma db push
RUN yarn c:api
RUN yarn build

# FROM node:lts-alpine AS prod

# WORKDIR /usr/src/dawn
# COPY --from=build /usr/src/dawn/apps/api/dist ./dist
# COPY --from=build /usr/src/dawn/apps/api/client ./client
# COPY --chown=node:node --from=build /memo /memo

EXPOSE 3000

# CMD [ "node", "./dist/main" ]
CMD [ "node", "./apps/api/dist/main" ]