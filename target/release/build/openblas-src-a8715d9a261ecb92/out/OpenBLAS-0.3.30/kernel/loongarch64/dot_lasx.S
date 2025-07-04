/***************************************************************************
Copyright (c) 2023, The OpenBLAS Project
All rights reserved.
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:
1. Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright
notice, this list of conditions and the following disclaimer in
the documentation and/or other materials provided with the
distribution.
3. Neither the name of the OpenBLAS project nor the names of
its contributors may be used to endorse or promote products
derived from this software without specific prior written permission.
THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE OPENBLAS PROJECT OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*****************************************************************************/

#define ASSEMBLER

#include "common.h"

#define N      $r4
#define X      $r5
#define INCX   $r6
#define Y      $r7
#define INCY   $r8

#define I      $r17
#define TEMP   $r18

/* Don't change following FR unless you know the effects. */
#define s1     $f8
#define s2     $f9
#define a1     $f10
#define b1     $f11

PROLOGUE

#ifdef F_INTERFACE
    LDINT     N,      0(N)
    LDINT     INCX,   0(INCX)
    LDINT     INCY,   0(INCY)
#endif

    /* init $f8 and $f9 to zero */
    xvxor.v   $xr8,   $xr8,    $xr8
    xvxor.v   $xr9,   $xr9,    $xr9
    slli.d    INCX,   INCX,    BASE_SHIFT
    li.d      TEMP,   SIZE
    slli.d    INCY,   INCY,    BASE_SHIFT
    bge       $r0,    N,       .L999
    bne       INCX,   TEMP,    .L20   /* inc_x=1 */
    bne       INCY,   TEMP,    .L20   /* inc_y=1 */

    /* !((inc_x == 1) && (inc_y == 1)) */


#ifdef DOUBLE
    srai.d    I,      N,       4
#else
    srai.d    I,      N,       5
#endif
    bge       $r0,    I,       .L12   /* FLOAT: <32 ; DOUBLE: <16 */
    .align  3
.L11:
    /* FLOAT: 32~ ; DOUBLE: 16~ */
    xvld      $xr0,   X,       0
    xvld      $xr1,   X,       32
    xvld      $xr2,   X,       64
    xvld      $xr3,   X,       96
    xvld      $xr4,   Y,       0
    xvld      $xr5,   Y,       32
    xvld      $xr6,   Y,       64
    xvld      $xr7,   Y,       96
    addi.w    I,      I,       -1
    addi.d    X,      X,       128
    addi.d    Y,      Y,       128
#ifndef DOUBLE
    xvfcvtl.d.s       $xr10,   $xr0
    xvfcvtl.d.s       $xr11,   $xr4
    xvfcvth.d.s       $xr12,   $xr0
    xvfcvth.d.s       $xr13,   $xr4
    xvfmadd.d $xr8,   $xr10,   $xr11,  $xr8
    xvfmadd.d $xr9,   $xr12,   $xr13,  $xr9
    xvfcvtl.d.s       $xr10,   $xr1
    xvfcvtl.d.s       $xr11,   $xr5
    xvfcvth.d.s       $xr12,   $xr1
    xvfcvth.d.s       $xr13,   $xr5
    xvfmadd.d $xr8,   $xr10,   $xr11,  $xr8
    xvfmadd.d $xr9,   $xr12,   $xr13,  $xr9
    xvfcvtl.d.s       $xr10,   $xr2
    xvfcvtl.d.s       $xr11,   $xr6
    xvfcvth.d.s       $xr12,   $xr2
    xvfcvth.d.s       $xr13,   $xr6
    xvfmadd.d $xr8,   $xr10,   $xr11,  $xr8
    xvfmadd.d $xr9,   $xr12,   $xr13,  $xr9
    xvfcvtl.d.s       $xr10,   $xr3
    xvfcvtl.d.s       $xr11,   $xr7
    xvfcvth.d.s       $xr12,   $xr3
    xvfcvth.d.s       $xr13,   $xr7
    xvfmadd.d $xr8,   $xr10,   $xr11,  $xr8
    xvfmadd.d $xr9,   $xr12,   $xr13,  $xr9
#else
    XVFMADD   $xr8,   $xr0,    $xr4,   $xr8
    XVFMADD   $xr9,   $xr1,    $xr5,   $xr9
    XVFMADD   $xr8,   $xr2,    $xr6,   $xr8
    XVFMADD   $xr9,   $xr3,    $xr7,   $xr9
#endif
    bnez      I,      .L11
    .align  3
.L12:
#ifdef DOUBLE
    andi      I,      N,       0xf
    srai.d    I,      I,       2
#else
    andi      I,      N,       0x1f
    srai.d    I,      I,       3
#endif
    bge       $r0,    I,       .L14   /* DOUBLE: <4 ; FLOAT: <8 */
    .align  3
.L13:
    /* FLOAT: 8~31 ; DOUBLE: 4~15 */
    xvld      $xr0,   X,       0
    xvld      $xr4,   Y,       0
    addi.w    I,      I,       -1
    addi.d    X,      X,       32
    addi.d    Y,      Y,       32
#ifndef DOUBLE
    xvfcvtl.d.s       $xr10,   $xr0
    xvfcvtl.d.s       $xr11,   $xr4
    xvfcvth.d.s       $xr12,   $xr0
    xvfcvth.d.s       $xr13,   $xr4
    xvfmadd.d $xr8,   $xr10,   $xr11,  $xr8
    xvfmadd.d $xr9,   $xr12,   $xr13,  $xr9
#else
    XVFMADD   $xr8,   $xr0,    $xr4,   $xr8
#endif
    bnez      I,      .L13
    .align  3
.L14:
    /* store dot in s1 $f8 */
    xvfadd.d  $xr8,   $xr8,    $xr9
    fsub.d    s2,     s2,      s2  /* set s2 to 0.0 */
    xvpermi.q $xr0,   $xr8,    0x1
    vfadd.d   $vr8,   $vr8,    $vr0
    vpackod.d $vr0,   $vr8,    $vr8
    vfadd.d   $vr8,   $vr8,    $vr0
    .align  3
.L15:
#ifdef DOUBLE
    andi      I,      N,       0x3
#else
    andi      I,      N,       0x7
#endif
    bge       $r0,    I,       .L999  /* =0 */
   .align  3
.L16:
    /* FLOAT: 1~7 ; DOUBLE: 1~3 */
    LD        a1,     X,       0
    LD        b1,     Y,       0
#ifndef DOUBLE
    fcvt.d.s  a1,     a1
    fcvt.d.s  b1,     b1
    fmadd.d   s1,     b1,      a1,     s1
#else
    MADD      s1,     b1,      a1,     s1
#endif
    addi.d    I,      I,       -1
    addi.d    X,      X,       SIZE
    addi.d    Y,      Y,       SIZE
    bnez      I,      .L16
    b         .L999
    .align 3

.L20:
/* !((inc_x == 1) && (inc_y == 1)) */
    srai.d    I,      N,       3
#ifdef F_INTERFACE
    bgez    INCX, .L21
    addi.d  TEMP, N, -1
    mult    TEMP, INCX
    mflo    TEMP
    dsub    X, X, TEMP
    .align 3

.L21:
    bgez    INCY, .L22
    addi.d  TEMP, N, -1
    mult    TEMP, INCY
    mflo    TEMP
    dsub    Y, Y, TEMP
    .align 3

.L22:
#endif
    bge $r0,    I, .L25  /* <8 */
    .align 3

.L23:
    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s1, b1, a1, s1
#else
    MADD  s1, b1, a1, s1
#endif

    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s2, b1, a1, s2
#else
    MADD  s2, b1, a1, s2
#endif

    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s1, b1, a1, s1
#else
    MADD  s1, b1, a1, s1
#endif

    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s2, b1, a1, s2
#else
    MADD  s2, b1, a1, s2
#endif

    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s1, b1, a1, s1
#else
    MADD  s1, b1, a1, s1
#endif

    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s2, b1, a1, s2
#else
    MADD  s2, b1, a1, s2
#endif

    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s1, b1, a1, s1
#else
    MADD  s1, b1, a1, s1
#endif

    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
    addi.d  I, I, -1
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s2, b1, a1, s2
#else
    MADD  s2, b1, a1, s2
#endif
    blt $r0,    I, .L23
    .align 3

.L25:
    andi    I,  N, 7
    bge $r0,    I, .L999
    .align  3

.L26:
    LD a1,  X,   0 * SIZE
    add.d   X, X, INCX
    LD b1,  Y,   0 * SIZE
    add.d   Y, Y, INCY
    addi.d  I, I, -1
#ifndef DOUBLE
    fcvt.d.s  a1, a1
    fcvt.d.s  b1, b1
    fmadd.d  s1, b1, a1, s1
#else
    MADD  s1, b1, a1, s1
#endif
    blt $r0,    I, .L26
    .align 3

.L999:
    fadd.d    $f0,    s1,      s2
    move      $r4,    $r17
#if defined(DOUBLE)
#elif defined(DSDOT)
#else
    fcvt.s.d  $f0,    $f0
#endif
    jirl      $r0,    $r1,     0x0

EPILOGUE
