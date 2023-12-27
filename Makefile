CC = gcc
OUT = bin

objs = \
    out/main.o \
    out/print.o \
	out/values.o \
	out/io.o

default: out/runtime.o

out/runtime.o: $(objs)
	ld -r -o out/runtime.o $(objs)

out/main.o: runtime/main.c
	$(CC) -c -o out/main.o runtime/main.c
out/print.o: runtime/print.c
	$(CC) -c -o out/print.o runtime/print.c
out/values.o: runtime/values.c
	$(CC) -c -o out/values.o runtime/values.c
out/io.o: runtime/io.c
	$(CC) -c -o out/io.o runtime/io.c
