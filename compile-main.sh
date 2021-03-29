cargo run -- main.ts > main.s
arm-linux-gnueabihf-gcc -static main.s -o main
