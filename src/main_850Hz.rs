#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![allow(unused)]


use cortex_m::delay;
use defmt_rtt as _;
use embassy_executor::Spawner;
use libm;
mod utils;
use core::array;
use libm::exp;
//use libm::log;
//use libm::fabs;
//use numeric_sort::sort;
//use std::fs::File;
//use std::io::{self, Write};

use u5_lib::{
    clock::{self, delay_ms, delay_s, delay_tick, delay_us, hclk_request}, exti, gpio::{self, GpioPort, TIM3_CH1_PB4, TIM3_CH4_PB1}, hal::Pin, low_power::{Executor, no_deep_sleep_request}, tim::{Config, TIM1, TIM3}, *
};

//use tim::{Config, TIM1};

// fn i2c_init() -> (I2c, I2c) {
//     let i2c_config_plus = i2c::I2cConfig::new(1, 100_000, gpio::I2C1_SCL_PB6, gpio::I2C1_SDA_PB3);
//     let i2c_plus = I2c::new(i2c_config_plus).unwrap();
//     let i2c_config_minus = i2c::I2cConfig::new(2, 100_000, gpio::I2C2_SCL_PB10, gpio::I2C2_SDA_PB14);
//     let i2c_minus = I2c::new(i2c_config_minus).unwrap();
//     (i2c_plus, i2c_minus)
// }
fn switch_led_setup() -> ( gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort,
    gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort,
    gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort,
    gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort,
    gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort,
    gpio::GpioPort, gpio::GpioPort, gpio::GpioPort, gpio::GpioPort){

    let red: gpio::GpioPort = gpio::PB8;
    let green: gpio::GpioPort = gpio::PB9;
    let blue: gpio::GpioPort = gpio::PC1;
    let yellow: gpio::GpioPort = gpio::PA3;

    let s0: gpio::GpioPort = gpio::PC11;
    let s1: gpio::GpioPort = gpio::PC12;
    let s2: gpio::GpioPort = gpio::PB4;
    let s3: gpio::GpioPort = gpio::PB5;
    let s4: gpio::GpioPort = gpio::PB13;
    let s5: gpio::GpioPort = gpio::PB12;
    let s6: gpio::GpioPort = gpio::PB2;
    let s7: gpio::GpioPort = gpio::PB1;
   
    let reset1: gpio::GpioPort = gpio::PC2;
    let reset2: gpio::GpioPort = gpio::PC5;

    let set1: gpio::GpioPort = gpio::PB7;
    let set2: gpio::GpioPort = gpio::PB0;

    let channel_sel0: gpio::GpioPort = gpio::PA10;
    let channel_sel1: gpio::GpioPort = gpio::PA15;
    let channel_sel2: gpio::GpioPort = gpio::PA9;
    let channel_sel3: gpio::GpioPort = gpio::PA8;
    
    let polarity_sel1: gpio::GpioPort = gpio::PC10;
    let polarity_sel2: gpio::GpioPort = gpio::PC9;

    let ADC_sel1: gpio::GpioPort = gpio::PC8;
    let ADC_sel2: gpio::GpioPort = gpio::PC7;
    let ADC_sel3: gpio::GpioPort = gpio::PC6;
    
    let MUX_EN: gpio::GpioPort = gpio::PB15;

    let D0: gpio::GpioPort = gpio::PC3;
    let D1: gpio::GpioPort = gpio::PA0;
    let D2: gpio::GpioPort = gpio::PA1;
    let D3: gpio::GpioPort = gpio::PA2;
    let D4: gpio::GpioPort = gpio::PC4;
    let D5: gpio::GpioPort = gpio::PA7;
    let D6: gpio::GpioPort = gpio::PA6;
    let D7: gpio::GpioPort = gpio::PA5;

    let SCL1: gpio::GpioPort = gpio::PB6;
    let SDA1: gpio::GpioPort = gpio::PB3;
    let SCL2: gpio::GpioPort = gpio::PB10;
    let SDA2: gpio::GpioPort = gpio::PB14;
    
    red.setup();
    green.setup();
    blue.setup();
    yellow.setup();
    s0.setup();
    s1.setup();
    s2.setup();
    s3.setup();
    s4.setup();
    s5.setup();
    s6.setup();
    s7.setup();
    reset1.setup();
    reset2.setup();
    set1.setup();
    set2.setup();
    channel_sel0.setup();
    channel_sel1.setup();
    channel_sel2.setup();
    channel_sel3.setup();
    polarity_sel1.setup();
    polarity_sel2.setup();
    ADC_sel1.setup();
    ADC_sel2.setup();
    ADC_sel3.setup();
    MUX_EN.setup();
    D0.setup();
    D1.setup();
    D2.setup();
    D3.setup();
    D4.setup();
    D5.setup();
    D6.setup();
    D7.setup();
    SCL1.setup();
    SDA1.setup();
    SCL2.setup();
    SDA2.setup();
    (red, green, blue, yellow, s0, s1, s2, s3, s4, s5, s6, s7, reset1, reset2, set1, set2, 
    channel_sel0, channel_sel1, channel_sel2, channel_sel3, polarity_sel1, polarity_sel2, ADC_sel1,
    ADC_sel2, ADC_sel3, MUX_EN, D0, D1, D2, D3, D4, D5, D6, D7, SCL1, SDA1, SCL2, SDA2)

}


struct Point {
    x: f64,
    y: f64,
}

#[embassy_executor::task]
async fn async_main(spawner: Spawner) {
    // be careful, if the dbg is not enabled, but using deep sleep. This framework will not able to connect to chip.
    // stm32cube programmer, stmcubeide can be used to program the chip, then this framework can be used to debug.
    clock::init_clock(true, clock::ClockFreqs::KernelFreq16Mhz);
    unsafe {
        no_deep_sleep_request();
    }

    let mut tim3_config = Config::default();
    tim3_config.prescaler = 10 - 1;
    let _ = TIM3.init(tim3_config);
   // let _ = TIM3.init(Config::default());
    
    defmt::info!("setup finished!");

    let (red, green, blue, yellow, s0, s1, s2, s3, s4, s5, s6, s7, 
    reset1, reset2, set1, set2, channel_sel0, channel_sel1, 
    channel_sel2, channel_sel3, polarity_sel1, polarity_sel2, ADC_sel1,
    ADC_sel2, ADC_sel3, MUX_EN, D0, D1, D2, D3, D4, D5, D6, D7, SCL1, SDA1, SCL2, SDA2) = switch_led_setup();
    
    // These chip includes 8 stimulator channels, devided to 2 groups. Each channel has 2 DACs and 4 switches. 
    //The circuit diagram could refer to the paper. 
    s0.set_low(); //switch to charge capacitor
    s1.set_low(); //switch connect to pos DAC
    s2.set_low(); //switch connect to neg DAC
    s3.set_low(); // switch connect pos DAC & charge capacior to electrode
    s4.set_low(); //switches in group2. s4 to s7 are corresponding to s0 to s3.
    s5.set_low();  
    s6.set_low(); 
    s7.set_low(); 
    reset1.set_high(); //reset1 is common for group1. Active low. At rising edge of SCL, all bit are set to 0
    reset2.set_high(); //reset2 is common for group2
    set1.set_high(); // Active low. At rising edge of SCL, all bit are set to 1
    set2.set_high();
    channel_sel0.set_low(); //data send selection bit. 4 channels in total. channel_sel0&1 decide the group1 4 channels
    channel_sel1.set_low();
    channel_sel2.set_low(); // channel_sel2&3 decide the group2  4 channels. 
    channel_sel3.set_low();
    polarity_sel1.set_low(); //decide pos DAC or neg DAC to be coded. For group 1
    polarity_sel2.set_low(); //decide pos DAC or neg DAC to be coded. For group 2
    ADC_sel1.set_low(); // There is only 1 ADC in STM32. Decide which channel want to exam. 
    ADC_sel2.set_low();
    ADC_sel3.set_low();
    MUX_EN.set_low(); // MUX output enable signal. Acive low.
    // D0.set_high(); //D0 - D7 are input bits for test DAC. 
    // D1.set_high();
    // D2.set_high();
    // D3.set_high();
    // D4.set_high();
    // D5.set_high();
    // D6.set_high();
    // D7.set_high();
    D0.set_low();
    D1.set_low();
    D2.set_low();
    D3.set_low();
    D4.set_low();
    D5.set_low();
    D6.set_low();
    D7.set_low();
    SCL1.set_low(); //SCL signal for neg DAC
    SDA1.set_low(); //SDA signal for neg DAC
    SCL2.set_low(); //SCL signal for pos DAC
    SDA2.set_low(); //SCL signal for pos DAC
    defmt::info!("Init finished!");
  

    ////////////////////////////////////////////ADC initial
    let tmp_adc = adc::ADC1;
    tmp_adc.init();
    let adc_pin = gpio::ADC1_IN1_PC0;
    adc_pin.setup();
    defmt::info!("ADC init!");
    delay_ms(100);
    let vref_data = tmp_adc.start_conversion_sw(0);
    delay_ms(100);
    let vref_data = tmp_adc.start_conversion_sw(0);
    let vref_raw =  tmp_adc.get_vref_int_raw();
    let vref = 3.0 * vref_raw as f64 / vref_data as f64;
    let vref = vref / 16384.0;
    defmt::info!("Vref init! {}", vref);
    delay_s(1);
    ////////////////////////////////////////////
   
    let mut Rs = 200.0; //unit is ohm
    let mut Rp = 1000.0; //unit is ohm
    let mut Cp = 1.0; //unit is uF
    let mut fc = 10.0;
    let mut V0 = 0.0;
    let mut I0 = 0.0;

    let mut Rs2 = 200.0; //unit is ohm
    let mut Rp2 = 1000.0; //unit is ohm
    let mut Cp2 = 1.0; //unit is uF
    let mut fc2 = 10.0;
    let mut V02 = 0.0;
    let mut I02 = 0.0;


    ///////////DAC 0///////////////////
    let DAC_data_pos0: f32 = 0.4;  // Channel 1 positive current amplitude. No more than 1.0mA.
    //let DAC_data_neg0: f32 = -1.0; //  Channel 1 negative current amplitude. No more than 1.0mA.
    let DAC_data_neg0: f32 = -0.8;
    let DAC_max_pos0:f32 = 1.06; //0.83
    let DAC_max_neg0:f32 = -1.22; //-0.91
    let mut Ipos_hex = utils::cur_coding(DAC_data_pos0, DAC_max_pos0, DAC_max_neg0);
    let mut Ineg_hex = utils::cur_coding(DAC_data_neg0, DAC_max_pos0, DAC_max_neg0);
    defmt::info!("DAC0 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos_hex, Ineg_hex);
    reset1.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL1.set_high(); //first rising edge
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_low();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ineg_hex = Ineg_hex >> 1;
    }

    delay_ms(2);
    polarity_sel1.set_high();//now coding positive DAC
    reset1.set_low();
    delay_ms(1);
    SCL1.set_high(); 
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_high();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ipos_hex = Ipos_hex >> 1;
    }

    /////////////////DAC 1 ////////////////////

    let DAC_data_pos1 = 0.4;
    let DAC_data_neg1 = -0.8;
    let DAC_max_pos1 = 0.92; //1.04
    let DAC_max_neg1:f32 = -0.92; //-1.04
    Ipos_hex = utils::cur_coding(DAC_data_pos1, DAC_max_pos1, DAC_max_neg1);
    Ineg_hex = utils::cur_coding(DAC_data_neg1, DAC_max_pos1, DAC_max_neg1);

    defmt::info!("DAC1 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos_hex, Ineg_hex);
    delay_ms(2);
    channel_sel0.set_high();
    polarity_sel1.set_low();
    reset1.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL1.set_high(); //first rising edge
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_low();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ineg_hex = Ineg_hex >> 1;
    }

    delay_ms(2);
    polarity_sel1.set_high();//now coding positive DAC
    reset1.set_low();
    delay_ms(1);
    SCL1.set_high(); 
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_high();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ipos_hex = Ipos_hex >> 1;
    }

    ///////////// DAC 2 //////////////////////////
    let DAC_data_pos2 = 0.4;
    let DAC_data_neg2 = -0.8;
    let DAC_max_pos2 = 0.86; //0.48
    let DAC_max_neg2:f32 = -0.86; //-0.51
    Ipos_hex = utils::cur_coding(DAC_data_pos2, DAC_max_pos2, DAC_max_neg2);
    Ineg_hex = utils::cur_coding(DAC_data_neg2, DAC_max_pos2, DAC_max_neg2);
    defmt::info!("DAC 2 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos_hex, Ineg_hex);
    channel_sel0.set_low();
    channel_sel1.set_high();
    reset1.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL1.set_high(); //first rising edge
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_low();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ineg_hex = Ineg_hex >> 1;
    }

    delay_ms(2);
    polarity_sel1.set_high();//now coding positive DAC
    reset1.set_low();
    delay_ms(1);
    SCL1.set_high(); 
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_high();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ipos_hex = Ipos_hex >> 1;
    }

    ///////////////DAC 3 /////////////////
    let DAC_data_pos3 = 0.4;
    let DAC_data_neg3 = -0.8;
    let DAC_max_pos3 = 0.86; //
    let DAC_max_neg3: f32 = -0.98; //-1.06
    Ipos_hex = utils::cur_coding(DAC_data_pos3, DAC_max_pos3, DAC_max_neg3);
    Ineg_hex = utils::cur_coding(DAC_data_neg3, DAC_max_pos3, DAC_max_neg3);
    defmt::info!("DAC3 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos_hex, Ineg_hex);
    channel_sel0.set_high();
    reset1.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL1.set_high(); //first rising edge
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_low();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ineg_hex = Ineg_hex >> 1;
    }

    delay_ms(2);
    polarity_sel1.set_high();//now coding positive DAC
    reset1.set_low();
    delay_ms(1);
    SCL1.set_high(); 
    delay_ms(1);
    SCL1.set_low();
    reset1.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos_hex % 2 == 0){
            SDA1.set_low();
            delay_us(500);
        } else {
            SDA1.set_high();
            delay_us(500);
        }
        // SDA1.set_high();
        // delay_us(500);
        SCL1.set_high();
        delay_us(500);
        SCL1.set_low();
        delay_us(500);
        Ipos_hex = Ipos_hex >> 1;
    }

    ///////////////////////////////////////////////////////////////////
    /////////////////////////// group 2 ///////////////////////////////
    ///////////////////////////////////////////////////////////////////

    ///////////DAC 0///////////////////
    let DAC2_data_pos0: f32 = 0.4; //
    //let DAC2_data_neg0: f32 = -0.8;
    let DAC2_data_neg0: f32 = -0.8;
    let DAC2_max_pos0:f32 = 0.8; //0.83
    let DAC2_max_neg0:f32 = -1.08; //-0.91
    let mut Ipos2_hex = utils::cur_coding(DAC2_data_pos0, DAC2_max_pos0, DAC2_max_neg0);
    let mut Ineg2_hex = utils::cur_coding(DAC2_data_neg0, DAC2_max_pos0, DAC2_max_neg0);
    defmt::info!("DAC0 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos2_hex, Ineg2_hex);
    reset2.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL2.set_high(); //first rising edge
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ineg2_hex = Ineg2_hex >> 1;
    }

    delay_ms(2);
    polarity_sel2.set_high();//now coding positive DAC
    reset2.set_low();
    delay_ms(1);
    SCL2.set_high(); 
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ipos2_hex = Ipos2_hex >> 1;
    }

    /////////////////DAC 1 ////////////////////

    let DAC2_data_pos1 = 0.4;
    let DAC2_data_neg1 = -0.8;
    let DAC2_max_pos1 = 0.75; //0.48
    let DAC2_max_neg1:f32 = -0.8; //-0.55
    Ipos2_hex = utils::cur_coding(DAC2_data_pos1, DAC2_max_pos1, DAC2_max_neg1);
    Ineg2_hex = utils::cur_coding(DAC2_data_neg1, DAC2_max_pos1, DAC2_max_neg1);

    defmt::info!("DAC1 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos2_hex, Ineg2_hex);
    delay_ms(2);
    channel_sel2.set_high();
    polarity_sel2.set_low();
    reset2.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL2.set_high(); //first rising edge
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ineg2_hex = Ineg2_hex >> 1;
    }

    delay_ms(2);
    polarity_sel2.set_high();//now coding positive DAC
    reset2.set_low();
    delay_ms(1);
    SCL2.set_high(); 
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        // SDA1.set_high();
        // delay_us(500);
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ipos2_hex = Ipos2_hex >> 1;
    }

    ///////////// DAC 2 //////////////////////////
    let DAC2_data_pos2 = 0.4;
    let DAC2_data_neg2 = -0.8;
    let DAC2_max_pos2 = 0.73; //0.48
    let DAC2_max_neg2:f32 = -0.8; //-0.51
    Ipos2_hex = utils::cur_coding(DAC2_data_pos2, DAC2_max_pos2, DAC2_max_neg2);
    Ineg2_hex = utils::cur_coding(DAC2_data_neg2, DAC2_max_pos2, DAC2_max_neg2);
    defmt::info!("DAC 2 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos2_hex, Ineg2_hex);
    channel_sel2.set_low();
    channel_sel3.set_high();
    reset2.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL2.set_high(); //first rising edge
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        // SDA1.set_low();
        // delay_us(500);
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ineg2_hex = Ineg2_hex >> 1;
    }

    delay_ms(2);
    polarity_sel2.set_high();//now coding positive DAC
    reset2.set_low();
    delay_ms(1);
    SCL2.set_high(); 
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        // SDA1.set_high();
        // delay_us(500);
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ipos2_hex = Ipos2_hex >> 1;
    }

    ///////////////DAC 3 /////////////////
    let DAC2_data_pos3 = 0.4;
    let DAC2_data_neg3 = -0.8;
    let DAC2_max_pos3 = 0.7; //0.46 
    let DAC2_max_neg3: f32 = -0.92; //-0.35
    Ipos2_hex = utils::cur_coding(DAC2_data_pos3, DAC2_max_pos3, DAC2_max_neg3);
    Ineg2_hex = utils::cur_coding(DAC2_data_neg3, DAC2_max_pos3, DAC2_max_neg3);
    defmt::info!("DAC3 Ipos hex is 0x{:X}. Ineg_hex is 0x{:X}", Ipos2_hex, Ineg2_hex);
    channel_sel2.set_high();
    reset2.set_low(); //Reset was high all time. Reset to low for at least 1 SCL rising edge. 
    delay_ms(1);
    SCL2.set_high(); //first rising edge
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500); //8 bit data in total. Every clock rising edge, the data was sent.
    for i in (0..8) {
        if(Ineg2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        // SDA1.set_low();
        // delay_us(500);
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ineg2_hex = Ineg2_hex >> 1;
    }

    delay_ms(2);
    polarity_sel2.set_high();//now coding positive DAC
    reset2.set_low();
    delay_ms(1);
    SCL2.set_high(); 
    delay_ms(1);
    SCL2.set_low();
    reset2.set_high();
    delay_us(500);
    for i in (0..8) {
        if(Ipos2_hex % 2 == 0){
            SDA2.set_low();
            delay_us(500);
        } else {
            SDA2.set_high();
            delay_us(500);
        }
        // SDA1.set_high();
        // delay_us(500);
        SCL2.set_high();
        delay_us(500);
        SCL2.set_low();
        delay_us(500);
        Ipos2_hex = Ipos2_hex >> 1;
    }


    defmt::info!("Data transfer finished!");
    delay_s(3);

    yellow.toggle();
    //red.toggle();
    green.toggle();
    //blue.toggle();
    
    let mut adc_sum_min: f64 = 5.0;
    let mut adc_sum_max: f64 = 0.0;
    let Ineg_f64:f64 = DAC_data_neg0 as f64;
    let Ineg2_f64:f64 = DAC2_data_neg0 as f64;
    let Ipos2_f64:f64 = DAC2_data_pos0 as f64;
    let mut current_compensation:f64 = 0.0;

    let mut array_impedance:[f64; 30] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let mut array_impedance2:[f64; 30] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let frequency_divider:[u16; 30] = [50000, 40000, 25000, 20000, 15873, 12500, 10000, 
    6400, 5000, 4000, 2500, 2000, 1587, 1250, 1000,  
    640, 500, 400, 250, 200, 159, 125, 100, 
    50, 40, 25, 20, 16, 13, 10]; 



    ////////////////////////////////////////////////////////
    let mut t1 = 100; //negative pulse time. unit is us
    let mut t2 = 200; //positive pulse time. unit is us 
    let mut t3 = 150; //charge time. unit is us
    let mut t4 = 50;
    let mut tauRC = 0.0;

    let mut t5 = 100; //negative pulse time. unit is us
    let mut t6 = 260; //positive pulse time. unit is us 
    let mut t7 = 150; //charge time. unit is us
    let mut t8 = 50;
    let mut tauRC2 = 0.0;


    let mut i = 0;
    let mut j = 0;
    let mut abnormal_counter = 0; 
    let mut counter1 = 35550; //31700
    let mut counter2 = 28; //54
    let mut minute_count = 28; //54
    let mut first_count = 0;
    let mut margin = 20.0;

    loop{
        if t3> t1 {
            t4 = t3 - t1;
        } else {
            t4 = t1 - t3;
        }

        if(t7 > t5){
            t8 = t7 - t5;
        } else {
            t8 = t5 - t7;
        }

        s3.set_high();
        s7.set_high();

        hclk_request(clock::ClockFreqs::KernelFreq160Mhz, ||{
//////////////square wave ////////////////////////////////
            s3.set_high();
            s2.set_high();
            delay_us(100);
            s2.set_low();
            delay_us(100);
            s1.set_high();
            delay_us(200);
            s1.set_low();
            delay_us(50);
////////////////////////////////////////////////////////////


/////////////////capacitor-coupled wave /////////////////
        //     s0.set_high();
        //     s1.set_high();
        //     s2.set_high();

        //     if t3 > t1 {
        //         delay_us(t1);
        //         s2.set_low();
        //         delay_us(t4);
        //         s1.set_low();
        //     }else {
        //         delay_us(t3);
        //         s1.set_low();
        //         delay_us(t4);
        //         s2.set_low();
        //     }

        //     delay_us(10);
        //     s3.set_high();
        //     //delay_tick(1);
        //     delay_us(t2);
        //     s3.set_low();
        //     delay_us(100);

            s4.set_high();
            s5.set_high();
            s6.set_high();

            if t7 > t5 {
                delay_us(t5);
                s6.set_low();
                delay_us(t8);
                s5.set_low();
            }else {
                delay_us(t7);
                s5.set_low();
                delay_us(t8);
                s6.set_low();
            }

            delay_us(10);
            s7.set_high();
            delay_us(t6);
            s7.set_low();
            delay_us(50);
             s4.set_low();
            delay_us(100);
        });
        // s0.set_low();
       
////////////////////////////////////////////////

//////////////DOC mode/////////////////////////////
        //     s2.set_high();
        //     delay_us(200);
        //     s2.set_low();
        //     delay_us(200);
        //     s5.set_high();
        //     delay_us(200);
        //     s5.set_low();
        //     delay_us(66);
        //     delay_ms(1);
        // });

////////////////////////////////////////////////////

        if counter1 >= 51000 { //32700
            s2.set_high();
            s6.set_high();
            delay_ms(1);
            s2.set_low();
            s6.set_low();
            counter1 = 0;
        } 
        else {
            counter1 = counter1 + 1;
        }
    }
}


#[cortex_m_rt::entry]
fn main() -> ! {
    Executor::take().run(|spawner| {
        spawner.spawn(async_main(spawner)).unwrap();
    });
}