/*****************************************************************************
  Copyright (c) 2014, Intel Corp.
  All rights reserved.

  Redistribution and use in source and binary forms, with or without
  modification, are permitted provided that the following conditions are met:

    * Redistributions of source code must retain the above copyright notice,
      this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of Intel Corporation nor the names of its contributors
      may be used to endorse or promote products derived from this software
      without specific prior written permission.

  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
  AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
  IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
  ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
  LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
  CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
  SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
  INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
  CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
  ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
  THE POSSIBILITY OF SUCH DAMAGE.
*****************************************************************************
* Contents: Native middle-level C interface to LAPACK function zgesdd
* Author: Intel Corporation
*****************************************************************************/

#include "lapacke_utils.h"

lapack_int LAPACKE_zgesdd_work( int matrix_layout, char jobz, lapack_int m,
                                lapack_int n, lapack_complex_double* a,
                                lapack_int lda, double* s,
                                lapack_complex_double* u, lapack_int ldu,
                                lapack_complex_double* vt, lapack_int ldvt,
                                lapack_complex_double* work, lapack_int lwork,
                                double* rwork, lapack_int* iwork )
{
    lapack_int info = 0;
    if( matrix_layout == LAPACK_COL_MAJOR ) {
        /* Call LAPACK function and adjust info */
        LAPACK_zgesdd( &jobz, &m, &n, a, &lda, s, u, &ldu, vt, &ldvt, work,
                       &lwork, rwork, iwork, &info );
        if( info < 0 ) {
            info = info - 1;
        }
    } else if( matrix_layout == LAPACK_ROW_MAJOR ) {
        lapack_int nrows_u = ( LAPACKE_lsame( jobz, 'a' ) ||
                             LAPACKE_lsame( jobz, 's' ) ||
                             ( LAPACKE_lsame( jobz, 'o' ) && m<n) ) ? m : 1;
        lapack_int ncols_u = ( LAPACKE_lsame( jobz, 'a' ) ||
                             ( LAPACKE_lsame( jobz, 'o' ) && m<n) ) ? m :
                             ( LAPACKE_lsame( jobz, 's' ) ? MIN(m,n) : 1);
        lapack_int nrows_vt = ( LAPACKE_lsame( jobz, 'a' ) ||
                              ( LAPACKE_lsame( jobz, 'o' ) && m>=n) ) ? n :
                              ( LAPACKE_lsame( jobz, 's' ) ? MIN(m,n) : 1);
        lapack_int ncols_vt = ( LAPACKE_lsame( jobz, 'a' ) ||
                                LAPACKE_lsame( jobz, 's' ) ||
                              ( LAPACKE_lsame( jobz, 'o' && m >=n) ) ? n : 1);
        lapack_int lda_t = MAX(1,m);
        lapack_int ldu_t = MAX(1,nrows_u);
        lapack_int ldvt_t = MAX(1,nrows_vt);
        lapack_complex_double* a_t = NULL;
        lapack_complex_double* u_t = NULL;
        lapack_complex_double* vt_t = NULL;
        /* Check leading dimension(s) */
        if( lda < n ) {
            info = -6;
            LAPACKE_xerbla( "LAPACKE_zgesdd_work", info );
            return info;
        }
        if( ldu < ncols_u ) {
            info = -9;
            LAPACKE_xerbla( "LAPACKE_zgesdd_work", info );
            return info;
        }
        if( ldvt < ncols_vt ) {
            info = -11;
            LAPACKE_xerbla( "LAPACKE_zgesdd_work", info );
            return info;
        }
        /* Query optimal working array(s) size if requested */
        if( lwork == -1 ) {
            LAPACK_zgesdd( &jobz, &m, &n, a, &lda_t, s, u, &ldu_t, vt, &ldvt_t,
                           work, &lwork, rwork, iwork, &info );
            return (info < 0) ? (info - 1) : info;
        }
        /* Allocate memory for temporary array(s) */
        a_t = (lapack_complex_double*)
            LAPACKE_malloc( sizeof(lapack_complex_double) * lda_t * MAX(1,n) );
        if( a_t == NULL ) {
            info = LAPACK_TRANSPOSE_MEMORY_ERROR;
            goto exit_level_0;
        }
        if( LAPACKE_lsame( jobz, 'a' ) || LAPACKE_lsame( jobz, 's' ) ||
            ( LAPACKE_lsame( jobz, 'o' ) && (m<n) ) ) {
            u_t = (lapack_complex_double*)
                LAPACKE_malloc( sizeof(lapack_complex_double) *
                                ldu_t * MAX(1,ncols_u) );
            if( u_t == NULL ) {
                info = LAPACK_TRANSPOSE_MEMORY_ERROR;
                goto exit_level_1;
            }
        }
        if( LAPACKE_lsame( jobz, 'a' ) || LAPACKE_lsame( jobz, 's' ) ||
            ( LAPACKE_lsame( jobz, 'o' ) && (m>=n) ) ) {
            vt_t = (lapack_complex_double*)
                LAPACKE_malloc( sizeof(lapack_complex_double) *
                                ldvt_t * MAX(1,n) );
            if( vt_t == NULL ) {
                info = LAPACK_TRANSPOSE_MEMORY_ERROR;
                goto exit_level_2;
            }
        }
        /* Transpose input matrices */
        LAPACKE_zge_trans( matrix_layout, m, n, a, lda, a_t, lda_t );
        /* Call LAPACK function and adjust info */
        LAPACK_zgesdd( &jobz, &m, &n, a_t, &lda_t, s, u_t, &ldu_t, vt_t,
                       &ldvt_t, work, &lwork, rwork, iwork, &info );
        if( info < 0 ) {
            info = info - 1;
        }
        /* Transpose output matrices */
        LAPACKE_zge_trans( LAPACK_COL_MAJOR, m, n, a_t, lda_t, a, lda );
        if( LAPACKE_lsame( jobz, 'a' ) || LAPACKE_lsame( jobz, 's' ) ||
            ( LAPACKE_lsame( jobz, 'o' ) && (m<n) ) ) {
            LAPACKE_zge_trans( LAPACK_COL_MAJOR, nrows_u, ncols_u, u_t, ldu_t,
                               u, ldu );
        }
        if( LAPACKE_lsame( jobz, 'a' ) || LAPACKE_lsame( jobz, 's' ) ||
            ( LAPACKE_lsame( jobz, 'o' ) && (m>=n) ) ) {
            LAPACKE_zge_trans( LAPACK_COL_MAJOR, nrows_vt, n, vt_t, ldvt_t, vt,
                               ldvt );
        }
        /* Release memory and exit */
        if( LAPACKE_lsame( jobz, 'a' ) || LAPACKE_lsame( jobz, 's' ) ||
            ( LAPACKE_lsame( jobz, 'o' ) && (m>=n) ) ) {
            LAPACKE_free( vt_t );
        }
exit_level_2:
        if( LAPACKE_lsame( jobz, 'a' ) || LAPACKE_lsame( jobz, 's' ) ||
            ( LAPACKE_lsame( jobz, 'o' ) && (m<n) ) ) {
            LAPACKE_free( u_t );
        }
exit_level_1:
        LAPACKE_free( a_t );
exit_level_0:
        if( info == LAPACK_TRANSPOSE_MEMORY_ERROR ) {
            LAPACKE_xerbla( "LAPACKE_zgesdd_work", info );
        }
    } else {
        info = -1;
        LAPACKE_xerbla( "LAPACKE_zgesdd_work", info );
    }
    return info;
}
