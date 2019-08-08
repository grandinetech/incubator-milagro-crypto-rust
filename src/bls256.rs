/*
Licensed to the Apache Software Foundation (ASF) under one
or more contributor license agreements.  See the NOTICE file
distributed with this work for additional information
regarding copyright ownership.  The ASF licenses this file
to you under the Apache License, Version 2.0 (the
"License"); you may not use this file except in compliance
with the License.  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing,
software distributed under the License is distributed on an
"AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, either express or implied.  See the License for the
specific language governing permissions and limitations
under the License.
*/

use super::ecp::ECP;
use super::ecp8::ECP8;
use std::str;
//use super::fp48::FP48;
use super::big;
use super::big::Big;
use super::pair256;
use super::rom;

use rand::RAND;
use sha3::SHA3;
use sha3::SHAKE256;

// BLS API Functions
pub const BFS: usize = big::MODBYTES as usize;
pub const BGS: usize = big::MODBYTES as usize;
pub const BLS_OK: isize = 0;
pub const BLS_FAIL: isize = -1;

// hash a message to an ECP point, using SHA3
#[allow(non_snake_case)]
fn bls_hashit(m: &str) -> ECP {
    let mut sh = SHA3::new(SHAKE256);
    let mut hm: [u8; BFS] = [0; BFS];
    let t = m.as_bytes();
    for i in 0..m.len() {
        sh.process(t[i]);
    }
    sh.shake(&mut hm, BFS);
    let P = ECP::mapit(&hm);
    P
}

/// Generate key pair, private key s, public key w
pub fn key_pair_generate(mut rng: &mut RAND, s: &mut [u8], w: &mut [u8]) -> isize {
    let q = Big::new_ints(&rom::CURVE_ORDER);
    let g = ECP8::generator();
    let mut sc = Big::randomnum(&q, &mut rng);
    sc.tobytes(s);
    pair256::g2mul(&g, &mut sc).tobytes(w);
    BLS_OK
}

// Sign message m using private key s to produce signature sig
pub fn sign(sig: &mut [u8], m: &str, s: &[u8]) -> isize {
    let d = bls_hashit(m);
    let mut sc = Big::frombytes(&s);
    pair256::g1mul(&d, &mut sc).tobytes(sig, true);
    BLS_OK
}

/// Verify signature given message m, the signature sig, and the public key w
pub fn verify(sig: &[u8], m: &str, w: &[u8]) -> isize {
    let hm = bls_hashit(m);
    let mut d = ECP::frombytes(&sig);
    let g = ECP8::generator();
    let pk = ECP8::frombytes(&w);
    d.neg();

    // Use new multi-pairing mechanism
    let mut r = pair256::initmp();
    pair256::another(&mut r, &g, &d);
    pair256::another(&mut r, &pk, &hm);
    let mut v = pair256::miller(&r);

    //.. or alternatively
    //    let mut v = pair256::ate2(&g, &d, &pk, &hm);

    v = pair256::fexp(&v);
    if v.isunity() {
        return BLS_OK;
    }
    BLS_FAIL
}