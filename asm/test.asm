.386
.model flat, stdcall
option casemap :none

include C:/masm32/include/kernel32.inc
include C:/masm32/include/masm32.inc
includelib C:/masm32/lib/kernel32.lib
includelib C:/masm32/lib/masm32.lib

.data
    message db "This is your first assembly program", 13, 10, 0
    Num1    equ     40h

.code

foo:
    add edx, edx
    lea eax, message
    invoke StdOut, addr message
    ret

main:
    call foo
    invoke ExitProcess, 0
end main