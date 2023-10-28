#include <tree_sitter/parser.h>

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#ifdef _MSC_VER
#pragma optimize("", off)
#elif defined(__clang__)
#pragma clang optimize off
#elif defined(__GNUC__)
#pragma GCC optimize ("O0")
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 318
#define LARGE_STATE_COUNT 9
#define SYMBOL_COUNT 185
#define ALIAS_COUNT 0
#define TOKEN_COUNT 99
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 11
#define MAX_ALIAS_SEQUENCE_LENGTH 7
#define PRODUCTION_ID_COUNT 12

enum {
  aux_sym_scope_token1 = 1,
  aux_sym_put_token1 = 2,
  aux_sym_grabmem_token1 = 3,
  anon_sym_COMMA = 4,
  aux_sym_writebin_token1 = 5,
  aux_sym_incbin_token1 = 6,
  aux_sym_importer_token1 = 7,
  aux_sym_incbinref_token1 = 8,
  aux_sym_bsz_token1 = 9,
  aux_sym_fill_token1 = 10,
  aux_sym_fdb_token1 = 11,
  aux_sym_fcb_token1 = 12,
  aux_sym_fcc_token1 = 13,
  aux_sym_zmb_token1 = 14,
  aux_sym_zmd_token1 = 15,
  aux_sym_rmb_token1 = 16,
  aux_sym_setdp_token1 = 17,
  aux_sym_org_token1 = 18,
  aux_sym_exec_addr_token1 = 19,
  aux_sym_include_token1 = 20,
  anon_sym_struct = 21,
  anon_sym_LBRACE = 22,
  anon_sym_RBRACE = 23,
  anon_sym_COLON = 24,
  anon_sym_byte = 25,
  anon_sym_word = 26,
  anon_sym_dword = 27,
  anon_sym_qword = 28,
  anon_sym_LBRACK = 29,
  anon_sym_RBRACK = 30,
  anon_sym_macro = 31,
  anon_sym_LPAREN = 32,
  anon_sym_RPAREN = 33,
  aux_sym_doc_text_token1 = 34,
  anon_sym_SEMI_SEMI_SEMI = 35,
  sym_long_doc_text = 36,
  anon_sym_LBRACK_LBRACK = 37,
  anon_sym_RBRACK_RBRACK = 38,
  anon_sym_SEMI = 39,
  anon_sym_SLASH_SLASH = 40,
  anon_sym_SLASH_STAR = 41,
  aux_sym_comment_token1 = 42,
  anon_sym_SLASH = 43,
  anon_sym_STAR = 44,
  anon_sym_PLUS = 45,
  anon_sym_DASH = 46,
  anon_sym_PERCENT = 47,
  anon_sym_PIPE_PIPE = 48,
  anon_sym_AMP_AMP = 49,
  anon_sym_PIPE = 50,
  anon_sym_CARET = 51,
  anon_sym_AMP = 52,
  anon_sym_EQ_EQ = 53,
  anon_sym_BANG_EQ = 54,
  anon_sym_GT = 55,
  anon_sym_GT_EQ = 56,
  anon_sym_LT_EQ = 57,
  anon_sym_LT = 58,
  anon_sym_LT_LT = 59,
  anon_sym_GT_GT = 60,
  anon_sym_BANG = 61,
  anon_sym_TILDE = 62,
  sym_escape_sequence = 63,
  anon_sym_DQUOTE = 64,
  aux_sym_string_literal_token1 = 65,
  anon_sym_SQUOTE = 66,
  aux_sym_char_literal_token1 = 67,
  sym__line_break = 68,
  sym_reg_list_mnemonics = 69,
  sym_regset_mnemonics = 70,
  sym_xfer_mnemonics = 71,
  anon_sym_POUND = 72,
  aux_sym_equate_token1 = 73,
  sym_a = 74,
  sym_b = 75,
  sym_d = 76,
  sym_x = 77,
  sym_y = 78,
  sym_u = 79,
  sym_s = 80,
  sym_pc = 81,
  sym_pcr = 82,
  sym_cc = 83,
  sym_dp = 84,
  anon_sym_pc = 85,
  anon_sym_pcr = 86,
  anon_sym_DASH_DASH = 87,
  anon_sym_PLUS_PLUS = 88,
  anon_sym_a = 89,
  anon_sym_b = 90,
  anon_sym_d = 91,
  aux_sym_mnemonic_token1 = 92,
  anon_sym_COLON_COLON = 93,
  aux_sym_local_label_token1 = 94,
  sym__global_label = 95,
  sym_bin_num = 96,
  sym_hex_num = 97,
  sym_dec_num = 98,
  sym_source_file = 99,
  sym_scope = 100,
  sym_put = 101,
  sym_grabmem = 102,
  sym_writebin = 103,
  sym_incbin = 104,
  sym_importer = 105,
  sym_incbinref = 106,
  sym_bsz = 107,
  sym_fill = 108,
  sym_fdb = 109,
  sym_fcb = 110,
  sym_fcc = 111,
  sym_zmb = 112,
  sym_zmd = 113,
  sym_rmb = 114,
  sym_setdp = 115,
  sym_org = 116,
  sym_exec_addr = 117,
  sym_include = 118,
  sym__incbinargs = 119,
  sym__command = 120,
  sym_struct_def = 121,
  sym_struct_elem = 122,
  sym_elem_type = 123,
  sym_array = 124,
  sym_macro_def = 125,
  sym_macro_args = 126,
  sym_macro_body = 127,
  sym_doc_text = 128,
  sym_doc = 129,
  sym_long_doc = 130,
  sym_comment = 131,
  sym__line = 132,
  sym__opcode_label = 133,
  sym__command_label = 134,
  sym_pc_expr = 135,
  sym_binary_expression = 136,
  sym_unary_expression = 137,
  sym_string_literal = 138,
  sym_char_literal = 139,
  sym__expression = 140,
  sym_parenthesized_expression = 141,
  sym_macro = 142,
  sym__regsets = 143,
  sym_reg_set = 144,
  sym__xfers = 145,
  sym_reg_xfer = 146,
  sym_operand = 147,
  sym_immediate = 148,
  sym_extended = 149,
  sym_direct_page = 150,
  sym_equate = 151,
  sym__index_reg = 152,
  sym__reg = 153,
  sym_constant_offset = 154,
  sym_pc_offset = 155,
  sym_pc_offset_rel = 156,
  sym_pre_dec = 157,
  sym_pre_dec_dec = 158,
  sym_post_inc = 159,
  sym_post_inc_inc = 160,
  sym_add_a = 161,
  sym_add_b = 162,
  sym_add_d = 163,
  sym_zero_index = 164,
  sym__indexed = 165,
  sym__indexed_direct = 166,
  sym_indirect = 167,
  sym_mnemonic = 168,
  sym_opcode = 169,
  sym__opcode_arg = 170,
  sym__identifier = 171,
  sym_global_scoped_id = 172,
  sym_label = 173,
  sym_local_label = 174,
  sym__number_literal = 175,
  aux_sym_source_file_repeat1 = 176,
  aux_sym_fdb_repeat1 = 177,
  aux_sym_fcc_repeat1 = 178,
  aux_sym_struct_def_repeat1 = 179,
  aux_sym_macro_args_repeat1 = 180,
  aux_sym_macro_body_repeat1 = 181,
  aux_sym_string_literal_repeat1 = 182,
  aux_sym_reg_set_repeat1 = 183,
  aux_sym_global_scoped_id_repeat1 = 184,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [aux_sym_scope_token1] = "scope_token1",
  [aux_sym_put_token1] = "put_token1",
  [aux_sym_grabmem_token1] = "grabmem_token1",
  [anon_sym_COMMA] = ",",
  [aux_sym_writebin_token1] = "writebin_token1",
  [aux_sym_incbin_token1] = "incbin_token1",
  [aux_sym_importer_token1] = "importer_token1",
  [aux_sym_incbinref_token1] = "incbinref_token1",
  [aux_sym_bsz_token1] = "bsz_token1",
  [aux_sym_fill_token1] = "fill_token1",
  [aux_sym_fdb_token1] = "fdb_token1",
  [aux_sym_fcb_token1] = "fcb_token1",
  [aux_sym_fcc_token1] = "fcc_token1",
  [aux_sym_zmb_token1] = "zmb_token1",
  [aux_sym_zmd_token1] = "zmd_token1",
  [aux_sym_rmb_token1] = "rmb_token1",
  [aux_sym_setdp_token1] = "setdp_token1",
  [aux_sym_org_token1] = "org_token1",
  [aux_sym_exec_addr_token1] = "exec_addr_token1",
  [aux_sym_include_token1] = "include_token1",
  [anon_sym_struct] = "struct",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_COLON] = ":",
  [anon_sym_byte] = "byte",
  [anon_sym_word] = "word",
  [anon_sym_dword] = "dword",
  [anon_sym_qword] = "qword",
  [anon_sym_LBRACK] = "[",
  [anon_sym_RBRACK] = "]",
  [anon_sym_macro] = "macro",
  [anon_sym_LPAREN] = "(",
  [anon_sym_RPAREN] = ")",
  [aux_sym_doc_text_token1] = "doc_text_token1",
  [anon_sym_SEMI_SEMI_SEMI] = ";;;",
  [sym_long_doc_text] = "long_doc_text",
  [anon_sym_LBRACK_LBRACK] = "[[",
  [anon_sym_RBRACK_RBRACK] = "]]",
  [anon_sym_SEMI] = ";",
  [anon_sym_SLASH_SLASH] = "//",
  [anon_sym_SLASH_STAR] = "/*",
  [aux_sym_comment_token1] = "comment_token1",
  [anon_sym_SLASH] = "/",
  [anon_sym_STAR] = "*",
  [anon_sym_PLUS] = "+",
  [anon_sym_DASH] = "-",
  [anon_sym_PERCENT] = "%",
  [anon_sym_PIPE_PIPE] = "||",
  [anon_sym_AMP_AMP] = "&&",
  [anon_sym_PIPE] = "|",
  [anon_sym_CARET] = "^",
  [anon_sym_AMP] = "&",
  [anon_sym_EQ_EQ] = "==",
  [anon_sym_BANG_EQ] = "!=",
  [anon_sym_GT] = ">",
  [anon_sym_GT_EQ] = ">=",
  [anon_sym_LT_EQ] = "<=",
  [anon_sym_LT] = "<",
  [anon_sym_LT_LT] = "<<",
  [anon_sym_GT_GT] = ">>",
  [anon_sym_BANG] = "!",
  [anon_sym_TILDE] = "~",
  [sym_escape_sequence] = "escape_sequence",
  [anon_sym_DQUOTE] = "\"",
  [aux_sym_string_literal_token1] = "string_literal_token1",
  [anon_sym_SQUOTE] = "'",
  [aux_sym_char_literal_token1] = "char_literal_token1",
  [sym__line_break] = "_line_break",
  [sym_reg_list_mnemonics] = "reg_list_mnemonics",
  [sym_regset_mnemonics] = "mnemonic",
  [sym_xfer_mnemonics] = "mnemonic",
  [anon_sym_POUND] = "#",
  [aux_sym_equate_token1] = "equate_token1",
  [sym_a] = "a",
  [sym_b] = "b",
  [sym_d] = "d",
  [sym_x] = "x",
  [sym_y] = "y",
  [sym_u] = "u",
  [sym_s] = "s",
  [sym_pc] = "pc",
  [sym_pcr] = "pcr",
  [sym_cc] = "cc",
  [sym_dp] = "dp",
  [anon_sym_pc] = "pc",
  [anon_sym_pcr] = "pcr",
  [anon_sym_DASH_DASH] = "--",
  [anon_sym_PLUS_PLUS] = "++",
  [anon_sym_a] = "a",
  [anon_sym_b] = "b",
  [anon_sym_d] = "d",
  [aux_sym_mnemonic_token1] = "mnemonic_token1",
  [anon_sym_COLON_COLON] = "::",
  [aux_sym_local_label_token1] = "local_label_token1",
  [sym__global_label] = "_global_label",
  [sym_bin_num] = "bin_num",
  [sym_hex_num] = "hex_num",
  [sym_dec_num] = "dec_num",
  [sym_source_file] = "source_file",
  [sym_scope] = "scope",
  [sym_put] = "put",
  [sym_grabmem] = "grabmem",
  [sym_writebin] = "writebin",
  [sym_incbin] = "incbin",
  [sym_importer] = "importer",
  [sym_incbinref] = "incbinref",
  [sym_bsz] = "bsz",
  [sym_fill] = "fill",
  [sym_fdb] = "fdb",
  [sym_fcb] = "fcb",
  [sym_fcc] = "fcc",
  [sym_zmb] = "zmb",
  [sym_zmd] = "zmd",
  [sym_rmb] = "rmb",
  [sym_setdp] = "setdp",
  [sym_org] = "org",
  [sym_exec_addr] = "exec_addr",
  [sym_include] = "include",
  [sym__incbinargs] = "_incbinargs",
  [sym__command] = "_command",
  [sym_struct_def] = "struct_def",
  [sym_struct_elem] = "struct_elem",
  [sym_elem_type] = "elem_type",
  [sym_array] = "array",
  [sym_macro_def] = "macro_def",
  [sym_macro_args] = "macro_args",
  [sym_macro_body] = "macro_body",
  [sym_doc_text] = "doc_text",
  [sym_doc] = "doc",
  [sym_long_doc] = "long_doc",
  [sym_comment] = "comment",
  [sym__line] = "_line",
  [sym__opcode_label] = "_opcode_label",
  [sym__command_label] = "_command_label",
  [sym_pc_expr] = "pc_expr",
  [sym_binary_expression] = "binary_expression",
  [sym_unary_expression] = "unary_expression",
  [sym_string_literal] = "string_literal",
  [sym_char_literal] = "char_literal",
  [sym__expression] = "_expression",
  [sym_parenthesized_expression] = "parenthesized_expression",
  [sym_macro] = "macro",
  [sym__regsets] = "_regsets",
  [sym_reg_set] = "reg_set",
  [sym__xfers] = "_xfers",
  [sym_reg_xfer] = "reg_xfer",
  [sym_operand] = "operand",
  [sym_immediate] = "immediate",
  [sym_extended] = "extended",
  [sym_direct_page] = "direct_page",
  [sym_equate] = "equate",
  [sym__index_reg] = "_index_reg",
  [sym__reg] = "_reg",
  [sym_constant_offset] = "constant_offset",
  [sym_pc_offset] = "pc_offset",
  [sym_pc_offset_rel] = "pc_offset_rel",
  [sym_pre_dec] = "pre_dec",
  [sym_pre_dec_dec] = "pre_dec_dec",
  [sym_post_inc] = "post_inc",
  [sym_post_inc_inc] = "post_inc_inc",
  [sym_add_a] = "add_a",
  [sym_add_b] = "add_b",
  [sym_add_d] = "add_d",
  [sym_zero_index] = "zero_index",
  [sym__indexed] = "_indexed",
  [sym__indexed_direct] = "_indexed_direct",
  [sym_indirect] = "indirect",
  [sym_mnemonic] = "mnemonic",
  [sym_opcode] = "opcode",
  [sym__opcode_arg] = "_opcode_arg",
  [sym__identifier] = "_identifier",
  [sym_global_scoped_id] = "global_scoped_id",
  [sym_label] = "label",
  [sym_local_label] = "local_label",
  [sym__number_literal] = "_number_literal",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_fdb_repeat1] = "fdb_repeat1",
  [aux_sym_fcc_repeat1] = "fcc_repeat1",
  [aux_sym_struct_def_repeat1] = "struct_def_repeat1",
  [aux_sym_macro_args_repeat1] = "macro_args_repeat1",
  [aux_sym_macro_body_repeat1] = "macro_body_repeat1",
  [aux_sym_string_literal_repeat1] = "string_literal_repeat1",
  [aux_sym_reg_set_repeat1] = "reg_set_repeat1",
  [aux_sym_global_scoped_id_repeat1] = "global_scoped_id_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [aux_sym_scope_token1] = aux_sym_scope_token1,
  [aux_sym_put_token1] = aux_sym_put_token1,
  [aux_sym_grabmem_token1] = aux_sym_grabmem_token1,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [aux_sym_writebin_token1] = aux_sym_writebin_token1,
  [aux_sym_incbin_token1] = aux_sym_incbin_token1,
  [aux_sym_importer_token1] = aux_sym_importer_token1,
  [aux_sym_incbinref_token1] = aux_sym_incbinref_token1,
  [aux_sym_bsz_token1] = aux_sym_bsz_token1,
  [aux_sym_fill_token1] = aux_sym_fill_token1,
  [aux_sym_fdb_token1] = aux_sym_fdb_token1,
  [aux_sym_fcb_token1] = aux_sym_fcb_token1,
  [aux_sym_fcc_token1] = aux_sym_fcc_token1,
  [aux_sym_zmb_token1] = aux_sym_zmb_token1,
  [aux_sym_zmd_token1] = aux_sym_zmd_token1,
  [aux_sym_rmb_token1] = aux_sym_rmb_token1,
  [aux_sym_setdp_token1] = aux_sym_setdp_token1,
  [aux_sym_org_token1] = aux_sym_org_token1,
  [aux_sym_exec_addr_token1] = aux_sym_exec_addr_token1,
  [aux_sym_include_token1] = aux_sym_include_token1,
  [anon_sym_struct] = anon_sym_struct,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_byte] = anon_sym_byte,
  [anon_sym_word] = anon_sym_word,
  [anon_sym_dword] = anon_sym_dword,
  [anon_sym_qword] = anon_sym_qword,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [anon_sym_macro] = anon_sym_macro,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [aux_sym_doc_text_token1] = aux_sym_doc_text_token1,
  [anon_sym_SEMI_SEMI_SEMI] = anon_sym_SEMI_SEMI_SEMI,
  [sym_long_doc_text] = sym_long_doc_text,
  [anon_sym_LBRACK_LBRACK] = anon_sym_LBRACK_LBRACK,
  [anon_sym_RBRACK_RBRACK] = anon_sym_RBRACK_RBRACK,
  [anon_sym_SEMI] = anon_sym_SEMI,
  [anon_sym_SLASH_SLASH] = anon_sym_SLASH_SLASH,
  [anon_sym_SLASH_STAR] = anon_sym_SLASH_STAR,
  [aux_sym_comment_token1] = aux_sym_comment_token1,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [anon_sym_STAR] = anon_sym_STAR,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_PERCENT] = anon_sym_PERCENT,
  [anon_sym_PIPE_PIPE] = anon_sym_PIPE_PIPE,
  [anon_sym_AMP_AMP] = anon_sym_AMP_AMP,
  [anon_sym_PIPE] = anon_sym_PIPE,
  [anon_sym_CARET] = anon_sym_CARET,
  [anon_sym_AMP] = anon_sym_AMP,
  [anon_sym_EQ_EQ] = anon_sym_EQ_EQ,
  [anon_sym_BANG_EQ] = anon_sym_BANG_EQ,
  [anon_sym_GT] = anon_sym_GT,
  [anon_sym_GT_EQ] = anon_sym_GT_EQ,
  [anon_sym_LT_EQ] = anon_sym_LT_EQ,
  [anon_sym_LT] = anon_sym_LT,
  [anon_sym_LT_LT] = anon_sym_LT_LT,
  [anon_sym_GT_GT] = anon_sym_GT_GT,
  [anon_sym_BANG] = anon_sym_BANG,
  [anon_sym_TILDE] = anon_sym_TILDE,
  [sym_escape_sequence] = sym_escape_sequence,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [aux_sym_string_literal_token1] = aux_sym_string_literal_token1,
  [anon_sym_SQUOTE] = anon_sym_SQUOTE,
  [aux_sym_char_literal_token1] = aux_sym_char_literal_token1,
  [sym__line_break] = sym__line_break,
  [sym_reg_list_mnemonics] = sym_reg_list_mnemonics,
  [sym_regset_mnemonics] = sym_mnemonic,
  [sym_xfer_mnemonics] = sym_mnemonic,
  [anon_sym_POUND] = anon_sym_POUND,
  [aux_sym_equate_token1] = aux_sym_equate_token1,
  [sym_a] = sym_a,
  [sym_b] = sym_b,
  [sym_d] = sym_d,
  [sym_x] = sym_x,
  [sym_y] = sym_y,
  [sym_u] = sym_u,
  [sym_s] = sym_s,
  [sym_pc] = sym_pc,
  [sym_pcr] = sym_pcr,
  [sym_cc] = sym_cc,
  [sym_dp] = sym_dp,
  [anon_sym_pc] = anon_sym_pc,
  [anon_sym_pcr] = anon_sym_pcr,
  [anon_sym_DASH_DASH] = anon_sym_DASH_DASH,
  [anon_sym_PLUS_PLUS] = anon_sym_PLUS_PLUS,
  [anon_sym_a] = anon_sym_a,
  [anon_sym_b] = anon_sym_b,
  [anon_sym_d] = anon_sym_d,
  [aux_sym_mnemonic_token1] = aux_sym_mnemonic_token1,
  [anon_sym_COLON_COLON] = anon_sym_COLON_COLON,
  [aux_sym_local_label_token1] = aux_sym_local_label_token1,
  [sym__global_label] = sym__global_label,
  [sym_bin_num] = sym_bin_num,
  [sym_hex_num] = sym_hex_num,
  [sym_dec_num] = sym_dec_num,
  [sym_source_file] = sym_source_file,
  [sym_scope] = sym_scope,
  [sym_put] = sym_put,
  [sym_grabmem] = sym_grabmem,
  [sym_writebin] = sym_writebin,
  [sym_incbin] = sym_incbin,
  [sym_importer] = sym_importer,
  [sym_incbinref] = sym_incbinref,
  [sym_bsz] = sym_bsz,
  [sym_fill] = sym_fill,
  [sym_fdb] = sym_fdb,
  [sym_fcb] = sym_fcb,
  [sym_fcc] = sym_fcc,
  [sym_zmb] = sym_zmb,
  [sym_zmd] = sym_zmd,
  [sym_rmb] = sym_rmb,
  [sym_setdp] = sym_setdp,
  [sym_org] = sym_org,
  [sym_exec_addr] = sym_exec_addr,
  [sym_include] = sym_include,
  [sym__incbinargs] = sym__incbinargs,
  [sym__command] = sym__command,
  [sym_struct_def] = sym_struct_def,
  [sym_struct_elem] = sym_struct_elem,
  [sym_elem_type] = sym_elem_type,
  [sym_array] = sym_array,
  [sym_macro_def] = sym_macro_def,
  [sym_macro_args] = sym_macro_args,
  [sym_macro_body] = sym_macro_body,
  [sym_doc_text] = sym_doc_text,
  [sym_doc] = sym_doc,
  [sym_long_doc] = sym_long_doc,
  [sym_comment] = sym_comment,
  [sym__line] = sym__line,
  [sym__opcode_label] = sym__opcode_label,
  [sym__command_label] = sym__command_label,
  [sym_pc_expr] = sym_pc_expr,
  [sym_binary_expression] = sym_binary_expression,
  [sym_unary_expression] = sym_unary_expression,
  [sym_string_literal] = sym_string_literal,
  [sym_char_literal] = sym_char_literal,
  [sym__expression] = sym__expression,
  [sym_parenthesized_expression] = sym_parenthesized_expression,
  [sym_macro] = sym_macro,
  [sym__regsets] = sym__regsets,
  [sym_reg_set] = sym_reg_set,
  [sym__xfers] = sym__xfers,
  [sym_reg_xfer] = sym_reg_xfer,
  [sym_operand] = sym_operand,
  [sym_immediate] = sym_immediate,
  [sym_extended] = sym_extended,
  [sym_direct_page] = sym_direct_page,
  [sym_equate] = sym_equate,
  [sym__index_reg] = sym__index_reg,
  [sym__reg] = sym__reg,
  [sym_constant_offset] = sym_constant_offset,
  [sym_pc_offset] = sym_pc_offset,
  [sym_pc_offset_rel] = sym_pc_offset_rel,
  [sym_pre_dec] = sym_pre_dec,
  [sym_pre_dec_dec] = sym_pre_dec_dec,
  [sym_post_inc] = sym_post_inc,
  [sym_post_inc_inc] = sym_post_inc_inc,
  [sym_add_a] = sym_add_a,
  [sym_add_b] = sym_add_b,
  [sym_add_d] = sym_add_d,
  [sym_zero_index] = sym_zero_index,
  [sym__indexed] = sym__indexed,
  [sym__indexed_direct] = sym__indexed_direct,
  [sym_indirect] = sym_indirect,
  [sym_mnemonic] = sym_mnemonic,
  [sym_opcode] = sym_opcode,
  [sym__opcode_arg] = sym__opcode_arg,
  [sym__identifier] = sym__identifier,
  [sym_global_scoped_id] = sym_global_scoped_id,
  [sym_label] = sym_label,
  [sym_local_label] = sym_local_label,
  [sym__number_literal] = sym__number_literal,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_fdb_repeat1] = aux_sym_fdb_repeat1,
  [aux_sym_fcc_repeat1] = aux_sym_fcc_repeat1,
  [aux_sym_struct_def_repeat1] = aux_sym_struct_def_repeat1,
  [aux_sym_macro_args_repeat1] = aux_sym_macro_args_repeat1,
  [aux_sym_macro_body_repeat1] = aux_sym_macro_body_repeat1,
  [aux_sym_string_literal_repeat1] = aux_sym_string_literal_repeat1,
  [aux_sym_reg_set_repeat1] = aux_sym_reg_set_repeat1,
  [aux_sym_global_scoped_id_repeat1] = aux_sym_global_scoped_id_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [aux_sym_scope_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_put_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_grabmem_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_writebin_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_incbin_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_importer_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_incbinref_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_bsz_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fill_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fdb_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fcb_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fcc_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_zmb_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_zmd_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_rmb_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_setdp_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_org_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_exec_addr_token1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_include_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_struct] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_byte] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_word] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_dword] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_qword] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_macro] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_doc_text_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_SEMI_SEMI_SEMI] = {
    .visible = true,
    .named = false,
  },
  [sym_long_doc_text] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_LBRACK_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SEMI] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH_STAR] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_comment_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_STAR] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PERCENT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PIPE_PIPE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_AMP_AMP] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PIPE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_CARET] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_AMP] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_LT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_TILDE] = {
    .visible = true,
    .named = false,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_string_literal_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_SQUOTE] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_char_literal_token1] = {
    .visible = false,
    .named = false,
  },
  [sym__line_break] = {
    .visible = false,
    .named = true,
  },
  [sym_reg_list_mnemonics] = {
    .visible = true,
    .named = true,
  },
  [sym_regset_mnemonics] = {
    .visible = true,
    .named = true,
  },
  [sym_xfer_mnemonics] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_POUND] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_equate_token1] = {
    .visible = false,
    .named = false,
  },
  [sym_a] = {
    .visible = true,
    .named = true,
  },
  [sym_b] = {
    .visible = true,
    .named = true,
  },
  [sym_d] = {
    .visible = true,
    .named = true,
  },
  [sym_x] = {
    .visible = true,
    .named = true,
  },
  [sym_y] = {
    .visible = true,
    .named = true,
  },
  [sym_u] = {
    .visible = true,
    .named = true,
  },
  [sym_s] = {
    .visible = true,
    .named = true,
  },
  [sym_pc] = {
    .visible = true,
    .named = true,
  },
  [sym_pcr] = {
    .visible = true,
    .named = true,
  },
  [sym_cc] = {
    .visible = true,
    .named = true,
  },
  [sym_dp] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_pc] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_pcr] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PLUS_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_a] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_b] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_d] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_mnemonic_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_COLON_COLON] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_local_label_token1] = {
    .visible = false,
    .named = false,
  },
  [sym__global_label] = {
    .visible = false,
    .named = true,
  },
  [sym_bin_num] = {
    .visible = true,
    .named = true,
  },
  [sym_hex_num] = {
    .visible = true,
    .named = true,
  },
  [sym_dec_num] = {
    .visible = true,
    .named = true,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym_scope] = {
    .visible = true,
    .named = true,
  },
  [sym_put] = {
    .visible = true,
    .named = true,
  },
  [sym_grabmem] = {
    .visible = true,
    .named = true,
  },
  [sym_writebin] = {
    .visible = true,
    .named = true,
  },
  [sym_incbin] = {
    .visible = true,
    .named = true,
  },
  [sym_importer] = {
    .visible = true,
    .named = true,
  },
  [sym_incbinref] = {
    .visible = true,
    .named = true,
  },
  [sym_bsz] = {
    .visible = true,
    .named = true,
  },
  [sym_fill] = {
    .visible = true,
    .named = true,
  },
  [sym_fdb] = {
    .visible = true,
    .named = true,
  },
  [sym_fcb] = {
    .visible = true,
    .named = true,
  },
  [sym_fcc] = {
    .visible = true,
    .named = true,
  },
  [sym_zmb] = {
    .visible = true,
    .named = true,
  },
  [sym_zmd] = {
    .visible = true,
    .named = true,
  },
  [sym_rmb] = {
    .visible = true,
    .named = true,
  },
  [sym_setdp] = {
    .visible = true,
    .named = true,
  },
  [sym_org] = {
    .visible = true,
    .named = true,
  },
  [sym_exec_addr] = {
    .visible = true,
    .named = true,
  },
  [sym_include] = {
    .visible = true,
    .named = true,
  },
  [sym__incbinargs] = {
    .visible = false,
    .named = true,
  },
  [sym__command] = {
    .visible = false,
    .named = true,
  },
  [sym_struct_def] = {
    .visible = true,
    .named = true,
  },
  [sym_struct_elem] = {
    .visible = true,
    .named = true,
  },
  [sym_elem_type] = {
    .visible = true,
    .named = true,
  },
  [sym_array] = {
    .visible = true,
    .named = true,
  },
  [sym_macro_def] = {
    .visible = true,
    .named = true,
  },
  [sym_macro_args] = {
    .visible = true,
    .named = true,
  },
  [sym_macro_body] = {
    .visible = true,
    .named = true,
  },
  [sym_doc_text] = {
    .visible = true,
    .named = true,
  },
  [sym_doc] = {
    .visible = true,
    .named = true,
  },
  [sym_long_doc] = {
    .visible = true,
    .named = true,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [sym__line] = {
    .visible = false,
    .named = true,
  },
  [sym__opcode_label] = {
    .visible = false,
    .named = true,
  },
  [sym__command_label] = {
    .visible = false,
    .named = true,
  },
  [sym_pc_expr] = {
    .visible = true,
    .named = true,
  },
  [sym_binary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_unary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_string_literal] = {
    .visible = true,
    .named = true,
  },
  [sym_char_literal] = {
    .visible = true,
    .named = true,
  },
  [sym__expression] = {
    .visible = false,
    .named = true,
  },
  [sym_parenthesized_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_macro] = {
    .visible = true,
    .named = true,
  },
  [sym__regsets] = {
    .visible = false,
    .named = true,
  },
  [sym_reg_set] = {
    .visible = true,
    .named = true,
  },
  [sym__xfers] = {
    .visible = false,
    .named = true,
  },
  [sym_reg_xfer] = {
    .visible = true,
    .named = true,
  },
  [sym_operand] = {
    .visible = true,
    .named = true,
  },
  [sym_immediate] = {
    .visible = true,
    .named = true,
  },
  [sym_extended] = {
    .visible = true,
    .named = true,
  },
  [sym_direct_page] = {
    .visible = true,
    .named = true,
  },
  [sym_equate] = {
    .visible = true,
    .named = true,
  },
  [sym__index_reg] = {
    .visible = false,
    .named = true,
  },
  [sym__reg] = {
    .visible = false,
    .named = true,
  },
  [sym_constant_offset] = {
    .visible = true,
    .named = true,
  },
  [sym_pc_offset] = {
    .visible = true,
    .named = true,
  },
  [sym_pc_offset_rel] = {
    .visible = true,
    .named = true,
  },
  [sym_pre_dec] = {
    .visible = true,
    .named = true,
  },
  [sym_pre_dec_dec] = {
    .visible = true,
    .named = true,
  },
  [sym_post_inc] = {
    .visible = true,
    .named = true,
  },
  [sym_post_inc_inc] = {
    .visible = true,
    .named = true,
  },
  [sym_add_a] = {
    .visible = true,
    .named = true,
  },
  [sym_add_b] = {
    .visible = true,
    .named = true,
  },
  [sym_add_d] = {
    .visible = true,
    .named = true,
  },
  [sym_zero_index] = {
    .visible = true,
    .named = true,
  },
  [sym__indexed] = {
    .visible = false,
    .named = true,
  },
  [sym__indexed_direct] = {
    .visible = false,
    .named = true,
  },
  [sym_indirect] = {
    .visible = true,
    .named = true,
  },
  [sym_mnemonic] = {
    .visible = true,
    .named = true,
  },
  [sym_opcode] = {
    .visible = true,
    .named = true,
  },
  [sym__opcode_arg] = {
    .visible = false,
    .named = true,
  },
  [sym__identifier] = {
    .visible = false,
    .named = true,
  },
  [sym_global_scoped_id] = {
    .visible = true,
    .named = true,
  },
  [sym_label] = {
    .visible = true,
    .named = true,
  },
  [sym_local_label] = {
    .visible = true,
    .named = true,
  },
  [sym__number_literal] = {
    .visible = false,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fdb_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fcc_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_struct_def_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_macro_args_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_macro_body_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_literal_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_reg_set_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_global_scoped_id_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum {
  field_addr = 1,
  field_argument = 2,
  field_count = 3,
  field_file = 4,
  field_left = 5,
  field_len = 6,
  field_offset = 7,
  field_operator = 8,
  field_right = 9,
  field_size = 10,
  field_value = 11,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_addr] = "addr",
  [field_argument] = "argument",
  [field_count] = "count",
  [field_file] = "file",
  [field_left] = "left",
  [field_len] = "len",
  [field_offset] = "offset",
  [field_operator] = "operator",
  [field_right] = "right",
  [field_size] = "size",
  [field_value] = "value",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 1},
  [2] = {.index = 1, .length = 1},
  [3] = {.index = 2, .length = 2},
  [4] = {.index = 4, .length = 3},
  [5] = {.index = 7, .length = 3},
  [6] = {.index = 10, .length = 2},
  [7] = {.index = 12, .length = 2},
  [8] = {.index = 14, .length = 1},
  [9] = {.index = 15, .length = 2},
  [10] = {.index = 17, .length = 3},
  [11] = {.index = 20, .length = 2},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_file, 1},
  [1] =
    {field_count, 1},
  [2] =
    {field_argument, 1},
    {field_operator, 0},
  [4] =
    {field_file, 1},
    {field_len, 2, .inherited = true},
    {field_offset, 2, .inherited = true},
  [7] =
    {field_left, 0},
    {field_operator, 1},
    {field_right, 2},
  [10] =
    {field_addr, 1},
    {field_size, 3},
  [12] =
    {field_addr, 3},
    {field_file, 1},
  [14] =
    {field_len, 1},
  [15] =
    {field_count, 1},
    {field_value, 3},
  [17] =
    {field_addr, 3},
    {field_file, 1},
    {field_len, 5},
  [20] =
    {field_len, 3},
    {field_offset, 1},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 5,
  [6] = 6,
  [7] = 4,
  [8] = 5,
  [9] = 9,
  [10] = 9,
  [11] = 11,
  [12] = 12,
  [13] = 13,
  [14] = 14,
  [15] = 15,
  [16] = 16,
  [17] = 17,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 19,
  [30] = 23,
  [31] = 17,
  [32] = 21,
  [33] = 26,
  [34] = 25,
  [35] = 24,
  [36] = 27,
  [37] = 28,
  [38] = 18,
  [39] = 16,
  [40] = 40,
  [41] = 41,
  [42] = 42,
  [43] = 43,
  [44] = 44,
  [45] = 45,
  [46] = 46,
  [47] = 47,
  [48] = 48,
  [49] = 49,
  [50] = 50,
  [51] = 51,
  [52] = 52,
  [53] = 53,
  [54] = 12,
  [55] = 55,
  [56] = 56,
  [57] = 13,
  [58] = 58,
  [59] = 59,
  [60] = 60,
  [61] = 61,
  [62] = 62,
  [63] = 63,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 96,
  [99] = 99,
  [100] = 100,
  [101] = 101,
  [102] = 102,
  [103] = 103,
  [104] = 81,
  [105] = 105,
  [106] = 106,
  [107] = 107,
  [108] = 108,
  [109] = 109,
  [110] = 110,
  [111] = 111,
  [112] = 112,
  [113] = 99,
  [114] = 100,
  [115] = 101,
  [116] = 102,
  [117] = 117,
  [118] = 118,
  [119] = 119,
  [120] = 83,
  [121] = 90,
  [122] = 103,
  [123] = 123,
  [124] = 105,
  [125] = 106,
  [126] = 107,
  [127] = 119,
  [128] = 128,
  [129] = 129,
  [130] = 42,
  [131] = 45,
  [132] = 46,
  [133] = 61,
  [134] = 55,
  [135] = 44,
  [136] = 47,
  [137] = 48,
  [138] = 49,
  [139] = 50,
  [140] = 51,
  [141] = 43,
  [142] = 63,
  [143] = 56,
  [144] = 60,
  [145] = 145,
  [146] = 146,
  [147] = 147,
  [148] = 148,
  [149] = 147,
  [150] = 150,
  [151] = 151,
  [152] = 152,
  [153] = 153,
  [154] = 154,
  [155] = 155,
  [156] = 155,
  [157] = 157,
  [158] = 158,
  [159] = 159,
  [160] = 160,
  [161] = 161,
  [162] = 162,
  [163] = 163,
  [164] = 164,
  [165] = 165,
  [166] = 166,
  [167] = 157,
  [168] = 168,
  [169] = 168,
  [170] = 165,
  [171] = 171,
  [172] = 172,
  [173] = 173,
  [174] = 174,
  [175] = 173,
  [176] = 176,
  [177] = 177,
  [178] = 178,
  [179] = 179,
  [180] = 180,
  [181] = 181,
  [182] = 182,
  [183] = 183,
  [184] = 184,
  [185] = 185,
  [186] = 186,
  [187] = 187,
  [188] = 188,
  [189] = 189,
  [190] = 190,
  [191] = 191,
  [192] = 192,
  [193] = 193,
  [194] = 194,
  [195] = 195,
  [196] = 196,
  [197] = 197,
  [198] = 198,
  [199] = 199,
  [200] = 200,
  [201] = 201,
  [202] = 202,
  [203] = 203,
  [204] = 204,
  [205] = 205,
  [206] = 206,
  [207] = 207,
  [208] = 201,
  [209] = 209,
  [210] = 210,
  [211] = 211,
  [212] = 212,
  [213] = 204,
  [214] = 188,
  [215] = 215,
  [216] = 216,
  [217] = 203,
  [218] = 218,
  [219] = 219,
  [220] = 220,
  [221] = 221,
  [222] = 218,
  [223] = 223,
  [224] = 224,
  [225] = 225,
  [226] = 226,
  [227] = 227,
  [228] = 228,
  [229] = 229,
  [230] = 202,
  [231] = 231,
  [232] = 232,
  [233] = 233,
  [234] = 234,
  [235] = 232,
  [236] = 231,
  [237] = 237,
  [238] = 238,
  [239] = 166,
  [240] = 240,
  [241] = 241,
  [242] = 242,
  [243] = 243,
  [244] = 244,
  [245] = 238,
  [246] = 246,
  [247] = 246,
  [248] = 248,
  [249] = 249,
  [250] = 248,
  [251] = 251,
  [252] = 252,
  [253] = 253,
  [254] = 251,
  [255] = 255,
  [256] = 256,
  [257] = 257,
  [258] = 258,
  [259] = 252,
  [260] = 260,
  [261] = 261,
  [262] = 262,
  [263] = 263,
  [264] = 249,
  [265] = 265,
  [266] = 266,
  [267] = 267,
  [268] = 268,
  [269] = 269,
  [270] = 225,
  [271] = 221,
  [272] = 20,
  [273] = 216,
  [274] = 190,
  [275] = 191,
  [276] = 192,
  [277] = 193,
  [278] = 16,
  [279] = 180,
  [280] = 280,
  [281] = 281,
  [282] = 282,
  [283] = 283,
  [284] = 284,
  [285] = 285,
  [286] = 286,
  [287] = 281,
  [288] = 288,
  [289] = 19,
  [290] = 290,
  [291] = 284,
  [292] = 210,
  [293] = 15,
  [294] = 294,
  [295] = 295,
  [296] = 296,
  [297] = 280,
  [298] = 298,
  [299] = 187,
  [300] = 300,
  [301] = 301,
  [302] = 301,
  [303] = 295,
  [304] = 282,
  [305] = 305,
  [306] = 306,
  [307] = 268,
  [308] = 286,
  [309] = 284,
  [310] = 306,
  [311] = 311,
  [312] = 312,
  [313] = 288,
  [314] = 314,
  [315] = 269,
  [316] = 286,
  [317] = 269,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(282);
      if (lookahead == '!') ADVANCE(378);
      if (lookahead == '"') ADVANCE(384);
      if (lookahead == '#') ADVANCE(401);
      if (lookahead == '$') ADVANCE(278);
      if (lookahead == '%') ADVANCE(362);
      if (lookahead == '&') ADVANCE(367);
      if (lookahead == '\'') ADVANCE(387);
      if (lookahead == '(') ADVANCE(340);
      if (lookahead == ')') ADVANCE(341);
      if (lookahead == '*') ADVANCE(357);
      if (lookahead == '+') ADVANCE(359);
      if (lookahead == ',') ADVANCE(289);
      if (lookahead == '-') ADVANCE(361);
      if (lookahead == '/') ADVANCE(356);
      if (lookahead == '0') ADVANCE(667);
      if (lookahead == ':') ADVANCE(329);
      if (lookahead == ';') ADVANCE(351);
      if (lookahead == '<') ADVANCE(374);
      if (lookahead == '=') ADVANCE(20);
      if (lookahead == '>') ADVANCE(371);
      if (lookahead == '@') ADVANCE(447);
      if (lookahead == 'A') ADVANCE(404);
      if (lookahead == 'B') ADVANCE(406);
      if (lookahead == 'C') ADVANCE(76);
      if (lookahead == 'D') ADVANCE(407);
      if (lookahead == 'E') ADVANCE(88);
      if (lookahead == 'F') ADVANCE(39);
      if (lookahead == 'G') ADVANCE(105);
      if (lookahead == 'I') ADVANCE(80);
      if (lookahead == 'J') ADVANCE(82);
      if (lookahead == 'L') ADVANCE(28);
      if (lookahead == 'M') ADVANCE(114);
      if (lookahead == 'N') ADVANCE(52);
      if (lookahead == 'O') ADVANCE(97);
      if (lookahead == 'P') ADVANCE(107);
      if (lookahead == 'R') ADVANCE(81);
      if (lookahead == 'S') ADVANCE(413);
      if (lookahead == 'T') ADVANCE(62);
      if (lookahead == 'U' ||
          lookahead == 'u') ADVANCE(411);
      if (lookahead == 'W') ADVANCE(104);
      if (lookahead == 'X' ||
          lookahead == 'x') ADVANCE(409);
      if (lookahead == 'Y' ||
          lookahead == 'y') ADVANCE(410);
      if (lookahead == 'Z') ADVANCE(77);
      if (lookahead == '[') ADVANCE(335);
      if (lookahead == '\\') ADVANCE(233);
      if (lookahead == ']') ADVANCE(337);
      if (lookahead == '^') ADVANCE(366);
      if (lookahead == 'a') ADVANCE(424);
      if (lookahead == 'b') ADVANCE(426);
      if (lookahead == 'c') ADVANCE(184);
      if (lookahead == 'd') ADVANCE(428);
      if (lookahead == 'e') ADVANCE(197);
      if (lookahead == 'f') ADVANCE(142);
      if (lookahead == 'g') ADVANCE(220);
      if (lookahead == 'i') ADVANCE(189);
      if (lookahead == 'j') ADVANCE(191);
      if (lookahead == 'l') ADVANCE(131);
      if (lookahead == 'm') ADVANCE(127);
      if (lookahead == 'n') ADVANCE(160);
      if (lookahead == 'o') ADVANCE(211);
      if (lookahead == 'p') ADVANCE(40);
      if (lookahead == 'q') ADVANCE(239);
      if (lookahead == 'r') ADVANCE(190);
      if (lookahead == 's') ADVANCE(414);
      if (lookahead == 't') ADVANCE(171);
      if (lookahead == 'w') ADVANCE(201);
      if (lookahead == 'z') ADVANCE(186);
      if (lookahead == '{') ADVANCE(326);
      if (lookahead == '|') ADVANCE(365);
      if (lookahead == '}') ADVANCE(327);
      if (lookahead == '~') ADVANCE(379);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(0)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(668);
      END_STATE();
    case 1:
      if (lookahead == '\n') ADVANCE(391);
      if (lookahead == '\r') ADVANCE(1);
      if (lookahead == '!') ADVANCE(19);
      if (lookahead == '%') ADVANCE(362);
      if (lookahead == '&') ADVANCE(367);
      if (lookahead == '(') ADVANCE(340);
      if (lookahead == '*') ADVANCE(357);
      if (lookahead == '+') ADVANCE(358);
      if (lookahead == ',') ADVANCE(289);
      if (lookahead == '-') ADVANCE(360);
      if (lookahead == '/') ADVANCE(356);
      if (lookahead == ':') ADVANCE(17);
      if (lookahead == ';') ADVANCE(351);
      if (lookahead == '<') ADVANCE(374);
      if (lookahead == '=') ADVANCE(20);
      if (lookahead == '>') ADVANCE(371);
      if (lookahead == 'A') ADVANCE(27);
      if (lookahead == 'B') ADVANCE(37);
      if (lookahead == 'C') ADVANCE(75);
      if (lookahead == 'D') ADVANCE(24);
      if (lookahead == 'E') ADVANCE(89);
      if (lookahead == 'F') ADVANCE(39);
      if (lookahead == 'G') ADVANCE(105);
      if (lookahead == 'I') ADVANCE(80);
      if (lookahead == 'J') ADVANCE(82);
      if (lookahead == 'L') ADVANCE(28);
      if (lookahead == 'M') ADVANCE(114);
      if (lookahead == 'N') ADVANCE(52);
      if (lookahead == 'O') ADVANCE(97);
      if (lookahead == 'P') ADVANCE(108);
      if (lookahead == 'R') ADVANCE(81);
      if (lookahead == 'S') ADVANCE(35);
      if (lookahead == 'T') ADVANCE(62);
      if (lookahead == 'W') ADVANCE(104);
      if (lookahead == 'Z') ADVANCE(77);
      if (lookahead == '^') ADVANCE(366);
      if (lookahead == 'a') ADVANCE(130);
      if (lookahead == 'b') ADVANCE(140);
      if (lookahead == 'c') ADVANCE(183);
      if (lookahead == 'd') ADVANCE(126);
      if (lookahead == 'e') ADVANCE(198);
      if (lookahead == 'f') ADVANCE(142);
      if (lookahead == 'g') ADVANCE(220);
      if (lookahead == 'i') ADVANCE(189);
      if (lookahead == 'j') ADVANCE(191);
      if (lookahead == 'l') ADVANCE(131);
      if (lookahead == 'm') ADVANCE(234);
      if (lookahead == 'n') ADVANCE(160);
      if (lookahead == 'o') ADVANCE(211);
      if (lookahead == 'p') ADVANCE(225);
      if (lookahead == 'r') ADVANCE(190);
      if (lookahead == 's') ADVANCE(139);
      if (lookahead == 't') ADVANCE(171);
      if (lookahead == 'w') ADVANCE(219);
      if (lookahead == 'z') ADVANCE(186);
      if (lookahead == '|') ADVANCE(365);
      if (lookahead == '\t' ||
          lookahead == ' ') SKIP(1)
      END_STATE();
    case 2:
      if (lookahead == '\n') SKIP(9)
      if (lookahead == '"') ADVANCE(384);
      if (lookahead == '\\') ADVANCE(233);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(385);
      if (lookahead != 0) ADVANCE(386);
      END_STATE();
    case 3:
      if (lookahead == '\n') SKIP(119)
      if (lookahead == '\\') ADVANCE(390);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(389);
      if (lookahead != 0 &&
          lookahead != '\'') ADVANCE(388);
      END_STATE();
    case 4:
      if (lookahead == '\n') ADVANCE(392);
      if (lookahead == '\r') ADVANCE(4);
      if (lookahead == '!') ADVANCE(377);
      if (lookahead == '#') ADVANCE(401);
      if (lookahead == '$') ADVANCE(278);
      if (lookahead == '%') ADVANCE(254);
      if (lookahead == '\'') ADVANCE(387);
      if (lookahead == '(') ADVANCE(340);
      if (lookahead == '*') ADVANCE(357);
      if (lookahead == '+') ADVANCE(358);
      if (lookahead == ',') ADVANCE(289);
      if (lookahead == '-') ADVANCE(360);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == '0') ADVANCE(666);
      if (lookahead == ';') ADVANCE(351);
      if (lookahead == '>') ADVANCE(370);
      if (lookahead == '@') ADVANCE(447);
      if (lookahead == '[') ADVANCE(334);
      if (lookahead == 'a') ADVANCE(425);
      if (lookahead == 'b') ADVANCE(427);
      if (lookahead == 'd') ADVANCE(429);
      if (lookahead == '~') ADVANCE(379);
      if (lookahead == '\t' ||
          lookahead == ' ') SKIP(4)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(668);
      if (lookahead == '.' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 5:
      if (lookahead == '\n') ADVANCE(393);
      if (lookahead == '\r') ADVANCE(5);
      if (lookahead == '+') ADVANCE(359);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == ';') ADVANCE(351);
      if (lookahead == '\t' ||
          lookahead == ' ') SKIP(5)
      END_STATE();
    case 6:
      if (lookahead == '!') ADVANCE(377);
      if (lookahead == '$') ADVANCE(278);
      if (lookahead == '%') ADVANCE(254);
      if (lookahead == '\'') ADVANCE(387);
      if (lookahead == '(') ADVANCE(340);
      if (lookahead == ')') ADVANCE(341);
      if (lookahead == '*') ADVANCE(357);
      if (lookahead == '+') ADVANCE(358);
      if (lookahead == '-') ADVANCE(360);
      if (lookahead == '0') ADVANCE(666);
      if (lookahead == '@') ADVANCE(447);
      if (lookahead == '~') ADVANCE(379);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(6)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(668);
      if (lookahead == '.' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 7:
      if (lookahead == '!') ADVANCE(377);
      if (lookahead == '$') ADVANCE(278);
      if (lookahead == '%') ADVANCE(254);
      if (lookahead == '\'') ADVANCE(387);
      if (lookahead == '(') ADVANCE(340);
      if (lookahead == '*') ADVANCE(357);
      if (lookahead == '+') ADVANCE(358);
      if (lookahead == ',') ADVANCE(289);
      if (lookahead == '-') ADVANCE(360);
      if (lookahead == '/') ADVANCE(355);
      if (lookahead == '0') ADVANCE(666);
      if (lookahead == '@') ADVANCE(447);
      if (lookahead == 'a') ADVANCE(425);
      if (lookahead == 'b') ADVANCE(427);
      if (lookahead == 'd') ADVANCE(429);
      if (lookahead == '~') ADVANCE(379);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(7)
      if (('1' <= lookahead && lookahead <= '9')) ADVANCE(668);
      if (lookahead == '.' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 8:
      if (lookahead == '!') ADVANCE(19);
      if (lookahead == '%') ADVANCE(362);
      if (lookahead == '&') ADVANCE(367);
      if (lookahead == '(') ADVANCE(340);
      if (lookahead == ')') ADVANCE(341);
      if (lookahead == '*') ADVANCE(357);
      if (lookahead == '+') ADVANCE(358);
      if (lookahead == ',') ADVANCE(289);
      if (lookahead == '-') ADVANCE(360);
      if (lookahead == '/') ADVANCE(355);
      if (lookahead == ':') ADVANCE(328);
      if (lookahead == '<') ADVANCE(374);
      if (lookahead == '=') ADVANCE(20);
      if (lookahead == '>') ADVANCE(371);
      if (lookahead == 'A' ||
          lookahead == 'a') ADVANCE(403);
      if (lookahead == 'B' ||
          lookahead == 'b') ADVANCE(405);
      if (lookahead == 'D' ||
          lookahead == 'd') ADVANCE(408);
      if (lookahead == 'S' ||
          lookahead == 's') ADVANCE(412);
      if (lookahead == 'U' ||
          lookahead == 'u') ADVANCE(411);
      if (lookahead == 'X' ||
          lookahead == 'x') ADVANCE(409);
      if (lookahead == 'Y' ||
          lookahead == 'y') ADVANCE(410);
      if (lookahead == ']') ADVANCE(336);
      if (lookahead == '^') ADVANCE(366);
      if (lookahead == '{') ADVANCE(326);
      if (lookahead == '|') ADVANCE(365);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(244);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(245);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(8)
      END_STATE();
    case 9:
      if (lookahead == '"') ADVANCE(384);
      if (lookahead == '\\') ADVANCE(233);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(9)
      END_STATE();
    case 10:
      if (lookahead == ')') ADVANCE(341);
      if (lookahead == ',') ADVANCE(289);
      if (lookahead == '}') ADVANCE(327);
      if (lookahead == '!' ||
          lookahead == '@') ADVANCE(447);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(10)
      if (lookahead == '.' ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 11:
      if (lookahead == '*') ADVANCE(353);
      if (lookahead == '/') ADVANCE(352);
      END_STATE();
    case 12:
      if (lookahead == '*') ADVANCE(354);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(12);
      if (lookahead != 0) ADVANCE(13);
      END_STATE();
    case 13:
      if (lookahead == '*') ADVANCE(354);
      if (lookahead != 0) ADVANCE(13);
      END_STATE();
    case 14:
      if (lookahead == '+') ADVANCE(423);
      END_STATE();
    case 15:
      if (lookahead == ',') ADVANCE(289);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == ':') ADVANCE(328);
      if (lookahead == ';') ADVANCE(350);
      if (lookahead == 'A') ADVANCE(454);
      if (lookahead == 'B') ADVANCE(464);
      if (lookahead == 'C') ADVANCE(499);
      if (lookahead == 'D') ADVANCE(451);
      if (lookahead == 'E') ADVANCE(511);
      if (lookahead == 'F') ADVANCE(466);
      if (lookahead == 'G') ADVANCE(527);
      if (lookahead == 'I') ADVANCE(503);
      if (lookahead == 'J') ADVANCE(505);
      if (lookahead == 'L') ADVANCE(455);
      if (lookahead == 'M') ADVANCE(535);
      if (lookahead == 'N') ADVANCE(478);
      if (lookahead == 'O') ADVANCE(519);
      if (lookahead == 'P') ADVANCE(529);
      if (lookahead == 'R') ADVANCE(504);
      if (lookahead == 'S') ADVANCE(462);
      if (lookahead == 'T') ADVANCE(487);
      if (lookahead == 'W') ADVANCE(526);
      if (lookahead == 'Z') ADVANCE(500);
      if (lookahead == '[') ADVANCE(334);
      if (lookahead == ']') ADVANCE(120);
      if (lookahead == 'a') ADVANCE(547);
      if (lookahead == 'b') ADVANCE(557);
      if (lookahead == 'c') ADVANCE(594);
      if (lookahead == 'd') ADVANCE(543);
      if (lookahead == 'e') ADVANCE(607);
      if (lookahead == 'f') ADVANCE(559);
      if (lookahead == 'g') ADVANCE(623);
      if (lookahead == 'i') ADVANCE(598);
      if (lookahead == 'j') ADVANCE(600);
      if (lookahead == 'l') ADVANCE(548);
      if (lookahead == 'm') ADVANCE(544);
      if (lookahead == 'n') ADVANCE(573);
      if (lookahead == 'o') ADVANCE(615);
      if (lookahead == 'p') ADVANCE(626);
      if (lookahead == 'r') ADVANCE(599);
      if (lookahead == 's') ADVANCE(556);
      if (lookahead == 't') ADVANCE(582);
      if (lookahead == 'w') ADVANCE(622);
      if (lookahead == 'z') ADVANCE(595);
      if (lookahead == '}') ADVANCE(327);
      if (lookahead == '!' ||
          lookahead == '@') ADVANCE(447);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(15)
      if (lookahead == '.' ||
          ('H' <= lookahead && lookahead <= 'Y') ||
          lookahead == '_' ||
          ('h' <= lookahead && lookahead <= 'y')) ADVANCE(661);
      END_STATE();
    case 16:
      if (lookahead == '-') ADVANCE(361);
      if (lookahead == 'S' ||
          lookahead == 's') ADVANCE(412);
      if (lookahead == 'U' ||
          lookahead == 'u') ADVANCE(411);
      if (lookahead == 'X' ||
          lookahead == 'x') ADVANCE(409);
      if (lookahead == 'Y' ||
          lookahead == 'y') ADVANCE(410);
      if (lookahead == 'b') ADVANCE(241);
      if (lookahead == 'd') ADVANCE(238);
      if (lookahead == 'p') ADVANCE(148);
      if (lookahead == 'q') ADVANCE(239);
      if (lookahead == 'w') ADVANCE(200);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(16)
      END_STATE();
    case 17:
      if (lookahead == ':') ADVANCE(446);
      END_STATE();
    case 18:
      if (lookahead == ';') ADVANCE(345);
      END_STATE();
    case 19:
      if (lookahead == '=') ADVANCE(369);
      END_STATE();
    case 20:
      if (lookahead == '=') ADVANCE(368);
      END_STATE();
    case 21:
      if (lookahead == 'A') ADVANCE(66);
      END_STATE();
    case 22:
      if (lookahead == 'A') ADVANCE(430);
      END_STATE();
    case 23:
      if (lookahead == 'A') ADVANCE(265);
      END_STATE();
    case 24:
      if (lookahead == 'A') ADVANCE(22);
      if (lookahead == 'E') ADVANCE(44);
      END_STATE();
    case 25:
      if (lookahead == 'A') ADVANCE(48);
      END_STATE();
    case 26:
      if (lookahead == 'A') ADVANCE(34);
      END_STATE();
    case 27:
      if (lookahead == 'B') ADVANCE(117);
      if (lookahead == 'D') ADVANCE(43);
      if (lookahead == 'N') ADVANCE(47);
      if (lookahead == 'S') ADVANCE(71);
      END_STATE();
    case 28:
      if (lookahead == 'B') ADVANCE(38);
      if (lookahead == 'D') ADVANCE(268);
      if (lookahead == 'E') ADVANCE(23);
      if (lookahead == 'S') ADVANCE(71);
      END_STATE();
    case 29:
      if (lookahead == 'B') ADVANCE(262);
      END_STATE();
    case 30:
      if (lookahead == 'B') ADVANCE(306);
      if (lookahead == 'C') ADVANCE(308);
      END_STATE();
    case 31:
      if (lookahead == 'B') ADVANCE(304);
      END_STATE();
    case 32:
      if (lookahead == 'B') ADVANCE(314);
      END_STATE();
    case 33:
      if (lookahead == 'B') ADVANCE(310);
      if (lookahead == 'D') ADVANCE(312);
      END_STATE();
    case 34:
      if (lookahead == 'B') ADVANCE(83);
      END_STATE();
    case 35:
      if (lookahead == 'B') ADVANCE(42);
      if (lookahead == 'C') ADVANCE(87);
      if (lookahead == 'E') ADVANCE(112);
      if (lookahead == 'T') ADVANCE(268);
      if (lookahead == 'U') ADVANCE(29);
      if (lookahead == 'W') ADVANCE(67);
      if (lookahead == 'Y') ADVANCE(86);
      END_STATE();
    case 36:
      if (lookahead == 'B') ADVANCE(69);
      END_STATE();
    case 37:
      if (lookahead == 'C') ADVANCE(243);
      if (lookahead == 'E') ADVANCE(96);
      if (lookahead == 'G') ADVANCE(246);
      if (lookahead == 'H') ADVANCE(247);
      if (lookahead == 'I') ADVANCE(109);
      if (lookahead == 'L') ADVANCE(264);
      if (lookahead == 'M') ADVANCE(66);
      if (lookahead == 'N') ADVANCE(53);
      if (lookahead == 'P') ADVANCE(70);
      if (lookahead == 'R') ADVANCE(242);
      if (lookahead == 'S') ADVANCE(99);
      if (lookahead == 'V') ADVANCE(243);
      END_STATE();
    case 38:
      if (lookahead == 'C') ADVANCE(243);
      if (lookahead == 'E') ADVANCE(96);
      if (lookahead == 'G') ADVANCE(246);
      if (lookahead == 'H') ADVANCE(247);
      if (lookahead == 'L') ADVANCE(264);
      if (lookahead == 'M') ADVANCE(66);
      if (lookahead == 'N') ADVANCE(53);
      if (lookahead == 'P') ADVANCE(70);
      if (lookahead == 'R') ADVANCE(242);
      if (lookahead == 'S') ADVANCE(98);
      if (lookahead == 'V') ADVANCE(243);
      END_STATE();
    case 39:
      if (lookahead == 'C') ADVANCE(30);
      if (lookahead == 'D') ADVANCE(31);
      if (lookahead == 'I') ADVANCE(74);
      END_STATE();
    case 40:
      if (lookahead == 'C') ADVANCE(415);
      if (lookahead == 'c') ADVANCE(419);
      if (lookahead == 's') ADVANCE(173);
      if (lookahead == 'u') ADVANCE(181);
      END_STATE();
    case 41:
      if (lookahead == 'C') ADVANCE(430);
      END_STATE();
    case 42:
      if (lookahead == 'C') ADVANCE(256);
      END_STATE();
    case 43:
      if (lookahead == 'C') ADVANCE(256);
      if (lookahead == 'D') ADVANCE(262);
      END_STATE();
    case 44:
      if (lookahead == 'C') ADVANCE(441);
      END_STATE();
    case 45:
      if (lookahead == 'C') ADVANCE(431);
      END_STATE();
    case 46:
      if (lookahead == 'C') ADVANCE(121);
      END_STATE();
    case 47:
      if (lookahead == 'D') ADVANCE(257);
      END_STATE();
    case 48:
      if (lookahead == 'D') ADVANCE(51);
      END_STATE();
    case 49:
      if (lookahead == 'D') ADVANCE(55);
      END_STATE();
    case 50:
      if (lookahead == 'D') ADVANCE(93);
      END_STATE();
    case 51:
      if (lookahead == 'D') ADVANCE(103);
      END_STATE();
    case 52:
      if (lookahead == 'E') ADVANCE(63);
      if (lookahead == 'O') ADVANCE(92);
      END_STATE();
    case 53:
      if (lookahead == 'E') ADVANCE(430);
      END_STATE();
    case 54:
      if (lookahead == 'E') ADVANCE(283);
      END_STATE();
    case 55:
      if (lookahead == 'E') ADVANCE(322);
      END_STATE();
    case 56:
      if (lookahead == 'E') ADVANCE(61);
      END_STATE();
    case 57:
      if (lookahead == 'E') ADVANCE(79);
      END_STATE();
    case 58:
      if (lookahead == 'E') ADVANCE(46);
      if (lookahead == 'G') ADVANCE(394);
      END_STATE();
    case 59:
      if (lookahead == 'E') ADVANCE(46);
      if (lookahead == 'G') ADVANCE(399);
      END_STATE();
    case 60:
      if (lookahead == 'E') ADVANCE(36);
      END_STATE();
    case 61:
      if (lookahead == 'F') ADVANCE(298);
      END_STATE();
    case 62:
      if (lookahead == 'F') ADVANCE(102);
      if (lookahead == 'S') ADVANCE(110);
      END_STATE();
    case 63:
      if (lookahead == 'G') ADVANCE(441);
      END_STATE();
    case 64:
      if (lookahead == 'H') ADVANCE(395);
      END_STATE();
    case 65:
      if (lookahead == 'H') ADVANCE(248);
      END_STATE();
    case 66:
      if (lookahead == 'I') ADVANCE(430);
      END_STATE();
    case 67:
      if (lookahead == 'I') ADVANCE(439);
      END_STATE();
    case 68:
      if (lookahead == 'I') ADVANCE(113);
      END_STATE();
    case 69:
      if (lookahead == 'I') ADVANCE(85);
      END_STATE();
    case 70:
      if (lookahead == 'L') ADVANCE(430);
      END_STATE();
    case 71:
      if (lookahead == 'L') ADVANCE(441);
      if (lookahead == 'R') ADVANCE(441);
      END_STATE();
    case 72:
      if (lookahead == 'L') ADVANCE(248);
      if (lookahead == 'T') ADVANCE(285);
      END_STATE();
    case 73:
      if (lookahead == 'L') ADVANCE(302);
      END_STATE();
    case 74:
      if (lookahead == 'L') ADVANCE(73);
      END_STATE();
    case 75:
      if (lookahead == 'L') ADVANCE(101);
      if (lookahead == 'M') ADVANCE(91);
      if (lookahead == 'O') ADVANCE(78);
      if (lookahead == 'W') ADVANCE(21);
      END_STATE();
    case 76:
      if (lookahead == 'L') ADVANCE(101);
      if (lookahead == 'M') ADVANCE(91);
      if (lookahead == 'O') ADVANCE(78);
      if (lookahead == 'W') ADVANCE(21);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(417);
      END_STATE();
    case 77:
      if (lookahead == 'M') ADVANCE(33);
      END_STATE();
    case 78:
      if (lookahead == 'M') ADVANCE(441);
      END_STATE();
    case 79:
      if (lookahead == 'M') ADVANCE(287);
      END_STATE();
    case 80:
      if (lookahead == 'M') ADVANCE(95);
      if (lookahead == 'N') ADVANCE(45);
      END_STATE();
    case 81:
      if (lookahead == 'M') ADVANCE(32);
      if (lookahead == 'O') ADVANCE(71);
      if (lookahead == 'T') ADVANCE(247);
      END_STATE();
    case 82:
      if (lookahead == 'M') ADVANCE(92);
      if (lookahead == 'S') ADVANCE(98);
      END_STATE();
    case 83:
      if (lookahead == 'M') ADVANCE(57);
      END_STATE();
    case 84:
      if (lookahead == 'N') ADVANCE(292);
      END_STATE();
    case 85:
      if (lookahead == 'N') ADVANCE(290);
      END_STATE();
    case 86:
      if (lookahead == 'N') ADVANCE(41);
      END_STATE();
    case 87:
      if (lookahead == 'O') ADVANCE(94);
      END_STATE();
    case 88:
      if (lookahead == 'O') ADVANCE(100);
      if (lookahead == 'Q') ADVANCE(115);
      if (lookahead == 'X') ADVANCE(58);
      END_STATE();
    case 89:
      if (lookahead == 'O') ADVANCE(100);
      if (lookahead == 'Q') ADVANCE(115);
      if (lookahead == 'X') ADVANCE(59);
      END_STATE();
    case 90:
      if (lookahead == 'O') ADVANCE(106);
      END_STATE();
    case 91:
      if (lookahead == 'P') ADVANCE(268);
      END_STATE();
    case 92:
      if (lookahead == 'P') ADVANCE(430);
      END_STATE();
    case 93:
      if (lookahead == 'P') ADVANCE(316);
      END_STATE();
    case 94:
      if (lookahead == 'P') ADVANCE(54);
      END_STATE();
    case 95:
      if (lookahead == 'P') ADVANCE(90);
      END_STATE();
    case 96:
      if (lookahead == 'Q') ADVANCE(430);
      END_STATE();
    case 97:
      if (lookahead == 'R') ADVANCE(258);
      END_STATE();
    case 98:
      if (lookahead == 'R') ADVANCE(430);
      END_STATE();
    case 99:
      if (lookahead == 'R') ADVANCE(430);
      if (lookahead == 'Z') ADVANCE(300);
      END_STATE();
    case 100:
      if (lookahead == 'R') ADVANCE(256);
      END_STATE();
    case 101:
      if (lookahead == 'R') ADVANCE(441);
      END_STATE();
    case 102:
      if (lookahead == 'R') ADVANCE(399);
      END_STATE();
    case 103:
      if (lookahead == 'R') ADVANCE(320);
      END_STATE();
    case 104:
      if (lookahead == 'R') ADVANCE(68);
      END_STATE();
    case 105:
      if (lookahead == 'R') ADVANCE(26);
      END_STATE();
    case 106:
      if (lookahead == 'R') ADVANCE(111);
      END_STATE();
    case 107:
      if (lookahead == 'S') ADVANCE(64);
      if (lookahead == 'U') ADVANCE(72);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(415);
      END_STATE();
    case 108:
      if (lookahead == 'S') ADVANCE(65);
      if (lookahead == 'U') ADVANCE(72);
      END_STATE();
    case 109:
      if (lookahead == 'T') ADVANCE(256);
      END_STATE();
    case 110:
      if (lookahead == 'T') ADVANCE(441);
      END_STATE();
    case 111:
      if (lookahead == 'T') ADVANCE(296);
      END_STATE();
    case 112:
      if (lookahead == 'T') ADVANCE(50);
      if (lookahead == 'X') ADVANCE(430);
      END_STATE();
    case 113:
      if (lookahead == 'T') ADVANCE(60);
      END_STATE();
    case 114:
      if (lookahead == 'U') ADVANCE(70);
      END_STATE();
    case 115:
      if (lookahead == 'U') ADVANCE(402);
      END_STATE();
    case 116:
      if (lookahead == 'U') ADVANCE(49);
      END_STATE();
    case 117:
      if (lookahead == 'X') ADVANCE(430);
      END_STATE();
    case 118:
      if (lookahead == '[') ADVANCE(348);
      END_STATE();
    case 119:
      if (lookahead == '\\') ADVANCE(233);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(119)
      END_STATE();
    case 120:
      if (lookahead == ']') ADVANCE(349);
      END_STATE();
    case 121:
      if (lookahead == '_') ADVANCE(25);
      END_STATE();
    case 122:
      if (lookahead == '_') ADVANCE(129);
      END_STATE();
    case 123:
      if (lookahead == 'a') ADVANCE(175);
      END_STATE();
    case 124:
      if (lookahead == 'a') ADVANCE(430);
      END_STATE();
    case 125:
      if (lookahead == 'a') ADVANCE(267);
      END_STATE();
    case 126:
      if (lookahead == 'a') ADVANCE(124);
      if (lookahead == 'e') ADVANCE(146);
      END_STATE();
    case 127:
      if (lookahead == 'a') ADVANCE(151);
      if (lookahead == 'u') ADVANCE(179);
      END_STATE();
    case 128:
      if (lookahead == 'a') ADVANCE(137);
      END_STATE();
    case 129:
      if (lookahead == 'a') ADVANCE(158);
      END_STATE();
    case 130:
      if (lookahead == 'b') ADVANCE(240);
      if (lookahead == 'd') ADVANCE(145);
      if (lookahead == 'n') ADVANCE(152);
      if (lookahead == 's') ADVANCE(180);
      END_STATE();
    case 131:
      if (lookahead == 'b') ADVANCE(141);
      if (lookahead == 'd') ADVANCE(269);
      if (lookahead == 'e') ADVANCE(125);
      if (lookahead == 's') ADVANCE(180);
      END_STATE();
    case 132:
      if (lookahead == 'b') ADVANCE(306);
      if (lookahead == 'c') ADVANCE(308);
      END_STATE();
    case 133:
      if (lookahead == 'b') ADVANCE(304);
      END_STATE();
    case 134:
      if (lookahead == 'b') ADVANCE(314);
      END_STATE();
    case 135:
      if (lookahead == 'b') ADVANCE(310);
      if (lookahead == 'd') ADVANCE(312);
      END_STATE();
    case 136:
      if (lookahead == 'b') ADVANCE(263);
      END_STATE();
    case 137:
      if (lookahead == 'b') ADVANCE(192);
      END_STATE();
    case 138:
      if (lookahead == 'b') ADVANCE(177);
      END_STATE();
    case 139:
      if (lookahead == 'b') ADVANCE(144);
      if (lookahead == 'c') ADVANCE(199);
      if (lookahead == 'e') ADVANCE(230);
      if (lookahead == 't') ADVANCE(269);
      if (lookahead == 'u') ADVANCE(136);
      if (lookahead == 'w') ADVANCE(176);
      if (lookahead == 'y') ADVANCE(195);
      END_STATE();
    case 140:
      if (lookahead == 'c') ADVANCE(250);
      if (lookahead == 'e') ADVANCE(210);
      if (lookahead == 'g') ADVANCE(251);
      if (lookahead == 'h') ADVANCE(252);
      if (lookahead == 'i') ADVANCE(226);
      if (lookahead == 'l') ADVANCE(266);
      if (lookahead == 'm') ADVANCE(175);
      if (lookahead == 'n') ADVANCE(161);
      if (lookahead == 'p') ADVANCE(179);
      if (lookahead == 'r') ADVANCE(249);
      if (lookahead == 's') ADVANCE(213);
      if (lookahead == 'v') ADVANCE(250);
      END_STATE();
    case 141:
      if (lookahead == 'c') ADVANCE(250);
      if (lookahead == 'e') ADVANCE(210);
      if (lookahead == 'g') ADVANCE(251);
      if (lookahead == 'h') ADVANCE(252);
      if (lookahead == 'l') ADVANCE(266);
      if (lookahead == 'm') ADVANCE(175);
      if (lookahead == 'n') ADVANCE(161);
      if (lookahead == 'p') ADVANCE(179);
      if (lookahead == 'r') ADVANCE(249);
      if (lookahead == 's') ADVANCE(212);
      if (lookahead == 'v') ADVANCE(250);
      END_STATE();
    case 142:
      if (lookahead == 'c') ADVANCE(132);
      if (lookahead == 'd') ADVANCE(133);
      if (lookahead == 'i') ADVANCE(185);
      END_STATE();
    case 143:
      if (lookahead == 'c') ADVANCE(430);
      END_STATE();
    case 144:
      if (lookahead == 'c') ADVANCE(259);
      END_STATE();
    case 145:
      if (lookahead == 'c') ADVANCE(259);
      if (lookahead == 'd') ADVANCE(263);
      END_STATE();
    case 146:
      if (lookahead == 'c') ADVANCE(443);
      END_STATE();
    case 147:
      if (lookahead == 'c') ADVANCE(435);
      END_STATE();
    case 148:
      if (lookahead == 'c') ADVANCE(420);
      END_STATE();
    case 149:
      if (lookahead == 'c') ADVANCE(122);
      END_STATE();
    case 150:
      if (lookahead == 'c') ADVANCE(229);
      END_STATE();
    case 151:
      if (lookahead == 'c') ADVANCE(221);
      END_STATE();
    case 152:
      if (lookahead == 'd') ADVANCE(260);
      END_STATE();
    case 153:
      if (lookahead == 'd') ADVANCE(331);
      END_STATE();
    case 154:
      if (lookahead == 'd') ADVANCE(332);
      END_STATE();
    case 155:
      if (lookahead == 'd') ADVANCE(333);
      END_STATE();
    case 156:
      if (lookahead == 'd') ADVANCE(207);
      END_STATE();
    case 157:
      if (lookahead == 'd') ADVANCE(164);
      END_STATE();
    case 158:
      if (lookahead == 'd') ADVANCE(159);
      END_STATE();
    case 159:
      if (lookahead == 'd') ADVANCE(217);
      END_STATE();
    case 160:
      if (lookahead == 'e') ADVANCE(172);
      if (lookahead == 'o') ADVANCE(206);
      END_STATE();
    case 161:
      if (lookahead == 'e') ADVANCE(430);
      END_STATE();
    case 162:
      if (lookahead == 'e') ADVANCE(330);
      END_STATE();
    case 163:
      if (lookahead == 'e') ADVANCE(283);
      END_STATE();
    case 164:
      if (lookahead == 'e') ADVANCE(322);
      END_STATE();
    case 165:
      if (lookahead == 'e') ADVANCE(170);
      END_STATE();
    case 166:
      if (lookahead == 'e') ADVANCE(188);
      END_STATE();
    case 167:
      if (lookahead == 'e') ADVANCE(149);
      if (lookahead == 'g') ADVANCE(394);
      END_STATE();
    case 168:
      if (lookahead == 'e') ADVANCE(149);
      if (lookahead == 'g') ADVANCE(399);
      END_STATE();
    case 169:
      if (lookahead == 'e') ADVANCE(138);
      END_STATE();
    case 170:
      if (lookahead == 'f') ADVANCE(298);
      END_STATE();
    case 171:
      if (lookahead == 'f') ADVANCE(214);
      if (lookahead == 's') ADVANCE(227);
      END_STATE();
    case 172:
      if (lookahead == 'g') ADVANCE(443);
      END_STATE();
    case 173:
      if (lookahead == 'h') ADVANCE(396);
      END_STATE();
    case 174:
      if (lookahead == 'h') ADVANCE(253);
      END_STATE();
    case 175:
      if (lookahead == 'i') ADVANCE(430);
      END_STATE();
    case 176:
      if (lookahead == 'i') ADVANCE(439);
      END_STATE();
    case 177:
      if (lookahead == 'i') ADVANCE(194);
      END_STATE();
    case 178:
      if (lookahead == 'i') ADVANCE(232);
      END_STATE();
    case 179:
      if (lookahead == 'l') ADVANCE(430);
      END_STATE();
    case 180:
      if (lookahead == 'l') ADVANCE(443);
      if (lookahead == 'r') ADVANCE(443);
      END_STATE();
    case 181:
      if (lookahead == 'l') ADVANCE(253);
      if (lookahead == 't') ADVANCE(285);
      END_STATE();
    case 182:
      if (lookahead == 'l') ADVANCE(302);
      END_STATE();
    case 183:
      if (lookahead == 'l') ADVANCE(216);
      if (lookahead == 'm') ADVANCE(205);
      if (lookahead == 'o') ADVANCE(187);
      if (lookahead == 'w') ADVANCE(123);
      END_STATE();
    case 184:
      if (lookahead == 'l') ADVANCE(216);
      if (lookahead == 'm') ADVANCE(205);
      if (lookahead == 'o') ADVANCE(187);
      if (lookahead == 'w') ADVANCE(123);
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(417);
      END_STATE();
    case 185:
      if (lookahead == 'l') ADVANCE(182);
      END_STATE();
    case 186:
      if (lookahead == 'm') ADVANCE(135);
      END_STATE();
    case 187:
      if (lookahead == 'm') ADVANCE(443);
      END_STATE();
    case 188:
      if (lookahead == 'm') ADVANCE(287);
      END_STATE();
    case 189:
      if (lookahead == 'm') ADVANCE(209);
      if (lookahead == 'n') ADVANCE(147);
      END_STATE();
    case 190:
      if (lookahead == 'm') ADVANCE(134);
      if (lookahead == 'o') ADVANCE(180);
      if (lookahead == 't') ADVANCE(252);
      END_STATE();
    case 191:
      if (lookahead == 'm') ADVANCE(206);
      if (lookahead == 's') ADVANCE(212);
      END_STATE();
    case 192:
      if (lookahead == 'm') ADVANCE(166);
      END_STATE();
    case 193:
      if (lookahead == 'n') ADVANCE(295);
      END_STATE();
    case 194:
      if (lookahead == 'n') ADVANCE(290);
      END_STATE();
    case 195:
      if (lookahead == 'n') ADVANCE(143);
      END_STATE();
    case 196:
      if (lookahead == 'o') ADVANCE(338);
      END_STATE();
    case 197:
      if (lookahead == 'o') ADVANCE(215);
      if (lookahead == 'q') ADVANCE(235);
      if (lookahead == 'x') ADVANCE(167);
      END_STATE();
    case 198:
      if (lookahead == 'o') ADVANCE(215);
      if (lookahead == 'q') ADVANCE(235);
      if (lookahead == 'x') ADVANCE(168);
      END_STATE();
    case 199:
      if (lookahead == 'o') ADVANCE(208);
      END_STATE();
    case 200:
      if (lookahead == 'o') ADVANCE(218);
      END_STATE();
    case 201:
      if (lookahead == 'o') ADVANCE(218);
      if (lookahead == 'r') ADVANCE(178);
      END_STATE();
    case 202:
      if (lookahead == 'o') ADVANCE(222);
      END_STATE();
    case 203:
      if (lookahead == 'o') ADVANCE(224);
      END_STATE();
    case 204:
      if (lookahead == 'o') ADVANCE(223);
      END_STATE();
    case 205:
      if (lookahead == 'p') ADVANCE(269);
      END_STATE();
    case 206:
      if (lookahead == 'p') ADVANCE(430);
      END_STATE();
    case 207:
      if (lookahead == 'p') ADVANCE(316);
      END_STATE();
    case 208:
      if (lookahead == 'p') ADVANCE(163);
      END_STATE();
    case 209:
      if (lookahead == 'p') ADVANCE(204);
      END_STATE();
    case 210:
      if (lookahead == 'q') ADVANCE(430);
      END_STATE();
    case 211:
      if (lookahead == 'r') ADVANCE(261);
      END_STATE();
    case 212:
      if (lookahead == 'r') ADVANCE(430);
      END_STATE();
    case 213:
      if (lookahead == 'r') ADVANCE(430);
      if (lookahead == 'z') ADVANCE(300);
      END_STATE();
    case 214:
      if (lookahead == 'r') ADVANCE(399);
      END_STATE();
    case 215:
      if (lookahead == 'r') ADVANCE(259);
      END_STATE();
    case 216:
      if (lookahead == 'r') ADVANCE(443);
      END_STATE();
    case 217:
      if (lookahead == 'r') ADVANCE(320);
      END_STATE();
    case 218:
      if (lookahead == 'r') ADVANCE(153);
      END_STATE();
    case 219:
      if (lookahead == 'r') ADVANCE(178);
      END_STATE();
    case 220:
      if (lookahead == 'r') ADVANCE(128);
      END_STATE();
    case 221:
      if (lookahead == 'r') ADVANCE(196);
      END_STATE();
    case 222:
      if (lookahead == 'r') ADVANCE(154);
      END_STATE();
    case 223:
      if (lookahead == 'r') ADVANCE(228);
      END_STATE();
    case 224:
      if (lookahead == 'r') ADVANCE(155);
      END_STATE();
    case 225:
      if (lookahead == 's') ADVANCE(174);
      if (lookahead == 'u') ADVANCE(181);
      END_STATE();
    case 226:
      if (lookahead == 't') ADVANCE(259);
      END_STATE();
    case 227:
      if (lookahead == 't') ADVANCE(443);
      END_STATE();
    case 228:
      if (lookahead == 't') ADVANCE(296);
      END_STATE();
    case 229:
      if (lookahead == 't') ADVANCE(324);
      END_STATE();
    case 230:
      if (lookahead == 't') ADVANCE(156);
      if (lookahead == 'x') ADVANCE(430);
      END_STATE();
    case 231:
      if (lookahead == 't') ADVANCE(162);
      END_STATE();
    case 232:
      if (lookahead == 't') ADVANCE(169);
      END_STATE();
    case 233:
      if (lookahead == 'u') ADVANCE(277);
      if (lookahead == 'x') ADVANCE(273);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(382);
      if (lookahead != 0) ADVANCE(380);
      END_STATE();
    case 234:
      if (lookahead == 'u') ADVANCE(179);
      END_STATE();
    case 235:
      if (lookahead == 'u') ADVANCE(402);
      END_STATE();
    case 236:
      if (lookahead == 'u') ADVANCE(157);
      END_STATE();
    case 237:
      if (lookahead == 'u') ADVANCE(150);
      END_STATE();
    case 238:
      if (lookahead == 'w') ADVANCE(202);
      END_STATE();
    case 239:
      if (lookahead == 'w') ADVANCE(203);
      END_STATE();
    case 240:
      if (lookahead == 'x') ADVANCE(430);
      END_STATE();
    case 241:
      if (lookahead == 'y') ADVANCE(231);
      END_STATE();
    case 242:
      if (lookahead == 'A' ||
          lookahead == 'N') ADVANCE(430);
      END_STATE();
    case 243:
      if (lookahead == 'C' ||
          lookahead == 'S') ADVANCE(430);
      END_STATE();
    case 244:
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(417);
      END_STATE();
    case 245:
      if (lookahead == 'C' ||
          lookahead == 'c') ADVANCE(415);
      END_STATE();
    case 246:
      if (lookahead == 'E' ||
          lookahead == 'T') ADVANCE(430);
      END_STATE();
    case 247:
      if (lookahead == 'I' ||
          lookahead == 'S') ADVANCE(430);
      END_STATE();
    case 248:
      if (lookahead == 'S' ||
          lookahead == 'U') ADVANCE(397);
      END_STATE();
    case 249:
      if (lookahead == 'a' ||
          lookahead == 'n') ADVANCE(430);
      END_STATE();
    case 250:
      if (lookahead == 'c' ||
          lookahead == 's') ADVANCE(430);
      END_STATE();
    case 251:
      if (lookahead == 'e' ||
          lookahead == 't') ADVANCE(430);
      END_STATE();
    case 252:
      if (lookahead == 'i' ||
          lookahead == 's') ADVANCE(430);
      END_STATE();
    case 253:
      if (lookahead == 's' ||
          lookahead == 'u') ADVANCE(397);
      END_STATE();
    case 254:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(663);
      END_STATE();
    case 255:
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(662);
      END_STATE();
    case 256:
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(430);
      END_STATE();
    case 257:
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(430);
      if (lookahead == 'C') ADVANCE(41);
      END_STATE();
    case 258:
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(430);
      if (lookahead == 'C') ADVANCE(41);
      if (lookahead == 'G') ADVANCE(318);
      END_STATE();
    case 259:
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(430);
      END_STATE();
    case 260:
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(430);
      if (lookahead == 'c') ADVANCE(143);
      END_STATE();
    case 261:
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(430);
      if (lookahead == 'c') ADVANCE(143);
      if (lookahead == 'g') ADVANCE(318);
      END_STATE();
    case 262:
      if (lookahead == 'A' ||
          lookahead == 'B' ||
          lookahead == 'D') ADVANCE(430);
      END_STATE();
    case 263:
      if (lookahead == 'a' ||
          lookahead == 'b' ||
          lookahead == 'd') ADVANCE(430);
      END_STATE();
    case 264:
      if (lookahead == 'E' ||
          lookahead == 'O' ||
          lookahead == 'S' ||
          lookahead == 'T') ADVANCE(430);
      END_STATE();
    case 265:
      if (lookahead == 'S' ||
          lookahead == 'U' ||
          lookahead == 'X' ||
          lookahead == 'Y') ADVANCE(430);
      END_STATE();
    case 266:
      if (lookahead == 'e' ||
          lookahead == 'o' ||
          lookahead == 's' ||
          lookahead == 't') ADVANCE(430);
      END_STATE();
    case 267:
      if (lookahead == 's' ||
          lookahead == 'u' ||
          lookahead == 'x' ||
          lookahead == 'y') ADVANCE(430);
      END_STATE();
    case 268:
      if (lookahead == 'A' ||
          lookahead == 'B' ||
          lookahead == 'D' ||
          lookahead == 'S' ||
          lookahead == 'U' ||
          lookahead == 'X' ||
          lookahead == 'Y') ADVANCE(430);
      END_STATE();
    case 269:
      if (lookahead == 'a' ||
          lookahead == 'b' ||
          lookahead == 'd' ||
          lookahead == 's' ||
          lookahead == 'u' ||
          lookahead == 'x' ||
          lookahead == 'y') ADVANCE(430);
      END_STATE();
    case 270:
      if (lookahead == 'a' ||
          lookahead == 'b' ||
          lookahead == 'd' ||
          lookahead == 's' ||
          lookahead == 'u' ||
          lookahead == 'x' ||
          lookahead == 'y') ADVANCE(430);
      if (lookahead == 'r') ADVANCE(237);
      END_STATE();
    case 271:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(380);
      END_STATE();
    case 272:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(383);
      END_STATE();
    case 273:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(271);
      END_STATE();
    case 274:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(272);
      END_STATE();
    case 275:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(273);
      END_STATE();
    case 276:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(274);
      END_STATE();
    case 277:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(276);
      END_STATE();
    case 278:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(665);
      END_STATE();
    case 279:
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(664);
      END_STATE();
    case 280:
      if (lookahead != 0 &&
          lookahead != '\r') ADVANCE(343);
      if (lookahead == '\r') ADVANCE(344);
      END_STATE();
    case 281:
      if (eof) ADVANCE(282);
      if (lookahead == '+') ADVANCE(14);
      if (lookahead == '/') ADVANCE(11);
      if (lookahead == ':') ADVANCE(17);
      if (lookahead == ';') ADVANCE(351);
      if (lookahead == 'A') ADVANCE(454);
      if (lookahead == 'B') ADVANCE(464);
      if (lookahead == 'C') ADVANCE(499);
      if (lookahead == 'D') ADVANCE(451);
      if (lookahead == 'E') ADVANCE(511);
      if (lookahead == 'F') ADVANCE(466);
      if (lookahead == 'G') ADVANCE(527);
      if (lookahead == 'I') ADVANCE(503);
      if (lookahead == 'J') ADVANCE(505);
      if (lookahead == 'L') ADVANCE(455);
      if (lookahead == 'M') ADVANCE(535);
      if (lookahead == 'N') ADVANCE(478);
      if (lookahead == 'O') ADVANCE(519);
      if (lookahead == 'P') ADVANCE(529);
      if (lookahead == 'R') ADVANCE(504);
      if (lookahead == 'S') ADVANCE(462);
      if (lookahead == 'T') ADVANCE(487);
      if (lookahead == 'W') ADVANCE(526);
      if (lookahead == 'Z') ADVANCE(500);
      if (lookahead == '[') ADVANCE(118);
      if (lookahead == ']') ADVANCE(336);
      if (lookahead == 'a') ADVANCE(547);
      if (lookahead == 'b') ADVANCE(557);
      if (lookahead == 'c') ADVANCE(594);
      if (lookahead == 'd') ADVANCE(543);
      if (lookahead == 'e') ADVANCE(607);
      if (lookahead == 'f') ADVANCE(559);
      if (lookahead == 'g') ADVANCE(623);
      if (lookahead == 'i') ADVANCE(598);
      if (lookahead == 'j') ADVANCE(600);
      if (lookahead == 'l') ADVANCE(548);
      if (lookahead == 'm') ADVANCE(544);
      if (lookahead == 'n') ADVANCE(573);
      if (lookahead == 'o') ADVANCE(615);
      if (lookahead == 'p') ADVANCE(626);
      if (lookahead == 'r') ADVANCE(599);
      if (lookahead == 's') ADVANCE(556);
      if (lookahead == 't') ADVANCE(582);
      if (lookahead == 'w') ADVANCE(622);
      if (lookahead == 'z') ADVANCE(595);
      if (lookahead == '!' ||
          lookahead == '@') ADVANCE(447);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') SKIP(281)
      if (lookahead == '.' ||
          ('H' <= lookahead && lookahead <= 'Y') ||
          lookahead == '_' ||
          ('h' <= lookahead && lookahead <= 'y')) ADVANCE(661);
      END_STATE();
    case 282:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 283:
      ACCEPT_TOKEN(aux_sym_scope_token1);
      END_STATE();
    case 284:
      ACCEPT_TOKEN(aux_sym_scope_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 285:
      ACCEPT_TOKEN(aux_sym_put_token1);
      END_STATE();
    case 286:
      ACCEPT_TOKEN(aux_sym_put_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 287:
      ACCEPT_TOKEN(aux_sym_grabmem_token1);
      END_STATE();
    case 288:
      ACCEPT_TOKEN(aux_sym_grabmem_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 289:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 290:
      ACCEPT_TOKEN(aux_sym_writebin_token1);
      END_STATE();
    case 291:
      ACCEPT_TOKEN(aux_sym_writebin_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 292:
      ACCEPT_TOKEN(aux_sym_incbin_token1);
      if (lookahead == 'R') ADVANCE(56);
      END_STATE();
    case 293:
      ACCEPT_TOKEN(aux_sym_incbin_token1);
      if (lookahead == 'R') ADVANCE(482);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 294:
      ACCEPT_TOKEN(aux_sym_incbin_token1);
      if (lookahead == 'r') ADVANCE(577);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 295:
      ACCEPT_TOKEN(aux_sym_incbin_token1);
      if (lookahead == 'r') ADVANCE(165);
      END_STATE();
    case 296:
      ACCEPT_TOKEN(aux_sym_importer_token1);
      END_STATE();
    case 297:
      ACCEPT_TOKEN(aux_sym_importer_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 298:
      ACCEPT_TOKEN(aux_sym_incbinref_token1);
      END_STATE();
    case 299:
      ACCEPT_TOKEN(aux_sym_incbinref_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 300:
      ACCEPT_TOKEN(aux_sym_bsz_token1);
      END_STATE();
    case 301:
      ACCEPT_TOKEN(aux_sym_bsz_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 302:
      ACCEPT_TOKEN(aux_sym_fill_token1);
      END_STATE();
    case 303:
      ACCEPT_TOKEN(aux_sym_fill_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 304:
      ACCEPT_TOKEN(aux_sym_fdb_token1);
      END_STATE();
    case 305:
      ACCEPT_TOKEN(aux_sym_fdb_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 306:
      ACCEPT_TOKEN(aux_sym_fcb_token1);
      END_STATE();
    case 307:
      ACCEPT_TOKEN(aux_sym_fcb_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 308:
      ACCEPT_TOKEN(aux_sym_fcc_token1);
      END_STATE();
    case 309:
      ACCEPT_TOKEN(aux_sym_fcc_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 310:
      ACCEPT_TOKEN(aux_sym_zmb_token1);
      END_STATE();
    case 311:
      ACCEPT_TOKEN(aux_sym_zmb_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 312:
      ACCEPT_TOKEN(aux_sym_zmd_token1);
      END_STATE();
    case 313:
      ACCEPT_TOKEN(aux_sym_zmd_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 314:
      ACCEPT_TOKEN(aux_sym_rmb_token1);
      END_STATE();
    case 315:
      ACCEPT_TOKEN(aux_sym_rmb_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 316:
      ACCEPT_TOKEN(aux_sym_setdp_token1);
      END_STATE();
    case 317:
      ACCEPT_TOKEN(aux_sym_setdp_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 318:
      ACCEPT_TOKEN(aux_sym_org_token1);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(aux_sym_org_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(aux_sym_exec_addr_token1);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(aux_sym_exec_addr_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(aux_sym_include_token1);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(aux_sym_include_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(anon_sym_struct);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(anon_sym_struct);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 329:
      ACCEPT_TOKEN(anon_sym_COLON);
      if (lookahead == ':') ADVANCE(446);
      END_STATE();
    case 330:
      ACCEPT_TOKEN(anon_sym_byte);
      END_STATE();
    case 331:
      ACCEPT_TOKEN(anon_sym_word);
      END_STATE();
    case 332:
      ACCEPT_TOKEN(anon_sym_dword);
      END_STATE();
    case 333:
      ACCEPT_TOKEN(anon_sym_qword);
      END_STATE();
    case 334:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 335:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      if (lookahead == '[') ADVANCE(348);
      END_STATE();
    case 336:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 337:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      if (lookahead == ']') ADVANCE(349);
      END_STATE();
    case 338:
      ACCEPT_TOKEN(anon_sym_macro);
      END_STATE();
    case 339:
      ACCEPT_TOKEN(anon_sym_macro);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 340:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 341:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 342:
      ACCEPT_TOKEN(aux_sym_doc_text_token1);
      if (lookahead == '\\') ADVANCE(280);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(342);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(343);
      END_STATE();
    case 343:
      ACCEPT_TOKEN(aux_sym_doc_text_token1);
      if (lookahead == '\\') ADVANCE(280);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(343);
      END_STATE();
    case 344:
      ACCEPT_TOKEN(aux_sym_doc_text_token1);
      if (lookahead != 0 &&
          lookahead != '\\') ADVANCE(343);
      if (lookahead == '\\') ADVANCE(280);
      END_STATE();
    case 345:
      ACCEPT_TOKEN(anon_sym_SEMI_SEMI_SEMI);
      END_STATE();
    case 346:
      ACCEPT_TOKEN(sym_long_doc_text);
      if (lookahead == '\t' ||
          lookahead == '\n' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(346);
      if (lookahead != 0 &&
          lookahead != ']') ADVANCE(347);
      END_STATE();
    case 347:
      ACCEPT_TOKEN(sym_long_doc_text);
      if (lookahead != 0 &&
          lookahead != ']') ADVANCE(347);
      END_STATE();
    case 348:
      ACCEPT_TOKEN(anon_sym_LBRACK_LBRACK);
      END_STATE();
    case 349:
      ACCEPT_TOKEN(anon_sym_RBRACK_RBRACK);
      END_STATE();
    case 350:
      ACCEPT_TOKEN(anon_sym_SEMI);
      END_STATE();
    case 351:
      ACCEPT_TOKEN(anon_sym_SEMI);
      if (lookahead == ';') ADVANCE(18);
      END_STATE();
    case 352:
      ACCEPT_TOKEN(anon_sym_SLASH_SLASH);
      END_STATE();
    case 353:
      ACCEPT_TOKEN(anon_sym_SLASH_STAR);
      END_STATE();
    case 354:
      ACCEPT_TOKEN(aux_sym_comment_token1);
      if (lookahead == '*') ADVANCE(354);
      if (lookahead != 0 &&
          lookahead != '/') ADVANCE(13);
      END_STATE();
    case 355:
      ACCEPT_TOKEN(anon_sym_SLASH);
      END_STATE();
    case 356:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '*') ADVANCE(353);
      if (lookahead == '/') ADVANCE(352);
      END_STATE();
    case 357:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 358:
      ACCEPT_TOKEN(anon_sym_PLUS);
      END_STATE();
    case 359:
      ACCEPT_TOKEN(anon_sym_PLUS);
      if (lookahead == '+') ADVANCE(423);
      END_STATE();
    case 360:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 361:
      ACCEPT_TOKEN(anon_sym_DASH);
      if (lookahead == '-') ADVANCE(422);
      END_STATE();
    case 362:
      ACCEPT_TOKEN(anon_sym_PERCENT);
      END_STATE();
    case 363:
      ACCEPT_TOKEN(anon_sym_PIPE_PIPE);
      END_STATE();
    case 364:
      ACCEPT_TOKEN(anon_sym_AMP_AMP);
      END_STATE();
    case 365:
      ACCEPT_TOKEN(anon_sym_PIPE);
      if (lookahead == '|') ADVANCE(363);
      END_STATE();
    case 366:
      ACCEPT_TOKEN(anon_sym_CARET);
      END_STATE();
    case 367:
      ACCEPT_TOKEN(anon_sym_AMP);
      if (lookahead == '&') ADVANCE(364);
      END_STATE();
    case 368:
      ACCEPT_TOKEN(anon_sym_EQ_EQ);
      END_STATE();
    case 369:
      ACCEPT_TOKEN(anon_sym_BANG_EQ);
      END_STATE();
    case 370:
      ACCEPT_TOKEN(anon_sym_GT);
      END_STATE();
    case 371:
      ACCEPT_TOKEN(anon_sym_GT);
      if (lookahead == '=') ADVANCE(372);
      if (lookahead == '>') ADVANCE(376);
      END_STATE();
    case 372:
      ACCEPT_TOKEN(anon_sym_GT_EQ);
      END_STATE();
    case 373:
      ACCEPT_TOKEN(anon_sym_LT_EQ);
      END_STATE();
    case 374:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '<') ADVANCE(375);
      if (lookahead == '=') ADVANCE(373);
      END_STATE();
    case 375:
      ACCEPT_TOKEN(anon_sym_LT_LT);
      END_STATE();
    case 376:
      ACCEPT_TOKEN(anon_sym_GT_GT);
      END_STATE();
    case 377:
      ACCEPT_TOKEN(anon_sym_BANG);
      END_STATE();
    case 378:
      ACCEPT_TOKEN(anon_sym_BANG);
      if (lookahead == '=') ADVANCE(369);
      END_STATE();
    case 379:
      ACCEPT_TOKEN(anon_sym_TILDE);
      END_STATE();
    case 380:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 381:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(380);
      END_STATE();
    case 382:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(381);
      END_STATE();
    case 383:
      ACCEPT_TOKEN(sym_escape_sequence);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(275);
      END_STATE();
    case 384:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 385:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead == '\t' ||
          lookahead == '\r' ||
          lookahead == ' ') ADVANCE(385);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(386);
      END_STATE();
    case 386:
      ACCEPT_TOKEN(aux_sym_string_literal_token1);
      if (lookahead != 0 &&
          lookahead != '\n' &&
          lookahead != '"' &&
          lookahead != '\\') ADVANCE(386);
      END_STATE();
    case 387:
      ACCEPT_TOKEN(anon_sym_SQUOTE);
      END_STATE();
    case 388:
      ACCEPT_TOKEN(aux_sym_char_literal_token1);
      END_STATE();
    case 389:
      ACCEPT_TOKEN(aux_sym_char_literal_token1);
      if (lookahead == '\\') ADVANCE(233);
      END_STATE();
    case 390:
      ACCEPT_TOKEN(aux_sym_char_literal_token1);
      if (lookahead == 'u') ADVANCE(277);
      if (lookahead == 'x') ADVANCE(273);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(382);
      if (lookahead != 0) ADVANCE(380);
      END_STATE();
    case 391:
      ACCEPT_TOKEN(sym__line_break);
      if (lookahead == '\n') ADVANCE(391);
      if (lookahead == '\r') ADVANCE(1);
      END_STATE();
    case 392:
      ACCEPT_TOKEN(sym__line_break);
      if (lookahead == '\n') ADVANCE(392);
      if (lookahead == '\r') ADVANCE(4);
      END_STATE();
    case 393:
      ACCEPT_TOKEN(sym__line_break);
      if (lookahead == '\n') ADVANCE(393);
      if (lookahead == '\r') ADVANCE(5);
      END_STATE();
    case 394:
      ACCEPT_TOKEN(sym_reg_list_mnemonics);
      END_STATE();
    case 395:
      ACCEPT_TOKEN(sym_reg_list_mnemonics);
      if (lookahead == 'S' ||
          lookahead == 'U') ADVANCE(397);
      END_STATE();
    case 396:
      ACCEPT_TOKEN(sym_reg_list_mnemonics);
      if (lookahead == 's' ||
          lookahead == 'u') ADVANCE(397);
      END_STATE();
    case 397:
      ACCEPT_TOKEN(sym_regset_mnemonics);
      END_STATE();
    case 398:
      ACCEPT_TOKEN(sym_regset_mnemonics);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 399:
      ACCEPT_TOKEN(sym_xfer_mnemonics);
      END_STATE();
    case 400:
      ACCEPT_TOKEN(sym_xfer_mnemonics);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 401:
      ACCEPT_TOKEN(anon_sym_POUND);
      END_STATE();
    case 402:
      ACCEPT_TOKEN(aux_sym_equate_token1);
      END_STATE();
    case 403:
      ACCEPT_TOKEN(sym_a);
      END_STATE();
    case 404:
      ACCEPT_TOKEN(sym_a);
      if (lookahead == 'B') ADVANCE(117);
      if (lookahead == 'D') ADVANCE(43);
      if (lookahead == 'N') ADVANCE(47);
      if (lookahead == 'S') ADVANCE(71);
      END_STATE();
    case 405:
      ACCEPT_TOKEN(sym_b);
      END_STATE();
    case 406:
      ACCEPT_TOKEN(sym_b);
      if (lookahead == 'C') ADVANCE(243);
      if (lookahead == 'E') ADVANCE(96);
      if (lookahead == 'G') ADVANCE(246);
      if (lookahead == 'H') ADVANCE(247);
      if (lookahead == 'I') ADVANCE(109);
      if (lookahead == 'L') ADVANCE(264);
      if (lookahead == 'M') ADVANCE(66);
      if (lookahead == 'N') ADVANCE(53);
      if (lookahead == 'P') ADVANCE(70);
      if (lookahead == 'R') ADVANCE(242);
      if (lookahead == 'S') ADVANCE(99);
      if (lookahead == 'V') ADVANCE(243);
      END_STATE();
    case 407:
      ACCEPT_TOKEN(sym_d);
      if (lookahead == 'A') ADVANCE(22);
      if (lookahead == 'E') ADVANCE(44);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(418);
      END_STATE();
    case 408:
      ACCEPT_TOKEN(sym_d);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(418);
      END_STATE();
    case 409:
      ACCEPT_TOKEN(sym_x);
      END_STATE();
    case 410:
      ACCEPT_TOKEN(sym_y);
      END_STATE();
    case 411:
      ACCEPT_TOKEN(sym_u);
      END_STATE();
    case 412:
      ACCEPT_TOKEN(sym_s);
      END_STATE();
    case 413:
      ACCEPT_TOKEN(sym_s);
      if (lookahead == 'B') ADVANCE(42);
      if (lookahead == 'C') ADVANCE(87);
      if (lookahead == 'E') ADVANCE(112);
      if (lookahead == 'T') ADVANCE(268);
      if (lookahead == 'U') ADVANCE(29);
      if (lookahead == 'W') ADVANCE(67);
      if (lookahead == 'Y') ADVANCE(86);
      END_STATE();
    case 414:
      ACCEPT_TOKEN(sym_s);
      if (lookahead == 'b') ADVANCE(144);
      if (lookahead == 'c') ADVANCE(199);
      if (lookahead == 'e') ADVANCE(230);
      if (lookahead == 't') ADVANCE(270);
      if (lookahead == 'u') ADVANCE(136);
      if (lookahead == 'w') ADVANCE(176);
      if (lookahead == 'y') ADVANCE(195);
      END_STATE();
    case 415:
      ACCEPT_TOKEN(sym_pc);
      if (lookahead == 'R' ||
          lookahead == 'r') ADVANCE(416);
      END_STATE();
    case 416:
      ACCEPT_TOKEN(sym_pcr);
      END_STATE();
    case 417:
      ACCEPT_TOKEN(sym_cc);
      END_STATE();
    case 418:
      ACCEPT_TOKEN(sym_dp);
      END_STATE();
    case 419:
      ACCEPT_TOKEN(anon_sym_pc);
      if (lookahead == 'R') ADVANCE(416);
      if (lookahead == 'r') ADVANCE(421);
      END_STATE();
    case 420:
      ACCEPT_TOKEN(anon_sym_pc);
      if (lookahead == 'r') ADVANCE(421);
      END_STATE();
    case 421:
      ACCEPT_TOKEN(anon_sym_pcr);
      END_STATE();
    case 422:
      ACCEPT_TOKEN(anon_sym_DASH_DASH);
      END_STATE();
    case 423:
      ACCEPT_TOKEN(anon_sym_PLUS_PLUS);
      END_STATE();
    case 424:
      ACCEPT_TOKEN(anon_sym_a);
      if (lookahead == 'b') ADVANCE(240);
      if (lookahead == 'd') ADVANCE(145);
      if (lookahead == 'n') ADVANCE(152);
      if (lookahead == 's') ADVANCE(180);
      END_STATE();
    case 425:
      ACCEPT_TOKEN(anon_sym_a);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 426:
      ACCEPT_TOKEN(anon_sym_b);
      if (lookahead == 'c') ADVANCE(250);
      if (lookahead == 'e') ADVANCE(210);
      if (lookahead == 'g') ADVANCE(251);
      if (lookahead == 'h') ADVANCE(252);
      if (lookahead == 'i') ADVANCE(226);
      if (lookahead == 'l') ADVANCE(266);
      if (lookahead == 'm') ADVANCE(175);
      if (lookahead == 'n') ADVANCE(161);
      if (lookahead == 'p') ADVANCE(179);
      if (lookahead == 'r') ADVANCE(249);
      if (lookahead == 's') ADVANCE(213);
      if (lookahead == 'v') ADVANCE(250);
      if (lookahead == 'y') ADVANCE(231);
      END_STATE();
    case 427:
      ACCEPT_TOKEN(anon_sym_b);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 428:
      ACCEPT_TOKEN(anon_sym_d);
      if (lookahead == 'a') ADVANCE(124);
      if (lookahead == 'e') ADVANCE(146);
      if (lookahead == 'w') ADVANCE(202);
      if (lookahead == 'P' ||
          lookahead == 'p') ADVANCE(418);
      END_STATE();
    case 429:
      ACCEPT_TOKEN(anon_sym_d);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 430:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      END_STATE();
    case 431:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'A') ADVANCE(430);
      if (lookahead == 'B') ADVANCE(433);
      if (lookahead == 'L') ADVANCE(116);
      END_STATE();
    case 432:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'A') ADVANCE(445);
      if (lookahead == 'B') ADVANCE(434);
      if (lookahead == 'L') ADVANCE(536);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('C' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 433:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'I') ADVANCE(84);
      END_STATE();
    case 434:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'I') ADVANCE(507);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 435:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'a') ADVANCE(430);
      if (lookahead == 'b') ADVANCE(437);
      if (lookahead == 'l') ADVANCE(236);
      END_STATE();
    case 436:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'a') ADVANCE(445);
      if (lookahead == 'b') ADVANCE(438);
      if (lookahead == 'l') ADVANCE(633);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 437:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'i') ADVANCE(193);
      END_STATE();
    case 438:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'i') ADVANCE(602);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 439:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == '2' ||
          lookahead == '3') ADVANCE(430);
      END_STATE();
    case 440:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == '2' ||
          lookahead == '3') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 441:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(430);
      END_STATE();
    case 442:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('C' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 443:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(430);
      END_STATE();
    case 444:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 445:
      ACCEPT_TOKEN(aux_sym_mnemonic_token1);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 446:
      ACCEPT_TOKEN(anon_sym_COLON_COLON);
      END_STATE();
    case 447:
      ACCEPT_TOKEN(aux_sym_local_label_token1);
      END_STATE();
    case 448:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A') ADVANCE(490);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 449:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 450:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A') ADVANCE(655);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 451:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A') ADVANCE(449);
      if (lookahead == 'E') ADVANCE(470);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 452:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A') ADVANCE(474);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 453:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A') ADVANCE(461);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 454:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(537);
      if (lookahead == 'D') ADVANCE(468);
      if (lookahead == 'N') ADVANCE(473);
      if (lookahead == 'S') ADVANCE(495);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 455:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(465);
      if (lookahead == 'D') ADVANCE(658);
      if (lookahead == 'E') ADVANCE(450);
      if (lookahead == 'S') ADVANCE(495);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 456:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(652);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 457:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(307);
      if (lookahead == 'C') ADVANCE(309);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 458:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(305);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 459:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(315);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 460:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(311);
      if (lookahead == 'D') ADVANCE(313);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 461:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(506);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 462:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(469);
      if (lookahead == 'C') ADVANCE(510);
      if (lookahead == 'E') ADVANCE(533);
      if (lookahead == 'T') ADVANCE(658);
      if (lookahead == 'U') ADVANCE(456);
      if (lookahead == 'W') ADVANCE(491);
      if (lookahead == 'Y') ADVANCE(509);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 463:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'B') ADVANCE(493);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 464:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(637);
      if (lookahead == 'E') ADVANCE(518);
      if (lookahead == 'G') ADVANCE(638);
      if (lookahead == 'H') ADVANCE(639);
      if (lookahead == 'I') ADVANCE(530);
      if (lookahead == 'L') ADVANCE(654);
      if (lookahead == 'M') ADVANCE(490);
      if (lookahead == 'N') ADVANCE(479);
      if (lookahead == 'P') ADVANCE(494);
      if (lookahead == 'R') ADVANCE(636);
      if (lookahead == 'S') ADVANCE(520);
      if (lookahead == 'V') ADVANCE(637);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 465:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(637);
      if (lookahead == 'E') ADVANCE(518);
      if (lookahead == 'G') ADVANCE(638);
      if (lookahead == 'H') ADVANCE(639);
      if (lookahead == 'L') ADVANCE(654);
      if (lookahead == 'M') ADVANCE(490);
      if (lookahead == 'N') ADVANCE(479);
      if (lookahead == 'P') ADVANCE(494);
      if (lookahead == 'R') ADVANCE(636);
      if (lookahead == 'S') ADVANCE(521);
      if (lookahead == 'V') ADVANCE(637);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 466:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(457);
      if (lookahead == 'D') ADVANCE(458);
      if (lookahead == 'I') ADVANCE(498);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 467:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 468:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(648);
      if (lookahead == 'D') ADVANCE(652);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 469:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(648);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 470:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(442);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 471:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(432);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 472:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C') ADVANCE(538);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 473:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'D') ADVANCE(647);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 474:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'D') ADVANCE(477);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 475:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'D') ADVANCE(481);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 476:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'D') ADVANCE(515);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 477:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'D') ADVANCE(525);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 478:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(488);
      if (lookahead == 'O') ADVANCE(514);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 479:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 480:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(284);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 481:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(323);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 482:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(486);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 483:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(502);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 484:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(472);
      if (lookahead == 'G') ADVANCE(400);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 485:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E') ADVANCE(463);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 486:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'F') ADVANCE(299);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 487:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'F') ADVANCE(524);
      if (lookahead == 'S') ADVANCE(531);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 488:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'G') ADVANCE(442);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 489:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'H') ADVANCE(640);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 490:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'I') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 491:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'I') ADVANCE(440);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 492:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'I') ADVANCE(534);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 493:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'I') ADVANCE(508);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 494:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'L') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 495:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'L') ADVANCE(442);
      if (lookahead == 'R') ADVANCE(442);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 496:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'L') ADVANCE(640);
      if (lookahead == 'T') ADVANCE(286);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 497:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'L') ADVANCE(303);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 498:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'L') ADVANCE(497);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 499:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'L') ADVANCE(523);
      if (lookahead == 'M') ADVANCE(513);
      if (lookahead == 'O') ADVANCE(501);
      if (lookahead == 'W') ADVANCE(448);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 500:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'M') ADVANCE(460);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 501:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'M') ADVANCE(442);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 502:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'M') ADVANCE(288);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 503:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'M') ADVANCE(517);
      if (lookahead == 'N') ADVANCE(471);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 504:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'M') ADVANCE(459);
      if (lookahead == 'O') ADVANCE(495);
      if (lookahead == 'T') ADVANCE(639);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 505:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'M') ADVANCE(514);
      if (lookahead == 'S') ADVANCE(521);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 506:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'M') ADVANCE(483);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 507:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'N') ADVANCE(293);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 508:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'N') ADVANCE(291);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 509:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'N') ADVANCE(467);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 510:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'O') ADVANCE(516);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 511:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'O') ADVANCE(522);
      if (lookahead == 'X') ADVANCE(484);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 512:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'O') ADVANCE(528);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 513:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'P') ADVANCE(658);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 514:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'P') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 515:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'P') ADVANCE(317);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 516:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'P') ADVANCE(480);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 517:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'P') ADVANCE(512);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 518:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'Q') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 519:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(646);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 520:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(445);
      if (lookahead == 'Z') ADVANCE(301);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Y') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 521:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 522:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(648);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 523:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(442);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 524:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(400);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 525:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(321);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 526:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(492);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 527:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(453);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 528:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'R') ADVANCE(532);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 529:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'S') ADVANCE(489);
      if (lookahead == 'U') ADVANCE(496);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 530:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'T') ADVANCE(648);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 531:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'T') ADVANCE(442);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 532:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'T') ADVANCE(297);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 533:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'T') ADVANCE(476);
      if (lookahead == 'X') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 534:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'T') ADVANCE(485);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 535:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'U') ADVANCE(494);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 536:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'U') ADVANCE(475);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 537:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'X') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 538:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == '_') ADVANCE(452);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 539:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == '_') ADVANCE(545);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 540:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a') ADVANCE(585);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 541:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 542:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a') ADVANCE(657);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 543:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a') ADVANCE(541);
      if (lookahead == 'e') ADVANCE(563);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 544:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a') ADVANCE(567);
      if (lookahead == 'u') ADVANCE(589);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 545:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a') ADVANCE(569);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 546:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a') ADVANCE(554);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 547:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(635);
      if (lookahead == 'd') ADVANCE(561);
      if (lookahead == 'n') ADVANCE(568);
      if (lookahead == 's') ADVANCE(590);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 548:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(558);
      if (lookahead == 'd') ADVANCE(660);
      if (lookahead == 'e') ADVANCE(542);
      if (lookahead == 's') ADVANCE(590);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 549:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(307);
      if (lookahead == 'c') ADVANCE(309);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 550:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(305);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 551:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(315);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 552:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(311);
      if (lookahead == 'd') ADVANCE(313);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 553:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(653);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 554:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(601);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 555:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(588);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 556:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'b') ADVANCE(562);
      if (lookahead == 'c') ADVANCE(606);
      if (lookahead == 'e') ADVANCE(631);
      if (lookahead == 't') ADVANCE(659);
      if (lookahead == 'u') ADVANCE(553);
      if (lookahead == 'w') ADVANCE(586);
      if (lookahead == 'y') ADVANCE(604);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 557:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(642);
      if (lookahead == 'e') ADVANCE(614);
      if (lookahead == 'g') ADVANCE(643);
      if (lookahead == 'h') ADVANCE(644);
      if (lookahead == 'i') ADVANCE(627);
      if (lookahead == 'l') ADVANCE(656);
      if (lookahead == 'm') ADVANCE(585);
      if (lookahead == 'n') ADVANCE(574);
      if (lookahead == 'p') ADVANCE(589);
      if (lookahead == 'r') ADVANCE(641);
      if (lookahead == 's') ADVANCE(616);
      if (lookahead == 'v') ADVANCE(642);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 558:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(642);
      if (lookahead == 'e') ADVANCE(614);
      if (lookahead == 'g') ADVANCE(643);
      if (lookahead == 'h') ADVANCE(644);
      if (lookahead == 'l') ADVANCE(656);
      if (lookahead == 'm') ADVANCE(585);
      if (lookahead == 'n') ADVANCE(574);
      if (lookahead == 'p') ADVANCE(589);
      if (lookahead == 'r') ADVANCE(641);
      if (lookahead == 's') ADVANCE(617);
      if (lookahead == 'v') ADVANCE(642);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 559:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(549);
      if (lookahead == 'd') ADVANCE(550);
      if (lookahead == 'i') ADVANCE(593);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 560:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 561:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(651);
      if (lookahead == 'd') ADVANCE(653);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 562:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(651);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 563:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(444);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 564:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(436);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 565:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(539);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 566:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(630);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 567:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c') ADVANCE(624);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 568:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'd') ADVANCE(650);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 569:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'd') ADVANCE(572);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 570:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'd') ADVANCE(576);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 571:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'd') ADVANCE(611);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 572:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'd') ADVANCE(621);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 573:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(583);
      if (lookahead == 'o') ADVANCE(610);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 574:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 575:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(284);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 576:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(323);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 577:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(581);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 578:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(597);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 579:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(565);
      if (lookahead == 'g') ADVANCE(400);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 580:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e') ADVANCE(555);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 581:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'f') ADVANCE(299);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 582:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'f') ADVANCE(618);
      if (lookahead == 's') ADVANCE(628);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 583:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'g') ADVANCE(444);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 584:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'h') ADVANCE(645);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 585:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'i') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 586:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'i') ADVANCE(440);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 587:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'i') ADVANCE(632);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 588:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'i') ADVANCE(603);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 589:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'l') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 590:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'l') ADVANCE(444);
      if (lookahead == 'r') ADVANCE(444);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 591:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'l') ADVANCE(645);
      if (lookahead == 't') ADVANCE(286);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 592:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'l') ADVANCE(303);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 593:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'l') ADVANCE(592);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 594:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'l') ADVANCE(620);
      if (lookahead == 'm') ADVANCE(609);
      if (lookahead == 'o') ADVANCE(596);
      if (lookahead == 'w') ADVANCE(540);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 595:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'm') ADVANCE(552);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 596:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'm') ADVANCE(444);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 597:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'm') ADVANCE(288);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 598:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'm') ADVANCE(613);
      if (lookahead == 'n') ADVANCE(564);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 599:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'm') ADVANCE(551);
      if (lookahead == 'o') ADVANCE(590);
      if (lookahead == 't') ADVANCE(644);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 600:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'm') ADVANCE(610);
      if (lookahead == 's') ADVANCE(617);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 601:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'm') ADVANCE(578);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 602:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'n') ADVANCE(294);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 603:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'n') ADVANCE(291);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 604:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'n') ADVANCE(560);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 605:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'o') ADVANCE(339);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 606:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'o') ADVANCE(612);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 607:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'o') ADVANCE(619);
      if (lookahead == 'x') ADVANCE(579);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 608:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'o') ADVANCE(625);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 609:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'p') ADVANCE(660);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 610:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'p') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 611:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'p') ADVANCE(317);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 612:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'p') ADVANCE(575);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 613:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'p') ADVANCE(608);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 614:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'q') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 615:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(649);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 616:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(445);
      if (lookahead == 'z') ADVANCE(301);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'y')) ADVANCE(661);
      END_STATE();
    case 617:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 618:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(400);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 619:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(651);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 620:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(444);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 621:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(321);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 622:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(587);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 623:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(546);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 624:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(605);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 625:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'r') ADVANCE(629);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 626:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 's') ADVANCE(584);
      if (lookahead == 'u') ADVANCE(591);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 627:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 't') ADVANCE(651);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 628:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 't') ADVANCE(444);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 629:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 't') ADVANCE(297);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 630:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 't') ADVANCE(325);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 631:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 't') ADVANCE(571);
      if (lookahead == 'x') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 632:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 't') ADVANCE(580);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 633:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'u') ADVANCE(570);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 634:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'u') ADVANCE(566);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 635:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'x') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 636:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A' ||
          lookahead == 'N') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('B' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 637:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'C' ||
          lookahead == 'S') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 638:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E' ||
          lookahead == 'T') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 639:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'I' ||
          lookahead == 'S') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 640:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'S' ||
          lookahead == 'U') ADVANCE(398);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 641:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a' ||
          lookahead == 'n') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 642:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'c' ||
          lookahead == 's') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 643:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e' ||
          lookahead == 't') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 644:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'i' ||
          lookahead == 's') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 645:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 's' ||
          lookahead == 'u') ADVANCE(398);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 646:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(445);
      if (lookahead == 'C') ADVANCE(467);
      if (lookahead == 'G') ADVANCE(319);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('D' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 647:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(445);
      if (lookahead == 'C') ADVANCE(467);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('D' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 648:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A' ||
          lookahead == 'B') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('C' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 649:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(445);
      if (lookahead == 'c') ADVANCE(560);
      if (lookahead == 'g') ADVANCE(319);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('d' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 650:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(445);
      if (lookahead == 'c') ADVANCE(560);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('d' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 651:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a' ||
          lookahead == 'b') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 652:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A' ||
          lookahead == 'B' ||
          lookahead == 'D') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('C' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 653:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a' ||
          lookahead == 'b' ||
          lookahead == 'd') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 654:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'E' ||
          lookahead == 'O' ||
          lookahead == 'S' ||
          lookahead == 'T') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 655:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'S' ||
          lookahead == 'U' ||
          lookahead == 'X' ||
          lookahead == 'Y') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 656:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'e' ||
          lookahead == 'o' ||
          lookahead == 's' ||
          lookahead == 't') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 657:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 's' ||
          lookahead == 'u' ||
          lookahead == 'x' ||
          lookahead == 'y') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 658:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'A' ||
          lookahead == 'B' ||
          lookahead == 'D' ||
          lookahead == 'S' ||
          lookahead == 'U' ||
          lookahead == 'X' ||
          lookahead == 'Y') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('C' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 659:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a' ||
          lookahead == 'b' ||
          lookahead == 'd' ||
          lookahead == 's' ||
          lookahead == 'u' ||
          lookahead == 'x' ||
          lookahead == 'y') ADVANCE(445);
      if (lookahead == 'r') ADVANCE(634);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 660:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == 'a' ||
          lookahead == 'b' ||
          lookahead == 'd' ||
          lookahead == 's' ||
          lookahead == 'u' ||
          lookahead == 'x' ||
          lookahead == 'y') ADVANCE(445);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('c' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 661:
      ACCEPT_TOKEN(sym__global_label);
      if (lookahead == '.' ||
          ('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(661);
      END_STATE();
    case 662:
      ACCEPT_TOKEN(sym_bin_num);
      if (lookahead == '_') ADVANCE(255);
      END_STATE();
    case 663:
      ACCEPT_TOKEN(sym_bin_num);
      if (lookahead == '_') ADVANCE(255);
      if (lookahead == '0' ||
          lookahead == '1') ADVANCE(663);
      END_STATE();
    case 664:
      ACCEPT_TOKEN(sym_hex_num);
      if (lookahead == '_') ADVANCE(279);
      END_STATE();
    case 665:
      ACCEPT_TOKEN(sym_hex_num);
      if (lookahead == '_') ADVANCE(279);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'F') ||
          ('a' <= lookahead && lookahead <= 'f')) ADVANCE(665);
      END_STATE();
    case 666:
      ACCEPT_TOKEN(sym_dec_num);
      if (lookahead == '_') ADVANCE(669);
      if (lookahead == 'b') ADVANCE(254);
      if (lookahead == 'x') ADVANCE(278);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(668);
      END_STATE();
    case 667:
      ACCEPT_TOKEN(sym_dec_num);
      if (lookahead == '_') ADVANCE(669);
      if (lookahead == 'x') ADVANCE(278);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(668);
      END_STATE();
    case 668:
      ACCEPT_TOKEN(sym_dec_num);
      if (lookahead == '_') ADVANCE(669);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(668);
      END_STATE();
    case 669:
      ACCEPT_TOKEN(sym_dec_num);
      if (('0' <= lookahead && lookahead <= '9') ||
          lookahead == '_') ADVANCE(669);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 281},
  [2] = {.lex_state = 281},
  [3] = {.lex_state = 281},
  [4] = {.lex_state = 15},
  [5] = {.lex_state = 15},
  [6] = {.lex_state = 15},
  [7] = {.lex_state = 15},
  [8] = {.lex_state = 15},
  [9] = {.lex_state = 1},
  [10] = {.lex_state = 1},
  [11] = {.lex_state = 4},
  [12] = {.lex_state = 1},
  [13] = {.lex_state = 1},
  [14] = {.lex_state = 7},
  [15] = {.lex_state = 281},
  [16] = {.lex_state = 281},
  [17] = {.lex_state = 281},
  [18] = {.lex_state = 281},
  [19] = {.lex_state = 281},
  [20] = {.lex_state = 281},
  [21] = {.lex_state = 281},
  [22] = {.lex_state = 281},
  [23] = {.lex_state = 281},
  [24] = {.lex_state = 281},
  [25] = {.lex_state = 281},
  [26] = {.lex_state = 281},
  [27] = {.lex_state = 281},
  [28] = {.lex_state = 281},
  [29] = {.lex_state = 15},
  [30] = {.lex_state = 15},
  [31] = {.lex_state = 15},
  [32] = {.lex_state = 15},
  [33] = {.lex_state = 15},
  [34] = {.lex_state = 15},
  [35] = {.lex_state = 15},
  [36] = {.lex_state = 15},
  [37] = {.lex_state = 15},
  [38] = {.lex_state = 15},
  [39] = {.lex_state = 15},
  [40] = {.lex_state = 1},
  [41] = {.lex_state = 1},
  [42] = {.lex_state = 1},
  [43] = {.lex_state = 1},
  [44] = {.lex_state = 1},
  [45] = {.lex_state = 1},
  [46] = {.lex_state = 1},
  [47] = {.lex_state = 1},
  [48] = {.lex_state = 1},
  [49] = {.lex_state = 1},
  [50] = {.lex_state = 1},
  [51] = {.lex_state = 1},
  [52] = {.lex_state = 1},
  [53] = {.lex_state = 1},
  [54] = {.lex_state = 8},
  [55] = {.lex_state = 1},
  [56] = {.lex_state = 1},
  [57] = {.lex_state = 8},
  [58] = {.lex_state = 1},
  [59] = {.lex_state = 4},
  [60] = {.lex_state = 1},
  [61] = {.lex_state = 1},
  [62] = {.lex_state = 1},
  [63] = {.lex_state = 1},
  [64] = {.lex_state = 1},
  [65] = {.lex_state = 1},
  [66] = {.lex_state = 1},
  [67] = {.lex_state = 1},
  [68] = {.lex_state = 1},
  [69] = {.lex_state = 1},
  [70] = {.lex_state = 1},
  [71] = {.lex_state = 1},
  [72] = {.lex_state = 6},
  [73] = {.lex_state = 1},
  [74] = {.lex_state = 1},
  [75] = {.lex_state = 1},
  [76] = {.lex_state = 1},
  [77] = {.lex_state = 1},
  [78] = {.lex_state = 1},
  [79] = {.lex_state = 1},
  [80] = {.lex_state = 6},
  [81] = {.lex_state = 6},
  [82] = {.lex_state = 6},
  [83] = {.lex_state = 6},
  [84] = {.lex_state = 6},
  [85] = {.lex_state = 6},
  [86] = {.lex_state = 6},
  [87] = {.lex_state = 6},
  [88] = {.lex_state = 6},
  [89] = {.lex_state = 6},
  [90] = {.lex_state = 6},
  [91] = {.lex_state = 6},
  [92] = {.lex_state = 6},
  [93] = {.lex_state = 6},
  [94] = {.lex_state = 6},
  [95] = {.lex_state = 6},
  [96] = {.lex_state = 6},
  [97] = {.lex_state = 6},
  [98] = {.lex_state = 6},
  [99] = {.lex_state = 6},
  [100] = {.lex_state = 6},
  [101] = {.lex_state = 6},
  [102] = {.lex_state = 6},
  [103] = {.lex_state = 6},
  [104] = {.lex_state = 6},
  [105] = {.lex_state = 6},
  [106] = {.lex_state = 6},
  [107] = {.lex_state = 6},
  [108] = {.lex_state = 6},
  [109] = {.lex_state = 6},
  [110] = {.lex_state = 6},
  [111] = {.lex_state = 6},
  [112] = {.lex_state = 6},
  [113] = {.lex_state = 6},
  [114] = {.lex_state = 6},
  [115] = {.lex_state = 6},
  [116] = {.lex_state = 6},
  [117] = {.lex_state = 6},
  [118] = {.lex_state = 6},
  [119] = {.lex_state = 6},
  [120] = {.lex_state = 6},
  [121] = {.lex_state = 6},
  [122] = {.lex_state = 6},
  [123] = {.lex_state = 6},
  [124] = {.lex_state = 6},
  [125] = {.lex_state = 6},
  [126] = {.lex_state = 6},
  [127] = {.lex_state = 6},
  [128] = {.lex_state = 6},
  [129] = {.lex_state = 8},
  [130] = {.lex_state = 8},
  [131] = {.lex_state = 8},
  [132] = {.lex_state = 8},
  [133] = {.lex_state = 8},
  [134] = {.lex_state = 8},
  [135] = {.lex_state = 8},
  [136] = {.lex_state = 8},
  [137] = {.lex_state = 8},
  [138] = {.lex_state = 8},
  [139] = {.lex_state = 8},
  [140] = {.lex_state = 8},
  [141] = {.lex_state = 8},
  [142] = {.lex_state = 8},
  [143] = {.lex_state = 8},
  [144] = {.lex_state = 8},
  [145] = {.lex_state = 8},
  [146] = {.lex_state = 8},
  [147] = {.lex_state = 8},
  [148] = {.lex_state = 8},
  [149] = {.lex_state = 8},
  [150] = {.lex_state = 8},
  [151] = {.lex_state = 8},
  [152] = {.lex_state = 8},
  [153] = {.lex_state = 8},
  [154] = {.lex_state = 8},
  [155] = {.lex_state = 10},
  [156] = {.lex_state = 10},
  [157] = {.lex_state = 10},
  [158] = {.lex_state = 1},
  [159] = {.lex_state = 1},
  [160] = {.lex_state = 1},
  [161] = {.lex_state = 1},
  [162] = {.lex_state = 1},
  [163] = {.lex_state = 1},
  [164] = {.lex_state = 1},
  [165] = {.lex_state = 1},
  [166] = {.lex_state = 1},
  [167] = {.lex_state = 10},
  [168] = {.lex_state = 10},
  [169] = {.lex_state = 10},
  [170] = {.lex_state = 1},
  [171] = {.lex_state = 1},
  [172] = {.lex_state = 16},
  [173] = {.lex_state = 16},
  [174] = {.lex_state = 1},
  [175] = {.lex_state = 16},
  [176] = {.lex_state = 1},
  [177] = {.lex_state = 1},
  [178] = {.lex_state = 1},
  [179] = {.lex_state = 5},
  [180] = {.lex_state = 1},
  [181] = {.lex_state = 1},
  [182] = {.lex_state = 10},
  [183] = {.lex_state = 10},
  [184] = {.lex_state = 1},
  [185] = {.lex_state = 16},
  [186] = {.lex_state = 1},
  [187] = {.lex_state = 1},
  [188] = {.lex_state = 8},
  [189] = {.lex_state = 1},
  [190] = {.lex_state = 1},
  [191] = {.lex_state = 1},
  [192] = {.lex_state = 1},
  [193] = {.lex_state = 1},
  [194] = {.lex_state = 1},
  [195] = {.lex_state = 1},
  [196] = {.lex_state = 10},
  [197] = {.lex_state = 1},
  [198] = {.lex_state = 1},
  [199] = {.lex_state = 16},
  [200] = {.lex_state = 10},
  [201] = {.lex_state = 10},
  [202] = {.lex_state = 8},
  [203] = {.lex_state = 10},
  [204] = {.lex_state = 8},
  [205] = {.lex_state = 1},
  [206] = {.lex_state = 1},
  [207] = {.lex_state = 1},
  [208] = {.lex_state = 10},
  [209] = {.lex_state = 1},
  [210] = {.lex_state = 1},
  [211] = {.lex_state = 1},
  [212] = {.lex_state = 1},
  [213] = {.lex_state = 8},
  [214] = {.lex_state = 8},
  [215] = {.lex_state = 1},
  [216] = {.lex_state = 1},
  [217] = {.lex_state = 10},
  [218] = {.lex_state = 8},
  [219] = {.lex_state = 1},
  [220] = {.lex_state = 1},
  [221] = {.lex_state = 1},
  [222] = {.lex_state = 8},
  [223] = {.lex_state = 1},
  [224] = {.lex_state = 1},
  [225] = {.lex_state = 1},
  [226] = {.lex_state = 1},
  [227] = {.lex_state = 1},
  [228] = {.lex_state = 1},
  [229] = {.lex_state = 8},
  [230] = {.lex_state = 8},
  [231] = {.lex_state = 2},
  [232] = {.lex_state = 2},
  [233] = {.lex_state = 2},
  [234] = {.lex_state = 15},
  [235] = {.lex_state = 2},
  [236] = {.lex_state = 2},
  [237] = {.lex_state = 0},
  [238] = {.lex_state = 0},
  [239] = {.lex_state = 0},
  [240] = {.lex_state = 0},
  [241] = {.lex_state = 0},
  [242] = {.lex_state = 281},
  [243] = {.lex_state = 0},
  [244] = {.lex_state = 0},
  [245] = {.lex_state = 0},
  [246] = {.lex_state = 0},
  [247] = {.lex_state = 0},
  [248] = {.lex_state = 0},
  [249] = {.lex_state = 0},
  [250] = {.lex_state = 0},
  [251] = {.lex_state = 3},
  [252] = {.lex_state = 342},
  [253] = {.lex_state = 0},
  [254] = {.lex_state = 3},
  [255] = {.lex_state = 0},
  [256] = {.lex_state = 0},
  [257] = {.lex_state = 0},
  [258] = {.lex_state = 0},
  [259] = {.lex_state = 342},
  [260] = {.lex_state = 0},
  [261] = {.lex_state = 281},
  [262] = {.lex_state = 0},
  [263] = {.lex_state = 0},
  [264] = {.lex_state = 0},
  [265] = {.lex_state = 0},
  [266] = {.lex_state = 0},
  [267] = {.lex_state = 0},
  [268] = {.lex_state = 1},
  [269] = {.lex_state = 12},
  [270] = {.lex_state = 281},
  [271] = {.lex_state = 281},
  [272] = {.lex_state = 1},
  [273] = {.lex_state = 281},
  [274] = {.lex_state = 281},
  [275] = {.lex_state = 281},
  [276] = {.lex_state = 281},
  [277] = {.lex_state = 281},
  [278] = {.lex_state = 1},
  [279] = {.lex_state = 0},
  [280] = {.lex_state = 0},
  [281] = {.lex_state = 6},
  [282] = {.lex_state = 0},
  [283] = {.lex_state = 281},
  [284] = {.lex_state = 7},
  [285] = {.lex_state = 15},
  [286] = {.lex_state = 342},
  [287] = {.lex_state = 6},
  [288] = {.lex_state = 0},
  [289] = {.lex_state = 1},
  [290] = {.lex_state = 0},
  [291] = {.lex_state = 7},
  [292] = {.lex_state = 281},
  [293] = {.lex_state = 1},
  [294] = {.lex_state = 0},
  [295] = {.lex_state = 0},
  [296] = {.lex_state = 6},
  [297] = {.lex_state = 0},
  [298] = {.lex_state = 0},
  [299] = {.lex_state = 0},
  [300] = {.lex_state = 0},
  [301] = {.lex_state = 0},
  [302] = {.lex_state = 0},
  [303] = {.lex_state = 0},
  [304] = {.lex_state = 0},
  [305] = {.lex_state = 346},
  [306] = {.lex_state = 0},
  [307] = {.lex_state = 1},
  [308] = {.lex_state = 342},
  [309] = {.lex_state = 7},
  [310] = {.lex_state = 0},
  [311] = {.lex_state = 0},
  [312] = {.lex_state = 0},
  [313] = {.lex_state = 0},
  [314] = {.lex_state = 15},
  [315] = {.lex_state = 12},
  [316] = {.lex_state = 342},
  [317] = {.lex_state = 12},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [ts_builtin_sym_end] = ACTIONS(1),
    [aux_sym_scope_token1] = ACTIONS(1),
    [aux_sym_put_token1] = ACTIONS(1),
    [aux_sym_grabmem_token1] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [aux_sym_writebin_token1] = ACTIONS(1),
    [aux_sym_incbin_token1] = ACTIONS(1),
    [aux_sym_importer_token1] = ACTIONS(1),
    [aux_sym_incbinref_token1] = ACTIONS(1),
    [aux_sym_bsz_token1] = ACTIONS(1),
    [aux_sym_fill_token1] = ACTIONS(1),
    [aux_sym_fdb_token1] = ACTIONS(1),
    [aux_sym_fcb_token1] = ACTIONS(1),
    [aux_sym_fcc_token1] = ACTIONS(1),
    [aux_sym_zmb_token1] = ACTIONS(1),
    [aux_sym_zmd_token1] = ACTIONS(1),
    [aux_sym_rmb_token1] = ACTIONS(1),
    [aux_sym_setdp_token1] = ACTIONS(1),
    [aux_sym_org_token1] = ACTIONS(1),
    [aux_sym_exec_addr_token1] = ACTIONS(1),
    [aux_sym_include_token1] = ACTIONS(1),
    [anon_sym_struct] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_byte] = ACTIONS(1),
    [anon_sym_word] = ACTIONS(1),
    [anon_sym_dword] = ACTIONS(1),
    [anon_sym_qword] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_macro] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_SEMI_SEMI_SEMI] = ACTIONS(1),
    [anon_sym_LBRACK_LBRACK] = ACTIONS(1),
    [anon_sym_RBRACK_RBRACK] = ACTIONS(1),
    [anon_sym_SEMI] = ACTIONS(1),
    [anon_sym_SLASH_SLASH] = ACTIONS(1),
    [anon_sym_SLASH_STAR] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [anon_sym_STAR] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_PERCENT] = ACTIONS(1),
    [anon_sym_PIPE_PIPE] = ACTIONS(1),
    [anon_sym_AMP_AMP] = ACTIONS(1),
    [anon_sym_PIPE] = ACTIONS(1),
    [anon_sym_CARET] = ACTIONS(1),
    [anon_sym_AMP] = ACTIONS(1),
    [anon_sym_EQ_EQ] = ACTIONS(1),
    [anon_sym_BANG_EQ] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_GT_EQ] = ACTIONS(1),
    [anon_sym_LT_EQ] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_LT_LT] = ACTIONS(1),
    [anon_sym_GT_GT] = ACTIONS(1),
    [anon_sym_BANG] = ACTIONS(1),
    [anon_sym_TILDE] = ACTIONS(1),
    [sym_escape_sequence] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [anon_sym_SQUOTE] = ACTIONS(1),
    [sym_reg_list_mnemonics] = ACTIONS(1),
    [sym_regset_mnemonics] = ACTIONS(1),
    [sym_xfer_mnemonics] = ACTIONS(1),
    [anon_sym_POUND] = ACTIONS(1),
    [aux_sym_equate_token1] = ACTIONS(1),
    [sym_a] = ACTIONS(1),
    [sym_b] = ACTIONS(1),
    [sym_d] = ACTIONS(1),
    [sym_x] = ACTIONS(1),
    [sym_y] = ACTIONS(1),
    [sym_u] = ACTIONS(1),
    [sym_s] = ACTIONS(1),
    [sym_pc] = ACTIONS(1),
    [sym_pcr] = ACTIONS(1),
    [sym_cc] = ACTIONS(1),
    [sym_dp] = ACTIONS(1),
    [anon_sym_pc] = ACTIONS(1),
    [anon_sym_pcr] = ACTIONS(1),
    [anon_sym_DASH_DASH] = ACTIONS(1),
    [anon_sym_PLUS_PLUS] = ACTIONS(1),
    [anon_sym_a] = ACTIONS(1),
    [anon_sym_b] = ACTIONS(1),
    [anon_sym_d] = ACTIONS(1),
    [aux_sym_mnemonic_token1] = ACTIONS(1),
    [anon_sym_COLON_COLON] = ACTIONS(1),
    [aux_sym_local_label_token1] = ACTIONS(1),
    [sym_hex_num] = ACTIONS(1),
    [sym_dec_num] = ACTIONS(1),
  },
  [1] = {
    [sym_source_file] = STATE(312),
    [sym_scope] = STATE(165),
    [sym_put] = STATE(165),
    [sym_grabmem] = STATE(165),
    [sym_writebin] = STATE(165),
    [sym_incbin] = STATE(165),
    [sym_importer] = STATE(165),
    [sym_incbinref] = STATE(165),
    [sym_bsz] = STATE(165),
    [sym_fill] = STATE(165),
    [sym_fdb] = STATE(165),
    [sym_fcb] = STATE(165),
    [sym_fcc] = STATE(165),
    [sym_zmb] = STATE(165),
    [sym_zmd] = STATE(165),
    [sym_rmb] = STATE(165),
    [sym_setdp] = STATE(165),
    [sym_org] = STATE(165),
    [sym_exec_addr] = STATE(165),
    [sym_include] = STATE(165),
    [sym__command] = STATE(165),
    [sym_struct_def] = STATE(3),
    [sym_macro_def] = STATE(3),
    [sym_doc] = STATE(3),
    [sym_long_doc] = STATE(3),
    [sym_comment] = STATE(3),
    [sym__line] = STATE(3),
    [sym__opcode_label] = STATE(165),
    [sym__command_label] = STATE(165),
    [sym_macro] = STATE(165),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(165),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(165),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(9),
    [sym_label] = STATE(9),
    [sym_local_label] = STATE(9),
    [aux_sym_source_file_repeat1] = STATE(3),
    [ts_builtin_sym_end] = ACTIONS(3),
    [aux_sym_scope_token1] = ACTIONS(5),
    [aux_sym_put_token1] = ACTIONS(7),
    [aux_sym_grabmem_token1] = ACTIONS(9),
    [aux_sym_writebin_token1] = ACTIONS(11),
    [aux_sym_incbin_token1] = ACTIONS(13),
    [aux_sym_importer_token1] = ACTIONS(15),
    [aux_sym_incbinref_token1] = ACTIONS(17),
    [aux_sym_bsz_token1] = ACTIONS(19),
    [aux_sym_fill_token1] = ACTIONS(21),
    [aux_sym_fdb_token1] = ACTIONS(23),
    [aux_sym_fcb_token1] = ACTIONS(25),
    [aux_sym_fcc_token1] = ACTIONS(27),
    [aux_sym_zmb_token1] = ACTIONS(29),
    [aux_sym_zmd_token1] = ACTIONS(31),
    [aux_sym_rmb_token1] = ACTIONS(33),
    [aux_sym_setdp_token1] = ACTIONS(35),
    [aux_sym_org_token1] = ACTIONS(37),
    [aux_sym_exec_addr_token1] = ACTIONS(39),
    [aux_sym_include_token1] = ACTIONS(41),
    [anon_sym_struct] = ACTIONS(43),
    [anon_sym_macro] = ACTIONS(45),
    [anon_sym_SEMI_SEMI_SEMI] = ACTIONS(47),
    [anon_sym_LBRACK_LBRACK] = ACTIONS(49),
    [anon_sym_SEMI] = ACTIONS(51),
    [anon_sym_SLASH_SLASH] = ACTIONS(53),
    [anon_sym_SLASH_STAR] = ACTIONS(55),
    [sym_regset_mnemonics] = ACTIONS(57),
    [sym_xfer_mnemonics] = ACTIONS(59),
    [aux_sym_mnemonic_token1] = ACTIONS(61),
    [aux_sym_local_label_token1] = ACTIONS(63),
    [sym__global_label] = ACTIONS(65),
  },
  [2] = {
    [sym_scope] = STATE(165),
    [sym_put] = STATE(165),
    [sym_grabmem] = STATE(165),
    [sym_writebin] = STATE(165),
    [sym_incbin] = STATE(165),
    [sym_importer] = STATE(165),
    [sym_incbinref] = STATE(165),
    [sym_bsz] = STATE(165),
    [sym_fill] = STATE(165),
    [sym_fdb] = STATE(165),
    [sym_fcb] = STATE(165),
    [sym_fcc] = STATE(165),
    [sym_zmb] = STATE(165),
    [sym_zmd] = STATE(165),
    [sym_rmb] = STATE(165),
    [sym_setdp] = STATE(165),
    [sym_org] = STATE(165),
    [sym_exec_addr] = STATE(165),
    [sym_include] = STATE(165),
    [sym__command] = STATE(165),
    [sym_struct_def] = STATE(2),
    [sym_macro_def] = STATE(2),
    [sym_doc] = STATE(2),
    [sym_long_doc] = STATE(2),
    [sym_comment] = STATE(2),
    [sym__line] = STATE(2),
    [sym__opcode_label] = STATE(165),
    [sym__command_label] = STATE(165),
    [sym_macro] = STATE(165),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(165),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(165),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(9),
    [sym_label] = STATE(9),
    [sym_local_label] = STATE(9),
    [aux_sym_source_file_repeat1] = STATE(2),
    [ts_builtin_sym_end] = ACTIONS(67),
    [aux_sym_scope_token1] = ACTIONS(69),
    [aux_sym_put_token1] = ACTIONS(72),
    [aux_sym_grabmem_token1] = ACTIONS(75),
    [aux_sym_writebin_token1] = ACTIONS(78),
    [aux_sym_incbin_token1] = ACTIONS(81),
    [aux_sym_importer_token1] = ACTIONS(84),
    [aux_sym_incbinref_token1] = ACTIONS(87),
    [aux_sym_bsz_token1] = ACTIONS(90),
    [aux_sym_fill_token1] = ACTIONS(93),
    [aux_sym_fdb_token1] = ACTIONS(96),
    [aux_sym_fcb_token1] = ACTIONS(99),
    [aux_sym_fcc_token1] = ACTIONS(102),
    [aux_sym_zmb_token1] = ACTIONS(105),
    [aux_sym_zmd_token1] = ACTIONS(108),
    [aux_sym_rmb_token1] = ACTIONS(111),
    [aux_sym_setdp_token1] = ACTIONS(114),
    [aux_sym_org_token1] = ACTIONS(117),
    [aux_sym_exec_addr_token1] = ACTIONS(120),
    [aux_sym_include_token1] = ACTIONS(123),
    [anon_sym_struct] = ACTIONS(126),
    [anon_sym_macro] = ACTIONS(129),
    [anon_sym_SEMI_SEMI_SEMI] = ACTIONS(132),
    [anon_sym_LBRACK_LBRACK] = ACTIONS(135),
    [anon_sym_SEMI] = ACTIONS(138),
    [anon_sym_SLASH_SLASH] = ACTIONS(141),
    [anon_sym_SLASH_STAR] = ACTIONS(144),
    [sym_regset_mnemonics] = ACTIONS(147),
    [sym_xfer_mnemonics] = ACTIONS(150),
    [aux_sym_mnemonic_token1] = ACTIONS(153),
    [aux_sym_local_label_token1] = ACTIONS(156),
    [sym__global_label] = ACTIONS(159),
  },
  [3] = {
    [sym_scope] = STATE(165),
    [sym_put] = STATE(165),
    [sym_grabmem] = STATE(165),
    [sym_writebin] = STATE(165),
    [sym_incbin] = STATE(165),
    [sym_importer] = STATE(165),
    [sym_incbinref] = STATE(165),
    [sym_bsz] = STATE(165),
    [sym_fill] = STATE(165),
    [sym_fdb] = STATE(165),
    [sym_fcb] = STATE(165),
    [sym_fcc] = STATE(165),
    [sym_zmb] = STATE(165),
    [sym_zmd] = STATE(165),
    [sym_rmb] = STATE(165),
    [sym_setdp] = STATE(165),
    [sym_org] = STATE(165),
    [sym_exec_addr] = STATE(165),
    [sym_include] = STATE(165),
    [sym__command] = STATE(165),
    [sym_struct_def] = STATE(2),
    [sym_macro_def] = STATE(2),
    [sym_doc] = STATE(2),
    [sym_long_doc] = STATE(2),
    [sym_comment] = STATE(2),
    [sym__line] = STATE(2),
    [sym__opcode_label] = STATE(165),
    [sym__command_label] = STATE(165),
    [sym_macro] = STATE(165),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(165),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(165),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(9),
    [sym_label] = STATE(9),
    [sym_local_label] = STATE(9),
    [aux_sym_source_file_repeat1] = STATE(2),
    [ts_builtin_sym_end] = ACTIONS(162),
    [aux_sym_scope_token1] = ACTIONS(5),
    [aux_sym_put_token1] = ACTIONS(7),
    [aux_sym_grabmem_token1] = ACTIONS(9),
    [aux_sym_writebin_token1] = ACTIONS(11),
    [aux_sym_incbin_token1] = ACTIONS(13),
    [aux_sym_importer_token1] = ACTIONS(15),
    [aux_sym_incbinref_token1] = ACTIONS(17),
    [aux_sym_bsz_token1] = ACTIONS(19),
    [aux_sym_fill_token1] = ACTIONS(21),
    [aux_sym_fdb_token1] = ACTIONS(23),
    [aux_sym_fcb_token1] = ACTIONS(25),
    [aux_sym_fcc_token1] = ACTIONS(27),
    [aux_sym_zmb_token1] = ACTIONS(29),
    [aux_sym_zmd_token1] = ACTIONS(31),
    [aux_sym_rmb_token1] = ACTIONS(33),
    [aux_sym_setdp_token1] = ACTIONS(35),
    [aux_sym_org_token1] = ACTIONS(37),
    [aux_sym_exec_addr_token1] = ACTIONS(39),
    [aux_sym_include_token1] = ACTIONS(41),
    [anon_sym_struct] = ACTIONS(43),
    [anon_sym_macro] = ACTIONS(45),
    [anon_sym_SEMI_SEMI_SEMI] = ACTIONS(47),
    [anon_sym_LBRACK_LBRACK] = ACTIONS(49),
    [anon_sym_SEMI] = ACTIONS(51),
    [anon_sym_SLASH_SLASH] = ACTIONS(53),
    [anon_sym_SLASH_STAR] = ACTIONS(55),
    [sym_regset_mnemonics] = ACTIONS(57),
    [sym_xfer_mnemonics] = ACTIONS(59),
    [aux_sym_mnemonic_token1] = ACTIONS(61),
    [aux_sym_local_label_token1] = ACTIONS(63),
    [sym__global_label] = ACTIONS(65),
  },
  [4] = {
    [sym_scope] = STATE(170),
    [sym_put] = STATE(170),
    [sym_grabmem] = STATE(170),
    [sym_writebin] = STATE(170),
    [sym_incbin] = STATE(170),
    [sym_importer] = STATE(170),
    [sym_incbinref] = STATE(170),
    [sym_bsz] = STATE(170),
    [sym_fill] = STATE(170),
    [sym_fdb] = STATE(170),
    [sym_fcb] = STATE(170),
    [sym_fcc] = STATE(170),
    [sym_zmb] = STATE(170),
    [sym_zmd] = STATE(170),
    [sym_rmb] = STATE(170),
    [sym_setdp] = STATE(170),
    [sym_org] = STATE(170),
    [sym_exec_addr] = STATE(170),
    [sym_include] = STATE(170),
    [sym__command] = STATE(170),
    [sym_struct_def] = STATE(6),
    [sym_macro_def] = STATE(6),
    [sym_comment] = STATE(6),
    [sym__line] = STATE(6),
    [sym__opcode_label] = STATE(170),
    [sym__command_label] = STATE(170),
    [sym_macro] = STATE(170),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(170),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(170),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(10),
    [sym_label] = STATE(10),
    [sym_local_label] = STATE(10),
    [aux_sym_macro_body_repeat1] = STATE(6),
    [aux_sym_scope_token1] = ACTIONS(5),
    [aux_sym_put_token1] = ACTIONS(7),
    [aux_sym_grabmem_token1] = ACTIONS(9),
    [aux_sym_writebin_token1] = ACTIONS(11),
    [aux_sym_incbin_token1] = ACTIONS(13),
    [aux_sym_importer_token1] = ACTIONS(15),
    [aux_sym_incbinref_token1] = ACTIONS(17),
    [aux_sym_bsz_token1] = ACTIONS(19),
    [aux_sym_fill_token1] = ACTIONS(21),
    [aux_sym_fdb_token1] = ACTIONS(23),
    [aux_sym_fcb_token1] = ACTIONS(25),
    [aux_sym_fcc_token1] = ACTIONS(27),
    [aux_sym_zmb_token1] = ACTIONS(29),
    [aux_sym_zmd_token1] = ACTIONS(31),
    [aux_sym_rmb_token1] = ACTIONS(33),
    [aux_sym_setdp_token1] = ACTIONS(35),
    [aux_sym_org_token1] = ACTIONS(37),
    [aux_sym_exec_addr_token1] = ACTIONS(39),
    [aux_sym_include_token1] = ACTIONS(41),
    [anon_sym_struct] = ACTIONS(164),
    [anon_sym_RBRACE] = ACTIONS(166),
    [anon_sym_macro] = ACTIONS(168),
    [anon_sym_SEMI] = ACTIONS(170),
    [anon_sym_SLASH_SLASH] = ACTIONS(170),
    [anon_sym_SLASH_STAR] = ACTIONS(172),
    [sym_regset_mnemonics] = ACTIONS(57),
    [sym_xfer_mnemonics] = ACTIONS(59),
    [aux_sym_mnemonic_token1] = ACTIONS(61),
    [aux_sym_local_label_token1] = ACTIONS(63),
    [sym__global_label] = ACTIONS(65),
  },
  [5] = {
    [sym_scope] = STATE(170),
    [sym_put] = STATE(170),
    [sym_grabmem] = STATE(170),
    [sym_writebin] = STATE(170),
    [sym_incbin] = STATE(170),
    [sym_importer] = STATE(170),
    [sym_incbinref] = STATE(170),
    [sym_bsz] = STATE(170),
    [sym_fill] = STATE(170),
    [sym_fdb] = STATE(170),
    [sym_fcb] = STATE(170),
    [sym_fcc] = STATE(170),
    [sym_zmb] = STATE(170),
    [sym_zmd] = STATE(170),
    [sym_rmb] = STATE(170),
    [sym_setdp] = STATE(170),
    [sym_org] = STATE(170),
    [sym_exec_addr] = STATE(170),
    [sym_include] = STATE(170),
    [sym__command] = STATE(170),
    [sym_struct_def] = STATE(4),
    [sym_macro_def] = STATE(4),
    [sym_comment] = STATE(4),
    [sym__line] = STATE(4),
    [sym__opcode_label] = STATE(170),
    [sym__command_label] = STATE(170),
    [sym_macro] = STATE(170),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(170),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(170),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(10),
    [sym_label] = STATE(10),
    [sym_local_label] = STATE(10),
    [aux_sym_macro_body_repeat1] = STATE(4),
    [aux_sym_scope_token1] = ACTIONS(5),
    [aux_sym_put_token1] = ACTIONS(7),
    [aux_sym_grabmem_token1] = ACTIONS(9),
    [aux_sym_writebin_token1] = ACTIONS(11),
    [aux_sym_incbin_token1] = ACTIONS(13),
    [aux_sym_importer_token1] = ACTIONS(15),
    [aux_sym_incbinref_token1] = ACTIONS(17),
    [aux_sym_bsz_token1] = ACTIONS(19),
    [aux_sym_fill_token1] = ACTIONS(21),
    [aux_sym_fdb_token1] = ACTIONS(23),
    [aux_sym_fcb_token1] = ACTIONS(25),
    [aux_sym_fcc_token1] = ACTIONS(27),
    [aux_sym_zmb_token1] = ACTIONS(29),
    [aux_sym_zmd_token1] = ACTIONS(31),
    [aux_sym_rmb_token1] = ACTIONS(33),
    [aux_sym_setdp_token1] = ACTIONS(35),
    [aux_sym_org_token1] = ACTIONS(37),
    [aux_sym_exec_addr_token1] = ACTIONS(39),
    [aux_sym_include_token1] = ACTIONS(41),
    [anon_sym_struct] = ACTIONS(164),
    [anon_sym_RBRACE] = ACTIONS(174),
    [anon_sym_macro] = ACTIONS(168),
    [anon_sym_SEMI] = ACTIONS(170),
    [anon_sym_SLASH_SLASH] = ACTIONS(170),
    [anon_sym_SLASH_STAR] = ACTIONS(172),
    [sym_regset_mnemonics] = ACTIONS(57),
    [sym_xfer_mnemonics] = ACTIONS(59),
    [aux_sym_mnemonic_token1] = ACTIONS(61),
    [aux_sym_local_label_token1] = ACTIONS(63),
    [sym__global_label] = ACTIONS(65),
  },
  [6] = {
    [sym_scope] = STATE(170),
    [sym_put] = STATE(170),
    [sym_grabmem] = STATE(170),
    [sym_writebin] = STATE(170),
    [sym_incbin] = STATE(170),
    [sym_importer] = STATE(170),
    [sym_incbinref] = STATE(170),
    [sym_bsz] = STATE(170),
    [sym_fill] = STATE(170),
    [sym_fdb] = STATE(170),
    [sym_fcb] = STATE(170),
    [sym_fcc] = STATE(170),
    [sym_zmb] = STATE(170),
    [sym_zmd] = STATE(170),
    [sym_rmb] = STATE(170),
    [sym_setdp] = STATE(170),
    [sym_org] = STATE(170),
    [sym_exec_addr] = STATE(170),
    [sym_include] = STATE(170),
    [sym__command] = STATE(170),
    [sym_struct_def] = STATE(6),
    [sym_macro_def] = STATE(6),
    [sym_comment] = STATE(6),
    [sym__line] = STATE(6),
    [sym__opcode_label] = STATE(170),
    [sym__command_label] = STATE(170),
    [sym_macro] = STATE(170),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(170),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(170),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(10),
    [sym_label] = STATE(10),
    [sym_local_label] = STATE(10),
    [aux_sym_macro_body_repeat1] = STATE(6),
    [aux_sym_scope_token1] = ACTIONS(176),
    [aux_sym_put_token1] = ACTIONS(179),
    [aux_sym_grabmem_token1] = ACTIONS(182),
    [aux_sym_writebin_token1] = ACTIONS(185),
    [aux_sym_incbin_token1] = ACTIONS(188),
    [aux_sym_importer_token1] = ACTIONS(191),
    [aux_sym_incbinref_token1] = ACTIONS(194),
    [aux_sym_bsz_token1] = ACTIONS(197),
    [aux_sym_fill_token1] = ACTIONS(200),
    [aux_sym_fdb_token1] = ACTIONS(203),
    [aux_sym_fcb_token1] = ACTIONS(206),
    [aux_sym_fcc_token1] = ACTIONS(209),
    [aux_sym_zmb_token1] = ACTIONS(212),
    [aux_sym_zmd_token1] = ACTIONS(215),
    [aux_sym_rmb_token1] = ACTIONS(218),
    [aux_sym_setdp_token1] = ACTIONS(221),
    [aux_sym_org_token1] = ACTIONS(224),
    [aux_sym_exec_addr_token1] = ACTIONS(227),
    [aux_sym_include_token1] = ACTIONS(230),
    [anon_sym_struct] = ACTIONS(233),
    [anon_sym_RBRACE] = ACTIONS(236),
    [anon_sym_macro] = ACTIONS(238),
    [anon_sym_SEMI] = ACTIONS(241),
    [anon_sym_SLASH_SLASH] = ACTIONS(241),
    [anon_sym_SLASH_STAR] = ACTIONS(244),
    [sym_regset_mnemonics] = ACTIONS(247),
    [sym_xfer_mnemonics] = ACTIONS(250),
    [aux_sym_mnemonic_token1] = ACTIONS(253),
    [aux_sym_local_label_token1] = ACTIONS(256),
    [sym__global_label] = ACTIONS(259),
  },
  [7] = {
    [sym_scope] = STATE(170),
    [sym_put] = STATE(170),
    [sym_grabmem] = STATE(170),
    [sym_writebin] = STATE(170),
    [sym_incbin] = STATE(170),
    [sym_importer] = STATE(170),
    [sym_incbinref] = STATE(170),
    [sym_bsz] = STATE(170),
    [sym_fill] = STATE(170),
    [sym_fdb] = STATE(170),
    [sym_fcb] = STATE(170),
    [sym_fcc] = STATE(170),
    [sym_zmb] = STATE(170),
    [sym_zmd] = STATE(170),
    [sym_rmb] = STATE(170),
    [sym_setdp] = STATE(170),
    [sym_org] = STATE(170),
    [sym_exec_addr] = STATE(170),
    [sym_include] = STATE(170),
    [sym__command] = STATE(170),
    [sym_struct_def] = STATE(6),
    [sym_macro_def] = STATE(6),
    [sym_comment] = STATE(6),
    [sym__line] = STATE(6),
    [sym__opcode_label] = STATE(170),
    [sym__command_label] = STATE(170),
    [sym_macro] = STATE(170),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(170),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(170),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(10),
    [sym_label] = STATE(10),
    [sym_local_label] = STATE(10),
    [aux_sym_macro_body_repeat1] = STATE(6),
    [aux_sym_scope_token1] = ACTIONS(5),
    [aux_sym_put_token1] = ACTIONS(7),
    [aux_sym_grabmem_token1] = ACTIONS(9),
    [aux_sym_writebin_token1] = ACTIONS(11),
    [aux_sym_incbin_token1] = ACTIONS(13),
    [aux_sym_importer_token1] = ACTIONS(15),
    [aux_sym_incbinref_token1] = ACTIONS(17),
    [aux_sym_bsz_token1] = ACTIONS(19),
    [aux_sym_fill_token1] = ACTIONS(21),
    [aux_sym_fdb_token1] = ACTIONS(23),
    [aux_sym_fcb_token1] = ACTIONS(25),
    [aux_sym_fcc_token1] = ACTIONS(27),
    [aux_sym_zmb_token1] = ACTIONS(29),
    [aux_sym_zmd_token1] = ACTIONS(31),
    [aux_sym_rmb_token1] = ACTIONS(33),
    [aux_sym_setdp_token1] = ACTIONS(35),
    [aux_sym_org_token1] = ACTIONS(37),
    [aux_sym_exec_addr_token1] = ACTIONS(39),
    [aux_sym_include_token1] = ACTIONS(41),
    [anon_sym_struct] = ACTIONS(164),
    [anon_sym_RBRACE] = ACTIONS(262),
    [anon_sym_macro] = ACTIONS(168),
    [anon_sym_SEMI] = ACTIONS(170),
    [anon_sym_SLASH_SLASH] = ACTIONS(170),
    [anon_sym_SLASH_STAR] = ACTIONS(172),
    [sym_regset_mnemonics] = ACTIONS(57),
    [sym_xfer_mnemonics] = ACTIONS(59),
    [aux_sym_mnemonic_token1] = ACTIONS(61),
    [aux_sym_local_label_token1] = ACTIONS(63),
    [sym__global_label] = ACTIONS(65),
  },
  [8] = {
    [sym_scope] = STATE(170),
    [sym_put] = STATE(170),
    [sym_grabmem] = STATE(170),
    [sym_writebin] = STATE(170),
    [sym_incbin] = STATE(170),
    [sym_importer] = STATE(170),
    [sym_incbinref] = STATE(170),
    [sym_bsz] = STATE(170),
    [sym_fill] = STATE(170),
    [sym_fdb] = STATE(170),
    [sym_fcb] = STATE(170),
    [sym_fcc] = STATE(170),
    [sym_zmb] = STATE(170),
    [sym_zmd] = STATE(170),
    [sym_rmb] = STATE(170),
    [sym_setdp] = STATE(170),
    [sym_org] = STATE(170),
    [sym_exec_addr] = STATE(170),
    [sym_include] = STATE(170),
    [sym__command] = STATE(170),
    [sym_struct_def] = STATE(7),
    [sym_macro_def] = STATE(7),
    [sym_comment] = STATE(7),
    [sym__line] = STATE(7),
    [sym__opcode_label] = STATE(170),
    [sym__command_label] = STATE(170),
    [sym_macro] = STATE(170),
    [sym__regsets] = STATE(211),
    [sym__xfers] = STATE(211),
    [sym_equate] = STATE(170),
    [sym_mnemonic] = STATE(11),
    [sym_opcode] = STATE(170),
    [sym__opcode_arg] = STATE(211),
    [sym__identifier] = STATE(10),
    [sym_label] = STATE(10),
    [sym_local_label] = STATE(10),
    [aux_sym_macro_body_repeat1] = STATE(7),
    [aux_sym_scope_token1] = ACTIONS(5),
    [aux_sym_put_token1] = ACTIONS(7),
    [aux_sym_grabmem_token1] = ACTIONS(9),
    [aux_sym_writebin_token1] = ACTIONS(11),
    [aux_sym_incbin_token1] = ACTIONS(13),
    [aux_sym_importer_token1] = ACTIONS(15),
    [aux_sym_incbinref_token1] = ACTIONS(17),
    [aux_sym_bsz_token1] = ACTIONS(19),
    [aux_sym_fill_token1] = ACTIONS(21),
    [aux_sym_fdb_token1] = ACTIONS(23),
    [aux_sym_fcb_token1] = ACTIONS(25),
    [aux_sym_fcc_token1] = ACTIONS(27),
    [aux_sym_zmb_token1] = ACTIONS(29),
    [aux_sym_zmd_token1] = ACTIONS(31),
    [aux_sym_rmb_token1] = ACTIONS(33),
    [aux_sym_setdp_token1] = ACTIONS(35),
    [aux_sym_org_token1] = ACTIONS(37),
    [aux_sym_exec_addr_token1] = ACTIONS(39),
    [aux_sym_include_token1] = ACTIONS(41),
    [anon_sym_struct] = ACTIONS(164),
    [anon_sym_RBRACE] = ACTIONS(264),
    [anon_sym_macro] = ACTIONS(168),
    [anon_sym_SEMI] = ACTIONS(170),
    [anon_sym_SLASH_SLASH] = ACTIONS(170),
    [anon_sym_SLASH_STAR] = ACTIONS(172),
    [sym_regset_mnemonics] = ACTIONS(57),
    [sym_xfer_mnemonics] = ACTIONS(59),
    [aux_sym_mnemonic_token1] = ACTIONS(61),
    [aux_sym_local_label_token1] = ACTIONS(63),
    [sym__global_label] = ACTIONS(65),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 33,
    ACTIONS(5), 1,
      aux_sym_scope_token1,
    ACTIONS(7), 1,
      aux_sym_put_token1,
    ACTIONS(9), 1,
      aux_sym_grabmem_token1,
    ACTIONS(11), 1,
      aux_sym_writebin_token1,
    ACTIONS(13), 1,
      aux_sym_incbin_token1,
    ACTIONS(15), 1,
      aux_sym_importer_token1,
    ACTIONS(17), 1,
      aux_sym_incbinref_token1,
    ACTIONS(19), 1,
      aux_sym_bsz_token1,
    ACTIONS(21), 1,
      aux_sym_fill_token1,
    ACTIONS(23), 1,
      aux_sym_fdb_token1,
    ACTIONS(25), 1,
      aux_sym_fcb_token1,
    ACTIONS(27), 1,
      aux_sym_fcc_token1,
    ACTIONS(29), 1,
      aux_sym_zmb_token1,
    ACTIONS(31), 1,
      aux_sym_zmd_token1,
    ACTIONS(33), 1,
      aux_sym_rmb_token1,
    ACTIONS(35), 1,
      aux_sym_setdp_token1,
    ACTIONS(37), 1,
      aux_sym_org_token1,
    ACTIONS(39), 1,
      aux_sym_exec_addr_token1,
    ACTIONS(41), 1,
      aux_sym_include_token1,
    ACTIONS(57), 1,
      sym_regset_mnemonics,
    ACTIONS(59), 1,
      sym_xfer_mnemonics,
    ACTIONS(61), 1,
      aux_sym_mnemonic_token1,
    ACTIONS(266), 1,
      anon_sym_LPAREN,
    ACTIONS(268), 1,
      anon_sym_SEMI_SEMI_SEMI,
    ACTIONS(272), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(274), 1,
      sym__line_break,
    ACTIONS(276), 1,
      aux_sym_equate_token1,
    STATE(11), 1,
      sym_mnemonic,
    STATE(220), 1,
      sym_opcode,
    ACTIONS(270), 2,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
    STATE(307), 2,
      sym_doc,
      sym_comment,
    STATE(211), 3,
      sym__regsets,
      sym__xfers,
      sym__opcode_arg,
    STATE(215), 20,
      sym_scope,
      sym_put,
      sym_grabmem,
      sym_writebin,
      sym_incbin,
      sym_importer,
      sym_incbinref,
      sym_bsz,
      sym_fill,
      sym_fdb,
      sym_fcb,
      sym_fcc,
      sym_zmb,
      sym_zmd,
      sym_rmb,
      sym_setdp,
      sym_org,
      sym_exec_addr,
      sym_include,
      sym__command,
  [123] = 33,
    ACTIONS(5), 1,
      aux_sym_scope_token1,
    ACTIONS(7), 1,
      aux_sym_put_token1,
    ACTIONS(9), 1,
      aux_sym_grabmem_token1,
    ACTIONS(11), 1,
      aux_sym_writebin_token1,
    ACTIONS(13), 1,
      aux_sym_incbin_token1,
    ACTIONS(15), 1,
      aux_sym_importer_token1,
    ACTIONS(17), 1,
      aux_sym_incbinref_token1,
    ACTIONS(19), 1,
      aux_sym_bsz_token1,
    ACTIONS(21), 1,
      aux_sym_fill_token1,
    ACTIONS(23), 1,
      aux_sym_fdb_token1,
    ACTIONS(25), 1,
      aux_sym_fcb_token1,
    ACTIONS(27), 1,
      aux_sym_fcc_token1,
    ACTIONS(29), 1,
      aux_sym_zmb_token1,
    ACTIONS(31), 1,
      aux_sym_zmd_token1,
    ACTIONS(33), 1,
      aux_sym_rmb_token1,
    ACTIONS(35), 1,
      aux_sym_setdp_token1,
    ACTIONS(37), 1,
      aux_sym_org_token1,
    ACTIONS(39), 1,
      aux_sym_exec_addr_token1,
    ACTIONS(41), 1,
      aux_sym_include_token1,
    ACTIONS(57), 1,
      sym_regset_mnemonics,
    ACTIONS(59), 1,
      sym_xfer_mnemonics,
    ACTIONS(61), 1,
      aux_sym_mnemonic_token1,
    ACTIONS(266), 1,
      anon_sym_LPAREN,
    ACTIONS(268), 1,
      anon_sym_SEMI_SEMI_SEMI,
    ACTIONS(272), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(276), 1,
      aux_sym_equate_token1,
    ACTIONS(278), 1,
      sym__line_break,
    STATE(11), 1,
      sym_mnemonic,
    STATE(220), 1,
      sym_opcode,
    ACTIONS(270), 2,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
    STATE(268), 2,
      sym_doc,
      sym_comment,
    STATE(211), 3,
      sym__regsets,
      sym__xfers,
      sym__opcode_arg,
    STATE(215), 20,
      sym_scope,
      sym_put,
      sym_grabmem,
      sym_writebin,
      sym_incbin,
      sym_importer,
      sym_incbinref,
      sym_bsz,
      sym_fill,
      sym_fdb,
      sym_fcb,
      sym_fcc,
      sym_zmb,
      sym_zmd,
      sym_rmb,
      sym_setdp,
      sym_org,
      sym_exec_addr,
      sym_include,
      sym__command,
  [246] = 19,
    ACTIONS(65), 1,
      sym__global_label,
    ACTIONS(280), 1,
      anon_sym_COMMA,
    ACTIONS(282), 1,
      anon_sym_LBRACK,
    ACTIONS(284), 1,
      anon_sym_LPAREN,
    ACTIONS(288), 1,
      anon_sym_STAR,
    ACTIONS(292), 1,
      anon_sym_GT,
    ACTIONS(294), 1,
      anon_sym_SQUOTE,
    ACTIONS(296), 1,
      sym__line_break,
    ACTIONS(298), 1,
      anon_sym_POUND,
    ACTIONS(300), 1,
      anon_sym_a,
    ACTIONS(302), 1,
      anon_sym_b,
    ACTIONS(304), 1,
      anon_sym_d,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    STATE(207), 1,
      sym_operand,
    ACTIONS(308), 3,
      sym_bin_num,
      sym_hex_num,
      sym_dec_num,
    ACTIONS(286), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
    ACTIONS(290), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(58), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
    STATE(189), 17,
      sym_immediate,
      sym_extended,
      sym_direct_page,
      sym_constant_offset,
      sym_pc_offset,
      sym_pc_offset_rel,
      sym_pre_dec,
      sym_pre_dec_dec,
      sym_post_inc,
      sym_post_inc_inc,
      sym_add_a,
      sym_add_b,
      sym_add_d,
      sym_zero_index,
      sym__indexed,
      sym__indexed_direct,
      sym_indirect,
  [337] = 2,
    ACTIONS(312), 1,
      sym__line_break,
    ACTIONS(310), 47,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      anon_sym_COMMA,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_LPAREN,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_equate_token1,
      aux_sym_mnemonic_token1,
  [390] = 2,
    ACTIONS(316), 1,
      sym__line_break,
    ACTIONS(314), 47,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      anon_sym_COMMA,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_LPAREN,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_equate_token1,
      aux_sym_mnemonic_token1,
  [443] = 14,
    ACTIONS(318), 1,
      anon_sym_COMMA,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(328), 1,
      anon_sym_a,
    ACTIONS(330), 1,
      anon_sym_b,
    ACTIONS(332), 1,
      anon_sym_d,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(336), 1,
      sym__global_label,
    ACTIONS(340), 1,
      sym_dec_num,
    ACTIONS(338), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(283), 9,
      sym_constant_offset,
      sym_pc_offset,
      sym_pc_offset_rel,
      sym_pre_dec_dec,
      sym_post_inc_inc,
      sym_add_a,
      sym_add_b,
      sym_add_d,
      sym_zero_index,
    STATE(145), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [507] = 2,
    ACTIONS(342), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(344), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [544] = 2,
    ACTIONS(346), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(348), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [581] = 2,
    ACTIONS(350), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(352), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [618] = 2,
    ACTIONS(354), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(356), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [655] = 2,
    ACTIONS(358), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(360), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [692] = 2,
    ACTIONS(362), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(364), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [729] = 2,
    ACTIONS(366), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(368), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [766] = 2,
    ACTIONS(370), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(372), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [803] = 2,
    ACTIONS(374), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(376), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [840] = 2,
    ACTIONS(378), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(380), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [877] = 2,
    ACTIONS(382), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(384), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [914] = 2,
    ACTIONS(386), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(388), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [951] = 2,
    ACTIONS(390), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(392), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [988] = 2,
    ACTIONS(394), 6,
      ts_builtin_sym_end,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_LBRACK_LBRACK,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(396), 26,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      anon_sym_SEMI,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1025] = 2,
    ACTIONS(358), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(360), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1060] = 2,
    ACTIONS(374), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(376), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1095] = 2,
    ACTIONS(350), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(352), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1130] = 2,
    ACTIONS(366), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(368), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1165] = 2,
    ACTIONS(386), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(388), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1200] = 2,
    ACTIONS(382), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(384), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1235] = 2,
    ACTIONS(378), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(380), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1270] = 2,
    ACTIONS(390), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(392), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1305] = 2,
    ACTIONS(394), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(396), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1340] = 2,
    ACTIONS(354), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(356), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1375] = 2,
    ACTIONS(346), 5,
      anon_sym_RBRACE,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      aux_sym_local_label_token1,
    ACTIONS(348), 25,
      aux_sym_scope_token1,
      aux_sym_put_token1,
      aux_sym_grabmem_token1,
      aux_sym_writebin_token1,
      aux_sym_incbin_token1,
      aux_sym_importer_token1,
      aux_sym_incbinref_token1,
      aux_sym_bsz_token1,
      aux_sym_fill_token1,
      aux_sym_fdb_token1,
      aux_sym_fcb_token1,
      aux_sym_fcc_token1,
      aux_sym_zmb_token1,
      aux_sym_zmd_token1,
      aux_sym_rmb_token1,
      aux_sym_setdp_token1,
      aux_sym_org_token1,
      aux_sym_exec_addr_token1,
      aux_sym_include_token1,
      anon_sym_struct,
      anon_sym_macro,
      sym_regset_mnemonics,
      sym_xfer_mnemonics,
      aux_sym_mnemonic_token1,
      sym__global_label,
  [1410] = 14,
    ACTIONS(398), 1,
      anon_sym_COMMA,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(422), 1,
      sym__line_break,
    STATE(162), 1,
      aux_sym_fdb_repeat1,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(400), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
  [1464] = 14,
    ACTIONS(398), 1,
      anon_sym_COMMA,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(426), 1,
      sym__line_break,
    STATE(161), 1,
      aux_sym_fdb_repeat1,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(424), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [1518] = 2,
    ACTIONS(430), 1,
      sym__line_break,
    ACTIONS(428), 23,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [1547] = 2,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(432), 23,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [1576] = 5,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(432), 16,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
  [1611] = 6,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(432), 12,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
  [1648] = 7,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(432), 10,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
  [1687] = 8,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(432), 9,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
  [1728] = 9,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(432), 8,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
  [1771] = 10,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(432), 7,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
  [1816] = 11,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(432), 6,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
  [1863] = 3,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(432), 20,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [1894] = 13,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(436), 1,
      anon_sym_COMMA,
    ACTIONS(440), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(438), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [1945] = 13,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(442), 1,
      anon_sym_COMMA,
    ACTIONS(446), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(444), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [1996] = 2,
    ACTIONS(310), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(312), 20,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_COLON,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [2025] = 4,
    ACTIONS(434), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(432), 18,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [2058] = 2,
    ACTIONS(450), 1,
      sym__line_break,
    ACTIONS(448), 23,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [2087] = 2,
    ACTIONS(314), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(316), 20,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_COLON,
      anon_sym_RBRACK,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [2116] = 13,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(452), 1,
      anon_sym_COMMA,
    ACTIONS(456), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(454), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2167] = 2,
    ACTIONS(460), 1,
      sym__line_break,
    ACTIONS(458), 23,
      anon_sym_COMMA,
      anon_sym_LBRACK,
      anon_sym_LPAREN,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_TILDE,
      anon_sym_SQUOTE,
      anon_sym_POUND,
      anon_sym_a,
      anon_sym_b,
      anon_sym_d,
      aux_sym_local_label_token1,
      sym__global_label,
      sym_bin_num,
      sym_hex_num,
      sym_dec_num,
  [2196] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(464), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(462), 5,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2245] = 2,
    ACTIONS(468), 1,
      sym__line_break,
    ACTIONS(466), 23,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [2274] = 13,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(470), 1,
      anon_sym_COMMA,
    ACTIONS(474), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(472), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2325] = 2,
    ACTIONS(478), 1,
      sym__line_break,
    ACTIONS(476), 23,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_PIPE,
      anon_sym_CARET,
      anon_sym_AMP,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [2354] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(482), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(480), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2402] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(486), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(484), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2450] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(490), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(488), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2498] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(494), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(492), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2546] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(498), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(496), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2594] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(502), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(500), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2642] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(506), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(504), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2690] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(510), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(508), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2738] = 10,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(512), 1,
      anon_sym_RPAREN,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(518), 1,
      sym_dec_num,
    ACTIONS(516), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(129), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [2782] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(522), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(520), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2830] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(526), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(524), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2878] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(530), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(528), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2926] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(534), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(532), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [2974] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(538), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(536), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [3022] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(542), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(540), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [3070] = 12,
    ACTIONS(406), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(408), 1,
      anon_sym_AMP_AMP,
    ACTIONS(410), 1,
      anon_sym_PIPE,
    ACTIONS(412), 1,
      anon_sym_CARET,
    ACTIONS(414), 1,
      anon_sym_AMP,
    ACTIONS(546), 1,
      sym__line_break,
    ACTIONS(404), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(416), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(420), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(402), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(418), 4,
      anon_sym_GT,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT,
    ACTIONS(544), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [3118] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(560), 1,
      sym_dec_num,
    ACTIONS(558), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(68), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3159] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(564), 1,
      sym_dec_num,
    ACTIONS(562), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(132), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3200] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(568), 1,
      sym_dec_num,
    ACTIONS(566), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(78), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3241] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(572), 1,
      sym_dec_num,
    ACTIONS(570), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(147), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3282] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(576), 1,
      sym_dec_num,
    ACTIONS(574), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(74), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3323] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(580), 1,
      sym_dec_num,
    ACTIONS(578), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(75), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3364] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(584), 1,
      sym_dec_num,
    ACTIONS(582), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(146), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3405] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(588), 1,
      sym_dec_num,
    ACTIONS(586), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(66), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3446] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(592), 1,
      sym_dec_num,
    ACTIONS(590), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(64), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3487] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(596), 1,
      sym_dec_num,
    ACTIONS(594), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(150), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3528] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(600), 1,
      sym_dec_num,
    ACTIONS(598), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(133), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3569] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(604), 1,
      sym_dec_num,
    ACTIONS(602), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(76), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3610] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(608), 1,
      sym_dec_num,
    ACTIONS(606), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(77), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3651] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(612), 1,
      sym_dec_num,
    ACTIONS(610), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(65), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3692] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(616), 1,
      sym_dec_num,
    ACTIONS(614), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(52), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3733] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(620), 1,
      sym_dec_num,
    ACTIONS(618), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(148), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3774] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(624), 1,
      sym_dec_num,
    ACTIONS(622), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(141), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3815] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(628), 1,
      sym_dec_num,
    ACTIONS(626), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(40), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3856] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(632), 1,
      sym_dec_num,
    ACTIONS(630), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(43), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3897] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(636), 1,
      sym_dec_num,
    ACTIONS(634), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(51), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3938] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(640), 1,
      sym_dec_num,
    ACTIONS(638), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(50), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [3979] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(644), 1,
      sym_dec_num,
    ACTIONS(642), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(49), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4020] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(648), 1,
      sym_dec_num,
    ACTIONS(646), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(48), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4061] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(652), 1,
      sym_dec_num,
    ACTIONS(650), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(47), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4102] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(656), 1,
      sym_dec_num,
    ACTIONS(654), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(46), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4143] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(660), 1,
      sym_dec_num,
    ACTIONS(658), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(45), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4184] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(664), 1,
      sym_dec_num,
    ACTIONS(662), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(44), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4225] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(668), 1,
      sym_dec_num,
    ACTIONS(666), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(55), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4266] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(672), 1,
      sym_dec_num,
    ACTIONS(670), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(67), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4307] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(676), 1,
      sym_dec_num,
    ACTIONS(674), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(41), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4348] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(680), 1,
      sym_dec_num,
    ACTIONS(678), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(69), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4389] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(684), 1,
      sym_dec_num,
    ACTIONS(682), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(53), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4430] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(688), 1,
      sym_dec_num,
    ACTIONS(686), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(62), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4471] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(692), 1,
      sym_dec_num,
    ACTIONS(690), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(140), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4512] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(696), 1,
      sym_dec_num,
    ACTIONS(694), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(139), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4553] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(700), 1,
      sym_dec_num,
    ACTIONS(698), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(138), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4594] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(704), 1,
      sym_dec_num,
    ACTIONS(702), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(137), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4635] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(708), 1,
      sym_dec_num,
    ACTIONS(706), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(70), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4676] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(712), 1,
      sym_dec_num,
    ACTIONS(710), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(71), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4717] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(716), 1,
      sym_dec_num,
    ACTIONS(714), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(60), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4758] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(720), 1,
      sym_dec_num,
    ACTIONS(718), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(149), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4799] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(724), 1,
      sym_dec_num,
    ACTIONS(722), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(61), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4840] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(728), 1,
      sym_dec_num,
    ACTIONS(726), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(136), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4881] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(732), 1,
      sym_dec_num,
    ACTIONS(730), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(73), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4922] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(736), 1,
      sym_dec_num,
    ACTIONS(734), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(131), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [4963] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(740), 1,
      sym_dec_num,
    ACTIONS(738), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(135), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [5004] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(744), 1,
      sym_dec_num,
    ACTIONS(742), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(134), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [5045] = 9,
    ACTIONS(320), 1,
      anon_sym_LPAREN,
    ACTIONS(322), 1,
      anon_sym_STAR,
    ACTIONS(326), 1,
      anon_sym_SQUOTE,
    ACTIONS(334), 1,
      aux_sym_local_label_token1,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(748), 1,
      sym_dec_num,
    ACTIONS(746), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(324), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(144), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [5086] = 9,
    ACTIONS(306), 1,
      aux_sym_local_label_token1,
    ACTIONS(548), 1,
      anon_sym_LPAREN,
    ACTIONS(550), 1,
      anon_sym_STAR,
    ACTIONS(554), 1,
      anon_sym_SQUOTE,
    ACTIONS(556), 1,
      sym__global_label,
    ACTIONS(752), 1,
      sym_dec_num,
    ACTIONS(750), 2,
      sym_bin_num,
      sym_hex_num,
    ACTIONS(552), 4,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_BANG,
      anon_sym_TILDE,
    STATE(79), 10,
      sym_pc_expr,
      sym_binary_expression,
      sym_unary_expression,
      sym_char_literal,
      sym__expression,
      sym_parenthesized_expression,
      sym__identifier,
      sym_label,
      sym_local_label,
      sym__number_literal,
  [5127] = 14,
    ACTIONS(754), 1,
      anon_sym_COMMA,
    ACTIONS(756), 1,
      anon_sym_RPAREN,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    STATE(241), 1,
      aux_sym_fdb_repeat1,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5177] = 2,
    ACTIONS(428), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(430), 17,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [5203] = 7,
    ACTIONS(432), 2,
      anon_sym_PIPE,
      anon_sym_AMP,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(434), 8,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
  [5239] = 8,
    ACTIONS(432), 2,
      anon_sym_PIPE,
      anon_sym_AMP,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(434), 6,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
  [5277] = 2,
    ACTIONS(466), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(468), 17,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [5303] = 4,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(432), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(434), 12,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [5333] = 5,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(432), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(434), 10,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
  [5365] = 9,
    ACTIONS(432), 1,
      anon_sym_PIPE,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(434), 6,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
  [5405] = 10,
    ACTIONS(432), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(434), 5,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
  [5447] = 10,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(434), 5,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
  [5489] = 11,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(434), 4,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PIPE_PIPE,
  [5533] = 3,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
    ACTIONS(432), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(434), 14,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [5561] = 2,
    ACTIONS(432), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(434), 17,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [5587] = 2,
    ACTIONS(476), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(478), 17,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [5613] = 2,
    ACTIONS(448), 4,
      anon_sym_PIPE,
      anon_sym_AMP,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(450), 17,
      anon_sym_COMMA,
      anon_sym_RBRACK,
      anon_sym_RPAREN,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PLUS,
      anon_sym_DASH,
      anon_sym_PERCENT,
      anon_sym_PIPE_PIPE,
      anon_sym_AMP_AMP,
      anon_sym_CARET,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
  [5639] = 12,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(464), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5684] = 13,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(780), 1,
      anon_sym_COMMA,
    ACTIONS(782), 1,
      anon_sym_RBRACK,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5731] = 12,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(784), 1,
      anon_sym_RBRACK,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5775] = 12,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(786), 1,
      anon_sym_RPAREN,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5819] = 12,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(788), 1,
      anon_sym_COMMA,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5863] = 12,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(790), 1,
      anon_sym_RPAREN,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5907] = 12,
    ACTIONS(762), 1,
      anon_sym_PIPE_PIPE,
    ACTIONS(764), 1,
      anon_sym_AMP_AMP,
    ACTIONS(766), 1,
      anon_sym_PIPE,
    ACTIONS(768), 1,
      anon_sym_CARET,
    ACTIONS(770), 1,
      anon_sym_AMP,
    ACTIONS(792), 1,
      anon_sym_COMMA,
    ACTIONS(760), 2,
      anon_sym_PLUS,
      anon_sym_DASH,
    ACTIONS(772), 2,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
    ACTIONS(774), 2,
      anon_sym_GT,
      anon_sym_LT,
    ACTIONS(776), 2,
      anon_sym_GT_EQ,
      anon_sym_LT_EQ,
    ACTIONS(778), 2,
      anon_sym_LT_LT,
      anon_sym_GT_GT,
    ACTIONS(758), 3,
      anon_sym_SLASH,
      anon_sym_STAR,
      anon_sym_PERCENT,
  [5951] = 4,
    STATE(171), 1,
      sym__reg,
    STATE(198), 1,
      sym_reg_set,
    ACTIONS(796), 2,
      sym_d,
      sym_pc,
    ACTIONS(794), 9,
      sym_a,
      sym_b,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
      sym_pcr,
      sym_cc,
      sym_dp,
  [5973] = 4,
    STATE(197), 1,
      sym_reg_xfer,
    STATE(311), 1,
      sym__reg,
    ACTIONS(800), 2,
      sym_d,
      sym_pc,
    ACTIONS(798), 9,
      sym_a,
      sym_b,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
      sym_pcr,
      sym_cc,
      sym_dp,
  [5995] = 3,
    STATE(227), 1,
      sym__reg,
    ACTIONS(804), 2,
      sym_d,
      sym_pc,
    ACTIONS(802), 9,
      sym_a,
      sym_b,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
      sym_pcr,
      sym_cc,
      sym_dp,
  [6014] = 3,
    STATE(181), 1,
      sym__reg,
    ACTIONS(808), 2,
      sym_d,
      sym_pc,
    ACTIONS(806), 9,
      sym_a,
      sym_b,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
      sym_pcr,
      sym_cc,
      sym_dp,
  [6033] = 6,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(810), 1,
      anon_sym_COMMA,
    ACTIONS(812), 1,
      anon_sym_RBRACE,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    STATE(245), 1,
      sym_struct_elem,
    STATE(314), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6054] = 6,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    ACTIONS(816), 1,
      anon_sym_COMMA,
    ACTIONS(818), 1,
      anon_sym_RBRACE,
    STATE(238), 1,
      sym_struct_elem,
    STATE(314), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6075] = 5,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    ACTIONS(820), 1,
      anon_sym_RBRACE,
    STATE(253), 1,
      sym_struct_elem,
    STATE(314), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6093] = 4,
    ACTIONS(822), 1,
      anon_sym_COMMA,
    ACTIONS(827), 1,
      sym__line_break,
    STATE(158), 1,
      aux_sym_fcc_repeat1,
    ACTIONS(825), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6109] = 4,
    ACTIONS(829), 1,
      anon_sym_COMMA,
    ACTIONS(833), 1,
      sym__line_break,
    STATE(224), 1,
      sym__incbinargs,
    ACTIONS(831), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6125] = 4,
    ACTIONS(835), 1,
      anon_sym_COMMA,
    ACTIONS(839), 1,
      sym__line_break,
    STATE(158), 1,
      aux_sym_fcc_repeat1,
    ACTIONS(837), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6141] = 4,
    ACTIONS(398), 1,
      anon_sym_COMMA,
    ACTIONS(843), 1,
      sym__line_break,
    STATE(166), 1,
      aux_sym_fdb_repeat1,
    ACTIONS(841), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6157] = 4,
    ACTIONS(398), 1,
      anon_sym_COMMA,
    ACTIONS(847), 1,
      sym__line_break,
    STATE(166), 1,
      aux_sym_fdb_repeat1,
    ACTIONS(845), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6173] = 4,
    ACTIONS(851), 1,
      sym__line_break,
    ACTIONS(853), 1,
      anon_sym_COLON_COLON,
    STATE(164), 1,
      aux_sym_global_scoped_id_repeat1,
    ACTIONS(849), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6189] = 4,
    ACTIONS(857), 1,
      sym__line_break,
    ACTIONS(859), 1,
      anon_sym_COLON_COLON,
    STATE(164), 1,
      aux_sym_global_scoped_id_repeat1,
    ACTIONS(855), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6205] = 5,
    ACTIONS(268), 1,
      anon_sym_SEMI_SEMI_SEMI,
    ACTIONS(272), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(274), 1,
      sym__line_break,
    ACTIONS(270), 2,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
    STATE(307), 2,
      sym_doc,
      sym_comment,
  [6223] = 4,
    ACTIONS(464), 1,
      sym__line_break,
    ACTIONS(862), 1,
      anon_sym_COMMA,
    STATE(166), 1,
      aux_sym_fdb_repeat1,
    ACTIONS(462), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6239] = 5,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    ACTIONS(865), 1,
      anon_sym_RBRACE,
    STATE(253), 1,
      sym_struct_elem,
    STATE(314), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6257] = 5,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    ACTIONS(867), 1,
      anon_sym_RBRACE,
    STATE(253), 1,
      sym_struct_elem,
    STATE(314), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6275] = 5,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    ACTIONS(869), 1,
      anon_sym_RBRACE,
    STATE(253), 1,
      sym_struct_elem,
    STATE(314), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6293] = 5,
    ACTIONS(268), 1,
      anon_sym_SEMI_SEMI_SEMI,
    ACTIONS(272), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(278), 1,
      sym__line_break,
    ACTIONS(270), 2,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
    STATE(268), 2,
      sym_doc,
      sym_comment,
  [6311] = 4,
    ACTIONS(871), 1,
      anon_sym_COMMA,
    ACTIONS(875), 1,
      sym__line_break,
    STATE(177), 1,
      aux_sym_reg_set_repeat1,
    ACTIONS(873), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6327] = 4,
    ACTIONS(877), 1,
      anon_sym_DASH,
    ACTIONS(881), 1,
      anon_sym_DASH_DASH,
    STATE(179), 1,
      sym__index_reg,
    ACTIONS(879), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6343] = 4,
    ACTIONS(885), 1,
      anon_sym_pc,
    ACTIONS(887), 1,
      anon_sym_pcr,
    STATE(277), 1,
      sym__index_reg,
    ACTIONS(883), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6359] = 4,
    ACTIONS(829), 1,
      anon_sym_COMMA,
    ACTIONS(891), 1,
      sym__line_break,
    STATE(228), 1,
      sym__incbinargs,
    ACTIONS(889), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6375] = 4,
    ACTIONS(895), 1,
      anon_sym_pc,
    ACTIONS(897), 1,
      anon_sym_pcr,
    STATE(193), 1,
      sym__index_reg,
    ACTIONS(893), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6391] = 4,
    ACTIONS(835), 1,
      anon_sym_COMMA,
    ACTIONS(901), 1,
      sym__line_break,
    STATE(160), 1,
      aux_sym_fcc_repeat1,
    ACTIONS(899), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6407] = 4,
    ACTIONS(871), 1,
      anon_sym_COMMA,
    ACTIONS(905), 1,
      sym__line_break,
    STATE(178), 1,
      aux_sym_reg_set_repeat1,
    ACTIONS(903), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6423] = 4,
    ACTIONS(907), 1,
      anon_sym_COMMA,
    ACTIONS(912), 1,
      sym__line_break,
    STATE(178), 1,
      aux_sym_reg_set_repeat1,
    ACTIONS(910), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6439] = 4,
    ACTIONS(916), 1,
      anon_sym_PLUS,
    ACTIONS(918), 1,
      sym__line_break,
    ACTIONS(920), 1,
      anon_sym_PLUS_PLUS,
    ACTIONS(914), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6455] = 2,
    ACTIONS(924), 1,
      sym__line_break,
    ACTIONS(922), 5,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6466] = 2,
    ACTIONS(912), 1,
      sym__line_break,
    ACTIONS(910), 5,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6477] = 4,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    ACTIONS(926), 1,
      anon_sym_RPAREN,
    STATE(240), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6492] = 4,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    STATE(253), 1,
      sym_struct_elem,
    STATE(314), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6507] = 2,
    ACTIONS(857), 1,
      sym__line_break,
    ACTIONS(855), 5,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
      anon_sym_COLON_COLON,
  [6518] = 3,
    ACTIONS(930), 1,
      anon_sym_DASH_DASH,
    STATE(261), 1,
      sym__index_reg,
    ACTIONS(928), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6531] = 2,
    ACTIONS(827), 1,
      sym__line_break,
    ACTIONS(825), 5,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6542] = 2,
    ACTIONS(934), 1,
      sym__line_break,
    ACTIONS(932), 5,
      anon_sym_COMMA,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6553] = 2,
    STATE(216), 1,
      sym__index_reg,
    ACTIONS(936), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6563] = 2,
    ACTIONS(940), 1,
      sym__line_break,
    ACTIONS(938), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6573] = 2,
    ACTIONS(944), 1,
      sym__line_break,
    ACTIONS(942), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6583] = 2,
    ACTIONS(948), 1,
      sym__line_break,
    ACTIONS(946), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6593] = 2,
    ACTIONS(952), 1,
      sym__line_break,
    ACTIONS(950), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6603] = 2,
    ACTIONS(956), 1,
      sym__line_break,
    ACTIONS(954), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6613] = 2,
    ACTIONS(960), 1,
      sym__line_break,
    ACTIONS(958), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6623] = 2,
    ACTIONS(964), 1,
      sym__line_break,
    ACTIONS(962), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6633] = 3,
    ACTIONS(63), 1,
      aux_sym_local_label_token1,
    ACTIONS(556), 1,
      sym__global_label,
    STATE(209), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6645] = 2,
    ACTIONS(968), 1,
      sym__line_break,
    ACTIONS(966), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6655] = 2,
    ACTIONS(972), 1,
      sym__line_break,
    ACTIONS(970), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6665] = 2,
    STATE(255), 1,
      sym_elem_type,
    ACTIONS(974), 4,
      anon_sym_byte,
      anon_sym_word,
      anon_sym_dword,
      anon_sym_qword,
  [6675] = 3,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    STATE(256), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6687] = 3,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    STATE(250), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6699] = 2,
    STATE(270), 1,
      sym__index_reg,
    ACTIONS(976), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6709] = 3,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    STATE(313), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6721] = 2,
    STATE(190), 1,
      sym__index_reg,
    ACTIONS(978), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6731] = 2,
    ACTIONS(982), 1,
      sym__line_break,
    ACTIONS(980), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6741] = 2,
    ACTIONS(986), 1,
      sym__line_break,
    ACTIONS(984), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6751] = 2,
    ACTIONS(990), 1,
      sym__line_break,
    ACTIONS(988), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6761] = 3,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    STATE(248), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6773] = 2,
    ACTIONS(994), 1,
      sym__line_break,
    ACTIONS(992), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6783] = 2,
    ACTIONS(998), 1,
      sym__line_break,
    ACTIONS(996), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6793] = 2,
    ACTIONS(296), 1,
      sym__line_break,
    ACTIONS(286), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6803] = 2,
    ACTIONS(1002), 1,
      sym__line_break,
    ACTIONS(1000), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6813] = 2,
    STATE(274), 1,
      sym__index_reg,
    ACTIONS(1004), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6823] = 2,
    STATE(273), 1,
      sym__index_reg,
    ACTIONS(1006), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6833] = 2,
    ACTIONS(1010), 1,
      sym__line_break,
    ACTIONS(1008), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6843] = 2,
    ACTIONS(1014), 1,
      sym__line_break,
    ACTIONS(1012), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6853] = 3,
    ACTIONS(514), 1,
      sym__global_label,
    ACTIONS(814), 1,
      aux_sym_local_label_token1,
    STATE(288), 3,
      sym__identifier,
      sym_label,
      sym_local_label,
  [6865] = 2,
    STATE(210), 1,
      sym__index_reg,
    ACTIONS(1016), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6875] = 2,
    ACTIONS(1020), 1,
      sym__line_break,
    ACTIONS(1018), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6885] = 2,
    ACTIONS(1024), 1,
      sym__line_break,
    ACTIONS(1022), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6895] = 2,
    ACTIONS(1028), 1,
      sym__line_break,
    ACTIONS(1026), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6905] = 2,
    STATE(292), 1,
      sym__index_reg,
    ACTIONS(1030), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6915] = 2,
    ACTIONS(1034), 1,
      sym__line_break,
    ACTIONS(1032), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6925] = 2,
    ACTIONS(1038), 1,
      sym__line_break,
    ACTIONS(1036), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6935] = 2,
    ACTIONS(1042), 1,
      sym__line_break,
    ACTIONS(1040), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6945] = 2,
    ACTIONS(1046), 1,
      sym__line_break,
    ACTIONS(1044), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6955] = 2,
    ACTIONS(1050), 1,
      sym__line_break,
    ACTIONS(1048), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6965] = 2,
    ACTIONS(1054), 1,
      sym__line_break,
    ACTIONS(1052), 4,
      anon_sym_SEMI_SEMI_SEMI,
      anon_sym_SEMI,
      anon_sym_SLASH_SLASH,
      anon_sym_SLASH_STAR,
  [6975] = 2,
    STATE(226), 1,
      sym__index_reg,
    ACTIONS(1056), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6985] = 2,
    STATE(225), 1,
      sym__index_reg,
    ACTIONS(1058), 4,
      sym_x,
      sym_y,
      sym_u,
      sym_s,
  [6995] = 4,
    ACTIONS(1060), 1,
      sym_escape_sequence,
    ACTIONS(1062), 1,
      anon_sym_DQUOTE,
    ACTIONS(1064), 1,
      aux_sym_string_literal_token1,
    STATE(233), 1,
      aux_sym_string_literal_repeat1,
  [7008] = 4,
    ACTIONS(1066), 1,
      sym_escape_sequence,
    ACTIONS(1068), 1,
      anon_sym_DQUOTE,
    ACTIONS(1070), 1,
      aux_sym_string_literal_token1,
    STATE(236), 1,
      aux_sym_string_literal_repeat1,
  [7021] = 4,
    ACTIONS(1072), 1,
      sym_escape_sequence,
    ACTIONS(1075), 1,
      anon_sym_DQUOTE,
    ACTIONS(1077), 1,
      aux_sym_string_literal_token1,
    STATE(233), 1,
      aux_sym_string_literal_repeat1,
  [7034] = 3,
    ACTIONS(1082), 1,
      anon_sym_LBRACK,
    STATE(263), 1,
      sym_array,
    ACTIONS(1080), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [7045] = 4,
    ACTIONS(1084), 1,
      sym_escape_sequence,
    ACTIONS(1086), 1,
      anon_sym_DQUOTE,
    ACTIONS(1088), 1,
      aux_sym_string_literal_token1,
    STATE(231), 1,
      aux_sym_string_literal_repeat1,
  [7058] = 4,
    ACTIONS(1060), 1,
      sym_escape_sequence,
    ACTIONS(1064), 1,
      aux_sym_string_literal_token1,
    ACTIONS(1090), 1,
      anon_sym_DQUOTE,
    STATE(233), 1,
      aux_sym_string_literal_repeat1,
  [7071] = 3,
    ACTIONS(1092), 1,
      anon_sym_COMMA,
    ACTIONS(1094), 1,
      anon_sym_RPAREN,
    STATE(243), 1,
      aux_sym_macro_args_repeat1,
  [7081] = 3,
    ACTIONS(1096), 1,
      anon_sym_COMMA,
    ACTIONS(1098), 1,
      anon_sym_RBRACE,
    STATE(247), 1,
      aux_sym_struct_def_repeat1,
  [7091] = 3,
    ACTIONS(464), 1,
      anon_sym_RPAREN,
    ACTIONS(1100), 1,
      anon_sym_COMMA,
    STATE(239), 1,
      aux_sym_fdb_repeat1,
  [7101] = 3,
    ACTIONS(1092), 1,
      anon_sym_COMMA,
    ACTIONS(1103), 1,
      anon_sym_RPAREN,
    STATE(237), 1,
      aux_sym_macro_args_repeat1,
  [7111] = 3,
    ACTIONS(754), 1,
      anon_sym_COMMA,
    ACTIONS(1105), 1,
      anon_sym_RPAREN,
    STATE(239), 1,
      aux_sym_fdb_repeat1,
  [7121] = 3,
    ACTIONS(1107), 1,
      anon_sym_COLON_COLON,
    STATE(163), 1,
      aux_sym_global_scoped_id_repeat1,
    STATE(195), 1,
      sym_global_scoped_id,
  [7131] = 3,
    ACTIONS(1109), 1,
      anon_sym_COMMA,
    ACTIONS(1112), 1,
      anon_sym_RPAREN,
    STATE(243), 1,
      aux_sym_macro_args_repeat1,
  [7141] = 3,
    ACTIONS(1114), 1,
      anon_sym_COMMA,
    ACTIONS(1117), 1,
      anon_sym_RBRACE,
    STATE(244), 1,
      aux_sym_struct_def_repeat1,
  [7151] = 3,
    ACTIONS(1119), 1,
      anon_sym_COMMA,
    ACTIONS(1121), 1,
      anon_sym_RBRACE,
    STATE(246), 1,
      aux_sym_struct_def_repeat1,
  [7161] = 3,
    ACTIONS(820), 1,
      anon_sym_RBRACE,
    ACTIONS(1123), 1,
      anon_sym_COMMA,
    STATE(244), 1,
      aux_sym_struct_def_repeat1,
  [7171] = 3,
    ACTIONS(865), 1,
      anon_sym_RBRACE,
    ACTIONS(1125), 1,
      anon_sym_COMMA,
    STATE(244), 1,
      aux_sym_struct_def_repeat1,
  [7181] = 2,
    ACTIONS(1127), 1,
      anon_sym_LPAREN,
    STATE(264), 1,
      sym_macro_args,
  [7188] = 2,
    ACTIONS(1129), 1,
      anon_sym_LBRACE,
    STATE(33), 1,
      sym_macro_body,
  [7195] = 2,
    ACTIONS(1127), 1,
      anon_sym_LPAREN,
    STATE(249), 1,
      sym_macro_args,
  [7202] = 1,
    ACTIONS(1131), 2,
      sym_escape_sequence,
      aux_sym_char_literal_token1,
  [7207] = 2,
    ACTIONS(1133), 1,
      aux_sym_doc_text_token1,
    STATE(15), 1,
      sym_doc_text,
  [7214] = 1,
    ACTIONS(1117), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [7219] = 1,
    ACTIONS(1135), 2,
      sym_escape_sequence,
      aux_sym_char_literal_token1,
  [7224] = 1,
    ACTIONS(1137), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [7229] = 1,
    ACTIONS(1112), 2,
      anon_sym_COMMA,
      anon_sym_RPAREN,
  [7234] = 2,
    ACTIONS(1139), 1,
      anon_sym_DQUOTE,
    STATE(298), 1,
      sym_string_literal,
  [7241] = 2,
    ACTIONS(1141), 1,
      anon_sym_DQUOTE,
    STATE(212), 1,
      sym_string_literal,
  [7248] = 2,
    ACTIONS(1143), 1,
      aux_sym_doc_text_token1,
    STATE(293), 1,
      sym_doc_text,
  [7255] = 2,
    ACTIONS(1141), 1,
      anon_sym_DQUOTE,
    STATE(159), 1,
      sym_string_literal,
  [7262] = 2,
    ACTIONS(918), 1,
      anon_sym_RBRACK,
    ACTIONS(1145), 1,
      anon_sym_PLUS_PLUS,
  [7269] = 2,
    ACTIONS(1141), 1,
      anon_sym_DQUOTE,
    STATE(174), 1,
      sym_string_literal,
  [7276] = 1,
    ACTIONS(1147), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [7281] = 2,
    ACTIONS(1149), 1,
      anon_sym_LBRACE,
    STATE(26), 1,
      sym_macro_body,
  [7288] = 2,
    ACTIONS(1141), 1,
      anon_sym_DQUOTE,
    STATE(186), 1,
      sym_string_literal,
  [7295] = 1,
    ACTIONS(1151), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [7300] = 2,
    ACTIONS(1141), 1,
      anon_sym_DQUOTE,
    STATE(176), 1,
      sym_string_literal,
  [7307] = 1,
    ACTIONS(1153), 1,
      sym__line_break,
  [7311] = 1,
    ACTIONS(1155), 1,
      aux_sym_comment_token1,
  [7315] = 1,
    ACTIONS(1042), 1,
      anon_sym_RBRACK,
  [7319] = 1,
    ACTIONS(1028), 1,
      anon_sym_RBRACK,
  [7323] = 1,
    ACTIONS(362), 1,
      sym__line_break,
  [7327] = 1,
    ACTIONS(1014), 1,
      anon_sym_RBRACK,
  [7331] = 1,
    ACTIONS(944), 1,
      anon_sym_RBRACK,
  [7335] = 1,
    ACTIONS(948), 1,
      anon_sym_RBRACK,
  [7339] = 1,
    ACTIONS(952), 1,
      anon_sym_RBRACK,
  [7343] = 1,
    ACTIONS(956), 1,
      anon_sym_RBRACK,
  [7347] = 1,
    ACTIONS(346), 1,
      sym__line_break,
  [7351] = 1,
    ACTIONS(924), 1,
      anon_sym_COMMA,
  [7355] = 1,
    ACTIONS(1157), 1,
      anon_sym_SQUOTE,
  [7359] = 1,
    ACTIONS(1159), 1,
      sym__global_label,
  [7363] = 1,
    ACTIONS(1161), 1,
      anon_sym_COMMA,
  [7367] = 1,
    ACTIONS(782), 1,
      anon_sym_RBRACK,
  [7371] = 1,
    ACTIONS(1163), 1,
      anon_sym_SLASH,
  [7375] = 1,
    ACTIONS(1165), 1,
      anon_sym_RBRACK_RBRACK,
  [7379] = 1,
    ACTIONS(1167), 1,
      aux_sym_doc_text_token1,
  [7383] = 1,
    ACTIONS(1169), 1,
      sym__global_label,
  [7387] = 1,
    ACTIONS(1171), 1,
      anon_sym_LBRACE,
  [7391] = 1,
    ACTIONS(358), 1,
      sym__line_break,
  [7395] = 1,
    ACTIONS(1173), 1,
      anon_sym_LBRACE,
  [7399] = 1,
    ACTIONS(1175), 1,
      anon_sym_SLASH,
  [7403] = 1,
    ACTIONS(998), 1,
      anon_sym_RBRACK,
  [7407] = 1,
    ACTIONS(342), 1,
      sym__line_break,
  [7411] = 1,
    ACTIONS(1177), 1,
      anon_sym_LBRACE,
  [7415] = 1,
    ACTIONS(1179), 1,
      anon_sym_COMMA,
  [7419] = 1,
    ACTIONS(1181), 1,
      sym__global_label,
  [7423] = 1,
    ACTIONS(1183), 1,
      anon_sym_SQUOTE,
  [7427] = 1,
    ACTIONS(1185), 1,
      anon_sym_COMMA,
  [7431] = 1,
    ACTIONS(934), 1,
      anon_sym_COMMA,
  [7435] = 1,
    ACTIONS(1187), 1,
      anon_sym_LBRACE,
  [7439] = 1,
    ACTIONS(1121), 1,
      anon_sym_RBRACE,
  [7443] = 1,
    ACTIONS(1098), 1,
      anon_sym_RBRACE,
  [7447] = 1,
    ACTIONS(1189), 1,
      anon_sym_COMMA,
  [7451] = 1,
    ACTIONS(1191), 1,
      anon_sym_COMMA,
  [7455] = 1,
    ACTIONS(1193), 1,
      sym_long_doc_text,
  [7459] = 1,
    ACTIONS(1195), 1,
      anon_sym_COMMA,
  [7463] = 1,
    ACTIONS(1197), 1,
      sym__line_break,
  [7467] = 1,
    ACTIONS(1199), 1,
      aux_sym_doc_text_token1,
  [7471] = 1,
    ACTIONS(1201), 1,
      anon_sym_SLASH,
  [7475] = 1,
    ACTIONS(1203), 1,
      anon_sym_COMMA,
  [7479] = 1,
    ACTIONS(1205), 1,
      anon_sym_COMMA,
  [7483] = 1,
    ACTIONS(1207), 1,
      ts_builtin_sym_end,
  [7487] = 1,
    ACTIONS(1209), 1,
      anon_sym_LBRACE,
  [7491] = 1,
    ACTIONS(1211), 1,
      anon_sym_COLON,
  [7495] = 1,
    ACTIONS(1213), 1,
      aux_sym_comment_token1,
  [7499] = 1,
    ACTIONS(1215), 1,
      aux_sym_doc_text_token1,
  [7503] = 1,
    ACTIONS(1217), 1,
      aux_sym_comment_token1,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(9)] = 0,
  [SMALL_STATE(10)] = 123,
  [SMALL_STATE(11)] = 246,
  [SMALL_STATE(12)] = 337,
  [SMALL_STATE(13)] = 390,
  [SMALL_STATE(14)] = 443,
  [SMALL_STATE(15)] = 507,
  [SMALL_STATE(16)] = 544,
  [SMALL_STATE(17)] = 581,
  [SMALL_STATE(18)] = 618,
  [SMALL_STATE(19)] = 655,
  [SMALL_STATE(20)] = 692,
  [SMALL_STATE(21)] = 729,
  [SMALL_STATE(22)] = 766,
  [SMALL_STATE(23)] = 803,
  [SMALL_STATE(24)] = 840,
  [SMALL_STATE(25)] = 877,
  [SMALL_STATE(26)] = 914,
  [SMALL_STATE(27)] = 951,
  [SMALL_STATE(28)] = 988,
  [SMALL_STATE(29)] = 1025,
  [SMALL_STATE(30)] = 1060,
  [SMALL_STATE(31)] = 1095,
  [SMALL_STATE(32)] = 1130,
  [SMALL_STATE(33)] = 1165,
  [SMALL_STATE(34)] = 1200,
  [SMALL_STATE(35)] = 1235,
  [SMALL_STATE(36)] = 1270,
  [SMALL_STATE(37)] = 1305,
  [SMALL_STATE(38)] = 1340,
  [SMALL_STATE(39)] = 1375,
  [SMALL_STATE(40)] = 1410,
  [SMALL_STATE(41)] = 1464,
  [SMALL_STATE(42)] = 1518,
  [SMALL_STATE(43)] = 1547,
  [SMALL_STATE(44)] = 1576,
  [SMALL_STATE(45)] = 1611,
  [SMALL_STATE(46)] = 1648,
  [SMALL_STATE(47)] = 1687,
  [SMALL_STATE(48)] = 1728,
  [SMALL_STATE(49)] = 1771,
  [SMALL_STATE(50)] = 1816,
  [SMALL_STATE(51)] = 1863,
  [SMALL_STATE(52)] = 1894,
  [SMALL_STATE(53)] = 1945,
  [SMALL_STATE(54)] = 1996,
  [SMALL_STATE(55)] = 2025,
  [SMALL_STATE(56)] = 2058,
  [SMALL_STATE(57)] = 2087,
  [SMALL_STATE(58)] = 2116,
  [SMALL_STATE(59)] = 2167,
  [SMALL_STATE(60)] = 2196,
  [SMALL_STATE(61)] = 2245,
  [SMALL_STATE(62)] = 2274,
  [SMALL_STATE(63)] = 2325,
  [SMALL_STATE(64)] = 2354,
  [SMALL_STATE(65)] = 2402,
  [SMALL_STATE(66)] = 2450,
  [SMALL_STATE(67)] = 2498,
  [SMALL_STATE(68)] = 2546,
  [SMALL_STATE(69)] = 2594,
  [SMALL_STATE(70)] = 2642,
  [SMALL_STATE(71)] = 2690,
  [SMALL_STATE(72)] = 2738,
  [SMALL_STATE(73)] = 2782,
  [SMALL_STATE(74)] = 2830,
  [SMALL_STATE(75)] = 2878,
  [SMALL_STATE(76)] = 2926,
  [SMALL_STATE(77)] = 2974,
  [SMALL_STATE(78)] = 3022,
  [SMALL_STATE(79)] = 3070,
  [SMALL_STATE(80)] = 3118,
  [SMALL_STATE(81)] = 3159,
  [SMALL_STATE(82)] = 3200,
  [SMALL_STATE(83)] = 3241,
  [SMALL_STATE(84)] = 3282,
  [SMALL_STATE(85)] = 3323,
  [SMALL_STATE(86)] = 3364,
  [SMALL_STATE(87)] = 3405,
  [SMALL_STATE(88)] = 3446,
  [SMALL_STATE(89)] = 3487,
  [SMALL_STATE(90)] = 3528,
  [SMALL_STATE(91)] = 3569,
  [SMALL_STATE(92)] = 3610,
  [SMALL_STATE(93)] = 3651,
  [SMALL_STATE(94)] = 3692,
  [SMALL_STATE(95)] = 3733,
  [SMALL_STATE(96)] = 3774,
  [SMALL_STATE(97)] = 3815,
  [SMALL_STATE(98)] = 3856,
  [SMALL_STATE(99)] = 3897,
  [SMALL_STATE(100)] = 3938,
  [SMALL_STATE(101)] = 3979,
  [SMALL_STATE(102)] = 4020,
  [SMALL_STATE(103)] = 4061,
  [SMALL_STATE(104)] = 4102,
  [SMALL_STATE(105)] = 4143,
  [SMALL_STATE(106)] = 4184,
  [SMALL_STATE(107)] = 4225,
  [SMALL_STATE(108)] = 4266,
  [SMALL_STATE(109)] = 4307,
  [SMALL_STATE(110)] = 4348,
  [SMALL_STATE(111)] = 4389,
  [SMALL_STATE(112)] = 4430,
  [SMALL_STATE(113)] = 4471,
  [SMALL_STATE(114)] = 4512,
  [SMALL_STATE(115)] = 4553,
  [SMALL_STATE(116)] = 4594,
  [SMALL_STATE(117)] = 4635,
  [SMALL_STATE(118)] = 4676,
  [SMALL_STATE(119)] = 4717,
  [SMALL_STATE(120)] = 4758,
  [SMALL_STATE(121)] = 4799,
  [SMALL_STATE(122)] = 4840,
  [SMALL_STATE(123)] = 4881,
  [SMALL_STATE(124)] = 4922,
  [SMALL_STATE(125)] = 4963,
  [SMALL_STATE(126)] = 5004,
  [SMALL_STATE(127)] = 5045,
  [SMALL_STATE(128)] = 5086,
  [SMALL_STATE(129)] = 5127,
  [SMALL_STATE(130)] = 5177,
  [SMALL_STATE(131)] = 5203,
  [SMALL_STATE(132)] = 5239,
  [SMALL_STATE(133)] = 5277,
  [SMALL_STATE(134)] = 5303,
  [SMALL_STATE(135)] = 5333,
  [SMALL_STATE(136)] = 5365,
  [SMALL_STATE(137)] = 5405,
  [SMALL_STATE(138)] = 5447,
  [SMALL_STATE(139)] = 5489,
  [SMALL_STATE(140)] = 5533,
  [SMALL_STATE(141)] = 5561,
  [SMALL_STATE(142)] = 5587,
  [SMALL_STATE(143)] = 5613,
  [SMALL_STATE(144)] = 5639,
  [SMALL_STATE(145)] = 5684,
  [SMALL_STATE(146)] = 5731,
  [SMALL_STATE(147)] = 5775,
  [SMALL_STATE(148)] = 5819,
  [SMALL_STATE(149)] = 5863,
  [SMALL_STATE(150)] = 5907,
  [SMALL_STATE(151)] = 5951,
  [SMALL_STATE(152)] = 5973,
  [SMALL_STATE(153)] = 5995,
  [SMALL_STATE(154)] = 6014,
  [SMALL_STATE(155)] = 6033,
  [SMALL_STATE(156)] = 6054,
  [SMALL_STATE(157)] = 6075,
  [SMALL_STATE(158)] = 6093,
  [SMALL_STATE(159)] = 6109,
  [SMALL_STATE(160)] = 6125,
  [SMALL_STATE(161)] = 6141,
  [SMALL_STATE(162)] = 6157,
  [SMALL_STATE(163)] = 6173,
  [SMALL_STATE(164)] = 6189,
  [SMALL_STATE(165)] = 6205,
  [SMALL_STATE(166)] = 6223,
  [SMALL_STATE(167)] = 6239,
  [SMALL_STATE(168)] = 6257,
  [SMALL_STATE(169)] = 6275,
  [SMALL_STATE(170)] = 6293,
  [SMALL_STATE(171)] = 6311,
  [SMALL_STATE(172)] = 6327,
  [SMALL_STATE(173)] = 6343,
  [SMALL_STATE(174)] = 6359,
  [SMALL_STATE(175)] = 6375,
  [SMALL_STATE(176)] = 6391,
  [SMALL_STATE(177)] = 6407,
  [SMALL_STATE(178)] = 6423,
  [SMALL_STATE(179)] = 6439,
  [SMALL_STATE(180)] = 6455,
  [SMALL_STATE(181)] = 6466,
  [SMALL_STATE(182)] = 6477,
  [SMALL_STATE(183)] = 6492,
  [SMALL_STATE(184)] = 6507,
  [SMALL_STATE(185)] = 6518,
  [SMALL_STATE(186)] = 6531,
  [SMALL_STATE(187)] = 6542,
  [SMALL_STATE(188)] = 6553,
  [SMALL_STATE(189)] = 6563,
  [SMALL_STATE(190)] = 6573,
  [SMALL_STATE(191)] = 6583,
  [SMALL_STATE(192)] = 6593,
  [SMALL_STATE(193)] = 6603,
  [SMALL_STATE(194)] = 6613,
  [SMALL_STATE(195)] = 6623,
  [SMALL_STATE(196)] = 6633,
  [SMALL_STATE(197)] = 6645,
  [SMALL_STATE(198)] = 6655,
  [SMALL_STATE(199)] = 6665,
  [SMALL_STATE(200)] = 6675,
  [SMALL_STATE(201)] = 6687,
  [SMALL_STATE(202)] = 6699,
  [SMALL_STATE(203)] = 6709,
  [SMALL_STATE(204)] = 6721,
  [SMALL_STATE(205)] = 6731,
  [SMALL_STATE(206)] = 6741,
  [SMALL_STATE(207)] = 6751,
  [SMALL_STATE(208)] = 6761,
  [SMALL_STATE(209)] = 6773,
  [SMALL_STATE(210)] = 6783,
  [SMALL_STATE(211)] = 6793,
  [SMALL_STATE(212)] = 6803,
  [SMALL_STATE(213)] = 6813,
  [SMALL_STATE(214)] = 6823,
  [SMALL_STATE(215)] = 6833,
  [SMALL_STATE(216)] = 6843,
  [SMALL_STATE(217)] = 6853,
  [SMALL_STATE(218)] = 6865,
  [SMALL_STATE(219)] = 6875,
  [SMALL_STATE(220)] = 6885,
  [SMALL_STATE(221)] = 6895,
  [SMALL_STATE(222)] = 6905,
  [SMALL_STATE(223)] = 6915,
  [SMALL_STATE(224)] = 6925,
  [SMALL_STATE(225)] = 6935,
  [SMALL_STATE(226)] = 6945,
  [SMALL_STATE(227)] = 6955,
  [SMALL_STATE(228)] = 6965,
  [SMALL_STATE(229)] = 6975,
  [SMALL_STATE(230)] = 6985,
  [SMALL_STATE(231)] = 6995,
  [SMALL_STATE(232)] = 7008,
  [SMALL_STATE(233)] = 7021,
  [SMALL_STATE(234)] = 7034,
  [SMALL_STATE(235)] = 7045,
  [SMALL_STATE(236)] = 7058,
  [SMALL_STATE(237)] = 7071,
  [SMALL_STATE(238)] = 7081,
  [SMALL_STATE(239)] = 7091,
  [SMALL_STATE(240)] = 7101,
  [SMALL_STATE(241)] = 7111,
  [SMALL_STATE(242)] = 7121,
  [SMALL_STATE(243)] = 7131,
  [SMALL_STATE(244)] = 7141,
  [SMALL_STATE(245)] = 7151,
  [SMALL_STATE(246)] = 7161,
  [SMALL_STATE(247)] = 7171,
  [SMALL_STATE(248)] = 7181,
  [SMALL_STATE(249)] = 7188,
  [SMALL_STATE(250)] = 7195,
  [SMALL_STATE(251)] = 7202,
  [SMALL_STATE(252)] = 7207,
  [SMALL_STATE(253)] = 7214,
  [SMALL_STATE(254)] = 7219,
  [SMALL_STATE(255)] = 7224,
  [SMALL_STATE(256)] = 7229,
  [SMALL_STATE(257)] = 7234,
  [SMALL_STATE(258)] = 7241,
  [SMALL_STATE(259)] = 7248,
  [SMALL_STATE(260)] = 7255,
  [SMALL_STATE(261)] = 7262,
  [SMALL_STATE(262)] = 7269,
  [SMALL_STATE(263)] = 7276,
  [SMALL_STATE(264)] = 7281,
  [SMALL_STATE(265)] = 7288,
  [SMALL_STATE(266)] = 7295,
  [SMALL_STATE(267)] = 7300,
  [SMALL_STATE(268)] = 7307,
  [SMALL_STATE(269)] = 7311,
  [SMALL_STATE(270)] = 7315,
  [SMALL_STATE(271)] = 7319,
  [SMALL_STATE(272)] = 7323,
  [SMALL_STATE(273)] = 7327,
  [SMALL_STATE(274)] = 7331,
  [SMALL_STATE(275)] = 7335,
  [SMALL_STATE(276)] = 7339,
  [SMALL_STATE(277)] = 7343,
  [SMALL_STATE(278)] = 7347,
  [SMALL_STATE(279)] = 7351,
  [SMALL_STATE(280)] = 7355,
  [SMALL_STATE(281)] = 7359,
  [SMALL_STATE(282)] = 7363,
  [SMALL_STATE(283)] = 7367,
  [SMALL_STATE(284)] = 7371,
  [SMALL_STATE(285)] = 7375,
  [SMALL_STATE(286)] = 7379,
  [SMALL_STATE(287)] = 7383,
  [SMALL_STATE(288)] = 7387,
  [SMALL_STATE(289)] = 7391,
  [SMALL_STATE(290)] = 7395,
  [SMALL_STATE(291)] = 7399,
  [SMALL_STATE(292)] = 7403,
  [SMALL_STATE(293)] = 7407,
  [SMALL_STATE(294)] = 7411,
  [SMALL_STATE(295)] = 7415,
  [SMALL_STATE(296)] = 7419,
  [SMALL_STATE(297)] = 7423,
  [SMALL_STATE(298)] = 7427,
  [SMALL_STATE(299)] = 7431,
  [SMALL_STATE(300)] = 7435,
  [SMALL_STATE(301)] = 7439,
  [SMALL_STATE(302)] = 7443,
  [SMALL_STATE(303)] = 7447,
  [SMALL_STATE(304)] = 7451,
  [SMALL_STATE(305)] = 7455,
  [SMALL_STATE(306)] = 7459,
  [SMALL_STATE(307)] = 7463,
  [SMALL_STATE(308)] = 7467,
  [SMALL_STATE(309)] = 7471,
  [SMALL_STATE(310)] = 7475,
  [SMALL_STATE(311)] = 7479,
  [SMALL_STATE(312)] = 7483,
  [SMALL_STATE(313)] = 7487,
  [SMALL_STATE(314)] = 7491,
  [SMALL_STATE(315)] = 7495,
  [SMALL_STATE(316)] = 7499,
  [SMALL_STATE(317)] = 7503,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0),
  [5] = {.entry = {.count = 1, .reusable = false}}, SHIFT(196),
  [7] = {.entry = {.count = 1, .reusable = false}}, SHIFT(85),
  [9] = {.entry = {.count = 1, .reusable = false}}, SHIFT(89),
  [11] = {.entry = {.count = 1, .reusable = false}}, SHIFT(257),
  [13] = {.entry = {.count = 1, .reusable = false}}, SHIFT(260),
  [15] = {.entry = {.count = 1, .reusable = false}}, SHIFT(242),
  [17] = {.entry = {.count = 1, .reusable = false}}, SHIFT(262),
  [19] = {.entry = {.count = 1, .reusable = false}}, SHIFT(94),
  [21] = {.entry = {.count = 1, .reusable = false}}, SHIFT(95),
  [23] = {.entry = {.count = 1, .reusable = false}}, SHIFT(97),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(109),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(267),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(110),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(128),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(92),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(91),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(88),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(84),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(258),
  [43] = {.entry = {.count = 1, .reusable = false}}, SHIFT(217),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(208),
  [47] = {.entry = {.count = 1, .reusable = true}}, SHIFT(252),
  [49] = {.entry = {.count = 1, .reusable = true}}, SHIFT(305),
  [51] = {.entry = {.count = 1, .reusable = false}}, SHIFT(316),
  [53] = {.entry = {.count = 1, .reusable = true}}, SHIFT(316),
  [55] = {.entry = {.count = 1, .reusable = true}}, SHIFT(317),
  [57] = {.entry = {.count = 1, .reusable = false}}, SHIFT(151),
  [59] = {.entry = {.count = 1, .reusable = false}}, SHIFT(152),
  [61] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [63] = {.entry = {.count = 1, .reusable = true}}, SHIFT(281),
  [65] = {.entry = {.count = 1, .reusable = false}}, SHIFT(13),
  [67] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2),
  [69] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(196),
  [72] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(85),
  [75] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(89),
  [78] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(257),
  [81] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(260),
  [84] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(242),
  [87] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(262),
  [90] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(94),
  [93] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(95),
  [96] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(97),
  [99] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(109),
  [102] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(267),
  [105] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(110),
  [108] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(128),
  [111] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(92),
  [114] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(91),
  [117] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(88),
  [120] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(84),
  [123] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(258),
  [126] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(217),
  [129] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(208),
  [132] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(252),
  [135] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(305),
  [138] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(316),
  [141] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(316),
  [144] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(317),
  [147] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(151),
  [150] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(152),
  [153] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(59),
  [156] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(281),
  [159] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_source_file_repeat1, 2), SHIFT_REPEAT(13),
  [162] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1),
  [164] = {.entry = {.count = 1, .reusable = false}}, SHIFT(203),
  [166] = {.entry = {.count = 1, .reusable = true}}, SHIFT(37),
  [168] = {.entry = {.count = 1, .reusable = false}}, SHIFT(201),
  [170] = {.entry = {.count = 1, .reusable = true}}, SHIFT(308),
  [172] = {.entry = {.count = 1, .reusable = true}}, SHIFT(315),
  [174] = {.entry = {.count = 1, .reusable = true}}, SHIFT(35),
  [176] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(196),
  [179] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(85),
  [182] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(89),
  [185] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(257),
  [188] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(260),
  [191] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(242),
  [194] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(262),
  [197] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(94),
  [200] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(95),
  [203] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(97),
  [206] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(109),
  [209] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(267),
  [212] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(110),
  [215] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(128),
  [218] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(92),
  [221] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(91),
  [224] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(88),
  [227] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(84),
  [230] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(258),
  [233] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(203),
  [236] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_macro_body_repeat1, 2),
  [238] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(201),
  [241] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(308),
  [244] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(315),
  [247] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(151),
  [250] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(152),
  [253] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(59),
  [256] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(281),
  [259] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_macro_body_repeat1, 2), SHIFT_REPEAT(13),
  [262] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [264] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [266] = {.entry = {.count = 1, .reusable = false}}, SHIFT(72),
  [268] = {.entry = {.count = 1, .reusable = false}}, SHIFT(259),
  [270] = {.entry = {.count = 1, .reusable = false}}, SHIFT(286),
  [272] = {.entry = {.count = 1, .reusable = false}}, SHIFT(269),
  [274] = {.entry = {.count = 1, .reusable = true}}, SHIFT(23),
  [276] = {.entry = {.count = 1, .reusable = false}}, SHIFT(93),
  [278] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [280] = {.entry = {.count = 1, .reusable = false}}, SHIFT(172),
  [282] = {.entry = {.count = 1, .reusable = false}}, SHIFT(14),
  [284] = {.entry = {.count = 1, .reusable = false}}, SHIFT(120),
  [286] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_opcode, 1),
  [288] = {.entry = {.count = 1, .reusable = false}}, SHIFT(42),
  [290] = {.entry = {.count = 1, .reusable = false}}, SHIFT(121),
  [292] = {.entry = {.count = 1, .reusable = false}}, SHIFT(80),
  [294] = {.entry = {.count = 1, .reusable = false}}, SHIFT(254),
  [296] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_opcode, 1),
  [298] = {.entry = {.count = 1, .reusable = false}}, SHIFT(87),
  [300] = {.entry = {.count = 1, .reusable = false}}, SHIFT(306),
  [302] = {.entry = {.count = 1, .reusable = false}}, SHIFT(304),
  [304] = {.entry = {.count = 1, .reusable = false}}, SHIFT(303),
  [306] = {.entry = {.count = 1, .reusable = false}}, SHIFT(281),
  [308] = {.entry = {.count = 1, .reusable = false}}, SHIFT(58),
  [310] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_local_label, 2),
  [312] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_local_label, 2),
  [314] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_label, 1),
  [316] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_label, 1),
  [318] = {.entry = {.count = 1, .reusable = true}}, SHIFT(185),
  [320] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [322] = {.entry = {.count = 1, .reusable = true}}, SHIFT(130),
  [324] = {.entry = {.count = 1, .reusable = true}}, SHIFT(90),
  [326] = {.entry = {.count = 1, .reusable = true}}, SHIFT(251),
  [328] = {.entry = {.count = 1, .reusable = false}}, SHIFT(310),
  [330] = {.entry = {.count = 1, .reusable = false}}, SHIFT(282),
  [332] = {.entry = {.count = 1, .reusable = false}}, SHIFT(295),
  [334] = {.entry = {.count = 1, .reusable = false}}, SHIFT(287),
  [336] = {.entry = {.count = 1, .reusable = false}}, SHIFT(57),
  [338] = {.entry = {.count = 1, .reusable = true}}, SHIFT(145),
  [340] = {.entry = {.count = 1, .reusable = false}}, SHIFT(145),
  [342] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_doc, 2),
  [344] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_doc, 2),
  [346] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_comment, 2),
  [348] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_comment, 2),
  [350] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__line, 3),
  [352] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__line, 3),
  [354] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_def, 7),
  [356] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_struct_def, 7),
  [358] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_comment, 3),
  [360] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_comment, 3),
  [362] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_doc_text, 1),
  [364] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_doc_text, 1),
  [366] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_def, 4),
  [368] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_struct_def, 4),
  [370] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_long_doc, 3),
  [372] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_long_doc, 3),
  [374] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__line, 2),
  [376] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__line, 2),
  [378] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro_body, 2),
  [380] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_macro_body, 2),
  [382] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_def, 5),
  [384] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_struct_def, 5),
  [386] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro_def, 4),
  [388] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_macro_def, 4),
  [390] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_def, 6),
  [392] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_struct_def, 6),
  [394] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro_body, 3),
  [396] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_macro_body, 3),
  [398] = {.entry = {.count = 1, .reusable = false}}, SHIFT(119),
  [400] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fdb, 2),
  [402] = {.entry = {.count = 1, .reusable = false}}, SHIFT(98),
  [404] = {.entry = {.count = 1, .reusable = false}}, SHIFT(99),
  [406] = {.entry = {.count = 1, .reusable = false}}, SHIFT(100),
  [408] = {.entry = {.count = 1, .reusable = false}}, SHIFT(101),
  [410] = {.entry = {.count = 1, .reusable = false}}, SHIFT(102),
  [412] = {.entry = {.count = 1, .reusable = false}}, SHIFT(103),
  [414] = {.entry = {.count = 1, .reusable = false}}, SHIFT(104),
  [416] = {.entry = {.count = 1, .reusable = false}}, SHIFT(105),
  [418] = {.entry = {.count = 1, .reusable = false}}, SHIFT(106),
  [420] = {.entry = {.count = 1, .reusable = false}}, SHIFT(107),
  [422] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fdb, 2),
  [424] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fcb, 2),
  [426] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fcb, 2),
  [428] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_pc_expr, 1),
  [430] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_pc_expr, 1),
  [432] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_binary_expression, 3, .production_id = 5),
  [434] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_binary_expression, 3, .production_id = 5),
  [436] = {.entry = {.count = 1, .reusable = false}}, SHIFT(117),
  [438] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_bsz, 2, .production_id = 2),
  [440] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_bsz, 2, .production_id = 2),
  [442] = {.entry = {.count = 1, .reusable = false}}, SHIFT(123),
  [444] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_writebin, 4, .production_id = 7),
  [446] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_writebin, 4, .production_id = 7),
  [448] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_char_literal, 3),
  [450] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_char_literal, 3),
  [452] = {.entry = {.count = 1, .reusable = false}}, SHIFT(175),
  [454] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_extended, 1),
  [456] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_extended, 1),
  [458] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_mnemonic, 1),
  [460] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_mnemonic, 1),
  [462] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_fdb_repeat1, 2),
  [464] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_fdb_repeat1, 2),
  [466] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_unary_expression, 2, .production_id = 3),
  [468] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_expression, 2, .production_id = 3),
  [470] = {.entry = {.count = 1, .reusable = false}}, SHIFT(82),
  [472] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__incbinargs, 2, .production_id = 8),
  [474] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__incbinargs, 2, .production_id = 8),
  [476] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_parenthesized_expression, 3),
  [478] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parenthesized_expression, 3),
  [480] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_org, 2),
  [482] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_org, 2),
  [484] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_equate, 3),
  [486] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_equate, 3),
  [488] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_immediate, 2),
  [490] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_immediate, 2),
  [492] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_grabmem, 4, .production_id = 6),
  [494] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_grabmem, 4, .production_id = 6),
  [496] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_direct_page, 2),
  [498] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_direct_page, 2),
  [500] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_zmb, 2),
  [502] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_zmb, 2),
  [504] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_bsz, 4, .production_id = 9),
  [506] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_bsz, 4, .production_id = 9),
  [508] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fill, 4, .production_id = 9),
  [510] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fill, 4, .production_id = 9),
  [512] = {.entry = {.count = 1, .reusable = true}}, SHIFT(206),
  [514] = {.entry = {.count = 1, .reusable = true}}, SHIFT(57),
  [516] = {.entry = {.count = 1, .reusable = true}}, SHIFT(129),
  [518] = {.entry = {.count = 1, .reusable = false}}, SHIFT(129),
  [520] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_writebin, 6, .production_id = 10),
  [522] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_writebin, 6, .production_id = 10),
  [524] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_exec_addr, 2),
  [526] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_exec_addr, 2),
  [528] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_put, 2),
  [530] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_put, 2),
  [532] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_setdp, 2),
  [534] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_setdp, 2),
  [536] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_rmb, 2),
  [538] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_rmb, 2),
  [540] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__incbinargs, 4, .production_id = 11),
  [542] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__incbinargs, 4, .production_id = 11),
  [544] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_zmd, 2),
  [546] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_zmd, 2),
  [548] = {.entry = {.count = 1, .reusable = true}}, SHIFT(120),
  [550] = {.entry = {.count = 1, .reusable = true}}, SHIFT(42),
  [552] = {.entry = {.count = 1, .reusable = true}}, SHIFT(121),
  [554] = {.entry = {.count = 1, .reusable = true}}, SHIFT(254),
  [556] = {.entry = {.count = 1, .reusable = true}}, SHIFT(13),
  [558] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [560] = {.entry = {.count = 1, .reusable = false}}, SHIFT(68),
  [562] = {.entry = {.count = 1, .reusable = true}}, SHIFT(132),
  [564] = {.entry = {.count = 1, .reusable = false}}, SHIFT(132),
  [566] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [568] = {.entry = {.count = 1, .reusable = false}}, SHIFT(78),
  [570] = {.entry = {.count = 1, .reusable = true}}, SHIFT(147),
  [572] = {.entry = {.count = 1, .reusable = false}}, SHIFT(147),
  [574] = {.entry = {.count = 1, .reusable = true}}, SHIFT(74),
  [576] = {.entry = {.count = 1, .reusable = false}}, SHIFT(74),
  [578] = {.entry = {.count = 1, .reusable = true}}, SHIFT(75),
  [580] = {.entry = {.count = 1, .reusable = false}}, SHIFT(75),
  [582] = {.entry = {.count = 1, .reusable = true}}, SHIFT(146),
  [584] = {.entry = {.count = 1, .reusable = false}}, SHIFT(146),
  [586] = {.entry = {.count = 1, .reusable = true}}, SHIFT(66),
  [588] = {.entry = {.count = 1, .reusable = false}}, SHIFT(66),
  [590] = {.entry = {.count = 1, .reusable = true}}, SHIFT(64),
  [592] = {.entry = {.count = 1, .reusable = false}}, SHIFT(64),
  [594] = {.entry = {.count = 1, .reusable = true}}, SHIFT(150),
  [596] = {.entry = {.count = 1, .reusable = false}}, SHIFT(150),
  [598] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [600] = {.entry = {.count = 1, .reusable = false}}, SHIFT(133),
  [602] = {.entry = {.count = 1, .reusable = true}}, SHIFT(76),
  [604] = {.entry = {.count = 1, .reusable = false}}, SHIFT(76),
  [606] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [608] = {.entry = {.count = 1, .reusable = false}}, SHIFT(77),
  [610] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [612] = {.entry = {.count = 1, .reusable = false}}, SHIFT(65),
  [614] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [616] = {.entry = {.count = 1, .reusable = false}}, SHIFT(52),
  [618] = {.entry = {.count = 1, .reusable = true}}, SHIFT(148),
  [620] = {.entry = {.count = 1, .reusable = false}}, SHIFT(148),
  [622] = {.entry = {.count = 1, .reusable = true}}, SHIFT(141),
  [624] = {.entry = {.count = 1, .reusable = false}}, SHIFT(141),
  [626] = {.entry = {.count = 1, .reusable = true}}, SHIFT(40),
  [628] = {.entry = {.count = 1, .reusable = false}}, SHIFT(40),
  [630] = {.entry = {.count = 1, .reusable = true}}, SHIFT(43),
  [632] = {.entry = {.count = 1, .reusable = false}}, SHIFT(43),
  [634] = {.entry = {.count = 1, .reusable = true}}, SHIFT(51),
  [636] = {.entry = {.count = 1, .reusable = false}}, SHIFT(51),
  [638] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [640] = {.entry = {.count = 1, .reusable = false}}, SHIFT(50),
  [642] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [644] = {.entry = {.count = 1, .reusable = false}}, SHIFT(49),
  [646] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [648] = {.entry = {.count = 1, .reusable = false}}, SHIFT(48),
  [650] = {.entry = {.count = 1, .reusable = true}}, SHIFT(47),
  [652] = {.entry = {.count = 1, .reusable = false}}, SHIFT(47),
  [654] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [656] = {.entry = {.count = 1, .reusable = false}}, SHIFT(46),
  [658] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [660] = {.entry = {.count = 1, .reusable = false}}, SHIFT(45),
  [662] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [664] = {.entry = {.count = 1, .reusable = false}}, SHIFT(44),
  [666] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [668] = {.entry = {.count = 1, .reusable = false}}, SHIFT(55),
  [670] = {.entry = {.count = 1, .reusable = true}}, SHIFT(67),
  [672] = {.entry = {.count = 1, .reusable = false}}, SHIFT(67),
  [674] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [676] = {.entry = {.count = 1, .reusable = false}}, SHIFT(41),
  [678] = {.entry = {.count = 1, .reusable = true}}, SHIFT(69),
  [680] = {.entry = {.count = 1, .reusable = false}}, SHIFT(69),
  [682] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [684] = {.entry = {.count = 1, .reusable = false}}, SHIFT(53),
  [686] = {.entry = {.count = 1, .reusable = true}}, SHIFT(62),
  [688] = {.entry = {.count = 1, .reusable = false}}, SHIFT(62),
  [690] = {.entry = {.count = 1, .reusable = true}}, SHIFT(140),
  [692] = {.entry = {.count = 1, .reusable = false}}, SHIFT(140),
  [694] = {.entry = {.count = 1, .reusable = true}}, SHIFT(139),
  [696] = {.entry = {.count = 1, .reusable = false}}, SHIFT(139),
  [698] = {.entry = {.count = 1, .reusable = true}}, SHIFT(138),
  [700] = {.entry = {.count = 1, .reusable = false}}, SHIFT(138),
  [702] = {.entry = {.count = 1, .reusable = true}}, SHIFT(137),
  [704] = {.entry = {.count = 1, .reusable = false}}, SHIFT(137),
  [706] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [708] = {.entry = {.count = 1, .reusable = false}}, SHIFT(70),
  [710] = {.entry = {.count = 1, .reusable = true}}, SHIFT(71),
  [712] = {.entry = {.count = 1, .reusable = false}}, SHIFT(71),
  [714] = {.entry = {.count = 1, .reusable = true}}, SHIFT(60),
  [716] = {.entry = {.count = 1, .reusable = false}}, SHIFT(60),
  [718] = {.entry = {.count = 1, .reusable = true}}, SHIFT(149),
  [720] = {.entry = {.count = 1, .reusable = false}}, SHIFT(149),
  [722] = {.entry = {.count = 1, .reusable = true}}, SHIFT(61),
  [724] = {.entry = {.count = 1, .reusable = false}}, SHIFT(61),
  [726] = {.entry = {.count = 1, .reusable = true}}, SHIFT(136),
  [728] = {.entry = {.count = 1, .reusable = false}}, SHIFT(136),
  [730] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [732] = {.entry = {.count = 1, .reusable = false}}, SHIFT(73),
  [734] = {.entry = {.count = 1, .reusable = true}}, SHIFT(131),
  [736] = {.entry = {.count = 1, .reusable = false}}, SHIFT(131),
  [738] = {.entry = {.count = 1, .reusable = true}}, SHIFT(135),
  [740] = {.entry = {.count = 1, .reusable = false}}, SHIFT(135),
  [742] = {.entry = {.count = 1, .reusable = true}}, SHIFT(134),
  [744] = {.entry = {.count = 1, .reusable = false}}, SHIFT(134),
  [746] = {.entry = {.count = 1, .reusable = true}}, SHIFT(144),
  [748] = {.entry = {.count = 1, .reusable = false}}, SHIFT(144),
  [750] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [752] = {.entry = {.count = 1, .reusable = false}}, SHIFT(79),
  [754] = {.entry = {.count = 1, .reusable = true}}, SHIFT(127),
  [756] = {.entry = {.count = 1, .reusable = true}}, SHIFT(194),
  [758] = {.entry = {.count = 1, .reusable = true}}, SHIFT(96),
  [760] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [762] = {.entry = {.count = 1, .reusable = true}}, SHIFT(114),
  [764] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [766] = {.entry = {.count = 1, .reusable = false}}, SHIFT(116),
  [768] = {.entry = {.count = 1, .reusable = true}}, SHIFT(122),
  [770] = {.entry = {.count = 1, .reusable = false}}, SHIFT(81),
  [772] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [774] = {.entry = {.count = 1, .reusable = false}}, SHIFT(125),
  [776] = {.entry = {.count = 1, .reusable = true}}, SHIFT(125),
  [778] = {.entry = {.count = 1, .reusable = true}}, SHIFT(126),
  [780] = {.entry = {.count = 1, .reusable = true}}, SHIFT(173),
  [782] = {.entry = {.count = 1, .reusable = true}}, SHIFT(219),
  [784] = {.entry = {.count = 1, .reusable = true}}, SHIFT(266),
  [786] = {.entry = {.count = 1, .reusable = true}}, SHIFT(142),
  [788] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [790] = {.entry = {.count = 1, .reusable = true}}, SHIFT(63),
  [792] = {.entry = {.count = 1, .reusable = true}}, SHIFT(108),
  [794] = {.entry = {.count = 1, .reusable = true}}, SHIFT(171),
  [796] = {.entry = {.count = 1, .reusable = false}}, SHIFT(171),
  [798] = {.entry = {.count = 1, .reusable = true}}, SHIFT(311),
  [800] = {.entry = {.count = 1, .reusable = false}}, SHIFT(311),
  [802] = {.entry = {.count = 1, .reusable = true}}, SHIFT(227),
  [804] = {.entry = {.count = 1, .reusable = false}}, SHIFT(227),
  [806] = {.entry = {.count = 1, .reusable = true}}, SHIFT(181),
  [808] = {.entry = {.count = 1, .reusable = false}}, SHIFT(181),
  [810] = {.entry = {.count = 1, .reusable = true}}, SHIFT(301),
  [812] = {.entry = {.count = 1, .reusable = true}}, SHIFT(32),
  [814] = {.entry = {.count = 1, .reusable = true}}, SHIFT(287),
  [816] = {.entry = {.count = 1, .reusable = true}}, SHIFT(302),
  [818] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [820] = {.entry = {.count = 1, .reusable = true}}, SHIFT(36),
  [822] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fcc_repeat1, 2), SHIFT_REPEAT(265),
  [825] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_fcc_repeat1, 2),
  [827] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_fcc_repeat1, 2),
  [829] = {.entry = {.count = 1, .reusable = false}}, SHIFT(112),
  [831] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_incbin, 2, .production_id = 1),
  [833] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_incbin, 2, .production_id = 1),
  [835] = {.entry = {.count = 1, .reusable = false}}, SHIFT(265),
  [837] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fcc, 3),
  [839] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fcc, 3),
  [841] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fcb, 3),
  [843] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fcb, 3),
  [845] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fdb, 3),
  [847] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fdb, 3),
  [849] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_global_scoped_id, 1),
  [851] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_global_scoped_id, 1),
  [853] = {.entry = {.count = 1, .reusable = false}}, SHIFT(296),
  [855] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_global_scoped_id_repeat1, 2),
  [857] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_global_scoped_id_repeat1, 2),
  [859] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_global_scoped_id_repeat1, 2), SHIFT_REPEAT(296),
  [862] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fdb_repeat1, 2), SHIFT_REPEAT(119),
  [865] = {.entry = {.count = 1, .reusable = true}}, SHIFT(27),
  [867] = {.entry = {.count = 1, .reusable = true}}, SHIFT(18),
  [869] = {.entry = {.count = 1, .reusable = true}}, SHIFT(38),
  [871] = {.entry = {.count = 1, .reusable = false}}, SHIFT(154),
  [873] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_reg_set, 1),
  [875] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_reg_set, 1),
  [877] = {.entry = {.count = 1, .reusable = false}}, SHIFT(229),
  [879] = {.entry = {.count = 1, .reusable = true}}, SHIFT(179),
  [881] = {.entry = {.count = 1, .reusable = true}}, SHIFT(230),
  [883] = {.entry = {.count = 1, .reusable = true}}, SHIFT(277),
  [885] = {.entry = {.count = 1, .reusable = false}}, SHIFT(275),
  [887] = {.entry = {.count = 1, .reusable = true}}, SHIFT(276),
  [889] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_incbinref, 2, .production_id = 1),
  [891] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_incbinref, 2, .production_id = 1),
  [893] = {.entry = {.count = 1, .reusable = true}}, SHIFT(193),
  [895] = {.entry = {.count = 1, .reusable = false}}, SHIFT(191),
  [897] = {.entry = {.count = 1, .reusable = true}}, SHIFT(192),
  [899] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fcc, 2),
  [901] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fcc, 2),
  [903] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_reg_set, 2),
  [905] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_reg_set, 2),
  [907] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_reg_set_repeat1, 2), SHIFT_REPEAT(154),
  [910] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_reg_set_repeat1, 2),
  [912] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_reg_set_repeat1, 2),
  [914] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_zero_index, 2),
  [916] = {.entry = {.count = 1, .reusable = false}}, SHIFT(223),
  [918] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_zero_index, 2),
  [920] = {.entry = {.count = 1, .reusable = false}}, SHIFT(221),
  [922] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 2),
  [924] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 2),
  [926] = {.entry = {.count = 1, .reusable = true}}, SHIFT(300),
  [928] = {.entry = {.count = 1, .reusable = true}}, SHIFT(261),
  [930] = {.entry = {.count = 1, .reusable = true}}, SHIFT(202),
  [932] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string_literal, 3),
  [934] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string_literal, 3),
  [936] = {.entry = {.count = 1, .reusable = true}}, SHIFT(216),
  [938] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_operand, 1),
  [940] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_operand, 1),
  [942] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_add_d, 3),
  [944] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_add_d, 3),
  [946] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_pc_offset, 3),
  [948] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_pc_offset, 3),
  [950] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_pc_offset_rel, 3),
  [952] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_pc_offset_rel, 3),
  [954] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_constant_offset, 3),
  [956] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_constant_offset, 3),
  [958] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_macro, 4),
  [960] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro, 4),
  [962] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_importer, 2),
  [964] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_importer, 2),
  [966] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__xfers, 2),
  [968] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__xfers, 2),
  [970] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__regsets, 2),
  [972] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__regsets, 2),
  [974] = {.entry = {.count = 1, .reusable = true}}, SHIFT(234),
  [976] = {.entry = {.count = 1, .reusable = true}}, SHIFT(270),
  [978] = {.entry = {.count = 1, .reusable = true}}, SHIFT(190),
  [980] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_macro, 5),
  [982] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro, 5),
  [984] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_macro, 3),
  [986] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro, 3),
  [988] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__opcode_arg, 2),
  [990] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__opcode_arg, 2),
  [992] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_scope, 2),
  [994] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_scope, 2),
  [996] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_add_a, 3),
  [998] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_add_a, 3),
  [1000] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_include, 2),
  [1002] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_include, 2),
  [1004] = {.entry = {.count = 1, .reusable = true}}, SHIFT(274),
  [1006] = {.entry = {.count = 1, .reusable = true}}, SHIFT(273),
  [1008] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__command_label, 2),
  [1010] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__command_label, 2),
  [1012] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_add_b, 3),
  [1014] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_add_b, 3),
  [1016] = {.entry = {.count = 1, .reusable = true}}, SHIFT(210),
  [1018] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_indirect, 3),
  [1020] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_indirect, 3),
  [1022] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__opcode_label, 2),
  [1024] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__opcode_label, 2),
  [1026] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_post_inc_inc, 3),
  [1028] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_post_inc_inc, 3),
  [1030] = {.entry = {.count = 1, .reusable = true}}, SHIFT(292),
  [1032] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_post_inc, 3),
  [1034] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_post_inc, 3),
  [1036] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_incbin, 3, .production_id = 4),
  [1038] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_incbin, 3, .production_id = 4),
  [1040] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_pre_dec_dec, 3),
  [1042] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_pre_dec_dec, 3),
  [1044] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_pre_dec, 3),
  [1046] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_pre_dec, 3),
  [1048] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_reg_xfer, 3),
  [1050] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_reg_xfer, 3),
  [1052] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_incbinref, 3, .production_id = 4),
  [1054] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_incbinref, 3, .production_id = 4),
  [1056] = {.entry = {.count = 1, .reusable = true}}, SHIFT(226),
  [1058] = {.entry = {.count = 1, .reusable = true}}, SHIFT(225),
  [1060] = {.entry = {.count = 1, .reusable = false}}, SHIFT(233),
  [1062] = {.entry = {.count = 1, .reusable = false}}, SHIFT(299),
  [1064] = {.entry = {.count = 1, .reusable = true}}, SHIFT(233),
  [1066] = {.entry = {.count = 1, .reusable = false}}, SHIFT(236),
  [1068] = {.entry = {.count = 1, .reusable = false}}, SHIFT(180),
  [1070] = {.entry = {.count = 1, .reusable = true}}, SHIFT(236),
  [1072] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_literal_repeat1, 2), SHIFT_REPEAT(233),
  [1075] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_literal_repeat1, 2),
  [1077] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_string_literal_repeat1, 2), SHIFT_REPEAT(233),
  [1080] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_elem_type, 1),
  [1082] = {.entry = {.count = 1, .reusable = true}}, SHIFT(86),
  [1084] = {.entry = {.count = 1, .reusable = false}}, SHIFT(231),
  [1086] = {.entry = {.count = 1, .reusable = false}}, SHIFT(279),
  [1088] = {.entry = {.count = 1, .reusable = true}}, SHIFT(231),
  [1090] = {.entry = {.count = 1, .reusable = false}}, SHIFT(187),
  [1092] = {.entry = {.count = 1, .reusable = true}}, SHIFT(200),
  [1094] = {.entry = {.count = 1, .reusable = true}}, SHIFT(290),
  [1096] = {.entry = {.count = 1, .reusable = true}}, SHIFT(167),
  [1098] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [1100] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fdb_repeat1, 2), SHIFT_REPEAT(127),
  [1103] = {.entry = {.count = 1, .reusable = true}}, SHIFT(294),
  [1105] = {.entry = {.count = 1, .reusable = true}}, SHIFT(205),
  [1107] = {.entry = {.count = 1, .reusable = true}}, SHIFT(296),
  [1109] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_macro_args_repeat1, 2), SHIFT_REPEAT(200),
  [1112] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_macro_args_repeat1, 2),
  [1114] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_struct_def_repeat1, 2), SHIFT_REPEAT(183),
  [1117] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_struct_def_repeat1, 2),
  [1119] = {.entry = {.count = 1, .reusable = true}}, SHIFT(157),
  [1121] = {.entry = {.count = 1, .reusable = true}}, SHIFT(34),
  [1123] = {.entry = {.count = 1, .reusable = true}}, SHIFT(169),
  [1125] = {.entry = {.count = 1, .reusable = true}}, SHIFT(168),
  [1127] = {.entry = {.count = 1, .reusable = true}}, SHIFT(182),
  [1129] = {.entry = {.count = 1, .reusable = true}}, SHIFT(5),
  [1131] = {.entry = {.count = 1, .reusable = false}}, SHIFT(297),
  [1133] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [1135] = {.entry = {.count = 1, .reusable = false}}, SHIFT(280),
  [1137] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_struct_elem, 3),
  [1139] = {.entry = {.count = 1, .reusable = true}}, SHIFT(235),
  [1141] = {.entry = {.count = 1, .reusable = true}}, SHIFT(232),
  [1143] = {.entry = {.count = 1, .reusable = true}}, SHIFT(272),
  [1145] = {.entry = {.count = 1, .reusable = true}}, SHIFT(271),
  [1147] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_elem_type, 2),
  [1149] = {.entry = {.count = 1, .reusable = true}}, SHIFT(8),
  [1151] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 3),
  [1153] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [1155] = {.entry = {.count = 1, .reusable = true}}, SHIFT(291),
  [1157] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [1159] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [1161] = {.entry = {.count = 1, .reusable = true}}, SHIFT(214),
  [1163] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [1165] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [1167] = {.entry = {.count = 1, .reusable = true}}, SHIFT(278),
  [1169] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [1171] = {.entry = {.count = 1, .reusable = true}}, SHIFT(156),
  [1173] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro_args, 4),
  [1175] = {.entry = {.count = 1, .reusable = true}}, SHIFT(289),
  [1177] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro_args, 3),
  [1179] = {.entry = {.count = 1, .reusable = true}}, SHIFT(213),
  [1181] = {.entry = {.count = 1, .reusable = true}}, SHIFT(184),
  [1183] = {.entry = {.count = 1, .reusable = true}}, SHIFT(143),
  [1185] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [1187] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_macro_args, 2),
  [1189] = {.entry = {.count = 1, .reusable = true}}, SHIFT(204),
  [1191] = {.entry = {.count = 1, .reusable = true}}, SHIFT(188),
  [1193] = {.entry = {.count = 1, .reusable = true}}, SHIFT(285),
  [1195] = {.entry = {.count = 1, .reusable = true}}, SHIFT(218),
  [1197] = {.entry = {.count = 1, .reusable = true}}, SHIFT(17),
  [1199] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [1201] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [1203] = {.entry = {.count = 1, .reusable = true}}, SHIFT(222),
  [1205] = {.entry = {.count = 1, .reusable = true}}, SHIFT(153),
  [1207] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [1209] = {.entry = {.count = 1, .reusable = true}}, SHIFT(155),
  [1211] = {.entry = {.count = 1, .reusable = true}}, SHIFT(199),
  [1213] = {.entry = {.count = 1, .reusable = true}}, SHIFT(309),
  [1215] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [1217] = {.entry = {.count = 1, .reusable = true}}, SHIFT(284),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef _WIN32
#define extern __declspec(dllexport)
#endif

extern const TSLanguage *tree_sitter_gazm(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
