pub mod assembler;
mod assembler6800;
pub mod frontend;

pub use assembler6800::*;

#[cfg(test)]
mod tests {
    use crate::assembler::Assembler;
    use crate::cpukind::CpuKind;
    use crate::opts::Opts;

    /// Assemble `src` as a 6800 project and return the bytes at `addr`.
    fn assemble_bytes(name: &str, src: &str, addr: usize, count: usize) -> Vec<u8> {
        let path = std::env::temp_dir().join(format!("gazm_6800_{name}.gazm"));
        std::fs::write(&path, src).unwrap();

        let opts = Opts {
            project_file: path.clone(),
            assemble_dir: Some(std::env::temp_dir()),
            cpu: CpuKind::Cpu6800,
            ..Default::default()
        };
        let mut asm = Assembler::new(opts).unwrap();
        let res = asm.assemble();
        let _ = std::fs::remove_file(&path);
        assert!(res.is_ok(), "Assembly failed: {:?}", res.err());

        asm.get_binary()
            .get_bytes(addr, count)
            .expect("Can't read bytes")
            .to_vec()
    }

    #[test]
    fn m6800_instructions_encode() {
        // Expected bytes verified against the MAME-verified v2 table
        // (emu6800/resources/opcodes6800.json).
        let cases: &[(&str, &[u8])] = &[
            ("ldaa #5", &[0x86, 0x05]),
            ("ldaa 5", &[0x96, 0x05]),  // extended -> direct (page zero)
            ("ldaa <5", &[0x96, 0x05]), // forced direct
            ("ldaa >5", &[0xb6, 0x00, 0x05]), // forced extended
            ("ldaa $1000", &[0xb6, 0x10, 0x00]),
            ("ldaa 5,x", &[0xa6, 0x05]),
            ("ldab #5", &[0xc6, 0x05]),
            ("ldab $1000", &[0xf6, 0x10, 0x00]),
            ("staa 5", &[0x97, 0x05]),
            ("staa $1000", &[0xb7, 0x10, 0x00]),
            ("staa 5,x", &[0xa7, 0x05]),
            ("ldx #$1234", &[0xce, 0x12, 0x34]),
            ("ldx 5", &[0xde, 0x05]),
            ("ldx 5,x", &[0xee, 0x05]),
            ("ldx $1000", &[0xfe, 0x10, 0x00]),
            ("sts 5", &[0x9f, 0x05]),
            ("sts $1000", &[0xbf, 0x10, 0x00]),
            ("cpx 5", &[0x9c, 0x05]),
            ("jmp $1000", &[0x7e, 0x10, 0x00]),
            ("jmp 5", &[0x7e, 0x00, 0x05]), // no direct form: stays extended
            ("jmp 5,x", &[0x6e, 0x05]),
            ("jsr $1000", &[0xbd, 0x10, 0x00]),
            ("jsr 5,x", &[0xad, 0x05]),
            ("adda #5", &[0x8b, 0x05]),
            ("suba #5", &[0x80, 0x05]),
            ("cmpa #5", &[0x81, 0x05]),
            ("sbca #5", &[0x82, 0x05]),
            ("anda #5", &[0x84, 0x05]),
            ("bita #5", &[0x85, 0x05]),
            ("eora #5", &[0x88, 0x05]),
            ("adca #5", &[0x89, 0x05]),
            ("oraa #5", &[0x8a, 0x05]),
            ("nop", &[0x01]),
            ("rts", &[0x39]),
            ("wai", &[0x3e]),
            ("tpa", &[0x07]),
            ("aba", &[0x1b]),
            ("sec", &[0x0d]),
            ("tab", &[0x16]),
            ("inca", &[0x4c]),
            ("clra", &[0x4f]),
            ("clrb", &[0x5f]),
            ("dex", &[0x09]),
            ("inx", &[0x08]),
            ("des", &[0x34]),
            ("ins", &[0x31]),
            ("psha", &[0x36]),
            ("pshb", &[0x37]),
            ("pula", &[0x32]),
            ("pulb", &[0x33]),
            ("tst $1000", &[0x7d, 0x10, 0x00]),
        ];

        for (i, (line, expected)) in cases.iter().enumerate() {
            let src = format!("org 0\n  {line}");
            let got = assemble_bytes(&format!("enc_{i}"), &src, 0, expected.len());
            assert_eq!(&got, expected, "encoding of {line:?}");
        }
    }

    #[test]
    fn m6800_relative_branches() {
        let src = "org 0\n  bra skip\n  nop\nskip: nop";
        // bra at 0 (2 bytes); skip is at 3: displacement = 3 - 2 = 1.
        assert_eq!(
            assemble_bytes("bra_skip", src, 0, 3),
            vec![0x20, 0x01, 0x01]
        );
    }
}
