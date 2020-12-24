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

.global ret_aaaaaaab
.type ret_aaaaaaab, @function

ret_aaaaaaab:
    lui a0, 0x55555
    xori a0, a0, -0x555
    ret

.global fizzbuzz_gcc:
.type fizzbuzz_gcc, @function

fizzbuzz_gcc:
	li      a4,5
	remuw   a4,a0,a4
	li      a5,3
	remuw   a5,a0,a5
	bnez    a4,.L2
	snez    a0,a5
	addi    a0,a0,-3
	ret

.L2:
	beqz    a5,.L5
	sext.w  a0,a0
	ret

.L5:
	li      a0,-1
	ret
