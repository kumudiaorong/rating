```c
mem_init();
mm_init();
void *p1 = mm_malloc(16);
void *p2 = mm_malloc(16);
mm_free(p1);
```