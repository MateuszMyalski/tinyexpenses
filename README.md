## Run locally




## Run using docker file
Build:

```bash
docker build -t tinyexpenses .
```

Run:
```bash
docker run -v ./accounts:/app/accounts -v ./logs:/app/logs -e SECRET_KEY=<32-char secret key> -p 8080:8080 -u 1000:1000 -d tinyexpenses:latest
```

## Run using docker compose
```bash


```