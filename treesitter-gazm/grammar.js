'use strict';

const fs = require('fs')

const PREC = {
    PAREN_DECLARATOR: -10,
    ASSIGNMENT: -1,
    CONDITIONAL: -2,
    DEFAULT: 0,
    LOGICAL_OR: 1,
    LOGICAL_AND: 2,
    INCLUSIVE_OR: 3,
    EXCLUSIVE_OR: 4,
    BITWISE_AND: 5,
    EQUAL: 6,
    RELATIONAL: 7,
    SIZEOF: 8,
    SHIFT: 9,
    ADD: 10,
    MULTIPLY: 11,
    CAST: 12,
    UNARY: 13,
    CALL: 14,
    FIELD: 15,
    SUBSCRIPT: 16,
};

const BIN_OPS = [
    ['+', PREC.ADD, 'add'],
    ['-', PREC.ADD, 'subtract'],
    ['*', PREC.MULTIPLY, 'multiply'],
    ['/', PREC.MULTIPLY, 'divide'],
    ['%', PREC.MULTIPLY, 'modulo'],
    ['||', PREC.LOGICAL_OR, 'logical_or'],
    ['&&', PREC.LOGICAL_AND, 'logical_and'],
    ['|', PREC.INCLUSIVE_OR, 'binary_or'],
    ['^', PREC.EXCLUSIVE_OR, 'binary_xor'],
    ['&', PREC.BITWISE_AND, 'binary_and'],
    ['==', PREC.EQUAL, 'equivalence'],
    ['!=', PREC.EQUAL, 'not_equal'],
    ['>', PREC.RELATIONAL, 'greater_than'],
    ['>=', PREC.RELATIONAL, 'greater_than_equal'],
    ['<=', PREC.RELATIONAL, 'less_than_equal'],
    ['<', PREC.RELATIONAL, 'less_than'],
    ['<<', PREC.SHIFT, 'left_shift'],
    ['>>', PREC.SHIFT, 'right_shift'],
];

const OPCODE_TABLE = getCommands('./opcodes.json')
const MNEMONICS_REGEX = arrayToRegex(OPCODE_TABLE.map(i => i.action))

module.exports = grammar({
    name: "gazm",

    rules: {
        source_file: $ =>

            repeat(choice(
                $.doc,
                $.comment,
                $.macro_def,
                $.struct_def,
                $._line
            )),

        scope: $ => seq(asRegex( 'scope' ), $._identifier),
        put: $ => seq(asRegex( 'put' ), $._expression),
        grabmem: $ => seq(asRegex( 'grabmem' ), field('addr', $._expression), ',', field('size', $._expression)),

        writebin: $ => seq(asRegex( 'writebin' ),
            field('file', $.string_literal), ',',
            field('addr', $._expression),
            optional(seq(',', field('len', $._expression)))
        ),

        incbin: $ => seq(asRegex('incbin'),
            field('file', $.string_literal),
            optional($._incbinargs)
        ),

        incbinref: $ => seq(asRegex( 'incbinref' ),
            field('file', $.string_literal),
            optional($._incbinargs)
        ),

        bsz: $ => seq(asRegex( 'bsz' ),
            field('count', $._expression),
            optional(seq(',', field('value', $._expression)))),

        fill: $ => seq(asRegex( 'fill' ),
            field('count', $._expression),
            seq(',', field('value', $._expression))),

        fdb: $ => seq(asRegex( 'fdb' ), commaSep1($._expression)),
        fcb: $ => seq(asRegex( 'fcb' ), commaSep1($._expression)),
        fcc: $ => seq(asRegex( 'fcc' ), commaSep1($.string_literal)),

        zmb: $ => seq(asRegex( 'zmb' ), $._expression),
        zmd: $ => seq(asRegex( 'zmd' ), $._expression),
        rmb: $ => seq(asRegex( 'rmb' ), $._expression),

        setdp: $ => seq(asRegex( 'setdp' ), $._expression),

        org: $ => seq(asRegex( 'org' ), $._expression),

        exec_addr: $ => seq(asRegex( 'exec_addr' ), $._expression),
        include: $ => seq(asRegex( 'include' ), $.string_literal),

        _incbinargs: $ =>
            choice(
                seq(',', field('len', $._expression)),
                seq(',', field('offset', $._expression), ',', field('len', $._expression))
            ),

        _command: $ => choice(
            $.scope,
            $.grabmem,
            $.put,
            $.writebin,
            $.incbin,
            $.incbinref,
            $.setdp,
            $.org,
            $.include,
            $.exec_addr,
            $.bsz,
            $.fill,
            $.fdb,
            $.fcb,
            $.fcc,
            $.zmb,
            $.zmd,
            $.rmb,
        ),

        struct_def: $ => seq("struct", $._identifier, '{', commaSep($.struct_elem), optional(','), '}'),
        struct_elem: $ => seq($._identifier, ':', $.elem_type),
        elem_type: $ => seq(choice('byte', 'word', 'dword', 'qword'), optional($.array)),
        array: $ => seq('[', $._expression, ']'),

        macro_def: $ => seq("macro", $._identifier, $.macro_args, $.macro_body),
        macro_args: $ => seq('(', commaSep($._identifier), ')'),
        macro_body: $ => seq('{', repeat(choice(
            $.comment,
            $.macro_def,
            $.struct_def,
            $._line
        )), '}'),

        doc: $ => token(seq(';;;', /(\\(.|\r?\n)|[^\\\n])*/)),

        // http://stackoverflow.com/questions/13014947/regex-to-match-a-c-style-multiline-comment/36328890#36328890
        comment: $ => token(choice(
            seq(';', /(\\(.|\r?\n)|[^\\\n])*/),
            seq('//', /(\\(.|\r?\n)|[^\\\n])*/),
            seq(
                '/*',
                /[^*]*\*+([^/*][^*]*\*+)*/,
                '/'
            )
        )),

        free_comment: $ => seq(
            '/*',
            /[^*]*\*+([^/*][^*]*\*+)*/,
            '/'
        ),

        expr_list1: $ => commaSep1($._expression),
        expr_list: $ => commaSep($._expression),


        _line: $ => seq(choice(
                $.equate,
                $.macro,
                $._command,
                $._command_label,
                $.opcode,
                $._opcode_label,
                $._identifier,
            ),
            choice(optional($.doc),optional($.comment)),
            $._line_break),

        _opcode_label: $ => seq(
            $._identifier,
            $.opcode,
        ),

        _command_label: $ => seq(
            $._identifier,
            $._command,
        ),

        pc_expr: $ => '*',

        binary_expression: $ => {
            return choice(...BIN_OPS.map(([operator, precedence, name]) => {
                return prec.left(precedence,
                    seq(
                        field('left', $._expression),
                        field('operator', operator),
                        field('right', $._expression)
                    )
                )
            }));
        },

        unary_expression: $ => prec.left(PREC.UNARY, seq(
            field('operator', choice('!', '~', '-', '+')),
            field('argument', $._expression)
        )),

        escape_sequence: $ => token(prec(1, seq(
            '\\',
            choice(
                /[^xuu]/,
                /\d{2,3}/,
                /x[0-9a-fa-f]{2,}/,
                /u[0-9a-fa-f]{4}/,
                /u[0-9a-fa-f]{8}/
            )
        ))),

        string_literal: $ => seq(
            '"',
            repeat(choice(
                token.immediate(prec(1, /[^\\"\n]+/)),
                $.escape_sequence
            )),
            '"',
        ),

        char_literal: $ => seq(
            '\'',
            choice(
                $.escape_sequence,
                token.immediate(/[^\n']/)
            ),
            '\''
        ),

        _expression: $ => choice(
            $.binary_expression,
            $.unary_expression,
            $._identifier,
            $._number_literal,
            $.char_literal,
            $.pc_expr,
            $.parenthesized_expression,
        ),

        parenthesized_expression: $ => seq(
            '(', $._expression, ')'
        ),

        _line_break: $ => /\n|\r\n/,


        macro: $ => seq($._identifier, '(', commaSep($._expression), ')'),

        reg_list_mnemonics: $ => {
            return arrayToRegex(['psh', 'exg']);
        },

        _regsets: $ => seq(alias($.regset_mnemonics, $.mnemonic), $.reg_set),
        regset_mnemonics: $ => mnemonicRegex('RegisterSet'),
        reg_set: $ => commaSep1($._reg),

        _xfers: $ => seq(alias($.xfer_mnemonics, $.mnemonic), $.reg_xfer),
        xfer_mnemonics: $ => mnemonicRegex('RegisterPair'),
        reg_xfer: $ => seq($._reg, ',', $._reg),

        _arg: $ => choice($.immediate, $.extended, $.direct_page, $._indexed),

        immediate: $ => seq('#', $._expression),
        extended: $ => $._expression,
        direct_page: $ => seq('>', $._expression),

        // equate
        equate: $ => seq($._identifier, asRegex('equ'), $._expression),

        // Regs
        a: $ => /A|a/,
        b: $ => /B|b/,
        d: $ => /D|d/,
        x: $ => /X|x/,
        y: $ => /Y|y/,
        u: $ => /U|u/,
        s: $ => /S|s/,
        pc: $ => /[pP][cC]/,
        pcr: $ => /[pP][cC][rR]/,
        cc: $ => /[cC][cC]/,
        dp: $ => /[dD][pP]/,

        _index_reg: $ => choice($.s, $.u, $.x, $.y),

        _reg: $ => choice(
            $.a,
            $.b,
            $.d,
            $.x,
            $.y,
            $.u,
            $.s,
            $.pc,
            $.pcr,
            $.cc,
            $.dp,
        ),

        constant_offset: $ => seq($._expression, ',', $._index_reg),
        pc_offset: $ => seq($._expression, ',', 'pc'),
        pc_offset_rel: $ => seq($._expression, ',', 'pcr'),
        pre_dec: $ => seq(',', '-', $._index_reg),
        pre_dec_dec: $ => seq(',', '--', $._index_reg),
        post_inc: $ => seq(',', $._index_reg, '+'),
        post_inc_inc: $ => seq(',', $._index_reg, '++'),
        add_a: $ => seq('a', ',', $._index_reg),
        add_b: $ => seq('b', ',', $._index_reg),
        add_d: $ => seq('d', ',', $._index_reg),
        zero_index: $ => seq(',', $._index_reg),

        _indexed: $ => choice($.indirect, $._indexed_direct),

        _indexed_direct: $ => choice(
            $.constant_offset,
            $.pc_offset,
            $.pc_offset_rel,
            $.pre_dec,
            $.pre_dec_dec,
            $.post_inc,
            $.post_inc_inc,
            $.add_a,
            $.add_b,
            $.add_d,
            $.zero_index,
        ),

        indirect: $ => seq('[', choice(
            $.constant_offset,
            $.pc_offset,
            $.pc_offset_rel,
            $.pre_dec_dec,
            $.post_inc_inc,
            $.add_a,
            $.add_b,
            $.add_d,
            $.zero_index,
            $._expression,
        ), ']'),

        mnemonic: $ => prec(1, MNEMONICS_REGEX),

        opcode: $ => choice($._xfers, $._regsets, $.mnemonic, $._opcode_arg, ),

        _opcode_arg: $ => seq(
            $.mnemonic,
            $._arg),

        _identifier: $ => choice($.local_label, $.label),

        label: $ => $._global_label,
        local_label: $ => seq(/[@!]/, $._global_label),
        _global_label: $ => /[a-zA-Z_.][a-zA-Z0-9_.]*/,


        // comment: $ => choice(
        //     /;(.*)/,
        //     /\/\/(.*)/
        // ),

        // numbers
        bin_num: $ => mkDigits(choice("%", "0b"), /[01]/),
        hex_num: $ => mkDigits(choice("$", "0x"), /[0-9a-fA-F]/),
        dec_num: $ => token(/[0-9]+[_0-9]*/),
        _number_literal: $ => choice($.hex_num, $.bin_num, $.dec_num),

    },
})


function commaSep(rule) {
    return optional(commaSep1(rule))
}

function commaSep1(rule) {
    return seq(rule, repeat(seq(',', rule)))
}

function mkDigits(pfix, digits) {
    const sep = '_';
    return token(seq(pfix, repeat1(digits), repeat(seq(sep, digits))))
}


function trailingCommaSep1(rule) {
    return seq(",", commaSep1(rule))
}

function strOptExprs($, name) {
    return seq($.string_literal, optional(seq(",", commaSep1($._expression))))
}


function loadJSON(fileName) {
    return fs.readFileSync(fileName, 'utf8');
}

function getCommands(fileName) {
    var ops = loadJSON(fileName);
    return JSON.parse(ops).instructions;
}

function arrayToRegex(arr) {
    var lower = arr.join('|').toLowerCase();
    var upper = arr.join('|').toUpperCase();
    var patterText = `${lower}|${upper}`;
    var pattern = new RegExp(patterText);
    return pattern
}

function asRegex(txt) {
    var lower = txt.toLowerCase();
    var upper = txt.toUpperCase();
    var patterText = `${lower}|${upper}`;
    var pattern = new RegExp(patterText);
    return pattern
}

function mnemonicRegex(addr_mode) {
    let opcodes = OPCODE_TABLE.filter(i => i.addr_mode == addr_mode).map(i => i.action)
    return arrayToRegex(opcodes)
}

