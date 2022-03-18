mod instr;

use self::instr::*;
use super::InstrFunction;

pub const THUMB_OPCODE_TABLE: [InstrFunction; 256] = [
    /* Bits 15-12 */
    /* 0x0 */
    thumb_lsl_imm,
    thumb_lsl_imm,
    thumb_lsl_imm,
    thumb_lsl_imm,
    thumb_lsl_imm,
    thumb_lsl_imm,
    thumb_lsl_imm,
    thumb_lsl_imm,
    thumb_lsr_imm,
    thumb_lsr_imm,
    thumb_lsr_imm,
    thumb_lsr_imm,
    thumb_lsr_imm,
    thumb_lsr_imm,
    thumb_lsr_imm,
    thumb_lsr_imm,
    /* 0x1 */
    thumb_asr_imm,
    thumb_asr_imm,
    thumb_asr_imm,
    thumb_asr_imm,
    thumb_asr_imm,
    thumb_asr_imm,
    thumb_asr_imm,
    thumb_asr_imm,
    thumb_add_reg,
    thumb_add_reg,
    thumb_sub_reg,
    thumb_sub_reg,
    thumb_add_imm3,
    thumb_add_imm3,
    thumb_sub_imm3,
    thumb_sub_imm3,
    /* 0x2 */
    thumb_mov_i8_r0,
    thumb_mov_i8_r1,
    thumb_mov_i8_r2,
    thumb_mov_i8_r3,
    thumb_mov_i8_r4,
    thumb_mov_i8_r5,
    thumb_mov_i8_r6,
    thumb_mov_i8_r7,
    thumb_cmp_i8_r0,
    thumb_cmp_i8_r1,
    thumb_cmp_i8_r2,
    thumb_cmp_i8_r3,
    thumb_cmp_i8_r4,
    thumb_cmp_i8_r5,
    thumb_cmp_i8_r6,
    thumb_cmp_i8_r7,
    /* 0x3 */
    thumb_add_i8_r0,
    thumb_add_i8_r1,
    thumb_add_i8_r2,
    thumb_add_i8_r3,
    thumb_add_i8_r4,
    thumb_add_i8_r5,
    thumb_add_i8_r6,
    thumb_add_i8_r7,
    thumb_sub_i8_r0,
    thumb_sub_i8_r1,
    thumb_sub_i8_r2,
    thumb_sub_i8_r3,
    thumb_sub_i8_r4,
    thumb_sub_i8_r5,
    thumb_sub_i8_r6,
    thumb_sub_i8_r7,
    /* 0x4 */
    thumb_dp_g1,
    thumb_dp_g2,
    thumb_dp_g3,
    thumb_dp_g4,
    thumb_addh,
    thumb_cmph,
    thumb_movh,
    thumb_bx_reg,
    thumb_ldr_pc_r0,
    thumb_ldr_pc_r1,
    thumb_ldr_pc_r2,
    thumb_ldr_pc_r3,
    thumb_ldr_pc_r4,
    thumb_ldr_pc_r5,
    thumb_ldr_pc_r6,
    thumb_ldr_pc_r7,
    /* 0x5 */
    thumb_str_reg,
    thumb_str_reg,
    thumb_strh_reg,
    thumb_strh_reg,
    thumb_strb_reg,
    thumb_strb_reg,
    thumb_ldrsb_reg,
    thumb_ldrsb_reg,
    thumb_ldr_reg,
    thumb_ldr_reg,
    thumb_ldrh_reg,
    thumb_ldrh_reg,
    thumb_ldrb_reg,
    thumb_ldrb_reg,
    thumb_ldrsh_reg,
    thumb_ldrsh_reg,
    /* 0x6 */
    thumb_str_imm5,
    thumb_str_imm5,
    thumb_str_imm5,
    thumb_str_imm5,
    thumb_str_imm5,
    thumb_str_imm5,
    thumb_str_imm5,
    thumb_str_imm5,
    thumb_ldr_imm5,
    thumb_ldr_imm5,
    thumb_ldr_imm5,
    thumb_ldr_imm5,
    thumb_ldr_imm5,
    thumb_ldr_imm5,
    thumb_ldr_imm5,
    thumb_ldr_imm5,
    /* 0x7 */
    thumb_strb_imm5,
    thumb_strb_imm5,
    thumb_strb_imm5,
    thumb_strb_imm5,
    thumb_strb_imm5,
    thumb_strb_imm5,
    thumb_strb_imm5,
    thumb_strb_imm5,
    thumb_ldrb_imm5,
    thumb_ldrb_imm5,
    thumb_ldrb_imm5,
    thumb_ldrb_imm5,
    thumb_ldrb_imm5,
    thumb_ldrb_imm5,
    thumb_ldrb_imm5,
    thumb_ldrb_imm5,
    /* 0x8 */
    thumb_strh_imm5,
    thumb_strh_imm5,
    thumb_strh_imm5,
    thumb_strh_imm5,
    thumb_strh_imm5,
    thumb_strh_imm5,
    thumb_strh_imm5,
    thumb_strh_imm5,
    thumb_ldrh_imm5,
    thumb_ldrh_imm5,
    thumb_ldrh_imm5,
    thumb_ldrh_imm5,
    thumb_ldrh_imm5,
    thumb_ldrh_imm5,
    thumb_ldrh_imm5,
    thumb_ldrh_imm5,
    /* 0x9 */
    thumb_strsp_r0,
    thumb_strsp_r1,
    thumb_strsp_r2,
    thumb_strsp_r3,
    thumb_strsp_r4,
    thumb_strsp_r5,
    thumb_strsp_r6,
    thumb_strsp_r7,
    thumb_ldrsp_r0,
    thumb_ldrsp_r1,
    thumb_ldrsp_r2,
    thumb_ldrsp_r3,
    thumb_ldrsp_r4,
    thumb_ldrsp_r5,
    thumb_ldrsp_r6,
    thumb_ldrsp_r7,
    /* 0xA */
    thumb_addpc_r0,
    thumb_addpc_r1,
    thumb_addpc_r2,
    thumb_addpc_r3,
    thumb_addpc_r4,
    thumb_addpc_r5,
    thumb_addpc_r6,
    thumb_addpc_r7,
    thumb_addsp_r0,
    thumb_addsp_r1,
    thumb_addsp_r2,
    thumb_addsp_r3,
    thumb_addsp_r4,
    thumb_addsp_r5,
    thumb_addsp_r6,
    thumb_addsp_r7,
    /* 0xB */
    thumb_addsp_imm7,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_push,
    thumb_push_lr,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_pop,
    thumb_pop_pc,
    thumb_undefined,
    thumb_undefined,
    /* 0xC */
    thumb_stmia_r0,
    thumb_stmia_r1,
    thumb_stmia_r2,
    thumb_stmia_r3,
    thumb_stmia_r4,
    thumb_stmia_r5,
    thumb_stmia_r6,
    thumb_stmia_r7,
    thumb_ldmia_r0,
    thumb_ldmia_r1,
    thumb_ldmia_r2,
    thumb_ldmia_r3,
    thumb_ldmia_r4,
    thumb_ldmia_r5,
    thumb_ldmia_r6,
    thumb_ldmia_r7,
    /* 0xD */
    thumb_beq,
    thumb_bne,
    thumb_bcs,
    thumb_bcc,
    thumb_bmi,
    thumb_bpl,
    thumb_bvs,
    thumb_bvc,
    thumb_bhi,
    thumb_bls,
    thumb_bge,
    thumb_blt,
    thumb_bgt,
    thumb_ble,
    thumb_undefined,
    thumb_swi,
    /* 0xE */
    thumb_b,
    thumb_b,
    thumb_b,
    thumb_b,
    thumb_b,
    thumb_b,
    thumb_b,
    thumb_b,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    thumb_undefined,
    /* 0xF */
    thumb_bl_setup,
    thumb_bl_setup,
    thumb_bl_setup,
    thumb_bl_setup,
    thumb_bl_setup,
    thumb_bl_setup,
    thumb_bl_setup,
    thumb_bl_setup,
    thumb_bl_off,
    thumb_bl_off,
    thumb_bl_off,
    thumb_bl_off,
    thumb_bl_off,
    thumb_bl_off,
    thumb_bl_off,
    thumb_bl_off,
];