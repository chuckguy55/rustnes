#[macro_escape]

use nes::{PrgRom, PRG_ROM_BANK_SIZE};
use nes::{ChrRom, CHR_ROM_BANK_SIZE};
use nes::test;

use cpu::{Cpu, CpuState, CpuFlags, Ram, RAM_SIZE};
use cpu::{C_FLAG, Z_FLAG, I_FLAG, D_FLAG, B_FLAG, X_FLAG, V_FLAG, N_FLAG};
use cpu::isa;

use ppu::Ppu;

/// # Macros
///
///
macro_rules! cpu(
    () => (
        get_empty_cpu()
    );
    ($prg_rom:expr) => (
        get_cpu_with_prg_rom($prg_rom)
    );
    ($prg_rom:expr, $ram:expr) => (
        get_cpu_with_prg_rom_and_ram($prg_rom, $ram)
    );
)

macro_rules! ram(
    () => (
        get_empty_ram()
    );
    ($init:expr) => (
        get_initialized_ram($init)
    );
)

/// # Macro Helpers
///     
///

fn get_empty_cpu_state() -> CpuState {
    CpuState::new()
}

fn get_empty_ram() -> Ram {
    [0u8, ..RAM_SIZE]
}

fn get_initialized_ram(init: u8) -> Ram {
    [init, ..RAM_SIZE]
}

fn get_empty_cpu() -> Cpu {
    let state = get_empty_cpu_state();
    let prg_rom = prg_rom!();
    let ram = [0u8, ..RAM_SIZE];

    Cpu {
        state: state,
        prg_rom: prg_rom,
        ram: ram,
        ppu: Ppu::new(vec![[0u8, ..CHR_ROM_BANK_SIZE]]),
    }
}

fn get_cpu_with_prg_rom(prg_rom: PrgRom) -> Cpu {
    let state = get_empty_cpu_state();
    let ram = [0u8, ..RAM_SIZE];

    Cpu {
        state: state,
        prg_rom: prg_rom,
        ram: ram,
        ppu: Ppu::new(vec![[0u8, ..CHR_ROM_BANK_SIZE]]),
    }
}

fn get_cpu_with_prg_rom_and_ram(prg_rom: PrgRom, ram: Ram) -> Cpu {
    let state = get_empty_cpu_state();

    Cpu {
        state: state,
        prg_rom: prg_rom,
        ram: ram,
        ppu: Ppu::new(vec![[0u8, ..CHR_ROM_BANK_SIZE]]),
    }
}

/// # Sanity Test
///
///

#[test]
fn cpu_sanity_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);

    //ADC $AA
    prg_rom_bank[0x0000] = 0x65;
    prg_rom_bank[0x0001] = 0xAA; 

    let mut prg_rom = prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5));

    let mut ram = ram!(0xC5);

    ram[0xAA] = 0x01;

    let mut cpu = cpu!(prg_rom, ram);

    cpu.state.PC = 0x8000;
    cpu.state.A = 0x01;

    let instr = cpu.instr_decode();
    assert_eq!(instr.instr, isa::ADC);
    assert_eq!(instr.address_mode, isa::ZP);
    assert_eq!(cpu.state.PC, 0x8001);

    let (m_addr, page_boundary_crossed) = cpu.instr_mem_addr(instr.address_mode);
    assert_eq!(m_addr, 0xAA);
    assert_eq!(cpu.state.PC, 0x8002);

    let m = cpu.instr_mem_read(m_addr, instr);
    assert_eq!(m, 0x01);
    
    let x = cpu.instr_exec(instr.instr, m);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x02);
}

/// # Address Mode Tests
///
///
#[test]
fn cpu_instr_mem_addr_zp_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xAA;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;
    
    //$AA
    assert_eq!(cpu.instr_mem_addr(isa::ZP), (0xAA, false));
    assert_eq!(cpu.state.PC, 0x8001);
}

#[test]
fn cpu_instr_mem_addr_zpx_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xF0;
    prg_rom_bank[1] = 0xFF;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;
    cpu.state.X = 0x0F;
    
    //$F0,$0F
    assert_eq!(cpu.instr_mem_addr(isa::ZPX), (0x00FF, false));
    assert_eq!(cpu.state.PC, 0x8001);

    //$FF,$0F
    assert_eq!(cpu.instr_mem_addr(isa::ZPX), (0x000E, false));
    assert_eq!(cpu.state.PC, 0x8002);
}

#[test]
fn cpu_instr_mem_addr_zpy_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xF0;
    prg_rom_bank[1] = 0xFF;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;
    cpu.state.Y = 0x0F;
    
    //$F0,$0F
    assert_eq!(cpu.instr_mem_addr(isa::ZPY), (0x00FF, false));
    assert_eq!(cpu.state.PC, 0x8001);

    //$FF,$0F
    assert_eq!(cpu.instr_mem_addr(isa::ZPY), (0x000E, false));
    assert_eq!(cpu.state.PC, 0x8002);
}

#[test]
fn cpu_instr_mem_addr_abs_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xAA;
    prg_rom_bank[1] = 0xBB;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;
    
    //$BBAA
    assert_eq!(cpu.instr_mem_addr(isa::ABS), (0xBBAA, false));
    assert_eq!(cpu.state.PC, 0x8002);
}

#[test]
fn cpu_instr_mem_addr_absx_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xAA;
    prg_rom_bank[1] = 0xBB;
    prg_rom_bank[2] = 0xFF;
    prg_rom_bank[3] = 0xFF;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;
    cpu.state.X = 2;
    
    //$BBAA,$02
    assert_eq!(cpu.instr_mem_addr(isa::ABSX), (0xBBAC, false));
    assert_eq!(cpu.state.PC, 0x8002);

    //$FFFF,$02
    assert_eq!(cpu.instr_mem_addr(isa::ABSX), (0x0001, true));
    assert_eq!(cpu.state.PC, 0x8004);
}

#[test]
fn cpu_instr_mem_addr_absy_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xAA;
    prg_rom_bank[1] = 0xBB;
    prg_rom_bank[2] = 0xFF;
    prg_rom_bank[3] = 0xFF;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!()));
    cpu.state.PC = 0x8000;
    cpu.state.Y = 2;
    
    //$BBAA,$02
    assert_eq!(cpu.instr_mem_addr(isa::ABSY), (0xBBAC, false));
    assert_eq!(cpu.state.PC, 0x8002);

    //$FFFF,$02
    assert_eq!(cpu.instr_mem_addr(isa::ABSY), (0x0001, true));
    assert_eq!(cpu.state.PC, 0x8004);
}

#[test]
fn cpu_instr_mem_addr_ind_test() {
    let mut prg_rom_bank_0 = prg_rom_bank!(0xC5);
    //$80AA in lower bank
    prg_rom_bank_0[0] = 0xAA;
    prg_rom_bank_0[1] = 0x80;
    //$C0AA in upper bank
    prg_rom_bank_0[2] = 0xAA;
    prg_rom_bank_0[3] = 0xC0;

    //$DDCC
    prg_rom_bank_0[0x00AA] = 0xCC;
    prg_rom_bank_0[0x00AB] = 0xDD;

    let mut prg_rom_bank_1 = prg_rom_bank!(0xC5);
    //$FFEE
    prg_rom_bank_1[0x00AA] = 0xEE;
    prg_rom_bank_1[0x00AB] = 0xFF;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank_0, prg_rom_bank_1));
    cpu.state.PC = 0x8000;
    
    //[ $CC ] $80AA
    //[ $DD ] $80AB
    //($80AA)
    assert_eq!(cpu.instr_mem_addr(isa::IND), (0xDDCC, false));
    assert_eq!(cpu.state.PC, 0x8002);

    //[ $EE ] $C0AA
    //[ $FF ] $C0AB
    //($C0AA)
    assert_eq!(cpu.instr_mem_addr(isa::IND), (0xFFEE, false));
    assert_eq!(cpu.state.PC, 0x8004);
}

#[test]
fn cpu_instr_mem_addr_imp_test() {
    let mut cpu = cpu!(prg_rom!());

    assert_eq!(cpu.instr_mem_addr(isa::IMP), (0x0000, false));
    assert_eq!(cpu.state.PC, 0x0000);
}

#[test]
fn cpu_instr_mem_addr_acc_test() {
    let mut cpu = cpu!(prg_rom!());

    assert_eq!(cpu.instr_mem_addr(isa::ACC), (0x0000, false));
    assert_eq!(cpu.state.PC, 0x0000);
}

#[test]
fn cpu_instr_mem_addr_imm_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0x0000] = 0xAA;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;

    assert_eq!(cpu.instr_mem_addr(isa::IMM), (0x00AA, false));
    assert_eq!(cpu.state.PC, 0x8001);
}

#[test]
fn cpu_instr_mem_addr_rel_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0x0000] = 0xAA;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;

    assert_eq!(cpu.instr_mem_addr(isa::REL), (0x00AA, false));
    assert_eq!(cpu.state.PC, 0x8001);
}

#[test]
fn cpu_instr_mem_addr_indx_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xAA;
    prg_rom_bank[1] = 0xFF;

    let mut ram = ram!();
    ram[0xAC] = 0xBB;
    ram[0xAD] = 0xCC;

    ram[0x01] = 0xDD;
    ram[0x02] = 0xEE;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)), ram);
    cpu.state.PC = 0x8000;
    cpu.state.X = 0x02;

    //($AA,$02)
    assert_eq!(cpu.instr_mem_addr(isa::INDX), (0xCCBB, false));
    assert_eq!(cpu.state.PC, 0x8001);

    //($FF,$02)
    assert_eq!(cpu.instr_mem_addr(isa::INDX), (0xEEDD, false));
    assert_eq!(cpu.state.PC, 0x8002);
}

#[test]
fn cpu_instr_mem_addr_indy_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0] = 0xAA;
    prg_rom_bank[1] = 0xBB;
    prg_rom_bank[2] = 0xCC;

    let mut ram = ram!();
    ram[0xAA] = 0xBB;
    ram[0xAB] = 0xCC;

    ram[0xBB] = 0xDD;
    ram[0xBC] = 0xEE;

    ram[0xCC] = 0xFF;
    ram[0xCD] = 0xFF;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)), ram);
    cpu.state.PC = 0x8000;
    cpu.state.Y = 0x02;

    // [ $BB ] $00AA 
    // [ $CC ] $00AB 
    //($AA),$02
    assert_eq!(cpu.instr_mem_addr(isa::INDY), (0xCCBD, false));
    assert_eq!(cpu.state.PC, 0x8001);

    // [ $DD ] $00BB
    // [ $EE ] $00BC
    //($BB),$02
    assert_eq!(cpu.instr_mem_addr(isa::INDY), (0xEEDF, false));
    assert_eq!(cpu.state.PC, 0x8002);

    // [ $FF ] $00CC
    // [ $FF ] $00CD
    //($CC),$02
    assert_eq!(cpu.instr_mem_addr(isa::INDY), (0x0001, true));
    assert_eq!(cpu.state.PC, 0x8003);
}

/// # Instruction Tests
/// 
///
/// ## Load and Store
#[test]
fn cpu_instr_exec_lda_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDA, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDA, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDA, 0xFF);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_ldx_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDX, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDX, 0xFF);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_ldy_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDY, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDY, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::LDY, 0xFF);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_sta_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::STA, 0x00);
    assert_eq!(x, 0x01);

    cpu = cpu!();
    cpu.state.A = 0x00;
    x = cpu.instr_exec(isa::STA, 0x00);
    assert_eq!(x, 0x00);

    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::STA, 0x00);
    assert_eq!(x, 0xFF);
}

#[test]
fn cpu_instr_exec_stx_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::STX, 0x00);
    assert_eq!(x, 0x01);

    cpu = cpu!();
    cpu.state.X = 0x00;
    x = cpu.instr_exec(isa::STX, 0x00);
    assert_eq!(x, 0x00);

    cpu = cpu!();
    cpu.state.X = 0xFF;
    x = cpu.instr_exec(isa::STX, 0x00);
    assert_eq!(x, 0xFF);
}

#[test]
fn cpu_instr_exec_sty_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.Y = 0x01;
    x = cpu.instr_exec(isa::STY, 0x00);
    assert_eq!(x, 0x01);

    cpu = cpu!();
    cpu.state.Y = 0x00;
    x = cpu.instr_exec(isa::STY, 0x00);
    assert_eq!(x, 0x00);

    cpu = cpu!();
    cpu.state.Y = 0xFF;
    x = cpu.instr_exec(isa::STY, 0x00);
    assert_eq!(x, 0xFF);
}

/// ## Arithmetic Tests
#[test]
fn cpu_instr_exec_adc_test() {
    let mut cpu;
    let mut x;

    //ADC 0x01 + 0x01, no carry, no flags set
    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::ADC, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x02);
    assert_eq!(cpu.state.P, CpuFlags::none());

    //ADC 0x01 + 0x01, with carry, no flags set
    cpu = cpu!();
    cpu.state.A = 0x01;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_exec(isa::ADC, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x03);
    assert_eq!(cpu.state.P, CpuFlags::none());

    //ADC 0xFF + 0x01, no carry, carry set, zero set
    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::ADC, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | Z_FLAG);

    //ADC 0xFF + 0x02, no carry, carry set
    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::ADC, 0x02);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    //ADC 0x7F + 0x01, no carry, overflow set, negative set
    cpu = cpu!();
    cpu.state.A = 0x7F;
    x = cpu.instr_exec(isa::ADC, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | V_FLAG | N_FLAG);

    //ADC 0xFF + 0xFF, no carry, carry set, negative set
    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::ADC, 0xFF);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0xFE);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | N_FLAG);

    //ADC 0x7F + 0x7F, no carry, overflow set, neagtive set
    cpu = cpu!();
    cpu.state.A = 0x7F;
    x = cpu.instr_exec(isa::ADC, 0x7F);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0xFE);
    assert_eq!(cpu.state.P, CpuFlags::none() | V_FLAG | N_FLAG);
    
    //ADC 0xFF + 0x7F, no carry, carry set
    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::ADC, 0x7F);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x7E);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    //ADC 0x80 + 0x80, no carry, carry set, zero set, overflow set
    cpu = cpu!();
    cpu.state.A = 0x80;
    x = cpu.instr_exec(isa::ADC, 0x80);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | V_FLAG | Z_FLAG);

    //ADC 0x80 + 0x7F, no carry, negative set
    cpu = cpu!();
    cpu.state.A = 0x80;
    x = cpu.instr_exec(isa::ADC, 0x7F);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
    
    //ADC 0x80 + 0xFF, no carry, carry set, overflow set
    cpu = cpu!();
    cpu.state.A = 0x80;
    x = cpu.instr_exec(isa::ADC, 0xFF);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x7F);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | V_FLAG);
}

//TODO rewrite these in the new test paradigm
#[test]
fn cpu_instr_exec_sbc_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);

    //SBC IMM 0x5 - 0x3, with carry, carry set
    prg_rom_bank[0x0000] = 0xE9;
    prg_rom_bank[0x0001] = 0x03;

    //SBC IMM 0x5 - 0x3, no carry, carry set
    prg_rom_bank[0x0002] = 0xE9;
    prg_rom_bank[0x0003] = 0x03;

    //SBC IMM 0x5 - 0x5, with carry, carry set, zero set
    prg_rom_bank[0x0004] = 0xE9;
    prg_rom_bank[0x0005] = 0x05;

    //SBC IMM 0x3 - 0x5, with carry, negative set, 
    prg_rom_bank[0x0006] = 0xE9;
    prg_rom_bank[0x0007] = 0x05;

    //SBC IMM 0x80 - 0x01, with carry, overflow set, carry set
    prg_rom_bank[0x0008] = 0xE9;
    prg_rom_bank[0x0009] = 0x01;

    //SBC IMM 0x7F - 0xFF, with carry, carry set, negative set
    prg_rom_bank[0x000A] = 0xE9;
    prg_rom_bank[0x000B] = 0xFF;

    //SBC IMM 0xC0 - 0x40, no carry, overflow set, carry set
    prg_rom_bank[0x000C] = 0xE9;
    prg_rom_bank[0x000D] = 0x40;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!(0xC5)));
    cpu.state.PC = 0x8000;

    //SBC IMM 0x5 - 0x3, with carry, carry set
    cpu.state.A = 0x05;
    cpu.state.P.clear();
    cpu.state.P.insert(C_FLAG);
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0x8002);
    assert_eq!(cpu.state.A, 0x02);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    //SBC IMM 0x5 - 0x3, no carry, carry set
    cpu.state.A = 0x05;
    cpu.state.P.clear();
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0x8004);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    //SBC IMM 0x5 - 0x5, with carry, carry set, zero set
    cpu.state.A = 0x05;
    cpu.state.P.clear();
    cpu.state.P.insert(C_FLAG);
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0x8006);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | Z_FLAG);

    //SBC IMM 0x3 - 0x5, with carry, negative set, overflow set
    cpu.state.A = 0x03;
    cpu.state.P.clear();
    cpu.state.P.insert(C_FLAG);
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0x8008);
    assert_eq!(cpu.state.A, 0xFE);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    //SBC IMM 0x80 - 0x01, with carry, overflow set, carry set
    cpu.state.A = 0x80;
    cpu.state.P.clear();
    cpu.state.P.insert(C_FLAG);
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0x800A);
    assert_eq!(cpu.state.A, 0x7F);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | V_FLAG);

    //SBC IMM 0x7F - 0xFF, with carry, overflow set, negative set
    cpu.state.A = 0x7F;
    cpu.state.P.clear();
    cpu.state.P.insert(C_FLAG);
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0x800C);
    assert_eq!(cpu.state.A, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | V_FLAG | N_FLAG);

    //SBC IMM 0xC0 - 0x40, no carry, overflow set, carry set
    cpu.state.A = 0xC0;
    cpu.state.P.clear();
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0x800E);
    assert_eq!(cpu.state.A, 0x7F);
    assert_eq!(cpu.state.P, CpuFlags::none() | V_FLAG | C_FLAG);
}

#[test]
fn cpu_instr_exec_inc_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::INC, 0x00);
    assert_eq!(x, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    x = cpu.instr_exec(isa::INC, 0xFF);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::INC, 0x7F);
    assert_eq!(x, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_inx_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.X = 0x00;
    x = cpu.instr_exec(isa::INX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.X = 0xFF;
    x = cpu.instr_exec(isa::INX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.X = 0x7F;
    x = cpu.instr_exec(isa::INX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_iny_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.Y = 0x00;
    x = cpu.instr_exec(isa::INY, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.Y = 0xFF;
    x = cpu.instr_exec(isa::INY, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.Y = 0x7F;
    x = cpu.instr_exec(isa::INY, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_dec_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::DEC, 0xFF);
    assert_eq!(x, 0xFE);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::DEC, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::DEC, 0x80);
    assert_eq!(x, 0x7F);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_dex_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.X = 0xFF;
    x = cpu.instr_exec(isa::DEX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0xFE);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::DEX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.X = 0x80;
    x = cpu.instr_exec(isa::DEX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x7F);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_asl_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::ASL, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::ASL, 0x80);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::ASL, 0xFF);
    assert_eq!(x, 0xFE);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | N_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::ASL, 0x01);
    assert_eq!(x, 0x02);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    x = cpu.instr_exec(isa::ASL, 0xAA);
    assert_eq!(x, 0x54);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::ASL, 0x55);
    assert_eq!(x, 0xAA);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_lsr_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::LSR, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::LSR, 0xFF);
    assert_eq!(x, 0x7F);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::LSR, 0x01);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::LSR, 0x02);
    assert_eq!(x, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    x = cpu.instr_exec(isa::LSR, 0xAA);
    assert_eq!(x, 0x55);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    x = cpu.instr_exec(isa::LSR, 0x55);
    assert_eq!(x, 0x2A);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);
}

#[test]
fn cpu_instr_exec_rol_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::ROL, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::ROL, 0xFF);
    assert_eq!(x, 0xFE);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | N_FLAG);

    cpu = cpu!();
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_exec(isa::ROL, 0xFF);
    assert_eq!(x, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | N_FLAG);

    cpu = cpu!();
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_exec(isa::ROL, 0x7F);
    assert_eq!(x, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() |  N_FLAG);
}

/// ## Logic
///
///
#[test]
fn cpu_instr_exec_and_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::AND, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::AND, 0x80);
    assert_eq!(cpu.state.A, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::AND, 0x0F);
    assert_eq!(cpu.state.A, 0x0F);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_ora_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::ORA, 0x00);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0xF0;
    x = cpu.instr_exec(isa::ORA, 0x0F);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x00;
    x = cpu.instr_exec(isa::ORA, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::ORA, 0x00);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_eor_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::EOR, 0xFF);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x80;
    x = cpu.instr_exec(isa::EOR, 0x7F);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::EOR, 0x00);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::EOR, 0x0F);
    assert_eq!(cpu.state.A, 0x0E);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

/// ## Compare and Test Bit
///
///
#[test]
fn cpu_instr_exec_cmp_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::CMP, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::CMP, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::CMP, 0x02);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x00;
    x = cpu.instr_exec(isa::CMP, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0xFF;
    x = cpu.instr_exec(isa::CMP, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::CMP, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0x7F;
    x = cpu.instr_exec(isa::CMP, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_cpx_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::CPX, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    cpu = cpu!();
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::CPX, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | Z_FLAG);

    cpu = cpu!();
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::CPX, 0x02);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.X = 0x00;
    x = cpu.instr_exec(isa::CPX, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.X = 0xFF;
    x = cpu.instr_exec(isa::CPX, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | N_FLAG);

    cpu = cpu!();
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::CPX, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.X = 0x7F;
    x = cpu.instr_exec(isa::CPX, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_cpy_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.Y = 0x01;
    x = cpu.instr_exec(isa::CPY, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);

    cpu = cpu!();
    cpu.state.Y = 0x01;
    x = cpu.instr_exec(isa::CPY, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | Z_FLAG);

    cpu = cpu!();
    cpu.state.Y = 0x01;
    x = cpu.instr_exec(isa::CPY, 0x02);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.Y = 0x00;
    x = cpu.instr_exec(isa::CPY, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.Y = 0xFF;
    x = cpu.instr_exec(isa::CPY, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG | N_FLAG);

    cpu = cpu!();
    cpu.state.Y = 0x01;
    x = cpu.instr_exec(isa::CPY, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.Y = 0x7F;
    x = cpu.instr_exec(isa::CPY, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_bit_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::BIT, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::BIT, 0x80);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG | N_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::BIT, 0x40);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG | V_FLAG);

    cpu = cpu!();
    x = cpu.instr_exec(isa::BIT, 0xC0);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG | V_FLAG | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::BIT, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::BIT, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::BIT, 0x81);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::BIT, 0x41);
    assert_eq!(cpu.state.P, CpuFlags::none() | V_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    x = cpu.instr_exec(isa::BIT, 0xC1);
    assert_eq!(cpu.state.P, CpuFlags::none() | V_FLAG | N_FLAG);
}

/// ## Branch
///
///
#[test]
fn cpu_instr_do_branch_bcc_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCC, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCC, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCC, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCC, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCC, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    x = cpu.instr_do_branch(isa::BCC, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCC, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCC, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCC, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCC, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

#[test]
fn cpu_instr_do_branch_bcs_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCS, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCS, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCS, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCS, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCS, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    cpu.state.P.insert(C_FLAG);
    x = cpu.instr_do_branch(isa::BCS, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCS, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCS, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCS, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BCS, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

#[test]
fn cpu_instr_do_branch_bne_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BNE, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BNE, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BNE, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BNE, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BNE, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    x = cpu.instr_do_branch(isa::BNE, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BNE, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BNE, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BNE, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BNE, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

#[test]
fn cpu_instr_do_branch_beq_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BEQ, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BEQ, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BEQ, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BEQ, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BEQ, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    cpu.state.P.insert(Z_FLAG);
    x = cpu.instr_do_branch(isa::BEQ, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BEQ, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BEQ, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BEQ, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BEQ, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

#[test]
fn cpu_instr_do_branch_bpl_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BPL, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BPL, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BPL, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BPL, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BPL, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    x = cpu.instr_do_branch(isa::BPL, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BPL, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BPL, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BPL, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BPL, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

#[test]
fn cpu_instr_do_branch_bmi_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BMI, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BMI, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BMI, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BMI, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BMI, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    cpu.state.P.insert(N_FLAG);
    x = cpu.instr_do_branch(isa::BMI, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BMI, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BMI, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BMI, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BMI, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

#[test]
fn cpu_instr_do_branch_bvc_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVC, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVC, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVC, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVC, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVC, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    x = cpu.instr_do_branch(isa::BVC, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVC, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVC, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVC, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVC, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

#[test]
fn cpu_instr_do_branch_bvs_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVS, 0x00);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVS, 0x01);
    assert_eq!(cpu.state.PC, 0x8001);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVS, 0xFF);
    assert_eq!(cpu.state.PC, 0x7FFF);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVS, 0x7F);
    assert_eq!(cpu.state.PC, 0x807F);
    assert_eq!(x, 1);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVS, 0x80);
    assert_eq!(cpu.state.PC, 0x7F80);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x80FF;
    cpu.state.P.insert(V_FLAG);
    x = cpu.instr_do_branch(isa::BVS, 0x01);
    assert_eq!(cpu.state.PC, 0x8100);
    assert_eq!(x, 2);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVS, 0x01);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVS, 0xFF);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVS, 0x7F);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);

    cpu = cpu!();
    cpu.state.PC = 0x8000;
    x = cpu.instr_do_branch(isa::BVS, 0x80);
    assert_eq!(cpu.state.PC, 0x8000);
    assert_eq!(x, 0);
}

/// ## Transfer
///
///
#[test]
fn cpu_instr_exec_tax_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0x00;
    cpu.state.X = 0xAA;
    x = cpu.instr_exec(isa::TAX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    cpu.state.X = 0xAA;
    x = cpu.instr_exec(isa::TAX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0xFF;
    cpu.state.X = 0xAA;
    x = cpu.instr_exec(isa::TAX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_txa_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.X = 0x00;
    x = cpu.instr_exec(isa::TXA, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::TXA, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.X = 0xFF;
    x = cpu.instr_exec(isa::TXA, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_tay_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0x00;
    cpu.state.Y = 0xAA;
    x = cpu.instr_exec(isa::TAY, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0x01;
    cpu.state.Y = 0xAA;
    x = cpu.instr_exec(isa::TAY, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0xFF;
    cpu.state.Y = 0xAA;
    x = cpu.instr_exec(isa::TAY, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.Y, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_tya_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.Y = 0x00;
    x = cpu.instr_exec(isa::TYA, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.Y = 0x01;
    x = cpu.instr_exec(isa::TYA, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.Y = 0xFF;
    x = cpu.instr_exec(isa::TYA, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_tsx_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.S = 0x00;
    cpu.state.X = 0xAA;
    x = cpu.instr_exec(isa::TSX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.S = 0x01;
    cpu.state.X = 0xAA;
    x = cpu.instr_exec(isa::TSX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.S = 0xFF;
    cpu.state.X = 0xAA;
    x = cpu.instr_exec(isa::TSX, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.X, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_txs_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.S = 0xAA;
    cpu.state.X = 0x00;
    x = cpu.instr_exec(isa::TXS, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.S, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.S = 0xAA;
    cpu.state.X = 0x01;
    x = cpu.instr_exec(isa::TXS, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.S, 0x01);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.S = 0xAA;
    cpu.state.X = 0xFF;
    x = cpu.instr_exec(isa::TXS, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.S, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

/// ## Stack
///
///
#[test]
fn cpu_push_pop_test() {
    let mut cpu;

    cpu = cpu!();
    cpu.push(0xAA);
    assert_eq!(cpu.state.S, 0xFE);
    assert_eq!(cpu.ram[0x01FF], 0xAA);
    cpu.push(0xBB);
    assert_eq!(cpu.state.S, 0xFD);
    assert_eq!(cpu.ram[0x01FE], 0xBB);
    
    assert_eq!(cpu.pop(), 0xBB);
    assert_eq!(cpu.state.S, 0xFE);
    assert_eq!(cpu.pop(), 0xAA);
    assert_eq!(cpu.state.S, 0xFF);

    cpu = cpu!();
    cpu.state.S = 0x00;
    cpu.push(0xAA);
    assert_eq!(cpu.state.S, 0xFF);
    assert_eq!(cpu.ram[0x0100], 0xAA);
    cpu.push(0xBB);
    assert_eq!(cpu.state.S, 0xFE);
    assert_eq!(cpu.ram[0x01FF], 0xBB);

    assert_eq!(cpu.pop(), 0xBB);
    assert_eq!(cpu.state.S, 0xFF);
    assert_eq!(cpu.pop(), 0xAA);
    assert_eq!(cpu.state.S, 0x00);
}

#[test]
fn cpu_push_pop_addr_test() {
    let mut cpu;

    cpu = cpu!();
    cpu.push_addr(0xAABB);
    assert_eq!(cpu.state.S, 0xFD);
    assert_eq!(cpu.ram[0x01FF], 0xAA);
    assert_eq!(cpu.ram[0x01FE], 0xBB);
    cpu.push_addr(0xCCDD);
    assert_eq!(cpu.state.S, 0xFB);
    assert_eq!(cpu.ram[0x01FD], 0xCC);
    assert_eq!(cpu.ram[0x01FC], 0xDD);

    assert_eq!(cpu.pop_addr(), 0xCCDD);
    assert_eq!(cpu.state.S, 0xFD);
    assert_eq!(cpu.pop_addr(), 0xAABB);
    assert_eq!(cpu.state.S, 0xFF);

    cpu = cpu!();
    cpu.state.S = 0x00;
    cpu.push_addr(0xAABB);
    assert_eq!(cpu.state.S, 0xFE);
    assert_eq!(cpu.ram[0x0100], 0xAA);
    assert_eq!(cpu.ram[0x01FF], 0xBB);

    assert_eq!(cpu.pop_addr(), 0xAABB);
    assert_eq!(cpu.state.S, 0x00);
}

#[test]
fn cpu_instr_exec_pha_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0xAA;
    x = cpu.instr_exec(isa::PHA, 0x00);
    assert_eq!(cpu.state.S, 0xFE);
    assert_eq!(cpu.ram[0x01FF], 0xAA);
}

#[test]
fn cpu_instr_exec_pla_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.S = 0xFE;
    cpu.ram[0x01FF] = 0x00;
    x = cpu.instr_exec(isa::PLA, 0x00);
    assert_eq!(cpu.state.A, 0x00);
    assert_eq!(cpu.state.S, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | Z_FLAG);

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.S = 0xFE;
    cpu.ram[0x01FF] = 0x01;
    x = cpu.instr_exec(isa::PLA, 0x00);
    assert_eq!(cpu.state.A, 0x01);
    assert_eq!(cpu.state.S, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none());

    cpu = cpu!();
    cpu.state.A = 0xAA;
    cpu.state.S = 0xFE;
    cpu.ram[0x01FF] = 0xFF;
    x = cpu.instr_exec(isa::PLA, 0x00);
    assert_eq!(cpu.state.A, 0xFF);
    assert_eq!(cpu.state.S, 0xFF);
    assert_eq!(cpu.state.P, CpuFlags::none() | N_FLAG);
}

#[test]
fn cpu_instr_exec_php_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.P.bits = 0xAA;
    x = cpu.instr_exec(isa::PHP, 0x00);
    assert_eq!(cpu.state.S, 0xFE);
    assert_eq!(cpu.ram[0x01FF], 0xAA);
}

#[test]
fn cpu_instr_exec_plp_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.P.bits = 0xAA;
    cpu.state.S = 0xFE;
    cpu.ram[0x01FF] = 0x00;
    x = cpu.instr_exec(isa::PLP, 0x00);
    assert_eq!(cpu.state.P.bits, 0x00);
    assert_eq!(cpu.state.S, 0xFF);

    cpu = cpu!();
    cpu.state.P.bits = 0xAA;
    cpu.state.S = 0xFE;
    cpu.ram[0x01FF] = 0xFF;
    x = cpu.instr_exec(isa::PLP, 0x00);
    assert_eq!(cpu.state.P.bits, 0xFF);
    assert_eq!(cpu.state.S, 0xFF);

    cpu = cpu!();
    cpu.state.P.bits = 0xAA;
    cpu.state.S = 0xFE;
    cpu.ram[0x01FF] = 0x01;
    x = cpu.instr_exec(isa::PLP, 0x00);
    assert_eq!(cpu.state.P.bits, 0x01);
    assert_eq!(cpu.state.S, 0xFF);
}

/// ## Set and Clear
/// 
///
#[test]
fn cpu_instr_exec_jmp_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0x0000] = 0x4C;
    prg_rom_bank[0x0001] = 0xAA;
    prg_rom_bank[0x0002] = 0xBB;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!()));
    cpu.state.PC = 0x8000;
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0xBBAA);

    let mut prg_rom_bank_0 = prg_rom_bank!(0xC5);
    prg_rom_bank_0[0x0000] = 0x6C;
    prg_rom_bank_0[0x0001] = 0x01;
    prg_rom_bank_0[0x0002] = 0xC0;

    let mut prg_rom_bank_1 = prg_rom_bank!(0xC5);
    prg_rom_bank_1[0x0001] = 0xAA;
    prg_rom_bank_1[0x0002] = 0xBB;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank_0, prg_rom_bank_1));
    cpu.state.PC = 0x8000;
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0xBBAA);
}

#[test]
fn cpu_instr_exec_jsr_test() {
    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0x0000] = 0x20;
    prg_rom_bank[0x0001] = 0xAA;
    prg_rom_bank[0x0002] = 0xBB;

    let mut cpu = cpu!(prg_rom!(prg_rom_bank, prg_rom_bank!()));
    cpu.state.PC = 0x8000;
    cpu.instr_run();
    assert_eq!(cpu.state.PC, 0xBBAA);
    assert_eq!(cpu.ram[0x01FF], 0x80);
    assert_eq!(cpu.ram[0x01FE], 0x02);
}

#[test]
fn cpu_instr_exec_rts_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.ram[0x01FF] = 0xAA;
    cpu.ram[0x01FE] = 0xBB;
    cpu.state.S = 0xFD;
    x = cpu.instr_exec(isa::RTS, 0x00);
    assert_eq!(cpu.state.PC, 0xAABC);
    assert_eq!(cpu.state.S, 0xFF);
}

#[test]
fn cpu_instr_exec_rti_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.ram[0x01FF] = 0xAA;
    cpu.ram[0x01FE] = 0xBB;
    cpu.ram[0x01FD] = 0xCC;
    cpu.state.S = 0xFC;
    x = cpu.instr_exec(isa::RTI, 0x00);
    assert_eq!(cpu.state.P.bits, 0xCC);
    assert_eq!(cpu.state.PC, 0xAABB);
    assert_eq!(cpu.state.S, 0xFF);
}

/// ## Set and Clear
///
///
#[test]
fn cpu_instr_exec_sec_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::SEC, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);
}

#[test]
fn cpu_instr_exec_sed_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::SED, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_sei_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::SEI, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none() | I_FLAG);
}

#[test]
fn cpu_instr_exec_clc_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.P.insert(C_FLAG);
    assert_eq!(cpu.state.P, CpuFlags::none() | C_FLAG);
    x = cpu.instr_exec(isa::CLC, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_cld_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::CLD, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_cli_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.P.insert(I_FLAG);
    assert_eq!(cpu.state.P, CpuFlags::none() | I_FLAG);
    x = cpu.instr_exec(isa::CLI, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_clv_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    cpu.state.P.insert(V_FLAG);
    assert_eq!(cpu.state.P, CpuFlags::none() | V_FLAG);
    x = cpu.instr_exec(isa::CLV, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

/// ## Miscellaneous
///
///
#[test]
fn cpu_instr_exec_nop_test() {
    let mut cpu;
    let mut x;

    cpu = cpu!();
    x = cpu.instr_exec(isa::NOP, 0x00);
    assert_eq!(x, 0x00);
    assert_eq!(cpu.state.P, CpuFlags::none());
}

#[test]
fn cpu_instr_exec_brk_test() {
    let mut cpu;
    let mut x;

    let mut prg_rom_bank = prg_rom_bank!(0xC5);
    prg_rom_bank[0x3FFE] = 0xAA;
    prg_rom_bank[0x3FFF] = 0xBB;

    cpu = cpu!(prg_rom!(prg_rom_bank!(0xC5), prg_rom_bank));
    cpu.state.PC = 0x8000;
    x = cpu.instr_exec(isa::BRK, 0x00);
    assert_eq!(cpu.ram[0x01FF], 0x80);
    assert_eq!(cpu.ram[0x01FE], 0x01);
    assert_eq!(cpu.ram[0x01FD], (CpuFlags::none() | B_FLAG).bits);
    assert_eq!(cpu.state.P, CpuFlags::none() | I_FLAG);
    assert_eq!(cpu.state.PC, 0xBBAA);
}
