Persistent Storage API for the Automotive Grade Linux demo.


cargo run --release --bin server

Insomnia wie in https://konghq.com/blog/engineering/building-grpc-apis-with-rust beschrieben

OpenDB:
{}

Write:
{
   "key": "testkey",
   "value": "testvalue"
}

Read:
{
    "key": "testkey"
}

Read:
{
    "key": "wrongkey"
}

CloseDB:
{}

DestroyDB:
{}