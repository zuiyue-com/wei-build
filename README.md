# sign.exe

- Y1

# linux静态编译

- rustup target add x86_64-unknown-linux-musl
- wget https://www.openssl.org/source/openssl-3.1.5.tar.gz
- tar zxvf openssl-3.1.5.tar.gz && cd openssl-3.1.5.tar.gz
- ./config no-shared --prefix=/usr/local/musl/
- make depend && make && sudo make install
- export OPENSSL_DIR=/usr/local/musl/
- cargo build --release --target=x86_64-unknown-linux-musl