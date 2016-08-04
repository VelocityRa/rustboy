use super::*;
use super::super::mmu;

impl Cpu {
	pub fn nop(&mut self) {
		println!("NOP:{:04X}", self.regs.pc);
	}

	pub fn stop(&mut self) {
		panic!("STOP instruction issued");
	}
}


pub fn exec(inst: u8, r: &mut Registers, m: &mut mmu::Memory) -> u32 {

	macro_rules! ld(
		($reg1:ident, $reg2:ident) => ({ r.$reg1 = r.$reg2; 1 }) );

	macro_rules! ld_n (
		($reg1:ident) => ({	r.$reg1 = m.rb(r.bump()); 2	}) );

	macro_rules! ld_nn (
		($reg1:ident, $reg2:ident) => ({
			r.$reg2 = m.rb(r.bump());
			r.$reg1 = m.rb(r.bump());
			3 }) );

	// Table is from https://github.com/alexcrichton/jba/blob/rust/src/cpu/z80/imp.rs#L279-L549
	// Instruction macros implemented by me
	match inst {
        0x00 => 1,                                                  // nop
        0x01 => ld_nn!(b, c),                                       // ld_bcnn
        
        0x02 => { m.wb(r.bc(), r.a); 2 }                            // ld_bca
        // 0x03 => inc_16!(b, c),                                      // inc_bc
        // 0x04 => inc!(b),                                            // inc_b
        // 0x05 => dec!(b),                                            // dec_b
        0x06 => ld_n!(b),                                           // ld_bn
        // 0x07 => rlc!(r.a, 1),                                       // rlca
        0x08 => { let a = m.rw(r.pc); m.ww(a, r.sp); r.pc += 2; 5 } // ld_nnsp
        // 0x09 => add_hl!(r.bc()),                                    // add_hlbc
        0x0a => { r.a = m.rb(r.bc()); 2 }                           // ld_abc
        // 0x0b => dec_16!(b, c),                                      // dec_bc
        // 0x0c => inc!(c),                                            // inc_c
        // 0x0d => dec!(c),                                            // dec_c
        0x0e => ld_n!(c),                                           // ld_cn
        // 0x0f => rrc!(r.a, 1),                                       // rrca

        // TODO: Handle STOP
        0x10 => { r.stop = true; 1}                                   // stop
        0x11 => ld_nn!(d, e),                                       // ld_denn
        0x12 => { m.wb(r.de(), r.a); 2 }                            // ld_dea
        // 0x13 => inc_16!(d, e),                                      // inc_de
        // 0x14 => inc!(d),                                            // inc_d
        // 0x15 => dec!(d),                                            // dec_d
        0x16 => ld_n!(d),                                           // ld_dn
        // 0x17 => rl!(r.a, 1),                                        // rla
        // 0x18 => jr!(),                                              // jr_n
        // 0x19 => add_hl!(r.de()),                                    // add_hlde
        0x1a => { r.a = m.rb(r.de()); 2 }                           // ld_ade
        // 0x1b => dec_16!(d, e),                                      // dec_de
        // 0x1c => inc!(e),                                            // inc_e
        // 0x1d => dec!(e),                                            // dec_e
        0x1e => ld_n!(e),                                           // ld_en
        // 0x1f => rr!(r.a, 1),                                        // rr_a

        // 0x20 => jr_n!((r.f & Z) == 0),                              // jr_nz_n
        0x21 => ld_nn!(h, l),                                       // ld_hlnn
        0x22 => { m.wb(r.hl(), r.a); r.hlpp(); 2 },                 // ld_hlma
        // 0x23 => inc_16!(h, l),                                      // inc_hl
        // 0x24 => inc!(h),                                            // inc_h
        // 0x25 => dec!(h),                                            // dec_h
        0x26 => ld_n!(h),                                           // ld_hn
        // 0x27 => { daa(r); 1 },                                      // daa
        // 0x28 => jr_n!((r.f & Z) != 0),                              // jr_z_n
        // 0x29 => add_hl!(r.hl()),                                    // add_hlhl
        0x2a => { r.a = m.rb(r.hl()); r.hlpp(); 2 },                // ldi_ahlm
        // 0x2b => dec_16!(h, l),                                      // dec_hl
        // 0x2c => inc!(l),                                            // inc_l
        // 0x2d => dec!(l),                                            // dec_l
        0x2e => ld_n!(l),                                           // ld_ln
        0x2f => { r.a ^= 0xff; 
        	r.flags.n.set(); r.flags.h.set(); 1 }                    // cpl

        // 0x30 => jr_n!((r.f & C) == 0),                              // jr_nc_n
        0x31 => { println!("SP is {:04X}", m.rw(r.pc)); r.sp = m.rw(r.pc); r.pc += 2; 3 }                 // ld_spnn
        0x32 => { m.wb(r.hl(), r.a); r.hlmm(); 2 }                  // ldd_hlma
        0x33 => { r.sp += 1; 2 }                                    // inc_sp
        // 0x34 => { inc_hlm(r, m); 3 }                                // inc_hlm
        // 0x35 => { dec_hlm(r, m); 3 }                                // dec_hlm
        0x36 => { let pc = m.rb(r.bump()); m.wb(r.hl(), pc); 3 }    // ld_hlmn
        //0x37 => { r.f = (r.f & Z) | C; 1 }                          // scf
        0x37 => { r.flags.n.unset(); r.flags.h.unset(); 
        	r.flags.cy.set(); 1 }                          // scf
        // 0x38 => jr_n!((r.f & C) != 0),                              // jr_c_n
        // 0x39 => { add_hlsp(r); 2 }                                  // add_hlsp
        0x3a => { r.a = m.rb(r.hl()); r.hlmm(); 2 }                 // ldd_ahlm
        0x3b => { r.sp -= 1; 2 }                                    // dec_sp
        // 0x3c => inc!(a),                                            // inc_a
        // 0x3d => dec!(a),                                            // dec_a
        0x3e => ld_n!(a),                                           // ld_an
        //0x3f => { r.f = (r.f & Z) | ((r.f & C) ^ C); 1 }            // ccf
        0x3f => { r.flags.h.unset(); r.flags.n.unset(); r.flags.cy.toggle(); 1 } // ccf

        0x40 => ld!(b, b),                                          // ld_bb
        0x41 => ld!(b, c),                                          // ld_bc
        0x42 => ld!(b, d),                                          // ld_bd
        0x43 => ld!(b, e),                                          // ld_be
        0x44 => ld!(b, h),                                          // ld_bh
        0x45 => ld!(b, l),                                          // ld_bl
        0x46 => { r.b = m.rb(r.hl()); 2 }                           // ld_bhlm
        0x47 => ld!(b, a),                                          // ld_ba
        0x48 => ld!(c, b),                                          // ld_cb
        0x49 => ld!(c, c),                                          // ld_cc
        0x4a => ld!(c, d),                                          // ld_cd
        0x4b => ld!(c, e),                                          // ld_ce
        0x4c => ld!(c, h),                                          // ld_ch
        0x4d => ld!(c, l),                                          // ld_cl
        0x4e => { r.c = m.rb(r.hl()); 2 }                           // ld_chlm
        0x4f => ld!(c, a),                                          // ld_ca

        0x50 => ld!(d, b),                                          // ld_db
        0x51 => ld!(d, c),                                          // ld_dc
        0x52 => ld!(d, d),                                          // ld_dd
        0x53 => ld!(d, e),                                          // ld_de
        0x54 => ld!(d, h),                                          // ld_dh
        0x55 => ld!(d, l),                                          // ld_dl
        0x56 => { r.d = m.rb(r.hl()); 2 }                           // ld_dhlm
        0x57 => ld!(d, a),                                          // ld_da
        0x58 => ld!(e, b),                                          // ld_eb
        0x59 => ld!(e, c),                                          // ld_ec
        0x5a => ld!(e, d),                                          // ld_ed
        0x5b => ld!(e, e),                                          // ld_ee
        0x5c => ld!(e, h),                                          // ld_eh
        0x5d => ld!(e, l),                                          // ld_el
        0x5e => { r.e = m.rb(r.hl()); 2 }                           // ld_ehlm
        0x5f => ld!(e, a),                                          // ld_ea

        0x60 => ld!(h, b),                                          // ld_hb
        0x61 => ld!(h, c),                                          // ld_hc
        0x62 => ld!(h, d),                                          // ld_hd
        0x63 => ld!(h, e),                                          // ld_he
        0x64 => ld!(h, h),                                          // ld_hh
        0x65 => ld!(h, l),                                          // ld_hl
        0x66 => { r.h = m.rb(r.hl()); 2 }                           // ld_hhlm
        0x67 => ld!(h, a),                                          // ld_ha
        0x68 => ld!(l, b),                                          // ld_lb
        0x69 => ld!(l, c),                                          // ld_lc
        0x6a => ld!(l, d),                                          // ld_ld
        0x6b => ld!(l, e),                                          // ld_le
        0x6c => ld!(l, h),                                          // ld_lh
        0x6d => ld!(l, l),                                          // ld_ll
        0x6e => { r.l = m.rb(r.hl()); 2 }                           // ld_lhlm
        0x6f => ld!(l, a),                                          // ld_la

        0x70 => { m.wb(r.hl(), r.b); 2 }                            // ld_hlmb
        0x71 => { m.wb(r.hl(), r.c); 2 }                            // ld_hlmc
        0x72 => { m.wb(r.hl(), r.d); 2 }                            // ld_hlmd
        0x73 => { m.wb(r.hl(), r.e); 2 }                            // ld_hlme
        0x74 => { m.wb(r.hl(), r.h); 2 }                            // ld_hlmh
        0x75 => { m.wb(r.hl(), r.l); 2 }                            // ld_hlml
        0x76 => { r.halt = true; 1 }                                   // halt
        0x77 => { m.wb(r.hl(), r.a); 2 }                            // ld_hlma
        0x78 => ld!(a, b),                                          // ld_ab
        0x79 => ld!(a, c),                                          // ld_ac
        0x7a => ld!(a, d),                                          // ld_ad
        0x7b => ld!(a, e),                                          // ld_ae
        0x7c => ld!(a, h),                                          // ld_ah
        0x7d => ld!(a, l),                                          // ld_al
        0x7e => { r.a = m.rb(r.hl()); 2 }                           // ld_ahlm
        0x7f => ld!(a, a),                                          // ld_aa

        // 0x80 => add_a!(r.b),                                        // add_ab
        // 0x81 => add_a!(r.c),                                        // add_ac
        // 0x82 => add_a!(r.d),                                        // add_ad
        // 0x83 => add_a!(r.e),                                        // add_ae
        // 0x84 => add_a!(r.h),                                        // add_ah
        // 0x85 => add_a!(r.l),                                        // add_al
        // 0x86 => { add_a!(m.rb(r.hl())); 2 }                         // add_ahlm
        // 0x87 => add_a!(r.a),                                        // add_aa
        // 0x88 => adc_a!(r.b),                                        // adc_ab
        // 0x89 => adc_a!(r.c),                                        // adc_ac
        // 0x8a => adc_a!(r.d),                                        // adc_ad
        // 0x8b => adc_a!(r.e),                                        // adc_ae
        // 0x8c => adc_a!(r.h),                                        // adc_ah
        // 0x8d => adc_a!(r.l),                                        // adc_al
        // 0x8e => { adc_a!(m.rb(r.hl())); 2 }                         // adc_ahlm
        // 0x8f => adc_a!(r.a),                                        // adc_aa

        // 0x90 => sub_a!(r.b),                                        // sub_ab
        // 0x91 => sub_a!(r.c),                                        // sub_ac
        // 0x92 => sub_a!(r.d),                                        // sub_ad
        // 0x93 => sub_a!(r.e),                                        // sub_ae
        // 0x94 => sub_a!(r.h),                                        // sub_ah
        // 0x95 => sub_a!(r.l),                                        // sub_al
        // 0x96 => { sub_a!(m.rb(r.hl())); 2 }                         // sub_ahlm
        // 0x97 => sub_a!(r.a),                                        // sub_aa
        // 0x98 => sbc_a!(r.b),                                        // sbc_ab
        // 0x99 => sbc_a!(r.c),                                        // sbc_ac
        // 0x9a => sbc_a!(r.d),                                        // sbc_ad
        // 0x9b => sbc_a!(r.e),                                        // sbc_ae
        // 0x9c => sbc_a!(r.h),                                        // sbc_ah
        // 0x9d => sbc_a!(r.l),                                        // sbc_al
        // 0x9e => { sbc_a!(m.rb(r.hl())); 2 }                         // sbc_ahlm
        // 0x9f => sbc_a!(r.a),                                        // sbc_aa

        // 0xa0 => and_a!(r.b),                                        // and_ab
        // 0xa1 => and_a!(r.c),                                        // and_ac
        // 0xa2 => and_a!(r.d),                                        // and_ad
        // 0xa3 => and_a!(r.e),                                        // and_ae
        // 0xa4 => and_a!(r.h),                                        // and_ah
        // 0xa5 => and_a!(r.l),                                        // and_al
        // 0xa6 => { and_a!(m.rb(r.hl())); 2 }                         // and_ahlm
        // 0xa7 => and_a!(r.a),                                        // and_aa
        // 0xa8 => xor_a!(r.b),                                        // xor_ab
        // 0xa9 => xor_a!(r.c),                                        // xor_ac
        // 0xaa => xor_a!(r.d),                                        // xor_ad
        // 0xab => xor_a!(r.e),                                        // xor_ae
        // 0xac => xor_a!(r.h),                                        // xor_ah
        // 0xad => xor_a!(r.l),                                        // xor_al
        // 0xae => { xor_a!(m.rb(r.hl())); 2 }                         // xor_ahlm
        // 0xaf => xor_a!(r.a),                                        // xor_aa

        // 0xb0 => or_a!(r.b),                                         // or_ab
        // 0xb1 => or_a!(r.c),                                         // or_ac
        // 0xb2 => or_a!(r.d),                                         // or_ad
        // 0xb3 => or_a!(r.e),                                         // or_ae
        // 0xb4 => or_a!(r.h),                                         // or_ah
        // 0xb5 => or_a!(r.l),                                         // or_al
        // 0xb6 => { or_a!(m.rb(r.hl())); 2 }                          // or_ahlm
        // 0xb7 => or_a!(r.a),                                         // or_aa
        // 0xb8 => cp_a!(r.b),                                         // cp_ab
        // 0xb9 => cp_a!(r.c),                                         // cp_ac
        // 0xba => cp_a!(r.d),                                         // cp_ad
        // 0xbb => cp_a!(r.e),                                         // cp_ae
        // 0xbc => cp_a!(r.h),                                         // cp_ah
        // 0xbd => cp_a!(r.l),                                         // cp_al
        // 0xbe => { cp_a!(m.rb(r.hl())); 2 }                          // cp_ahlm
        // 0xbf => cp_a!(r.a),                                         // cp_aa

        // 0xc0 => ret_if!((r.f & Z) == 0),                            // ret_nz
        // 0xc1 => pop!(b, c),                                         // pop_bc
        // 0xc2 => jp_n!((r.f & Z) == 0),                              // jp_nz_nn
        // 0xc3 => jp!(),                                              // jp_nn
        // 0xc4 => call_if!((r.f & Z) == 0),                           // call_nz_n
        // 0xc5 => push!(b, c),                                        // push_bc
        // 0xc6 => { add_a!(m.rb(r.bump())); 2 }                       // add_an
        // 0xc7 => rst!(0x00),                                         // rst_00
        // 0xc8 => ret_if!((r.f & Z) != 0),                            // ret_z
        // 0xc9 => { r.ret(m); 4 }                                     // ret
        // 0xca => jp_n!((r.f & Z) != 0),                              // jp_z_nn
        // 0xcb => { exec_cb(m.rb(r.bump()), r, m) }                   // map_cb
        // 0xcc => call_if!((r.f & Z) != 0),                           // call_z_n
        // 0xcd => call!(),                                            // call
        // 0xce => { adc_a!(m.rb(r.bump())); 2 }                       // adc_an
        // 0xcf => rst!(0x08),                                         // rst_08

        // 0xd0 => ret_if!((r.f & C) == 0),                            // ret_nc
        // 0xd1 => pop!(d, e),                                         // pop_de
        // 0xd2 => jp_n!((r.f & C) == 0),                              // jp_nc_nn
        0xd3 => xx(),                                               // xx
        // 0xd4 => call_if!((r.f & C) == 0),                           // call_nc_n
        // 0xd5 => push!(d, e),                                        // push_de
        // 0xd6 => { sub_a!(m.rb(r.bump())); 2 }                       // sub_an
        // 0xd7 => rst!(0x10),                                         // rst_10
        // 0xd8 => ret_if!((r.f & C) != 0),                            // ret_c
        // 0xd9 => { r.ei(m); r.ret(m); 4 }                            // reti
        // 0xda => jp_n!((r.f & C) != 0),                              // jp_c_nn
        0xdb => xx(),                                               // xx
        // 0xdc => call_if!((r.f & C) != 0),                           // call_c_n
        0xdd => xx(),                                               // xx
        // 0xde => { sbc_a!(m.rb(r.bump())); 2 }                       // sbc_an
        // 0xdf => rst!(0x18),                                         // rst_18

        // 0xe0 => { ld_IOan(r, m); 3 }                                // ld_IOan
        // 0xe1 => pop!(h, l),                                         // pop_hl
        // 0xe2 => { m.wb(0xff00 | (r.c as u16), r.a); 2 }             // ld_IOca
        0xe3 => xx(),                                               // xx
        0xe4 => xx(),                                               // xx
        // 0xe5 => push!(h, l),                                        // push_hl
        // 0xe6 => { and_a!(m.rb(r.bump())); 2 }                       // and_an
        // 0xe7 => rst!(0x20),                                         // rst_20
        // 0xe8 => { add_spn(r, m); 4 }                                // add_spn
        0xe9 => { r.pc = r.hl(); 1 }                                // jp_hl
        0xea => { let n = m.rw(r.pc); m.wb(n, r.a); r.pc += 2; 4 }  // ld_nna
        0xeb => xx(),                                               // xx
        0xec => xx(),                                               // xx
        0xed => xx(),                                               // xx
        // 0xee => { xor_a!(m.rb(r.bump())); 2 }                       // xor_an
        // 0xef => rst!(0x28),                                         // rst_28

        // 0xf0 => { r.a = m.rb(0xff00 | (m.rb(r.bump()) as u16)); 3 } // ld_aIOn
        // 0xf1 => { pop_af(r, m); 3 }                                 // pop_af
        // 0xf2 => { r.a = m.rb(0xff00 | (r.c as u16)); 2 }            // ld_aIOc
        // 0xf3 => { r.di(); 1 }                                       // di
        0xf4 => xx(),                                               // xx
        // 0xf5 => push!(a, f),                                        // push_af
        // 0xf6 => { or_a!(m.rb(r.bump())); 2 }                        // or_an
        // 0xf7 => rst!(0x30),                                         // rst_30
        // 0xf8 => { ld_hlspn(r, m); 3 }                               // ld_hlspn
        // 0xf9 => { r.sp = r.hl(); 2 }                                // ld_sphl
        // 0xfa => { r.a = m.rb(m.rw(r.pc)); r.pc += 2; 4 }            // ld_ann
        // 0xfb => { r.ei(m); 1 }                                      // ei
        0xfc => xx(),                                               // xx
        0xfd => xx(),                                               // xx
        // 0xfe => { cp_a!(m.rb(r.bump())); 2 }                        // cp_an
        // 0xff => rst!(0x38),                                         // rst_38

        _ => 0
	}
	
}

fn xx() -> u32 { panic!(); 0 }
