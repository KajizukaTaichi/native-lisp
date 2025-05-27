cargo run > example.asm
nasm -f macho64 example.asm
clang -o example example.o -e _start
./example
echo $?
