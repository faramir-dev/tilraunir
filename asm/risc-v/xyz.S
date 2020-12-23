.section .text

.global add
.type add, @function

add:
    add a0, a0, a1
    ret

.global ret_55555556
.type ret_55555556, @function

ret_55555556:
    lui a0, 0x55555
    ori a0, a0, 0x556
    ret
