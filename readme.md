# rust-httpfs

> A basic file server written in Rust

Why? It's a fun and interesting thing to build.

Should you use it? Not for anything important.

## Features

- `GET /` -> Return index (html if `Accept: text/html`, plaintext otherwise)
- `GET /:path` -> Return file or directory listing at path or 404
- `POST /:path` -> Create/overwrite file at path
- `Content-Type` & `Content-Disposition`:
  - Automatically computed from file extention, should display image/video/etc just fine in-browser
- Concurrent clients just works™️

  - Try it out with Apache Benchmark `ab -c 50 -n 2000 localhost:8080`

- CLI:

```js
httpfs is a simple file server.
usage: httpfs [-v] [-p PORT] [-d PATH-TO-DIR]
  -v Prints debugging messages.
  -p Specifies the port number that the server will listen and serve at. Default is 8080.
  -d Specifies the directory that the server will use to read/write requested files. Default is the current directory when launching the application.
```
