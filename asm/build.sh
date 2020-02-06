#!/bin/bash
clear
echo "assemble
"
/c/masm32/bin/ml.exe -c -Zd -coff test.asm

if [ $? != 0 ]
then
exit $?
fi

echo "
link
"
/c/masm32/bin/link.exe -SUBSYSTEM:CONSOLE test.obj

if [ $? != 0 ]
then
exit $?
fi

echo "
run
"
./test.exe