CFLAGS  = -O3 -march=native -ggdb3 -m32 -std=gnu99 -fshort-wchar -Wno-multichar -Iloadlibrary/include -mstackrealign
CPPFLAGS=-DNDEBUG -D_GNU_SOURCE -I. -Iloadlibrary/intercept -Iloadlibrary/peloader
LDFLAGS = $(CFLAGS) -m32 -lm
LDLIBS  = loadlibrary/intercept/libdisasm.a -Wl,--whole-archive,loadlibrary/peloader/libpeloader.a,--no-whole-archive

.PHONY: clean peloader intercept

TARGETS=loader | peloader

all: $(TARGETS)
	-mkdir -p faketemp

intercept:
	make -C loadlibrary/intercept all

peloader:
	make -C loadlibrary/peloader all

intercept/hook.o: intercept

loader: loader.o loadlibrary/intercept/hook.o | peloader
	$(CC) $(CFLAGS) $^ -o $@ $(LDLIBS) $(LDFLAGS)

clean:
	rm -f a.out core *.o core.* vgcore.* gmon.out loader
	make -C loadlibrary/intercept clean
	make -C loadlibrary/peloader clean
	rm -rf faketemp
