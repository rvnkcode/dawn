# Dawn

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
