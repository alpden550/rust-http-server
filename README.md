# HTTP Server

Simple http server build on Rust std library.

[HTTP](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol) is the
protocol that powers the web. In this challenge, you'll build a HTTP/1.1 server
that is capable of serving multiple clients.

Along the way you'll learn about TCP servers,
[HTTP request syntax](https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html),
and more.

## Run

1. Ensure you have min `cargo (1.70)` installed locally
2. Run `./server.sh --directory /tmp/` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run it. Subsequent runs will be fast.
3. Open terminal app and send curl requests:
```sh
curl -i http://localhost:4221
curl -v http://localhost:4221/not-found
curl -i http://localhost:4221/echo/abc
curl -i --header "User-Agent: foobar/1.2.3" http://localhost:4221/user-agent

echo -n 'Hello, World!' > /tmp/foo
curl -i http://localhost:4221/files/foo

curl -v --data "12345" -H "Content-Type: application/octet-stream" http://localhost:4221/files/file_123

curl -v -H "Accept-Encoding: gzip" http://localhost:4221/echo/abc | hexdump -C
```
