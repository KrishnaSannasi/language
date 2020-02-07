#include <setjmp.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
// #include <threads.h>

static __thread jmp_buf *__target;
static __thread _Bool __is_unwinding = 0;

static __thread struct panic {
    enum Type { STATUS, PAYLOAD, MESSAGE } type;
    
    union {
        int status;
        void * payload;
        const char * message;
    };
} panic_info;

struct __land_except_landing_pad {
    volatile jmp_buf target;
    volatile jmp_buf *prev;
    volatile int status;
};

#define _except_return(value)                                       \
    __target = (jmp_buf*)__pad.prev;                                \
    return value

#define __EXCEPTION_LANDING_PAD(guarded, handler)                   \
    struct __land_except_landing_pad __pad;                         \
                                                                    \
    if(__pad.status = setjmp(*(jmp_buf*)__pad.target)) {            \
        __is_unwinding = 1;                                         \
        handler                                                     \
        __target = (jmp_buf*)__pad.prev;                            \
        longjmp(*__target, 3);                                      \
    } else {                                                        \
        __pad.prev = __target;                                      \
        __target = (jmp_buf*)&__pad.target;                         \
        guarded                                                     \
        __target = (jmp_buf*)__pad.prev;                            \
    }

void lang_start_panic_unwind() {
    if(__is_unwinding) {
        printf("paniced while panicing, ABORT!");
        abort();
    }
    longjmp(*__target, 2);
}

int __lang_main();

int main() {
    jmp_buf final_target;
    int __status;

    __target = &final_target;

    if(__status = setjmp(final_target)) {
        switch(panic_info.type) {
        case MESSAGE:
            printf("(start) panic detected: message = %s\n", panic_info.message);
            break;
        case STATUS:
            printf("(start) panic detected: status = %s\n", panic_info.status);
            break;
        case PAYLOAD:
            printf("(start) panic detected: payload = <unknown>\n");
            break;
        }
        
        return __status;
    }

    return __lang_main();
}

int bar() {
    static __thread int COUNTER = 0;

    printf("bar\n");

    if (++COUNTER == 10) {
        panic_info.type = MESSAGE;
        panic_info.message = "bar panicked unexpectedly!";
        lang_start_panic_unwind();
    }
}

int foo() {
    __EXCEPTION_LANDING_PAD(
        printf("foo start\n");
        bar();
        printf("foo end\n");
        ,
        switch(panic_info.type) {
        case MESSAGE:
            printf("(foo) panic detected: message = %s\n", panic_info.message);
            break;
        case STATUS:
            printf("(foo) panic detected: status = %s\n", panic_info.status);
            break;
        case PAYLOAD:
            printf("(foo) panic detected: payload = <unknown>\n");
            break;
        }
    )
}

void boom() {
    __EXCEPTION_LANDING_PAD(
        _except_return();
        ,
    )
}

int __lang_main() {
    char volatile locals[sizeof(char*)];
    
    __EXCEPTION_LANDING_PAD(
        // main code

        *(char* volatile*)(locals + 0) = "hello";
        
        for(int i = 0; i < 10; ++i)
            foo();
        
        ,
        // panic handler

        printf("(main) PANIC: %s\n", *(char**)(locals + 0));

        switch(panic_info.type) {
        case MESSAGE:
            printf("(main) panic detected: message = %s\n", panic_info.message);
            break;
        case STATUS:
            printf("(main) panic detected: status = %s\n", panic_info.status);
            break;
        case PAYLOAD:
            printf("(main) panic detected: payload = <unknown>\n");
            break;
        }

        // continue unwind
    )
}
