#include <stddef.h>

void *memset(void *dst, int value, unsigned int count)
{
    unsigned char *ptr = (unsigned char *)dst;
    while (count-- > 0)
    {
        *ptr = (unsigned char)value;
    }
    return dst;
}