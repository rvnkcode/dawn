FROM node:18

RUN npm i -g @nestjs/cli

WORKDIR /app
COPY ["package.json", "yarn.lock", "./"]
RUN yarn install
COPY . .
RUN mkdir /memo && chown node /memo
RUN yarn prisma db push && yarn prisma generate
RUN yarn c:api
EXPOSE 3000
RUN yarn build
CMD [ "node", "dist/main" ]