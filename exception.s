	.file	"exception.c"
	.section .rdata,"dr"
	.align 8
.LC0:
	.ascii "paniced while panicing, ABORT!\0"
	.text
	.p2align 4,,15
	.globl	lang_start_panic_unwind
	.def	lang_start_panic_unwind;	.scl	2;	.type	32;	.endef
	.seh_proc	lang_start_panic_unwind
lang_start_panic_unwind:
	subq	$40, %rsp
	.seh_stackalloc	40
	.seh_endprologue
	leaq	__emutls_v.__is_unwinding(%rip), %rcx
	call	__emutls_get_address
	cmpb	$0, (%rax)
	je	.L2
	leaq	.LC0(%rip), %rcx
	call	printf
	call	abort
.L2:
	leaq	__emutls_v.__target(%rip), %rcx
	call	__emutls_get_address
	movq	(%rax), %rcx
	movl	$2, %edx
	call	longjmp
	nop
	.seh_endproc
	.section .rdata,"dr"
.LC1:
	.ascii "bar\0"
.LC2:
	.ascii "bar panicked unexpectedly!\0"
	.text
	.p2align 4,,15
	.globl	bar
	.def	bar;	.scl	2;	.type	32;	.endef
	.seh_proc	bar
bar:
	subq	$40, %rsp
	.seh_stackalloc	40
	.seh_endprologue
	leaq	.LC1(%rip), %rcx
	call	puts
	leaq	__emutls_v.COUNTER.4572(%rip), %rcx
	call	__emutls_get_address
	movl	(%rax), %ecx
	leal	1(%rcx), %edx
	cmpl	$10, %edx
	movl	%edx, (%rax)
	je	.L6
	addq	$40, %rsp
	ret
.L6:
	leaq	__emutls_v.panic_info(%rip), %rcx
	call	__emutls_get_address
	leaq	.LC2(%rip), %rdx
	movl	$2, (%rax)
	movq	%rdx, 8(%rax)
	call	lang_start_panic_unwind
	nop
	.seh_endproc
	.section .rdata,"dr"
	.align 8
.LC3:
	.ascii "(foo) panic detected: message = %s\12\0"
	.align 8
.LC4:
	.ascii "(foo) panic detected: status = %s\12\0"
	.align 8
.LC5:
	.ascii "(foo) panic detected: payload = <unknown>\0"
.LC6:
	.ascii "foo start\0"
.LC7:
	.ascii "foo end\0"
	.text
	.p2align 4,,15
	.globl	foo
	.def	foo;	.scl	2;	.type	32;	.endef
	.seh_proc	foo
foo:
	pushq	%rbp
	.seh_pushreg	%rbp
	pushq	%rbx
	.seh_pushreg	%rbx
	movq	%rsp, %rbp
	.seh_setframe	%rbp, 0
	subq	$312, %rsp
	.seh_stackalloc	312
	.seh_endprologue
	leaq	-280(%rbp), %rcx
	movq	%rbp, %rdx
	call	_setjmp
	testl	%eax, %eax
	movl	%eax, -16(%rbp)
	jne	.L16
	leaq	__emutls_v.__target(%rip), %rcx
	call	__emutls_get_address
	movq	%rax, %rbx
	movq	(%rax), %rax
	leaq	.LC6(%rip), %rcx
	movq	%rax, -24(%rbp)
	leaq	-280(%rbp), %rax
	movq	%rax, (%rbx)
	call	puts
	leaq	.LC1(%rip), %rcx
	call	puts
	leaq	__emutls_v.COUNTER.4572(%rip), %rcx
	call	__emutls_get_address
	movl	(%rax), %edx
	addl	$1, %edx
	cmpl	$10, %edx
	movl	%edx, (%rax)
	je	.L17
	leaq	.LC7(%rip), %rcx
	call	puts
	movq	-24(%rbp), %rax
	movq	%rax, (%rbx)
	addq	$312, %rsp
	popq	%rbx
	popq	%rbp
	ret
.L16:
	leaq	__emutls_v.__is_unwinding(%rip), %rcx
	call	__emutls_get_address
	leaq	__emutls_v.panic_info(%rip), %rcx
	movb	$1, (%rax)
	call	__emutls_get_address
	movl	(%rax), %edx
	cmpl	$1, %edx
	je	.L11
	jnb	.L18
	movl	8(%rax), %edx
	leaq	.LC4(%rip), %rcx
	call	printf
.L10:
	movq	-24(%rbp), %rbx
	leaq	__emutls_v.__target(%rip), %rcx
	call	__emutls_get_address
	movl	$3, %edx
	movq	%rbx, (%rax)
	movq	%rbx, %rcx
	call	longjmp
.L17:
	leaq	__emutls_v.panic_info(%rip), %rcx
	leaq	.LC2(%rip), %rbx
	call	__emutls_get_address
	movl	$2, (%rax)
	movq	%rbx, 8(%rax)
	call	lang_start_panic_unwind
.L18:
	cmpl	$2, %edx
	jne	.L10
	movq	8(%rax), %rdx
	leaq	.LC3(%rip), %rcx
	call	printf
	jmp	.L10
.L11:
	leaq	.LC5(%rip), %rcx
	call	puts
	jmp	.L10
	.seh_endproc
	.p2align 4,,15
	.globl	boom
	.def	boom;	.scl	2;	.type	32;	.endef
	.seh_proc	boom
boom:
	pushq	%rbp
	.seh_pushreg	%rbp
	pushq	%rbx
	.seh_pushreg	%rbx
	movq	%rsp, %rbp
	.seh_setframe	%rbp, 0
	subq	$312, %rsp
	.seh_stackalloc	312
	.seh_endprologue
	leaq	-280(%rbp), %rcx
	movq	%rbp, %rdx
	call	_setjmp
	testl	%eax, %eax
	movl	%eax, -16(%rbp)
	jne	.L22
	addq	$312, %rsp
	popq	%rbx
	popq	%rbp
	ret
.L22:
	leaq	__emutls_v.__is_unwinding(%rip), %rcx
	call	__emutls_get_address
	movq	-24(%rbp), %rbx
	leaq	__emutls_v.__target(%rip), %rcx
	movb	$1, (%rax)
	call	__emutls_get_address
	movl	$3, %edx
	movq	%rbx, (%rax)
	movq	%rbx, %rcx
	call	longjmp
	nop
	.seh_endproc
	.section .rdata,"dr"
.LC8:
	.ascii "(main) PANIC: %s\12\0"
	.align 8
.LC9:
	.ascii "(main) panic detected: message = %s\12\0"
	.align 8
.LC10:
	.ascii "(main) panic detected: status = %s\12\0"
	.align 8
.LC11:
	.ascii "(main) panic detected: payload = <unknown>\0"
.LC12:
	.ascii "hello\0"
	.text
	.p2align 4,,15
	.globl	__lang_main
	.def	__lang_main;	.scl	2;	.type	32;	.endef
	.seh_proc	__lang_main
__lang_main:
	pushq	%rbp
	.seh_pushreg	%rbp
	pushq	%rsi
	.seh_pushreg	%rsi
	pushq	%rbx
	.seh_pushreg	%rbx
	movq	%rsp, %rbp
	.seh_setframe	%rbp, 0
	subq	$320, %rsp
	.seh_stackalloc	320
	.seh_endprologue
	leaq	-272(%rbp), %rcx
	movq	%rbp, %rdx
	call	_setjmp
	testl	%eax, %eax
	movl	%eax, -8(%rbp)
	jne	.L33
	leaq	__emutls_v.__target(%rip), %rcx
	movl	$10, %ebx
	call	__emutls_get_address
	movq	%rax, %rsi
	movq	(%rax), %rax
	movq	%rax, -16(%rbp)
	leaq	-272(%rbp), %rax
	movq	%rax, (%rsi)
	leaq	.LC12(%rip), %rax
	movq	%rax, -280(%rbp)
	.p2align 4,,10
.L31:
	call	foo
	subl	$1, %ebx
	jne	.L31
	movq	-16(%rbp), %rax
	movq	%rax, (%rsi)
	addq	$320, %rsp
	popq	%rbx
	popq	%rsi
	popq	%rbp
	ret
.L33:
	leaq	__emutls_v.__is_unwinding(%rip), %rcx
	call	__emutls_get_address
	movq	-280(%rbp), %rdx
	leaq	.LC8(%rip), %rcx
	movb	$1, (%rax)
	call	printf
	leaq	__emutls_v.panic_info(%rip), %rcx
	call	__emutls_get_address
	movl	(%rax), %edx
	cmpl	$1, %edx
	je	.L27
	jb	.L28
	cmpl	$2, %edx
	jne	.L26
	movq	8(%rax), %rdx
	leaq	.LC9(%rip), %rcx
	call	printf
.L26:
	movq	-16(%rbp), %rbx
	leaq	__emutls_v.__target(%rip), %rcx
	call	__emutls_get_address
	movl	$3, %edx
	movq	%rbx, (%rax)
	movq	%rbx, %rcx
	call	longjmp
.L28:
	movl	8(%rax), %edx
	leaq	.LC10(%rip), %rcx
	call	printf
	jmp	.L26
.L27:
	leaq	.LC11(%rip), %rcx
	call	puts
	jmp	.L26
	.seh_endproc
	.def	__main;	.scl	2;	.type	32;	.endef
	.section .rdata,"dr"
	.align 8
.LC13:
	.ascii "(start) panic detected: message = %s\12\0"
	.align 8
.LC14:
	.ascii "(start) panic detected: status = %s\12\0"
	.align 8
.LC15:
	.ascii "(start) panic detected: payload = <unknown>\0"
	.section	.text.startup,"x"
	.p2align 4,,15
	.globl	main
	.def	main;	.scl	2;	.type	32;	.endef
	.seh_proc	main
main:
	pushq	%rbp
	.seh_pushreg	%rbp
	pushq	%rbx
	.seh_pushreg	%rbx
	movq	%rsp, %rbp
	.seh_setframe	%rbp, 0
	subq	$296, %rsp
	.seh_stackalloc	296
	.seh_endprologue
	call	__main
	leaq	__emutls_v.__target(%rip), %rcx
	call	__emutls_get_address
	leaq	-264(%rbp), %rcx
	movq	%rbp, %rdx
	movq	%rcx, (%rax)
	call	_setjmp
	testl	%eax, %eax
	movl	%eax, %ebx
	je	.L36
	leaq	__emutls_v.panic_info(%rip), %rcx
	call	__emutls_get_address
	movl	(%rax), %edx
	cmpl	$1, %edx
	je	.L38
	jb	.L39
	cmpl	$2, %edx
	jne	.L34
	movq	8(%rax), %rdx
	leaq	.LC13(%rip), %rcx
	call	printf
.L34:
	movl	%ebx, %eax
	addq	$296, %rsp
	popq	%rbx
	popq	%rbp
	ret
.L36:
	call	__lang_main
	movl	%eax, %ebx
	jmp	.L34
.L39:
	movl	8(%rax), %edx
	leaq	.LC14(%rip), %rcx
	call	printf
	jmp	.L34
.L38:
	leaq	.LC15(%rip), %rcx
	call	puts
	jmp	.L34
	.seh_endproc
	.data
	.align 32
__emutls_v.__target:
	.quad	8
	.quad	8
	.quad	0
	.quad	0
	.section .rdata,"dr"
__emutls_t.__is_unwinding:
	.space 1
	.data
	.align 32
__emutls_v.__is_unwinding:
	.quad	1
	.quad	1
	.quad	0
	.quad	__emutls_t.__is_unwinding
	.align 32
__emutls_v.panic_info:
	.quad	16
	.quad	8
	.quad	0
	.quad	0
	.section .rdata,"dr"
	.align 4
__emutls_t.COUNTER.4572:
	.space 4
	.data
	.align 32
__emutls_v.COUNTER.4572:
	.quad	4
	.quad	4
	.quad	0
	.quad	__emutls_t.COUNTER.4572
	.ident	"GCC: (Rev1, Built by MSYS2 project) 7.2.0"
	.def	__emutls_get_address;	.scl	2;	.type	32;	.endef
	.def	printf;	.scl	2;	.type	32;	.endef
	.def	abort;	.scl	2;	.type	32;	.endef
	.def	longjmp;	.scl	2;	.type	32;	.endef
	.def	puts;	.scl	2;	.type	32;	.endef
	.def	_setjmp;	.scl	2;	.type	32;	.endef
