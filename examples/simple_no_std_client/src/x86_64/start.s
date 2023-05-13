# Code copied from
# https://gist.github.com/tcoppex/443d1dd45f873d96260195d6431b0989

    .text
    .globl _start, __exit__
_start:
    xor rbp,rbp /* xoring a value with itself = 0 */
    pop rdi /* rdi = argc */
    /* the pop instruction already added 8 to rsp */
    mov rsi,rsp /* rest of the stack as an array of char ptr */

    /* zero the las 4 bits of rsp, aligning it to 16 bytes
    same as "and rsp,0xfffffffffffffff0" because negative
    numbers are represented as
    max_unsigned_value + 1 - abs(negative_num) */
    and rsp, -16
    call main
    mov ebx, eax
    mov eax, 1
    int 0x80
    ret

__exit__:
    mov rbx, rdi
    mov rax, 1
    int 0x80
    ret
