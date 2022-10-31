# biliroaming-rust-simple

## Build

```
git clone https://github.com/woshiluo/biliroaming-rust-simple
cd biliroaming-rust-simple
cargo build --release
```

## Features

- Only support hk/cn playurl
- Memory cache
- WhiteList

## Config

The program will only read config from `config.json` in current directory.

It should looks like it:
```json
{
	"address": "127.0.0.1",
	"port": 8080,
	"users": [1],
}
```
- `address`: Listen address
- `port`: Listen port
- `users`: Whitelist

## Thanks 

- <https://github.com/yujincheng08/BiliRoaming>
- <https://github.com/pchpub/BiliRoaming-Rust-Server/>
