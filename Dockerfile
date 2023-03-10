FROM node:lts-alpine AS build

WORKDIR /usr/src/dawn
COPY ["package.json", "yarn.lock", "./"]
COPY apps/api/package.json ./apps/api/
COPY apps/client/package.json ./apps/client/

RUN yarn config delete proxy && yarn config delete https-proxy
RUN yarn install --frozen-lockfile --network-timeout 1000000

COPY . .

RUN yarn workspace @dawn/api prisma generate
RUN yarn api

RUN yarn build

FROM node:lts-alpine AS prod

WORKDIR /usr/src/dawn
COPY ["package.json", "yarn.lock", "./"]
COPY apps/api/package.json ./apps/api/
COPY --from=build /usr/src/dawn/apps/api/node_modules ./apps/api/node_modules
COPY --from=build /usr/src/dawn/apps/api/prisma ./apps/api/prisma

COPY --from=build /usr/src/dawn/apps/api/dist ./apps/api/dist
COPY --from=build /usr/src/dawn/apps/api/client ./apps/api/client

RUN yarn install --production=true
RUN yarn cache clean

RUN mkdir /memo && chown node /memo

EXPOSE 3000

CMD [ "yarn", "start" ]