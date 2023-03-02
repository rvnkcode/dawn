# Dawn

[![Release Docker Image](https://github.com/rvnkcode/dawn/actions/workflows/release-docker-image.yml/badge.svg)](https://github.com/rvnkcode/dawn/actions/workflows/release-docker-image.yml)

## Deploy with Docker

### Docker Run

```bash
docker run -d --name dawn -p 3000:80 -v ~/.dawn/:/memo rvnk/dawn:latest
```

### Docker Compose

Provided `docker-compose.yml` is [here](./docker-compose.yml).

```bash
docker-compose down && docker image rm rvnk/dawn:latest && docker-compose up -d
```
