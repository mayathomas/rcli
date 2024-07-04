# Assets

- [juventus.csv](./juventus.csv): dataset from [The-Football-Data](https://github.com/buckthorndev/The-Football-Data).

# 作业一
--nonce指定随机数文件，大小为12字节，96位
- rcli text encrypt --key fixtures/blake3.txt --nonce fixtures/nonce.txt
- rcli text decrypt --key fixtures/blake3.txt --nonce fixtures/nonce.txt
