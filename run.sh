#! /bin/bash

make FEATURES=irq A=apps/helloworld ARCH=x86_64 GRAPHIC=on BLK=y AX_LOG=info U_LOG=1 run
