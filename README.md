# Geektime Rust 语言训练营

# 作业一
- --nonce指定随机数文件，大小为12字节，96位
- rcli text encrypt --key fixtures/blake3.txt --nonce fixtures/nonce.txt
- rcli text decrypt --key fixtures/blake3.txt --nonce fixtures/nonce.txt

# 作业二

## 生成
- rcli -- jwt sign --sub wwy --aud device1 --exp 14d
- 或
- rcli -- jwt sign --sub wwy --aud device1 --exp 1723107846
## 验证
- rcli -- jwt verify -t eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ3d3kiLCJhdWQiOiJkZXZpY2UxIiwiZXhwIjoxNzIzMTA3ODQ2fQ.LtRJH5ZrYIYScT0-g1Nw4Z32BYj4XNE9xkvTIQy3Mvs

# 作业三
- rcli -- http serve
访问test.rest
