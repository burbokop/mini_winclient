# Code copied from
# https://gist.github.com/tcoppex/443d1dd45f873d96260195d6431b0989

    .text
    .globl __mini_wc_syscall1__
    .globl __mini_wc_syscall2__
    .globl __mini_wc_syscall3__
    .globl __mini_wc_syscall4__
    .globl __mini_wc_syscall5__
    .globl __mini_wc_exit__

__mini_wc_syscall1__:
    mov rax, rdi
    mov rdi, rsi
    syscall
    ret

__mini_wc_syscall2__:
    mov rax, rdi
    mov rdi, rsi
    mov rsi, rdx
    syscall
    ret

__mini_wc_syscall3__:
    mov rax, rdi
    mov rdi, rsi
    mov rsi, rdx
    mov rdx, rcx
    syscall
    ret

__mini_wc_syscall4__:
    mov rax, rdi
    mov rdi, rsi
    mov rsi, rdx
    mov rdx, rcx
    mov r10, r8
    syscall
    ret

__mini_wc_syscall5__:
    mov rax, rdi
    mov rdi, rsi
    mov rsi, rdx
    mov rdx, rcx
    mov r10, r8
    mov r8, r9
    syscall
    ret

__mini_wc_exit__:
    mov rbx, rdi
    mov rax, 1
    int 0x80
    ret
