FROM node:18

WORKDIR /dawn
# TODO:
# COPY ["package.json", "yarn.lock", "./"]
# COPY apps/api/package.json /app/api/
# COPY apps/client/package.json /app/client/

# RUN echo 
COPY . .

RUN yarn install
RUN mkdir /memo && chown node /memo
RUN yarn db
RUN yarn c:api
EXPOSE 3000
RUN yarn build
CMD [ "node", "apps/api/dist/main" ]