# Web3 game server

There is no readme or any doc comments since it requires alot to write it and there so much to document regarding the base architecture of this repo and its pakcages, so it will be added later. For now the code is not that much large and I will try to explain the workspace and its module here brefily:

This workspace contains:

1. [server](./server/): A rust implementation of high performance multi threaded message broadcaster.
2. [sdk](./sdk/): A kit which can be used in game clients to enable the interaction with the server. All other sdks like c# for unity and cpp for unreal engine can be created from this base rust sdk.
3. [protos](./protos/): These are the raw protobuf files for network data interfacing.
4. [proto-inteface](./proto-interface/): This is a crate which defines a common protocol interface between client and server.
5. [examples](./examples/): All of the examples will be included as a specefic binary in this package. For now it simply hasa broadcast example between two clients.

And rest of the packages can be ignored for now.

The server is still very immature and lacks major features such as relevance management, authority filteration and ownership rules for entities,  but for now it has a strong base for high performance communications based on versions.
No gracefule shutdowns are implemented in both server/sdk sessions.

Web3 service running is still in progress and I am desiging a protocol which can be share between solana and our server, aiming as much less memory and rent allocation as possible.