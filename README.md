# Dawn

[![Docker Pulls](https://img.shields.io/docker/pulls/rvnk/dawn)](https://hub.docker.com/repository/docker/rvnk/dawn/general)  
[![Test Docker Image](https://github.com/rvnkcode/dawn/actions/workflows/test-image.yml/badge.svg)](https://github.com/rvnkcode/dawn/actions/workflows/test-image.yml)
[![Release Docker Image](https://github.com/rvnkcode/dawn/actions/workflows/release-docker-image.yml/badge.svg)](https://github.com/rvnkcode/dawn/actions/workflows/release-docker-image.yml)  
Self-hosted GTD based task management application

## Deploy with Docker

### Docker Run

```bash
docker run -d --name dawn -p 3000:3000 -v ~/.dawn/:/memo rvnk/dawn:latest
```

### Docker Compose

Provided `docker-compose.yml` is [here](./docker-compose.yml).

```bash
docker-compose down && docker image rm rvnk/dawn:latest && docker-compose up -d
```
