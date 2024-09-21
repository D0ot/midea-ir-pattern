// Generate IR Pattern for Android IR API
// I use it in Termux-API
// The leaded Media R05D Design https://wenku.baidu.com/view/c46594141ed9ad51f01df2c3.html

#[derive(PartialEq, Debug)]
enum IrOutputState {
    IrHigh, // means sender otuput level is high, so the receiver is low.
    IrLow,
}

#[derive(Debug)]
struct IrGenState {
    output: IrOutputState,
    duration: u64,
}

impl IrGenState {
    fn default() -> IrGenState {
        IrGenState {
            output: IrOutputState::IrHigh,
            duration: 0,
        }
    }

    fn ir_gen(&self) {
        print!("{},", self.duration as u32);
    }

    fn ir_high(&mut self, us: u64) {
        self.ir_low_or_high(us, IrOutputState::IrHigh);
    }

    fn ir_low(&mut self, us: u64) {
        self.ir_low_or_high(us, IrOutputState::IrLow);
    }

    fn ir_end(&self) {
        print!("{}", self.duration as u32);
    }

    fn ir_low_or_high(&mut self, us: u64, output: IrOutputState) {
        if self.output == output {
            self.duration += us;
        } else {
            self.ir_gen();
            self.duration = us;
            self.output = output
        }
    }
}

fn code_lead(state: &mut IrGenState) {
    state.ir_high(4400);
    state.ir_low(4400);
}

fn code_one(state: &mut IrGenState) {
    state.ir_high(540);
    state.ir_low(1620);
}

fn code_zero(state: &mut IrGenState) {
    state.ir_high(540);
    state.ir_low(540);
}

fn code_byte(state: &mut IrGenState, byte: u8) {
    for x in (0..8).rev() {
        if (byte >> (x)) & 1 == 0 {
            code_zero(state);
        } else {
            code_one(state);
        }
    }
}

fn code_stop(state: &mut IrGenState) {
    state.ir_high(540);
    state.ir_low(5220);
}

fn code_pair(state: &mut IrGenState, byte: u8) {
    code_byte(state, byte);
    code_byte(state, !byte);
}

fn midea_gen_abc(state: &mut IrGenState, a: u8, b: u8, c: u8, stop: bool) {
    code_lead(state);
    code_pair(state, a);
    code_pair(state, b);
    code_pair(state, c);
    code_stop(state);

    code_lead(state);
    code_pair(state, a);
    code_pair(state, b);
    code_pair(state, c);

    if stop { state.ir_end(); }
}

fn midea_gen_off(state: &mut IrGenState) {
    midea_gen_abc(state, 0xB2, 0b0111_1011, 0b1110_0000, false);

    code_stop(state);
    code_lead(state);
    code_pair(state, 0xB2);
    code_pair(state, 0);
    code_pair(state, 0);

    state.ir_end();
}

enum MideaMode {
    Auto,
    Cool,
    Dry,
    Fan,
    Warm,
}

enum MideaSpeed {
    Auto,
    High,
    Low,
    Middle,
}

struct MideaTemp {
    temp: u8
}

impl MideaTemp {
    fn new(temp: u8) -> MideaTemp {
        if temp < 17 || temp > 30 {
            panic!("MideaTemp Temp out of range.");
        }
        MideaTemp { temp }
    }
}

fn bin_to_grey(bin: u8) -> u8 {
    (bin >> 1) ^ bin
}

fn midea_ac_pattern(on: bool, mode: MideaMode, speed: MideaSpeed, temp: MideaTemp) {
    let mut state = IrGenState::default();

    if !on {
        midea_gen_off(&mut state);
        return;
    }

    let mut b = 0b11111;
    let mut c = 0;

    //b |= (on as u8) << 3;

    match speed {
        MideaSpeed::Auto => b |= 0b101_00000,
        MideaSpeed::Low => b |= 0b100_00000,
        MideaSpeed::Middle => b |= 0b010_00000,
        MideaSpeed::High => b |= 0b001_00000,
    }

    match mode {
        MideaMode::Auto => c |= 0b1000,
        MideaMode::Cool => c |= 0b0000,
        MideaMode::Dry | MideaMode::Fan => c |= 0b0100,
        MideaMode::Warm => c |= 0b1100,
    }

    c |= (bin_to_grey(temp.temp - 17)) << 4;

    midea_gen_abc(&mut state, 0xB2, b, c, true);
}

fn main() {
    let temp_str = std::env::args().nth(2).expect("2nd positional arg: Temperature");
    let temp_raw: u8 = temp_str.trim().parse().expect("Temperature is not a positive integer");

    let ac_on_str = std::env::args().nth(1).expect("1st positional arg: ON/on/On/Off/OFF/off/0/1");
    let ac_on;

    match ac_on_str.as_str() {
        "ON" | "on" | "On" | "1" => ac_on = true,
        "OFF" | "off" | "Off" | "0" => ac_on = false,
        _ => panic!(),
    }

    midea_ac_pattern(ac_on ,MideaMode::Cool, MideaSpeed::Auto, MideaTemp::new(temp_raw));

}
