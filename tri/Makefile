main.o: main.c
	gcc -c main.c -o main.o

tri.o: tri.s
	nasm -f elf64 -o tri.o tri.s

tri: main.o tri.o
	gcc main.o tri.o -o tri
