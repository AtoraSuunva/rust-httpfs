# rust-httpfs

> A basic file server written in Rust

## TODO

- `GET /` -> Return index (Based on `Accept`?)
- [x] `GET /:path` -> Return file at path or 404
  - [x] Only files in data directory (`-d <PATH>` or default `.`)
  - [x] Prevent escape from data directory and accessing other files (`GET /../Cargo.toml`)
- [x] `POST /:path` -> Create file at path (`?overwrite=<bool>` option?)
- Appropriate status codes & Human-readable error messages

  - 200 OK
  - 201 Created
  - 204 No Content (`DELETE`?)
  - 400 Bad Request
  - 404 Not Found
  - 405 Method Not Allowed (Not expected method, trying to `POST /`)
  - 411 Length Required (missing `Content-Length`)
  - 413 Payload Too Large (`Content-Length` too large)
  - 500 Internal Server Error (whoops)
  - 505 HTTP Version not supported (HTTP/1.0, 2, 3)

- CLI:

```js
httpfs is a simple file server.
usage: httpfs [-v] [-p PORT] [-d PATH-TO-DIR]
  -v Prints debugging messages.
  -p Specifies the port number that the server will listen and serve at. Default is 8080.
  -d Specifies the directory that the server will use to read/write requested files. Default is the current directory when launching the application.
```

- Concurrent clients:

  - Lol im using tokio it'll just work
  - `ab -c 200 -n 2000 localhost:8080`
    - Apache Benchmark, 200 concurrent requests, 2000 total requests
  - Although make sure we don't run into concurrency issues (two writers, reader + writer...)
    - RWLock? `parking_lot`?

- [x] `Content-Type` & `Content-Disposition`:

  - Using mime_guess, works on the path
  - Returns the "best guess" for a file extension, or "application/octet-stream" if it can't guess
  - This means our server can make images/html render in browser :D

- Optional:
  - `PUT /:path` -> Create or update file at path
  - `PATCH /:path` -> Update file if exists
  - `DELETE /:path` -> Delete file if exists
  - Cache-control headers? `If-Modified-Since`?
