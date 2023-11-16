# Axum

```shell
cargo run --package examples-axum
```

## Anonymous Requests

```shell
curl --request GET 'http://localhost:3000/users'
```

```shell
curl --request GET 'http://localhost:3000/users/00000000-0000-0000-0000-000000000000'
```

## Authorized Requests

```shell
curl --request GET 'http://localhost:3000/users' \
  --header 'Authorization: Example local:user:00000000-0000-0000-0000-000000000000'
```

```shell
curl --request DELETE 'http://localhost:3000/users/00000000-0000-0000-0000-000000000000' \
  --header 'Authorization: Example local:user:00000000-0000-0000-0000-000000000000'
```

```shell
curl --request GET 'http://localhost:3000/users/00000000-0000-0000-0000-000000000000' \
  --header 'Authorization: Example local:user:00000000-0000-0000-0000-000000000000'
```

```shell
curl --request PUT 'http://localhost:3000/users/00000000-0000-0000-0000-000000000000' \
  --header 'Authorization: Example local:user:00000000-0000-0000-0000-000000000000'
```

## Unauthorized Requests

Anonymous user:

```shell
curl --request PUT 'http://localhost:3000/users/00000000-0000-0000-0000-000000000000'
```

Non admin user:

```shell
curl --request PUT 'http://localhost:3000/users/00000000-0000-0000-0000-000000000000' \
--header 'Authorization: Example local:user:00000000-0000-0000-0000-000000000001'
```

Admin user using unauthorized host:

```shell
curl --request PUT 'http://127.0.0.1:3000/users/00000000-0000-0000-0000-000000000000' \
  --header 'Authorization: Example local:user:00000000-0000-0000-0000-000000000000'
```
