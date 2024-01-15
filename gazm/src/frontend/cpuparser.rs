use super::{ Node, PResult, TSpan };

// Isolating CPU specific parsers
trait CpuParser {
    fn parse_opcode(input: TSpan) -> PResult<Node>;
    fn parse_command(input: TSpan) -> PResult<Node>;
}

