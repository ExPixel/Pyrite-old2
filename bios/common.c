#include <stddef.h>

/* Add the compiler optimization to inhibit loop transformation to library
   calls.  This is used to avoid recursive calls in memset and memmove
   default implementations.  */
#ifdef HAVE_CC_INHIBIT_LOOP_TO_LIBCALL
#define inhibit_loop_to_libcall \
    __attribute__((__optimize__("-fno-tree-loop-distribute-patterns")))
#else
#define inhibit_loop_to_libcall
#endif

inhibit_loop_to_libcall void *memset(void *dest, int val, size_t len)
{
    unsigned char *ptr = dest;
    while (len-- > 0)
        *ptr++ = val;
    return dest;
}

inhibit_loop_to_libcall void *memcpy(void *dest, const void *src, size_t len)
{
    char *d = dest;
    const char *s = src;
    while (len--)
        *d++ = *s++;
    return dest;
}

inhibit_loop_to_libcall void *memmove(void *dest, const void *src, size_t len)
{
    char *d = dest;
    const char *s = src;
    if (d < s)
        while (len--)
            *d++ = *s++;
    else
    {
        char *lasts = (char *)s + (len - 1);
        char *lastd = d + (len - 1);
        while (len--)
            *lastd-- = *lasts--;
    }
    return dest;
}