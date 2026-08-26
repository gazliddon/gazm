pub mod assembler;
mod assemblerz80;
pub mod frontend;

pub use assemblerz80::*;

#[cfg(test)]
mod tests {
    use crate::assembler::Assembler;
    use crate::cpukind::CpuKind;
    use crate::opts::Opts;

    /// Assemble `src` as a Z80 project and return the bytes at `addr`.
    fn assemble_bytes(name: &str, src: &str, addr: usize, count: usize) -> Vec<u8> {
        let path = std::env::temp_dir().join(format!("gazm_z80_{name}.gazm"));
        std::fs::write(&path, src).unwrap();

        let opts = Opts {
            project_file: path.clone(),
            assemble_dir: Some(std::env::temp_dir()),
            cpu: CpuKind::CpuZ80,
            ..Default::default()
        };
        let mut asm = Assembler::new(opts);
        let res = asm.assemble();
        let _ = std::fs::remove_file(&path);
        assert!(res.is_ok(), "Assembly failed: {:?}", res.err());

        asm.get_binary()
            .get_bytes(addr, count)
            .expect("Can't read bytes")
            .to_vec()
    }

    #[test]
    fn z80_instructions_encode() {
        let cases: &[(&str, &[u8])] = &[
            ("nop", &[0x00]),
            ("halt", &[0x76]),
            ("ld a,5", &[0x3e, 0x05]),
            ("ld b,c", &[0x41]),
            ("ld a,b", &[0x78]),
            ("ld hl,1234", &[0x21, 0xd2, 0x04]),
            ("ld (254),a", &[0x32, 0xfe, 0x00]),
            ("ld a,(254)", &[0x3a, 0xfe, 0x00]),
            ("ld a,(bc)", &[0x0a]),
            ("ld (de),a", &[0x12]),
            ("ld a,(ix-1)", &[0xdd, 0x7e, 0xff]),
            ("ld (ix+2),3", &[0xdd, 0x36, 0x02, 0x03]),
            ("ld ixh,5", &[0xdd, 0x26, 0x05]),
            ("ld iyh,5", &[0xfd, 0x26, 0x05]),
            ("ldir", &[0xed, 0xb0]),
            ("bit 0,(ix+1)", &[0xdd, 0xcb, 0x01, 0x46]),
            ("set 3,a", &[0xcb, 0xdf]),
            ("res 7,(hl)", &[0xcb, 0xbe]),
            ("inc ix", &[0xdd, 0x23]),
            ("dec b", &[0x05]),
            ("add hl,de", &[0x19]),
            ("add a,c", &[0x81]),
            ("and l", &[0xa5]),
            ("xor (hl)", &[0xae]),
            ("cp 10", &[0xfe, 0x0a]),
            ("push bc", &[0xc5]),
            ("pop af", &[0xf1]),
            ("ex de,hl", &[0xeb]),
            ("ex af,af'", &[0x08]),
            ("ret", &[0xc9]),
            ("ret z", &[0xc8]),
            ("ret nz", &[0xc0]),
            ("jp 1234", &[0xc3, 0xd2, 0x04]),
            ("jp (ix)", &[0xdd, 0xe9]),
            ("call 1234", &[0xcd, 0xd2, 0x04]),
            ("call nc,1234", &[0xd4, 0xd2, 0x04]),
            ("rst 8", &[0xcf]),
            ("rst 0", &[0xc7]),
            ("in a,(254)", &[0xdb, 0xfe]),
            ("out (254),a", &[0xd3, 0xfe]),
            ("in a,(c)", &[0xed, 0x78]),
            ("out (c),b", &[0xed, 0x41]),
            ("ld a,i", &[0xed, 0x57]),
            ("ld sp,hl", &[0xf9]),
            ("ex (sp),ix", &[0xdd, 0xe3]),
            ("di", &[0xf3]),
            ("ei", &[0xfb]),
            ("scf", &[0x37]),
            ("ccf", &[0x3f]),
            ("daa", &[0x27]),
            ("cpl", &[0x2f]),
            ("neg", &[0xed, 0x44]),
            ("im 1", &[0xed, 0x56]),
            ("rla", &[0x17]),
            ("rrca", &[0x0f]),
            ("rlc b", &[0xcb, 0x00]),
            ("sla (hl)", &[0xcb, 0x26]),
            ("srl (ix+0)", &[0xdd, 0xcb, 0x00, 0x3e]),
        ];

        for (i, (line, expected)) in cases.iter().enumerate() {
            let src = format!("org 0\n  {line}");
            let got = assemble_bytes(&format!("enc_{i}"), &src, 0, expected.len());
            assert_eq!(&got, expected, "encoding of {line:?}");
        }
    }

    #[test]
    fn z80_relative_jumps() {
        let src = "org 0\n  jr skip\n  nop\n  nop\n  nop\n  nop\nskip: nop";
        // jr at 0 (2 bytes); skip is at 6: displacement = 6 - 2 = 4.
        assert_eq!(assemble_bytes("jr_skip", src, 0, 3), vec![0x18, 0x04, 0x00]);
    }

    #[test]
    fn z80_directives() {
        let src = "org 0\n  defb 1,2,3\n  defw 1234\n  defs 2\n  db 4\n  defm \"hi\"";
        assert_eq!(
            assemble_bytes("z80_defs", src, 0, 10),
            vec![1, 2, 3, 0xd2, 0x04, 0, 0, 4, b'h', b'i']
        );
    }
}
