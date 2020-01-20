# Rust demos


Some Http server toy example benchmark:

```
Rust Hyper + tokio result:

$ wrk -t8 -c400 -d10s http://127.0.0.1:3000/add
Running 10s test @ http://127.0.0.1:3000/add
  8 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.46ms  675.21us  18.22ms   67.22%
    Req/Sec    13.35k     0.91k   16.77k    74.00%
  1062344 requests in 10.01s, 121.66MB read
  Socket errors: connect 0, read 252, write 0, timeout 0
Requests/sec: 106142.83
Transfer/sec:     12.16MB

Rust Tide + async-std result:

$ wrk -t8 -c400 -d10s http://127.0.0.1:8081/add
Running 10s test @ http://127.0.0.1:8081/add
  8 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     4.25ms    0.99ms  25.56ms   64.66%
    Req/Sec    10.85k     1.11k   14.98k    73.88%
  863533 requests in 10.01s, 140.66MB read
  Socket errors: connect 0, read 254, write 0, timeout 0
Requests/sec:  86286.49
Transfer/sec:     14.05MB

Java Apollo result:

$ wrk -t8 -c400 -d10s http://127.0.0.1:8080/add
Running 10s test @ http://127.0.0.1:8080/add
  8 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    15.49ms   31.40ms 434.46ms   91.31%
    Req/Sec     4.86k     2.82k   10.95k    55.37%
  382237 requests in 10.10s, 71.66MB read
  Socket errors: connect 0, read 597, write 0, timeout 0
Requests/sec:  37851.24
Transfer/sec:      7.10MB

```
