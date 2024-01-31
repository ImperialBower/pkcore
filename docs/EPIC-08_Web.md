# Epic 8 - Web

I need a container with a web service on it that allows me to get and cache
preflop results for a given hand. While my code is at the point where I can
calculate the odds in about 8 seconds, it takes several minutes for the 4GB 
`BCM` aka Binary Card Map file to be lazy-loaded into memory on the first call.
This makes it almost useless for simple command line calls. I've been resisting
doing networking for this project, but I need to face my fears and learn `Rust`
web programming. 

There are many ways to architect this out. 

* Swagger/OpenAPI restful web service running on a Docker container
  * [Rust Web Development](https://www.manning.com/books/rust-web-development?ar=false&lpse=B)
  * [Building a Secure WebSocket Server using Rust & Warp with Docker](https://blog.devgenius.io/building-a-secure-websocket-server-with-rust-warp-in-docker-20e842d143af)
* Protocol buffer backed 



